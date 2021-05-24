pub mod fraction;
pub mod quantity;

pub use fraction::Fraction;
pub use quantity::Quantity;

use crate::{bytes_format::BytesFormat, visualizer::ColumnWidthDistribution};
use std::{num::NonZeroUsize, path::PathBuf};
use structopt::StructOpt;
use strum::VariantNames;
use terminal_size::{terminal_size, Width};
use text_block_macros::text_block;

/// The CLI arguments.
#[derive(Debug, Clone, StructOpt)]
#[structopt(
    name = "pdu",

    long_about = text_block! {
        "Summarize disk usage of the set of files, recursively for directories."
        ""
        "Copyright: Apache-2.0 © 2021 Hoàng Văn Khải <https://ksxgithub.github.io/>"
        "Donation: https://patreon.com/khai96_"
    },

    after_help = text_block! {
        "EXAMPLES:"
        "    Show disk usage chart of current working directory"
        "    $ gdu"
        ""
        "    Show disk usage chart of a single file or directory"
        "    $ gdu path/to/file/or/directory"
        ""
        "    Compare disk usages of multiple files and/or directories"
        "    $ gdu file.txt dir/"
        ""
        "    Show chart in block sizes instead of apparent sizes"
        "    $ gdu --quantity=blksize"
        ""
        "    Show data in plain numbers instead of metric units"
        "    $ gdu --bytes-format=plain"
        ""
        "    Show data in base 2¹⁰ units (binary) instead of base 10³ units (metric)"
        "    $ gdu --bytes-format=binary"
        ""
        "    Show disk usage chart of all entries regardless of size"
        "    $ gdu --min-ratio=0"
        ""
        "    Only show disk usage chart of entries whose size is at least 5% of total"
        "    $ gdu --min-ratio=0.05"
    },
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
    #[structopt(long, possible_values = Quantity::VARIANTS, default_value = Quantity::default_value())]
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
    pub min_ratio: Fraction,

    /// Preserve order of entries.
    #[structopt(long)]
    pub no_sort: bool,

    /// Prevent filesystem error messages from appearing in stderr.
    #[structopt(long)]
    pub silent_errors: bool,

    /// Report progress being made at the expense of performance.
    #[structopt(long)]
    pub progress: bool,
}

impl Args {
    /// Deduce [`ColumnWidthDistribution`] from `--total-width` or `--column-width`.
    pub(crate) fn column_width_distribution(&self) -> ColumnWidthDistribution {
        match (self.total_width, self.column_width.as_deref()) {
            (None, None) => {
                ColumnWidthDistribution::total(if let Some((Width(width), _)) = terminal_size() {
                    width as usize
                } else {
                    150
                })
            }
            (Some(total_width), None) => ColumnWidthDistribution::total(total_width),
            (None, Some([tree_width, bar_width])) => {
                ColumnWidthDistribution::components(*tree_width, *bar_width)
            }
            (total_width, column_width) => {
                dbg!(total_width, column_width);
                panic!("Something goes wrong")
            }
        }
    }
}
