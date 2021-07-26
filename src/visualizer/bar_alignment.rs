/// The alignment of the bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarAlignment {
    /// Fill the bar from left to right.
    Left,
    /// Fill the bar from right to left.
    Right,
}

impl BarAlignment {
    #[cfg(feature = "cli")]
    pub(crate) const fn from_align_left(align_left: bool) -> Self {
        if align_left {
            BarAlignment::Left
        } else {
            BarAlignment::Right
        }
    }
}
