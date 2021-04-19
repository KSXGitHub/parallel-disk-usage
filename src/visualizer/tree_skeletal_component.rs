use super::{ChildPosition, Direction, Parenthood};
use derive_more::{AsRef, Deref, Display, From, Into};
use std::fmt::{Display, Error, Formatter};

/// Determine 3 characters to use as skeletal component that connect a node
/// to the rest of the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeSkeletalComponent {
    /// Whether the node is the last child amongst its parent's `children`.
    pub child_position: ChildPosition,
    /// The direction of the visualization of the tree.
    pub direction: Direction,
    /// Whether the node has children.
    pub parenthood: Parenthood,
}

/// String made by calling [`TreeSkeletalComponent::visualize`](TreeSkeletalComponent).
#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRef, Deref, Display, From, Into)]
pub struct TreeSkeletalComponentVisualization(&'static str);

impl TreeSkeletalComponent {
    /// Determine 3 characters to use as skeletal component that connect a node
    /// to the rest of the tree.
    pub const fn visualize(self) -> TreeSkeletalComponentVisualization {
        use ChildPosition::*;
        use Direction::*;
        use Parenthood::*;
        let result = match (self.child_position, self.direction, self.parenthood) {
            (Init, BottomUp, Parent) => "├─┴",
            (Init, BottomUp, Childless) => "├──",
            (Init, TopDown, Parent) => "├─┬",
            (Init, TopDown, Childless) => "├──",
            (Last, BottomUp, Parent) => "┌─┴",
            (Last, BottomUp, Childless) => "┌──",
            (Last, TopDown, Parent) => "└─┬",
            (Last, TopDown, Childless) => "└──",
        };
        TreeSkeletalComponentVisualization(result)
    }
}

impl Display for TreeSkeletalComponent {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        write!(formatter, "{}", self.visualize())
    }
}
