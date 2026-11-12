//! Records real responses of the open (class A) providers, to check the hand-written
//! fixtures against the live services and to replace them.
//!
//! ```sh
//! EBC_LIVE=1 cargo run -p ebc-catalog --features client --example record-fixtures -- \
//!     target/recorded 9782070360024 9783596294312 9780141439518
//! ```
//!
//! For every provider and ISBN the example runs the provider's state machine and writes
//! each response body to `<dir>/<provider>/<isbn>-<n>.<ext>`, plus `<dir>/index.json`
//! with the URL, status, content type and retrieval time of every file. Providers that
//! are not class A (Google Books) are skipped: their responses must not be committed.
//! Optional providers are recorded too, so that they can be verified before being
//! switched on. Set `EBC_CONTACT` to an e-mail address to identify yourself to Crossref.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ebc_catalog::client::{Client, ClientConfig};
use ebc_catalog::{Config, PROVIDER_IDS, Session, SourceClass, provider_by_id, wire_url};
use ebc_model::Isbn13;
use serde_json::json;

const DEFAULT_ISBNS: [&str; 3] = ["9782070360024", "9783596294312", "9780141439518"];

fn extension(content_type: Option<&str>, body: &[u8]) -> &'static str {
    let kind = content_type.unwrap_or("");
    let first = body.iter().find(|b| !b.is_ascii_whitespace()).copied();
    if kind.contains("json") || matches!(first, Some(b'{' | b'[')) {
        "json"
    } else if kind.contains("html") {
        "html"
    } else if kind.contains("xml") || first == Some(b'<') {
        "xml"
    } else {
        "txt"
    }
}

async fn record(out: &Path, isbns: &[Isbn13]) -> Result<(), String> {
    let mut config = Config::default();
    if let Ok(contact) = std::env::var("EBC_CONTACT") {
        config = config.with_contact_email(contact);
    }
    let client = Client::new(ClientConfig::default()).map_err(|e| e.to_string())?;
    let mut index = Vec::new();
    for id in PROVIDER_IDS {
        let Some(provider) = provider_by_id(id, &config) else { continue };
        let info = provider.info().clone();
        if info.class != SourceClass::Open {
            eprintln!("{id}: skipped (not an open source; do not commit its responses)");
            continue;
        }
        let dir = out.join(id);
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let interval = Duration::from_millis(u64::from(info.min_interval_ms));
        for isbn in isbns {
            let mut session = Session::new(&provider, isbn, 6);
            let mut count = 0;
            while let Some(request) = session.next_request() {
                match client.execute(&request, interval).await {
                    Ok(response) => {
                        count += 1;
                        let content_type = response.content_type();
                        let name = format!(
                            "{isbn}-{count}.{}",
                            extension(content_type.as_deref(), &response.body)
                        );
                        let file = dir.join(&name);
                        std::fs::write(&file, &response.body)
                            .map_err(|e| format!("{}: {e}", file.display()))?;
                        let retrieved =
                            SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
                        index.push(json!({
                            "provider": id,
                            "isbn": isbn.to_plain(),
                            "url": wire_url(&request),
                            "status": response.status,
                            "content_type": content_type,
                            "file": format!("{id}/{name}"),
                            "retrieved_unix": retrieved,
                            "licence": info.data_licence,
                        }));
                        session.feed(&request, &response);
                    }
                    Err(error) => session.fail(&request, &error.to_string()),
                }
            }
            match session.finish() {
                Ok(outcome) => match outcome.record {
                    Some(book) => eprintln!(
                        "{id} {isbn}: {} — {} ({} request(s))",
                        book.title,
                        book.author_line().unwrap_or_else(|| "no author".to_owned()),
                        outcome.requests
                    ),
                    None => eprintln!("{id} {isbn}: not found {:?}", outcome.notes),
                },
                Err(error) => eprintln!("{id} {isbn}: ERROR {error}"),
            }
        }
    }
    let index_file = out.join("index.json");
    let text = serde_json::to_string_pretty(&index).map_err(|e| e.to_string())?;
    std::fs::write(&index_file, text).map_err(|e| format!("{}: {e}", index_file.display()))?;
    eprintln!("wrote {} responses; index in {}", index.len(), index_file.display());
    Ok(())
}

fn main() -> std::process::ExitCode {
    let mut args = std::env::args().skip(1);
    let (Some(out), true) = (args.next(), std::env::var("EBC_LIVE").as_deref() == Ok("1")) else {
        eprintln!(
            "usage: EBC_LIVE=1 record-fixtures <output-dir> [isbn ...]\n\
             This example sends real requests to public catalogues (one per second per \
             host): set EBC_LIVE=1 to confirm."
        );
        return std::process::ExitCode::from(2);
    };
    let mut texts: Vec<String> = args.collect();
    if texts.is_empty() {
        texts = DEFAULT_ISBNS.iter().map(|s| (*s).to_owned()).collect();
    }
    let mut isbns = Vec::new();
    for text in &texts {
        match Isbn13::parse(text) {
            Ok(isbn) => isbns.push(isbn),
            Err(error) => {
                eprintln!("{text}: {error}");
                return std::process::ExitCode::from(2);
            }
        }
    }
    let runtime = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("cannot start the async runtime: {error}");
            return std::process::ExitCode::FAILURE;
        }
    };
    match runtime.block_on(record(&PathBuf::from(out), &isbns)) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}
