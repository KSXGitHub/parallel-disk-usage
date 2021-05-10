/// The direction of the visualization of the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// The root of the tree is placed at the bottom of the visualization.
    BottomUp,
    /// The root of the tree is placed at the top of the visualization.
    TopDown,
}
