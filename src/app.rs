pub mod sub;

pub use sub::Sub;

use crate::{
    args::{Args, Quantity},
    get_size::GetApparentSize,
    json_data::{JsonData, UnitAndTree},
    reporter::{ErrorOnlyReporter, ErrorReport, ProgressAndErrorReporter, ProgressReport},
    runtime_error::RuntimeError,
    size::{Bytes, Size},
    visualizer::{BarAlignment, Direction, Visualizer},
};
use clap::Parser;
use pipe_trait::Pipe;
use std::{io::stdin, time::Duration};

#[cfg(unix)]
use crate::{
    get_size::{GetBlockCount, GetBlockSize},
    size::Blocks,
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
            args: Args::parse(),
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

        if self.args.json_input {
            if !self.args.files.is_empty() {
                return Err(RuntimeError::JsonInputArgConflict);
            }

            let Args {
                bytes_format,
                top_down,
                align_right,
                max_depth,
                ..
            } = self.args;
            let direction = Direction::from_top_down(top_down);
            let bar_alignment = BarAlignment::from_align_right(align_right);

            let unit_and_tree = stdin()
                .pipe(serde_json::from_reader::<_, JsonData>)
                .map_err(RuntimeError::DeserializationFailure)?
                .unit_and_tree;

            macro_rules! visualize {
                ($reflection:expr, $bytes_format: expr) => {{
                    let data_tree = $reflection
                        .par_try_into_tree()
                        .map_err(|error| RuntimeError::InvalidInputReflection(error.to_string()))?;
                    Visualizer {
                        data_tree: &data_tree,
                        bytes_format: $bytes_format,
                        column_width_distribution,
                        direction,
                        bar_alignment,
                        max_depth,
                    }
                    .to_string()
                }};
            }

            let visualization = match unit_and_tree {
                UnitAndTree::Bytes(reflection) => visualize!(reflection, bytes_format),
                UnitAndTree::Blocks(reflection) => visualize!(reflection, ()),
            };

            print!("{}", visualization); // it already ends with "\n", println! isn't needed here.
            return Ok(());
        }

        let report_error = if self.args.silent_errors {
            ErrorReport::SILENT
        } else {
            ErrorReport::TEXT
        };

        #[allow(clippy::extra_unused_type_parameters)]
        fn error_only_reporter<Data>(
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
            u64: Into<Data>,
        {
            ProgressAndErrorReporter::new(
                ProgressReport::TEXT,
                Duration::from_millis(100),
                report_error,
            )
        }

        macro_rules! run {
            ($(
                $(#[$variant_attrs:meta])*
                {
                    $data:ty => $format:expr;
                    $quantity:ident => $get_data:ident;
                    $progress:literal => $create_reporter:ident;
                }
            )*) => { match self.args {$(
                $(#[$variant_attrs])*
                Args {
                    quantity: Quantity::$quantity,
                    progress: $progress,
                    files,
                    json_output,
                    bytes_format,
                    top_down,
                    align_right,
                    max_depth,
                    min_ratio,
                    no_sort,
                    ..
                } => Sub {
                    direction: Direction::from_top_down(top_down),
                    bar_alignment: BarAlignment::from_align_right(align_right),
                    get_data: $get_data,
                    reporter: $create_reporter::<$data>(report_error),
                    bytes_format: $format(bytes_format),
                    files,
                    json_output,
                    column_width_distribution,
                    max_depth,
                    min_ratio,
                    no_sort,
                }
                .run(),
            )*} };
        }

        run! {
            {
                Bytes => |x| x;
                ApparentSize => GetApparentSize;
                false => error_only_reporter;
            }

            {
                Bytes => |x| x;
                ApparentSize => GetApparentSize;
                true => progress_and_error_reporter;
            }

            #[cfg(unix)]
            {
                Bytes => |x| x;
                BlockSize => GetBlockSize;
                false => error_only_reporter;
            }

            #[cfg(unix)]
            {
                Bytes => |x| x;
                BlockSize => GetBlockSize;
                true => progress_and_error_reporter;
            }

            #[cfg(unix)]
            {
                Blocks => |_| ();
                BlockCount => GetBlockCount;
                false => error_only_reporter;
            }

            #[cfg(unix)]
            {
                Blocks => |_| ();
                BlockCount => GetBlockCount;
                true => progress_and_error_reporter;
            }
        }
    }
}
