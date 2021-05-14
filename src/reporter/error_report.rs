pub mod operation;

pub use operation::Operation;

use std::{io::Error, path::Path};

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
    pub(crate) const SILENT: fn(ErrorReport) = |report| report.silent();
    pub(crate) const TEXT: fn(ErrorReport) = |report| report.text_report();

    /// Do nothing.
    pub fn silent(&self) {}
}

mod text_report;
