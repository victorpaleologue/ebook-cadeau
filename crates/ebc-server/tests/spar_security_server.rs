//! Sparring review (security lens): failing tests that demonstrate defects.
//! Every test here asserts the SECURE behaviour; a failure is the finding.

#![allow(clippy::unwrap_used, missing_docs, dead_code, unreachable_pub)]

#[path = "it/common.rs"]
mod common;

use std::io::Write as _;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use axum::body::{Body, Bytes};
use axum::http::{Method, Request, StatusCode};
use common::{Harness, PARTNER_ID, PARTNER_SECRET, SmtpSink, T0, partner};
use serde_json::json;
use tower::ServiceExt;

fn code_of(gift: &serde_json::Value) -> String {
    gift["code"].as_str().unwrap().to_owned()
}

// ---------------------------------------------------------------------------------------
// F1: replay protection of signed partner POSTs is bypassed by unsigned header changes.
// ---------------------------------------------------------------------------------------
#[tokio::test]
async fn spar_captured_reissue_cannot_be_replayed_with_another_idempotency_key() {
    let h = Harness::with(|c| c.partners = vec![partner(&["store.example"], None)]);
    let gift = h
        .partner_gift(
            json!({ "type": "reseller_redirect", "url": "https://store.example/activer" }),
            json!({}),
        )
        .await;
    let id = gift["id"].as_str().unwrap().to_owned();
    let path = format!("/api/partner/v1/gifts/{id}/reissue");
    // The partner's genuine request, as it would be captured (logs, proxy, ...).
    let ts = h.state.now();
    let sig = ebc_server::codes::request_signature(PARTNER_SECRET, ts, "POST", &path, b"");
    let ts = ts.to_string();
    let base = [("x-ebc-partner", PARTNER_ID), ("x-ebc-timestamp", ts.as_str())];

    let send = |extra: Vec<(&'static str, String)>, sig: String| {
        let mut builder = Request::builder().method(Method::POST).uri(path.clone());
        for (n, v) in base {
            builder = builder.header(n, v);
        }
        builder = builder.header("x-ebc-signature", sig);
        for (n, v) in extra {
            builder = builder.header(n, v);
        }
        builder.body(Body::empty()).unwrap()
    };

    let first = h.send(send(vec![], sig.clone())).await;
    assert_eq!(first.status, StatusCode::OK, "{}", first.text());
    let issued = first.json()["code"].as_str().unwrap().to_owned();

    // Byte-identical replay: answered from the idempotency store (documented, works).
    let replay = h.send(send(vec![], sig.clone())).await;
    assert_eq!(replay.json()["code"].as_str().unwrap(), issued);

    // Same captured signature, plus an Idempotency-Key header (not covered by the MAC).
    let with_key = h.send(send(vec![("idempotency-key", "attacker-1".to_owned())], sig.clone())).await;
    // Same captured signature, hex upper-cased (unhex accepts both cases).
    let upper = format!("v1={}", sig.trim_start_matches("v1=").to_ascii_uppercase());
    let upper_case = h.send(send(vec![], upper)).await;

    let still_valid = h.get(&format!("/k/{issued}")).await.status;
    eprintln!(
        "with_key: {} {:?}; upper: {} {:?}; code issued to the partner now answers {}",
        with_key.status,
        with_key.json().get("code"),
        upper_case.status,
        upper_case.json().get("code"),
        still_valid
    );
    assert!(
        with_key.json().get("code").and_then(|c| c.as_str()) == Some(issued.as_str())
            || with_key.status.is_client_error(),
        "a replay with a fresh Idempotency-Key executed the reissue again"
    );
    assert!(
        upper_case.json().get("code").and_then(|c| c.as_str()) == Some(issued.as_str())
            || upper_case.status.is_client_error(),
        "a replay with an upper-cased signature executed the reissue again"
    );
    assert_eq!(still_valid, StatusCode::OK, "the code the partner printed was voided by a replay");
}

// ---------------------------------------------------------------------------------------
// F2: send-to-device daily limit (3) is a check-then-act race.
// ---------------------------------------------------------------------------------------
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn spar_concurrent_sends_respect_the_daily_limit() {
    let sink = SmtpSink::start().await;
    let url = sink.url();
    let h = Harness::with(|c| {
        c.smtp_url = Some(url);
        c.send_from = "livraison@ebk.example".to_owned();
    });
    let code = code_of(&h.hosted_gift(json!({})).await);
    let mut tasks = Vec::new();
    for i in 0..12 {
        let app = h.app.clone();
        let path = format!("/k/{code}/send");
        tasks.push(tokio::spawn(async move {
            let request = Request::builder()
                .method(Method::POST)
                .uri(path)
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(format!("device_email=victim{i}%40kindle.com")))
                .unwrap();
            app.oneshot(request).await.unwrap().status()
        }));
    }
    let mut ok = 0;
    for task in tasks {
        if task.await.unwrap() == StatusCode::OK {
            ok += 1;
        }
    }
    let mails = sink.messages().len();
    let claims = h.state.db.view(|db| db.gifts.values().next().map(|g| (g.claims, g.sends.len())));
    eprintln!("200 answers: {ok}; mails accepted by SMTP: {mails}; (claims, sends) stored: {claims:?}");
    assert!(mails <= 3, "{mails} e-mails sent although MAX_SENDS_PER_DAY is 3");
}

// ---------------------------------------------------------------------------------------
// F3: a failed licence fetch logs the signed store URL (ebc_sig) verbatim.
// ---------------------------------------------------------------------------------------
#[derive(Clone, Default)]
struct LogBuf(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for LogBuf {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for LogBuf {
    type Writer = LogBuf;
    fn make_writer(&'a self) -> LogBuf {
        self.clone()
    }
}

#[tokio::test]
async fn spar_licence_fetch_failure_does_not_log_the_signed_url() {
    // A port nothing listens on: the licence request fails with a connection error.
    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };
    let host = format!("127.0.0.1:{port}");
    let h = Harness::with(|c| c.partners = vec![partner(&[host.as_str()], None)]);
    let gift = h
        .partner_gift(
            json!({ "type": "reseller_licence", "format": "lcpl", "url": format!("http://{host}/lcp/licence") }),
            json!({}),
        )
        .await;
    let (id, code) = (gift["id"].as_str().unwrap().to_owned(), code_of(&gift));

    let logs = LogBuf::default();
    let subscriber = tracing_subscriber::fmt().with_writer(logs.clone()).with_ansi(false).finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    let reply = h.post_form(&format!("/k/{code}/redeem"), &[]).await;
    assert_eq!(reply.status, StatusCode::BAD_GATEWAY);
    let text = String::from_utf8(logs.0.lock().unwrap().clone()).unwrap();
    let signature = ebc_server::codes::redirect_signature(PARTNER_SECRET, &id, "ORD-2026-000123", T0);
    let _ = std::io::stderr().write_all(text.as_bytes());
    assert!(
        !text.contains(&signature) && !text.contains("ebc_sig="),
        "the log line carries a valid store signature:\n{text}"
    );
}

// ---------------------------------------------------------------------------------------
// F4: an unauthenticated partner upload is buffered in full before the MAC is checked.
// ---------------------------------------------------------------------------------------
struct Zeros {
    left: usize,
    pulled: Arc<AtomicUsize>,
}

impl tokio::io::AsyncRead for Zeros {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let n = self.left.min(buf.remaining()).min(64 * 1024);
        buf.put_slice(&vec![0u8; n]);
        self.left -= n;
        self.pulled.fetch_add(n, Ordering::SeqCst);
        Poll::Ready(Ok(()))
    }
}

#[tokio::test]
async fn spar_unsigned_partner_upload_is_refused_before_its_body_is_buffered() {
    let h = Harness::new();
    let limit = h.state.config.max_upload_bytes;
    let pulled = Arc::new(AtomicUsize::new(0));
    let body = Body::from_stream(tokio_util::io::ReaderStream::new(Zeros {
        left: limit,
        pulled: pulled.clone(),
    }));
    // Partner ids are not secret; the timestamp is just the current time.
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/partner/v1/uploads")
        .header("x-ebc-partner", PARTNER_ID)
        .header("x-ebc-timestamp", T0.to_string())
        .header("x-ebc-signature", "v1=00")
        .body(body)
        .unwrap();
    let reply = h.send(request).await;
    let read = pulled.load(Ordering::SeqCst);
    eprintln!("status {} after the server pulled {read} of {limit} bytes", reply.status);
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    assert!(read < 1024 * 1024, "{read} bytes buffered for a request with a forged signature");
}

// ---------------------------------------------------------------------------------------
// F5: personal data of a cancelled gift is kept for ever.
// ---------------------------------------------------------------------------------------
#[tokio::test]
async fn spar_cancelled_gift_forgets_names_and_message() {
    let h = Harness::new();
    let gift = h.hosted_gift(json!({})).await;
    let id = gift["id"].as_str().unwrap().to_owned();
    let token = gift["manage_token"].as_str().unwrap().to_owned();
    let auth = format!("Bearer {token}");
    let cancelled = h
        .call(Method::DELETE, &format!("/api/v1/gifts/{id}"), &[("authorization", auth.as_str())], Body::empty())
        .await;
    assert_eq!(cancelled.status, StatusCode::OK, "{}", cancelled.text());
    // A year later.
    h.clock.advance(365 * 86_400);
    let _ = h.hosted_gift(json!({})).await; // any later write runs the purge
    let kept = h.state.db.view(|db| {
        db.gifts.get(&id).map(|g| (g.giver_name.clone(), g.recipient_name.clone(), g.message.clone()))
    });
    eprintln!("stored a year after cancellation: {kept:?}");
    assert!(
        kept.as_ref().is_none_or(|(g, r, m)| g.is_none() && r.is_none() && m.is_none()),
        "names and message of a cancelled gift are still stored: {kept:?}"
    );
}

// ---------------------------------------------------------------------------------------
// F6: the "global" failure budget does not bound how many guesses are looked up.
// ---------------------------------------------------------------------------------------
#[tokio::test]
async fn spar_global_budget_bounds_looked_up_guesses() {
    let h = Harness::new();
    let code = code_of(&h.hosted_gift(json!({})).await);
    let wrong = |n: u64| ebc_model::GiftCode::from_payload(n).unwrap().compact();
    // 500 wrong guesses from 50 networks: the global budget is spent.
    for i in 0..500u64 {
        let client = format!("10.{}.{}.1", i / 10 / 256, (i / 10) % 256);
        let _ = h.get_from(&format!("/k/{}", wrong(i + 1)), &client).await;
    }
    // 2 000 more guesses from fresh /56 networks are all still looked up …
    let mut answered_429 = 0;
    for i in 0..2_000u64 {
        let client = format!("2001:db8:{:x}:{:x}00::1", i / 256, i % 256);
        if h.get_from(&format!("/k/{}", wrong(10_000 + i)), &client).await.status
            == StatusCode::TOO_MANY_REQUESTS
        {
            answered_429 += 1;
        }
    }
    // … as the next fresh network shows: its guess hits.
    let hit = h.get_from(&format!("/k/{code}"), "2001:db8:ffff:ff00::1").await.status;
    eprintln!("after 2 500 failures in one window (global budget 500): {answered_429} answered 429, a fresh network's correct guess answers {hit}");
    assert_ne!(hit, StatusCode::OK, "guesses beyond the global budget are still evaluated");
}

// ---------------------------------------------------------------------------------------
// F7: the binary's server has no header/body read timeout (axum::serve sets no hyper timer).
// ---------------------------------------------------------------------------------------
#[tokio::test]
async fn spar_a_client_that_never_finishes_its_headers_is_disconnected() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let h = Harness::new();
    // Mounted exactly as src/main.rs mounts it.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = h.app.clone();
    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>())
            .await
            .unwrap();
    });
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    stream.write_all(b"GET / HTTP/1.1\r\nHost: x\r\n").await.unwrap(); // never ends
    // hyper's documented default header timeout is 30 s; wait past it.
    tokio::time::sleep(std::time::Duration::from_secs(33)).await;
    let mut buf = [0u8; 64];
    let read = tokio::time::timeout(std::time::Duration::from_millis(500), stream.read(&mut buf)).await;
    // Ok(Ok(0)) = closed by the server; Err(_) = still open and waiting.
    eprintln!("after 33 s of an unfinished request header: {read:?}");
    assert!(matches!(read, Ok(Ok(0)) | Ok(Err(_))), "the connection is still held open by the server");
}

// ---------------------------------------------------------------------------------------
// F8: an unauthenticated X-EBC-Timestamp overflows the skew arithmetic.
// ---------------------------------------------------------------------------------------
#[tokio::test]
async fn spar_extreme_partner_timestamps_are_refused_without_panicking() {
    let h = Harness::new();
    let now = h.state.now();
    for ts in [i64::MIN.to_string(), now.wrapping_sub(i64::MIN).to_string()] {
        let app = h.app.clone();
        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/partner/v1/gifts/g_00000000000000000000000000000000")
            .header("x-ebc-partner", PARTNER_ID)
            .header("x-ebc-timestamp", ts.clone())
            .header("x-ebc-signature", "v1=00")
            .body(Body::empty())
            .unwrap();
        let outcome = tokio::spawn(async move { app.oneshot(request).await.unwrap().status() }).await;
        eprintln!("timestamp {ts}: {outcome:?}");
        assert!(
            matches!(outcome, Ok(StatusCode::UNAUTHORIZED)),
            "timestamp {ts}: the handler panicked or did not refuse: {outcome:?}"
        );
    }
}
