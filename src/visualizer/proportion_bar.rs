use derive_more::{AsRef, Deref, Display, From, Into};
use fmt_iter::repeat;
use std::fmt::{Display, Error, Formatter};

/// Block of proportion bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRef, Deref, Display, Into)]
pub struct ProportionBarBlock(char);

impl ProportionBarBlock {
    /// Create a new [`ProportionBarBlock`] according to a distance level.
    pub const fn new(level: u8) -> Self {
        ProportionBarBlock(lookup_block(level))
    }
}

/// Lookup visualization block according to distance from parent.
///
/// The closer the child, the bolder the block.
pub const fn lookup_block(level: u8) -> char {
    match level {
        0 => '█',
        1 => '▓',
        2 => '▒',
        3 => '░',
        _ => ' ',
    }
}

/// Proportion bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into)]
pub struct ProportionBar {
    pub level0: usize,
    pub level1: usize,
    pub level2: usize,
    pub level3: usize,
    pub spaces: usize,
}

impl ProportionBar {
    pub fn display_level0(self) -> impl Display {
        repeat(lookup_block(0), self.level0)
    }

    pub fn display_level1(self) -> impl Display {
        repeat(lookup_block(1), self.level1)
    }

    pub fn display_level2(self) -> impl Display {
        repeat(lookup_block(2), self.level2)
    }

    pub fn display_level3(self) -> impl Display {
        repeat(lookup_block(3), self.level3)
    }

    pub fn display_spaces(self) -> impl Display {
        repeat(lookup_block(4), self.spaces)
    }
}

impl Display for ProportionBar {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            formatter,
            "{spaces}{level3}{level2}{level1}{level0}",
            spaces = self.display_spaces(),
            level3 = self.display_level3(),
            level2 = self.display_level2(),
            level1 = self.display_level1(),
            level0 = self.display_level0(),
        )
    }
}
