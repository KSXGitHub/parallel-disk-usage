#![cfg_attr(
    dylint_lib = "perfectionist",
    expect(
        perfectionist::import_grouping_mismatch,
        reason = "pub use re-exports are kept in their own group; see #442"
    )
)]

pub mod operation;

pub use operation::Operation;

use std::io::Error;
use std::path::Path;

/// Information regarding a filesystem error.
#[derive(Debug)]
pub struct ErrorReport<'a> {
    /// Operation that caused the error.
    pub operation: Operation,
    /// Path where the error occurred.
    pub path: &'a Path,
    /// The error.
    pub error: Error,
}

impl<'a> ErrorReport<'a> {
    /// Do nothing.
    pub const SILENT: fn(ErrorReport) = |_| {};
}

mod text_report;
