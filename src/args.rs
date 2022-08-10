pub mod fraction;
pub mod quantity;

pub use fraction::Fraction;
pub use quantity::Quantity;

use crate::{bytes_format::BytesFormat, visualizer::ColumnWidthDistribution};
use clap::{ColorChoice, Parser};
use std::{num::NonZeroUsize, path::PathBuf};
use terminal_size::{terminal_size, Width};
use text_block_macros::text_block;

/// The CLI arguments.
#[derive(Debug, Clone, Parser)]
#[clap(
    name = "pdu",

    long_about = text_block! {
        "Summarize disk usage of the set of files, recursively for directories."
        ""
        "Copyright: Apache-2.0 © 2021 Hoàng Văn Khải <https://ksxgithub.github.io/>"
        "Donation: https://patreon.com/khai96_"
    },

    after_help = text_block! {
        "EXAMPLES:"
        "    $ pdu"
        "    $ pdu path/to/file/or/directory"
        "    $ pdu file.txt dir/"
        "    $ pdu --quantity=blksize"
        "    $ pdu --bytes-format=plain"
        "    $ pdu --bytes-format=binary"
        "    $ pdu --min-ratio=0"
        "    $ pdu --min-ratio=0.05"
        "    $ pdu --min-ratio=0 --json-output | jq"
        "    $ pdu --min-ratio=0 < disk-usage.json"
    },

    after_long_help = text_block! {
        "EXAMPLES:"
        "    Show disk usage chart of current working directory"
        "    $ pdu"
        ""
        "    Show disk usage chart of a single file or directory"
        "    $ pdu path/to/file/or/directory"
        ""
        "    Compare disk usages of multiple files and/or directories"
        "    $ pdu file.txt dir/"
        ""
        "    Show chart in block sizes instead of apparent sizes"
        "    $ pdu --quantity=blksize"
        ""
        "    Show data in plain numbers instead of metric units"
        "    $ pdu --bytes-format=plain"
        ""
        "    Show data in base 2¹⁰ units (binary) instead of base 10³ units (metric)"
        "    $ pdu --bytes-format=binary"
        ""
        "    Show disk usage chart of all entries regardless of size"
        "    $ pdu --min-ratio=0"
        ""
        "    Only show disk usage chart of entries whose size is at least 5% of total"
        "    $ pdu --min-ratio=0.05"
        ""
        "    Show disk usage data as JSON instead of chart"
        "    $ pdu --min-ratio=0 --json-output | jq"
        ""
        "    Visualize existing JSON representation of disk usage data"
        "    $ pdu --min-ratio=0 < disk-usage.json"
    },

    color = ColorChoice::Never,
)]
pub struct Args {
    /// List of files and/or directories.
    pub files: Vec<PathBuf>,

    /// Read JSON data from stdin.
    #[clap(long, conflicts_with = "quantity")]
    pub json_input: bool,

    /// Print JSON data instead of an ASCII chart.
    #[clap(long)]
    pub json_output: bool,

    /// How to display the numbers of bytes.
    #[clap(long, value_enum, default_value_t = BytesFormat::MetricUnits)]
    pub bytes_format: BytesFormat,

    /// Print the tree top-down instead of bottom-up.
    #[clap(long)]
    pub top_down: bool,

    /// Fill the bars from left to right.
    #[clap(long)]
    pub align_left: bool,

    /// Aspect of the files/directories to be measured.
    #[clap(long, value_enum, default_value_t = Quantity::ApparentSize)]
    pub quantity: Quantity,

    /// Maximum depth to display the data (must be greater than 0).
    #[clap(long, default_value = "10")]
    pub max_depth: NonZeroUsize,

    /// Width of the visualization.
    #[clap(long, conflicts_with = "column-width")]
    pub total_width: Option<usize>,

    /// Maximum widths of the tree column and width of the bar column.
    #[clap(long, number_of_values = 2, value_names = &["TREE_WIDTH", "BAR_WIDTH"])]
    pub column_width: Option<Vec<usize>>,

    /// Minimal size proportion required to appear.
    #[clap(long, default_value = "0.01")]
    pub min_ratio: Fraction,

    /// Preserve order of entries.
    #[clap(long)]
    pub no_sort: bool,

    /// Prevent filesystem error messages from appearing in stderr.
    #[clap(long)]
    pub silent_errors: bool,

    /// Report progress being made at the expense of performance.
    #[clap(long)]
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
