use super::{iter::Item as IterItem, reflection::ReflectionEntry, HardlinkList, Reflection};
use crate::size;
use derive_more::{Add, AddAssign, Sum};
use std::{
    cmp::Ordering,
    fmt::{self, Display},
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Summary from [`HardlinkList`] or [`Reflection`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Add, AddAssign, Sum)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub struct Summary<Size> {
    /// Number of shared inodes, each with more than 1 links (i.e. `nlink > 1`).
    pub inodes: usize,

    /// Number of [shared inodes](Self::inodes) that don't have links outside the measured tree.
    ///
    /// This number is expected to be less than or equal to [`Self::inodes`].
    pub owned_inodes: usize,

    /// Totality of the numbers of links of all [shared inodes](Self::inodes).
    pub all_links: u64,

    /// Total number of links of [shared inodes](Self::inodes) that were detected within the measured tree.
    ///
    /// This number is expected to be less than or equal to [`Self::all_links`].
    pub detected_links: usize,

    /// Total number of links of [shared inodes](Self::inodes) that don't have links outside the measured tree.
    ///
    /// This number is expected to be less than or equal to [`Self::all_links`].
    pub owned_links: usize,

    /// Totality of the sizes of all [shared inodes](Self::inodes).
    pub shared_size: Size,

    /// Totality of the sizes of all [shared inodes](Self::inodes) that don't have links outside the measured tree.
    ///
    /// This number is expected to be less than or equal to [`Self::all_links`].
    pub owned_shared_size: Size,
}

/// Ability to summarize into a [`Summary`].
pub trait SummarizeHardlinks<Size>: Sized {
    /// The result of [`SummarizeHardlinks::summarize_hardlinks`].
    type Summary;
    /// Summarize into a summary of shared links and size.
    fn summarize_hardlinks(self) -> Self::Summary;
}

/// Summary of a single unique file.
#[derive(Debug, Clone, Copy)]
pub struct SingleInodeSummary<Size> {
    /// Total number of all links to the file.
    links: u64,
    /// Number of detected links to the file.
    paths: usize,
    /// Size of the file.
    size: Size,
}

impl<Size, Iter> SummarizeHardlinks<Size> for Iter
where
    Size: size::Size,
    Iter: IntoIterator,
    Iter::Item: SummarizeHardlinks<Size>,
    <Iter::Item as SummarizeHardlinks<Size>>::Summary: Into<SingleInodeSummary<Size>>,
{
    type Summary = Summary<Size>;
    fn summarize_hardlinks(self) -> Self::Summary {
        let mut summary = Summary::default();
        for item in self {
            let SingleInodeSummary { links, paths, size } = item.summarize_hardlinks().into();
            summary.inodes += 1;
            summary.all_links += links;
            summary.detected_links += paths;
            summary.shared_size += size;
            match links.cmp(&(paths as u64)) {
                Ordering::Greater => {}
                Ordering::Equal => {
                    summary.owned_inodes += 1;
                    summary.owned_links += paths; // `links` and `paths` are both fine, but `paths` doesn't require type cast
                    summary.owned_shared_size += size;
                }
                Ordering::Less => {
                    panic!("Impossible! Total of nlink ({links}) is less than detected paths ({paths}). Something must have gone wrong!");
                }
            }
        }
        summary
    }
}

impl<Size: size::Size> HardlinkList<Size> {
    /// Create summary for the shared links and size.
    pub fn summarize(&self) -> Summary<Size> {
        self.iter().summarize_hardlinks()
    }
}

impl<Size: size::Size> SummarizeHardlinks<Size> for &HardlinkList<Size> {
    type Summary = Summary<Size>;
    fn summarize_hardlinks(self) -> Self::Summary {
        self.summarize()
    }
}

impl<Size: size::Size> Reflection<Size> {
    /// Create summary for the shared links and size.
    pub fn summarize(&self) -> Summary<Size> {
        self.iter().summarize_hardlinks()
    }
}

impl<Size: size::Size> SummarizeHardlinks<Size> for &Reflection<Size> {
    type Summary = Summary<Size>;
    fn summarize_hardlinks(self) -> Self::Summary {
        self.summarize()
    }
}

/// Return type of [`Summary::display`] which implements [`Display`].
#[derive(Debug, Clone, Copy)]
pub struct SummaryDisplay<'a, Size: size::Size> {
    format: Size::DisplayFormat,
    summary: &'a Summary<Size>,
}

impl<Size: size::Size> Display for SummaryDisplay<'_, Size> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let SummaryDisplay { format, summary } = self;
        let Summary {
            inodes,
            owned_inodes,
            all_links,
            detected_links,
            owned_links,
            shared_size,
            owned_shared_size,
        } = summary;

        let shared_size = shared_size.display(*format);
        let owned_shared_size = owned_shared_size.display(*format);

        macro_rules! ln {
            ($($args:tt)*) => {
                writeln!(f, $($args)*)
            };
        }

        write!(f, "Hardlinks detected! ")?;
        if owned_inodes == inodes {
            ln!("No files have links outside this tree")?;
            ln!("* Number of shared inodes: {inodes}")?;
            ln!("* Total number of links: {all_links}")?;
            ln!("* Total shared size: {shared_size}")?;
        } else if owned_inodes == &0 {
            ln!("All hardlinks within this tree have links without")?;
            ln!("* Number of shared inodes: {inodes}")?;
            ln!("* Total number of links: {all_links} total, {detected_links} detected")?;
            ln!("* Total shared size: {shared_size}")?;
        } else {
            ln!("Some files have links outside this tree")?;
            ln!("* Number of shared inodes: {inodes} total, {owned_inodes} exclusive")?;
            ln!("* Total number of links: {all_links} total, {detected_links} detected, {owned_links} exclusive")?;
            ln!("* Total shared size: {shared_size} total, {owned_shared_size} exclusive")?;
        }

        Ok(())
    }
}

impl<Size: size::Size> Summary<Size> {
    /// Turns this [`Summary`] into something [displayable](Display).
    pub fn display(&self, format: Size::DisplayFormat) -> SummaryDisplay<Size> {
        SummaryDisplay {
            format,
            summary: self,
        }
    }
}

impl<Size: Copy> SummarizeHardlinks<Size> for ReflectionEntry<Size> {
    type Summary = SingleInodeSummary<Size>;
    fn summarize_hardlinks(self) -> Self::Summary {
        (&self).summarize_hardlinks()
    }
}

impl<Size: Copy> SummarizeHardlinks<Size> for &ReflectionEntry<Size> {
    type Summary = SingleInodeSummary<Size>;
    fn summarize_hardlinks(self) -> Self::Summary {
        SingleInodeSummary {
            links: self.links,
            paths: self.paths.len(),
            size: self.size,
        }
    }
}

impl<'a, Size: Copy> SummarizeHardlinks<Size> for IterItem<'a, Size> {
    type Summary = SingleInodeSummary<Size>;
    fn summarize_hardlinks(self) -> Self::Summary {
        (&self).summarize_hardlinks()
    }
}

impl<'a, Size: Copy> SummarizeHardlinks<Size> for &IterItem<'a, Size> {
    type Summary = SingleInodeSummary<Size>;
    fn summarize_hardlinks(self) -> Self::Summary {
        SingleInodeSummary {
            links: self.links(),
            paths: self.paths().len(),
            size: *self.size(),
        }
    }
}
