pub mod child_position;
pub mod direction;
pub mod lookup_block;
pub mod parenthood;
pub mod tree_skeletal_component;

pub use child_position::ChildPosition;
pub use direction::Direction;
pub use lookup_block::lookup_block;
pub use parenthood::Parenthood;
pub use tree_skeletal_component::{TreeSkeletalComponent, TreeSkeletalComponentVisualization};

use super::{size::Size, tree::Tree};
use std::fmt::Display;

/// Visualize a [`Tree`].
#[derive(Debug)]
pub struct Visualizer<Name, Data>
where
    Name: Display,
    Data: Size,
{
    /// The tree to visualize.
    pub tree: Tree<Name, Data>,
    /// The direction of the visualization of the tree.
    pub direction: Direction,
    /// Maximum number of characters/blocks can be placed in a line.
    pub max_width: u16,
}
