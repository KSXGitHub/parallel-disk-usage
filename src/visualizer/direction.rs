/// The direction of the visualization of the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// The root of the tree is placed at the bottom of the visualization.
    BottomUp,
    /// The root of the tree is placed at the top of the visualization.
    TopDown,
}

impl Direction {
    pub(crate) const fn from_top_down(top_down: bool) -> Self {
        if top_down {
            Direction::TopDown
        } else {
            Direction::BottomUp
        }
    }
}
