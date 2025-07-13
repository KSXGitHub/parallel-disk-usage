pub mod sub;

pub use sub::Sub;

use crate::{
    args::{Args, Quantity, Threads},
    bytes_format::BytesFormat,
    get_size::{GetApparentSize, GetSize},
    json_data::{JsonData, JsonDataBody},
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

            let body = stdin()
                .pipe(serde_json::from_reader::<_, JsonData>)
                .map_err(RuntimeError::DeserializationFailure)?
                .body;

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

            let visualization = match body {
                JsonDataBody::Bytes(reflection) => visualize!(reflection, bytes_format),
                JsonDataBody::Blocks(reflection) => visualize!(reflection, ()),
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

        trait GetSizeUtils: GetSize<Size: size::Size> {
            const INSTANCE: Self;
            const QUANTITY: Quantity;
            fn formatter(bytes_format: BytesFormat) -> <Self::Size as size::Size>::DisplayFormat;
        }

        impl GetSizeUtils for GetApparentSize {
            const INSTANCE: Self = GetApparentSize;
            const QUANTITY: Quantity = Quantity::ApparentSize;
            fn formatter(bytes_format: BytesFormat) -> BytesFormat {
                bytes_format
            }
        }

        #[cfg(unix)]
        impl GetSizeUtils for GetBlockSize {
            const INSTANCE: Self = GetBlockSize;
            const QUANTITY: Quantity = Quantity::BlockSize;
            fn formatter(bytes_format: BytesFormat) -> BytesFormat {
                bytes_format
            }
        }

        #[cfg(unix)]
        impl GetSizeUtils for GetBlockCount {
            const INSTANCE: Self = GetBlockCount;
            const QUANTITY: Quantity = Quantity::BlockCount;
            fn formatter(_: BytesFormat) {}
        }

        trait CreateReporter<const REPORT_PROGRESS: bool>: GetSizeUtils {
            type Reporter;
            fn create_reporter(report_error: fn(ErrorReport)) -> Self::Reporter;
        }

        impl<SizeGetter> CreateReporter<false> for SizeGetter
        where
            Self: GetSizeUtils,
        {
            type Reporter = ErrorOnlyReporter<fn(ErrorReport)>;
            fn create_reporter(report_error: fn(ErrorReport)) -> Self::Reporter {
                ErrorOnlyReporter::new(report_error)
            }
        }

        impl<SizeGetter> CreateReporter<true> for SizeGetter
        where
            Self: GetSizeUtils,
            Self::Size: Into<u64> + Send + Sync,
            ProgressReport<Self::Size>: Default + 'static,
            u64: Into<Self::Size>,
        {
            type Reporter = ProgressAndErrorReporter<Self::Size, fn(ErrorReport)>;
            fn create_reporter(report_error: fn(ErrorReport)) -> Self::Reporter {
                ProgressAndErrorReporter::new(
                    ProgressReport::TEXT,
                    Duration::from_millis(100),
                    report_error,
                )
            }
        }

        macro_rules! run {
            ($(
                $(#[$variant_attrs:meta])*
                $size_getter:ident, $progress:literal;
            )*) => { match self.args {$(
                $(#[$variant_attrs])*
                Args {
                    quantity: <$size_getter as GetSizeUtils>::QUANTITY,
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
                    size_getter: <$size_getter as GetSizeUtils>::INSTANCE,
                    reporter: <$size_getter as CreateReporter<$progress>>::create_reporter(report_error),
                    bytes_format: <$size_getter as GetSizeUtils>::formatter(bytes_format),
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
            GetApparentSize, false;
            GetApparentSize, true;
            #[cfg(unix)] GetBlockSize, false;
            #[cfg(unix)] GetBlockSize, true;
            #[cfg(unix)] GetBlockCount, false;
            #[cfg(unix)] GetBlockCount, true;
        }
    }
}

mod hdd;
mod mount_point;
