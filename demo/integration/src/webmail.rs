//! "Webmail (démo)": a read-only page over the demo mailboxes of the mail sink, so that a
//! person (or a browser test) can open the sign-in links and notices the demo shelf
//! sends. Fictional, loopback only, no authentication — never deploy it.
//!
//! `GET /` lists every message (newest first), `GET /?to=address` one mailbox; links in
//! a message are clickable (`id="mail-link-{message}-{n}"`). `GET /messages.json` is the
//! same as JSON.

use std::sync::Arc;

use axum::Router;
use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use ebc_demo_ereaders::{MailDevice, MailboxMessage};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Filter {
    #[serde(default)]
    to: String,
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn messages(sink: &MailDevice, to: &str) -> Vec<MailboxMessage> {
    let mut all = sink.mailbox(to);
    all.reverse();
    all
}

async fn page(State(sink): State<Arc<MailDevice>>, Query(filter): Query<Filter>) -> Response {
    let mut html = String::from(
        "<!DOCTYPE html><html lang=\"fr\"><head><meta charset=\"utf-8\"><title>Webmail (démo)</title>\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"></head>\
         <body style=\"font-family:sans-serif;max-width:50em;margin:0 auto;padding:12px\">\
         <h1>Webmail (démo)</h1><p>Démo — fictif, non affilié. Boîtes aux lettres de démonstration.</p>",
    );
    for message in messages(&sink, &filter.to) {
        html.push_str(&format!(
            "<article id=\"mail-{id}\" style=\"border-top:1px solid #000;padding:8px 0\">\
             <p><strong>À :</strong> <span class=\"to\">{to}</span><br><strong>De :</strong> {from}<br>\
             <strong>Objet :</strong> <span class=\"subject\">{subject}</span></p><pre style=\"white-space:pre-wrap\">",
            id = message.id,
            to = escape(&message.to),
            from = escape(&message.from),
            subject = escape(&message.subject)
        ));
        let mut links = 0;
        for line in message.text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                links += 1;
                html.push_str(&format!(
                    "<a id=\"mail-link-{}-{links}\" href=\"{url}\">{url}</a>\n",
                    message.id,
                    url = escape(trimmed)
                ));
            } else {
                html.push_str(&escape(line));
                html.push('\n');
            }
        }
        html.push_str("</pre></article>");
    }
    html.push_str("</body></html>");
    Html(html).into_response()
}

async fn json(State(sink): State<Arc<MailDevice>>, Query(filter): Query<Filter>) -> Response {
    let list: Vec<serde_json::Value> = messages(&sink, &filter.to)
        .into_iter()
        .map(|m| serde_json::json!({"id": m.id, "to": m.to, "from": m.from, "subject": m.subject, "text": m.text}))
        .collect();
    axum::Json(list).into_response()
}

/// The webmail over `sink`'s mailboxes.
pub(crate) fn router(sink: Arc<MailDevice>) -> Router {
    Router::new().route("/", get(page)).route("/messages.json", get(json)).with_state(sink)
}
