use crate::{size, status_board::GLOBAL_STATUS_BOARD};
use derive_setters::Setters;
use std::fmt::Write;

/// Scan progress.
#[derive(Debug, Default, Setters, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[setters(prefix = "with_")]
pub struct ProgressReport<Size: size::Size> {
    /// Number of scanned items.
    pub items: u64,
    /// Total size of scanned items.
    pub total: Size,
    /// Number of occurred errors.
    pub errors: u64,
    /// Total number of detected hardlinks.
    pub linked: u64,
    /// Total size of detected hardlinks.
    pub shared: Size,
}

impl<Size: size::Size + Into<u64>> ProgressReport<Size> {
    /// Maximum length by which the progress text may extend.
    ///
    /// This constant is used as capacity in [`Self::TEXT`] to prevent
    /// performance penalty from string resizing.
    ///
    /// The value of this constant is made correct by a unit test.
    const TEXT_MAX_LEN: usize = 145;

    /// Create a text to be used in [`Self::TEXT`].
    fn text(self) -> String {
        let ProgressReport {
            items,
            total,
            errors,
            linked,
            shared,
        } = self;
        let mut text = String::with_capacity(Self::TEXT_MAX_LEN);
        let total: u64 = total.into();
        write!(text, "\r(scanned {items}, total {total}").unwrap();
        if linked != 0 {
            write!(text, ", linked {linked}").unwrap();
        }
        let shared: u64 = shared.into();
        if shared != 0 {
            write!(text, ", shared {shared}").unwrap();
        }
        if errors != 0 {
            write!(text, ", erred {errors}").unwrap();
        }
        text.push(')');
        text
    }

    /// Print progress to stderr.
    pub const TEXT: fn(Self) = |report| {
        GLOBAL_STATUS_BOARD.temporary_message(&report.text());
    };
}

#[test]
fn text_max_len() {
    use crate::size::Bytes;
    let correct_value = ProgressReport::<Bytes> {
        items: u64::MAX,
        total: u64::MAX.into(),
        errors: u64::MAX,
        linked: u64::MAX,
        shared: u64::MAX.into(),
    }
    .text()
    .len();
    assert_eq!(ProgressReport::<Bytes>::TEXT_MAX_LEN, correct_value);
}
