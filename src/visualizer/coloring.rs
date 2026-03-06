/// The coloring to apply to a node name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Color {
    /// Color as a directory.
    Directory,
    /// No color.
    #[default]
    Colorless,
}
