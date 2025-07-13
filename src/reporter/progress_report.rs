use crate::{size, status_board::GLOBAL_STATUS_BOARD};
use std::fmt::Write;

/// Scan progress.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProgressReport<Size: size::Size> {
    /// Number of scanned items.
    pub items: u64,
    /// Total size of scanned items.
    pub total: Size,
    /// Number of occurred errors.
    pub errors: u64,
}

impl<Size: size::Size + Into<u64>> ProgressReport<Size> {
    /// Print progress to stderr.
    pub const TEXT: fn(Self) = |report| {
        let ProgressReport {
            items,
            total,
            errors,
        } = report;
        let mut text = String::new();
        let total: u64 = total.into();
        write!(text, "\r(scanned {items}, total {total}").unwrap();
        if errors != 0 {
            write!(text, ", erred {errors}").unwrap();
        }
        text.push(')');
        GLOBAL_STATUS_BOARD.temporary_message(&text);
    };
}
