//! Containment of panics raised inside the font libraries.
//!
//! `read-fonts` and `harfrust` are written not to panic on malformed fonts — in
//! **release** builds. In builds with debug assertions and arithmetic overflow checks
//! (the `dev` and `test` profiles), a hostile font can still trip one of their internal
//! `debug_assert!`s or an overflow check (for instance a table that declares 2^32 − 1
//! records). Every entry point of this crate that hands font data to those libraries
//! therefore runs inside [`contain`], and degrades to an error or an empty result
//! instead of unwinding into the caller.
//!
//! Limits: on targets or profiles where panics abort instead of unwinding
//! (`wasm32-unknown-unknown`, `panic = "abort"`), nothing can be caught. Release builds
//! of those targets have debug assertions off, which is the configuration the font
//! libraries are fuzzed in. The process-wide panic hook still runs, so a contained
//! panic is reported on stderr.

use std::panic::{AssertUnwindSafe, catch_unwind};

/// Runs `f`; returns `None` if it panicked.
///
/// Used only around code that reads immutable font bytes and builds fresh values, so
/// there is no shared state a panic could leave half-updated.
pub(crate) fn contain<T>(f: impl FnOnce() -> T) -> Option<T> {
    catch_unwind(AssertUnwindSafe(f)).ok()
}

#[cfg(test)]
mod tests {
    use super::contain;

    #[test]
    fn values_pass_through_and_panics_become_none() {
        assert_eq!(contain(|| 7), Some(7));
        let caught: Option<u8> =
            contain(|| panic!("contained on purpose (expected in test output)"));
        // `panic = "abort"` test profiles would not get here at all.
        assert_eq!(caught, None);
    }
}
