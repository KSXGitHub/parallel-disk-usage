pub mod sub;

pub use sub::Sub;

use crate::{
    args::{Args, Quantity, Threads},
    bytes_format::BytesFormat,
    get_size::{GetApparentSize, GetSize},
    json_data::{JsonData, UnitAndTree},
    reporter::{ErrorOnlyReporter, ErrorReport, ProgressAndErrorReporter, ProgressReport},
    runtime_error::RuntimeError,
    size,
    visualizer::{BarAlignment, Direction, Visualizer},
};
use clap::Parser;
use hdd::any_path_is_in_hdd;
use pipe_trait::Pipe;
use std::{io::stdin, time::Duration};
use sysinfo::Disks;

#[cfg(unix)]
use crate::get_size::{GetBlockCount, GetBlockSize};

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
                    }
                    .to_string()
                }};
            }

            let visualization = match unit_and_tree {
                UnitAndTree::Bytes(reflection) => visualize!(reflection, bytes_format),
                UnitAndTree::Blocks(reflection) => visualize!(reflection, ()),
            };

            print!("{visualization}"); // it already ends with "\n", println! isn't needed here.
            return Ok(());
        }

        let threads = match self.args.threads {
            Threads::Auto => {
                let disks = Disks::new_with_refreshed_list();
                if any_path_is_in_hdd::<hdd::RealApi>(&self.args.files, &disks) {
                    eprintln!("warning: HDD detected, the thread limit will be set to 1");
                    Some(1)
                } else {
                    None
                }
            }
            Threads::Max => None,
            Threads::Fixed(threads) => Some(threads),
        };

        if let Some(threads) = threads {
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build_global()
                .unwrap_or_else(|_| eprintln!("warning: Failed to set thread limit to {threads}"));
        }

        let report_error = if self.args.silent_errors {
            ErrorReport::SILENT
        } else {
            ErrorReport::TEXT
        };

        trait CreateReporter<const REPORT_PROGRESS: bool, Size> {
            type Reporter;
            fn create_reporter(report_error: fn(ErrorReport)) -> Self::Reporter;
        }

        impl<Size> CreateReporter<false, Size> for ()
        where
            Size: size::Size,
        {
            type Reporter = ErrorOnlyReporter<fn(ErrorReport)>;
            fn create_reporter(report_error: fn(ErrorReport)) -> Self::Reporter {
                ErrorOnlyReporter::new(report_error)
            }
        }

        impl<Size> CreateReporter<true, Size> for ()
        where
            Size: size::Size + Into<u64> + Send + Sync,
            ProgressReport<Size>: Default + 'static,
            u64: Into<Size>,
        {
            type Reporter = ProgressAndErrorReporter<Size, fn(ErrorReport)>;
            fn create_reporter(report_error: fn(ErrorReport)) -> Self::Reporter {
                ProgressAndErrorReporter::new(
                    ProgressReport::TEXT,
                    Duration::from_millis(100),
                    report_error,
                )
            }
        }

        trait GetSizeUtils: GetSize {
            type FormatSizeOutput;
            fn format_size(bytes_format: BytesFormat) -> Self::FormatSizeOutput;
        }

        impl GetSizeUtils for GetApparentSize {
            type FormatSizeOutput = BytesFormat;
            fn format_size(bytes_format: BytesFormat) -> Self::FormatSizeOutput {
                bytes_format
            }
        }

        #[cfg(unix)]
        impl GetSizeUtils for GetBlockSize {
            type FormatSizeOutput = BytesFormat;
            fn format_size(bytes_format: BytesFormat) -> Self::FormatSizeOutput {
                bytes_format
            }
        }

        #[cfg(unix)]
        impl GetSizeUtils for GetBlockCount {
            type FormatSizeOutput = ();
            fn format_size(_: BytesFormat) -> Self::FormatSizeOutput {}
        }

        macro_rules! run {
            ($(
                $(#[$variant_attrs:meta])*
                $quantity:ident, $size_getter:ident, $progress:literal;
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
                    size_getter: $size_getter,
                    reporter: <() as CreateReporter<$progress, <$size_getter as GetSize>::Size>>::create_reporter(report_error),
                    bytes_format: <$size_getter as GetSizeUtils>::format_size(bytes_format),
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
            ApparentSize, GetApparentSize, false;
            ApparentSize, GetApparentSize, true;
            #[cfg(unix)] BlockSize, GetBlockSize, false;
            #[cfg(unix)] BlockSize, GetBlockSize, true;
            #[cfg(unix)] BlockCount, GetBlockCount, false;
            #[cfg(unix)] BlockCount, GetBlockCount, true;
        }
    }
}

mod hdd;
mod mount_point;
