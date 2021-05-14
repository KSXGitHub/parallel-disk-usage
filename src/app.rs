pub mod sub;

pub use sub::Sub;

use crate::{
    args::{Args, Quantity},
    reporter::{ErrorOnlyReporter, ErrorReport, ProgressAndErrorReporter, ProgressReport},
    size::{Blocks, Bytes, Size},
    size_getters::GET_APPARENT_SIZE,
    visualizer::Direction,
};
use std::fmt::Write;
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

        fn error_only_reporter<Data: Size>(
            report_error: fn(ErrorReport),
        ) -> ErrorOnlyReporter<fn(ErrorReport)> {
            ErrorOnlyReporter { report_error }
        }

        // TODO: move the logics within this function to somewhere within crate::reporter
        #[allow(clippy::type_complexity)]
        fn progress_and_error_reporter<Data: Size + Into<u64>>(
            report_error: fn(ErrorReport),
        ) -> ProgressAndErrorReporter<Data, fn(&ProgressReport<Data>), fn(ErrorReport)> {
            ProgressAndErrorReporter {
                progress: Default::default(),
                report_progress: |report| {
                    let ProgressReport {
                        known_items,
                        scanned_items,
                        scanned_total,
                        errors,
                    } = report;
                    let mut text = String::new();
                    write!(
                        text,
                        "\r(known {known}, scanned {scanned}, total {total}",
                        known = known_items,
                        scanned = scanned_items,
                        total = (*scanned_total).into(),
                    )
                    .unwrap();
                    if *errors != 0 {
                        write!(text, ", erred {}", errors).unwrap();
                    }
                    write!(text, ")").unwrap();
                    eprint!("{}", text);
                },
                report_error,
            }
        }

        // TODO: re-implement progress reporting as time-based
        // TODO: switch from --progress back to --silent-progress (i.e. progress is reported by default)

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
                    minimal_ratio,
                    ..
                } = self.args
                {
                    return Sub {
                        direction: Direction::from_top_down(top_down),
                        get_data: $get_data,
                        post_process_children: |children| {
                            children
                                .sort_by(|left, right| left.data().cmp(&right.data()).reverse());
                        },
                        reporter: &$create_reporter::<$data>(report_error),
                        bytes_format: $format(bytes_format),
                        files,
                        column_width_distribution,
                        max_depth,
                        minimal_ratio,
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

        // TODO: fill the rest
        // TODO: customize progress reporting (reporter)
        // TODO: customize error reporting (reporter)
        // TODO: customize sorting (post_process_children)
        // TODO: hide items whose size are too small in comparison to total
        // TODO: convert all panics to Err
    }
}
