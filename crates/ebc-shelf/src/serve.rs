//! Serving the router on a TCP listener with the time-outs and limits a public service
//! needs (`axum::serve` sets none: hyper has no timer by default, so a client that never
//! finishes its request headers would hold its connection for ever).
//!
//! * request line and headers must arrive within `EBC_SHELF_HEADER_TIMEOUT_SECS` (hyper's
//!   `header_read_timeout` with a Tokio timer; it also closes idle keep-alive
//!   connections);
//! * a request must produce its response within `EBC_SHELF_REQUEST_TIMEOUT_SECS`, and its body
//!   may not pause longer than `EBC_SHELF_BODY_IDLE_TIMEOUT_SECS` (both set by [`crate::app`],
//!   so in-process mounts have them too);
//! * at most `EBC_SHELF_MAX_CONNECTIONS` connections are served at once; further ones wait in
//!   the listen backlog.
//!
//! Not covered: a client that reads a *response* very slowly is only bounded by the
//! connection cap (hyper has no write time-out). Put a reverse proxy in front for that.

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::ConnectInfo;
use hyper::body::Incoming;
use hyper_util::rt::{TokioIo, TokioTimer};
use hyper_util::server::graceful::GracefulShutdown;
use hyper_util::service::TowerToHyperService;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tower::ServiceExt as _;

use crate::routes::app;
use crate::state::AppState;

/// Time open connections get to finish once shutdown has been asked for.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(30);

/// Serves [`app`]`(state)` on `listener` until `shutdown` resolves, with the header,
/// request and body time-outs and the connection cap of the configuration. This is what
/// the `ebc-shelf` binary runs; tests mount the service the same way.
///
/// Each request carries the peer address as `ConnectInfo<SocketAddr>`, as with
/// `into_make_service_with_connect_info`, so that clients are told apart.
///
/// # Errors
/// Only if the connection semaphore is closed, which does not happen; accept errors are
/// logged and retried.
pub async fn serve<F>(listener: TcpListener, state: AppState, shutdown: F) -> std::io::Result<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    let header_timeout = Duration::from_secs(state.config.header_read_timeout_secs.max(1));
    let connections =
        Arc::new(Semaphore::new(usize::try_from(state.config.max_connections.max(1)).unwrap_or(1)));
    let router = app(state);
    let graceful = GracefulShutdown::new();
    let mut shutdown = std::pin::pin!(shutdown);

    loop {
        let permit = tokio::select! {
            permit = Arc::clone(&connections).acquire_owned() => {
                permit.map_err(std::io::Error::other)?
            }
            () = &mut shutdown => break,
        };
        let (stream, peer) = tokio::select! {
            accepted = listener.accept() => match accepted {
                Ok(accepted) => accepted,
                Err(error) => {
                    if !is_connection_error(&error) {
                        // Typically out of file descriptors: back off instead of spinning.
                        tracing::warn!(%error, "accept failed");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                    continue;
                }
            },
            () = &mut shutdown => break,
        };
        let service = router.clone().map_request(move |mut request: hyper::Request<Incoming>| {
            request.extensions_mut().insert(ConnectInfo(peer));
            request
        });
        let connection = hyper::server::conn::http1::Builder::new()
            .timer(TokioTimer::new())
            .header_read_timeout(header_timeout)
            .serve_connection(TokioIo::new(stream), TowerToHyperService::new(service));
        let connection = graceful.watch(connection);
        tokio::spawn(async move {
            if let Err(error) = connection.await {
                tracing::debug!(%error, "connection closed with an error");
            }
            drop(permit);
        });
    }

    drop(listener);
    tokio::select! {
        () = graceful.shutdown() => {}
        () = tokio::time::sleep(SHUTDOWN_GRACE) => {
            tracing::warn!("connections still open after the shutdown grace period");
        }
    }
    Ok(())
}

/// Errors that concern one incoming connection, not the listener.
fn is_connection_error(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::ConnectionRefused
            | std::io::ErrorKind::ConnectionAborted
            | std::io::ErrorKind::ConnectionReset
    )
}
