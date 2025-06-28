pub mod bar_alignment;
pub mod child_position;
pub mod column_width_distribution;
pub mod direction;
pub mod parenthood;
pub mod proportion_bar;
pub mod tree;

pub use bar_alignment::BarAlignment;
pub use child_position::ChildPosition;
pub use column_width_distribution::ColumnWidthDistribution;
pub use direction::Direction;
pub use parenthood::Parenthood;
pub use proportion_bar::{ProportionBar, ProportionBarBlock};
pub use tree::{TreeHorizontalSlice, TreeSkeletalComponent};

use super::{data_tree::DataTree, size};
use std::fmt::Display;

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
/// # use parallel_disk_usage::visualizer::{Visualizer, Direction, BarAlignment, ColumnWidthDistribution};
/// # fn _wrapper(create_data_tree: fn() -> DataTree<OsStringDisplay, Bytes>) {
/// let data_tree: DataTree<OsStringDisplay, Bytes> = create_data_tree();
/// let visualizer = Visualizer {
///     data_tree: &data_tree,
///     bytes_format: BytesFormat::MetricUnits,
///     direction: Direction::BottomUp,
///     bar_alignment: BarAlignment::Right,
///     column_width_distribution: ColumnWidthDistribution::total(100),
/// };
/// println!("{visualizer}");
/// # }
/// ```
#[derive(Debug)]
pub struct Visualizer<'a, Name, Size>
where
    Name: Display,
    Size: size::Size,
{
    /// The tree to visualize.
    pub data_tree: &'a DataTree<Name, Size>,
    /// Format to be used to [`display`](size::Size::display) the sizes.
    pub bytes_format: Size::DisplayFormat,
    /// The direction of the visualization of the tree.
    pub direction: Direction,
    /// The alignment of the bars.
    pub bar_alignment: BarAlignment,
    /// Distribution and total number of characters/blocks can be placed in a line.
    pub column_width_distribution: ColumnWidthDistribution,
}

mod copy;
mod display;
mod methods;
