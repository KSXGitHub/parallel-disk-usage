pub mod sub;

pub use sub::Sub;

use crate::{
    args::{Args, Quantity, Threads},
    bytes_format::BytesFormat,
    get_size::{GetApparentSize, GetSize},
    hook,
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
            SizeGetter: GetSizeUtils,
        {
            type Reporter = ErrorOnlyReporter<fn(ErrorReport)>;
            fn create_reporter(report_error: fn(ErrorReport)) -> Self::Reporter {
                ErrorOnlyReporter::new(report_error)
            }
        }

        impl<SizeGetter> CreateReporter<true> for SizeGetter
        where
            SizeGetter: GetSizeUtils,
            SizeGetter::Size: Into<u64> + Send + Sync,
            ProgressReport<SizeGetter::Size>: Default + 'static,
            u64: Into<SizeGetter::Size>,
        {
            type Reporter = ProgressAndErrorReporter<SizeGetter::Size, fn(ErrorReport)>;
            fn create_reporter(report_error: fn(ErrorReport)) -> Self::Reporter {
                ProgressAndErrorReporter::new(
                    ProgressReport::TEXT,
                    Duration::from_millis(100),
                    report_error,
                )
            }
        }

        trait HardlinkDeduplicationSystem<const DEDUPLICATE_HARDLINKS: bool>: GetSizeUtils {
            type Hook: hook::Hook<Self::Size> + sub::DeduplicateHardlinkSizes<Self::Size>;
            fn create_hook(
                record: <Self::Hook as sub::DeduplicateHardlinkSizes<Self::Size>>::HardlinkRecord,
            ) -> Self::Hook;
            fn init_hardlink_record(
            ) -> <Self::Hook as sub::DeduplicateHardlinkSizes<Self::Size>>::HardlinkRecord;
        }

        impl<SizeGetter> HardlinkDeduplicationSystem<false> for SizeGetter
        where
            SizeGetter: GetSizeUtils,
            SizeGetter::Size: Send + Sync,
        {
            type Hook = hook::DoNothing;
            fn create_hook((): ()) -> Self::Hook {
                hook::DoNothing
            }
            fn init_hardlink_record() {}
        }

        #[cfg(unix)]
        impl<SizeGetter> HardlinkDeduplicationSystem<true> for SizeGetter
        where
            SizeGetter: GetSizeUtils,
            SizeGetter::Size: From<u64> + Send + Sync + 'static,
        {
            type Hook = hook::RecordHardLink<'static, Self::Size>;
            fn create_hook(record: &'static hook::RecordHardLinkStorage<Self::Size>) -> Self::Hook {
                hook::RecordHardLink::new(record)
            }
            fn init_hardlink_record() -> &'static hook::RecordHardLinkStorage<Self::Size> {
                hook::RecordHardLinkStorage::new()
                    .pipe(Box::new)
                    .pipe(Box::leak)
            }
        }

        macro_rules! run {
            ($(
                $(#[$variant_attrs:meta])*
                $size_getter:ident, $progress:literal, $deduplicate_hardlinks:ident;
            )*) => { match self.args {$(
                $(#[$variant_attrs])*
                Args {
                    quantity: <$size_getter as GetSizeUtils>::QUANTITY,
                    progress: $progress,
                    #[cfg(unix)] deduplicate_hardlinks: $deduplicate_hardlinks,
                    files,
                    json_output,
                    bytes_format,
                    top_down,
                    align_right,
                    max_depth,
                    min_ratio,
                    no_sort,
                    ..
                } => {
                    const DEDUPLICATE_HARDLINKS: bool = cfg!(unix) && $deduplicate_hardlinks;
                    let hardlink_record = <$size_getter as HardlinkDeduplicationSystem<DEDUPLICATE_HARDLINKS>>::init_hardlink_record();
                    let hook = <$size_getter as HardlinkDeduplicationSystem<DEDUPLICATE_HARDLINKS>>::create_hook(hardlink_record);

                    Sub {
                        direction: Direction::from_top_down(top_down),
                        bar_alignment: BarAlignment::from_align_right(align_right),
                        size_getter: <$size_getter as GetSizeUtils>::INSTANCE,
                        hook,
                        hardlink_record,
                        reporter: <$size_getter as CreateReporter<$progress>>::create_reporter(report_error),
                        bytes_format: <$size_getter as GetSizeUtils>::formatter(bytes_format),
                        files,
                        json_output,
                        column_width_distribution,
                        max_depth,
                        min_ratio,
                        no_sort,
                    }
                    .run()
                },
            )*} };
        }

        run! {
            GetApparentSize, false, false;
            GetApparentSize, true, false;
            #[cfg(unix)] GetBlockSize, false, false;
            #[cfg(unix)] GetBlockSize, true, false;
            #[cfg(unix)] GetBlockCount, false, false;
            #[cfg(unix)] GetBlockCount, true, false;
            #[cfg(unix)] GetApparentSize, false, true;
            #[cfg(unix)] GetApparentSize, true, true;
            #[cfg(unix)] GetBlockSize, false, true;
            #[cfg(unix)] GetBlockSize, true, true;
            #[cfg(unix)] GetBlockCount, false, true;
            #[cfg(unix)] GetBlockCount, true, true;
        }
    }
}

mod hdd;
mod mount_point;
