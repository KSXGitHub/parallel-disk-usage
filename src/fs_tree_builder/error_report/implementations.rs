use super::{ErrorReport, Operation};

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

impl Operation {
    /// Get name of the operation
    pub const fn name(self) -> &'static str {
        use Operation::*;
        match self {
            SymlinkMetadata => "symlink_metadata",
            ReadDirectory => "read_dir",
            AccessEntry => "access entry",
        }
    }
}

#[cfg(test)]
mod test_operation {
    use super::*;

    macro_rules! name_display {
        ($name:ident, $variant:ident, $text:literal) => {
            #[test]
            fn $name() {
                assert_eq!(Operation::$variant.name(), $text);
            }
        };
    }

    name_display!(symlink_metadata, SymlinkMetadata, "symlink_metadata");
    name_display!(read_directory, ReadDirectory, "read_dir");
    name_display!(access_entry, AccessEntry, "access entry");
}
