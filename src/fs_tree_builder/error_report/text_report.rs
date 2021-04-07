use super::ErrorReport;

impl<'a> ErrorReport<'a> {
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
