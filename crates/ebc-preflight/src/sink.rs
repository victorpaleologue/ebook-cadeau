//! Collects findings, dropping duplicates and capping floods, and tracks the work budget
//! that keeps hostile documents from hanging the checker.

use std::collections::{HashMap, HashSet};

use crate::codes;
use crate::report::{Finding, Severity};

/// At most this many distinct findings are kept per code and page; one more line then says
/// that the rest were omitted.
const MAX_PER_CODE_AND_PAGE: u32 = 20;

/// Accumulates findings.
#[derive(Debug, Default)]
pub(crate) struct Sink {
    findings: Vec<Finding>,
    seen: HashSet<(&'static str, Option<u32>, String)>,
    counts: HashMap<(&'static str, Option<u32>), u32>,
}

impl Sink {
    pub(crate) fn push(
        &mut self,
        severity: Severity,
        code: &'static str,
        page: Option<u32>,
        message: impl Into<String>,
    ) {
        let message = message.into();
        if !self.seen.insert((code, page, message.clone())) {
            return;
        }
        let count = self.counts.entry((code, page)).or_insert(0);
        *count += 1;
        if *count <= MAX_PER_CODE_AND_PAGE {
            self.findings.push(Finding { severity, code, page, message });
        } else if *count == MAX_PER_CODE_AND_PAGE + 1 {
            self.findings.push(Finding {
                severity,
                code,
                page,
                message: format!("further {code} findings are omitted"),
            });
        }
    }

    pub(crate) fn error(&mut self, code: &'static str, page: Option<u32>, msg: impl Into<String>) {
        self.push(Severity::Error, code, page, msg);
    }

    pub(crate) fn warning(&mut self, code: &'static str, page: Option<u32>, msg: impl Into<String>) {
        self.push(Severity::Warning, code, page, msg);
    }

    pub(crate) fn info(&mut self, code: &'static str, page: Option<u32>, msg: impl Into<String>) {
        self.push(Severity::Info, code, page, msg);
    }

    /// The findings: document-level first, then by page; original order within each group.
    pub(crate) fn into_sorted(mut self) -> Vec<Finding> {
        self.findings.sort_by_key(|f| f.page.map_or(0, |p| u64::from(p) + 1));
        self.findings
    }
}

/// Work limits shared by all pages of one document.
#[derive(Debug)]
pub(crate) struct Budget {
    /// Content stream tokens still allowed.
    operations: u64,
    /// Decoded content stream bytes still allowed.
    bytes: usize,
    /// Form XObject and tiling pattern executions still allowed.
    nested_runs: u32,
    reported: bool,
}

impl Budget {
    /// Content stream tokens interpreted per document before giving up.
    pub(crate) const MAX_OPERATIONS: u64 = 20_000_000;
    /// Decoded content stream bytes per document.
    pub(crate) const MAX_CONTENT_BYTES: usize = 128 * 1024 * 1024;
    /// Decoded size of a single stream.
    pub(crate) const MAX_STREAM_BYTES: usize = 32 * 1024 * 1024;

    /// Form XObject and tiling pattern executions per document. Real documents draw a few
    /// thousand at most; recursion bombs draw billions.
    pub(crate) const MAX_NESTED_RUNS: u32 = 50_000;

    pub(crate) fn new() -> Self {
        Self {
            operations: Self::MAX_OPERATIONS,
            bytes: Self::MAX_CONTENT_BYTES,
            nested_runs: Self::MAX_NESTED_RUNS,
            reported: false,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_operations(operations: u64) -> Self {
        Self { operations, ..Self::new() }
    }

    /// Consumes one operation; `false` when the budget is exhausted.
    pub(crate) fn take_operation(&mut self) -> bool {
        if self.operations == 0 {
            return false;
        }
        self.operations -= 1;
        true
    }

    /// Consumes one form or pattern execution; `false` when the budget is exhausted.
    pub(crate) fn take_nested_run(&mut self) -> bool {
        if self.nested_runs == 0 {
            return false;
        }
        self.nested_runs -= 1;
        true
    }

    /// The decode limit for the next stream.
    pub(crate) fn stream_limit(&self) -> usize {
        self.bytes.min(Self::MAX_STREAM_BYTES)
    }

    /// Charges decoded bytes against the budget.
    pub(crate) fn charge_bytes(&mut self, n: usize) {
        self.bytes = self.bytes.saturating_sub(n);
    }

    /// Reports (once per document) that analysis stopped early.
    pub(crate) fn report_exhausted(&mut self, sink: &mut Sink, page: Option<u32>, what: &str) {
        if !self.reported {
            self.reported = true;
            sink.error(
                codes::LIMIT_EXCEEDED,
                page,
                format!("{what}: the document is too complex to analyse completely, the report is partial"),
            );
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn duplicates_are_dropped_and_floods_capped() {
        let mut sink = Sink::default();
        sink.error(codes::COLOR_RGB, Some(1), "same");
        sink.error(codes::COLOR_RGB, Some(1), "same");
        sink.error(codes::COLOR_RGB, Some(2), "same");
        for i in 0..100 {
            sink.warning(codes::IMAGE_INVALID, Some(1), format!("image {i}"));
        }
        sink.info(codes::NO_TITLE, None, "doc level, pushed last");
        let findings = sink.into_sorted();
        assert_eq!(findings[0].page, None, "document-level findings come first");
        let rgb: Vec<_> = findings.iter().filter(|f| f.code == codes::COLOR_RGB).collect();
        assert_eq!(rgb.len(), 2);
        let images: Vec<_> = findings.iter().filter(|f| f.code == codes::IMAGE_INVALID).collect();
        assert_eq!(images.len(), 21);
        assert_eq!(images[20].message, "further IMAGE_INVALID findings are omitted");
        let pages: Vec<_> = findings.iter().map(|f| f.page).collect();
        let mut sorted = pages.clone();
        sorted.sort();
        assert_eq!(pages, sorted);
    }

    #[test]
    fn budget_runs_out_once() {
        let mut budget = Budget::with_operations(2);
        let mut sink = Sink::default();
        assert!(budget.take_operation());
        assert!(budget.take_operation());
        assert!(!budget.take_operation());
        budget.report_exhausted(&mut sink, Some(3), "page content");
        budget.report_exhausted(&mut sink, Some(4), "page content");
        let findings = sink.into_sorted();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code, codes::LIMIT_EXCEEDED);
        assert_eq!(findings[0].severity, Severity::Error);
        budget.charge_bytes(usize::MAX);
        assert_eq!(budget.stream_limit(), 0);
        let mut budget = Budget::new();
        let runs = (0..Budget::MAX_NESTED_RUNS + 10).filter(|_| budget.take_nested_run()).count();
        assert_eq!(runs, Budget::MAX_NESTED_RUNS as usize);
    }
}
