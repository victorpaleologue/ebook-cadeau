//! `ebc-shelf`: reads the configuration from the environment (see the crate's README),
//! opens the data directory, serves the shelf and runs the retention purge.

use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use ebc_shelf::{AppState, Config, FileStore, ShelfError, serve};

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt().with_writer(std::io::stderr).init();
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(%error, "ebc-shelf stopped");
            ExitCode::FAILURE
        }
    }
}

/// The secret of a loopback development shelf: generated once in the data directory.
fn development_secret(config: &Config) -> Result<String, ShelfError> {
    let path = config.data_dir.join("shelf-secret");
    if let Ok(existing) = std::fs::read_to_string(&path)
        && existing.trim().len() >= 16
    {
        return Ok(existing.trim().to_owned());
    }
    let mut bytes = [0u8; 32];
    ebc_shelf::Entropy::fill(&ebc_shelf::OsEntropy, &mut bytes)
        .map_err(|e| ShelfError::Config(e.to_string()))?;
    let secret = ebc_shelf::crypto::hex(&bytes);
    std::fs::write(&path, &secret).map_err(|e| ShelfError::Storage(e.to_string()))?;
    tracing::warn!("EBC_SHELF_SECRET unset: generated one in the data directory (loopback only)");
    Ok(secret)
}

async fn run() -> Result<(), ShelfError> {
    let mut config = Config::from_env().map_err(|e| ShelfError::Config(e.to_string()))?;
    let store =
        FileStore::open(&config.data_dir).map_err(|e| ShelfError::Storage(e.to_string()))?;
    if config.secret.is_empty() {
        config.secret = development_secret(&config)?;
    }
    if config.smtp_url.is_none() {
        tracing::warn!("EBC_SMTP_URL unset: nobody can sign in (sign-in links go by mail)");
    }
    if config.contact_email.is_none() {
        tracing::warn!("EBC_SHELF_CONTACT_EMAIL unset: the DSA point of contact is missing");
    }
    let bind = config.bind.clone();
    let public_url = config.public_url.clone();
    let mode = config.mode;
    let state = AppState::builder(config).store(Arc::new(store)).build()?;

    // Retention and pending cancellations, every five minutes.
    let maintenance = state.clone();
    tokio::spawn(async move {
        loop {
            maintenance.maintain().await;
            tokio::time::sleep(Duration::from_secs(300)).await;
        }
    });

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(%bind, %public_url, mode = mode.as_str(), "ebc-shelf is listening");
    serve(listener, state, async {
        if tokio::signal::ctrl_c().await.is_err() {
            std::future::pending::<()>().await;
        }
    })
    .await?;
    Ok(())
}
