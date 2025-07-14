pub mod fraction;
pub mod quantity;
pub mod threads;

pub use fraction::Fraction;
pub use quantity::Quantity;
pub use threads::Threads;

use crate::{bytes_format::BytesFormat, visualizer::ColumnWidthDistribution};
use clap::{ColorChoice, Parser};
use smart_default::SmartDefault;
use std::{num::NonZeroU64, path::PathBuf};
use terminal_size::{terminal_size, Width};
use text_block_macros::text_block;

/// The CLI arguments.
#[derive(Debug, SmartDefault, Clone, Parser)]
#[clap(
    name = "pdu",

    version,

    about = "Summarize disk usage of the set of files, recursively for directories.",

    long_about = text_block! {
        "Summarize disk usage of the set of files, recursively for directories."
        ""
        "Copyright: Apache-2.0 © 2021 Hoàng Văn Khải <https://github.com/KSXGitHub/>"
        "Sponsor: https://github.com/sponsors/KSXGitHub"
    },

    after_help = text_block! {
        "Examples:"
        "    $ pdu"
        "    $ pdu path/to/file/or/directory"
        "    $ pdu file.txt dir/"
        "    $ pdu --quantity=apparent-size"
        "    $ pdu --bytes-format=plain"
        "    $ pdu --bytes-format=binary"
        "    $ pdu --min-ratio=0"
        "    $ pdu --min-ratio=0.05"
        "    $ pdu --min-ratio=0 --json-output | jq"
        "    $ pdu --min-ratio=0 < disk-usage.json"
    },

    after_long_help = text_block! {
        "Examples:"
        "    Show disk usage chart of current working directory"
        "    $ pdu"
        ""
        "    Show disk usage chart of a single file or directory"
        "    $ pdu path/to/file/or/directory"
        ""
        "    Compare disk usages of multiple files and/or directories"
        "    $ pdu file.txt dir/"
        ""
        "    Show chart in apparent sizes instead of block sizes"
        "    $ pdu --quantity=apparent-size"
        ""
        "    Show sizes in plain numbers instead of metric units"
        "    $ pdu --bytes-format=plain"
        ""
        "    Show sizes in base 2¹⁰ units (binary) instead of base 10³ units (metric)"
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
#[non_exhaustive]
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
    #[default(BytesFormat::MetricUnits)]
    pub bytes_format: BytesFormat,

    /// Detect duplicated hardlinks and remove their sizes from total.
    #[clap(long)]
    #[cfg_attr(not(unix), clap(hide = true))]
    pub deduplicate_hardlinks: bool,

    /// Print the tree top-down instead of bottom-up.
    #[clap(long)]
    pub top_down: bool,

    /// Set the root of the bars to the right.
    #[clap(long)]
    pub align_right: bool,

    /// Aspect of the files/directories to be measured.
    #[clap(long, value_enum, default_value_t = Quantity::DEFAULT)]
    #[default(Quantity::DEFAULT)]
    pub quantity: Quantity,

    /// Maximum depth to display the data (must be greater than 0).
    #[clap(long, default_value = "10")]
    #[default(_code = "10.try_into().unwrap()")]
    pub max_depth: NonZeroU64,

    /// Width of the visualization.
    #[clap(long, conflicts_with = "column_width")]
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

    /// Set the maximum number of threads to spawn. Could be either "auto", "max", or a number.
    #[clap(long, default_value_t = Threads::Auto)]
    pub threads: Threads,

    /// Do not output `.shared.details` in the JSON output.
    #[clap(long, requires = "json_output", requires = "deduplicate_hardlinks")]
    pub omit_json_shared_details: bool,

    /// Do not output `.shared.summary` in the JSON output.
    #[clap(long, requires = "json_output", requires = "deduplicate_hardlinks")]
    pub omit_json_shared_summary: bool,
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
