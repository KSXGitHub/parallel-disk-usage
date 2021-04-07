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

/// Operation that caused the error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    /// Error is caused by calling [`std::fs::symlink_metadata`]
    SymlinkMetadata,
    /// Error is caused by calling [`std::fs::read_dir`]
    ReadDirectory,
    /// Error when trying to access [`std::fs::DirEntry`] of one of the element of [`std::fs::read_dir`]
    AccessEntry,
}

mod implementations;
