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
    pub(crate) const fn from_align_right(align_right: bool) -> Self {
        if align_right {
            BarAlignment::Right
        } else {
            BarAlignment::Left
        }
    }
}
