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

        // we can't use `Quantity` directly as `const` parameter so we have to use numbers.
        mod quantity_index {
            pub const APPARENT_SIZE: u8 = 0;
            #[cfg(unix)]
            pub const BLOCK_SIZE: u8 = 1;
            #[cfg(unix)]
            pub const BLOCK_COUNT: u8 = 2;
        }

        trait IndexToQuantity<const INDEX: u8> {
            const QUANTITY: Quantity;
            type SizeGetter;
            const SIZE_GETTER: Self::SizeGetter;
        }

        impl IndexToQuantity<{ quantity_index::APPARENT_SIZE }> for () {
            const QUANTITY: Quantity = Quantity::ApparentSize;
            type SizeGetter = GetApparentSize;
            const SIZE_GETTER: Self::SizeGetter = GetApparentSize;
        }

        impl IndexToQuantity<{ quantity_index::BLOCK_SIZE }> for () {
            const QUANTITY: Quantity = Quantity::BlockSize;
            type SizeGetter = GetBlockSize;
            const SIZE_GETTER: Self::SizeGetter = GetBlockSize;
        }

        impl IndexToQuantity<{ quantity_index::BLOCK_COUNT }> for () {
            const QUANTITY: Quantity = Quantity::BlockCount;
            type SizeGetter = GetBlockCount;
            const SIZE_GETTER: Self::SizeGetter = GetBlockCount;
        }

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

        trait GetSizeUtils: GetSize<Size: size::Size> {
            fn formatter(bytes_format: BytesFormat) -> <Self::Size as size::Size>::DisplayFormat;
        }

        impl GetSizeUtils for GetApparentSize {
            fn formatter(bytes_format: BytesFormat) -> BytesFormat {
                bytes_format
            }
        }

        #[cfg(unix)]
        impl GetSizeUtils for GetBlockSize {
            fn formatter(bytes_format: BytesFormat) -> BytesFormat {
                bytes_format
            }
        }

        #[cfg(unix)]
        impl GetSizeUtils for GetBlockCount {
            fn formatter(_: BytesFormat) {}
        }

        macro_rules! run {
            ($(
                $(#[$variant_attrs:meta])*
                $quantity_index:ident, $progress:literal;
            )*) => { match self.args {$(
                $(#[$variant_attrs])*
                Args {
                    quantity: <() as IndexToQuantity<{ quantity_index::$quantity_index }>>::QUANTITY,
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
                    size_getter: <() as IndexToQuantity<{ quantity_index::$quantity_index }>>::SIZE_GETTER,
                    reporter: <() as CreateReporter<$progress, <<() as IndexToQuantity<{ quantity_index::$quantity_index }>>::SizeGetter as GetSize>::Size>>::create_reporter(report_error),
                    bytes_format: <<() as IndexToQuantity<{ quantity_index::$quantity_index }>>::SizeGetter as GetSizeUtils>::formatter(bytes_format),
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
            APPARENT_SIZE, false;
            APPARENT_SIZE, true;
            #[cfg(unix)] BLOCK_SIZE, false;
            #[cfg(unix)] BLOCK_SIZE, true;
            #[cfg(unix)] BLOCK_COUNT, false;
            #[cfg(unix)] BLOCK_COUNT, true;
        }
    }
}

mod hdd;
mod mount_point;
