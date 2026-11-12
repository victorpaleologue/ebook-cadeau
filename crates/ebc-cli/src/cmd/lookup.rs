//! `ebc lookup`: metadata by ISBN through the catalogue providers.

use std::time::Duration;

use ebc_catalog::{
    Config, DEFAULT_USER_AGENT, LookupOptions, LookupReport, PROVIDER_IDS, Provider, ProviderInfo,
    provider_by_id, provider_infos, providers_for,
};
use ebc_model::{BookMeta, ContributorRole};
use serde_json::{Value, json};

use crate::args::LookupArgs;
use crate::cmd::parse_isbn;
use crate::error::{CliError, Status};
use crate::transport::{NetOptions, Transport};
use crate::util::Streams;

/// Largest catalogue answer accepted from the network (the library's own limit).
const MAX_CATALOG_BODY: usize = 2 * 1024 * 1024;

/// Splits `PROVIDER=URL`.
fn parse_base_url(text: &str) -> Result<(String, String), CliError> {
    let (provider, url) = text
        .split_once('=')
        .ok_or_else(|| CliError::usage(format!("--base-url {text:?}: expected PROVIDER=URL")))?;
    let provider = provider.trim().to_ascii_lowercase();
    if !PROVIDER_IDS.contains(&provider.as_str()) {
        return Err(CliError::usage(format!(
            "--base-url: unknown provider {provider:?}; known: {}",
            PROVIDER_IDS.join(", ")
        )));
    }
    let url = url.trim();
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(CliError::usage(format!(
            "--base-url {provider}=…: the URL must start with http:// or https://"
        )));
    }
    Ok((provider, url.to_owned()))
}

/// `true` when the URL's host is this machine (`localhost`, 127.0.0.0/8, `[::1]`).
fn is_loopback_url(url: &str) -> bool {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let authority = authority.rsplit_once('@').map_or(authority, |(_, host)| host);
    let host = if let Some(bracketed) = authority.strip_prefix('[') {
        bracketed.split(']').next().unwrap_or("")
    } else {
        authority.split(':').next().unwrap_or("")
    };
    host.eq_ignore_ascii_case("localhost")
        || host.parse::<std::net::IpAddr>().is_ok_and(|ip| ip.is_loopback())
}

fn list_providers(json: bool, streams: &mut Streams<'_>) -> Result<Status, CliError> {
    let infos = provider_infos();
    if json {
        let rows: Vec<Value> = infos.iter().map(|info| info_json(info, None)).collect();
        streams.json(&Value::Array(rows))?;
    } else {
        for info in &infos {
            streams.line(format!(
                "{:<12} {}{}",
                info.id,
                info.name,
                match (info.default_enabled, info.requires_key) {
                    (_, true) => "  [needs a key]",
                    (true, false) => "",
                    (false, false) => "  [off by default: name it with --provider]",
                }
            ))?;
            streams.line(format!("             licence: {}", info.data_licence))?;
            streams.line(format!("             terms:   {}", info.terms_url))?;
        }
        streams.note(
            "note: licence and terms statements come from the project's research and were \
             not verified against the live services",
        );
    }
    Ok(Status::Success)
}

fn info_json(info: &ProviderInfo, record_url: Option<&str>) -> Value {
    json!({
        "id": info.id,
        "name": info.name,
        "homepage": info.homepage,
        "licence": info.data_licence,
        "licence_id": info.licence_id,
        "attribution": info.attribution,
        "terms_url": info.terms_url,
        "requires_key": info.requires_key,
        "default_enabled": info.default_enabled,
        "min_interval_ms": info.min_interval_ms,
        "record_url": record_url,
    })
}

/// The providers to query, in order.
fn select_providers(
    args: &LookupArgs,
    isbn: &ebc_model::Isbn13,
    config: &Config,
) -> Result<Vec<Box<dyn Provider>>, CliError> {
    if args.providers.is_empty() {
        return Ok(providers_for(isbn, config));
    }
    let mut chosen: Vec<Box<dyn Provider>> = Vec::new();
    for id in &args.providers {
        let id = id.trim().to_ascii_lowercase();
        let provider = provider_by_id(&id, config).ok_or_else(|| {
            CliError::usage(format!("unknown provider {id:?}; known: {}", PROVIDER_IDS.join(", ")))
        })?;
        if provider.info().requires_key && config.google_books_key.is_none() {
            return Err(CliError::usage(format!(
                "provider {id} needs an API key: set EBC_GOOGLE_BOOKS_KEY"
            )));
        }
        if !chosen.iter().any(|p| p.info().id == provider.info().id) {
            chosen.push(provider);
        }
    }
    Ok(chosen)
}

fn record_url<'a>(book: Option<&'a BookMeta>, provider: &str) -> Option<&'a str> {
    book?.sources.iter().find(|s| s.provider == provider)?.record_url.as_deref()
}

fn report_json(isbn: &ebc_model::Isbn13, report: &LookupReport, infos: &[ProviderInfo]) -> Value {
    let providers: Vec<Value> = report
        .providers
        .iter()
        .map(|entry| {
            let info = infos.iter().find(|i| i.id == entry.provider);
            let (status, requests, notes, error, record) = match &entry.result {
                Ok(outcome) => (
                    if outcome.record.is_some() { "found" } else { "not_found" },
                    outcome.requests,
                    outcome.notes.clone(),
                    None,
                    outcome.record.as_ref(),
                ),
                Err(error) => ("error", 0, Vec::new(), Some(error.to_string()), None),
            };
            let mut row = info.map_or_else(
                || json!({ "id": entry.provider }),
                |info| info_json(info, record_url(record, entry.provider)),
            );
            if let Some(map) = row.as_object_mut() {
                map.insert("status".to_owned(), json!(status));
                map.insert("requests".to_owned(), json!(requests));
                map.insert("notes".to_owned(), json!(notes));
                map.insert("error".to_owned(), json!(error));
            }
            row
        })
        .collect();
    json!({
        "isbn": isbn.to_plain(),
        "found": report.merged.is_some(),
        "record": report.merged,
        "providers": providers,
        "skipped": report.skipped,
    })
}

fn role_name(role: &ContributorRole) -> &str {
    match role {
        ContributorRole::Author => "author",
        ContributorRole::Translator => "translator",
        ContributorRole::Illustrator => "illustrator",
        ContributorRole::Editor => "editor",
        ContributorRole::Preface => "preface",
        ContributorRole::Narrator => "narrator",
        ContributorRole::Other(text) => text,
    }
}

fn print_record(book: &BookMeta, streams: &mut Streams<'_>) -> Result<(), CliError> {
    let mut field = |name: &str, value: Option<String>| -> Result<(), CliError> {
        match value.filter(|v| !v.trim().is_empty()) {
            Some(value) => streams.line(format!("{name:<11}{value}")),
            None => Ok(()),
        }
    };
    field("ISBN", book.isbn13.map(|i| i.hyphenated()))?;
    field("Title", Some(book.title.clone()))?;
    field("Subtitle", book.subtitle.clone())?;
    field("Authors", book.author_line())?;
    let others: Vec<String> = book
        .contributors
        .iter()
        .filter(|c| c.role != ContributorRole::Author)
        .map(|c| format!("{} ({})", c.name, role_name(&c.role)))
        .collect();
    field("Others", (!others.is_empty()).then(|| others.join(", ")))?;
    field("Publisher", book.publisher.clone())?;
    field("Imprint", book.imprint.clone())?;
    field("Series", book.series.clone())?;
    field("Published", book.published.clone())?;
    field("Language", book.language.clone())?;
    field("Pages", book.page_count.map(|n| n.to_string()))?;
    field("Format", book.format.map(|f| format!("{f:?}").to_lowercase()))?;
    field(
        "Covers",
        (!book.covers.is_empty()).then(|| {
            format!(
                "{} reference(s): for locating and previewing only, never printable by \
                 themselves (see `ebc cover`)",
                book.covers.len()
            )
        }),
    )?;
    Ok(())
}

fn print_sources(
    report: &LookupReport,
    infos: &[ProviderInfo],
    streams: &mut Streams<'_>,
) -> Result<(), CliError> {
    streams.line("")?;
    streams.line("Sources")?;
    for entry in &report.providers {
        let info = infos.iter().find(|i| i.id == entry.provider);
        let name = info.map_or(entry.provider, |i| i.name);
        match &entry.result {
            Ok(outcome) => {
                let verdict = if outcome.record.is_some() { "found" } else { "not found" };
                streams.line(format!(
                    "  {:<12} {verdict} ({} request(s)): {name}",
                    entry.provider, outcome.requests
                ))?;
                if let (Some(info), Some(record)) = (info, &outcome.record) {
                    streams.line(format!("               licence: {}", info.data_licence))?;
                    if let Some(attribution) = info.attribution {
                        streams.line(format!("               attribution: {attribution}"))?;
                    }
                    if let Some(url) = record_url(Some(record), entry.provider) {
                        streams.line(format!("               record: {url}"))?;
                    }
                    streams.line(format!("               terms: {}", info.terms_url))?;
                }
                for note in &outcome.notes {
                    streams.note(format!("note: {}: {note}", entry.provider));
                }
            }
            Err(error) => {
                streams.line(format!("  {:<12} error: {name}", entry.provider))?;
                streams.note(format!("warning: {error}"));
            }
        }
    }
    if !report.skipped.is_empty() {
        streams.line(format!(
            "  not queried (the record was already complete; --all asks them too): {}",
            report.skipped.join(", ")
        ))?;
    }
    Ok(())
}

/// Runs `ebc lookup`.
pub(crate) fn run(args: &LookupArgs, streams: &mut Streams<'_>) -> Result<Status, CliError> {
    if args.list_providers {
        return list_providers(args.json, streams);
    }
    let isbn = parse_isbn(args.isbn.as_deref().unwrap_or_default())?;

    let mut config = Config::default();
    if let Some(key) = args.google_books_key.as_deref().filter(|k| !k.trim().is_empty()) {
        config = config.with_google_books_key(key.trim());
    }
    if let Some(contact) = args.contact.as_deref().filter(|c| !c.trim().is_empty()) {
        config = config.with_contact_email(contact.trim());
    }
    for text in &args.base_urls {
        let (provider, url) = parse_base_url(text)?;
        config = config.with_base_url(provider, url);
    }
    for id in &args.providers {
        config = config.with_enabled(id.trim().to_ascii_lowercase());
    }
    let providers = select_providers(args, &isbn, &config)?;
    if providers.is_empty() {
        return Err(CliError::failure("no provider is enabled for this ISBN"));
    }

    // A lookup that only talks to fixture servers on this machine has no remote service
    // to be polite to and no proxy to go through.
    let all_local = providers
        .iter()
        .all(|p| config.base_urls.get(p.info().id).is_some_and(|url| is_loopback_url(url)));
    let transport = Transport::open(
        args.net.offline_fixtures.as_deref(),
        NetOptions {
            user_agent: args
                .net
                .user_agent
                .clone()
                .unwrap_or_else(|| DEFAULT_USER_AGENT.to_owned()),
            timeout: Duration::from_secs(args.net.timeout),
            use_proxy: !(args.net.no_proxy || all_local),
            min_interval: all_local.then_some(Duration::ZERO),
            max_body_bytes: MAX_CATALOG_BODY,
        },
    )?;

    let options = LookupOptions { stop_when_complete: !args.all, ..LookupOptions::default() };
    let report = transport.lookup_all(&providers, &isbn, &options);
    let infos = provider_infos();

    if args.json {
        streams.json(&report_json(&isbn, &report, &infos))?;
        for error in report.errors() {
            streams.note(format!("warning: {error}"));
        }
    } else {
        match &report.merged {
            Some(book) => print_record(book, streams)?,
            None => streams.line(format!("ISBN       {}", isbn.hyphenated()))?,
        }
        print_sources(&report, &infos, streams)?;
    }

    if transport.is_offline()
        && report.errors().any(|e| matches!(e, ebc_catalog::CatalogError::Transport { .. }))
    {
        streams.note(
            "note: --offline-fixtures: a request without a fixture fails like an unreachable host",
        );
    }

    match (&report.merged, report.errors().next()) {
        (Some(_), _) => Ok(Status::Success),
        (None, None) => {
            streams.note(format!("no provider knows ISBN {}", isbn.hyphenated()));
            Ok(Status::NotFound)
        }
        (None, Some(_)) => Err(CliError::failure(format!(
            "no record for ISBN {}: {} of {} provider(s) failed",
            isbn.hyphenated(),
            report.errors().count(),
            report.providers.len()
        ))),
    }
}
