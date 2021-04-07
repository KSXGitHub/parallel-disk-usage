pub mod operation;

pub use operation::Operation;

use std::{io::Error, path::Path};

/// Information regarding a filesystem error
#[derive(Debug)]
pub struct ErrorReport<'a> {
    /// Operation that caused the error
    pub operation: Operation,
    /// Path where the error occurred
    pub path: &'a Path,
    /// The error
    pub error: Error,
}

impl<'a> ErrorReport<'a> {
    /// Do nothing
    pub fn silent(&self) {}

    /// Prints error message in form of a line of text to stderr
    pub fn text_report(&self) {
        eprintln!(
            "\r[error] {operation} {path:?}: {error}",
            operation = self.operation.name(),
            path = self.path,
            error = self.error,
        );
    }
}
