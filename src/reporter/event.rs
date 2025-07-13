use super::ErrorReport;
use crate::size;
use std::{fs::Metadata, path::Path};

/// Report trigger event.
#[derive(Debug)]
pub enum Event<'a, Size: size::Size> {
    ReceiveData(Size),
    EncounterError(ErrorReport<'a>),
    EncounterHardlink(EncounterHardlink<'a, Size>),
}

/// Data of [`Event::EncounterHardlink`].
#[derive(Debug, Clone, Copy)]
pub struct EncounterHardlink<'a, Size: size::Size> {
    /// Path of the detected hardlink.
    pub path: &'a Path,
    /// Stats of the detected hardlink.
    pub stats: &'a Metadata,
    /// Size of the file.
    pub size: Size,
    /// Number of links, including this one.
    pub links: u64,
}
