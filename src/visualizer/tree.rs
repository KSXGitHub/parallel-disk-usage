use super::{ChildPosition, Direction, Parenthood};
use derive_more::{AsRef, Deref, Display, Into};
use fmt_iter::repeat;
use std::fmt::{Display, Error, Formatter};
use zero_copy_pads::Width;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRef, Deref, Display, Into)]
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

impl Width for TreeSkeletalComponentVisualization {
    fn width(&self) -> usize {
        self.len()
    }
}

/// Horizontal slice of a tree of the height of exactly 1 line of text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeHorizontalSlice<Name: Width> {
    depth: usize,
    skeleton: TreeSkeletalComponentVisualization,
    name: Name,
}

impl<Name: Width> TreeHorizontalSlice<Name> {
    #[inline]
    fn indent_width(&self) -> usize {
        self.depth
    }

    #[inline]
    fn indent(&self) -> impl Display {
        repeat(' ', self.indent_width())
    }
}

impl<Name: Width> Display for TreeHorizontalSlice<Name> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            formatter,
            "{}{}{}",
            self.indent(),
            self.skeleton,
            &self.name,
        )
    }
}

impl<Name: Width> Width for TreeHorizontalSlice<Name> {
    fn width(&self) -> usize {
        self.indent_width() + self.skeleton.width() + self.name.width()
    }
}
