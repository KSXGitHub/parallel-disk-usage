/// Operation that caused the error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    /// Error is caused by calling [`std::fs::symlink_metadata`].
    SymlinkMetadata,
    /// Error is caused by calling [`std::fs::read_dir`].
    ReadDirectory,
    /// Error when trying to access [`std::fs::DirEntry`] of one of the element of [`std::fs::read_dir`].
    AccessEntry,
}

impl Operation {
    /// Get name of the operation.
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
