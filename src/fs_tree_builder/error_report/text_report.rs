use super::ErrorReport;
use std::fmt::{Display, Error, Formatter};

impl<'a> ErrorReport<'a> {
    /// Prints error message in form of a line of text to stderr
    pub fn text_report(&self) {
        eprint!("{}", TextReport(self));
    }
}

/// Wrapper around [`ErrorReport`] that `impl`s [`Display`]
/// to make `ErrorReport::text_report` testable
struct TextReport<'a>(&'a ErrorReport<'a>);

impl<'a> Display for TextReport<'a> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        writeln!(
            formatter,
            "\r[error] {operation} {path:?}: {error}",
            operation = self.0.operation.name(),
            path = self.0.path,
            error = self.0.error,
        )
    }
}
