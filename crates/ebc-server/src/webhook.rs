//! Signed webhooks to partners (`docs/api.md` §4.4).
//!
//! An event is written to the outbox (so it survives a restart), then delivered by a
//! background task: `POST {webhook_url}` with the JSON event, signed like a partner
//! request (§4.1) with a fresh timestamp on every attempt. Anything but a `2xx` answer is
//! retried with exponential backoff ([`WebhookPolicy`](crate::config::WebhookPolicy));
//! after the last attempt the event is dropped and logged. Events may arrive more than
//! once and out of order: receivers must be idempotent (each event carries an `id`).

use std::time::Duration;

use crate::codes;
use crate::state::AppState;
use crate::store::{GiftRecord, WebhookRecord};

impl AppState {
    /// Queues `event` (`gift.redeemed`, `gift.downloaded`, `gift.cancelled`) for the
    /// partner of `gift`, if it has one with a webhook URL. Never fails the request that
    /// caused the event.
    pub(crate) fn emit(&self, gift: &GiftRecord, event: &'static str) {
        let Some(partner) = self.partner_of(gift) else { return };
        if partner.webhook_url.is_none() {
            return;
        }
        let Ok(token) = codes::random_hex(self.entropy.as_ref(), 12) else { return };
        let id = format!("evt_{token}");
        let now = self.now();
        let body = serde_json::json!({
            "id": id,
            "event": event,
            "gift_id": gift.id,
            "order_ref": gift.order_ref,
            "at": now,
        })
        .to_string();
        let record = WebhookRecord { partner: partner.id.clone(), body, created_at: now };
        if let Err(error) = self.db.update(|db| db.webhooks.insert(id.clone(), record)) {
            tracing::error!(%error, "could not queue a webhook");
            return;
        }
        self.spawn_delivery(id);
    }

    /// Starts delivering every event left in the outbox (call once at start-up).
    pub fn resume_webhooks(&self) {
        for id in self.db.view(|db| db.webhooks.keys().cloned().collect::<Vec<_>>()) {
            self.spawn_delivery(id);
        }
    }

    /// Number of events waiting for delivery (for tests and metrics).
    #[must_use]
    pub fn pending_webhooks(&self) -> usize {
        self.db.view(|db| db.webhooks.len())
    }

    fn spawn_delivery(&self, id: String) {
        // Outside a runtime (synchronous unit tests) the event stays in the outbox.
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            runtime.spawn(deliver(self.clone(), id));
        }
    }

    fn forget_webhook(&self, id: &str) {
        if let Err(error) = self.db.update(|db| db.webhooks.remove(id)) {
            tracing::error!(%error, "could not update the webhook outbox");
        }
    }
}

/// Delivers one event, retrying with backoff.
async fn deliver(state: AppState, id: String) {
    let policy = state.config.webhook;
    for attempt in 1..=policy.max_attempts.max(1) {
        let Some(record) = state.db.view(|db| db.webhooks.get(&id).cloned()) else { return };
        let target = state
            .config
            .partner(&record.partner)
            .and_then(|p| Some((p, reqwest::Url::parse(p.webhook_url.as_deref()?).ok()?)));
        let Some((partner, url)) = target else {
            // The partner or its webhook was removed from the configuration.
            state.forget_webhook(&id);
            return;
        };
        let path_and_query = match url.query() {
            Some(query) => format!("{}?{query}", url.path()),
            None => url.path().to_owned(),
        };
        let timestamp = state.now();
        let signature = codes::request_signature(
            &partner.secret,
            timestamp,
            "POST",
            &path_and_query,
            record.body.as_bytes(),
        );
        let outcome = state
            .http
            .post(url)
            .timeout(Duration::from_millis(policy.timeout_ms.max(1)))
            .header("Content-Type", "application/json")
            .header("X-EBC-Partner", &partner.id)
            .header("X-EBC-Timestamp", timestamp.to_string())
            .header("X-EBC-Signature", signature)
            .body(record.body.clone())
            .send()
            .await;
        match outcome {
            Ok(response) if response.status().is_success() => {
                state.forget_webhook(&id);
                return;
            }
            Ok(response) => {
                tracing::warn!(event = %id, attempt, status = %response.status(), "webhook refused");
            }
            Err(error) => {
                // Webhook URLs may carry credentials or tokens in their query: not logged.
                let error = error.without_url();
                tracing::warn!(event = %id, attempt, %error, "webhook not delivered");
            }
        }
        if attempt < policy.max_attempts {
            tokio::time::sleep(policy.delay_after(attempt)).await;
        }
    }
    tracing::error!(event = %id, "webhook given up after the last attempt");
    state.forget_webhook(&id);
}
