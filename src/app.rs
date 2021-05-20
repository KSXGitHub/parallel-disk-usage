pub mod sub;

pub use sub::Sub;

use crate::{
    args::{Args, Quantity},
    reporter::{ErrorOnlyReporter, ErrorReport, ProgressAndErrorReporter, ProgressReport},
    runtime_error::RuntimeError,
    size::{Bytes, Size},
    size_getters::GET_APPARENT_SIZE,
    visualizer::Direction,
};
use std::time::Duration;
use structopt_utilities::StructOptUtils;

#[cfg(unix)]
use crate::{
    size::Blocks,
    size_getters::{GET_BLOCK_COUNT, GET_BLOCK_SIZE},
};

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
    pub fn run(self) -> Result<(), RuntimeError> {
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

        fn error_only_reporter<Data: Size>(
            report_error: fn(ErrorReport),
        ) -> ErrorOnlyReporter<fn(ErrorReport)> {
            ErrorOnlyReporter::new(report_error)
        }

        fn progress_and_error_reporter<Data>(
            report_error: fn(ErrorReport),
        ) -> ProgressAndErrorReporter<Data, fn(ErrorReport)>
        where
            Data: Size + Into<u64> + Send + Sync,
            ProgressReport<Data>: Default + 'static,
        {
            ProgressAndErrorReporter::new(
                ProgressReport::TEXT,
                Duration::from_millis(100),
                report_error,
            )
        }

        macro_rules! sub {
            (
                $data:ty => $format:expr;
                $quantity:ident => $get_data:ident;
                $progress:literal => $create_reporter:ident;
            ) => {
                if let Args {
                    quantity: Quantity::$quantity,
                    progress: $progress,
                    files,
                    bytes_format,
                    top_down,
                    max_depth,
                    min_ratio,
                    no_sort,
                    ..
                } = self.args
                {
                    return Sub {
                        direction: Direction::from_top_down(top_down),
                        get_data: $get_data,
                        reporter: $create_reporter::<$data>(report_error),
                        bytes_format: $format(bytes_format),
                        files,
                        column_width_distribution,
                        max_depth,
                        min_ratio,
                        no_sort,
                    }
                    .run();
                }
            };
        }

        sub! {
            Bytes => |x| x;
            ApparentSize => GET_APPARENT_SIZE;
            false => error_only_reporter;
        }

        sub! {
            Bytes => |x| x;
            ApparentSize => GET_APPARENT_SIZE;
            true => progress_and_error_reporter;
        }

        #[cfg(unix)]
        sub! {
            Bytes => |x| x;
            BlockSize => GET_BLOCK_SIZE;
            false => error_only_reporter;
        }

        #[cfg(unix)]
        sub! {
            Bytes => |x| x;
            BlockSize => GET_BLOCK_SIZE;
            true => progress_and_error_reporter;
        }

        #[cfg(unix)]
        sub! {
            Blocks => |_| ();
            BlockCount => GET_BLOCK_COUNT;
            false => error_only_reporter;
        }

        #[cfg(unix)]
        sub! {
            Blocks => |_| ();
            BlockCount => GET_BLOCK_COUNT;
            true => progress_and_error_reporter;
        }

        dbg!(self.args);
        panic!("Invalid combination of arguments")
    }
}
