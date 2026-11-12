//! Demo binary: serves the two fictional eBook stores, each on its own port.
//!
//! | Variable | Default | Meaning |
//! |---|---|---|
//! | `DEMO_SEPTANTE_BIND` | `127.0.0.1:8101` | listen address of Septante |
//! | `DEMO_GRANDE_ENSEIGNE_BIND` | `127.0.0.1:8102` | listen address of Grande Enseigne |
//!
//! plus the variables documented on [`ebc_demo_stores::StoreConfig::from_env`]
//! (`EBC_URL`, `EBC_PRINTER_URL`, `DEMO_*_PUBLIC_URL`, `DEMO_*_PARTNER_ID`,
//! `DEMO_*_PARTNER_SECRET`, `DEMO_*_DELIVERY`, `DEMO_TEST_HOOKS`). See `demo/README.md`.

use std::error::Error;

use ebc_demo_stores::assets::Assets;
use ebc_demo_stores::{StoreConfig, StoreKind, app};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let env = |name: &str| std::env::var(name).ok();

    // Generate the EPUBs and covers before accepting requests, and fail fast if that is
    // impossible.
    let assets = Assets::get()?;
    assets.warm_up()?;

    let mut servers = Vec::new();
    for kind in StoreKind::ALL {
        let config = StoreConfig::from_env(kind, env)?;
        let bind_var = format!("DEMO_{}_BIND", kind.slug().to_ascii_uppercase().replace('-', "_"));
        let bind = env(&bind_var)
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| format!("127.0.0.1:{}", kind.default_port()));
        let listener = tokio::net::TcpListener::bind(&bind)
            .await
            .map_err(|e| format!("cannot listen on {bind} ({bind_var}): {e}"))?;
        eprintln!(
            "{} (démo) — http://{} — public URL {} — gifts: {} via {} — printer: {} — test hooks: {}",
            kind.display_name(),
            listener.local_addr()?,
            config.public_url,
            config.gift_delivery.name(),
            config.partner.as_ref().map_or("disabled", |p| p.base_url.as_str()),
            config.printer_url.as_deref().unwrap_or("none"),
            if config.test_hooks { "on" } else { "off" },
        );
        let router = app(config);
        servers.push(tokio::spawn(async move {
            axum::serve(listener, router).with_graceful_shutdown(shutdown()).await
        }));
    }
    for server in servers {
        server.await??;
    }
    Ok(())
}

/// Resolves on Ctrl-C.
async fn shutdown() {
    // If the signal handler cannot be installed, keep serving until the process is killed.
    if tokio::signal::ctrl_c().await.is_err() {
        std::future::pending::<()>().await;
    }
}
