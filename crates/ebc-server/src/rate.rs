//! Throttling of code guesses (`docs/api.md` §2 and §6).
//!
//! A gift code has no user name: a guess is a guess against **every** live code, so
//! per-code lockout is useless (`docs/research/notes/ereader-ux.ux.md` §3.2). Two budgets
//! apply to *unknown* codes (well-formed but not issued — typos caught by the check
//! symbol never count), each over a sliding ten-minute window:
//!
//! * **per client** (`EBC_RATE_UNKNOWN_PER_10MIN`, default 10): once a client has used its
//!   attempts, every code it submits is answered `429` **without being looked up** until
//!   the oldest attempt leaves the window;
//! * **global** (`EBC_RATE_UNKNOWN_GLOBAL_PER_10MIN`, default 500): once all clients
//!   together have exhausted it, the service is *under attack*: every further wrong guess
//!   is answered `429`, and a client that already guessed wrong inside the window is not
//!   looked up at all. Clients without a recent failure keep working, so a recipient
//!   scanning a valid card is not locked out by somebody else's attack.
//!
//! State is in memory only: a restart forgets recent failures.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

/// Length of the sliding window, in seconds.
pub const WINDOW_SECS: i64 = 600;

/// What the limiter says about a client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The client may submit a code. `remaining` unknown codes are left in its budget
    /// (`None` when the per-client limit is disabled).
    Allowed {
        /// Attempts left before the client is blocked.
        remaining: Option<u32>,
    },
    /// The client must wait.
    Blocked {
        /// Seconds until a new attempt is accepted (at least 1).
        retry_after_secs: u64,
        /// Whether the global budget, rather than the client's own, is the reason.
        global: bool,
    },
}

impl Verdict {
    /// Whether the verdict is [`Verdict::Blocked`].
    #[must_use]
    pub const fn is_blocked(&self) -> bool {
        matches!(self, Self::Blocked { .. })
    }
}

#[derive(Debug, Default)]
struct State {
    per_client: HashMap<String, VecDeque<i64>>,
    global: VecDeque<i64>,
    calls_since_sweep: u32,
}

/// Sliding-window failure budgets, per client and global.
#[derive(Debug)]
pub struct RateLimiter {
    per_client_limit: u32,
    global_limit: u32,
    state: Mutex<State>,
}

/// Clients tracked before stale entries are swept on every call.
const SWEEP_EVERY: u32 = 256;

impl RateLimiter {
    /// A limiter with the given budgets; `0` disables a budget.
    #[must_use]
    pub fn new(per_client_limit: u32, global_limit: u32) -> Self {
        Self { per_client_limit, global_limit, state: Mutex::new(State::default()) }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        // A poisoned lock only means another request panicked while counting; the counters
        // are still usable.
        self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Whether `client` may submit a code now. Records nothing.
    #[must_use]
    pub fn check(&self, client: &str, now: i64) -> Verdict {
        let mut state = self.lock();
        self.verdict(&mut state, client, now)
    }

    /// Records one unknown code from `client` and returns the verdict that applies to this
    /// very attempt: [`Verdict::Blocked`] with `global: true` if the global budget was
    /// already exhausted (the wrong guess gets `429`), otherwise [`Verdict::Allowed`] with
    /// the attempts the client has left *after* this one.
    pub fn record_failure(&self, client: &str, now: i64) -> Verdict {
        let mut state = self.lock();
        prune(&mut state.global, now);
        let under_attack = self.global_limit > 0 && state.global.len() >= self.global_limit as usize;
        let global_retry = retry_after(state.global.front().copied(), now);

        state.global.push_back(now);
        let global_cap = self.global_limit.max(1) as usize;
        while state.global.len() > global_cap {
            state.global.pop_front();
        }
        let entry = state.per_client.entry(client.to_owned()).or_default();
        prune(entry, now);
        entry.push_back(now);
        let client_cap = self.per_client_limit.max(1) as usize;
        while entry.len() > client_cap {
            entry.pop_front();
        }
        let used = u32::try_from(entry.len()).unwrap_or(u32::MAX);
        self.sweep(&mut state, now);

        if under_attack {
            return Verdict::Blocked { retry_after_secs: global_retry, global: true };
        }
        Verdict::Allowed {
            remaining: (self.per_client_limit > 0).then(|| self.per_client_limit.saturating_sub(used)),
        }
    }

    fn verdict(&self, state: &mut State, client: &str, now: i64) -> Verdict {
        prune(&mut state.global, now);
        let under_attack = self.global_limit > 0 && state.global.len() >= self.global_limit as usize;
        let global_retry = retry_after(state.global.front().copied(), now);
        let (used, oldest) = match state.per_client.get_mut(client) {
            Some(entry) => {
                prune(entry, now);
                (u32::try_from(entry.len()).unwrap_or(u32::MAX), entry.front().copied())
            }
            None => (0, None),
        };
        if self.per_client_limit > 0 && used >= self.per_client_limit {
            return Verdict::Blocked { retry_after_secs: retry_after(oldest, now), global: false };
        }
        if under_attack && used > 0 {
            return Verdict::Blocked { retry_after_secs: global_retry, global: true };
        }
        Verdict::Allowed {
            remaining: (self.per_client_limit > 0).then(|| self.per_client_limit - used),
        }
    }

    /// Drops clients whose failures have all left the window, every few calls.
    fn sweep(&self, state: &mut State, now: i64) {
        state.calls_since_sweep += 1;
        if state.calls_since_sweep < SWEEP_EVERY {
            return;
        }
        state.calls_since_sweep = 0;
        state.per_client.retain(|_, entry| {
            prune(entry, now);
            !entry.is_empty()
        });
    }

    /// Number of clients currently tracked (for tests and metrics).
    #[must_use]
    pub fn tracked_clients(&self) -> usize {
        self.lock().per_client.len()
    }
}

/// Removes timestamps that have left the window (or lie in the future after a clock jump).
fn prune(entries: &mut VecDeque<i64>, now: i64) {
    while entries.front().is_some_and(|t| now - *t >= WINDOW_SECS) {
        entries.pop_front();
    }
    entries.retain(|t| *t <= now);
}

/// Seconds until the entry at `oldest` leaves the window.
fn retry_after(oldest: Option<i64>, now: i64) -> u64 {
    let wait = oldest.map_or(1, |t| t + WINDOW_SECS - now);
    u64::try_from(wait.clamp(1, WINDOW_SECS)).unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    const T0: i64 = 1_800_000_000;

    #[test]
    fn a_client_is_blocked_after_its_budget_and_recovers_when_the_window_slides() {
        let limiter = RateLimiter::new(3, 0);
        assert_eq!(limiter.check("a", T0), Verdict::Allowed { remaining: Some(3) });
        assert_eq!(limiter.record_failure("a", T0), Verdict::Allowed { remaining: Some(2) });
        assert_eq!(limiter.record_failure("a", T0 + 10), Verdict::Allowed { remaining: Some(1) });
        assert_eq!(limiter.check("a", T0 + 11), Verdict::Allowed { remaining: Some(1) });
        assert_eq!(limiter.record_failure("a", T0 + 20), Verdict::Allowed { remaining: Some(0) });
        // Blocked until the first failure (T0) leaves the ten-minute window.
        assert_eq!(
            limiter.check("a", T0 + 21),
            Verdict::Blocked { retry_after_secs: 579, global: false }
        );
        assert_eq!(
            limiter.check("a", T0 + 599),
            Verdict::Blocked { retry_after_secs: 1, global: false }
        );
        assert_eq!(limiter.check("a", T0 + 600), Verdict::Allowed { remaining: Some(1) });
        assert_eq!(limiter.check("a", T0 + 620), Verdict::Allowed { remaining: Some(3) });
        // Other clients are unaffected throughout.
        assert_eq!(limiter.check("b", T0 + 21), Verdict::Allowed { remaining: Some(3) });
    }

    #[test]
    fn checking_never_consumes_the_budget() {
        let limiter = RateLimiter::new(2, 5);
        for _ in 0..100 {
            assert_eq!(limiter.check("a", T0), Verdict::Allowed { remaining: Some(2) });
        }
        assert_eq!(limiter.tracked_clients(), 0);
    }

    #[test]
    fn the_global_budget_turns_wrong_guesses_into_429_for_everyone() {
        let limiter = RateLimiter::new(10, 4);
        for i in 0..4 {
            let verdict = limiter.record_failure(&format!("bot-{i}"), T0 + i);
            assert_eq!(verdict, Verdict::Allowed { remaining: Some(9) });
        }
        // The budget is spent: a fresh client may still present a code (it could be a
        // valid one)…
        assert_eq!(limiter.check("fresh", T0 + 5), Verdict::Allowed { remaining: Some(10) });
        // …but a wrong guess from anybody now gets 429,
        assert_eq!(
            limiter.record_failure("fresh", T0 + 5),
            Verdict::Blocked { retry_after_secs: 595, global: true }
        );
        // and a client that already guessed wrong is not even looked up.
        assert_eq!(
            limiter.check("bot-0", T0 + 6),
            Verdict::Blocked { retry_after_secs: 595, global: true }
        );
        assert!(limiter.check("fresh", T0 + 6).is_blocked());
        // Sustained guessing keeps the service in that state…
        assert!(limiter.record_failure("bot-9", T0 + 300).is_blocked());
        // …and it recovers once the window has slid past the burst.
        assert_eq!(limiter.check("bot-0", T0 + 601), Verdict::Allowed { remaining: Some(10) });
        assert_eq!(limiter.record_failure("late", T0 + 901), Verdict::Allowed { remaining: Some(9) });
    }

    #[test]
    fn zero_disables_a_budget() {
        let limiter = RateLimiter::new(0, 0);
        for i in 0..1_000 {
            assert_eq!(limiter.record_failure("a", T0 + i % 7), Verdict::Allowed { remaining: None });
        }
        assert_eq!(limiter.check("a", T0 + 8), Verdict::Allowed { remaining: None });

        let global_only = RateLimiter::new(0, 2);
        assert!(!global_only.record_failure("a", T0).is_blocked());
        assert!(!global_only.record_failure("a", T0).is_blocked());
        assert!(global_only.record_failure("a", T0).is_blocked());
    }

    #[test]
    fn memory_stays_bounded_under_a_flood() {
        let limiter = RateLimiter::new(5, 50);
        for i in 0..20_000i64 {
            let _ = limiter.record_failure(&format!("10.0.{}.{}", i / 250, i % 250), T0 + i / 100);
        }
        {
            let state = limiter.lock();
            assert!(state.global.len() <= 50);
            assert!(state.per_client.values().all(|entry| entry.len() <= 5));
        }
        // Long after the flood a single call sweeps (almost) everything away.
        for i in 0..i64::from(SWEEP_EVERY) {
            let _ = limiter.record_failure("one-client", T0 + 10_000 + i);
        }
        assert!(limiter.tracked_clients() <= 2, "{}", limiter.tracked_clients());
    }

    #[test]
    fn a_clock_jumping_backwards_does_not_lock_clients_out_forever() {
        let limiter = RateLimiter::new(2, 0);
        let _ = limiter.record_failure("a", T0 + 5_000);
        let _ = limiter.record_failure("a", T0 + 5_000);
        assert!(limiter.check("a", T0 + 5_001).is_blocked());
        // The clock is set back: entries from "the future" are dropped.
        assert_eq!(limiter.check("a", T0), Verdict::Allowed { remaining: Some(2) });
    }
}
