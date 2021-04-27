use derive_more::{AsRef, Deref, Display, From, Into};
use fmt_iter::repeat;
use std::fmt::{Display, Error, Formatter};

/// Block of proportion bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRef, Deref, Display, Into)]
pub struct ProportionBarBlock(char);

macro_rules! make_const {
    ($name:ident = $content:literal) => {
        pub const $name: ProportionBarBlock = ProportionBarBlock($content);
    };
}

make_const!(LEVEL0_BLOCK = '█');
make_const!(LEVEL1_BLOCK = '▓');
make_const!(LEVEL2_BLOCK = '▒');
make_const!(LEVEL3_BLOCK = '░');
make_const!(SPACE_BLOCK = ' ');

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
        repeat(LEVEL0_BLOCK, self.level0)
    }

    pub fn display_level1(self) -> impl Display {
        repeat(LEVEL1_BLOCK, self.level1)
    }

    pub fn display_level2(self) -> impl Display {
        repeat(LEVEL2_BLOCK, self.level2)
    }

    pub fn display_level3(self) -> impl Display {
        repeat(LEVEL3_BLOCK, self.level3)
    }

    pub fn display_spaces(self) -> impl Display {
        repeat(SPACE_BLOCK, self.spaces)
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
