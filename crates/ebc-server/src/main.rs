//! `ebc-server`: reads the configuration from the environment (`docs/api.md` §6), opens
//! the data directory and serves the eBook Cadeau web service.

use std::process::ExitCode;
use std::sync::Arc;

use ebc_server::{AppState, Config, FileStore, ServerError, serve};

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt().with_writer(std::io::stderr).init();
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(%error, "ebc-server stopped");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), ServerError> {
    let config = Config::from_env()?;
    for warning in config.public_url_warnings() {
        tracing::warn!("{warning}");
    }
    let store =
        FileStore::open(&config.data_dir).map_err(|e| ServerError::Storage(e.to_string()))?;
    let bind = config.bind.clone();
    let public_url = config.public_url.clone();
    let state = AppState::new(config, Arc::new(store))?;
    state.resume_webhooks();

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(%bind, %public_url, "ebc-server is listening");
    // Header, request and body time-outs and the connection cap come from the
    // configuration (`ebc_server::serve`).
    serve(listener, state, async {
        // Without a signal handler the server simply runs until it is killed.
        if tokio::signal::ctrl_c().await.is_err() {
            std::future::pending::<()>().await;
        }
    })
    .await?;
    Ok(())
}
