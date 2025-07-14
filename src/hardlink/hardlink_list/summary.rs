use super::{iter::Item as IterItem, reflection::ReflectionEntry, HardlinkList, Reflection};
use crate::size;
use derive_more::{Add, AddAssign, Sum};
use std::fmt::{self, Display};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Summary from [`HardlinkList`] or [`Reflection`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Add, AddAssign, Sum)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub struct Summary<Size> {
    /// Number of unique files, each with more than 1 links.
    pub inodes: usize,
    /// Total number of links of all the unique files as counted by [`Summary::inodes`].
    pub links: usize,
    /// Total size of all the unique files.
    pub size: Size,
}

impl<Size> Summary<Size> {
    /// Create a new summary.
    pub fn new(inodes: usize, links: usize, size: Size) -> Self {
        Summary {
            inodes,
            links,
            size,
        }
    }
}

/// Ability to summarize into a [`Summary`].
pub trait SummarizeHardlinks<Size>: Sized {
    /// Summarize into a summary of shared links and size.
    fn summarize_hardlinks(self) -> Summary<Size>;
}

/// Summary of a single unique file.
#[derive(Debug, Clone, Copy)]
pub struct SingleInodeSummary<Size> {
    /// Number of detected links to the file.
    links: usize,
    /// Size of the file.
    size: Size,
}

impl<Size, Iter> SummarizeHardlinks<Size> for Iter
where
    Size: size::Size,
    Iter: IntoIterator,
    Iter::Item: Into<SingleInodeSummary<Size>>,
{
    fn summarize_hardlinks(self) -> Summary<Size> {
        let mut summary = Summary::default();
        for item in self {
            let SingleInodeSummary { links, size } = item.into();
            if links <= 1 {
                continue;
            }
            summary.inodes += 1;
            summary.links += links;
            summary.size += size;
        }
        summary
    }
}

/// Summarize an iterator.
impl<Size, Item> FromIterator<Item> for Summary<Size>
where
    Size: size::Size,
    Item: Into<SingleInodeSummary<Size>>,
{
    /// Create a summary of shared links and size from an iterator.
    fn from_iter<Iter: IntoIterator<Item = Item>>(iter: Iter) -> Self {
        iter.summarize_hardlinks()
    }
}

impl<Size: size::Size> HardlinkList<Size> {
    /// Create summary for the shared links and size.
    pub fn summarize(&self) -> Summary<Size> {
        self.iter().summarize_hardlinks()
    }
}

impl<Size: size::Size> SummarizeHardlinks<Size> for &HardlinkList<Size> {
    fn summarize_hardlinks(self) -> Summary<Size> {
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
    fn summarize_hardlinks(self) -> Summary<Size> {
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
            links,
            size,
        } = summary;
        let size = size.display(*format);
        write!(
            f,
            "Detected {links} hardlinks for {inodes} unique files (total: {size})"
        )
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

impl<Size: Copy> From<ReflectionEntry<Size>> for SingleInodeSummary<Size> {
    fn from(reflection: ReflectionEntry<Size>) -> Self {
        (&reflection).into()
    }
}

impl<'r, Size: Copy> From<&'r ReflectionEntry<Size>> for SingleInodeSummary<Size> {
    fn from(reflection: &'r ReflectionEntry<Size>) -> Self {
        SingleInodeSummary {
            links: reflection.links.len(),
            size: reflection.size,
        }
    }
}

impl<'a, Size: Copy> From<IterItem<'a, Size>> for SingleInodeSummary<Size> {
    fn from(value: IterItem<'a, Size>) -> Self {
        (&value).into()
    }
}

impl<'r, 'a, Size: Copy> From<&'r IterItem<'a, Size>> for SingleInodeSummary<Size> {
    fn from(value: &'r IterItem<'a, Size>) -> Self {
        SingleInodeSummary {
            links: value.links().len(),
            size: *value.size(),
        }
    }
}
