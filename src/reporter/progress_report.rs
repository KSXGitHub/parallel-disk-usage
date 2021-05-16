use crate::{size::Size, status_board::GLOBAL_STATUS_BOARD};
use std::fmt::Write;

/// Scan progress.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProgressReport<Data: Size> {
    /// Number of scanned items.
    pub items: u64,
    /// Total size of scanned items.
    pub total: Data,
    /// Number of occurred errors.
    pub errors: u64,
}

impl<Data: Size + Into<u64>> ProgressReport<Data> {
    /// Print progress to stderr.
    pub const TEXT: fn(Self) = |report| {
        let ProgressReport {
            items,
            total,
            errors,
        } = report;
        let mut text = String::new();
        write!(
            text,
            "\r(scanned {items}, total {total}",
            items = items,
            total = total.into(),
        )
        .unwrap();
        if errors != 0 {
            write!(text, ", erred {}", errors).unwrap();
        }
        write!(text, ")").unwrap();
        GLOBAL_STATUS_BOARD.temporary_message(&text);
    };
}
