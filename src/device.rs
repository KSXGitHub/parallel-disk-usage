/// Whether to cross device boundary into a different filesystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceBoundary {
    Cross,
    Stay,
}

impl DeviceBoundary {
    /// Derive device boundary from `--one-file-system`.
    #[cfg(feature = "cli")]
    pub(crate) fn from_one_file_system(one_file_system: bool) -> Self {
        match one_file_system {
            false => DeviceBoundary::Cross,
            true => DeviceBoundary::Stay,
        }
    }
}
