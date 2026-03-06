/// The coloring to apply to a node name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// Color as a directory.
    Directory,
    /// No color.
    Normal,
}
