pub mod child_position;
pub mod column_width_distribution;
pub mod direction;
pub mod parenthood;
pub mod proportion_bar;
pub mod tree;

pub use child_position::ChildPosition;
pub use column_width_distribution::ColumnWidthDistribution;
pub use direction::Direction;
pub use parenthood::Parenthood;
pub use proportion_bar::{ProportionBar, ProportionBarBlock};
pub use tree::{TreeHorizontalSlice, TreeSkeletalComponent};

use super::{size::Size, tree::Tree};
use std::fmt::Display;

/// Visualize a [`Tree`].
#[derive(Debug)]
pub struct Visualizer<'a, Name, Data>
where
    Name: Display,
    Data: Size,
{
    /// The tree to visualize.
    pub tree: &'a Tree<Name, Data>,
    /// The direction of the visualization of the tree.
    pub direction: Direction,
    /// Distribution and maximum number of characters/blocks can be placed in a line.
    pub column_width_distribution: ColumnWidthDistribution,
    /// Maximum number of levels that should be visualized.
    pub max_depth: usize,
}

mod copy;
mod display;
mod methods;

#[cfg(test)]
mod test;
