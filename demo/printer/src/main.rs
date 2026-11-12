//! Demo binary: the fictional print-on-demand API.
//!
//! | Variable | Default | Meaning |
//! |---|---|---|
//! | `DEMO_PRINTER_BIND` | `127.0.0.1:8103` | listen address |
//! | `DEMO_PRINTER_TICK_MS` | `2000` | time between two steps of an order's life cycle; `0` = manual (orders move only through `POST /__test/advance`) |
//! | `DEMO_TEST_HOOKS` | unset | `1` enables `GET /__test/orders` and `POST /__test/advance` |

use std::error::Error;
use std::time::Duration;

use ebc_demo_printer::{DEFAULT_PORT, PrinterConfig, app};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let env = |name: &str| {
        std::env::var(name).ok().map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
    };

    let mut config = PrinterConfig::default();
    if let Some(value) = env("DEMO_PRINTER_TICK_MS") {
        let millis: u64 = value.parse().map_err(|_| {
            format!("DEMO_PRINTER_TICK_MS must be a number of milliseconds, got {value:?}")
        })?;
        config.tick = (millis > 0).then(|| Duration::from_millis(millis));
    }
    config.test_hooks = env("DEMO_TEST_HOOKS").is_some_and(|v| v == "1");
    // Order identifiers double as read credentials: make them unguessable per run.
    let mut salt = [0u8; 16];
    getrandom::fill(&mut salt).map_err(|e| format!("no entropy for the identifier salt: {e}"))?;
    config.id_salt = salt.iter().map(|b| format!("{b:02x}")).collect();

    let bind = env("DEMO_PRINTER_BIND").unwrap_or_else(|| format!("127.0.0.1:{DEFAULT_PORT}"));
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .map_err(|e| format!("cannot listen on {bind} (DEMO_PRINTER_BIND): {e}"))?;
    eprintln!(
        "Imprimerie du Dos Carré (démo) — http://{} — tick: {} — test hooks: {}",
        listener.local_addr()?,
        config.tick.map_or_else(|| "manual".to_owned(), |t| format!("{} ms", t.as_millis())),
        if config.test_hooks { "on" } else { "off" },
    );
    axum::serve(listener, app(config)).with_graceful_shutdown(shutdown()).await?;
    Ok(())
}

/// Resolves on Ctrl-C.
async fn shutdown() {
    // If the signal handler cannot be installed, keep serving until the process is killed.
    if tokio::signal::ctrl_c().await.is_err() {
        std::future::pending::<()>().await;
    }
}
