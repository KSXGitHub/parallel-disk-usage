use derive_more::{AsRef, Deref, Display, Into};

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
