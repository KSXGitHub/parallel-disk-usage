pub mod direction;
pub mod quantity;

pub use direction::*;
pub use quantity::*;

use std::{num::NonZeroUsize, path::PathBuf};
use structopt::StructOpt;
use text_block_macros::text_block;

/// The CLI arguments.
#[derive(Debug, Clone, StructOpt)]
#[structopt(
    name = "dirt",

    long_about = text_block! {
        "Summarize disk usage of the set of files, recursively for directories."
        ""
        "Copyright: Apache-2.0 © 2021 Hoàng Văn Khải <https://ksxgithub.github.io/>"
        "Donation: https://patreon.com/khai96_"
    }
)]
pub struct Args {
    /// List of files and/or directories.
    #[structopt(name = "files", about = "List of files and/or directories.")]
    pub files: Vec<PathBuf>,

    /// The direction of the tree.
    #[structopt(long, possible_values = DIRECTION_VALUES, default_value = Direction::default_value())]
    pub direction: Direction,

    /// The direction of the tree.
    #[structopt(long, possible_values = QUANTITY_VALUES, default_value = Quantity::default_value())]
    pub quantity: Quantity,

    /// Use binary units (KiB, MiB, GiB, etc.) instead of metric units (KB, MB, GB, etc.).
    #[structopt(long)]
    pub binary_units: bool,

    /// Maximum depth to display the data (must be greater than 0).
    #[structopt(long, default_value = "10")]
    pub max_depth: NonZeroUsize,

    /// Width of the visualization.
    #[structopt(long, conflicts_with = "column-width")]
    pub total_width: Option<usize>,

    /// Maximum widths of the tree column and width of the bar column.
    #[structopt(long, number_of_values = 2, value_names = &["tree-width", "bar-width"])]
    pub column_width: Option<Vec<usize>>,
}
