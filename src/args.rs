pub mod fraction;
pub mod quantity;

pub use fraction::Fraction;
pub use quantity::{Quantity, QUANTITY_VALUES};

use crate::bytes_format::BytesFormat;
use std::{num::NonZeroUsize, path::PathBuf};
use structopt::StructOpt;
use strum::VariantNames;
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
    #[structopt(name = "files")]
    pub files: Vec<PathBuf>,

    /// How to display the numbers of bytes.
    #[structopt(long, possible_values = BytesFormat::VARIANTS, default_value = BytesFormat::default_value())]
    pub bytes_format: BytesFormat,

    /// Print the tree top-down instead of bottom-up.
    #[structopt(long)]
    pub top_down: bool,

    /// Aspect of the files/directories to be measured.
    #[structopt(long, possible_values = QUANTITY_VALUES, default_value = Quantity::default_value())]
    pub quantity: Quantity,

    /// Maximum depth to display the data (must be greater than 0).
    #[structopt(long, default_value = "10")]
    pub max_depth: NonZeroUsize,

    /// Width of the visualization.
    #[structopt(long, conflicts_with = "column-width")]
    pub total_width: Option<usize>,

    /// Maximum widths of the tree column and width of the bar column.
    #[structopt(long, number_of_values = 2, value_names = &["tree-width", "bar-width"])]
    pub column_width: Option<Vec<usize>>,

    /// Minimal size proportion required to appear.
    #[structopt(long, default_value = "0.01")]
    pub minimal_ratio: Fraction,
}
