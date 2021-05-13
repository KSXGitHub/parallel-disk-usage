pub mod sub;

pub use sub::Sub;

use crate::{
    args::{Args, Quantity},
    data_tree::DataTree,
    os_string_display::OsStringDisplay,
    reporter::{ErrorOnlyReporter, ErrorReport},
    size::{Bytes, BytesDisplayFormat},
    size_getters::get_apparent_size,
    visualizer::{ColumnWidthDistribution, Direction},
};
use structopt_utilities::StructOptUtils;

/// The main application.
pub struct App {
    /// The CLI arguments.
    args: Args,
}

impl App {
    /// Initialize the application from the environment.
    pub fn from_env() -> Self {
        App {
            args: Args::strict_from_args(),
        }
    }

    /// Run the application.
    pub fn run(self) {
        // DYNAMIC DISPATCH POLICY:
        //
        // Errors rarely occur, therefore, using dynamic dispatch to report errors have an acceptable
        // impact on performance.
        //
        // The other operations which are invoked frequently should not utilize dynamic dispatch.

        // TODO: use flag to customize this.
        let report_error: fn(ErrorReport) = |_| {};

        match self.args {
            Args {
                quantity: Quantity::ApparentSize,
                binary_units: false,
                total_width: Some(total_width),
                column_width: None,
                files,
                top_down,
                max_depth,
                minimal_ratio,
            } => Sub {
                direction: Direction::from_top_down(top_down),
                data_display_format: BytesDisplayFormat::MetricUnits, // TODO: use flag to customize this.
                column_width_distribution: ColumnWidthDistribution::total(total_width),
                get_data: get_apparent_size,
                post_process_children: |children: &mut Vec<DataTree<OsStringDisplay, Bytes>>| {
                    children.sort_by(|left, right| left.data().cmp(&right.data()).reverse());
                },
                reporter: &ErrorOnlyReporter { report_error },
                files,
                max_depth,
                minimal_ratio,
            }
            .run(),

            // TODO: fill the rest
            // TODO: automatically deduce total_width from terminal size
            // TODO: customize progress reporting (reporter)
            // TODO: customize error reporting (reporter)
            // TODO: customize sorting (post_process_children)
            // TODO: hide items whose size are too small in comparison to total
            args => {
                dbg!(args);
                panic!("Invalid combination of arguments")
            }
        }
    }
}
