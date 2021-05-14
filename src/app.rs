pub mod sub;

pub use sub::Sub;

use crate::{
    args::{Args, Quantity},
    data_tree::DataTree,
    os_string_display::OsStringDisplay,
    reporter::{ErrorOnlyReporter, ErrorReport},
    size::{Blocks, Bytes},
    size_getters::GET_APPARENT_SIZE,
    visualizer::Direction,
};
use structopt_utilities::StructOptUtils;

#[cfg(unix)]
use crate::size_getters::{GET_BLOCK_COUNT, GET_BLOCK_SIZE};

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

        let column_width_distribution = self.args.column_width_distribution();

        let report_error = if self.args.silent_errors {
            ErrorReport::SILENT
        } else {
            ErrorReport::TEXT
        };

        match self.args {
            Args {
                quantity: Quantity::ApparentSize,
                files,
                bytes_format,
                top_down,
                max_depth,
                minimal_ratio,
                ..
            } => Sub {
                direction: Direction::from_top_down(top_down),
                get_data: GET_APPARENT_SIZE,
                post_process_children: |children: &mut Vec<DataTree<OsStringDisplay, Bytes>>| {
                    children.sort_by(|left, right| left.data().cmp(&right.data()).reverse());
                },
                reporter: &ErrorOnlyReporter { report_error },
                files,
                bytes_format,
                column_width_distribution,
                max_depth,
                minimal_ratio,
            }
            .run(),

            #[cfg(unix)]
            Args {
                quantity: Quantity::BlockSize,
                files,
                bytes_format,
                top_down,
                max_depth,
                minimal_ratio,
                ..
            } => Sub {
                direction: Direction::from_top_down(top_down),
                get_data: GET_BLOCK_SIZE,
                post_process_children: |children: &mut Vec<DataTree<OsStringDisplay, Bytes>>| {
                    children.sort_by(|left, right| left.data().cmp(&right.data()).reverse());
                },
                reporter: &ErrorOnlyReporter { report_error },
                files,
                bytes_format,
                column_width_distribution,
                max_depth,
                minimal_ratio,
            }
            .run(),

            #[cfg(unix)]
            Args {
                quantity: Quantity::BlockCount,
                files,
                top_down,
                max_depth,
                minimal_ratio,
                ..
            } => Sub {
                direction: Direction::from_top_down(top_down),
                get_data: GET_BLOCK_COUNT,
                post_process_children: |children: &mut Vec<DataTree<OsStringDisplay, Blocks>>| {
                    children.sort_by(|left, right| left.data().cmp(&right.data()).reverse());
                },
                reporter: &ErrorOnlyReporter { report_error },
                bytes_format: (),
                files,
                column_width_distribution,
                max_depth,
                minimal_ratio,
            }
            .run(),

            // TODO: fill the rest
            // TODO: customize progress reporting (reporter)
            // TODO: customize error reporting (reporter)
            // TODO: customize sorting (post_process_children)
            // TODO: hide items whose size are too small in comparison to total
            // TODO: convert all panics to Err
            // TODO: remove #[allow(unreachable_patterns)]
            #[allow(unreachable_patterns)]
            args => {
                dbg!(args);
                panic!("Invalid combination of arguments")
            }
        }
    }
}
