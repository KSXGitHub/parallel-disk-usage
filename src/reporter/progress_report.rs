use crate::size::Size;
use std::fmt::Write;

/// Scan progress.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProgressReport<Data: Size> {
    /// Number of known items.
    pub known_items: u64,
    /// Number of scanned items.
    pub scanned_items: u64,
    /// Total size of scanned items.
    pub scanned_total: Data,
    /// Number of occurred errors.
    pub errors: u64,
}

impl<Data: Size + Into<u64>> ProgressReport<Data> {
    /// Print progress to stderr.
    pub const TEXT: fn(Self) = |report| {
        let ProgressReport {
            known_items,
            scanned_items,
            scanned_total,
            errors,
        } = report;
        let mut text = String::new();
        write!(
            text,
            "\r(known {known}, scanned {scanned}, total {total}",
            known = known_items,
            scanned = scanned_items,
            total = scanned_total.into(),
        )
        .unwrap();
        if errors != 0 {
            write!(text, ", erred {}", errors).unwrap();
        }
        write!(text, ")").unwrap();
        eprint!("{}", text);
    };
}
