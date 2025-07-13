use super::ErrorReport;
use crate::size;
use std::{fs::Metadata, path::Path};

/// Report trigger event.
#[derive(Debug)]
#[non_exhaustive]
pub enum Event<'a, Size: size::Size> {
    ReceiveData(Size),
    EncounterError(ErrorReport<'a>),
    DetectHardlink(HardlinkDetection<'a, Size>),
}

/// Data of [`Event::DetectHardlink`].
#[derive(Debug, Clone, Copy)]
pub struct HardlinkDetection<'a, Size: size::Size> {
    /// Path of the detected hardlink.
    pub path: &'a Path,
    /// Stats of the detected hardlink.
    pub stats: &'a Metadata,
    /// Size of the file.
    pub size: Size,
    /// Number of links, including this one.
    pub links: u64,
}
