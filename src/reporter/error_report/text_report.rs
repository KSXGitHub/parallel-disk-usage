use super::ErrorReport;
use std::fmt::{Display, Error, Formatter};

impl<'a> ErrorReport<'a> {
    /// Prints error message in form of a line of text to stderr.
    pub const TEXT: fn(ErrorReport) = |report| eprint!("{}", TextReport(report));
}

/// Wrapper around [`ErrorReport`] that `impl`s [`Display`]
/// to make `ErrorReport::text_report` testable.
struct TextReport<'a>(ErrorReport<'a>);

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

#[cfg(test)]
use super::Operation;
#[cfg(test)]
use std::{io, path::PathBuf};

#[test]
fn test() {
    let report = ErrorReport {
        operation: Operation::ReadDirectory,
        path: &PathBuf::from("path/to/a/directory"),
        error: io::Error::new(
            io::ErrorKind::NotFound,
            "Something goes wrong (os error 420)",
        ),
    };
    let actual = TextReport(report).to_string();
    let expected =
        "\r[error] read_dir \"path/to/a/directory\": Something goes wrong (os error 420)\n";
    assert_eq!(actual, expected);
}
