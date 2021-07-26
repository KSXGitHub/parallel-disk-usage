use super::BarAlignment;
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
make_const!(LEVEL4_BLOCK = ' ');

/// Proportion bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into)]
pub struct ProportionBar {
    pub level0: usize,
    pub level1: usize,
    pub level2: usize,
    pub level3: usize,
    pub level4: usize,
}

impl ProportionBar {
    fn display_level0(self) -> impl Display {
        repeat(LEVEL0_BLOCK, self.level0)
    }

    fn display_level1(self) -> impl Display {
        repeat(LEVEL1_BLOCK, self.level1)
    }

    fn display_level2(self) -> impl Display {
        repeat(LEVEL2_BLOCK, self.level2)
    }

    fn display_level3(self) -> impl Display {
        repeat(LEVEL3_BLOCK, self.level3)
    }

    fn display_level4(self) -> impl Display {
        repeat(LEVEL4_BLOCK, self.level4)
    }

    /// Create a [displayable](Display) value.
    pub fn display(self, align: BarAlignment) -> ProportionBarDisplay {
        ProportionBarDisplay { bar: self, align }
    }
}

/// Result of [`ProportionBar::display`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProportionBarDisplay {
    pub bar: ProportionBar,
    pub align: BarAlignment,
}

impl Display for ProportionBarDisplay {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        let ProportionBarDisplay { bar, align } = self;
        macro_rules! fmt {
            ($pattern:literal) => {
                write!(
                    formatter,
                    $pattern,
                    level0 = bar.display_level0(),
                    level1 = bar.display_level1(),
                    level2 = bar.display_level2(),
                    level3 = bar.display_level3(),
                    level4 = bar.display_level4(),
                );
            };
        }
        match align {
            BarAlignment::Left => fmt!("{level0}{level1}{level2}{level3}{level4}"),
            BarAlignment::Right => fmt!("{level4}{level3}{level2}{level1}{level0}"),
        }
    }
}
