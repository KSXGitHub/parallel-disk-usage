use super::{Direction::*, Visualizer};
use crate::size;
use std::{
    ffi::OsStr,
    fmt::{Display, Error, Formatter},
    hash::Hash,
};

impl<'a, Name, Size> Display for Visualizer<'a, Name, Size>
where
    Name: Display + Hash + Eq + AsRef<OsStr>,
    Size: size::Size + Into<u64>,
{
    /// Create the ASCII chart.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        let write = |line: &String| writeln!(formatter, "{line}");
        match self.direction {
            BottomUp => self.rows().iter().rev().try_for_each(write),
            TopDown => self.rows().iter().try_for_each(write),
        }
    }
}
