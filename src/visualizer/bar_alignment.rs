/// The alignment of the bars.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarAlignment {
    /// Fill the bars from left to right.
    Left,
    /// Fill the bars from right to left.
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
