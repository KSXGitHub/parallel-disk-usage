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

use super::{data_tree::DataTree, size::Size};
use std::{fmt::Display, num::NonZeroUsize};

/// Visualize a [`DataTree`].
///
/// The fields of the struct are the construction parameters of the ASCII chart.
/// The [`Display`] trait can be used to create the ASCII chart.
///
/// **Example:**
///
/// ```no_run
/// # use parallel_disk_usage::data_tree::DataTree;
/// # use parallel_disk_usage::os_string_display::OsStringDisplay;
/// # use parallel_disk_usage::size::Bytes;
/// # use parallel_disk_usage::bytes_format::BytesFormat;
/// # use parallel_disk_usage::visualizer::{Visualizer, Direction, ColumnWidthDistribution};
/// # fn _wrapper(create_data_tree: fn() -> DataTree<OsStringDisplay, Bytes>) {
/// let data_tree: DataTree<OsStringDisplay, Bytes> = create_data_tree();
/// let visualizer = Visualizer {
///     data_tree: &data_tree,
///     bytes_format: BytesFormat::MetricUnits,
///     direction: Direction::BottomUp,
///     column_width_distribution: ColumnWidthDistribution::total(100),
///     max_depth: std::num::NonZeroUsize::new(10).unwrap(),
/// };
/// println!("{}", visualizer);
/// # }
/// ```
#[derive(Debug)]
pub struct Visualizer<'a, Name, Data>
where
    Name: Display,
    Data: Size,
{
    /// The tree to visualize.
    pub data_tree: &'a DataTree<Name, Data>,
    /// Format to be used to [`display`](Size::display) the data.
    pub bytes_format: Data::DisplayFormat,
    /// The direction of the visualization of the tree.
    pub direction: Direction,
    /// Distribution and total number of characters/blocks can be placed in a line.
    pub column_width_distribution: ColumnWidthDistribution,
    /// Maximum number of levels that should be visualized.
    pub max_depth: NonZeroUsize,
}

mod copy;
mod display;
mod methods;
