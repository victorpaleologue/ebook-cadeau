//! `ebc-demo`: the whole demo constellation on fixed local ports — the real eBook Cadeau
//! server, the two fictional stores, the fictional printer and a Send-to-Kindle mail sink —
//! with a printed table of URLs. For manual demos and for browser-driven (Playwright)
//! tests. Everything is in memory and fictional; stop it with Ctrl-C.
//!
//! | Variable | Default | Meaning |
//! |---|---|---|
//! | `EBC_DEMO_SERVER_PORT` | `8080` | eBook Cadeau server |
//! | `EBC_DEMO_SEPTANTE_PORT` | `8101` | store "Septante" (`hosted_file`) |
//! | `EBC_DEMO_GRANDE_ENSEIGNE_PORT` | `8102` | store "Grande Enseigne" (`reseller_redirect`) |
//! | `EBC_DEMO_PRINTER_PORT` | `8103` | printer |
//! | `EBC_DEMO_KINDLE_ADDRESS` | `demo@kindle.com` | device address of the mail sink |
//! | `EBC_DEMO_PRINTER_TICK_MS` | `2000` | time between two steps of a print order (`0` = manual) |
//! | `EBC_WEB_DIR` | unset | directory of the static card designer, served at `/app/` (e.g. `web`) |
//!
//! A port set to `0` is chosen by the operating system (the table shows it). See
//! `demo/README.md`.

#![forbid(unsafe_code)]

use std::io::Write;
use std::process::ExitCode;
use std::time::Duration;

use ebc_demo_ereaders::DeviceProfile;
use ebc_demo_integration::{World, WorldPorts};

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
}

fn port(name: &str, default: u16) -> Result<u16, String> {
    match env(name) {
        Some(value) => value.parse().map_err(|_| format!("{name} must be a port, got {value:?}")),
        None => Ok(default),
    }
}

async fn run() -> Result<(), String> {
    let ports = WorldPorts {
        server: port("EBC_DEMO_SERVER_PORT", WorldPorts::DEMO.server)?,
        septante: port("EBC_DEMO_SEPTANTE_PORT", WorldPorts::DEMO.septante)?,
        grande_enseigne: port("EBC_DEMO_GRANDE_ENSEIGNE_PORT", WorldPorts::DEMO.grande_enseigne)?,
        printer: port("EBC_DEMO_PRINTER_PORT", WorldPorts::DEMO.printer)?,
    };
    let kindle_address =
        env("EBC_DEMO_KINDLE_ADDRESS").unwrap_or_else(|| "demo@kindle.com".to_owned());
    let tick_ms = match env("EBC_DEMO_PRINTER_TICK_MS") {
        Some(value) => value
            .parse::<u64>()
            .map_err(|_| format!("EBC_DEMO_PRINTER_TICK_MS must be milliseconds, got {value:?}"))?,
        None => 2_000,
    };

    let mut builder = World::builder()
        .ports(ports)
        .system_clock()
        .slow_passwords()
        .mail_device(DeviceProfile::kindle_paperwhite(), &kindle_address);
    if tick_ms > 0 {
        builder = builder.printer_tick(Duration::from_millis(tick_ms));
    }
    // Serve the card designer at /app/ when EBC_WEB_DIR is set, as the real server does.
    if let Some(web_dir) = env("EBC_WEB_DIR") {
        builder = builder.server_config(move |config| {
            config.web_dir = Some(std::path::PathBuf::from(web_dir));
        });
    }
    let world = builder.start().await.map_err(|e| e.to_string())?;
    // In a demo the recipient has already approved our sender (the step Send to Kindle
    // requires); the integration tests cover the unapproved case.
    if let Some(mail) = &world.mail {
        mail.approve_sender(&world.server.state.config.send_from);
    }

    let mut out = std::io::stdout().lock();
    let io = |e: std::io::Error| e.to_string();
    writeln!(
        out,
        "eBook Cadeau — demo constellation (demo — not affiliated with any real store or printer)\n"
    )
    .map_err(io)?;
    let rows = world.services();
    let name_width = rows.iter().map(|(name, _, _)| name.chars().count()).max().unwrap_or(0);
    let url_width = rows.iter().map(|(_, url, _)| url.chars().count()).max().unwrap_or(0);
    for (name, url, note) in &rows {
        writeln!(out, "  {name:<name_width$}  {url:<url_width$}  {note}").map_err(io)?;
    }
    writeln!(out).map_err(io)?;
    writeln!(out, "Try it:").map_err(io)?;
    writeln!(out, "  1. Buy a book as a gift at {} (any e-mail, no payment).", world.septante.base)
        .map_err(io)?;
    writeln!(out, "  2. The order page shows the claim URL and the code XXXX-XXXX-XXXX.")
        .map_err(io)?;
    writeln!(
        out,
        "  3. Redeem like an e-reader would:\n     cargo run -p ebc-demo-ereaders --bin ebc-ereader -- --profile kobo open {}/ --code <CODE>",
        world.server.base
    )
    .map_err(io)?;
    writeln!(
        out,
        "  4. Send to Kindle from the gift page: use {kindle_address} (the sink accepts it; sender already approved)."
    )
    .map_err(io)?;
    writeln!(
        out,
        "  5. At {} the gift is added to a store account instead; a device syncs it through /api/device/*.",
        world.grande_enseigne.base
    )
    .map_err(io)?;
    writeln!(out, "\nState hooks: <store>/__test/state, <printer>/__test/orders. Ctrl-C to stop.")
        .map_err(io)?;
    out.flush().map_err(io)?;
    drop(out);

    // Without a signal handler the constellation simply runs until it is killed.
    if tokio::signal::ctrl_c().await.is_err() {
        std::future::pending::<()>().await;
    }
    drop(world);
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // The only output channel of a command-line tool for its failure.
            let _ = writeln!(std::io::stderr(), "ebc-demo: {error}");
            ExitCode::FAILURE
        }
    }
}
