pub mod sub;

pub use sub::Sub;

use crate::{
    args::{Args, Quantity, Threads},
    bytes_format::BytesFormat,
    get_size::{GetApparentSize, GetSize},
    hardlink,
    json_data::{JsonData, JsonDataBody, JsonShared, JsonTree},
    reporter::{ErrorOnlyReporter, ErrorReport, ProgressAndErrorReporter, ProgressReport},
    runtime_error::RuntimeError,
    size,
    visualizer::{BarAlignment, ColumnWidthDistribution, Direction, Visualizer},
};
use clap::Parser;
use hdd::any_path_is_in_hdd;
use pipe_trait::Pipe;
use std::{io::stdin, time::Duration};
use sub::JsonOutputParam;
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
    pub fn run(mut self) -> Result<(), RuntimeError> {
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

            trait VisualizeJsonTree: size::Size + Into<u64> + Send {
                fn visualize_json_tree(
                    tree: JsonTree<Self>,
                    bytes_format: Self::DisplayFormat,
                    column_width_distribution: ColumnWidthDistribution,
                    direction: Direction,
                    bar_alignment: BarAlignment,
                ) -> Result<String, RuntimeError> {
                    let JsonTree { tree, shared } = tree;

                    let data_tree = tree
                        .par_try_into_tree()
                        .map_err(|error| RuntimeError::InvalidInputReflection(error.to_string()))?;
                    let visualizer = Visualizer {
                        data_tree: &data_tree,
                        bytes_format,
                        column_width_distribution,
                        direction,
                        bar_alignment,
                    };

                    let JsonShared { details, summary } = shared;
                    let summary = summary.or_else(|| details.map(|details| details.summarize()));

                    let visualization = if let Some(summary) = summary {
                        let summary = summary.display(bytes_format);
                        // visualizer already ends with "\n"
                        format!("{visualizer}{summary}\n")
                    } else {
                        visualizer.to_string()
                    };

                    Ok(visualization)
                }
            }

            impl<Size: size::Size + Into<u64> + Send> VisualizeJsonTree for Size {}

            macro_rules! visualize {
                ($tree:expr, $bytes_format:expr) => {
                    VisualizeJsonTree::visualize_json_tree(
                        $tree,
                        $bytes_format,
                        column_width_distribution,
                        direction,
                        bar_alignment,
                    )
                };
            }

            let visualization = match body {
                JsonDataBody::Bytes(tree) => visualize!(tree, bytes_format),
                JsonDataBody::Blocks(tree) => visualize!(tree, ()),
            }?;

            print!("{visualization}"); // it already ends with "\n", println! isn't needed here.
            return Ok(());
        }

        #[cfg(not(unix))]
        if self.args.deduplicate_hardlinks {
            return crate::runtime_error::UnsupportedFeature::DeduplicateHardlink
                .pipe(RuntimeError::UnsupportedFeature)
                .pipe(Err);
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

        if cfg!(unix) && self.args.deduplicate_hardlinks && self.args.files.len() > 1 {
            // Hardlinks deduplication doesn't work properly if there are more than 1 paths pointing to
            // the same tree or if a path points to a subtree of another path. Therefore, we must find
            // and remove such overlapping paths before they cause problem.
            use overlapping_arguments::{remove_overlapping_paths, RealApi};
            remove_overlapping_paths::<RealApi>(&mut self.args.files);
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
            #[inline]
            fn formatter(bytes_format: BytesFormat) -> BytesFormat {
                bytes_format
            }
        }

        #[cfg(unix)]
        impl GetSizeUtils for GetBlockSize {
            const INSTANCE: Self = GetBlockSize;
            const QUANTITY: Quantity = Quantity::BlockSize;
            #[inline]
            fn formatter(bytes_format: BytesFormat) -> BytesFormat {
                bytes_format
            }
        }

        #[cfg(unix)]
        impl GetSizeUtils for GetBlockCount {
            const INSTANCE: Self = GetBlockCount;
            const QUANTITY: Quantity = Quantity::BlockCount;
            #[inline]
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
            #[inline]
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
            #[inline]
            fn create_reporter(report_error: fn(ErrorReport)) -> Self::Reporter {
                ProgressAndErrorReporter::new(
                    ProgressReport::TEXT,
                    Duration::from_millis(100),
                    report_error,
                )
            }
        }

        trait CreateHardlinksHandler<const DEDUPLICATE_HARDLINKS: bool, const REPORT_PROGRESS: bool>:
            CreateReporter<REPORT_PROGRESS>
        {
            type HardlinksHandler: hardlink::RecordHardlinks<Self::Size, Self::Reporter>
                + sub::HardlinkSubroutines<Self::Size>;
            fn create_hardlinks_handler() -> Self::HardlinksHandler;
        }

        impl<const REPORT_PROGRESS: bool, SizeGetter> CreateHardlinksHandler<false, REPORT_PROGRESS>
            for SizeGetter
        where
            Self: CreateReporter<REPORT_PROGRESS>,
            Self::Size: Send + Sync,
        {
            type HardlinksHandler = hardlink::HardlinkIgnorant;
            #[inline]
            fn create_hardlinks_handler() -> Self::HardlinksHandler {
                hardlink::HardlinkIgnorant
            }
        }

        #[cfg(unix)]
        impl<const REPORT_PROGRESS: bool, SizeGetter> CreateHardlinksHandler<true, REPORT_PROGRESS>
            for SizeGetter
        where
            Self: CreateReporter<REPORT_PROGRESS>,
            Self::Size: Send + Sync + 'static,
            Self::Reporter: crate::reporter::Reporter<Self::Size>,
        {
            type HardlinksHandler = hardlink::HardlinkAware<Self::Size>;
            #[inline]
            fn create_hardlinks_handler() -> Self::HardlinksHandler {
                hardlink::HardlinkAware::new()
            }
        }

        macro_rules! run {
            ($(
                $(#[$variant_attrs:meta])*
                $size_getter:ident, $progress:literal, $hardlinks:ident;
            )*) => { match self.args {$(
                $(#[$variant_attrs])*
                Args {
                    quantity: <$size_getter as GetSizeUtils>::QUANTITY,
                    progress: $progress,
                    #[cfg(unix)] deduplicate_hardlinks: $hardlinks,
                    #[cfg(not(unix))] deduplicate_hardlinks: _,
                    files,
                    json_output,
                    bytes_format,
                    top_down,
                    align_right,
                    max_depth,
                    min_ratio,
                    no_sort,
                    omit_json_shared_details,
                    omit_json_shared_summary,
                    ..
                } => Sub {
                    direction: Direction::from_top_down(top_down),
                    bar_alignment: BarAlignment::from_align_right(align_right),
                    size_getter: <$size_getter as GetSizeUtils>::INSTANCE,
                    hardlinks_handler: <$size_getter as CreateHardlinksHandler<{ cfg!(unix) && $hardlinks }, $progress>>::create_hardlinks_handler(),
                    reporter: <$size_getter as CreateReporter<$progress>>::create_reporter(report_error),
                    bytes_format: <$size_getter as GetSizeUtils>::formatter(bytes_format),
                    files,
                    json_output: JsonOutputParam::from_cli_flags(json_output, omit_json_shared_details, omit_json_shared_summary),
                    column_width_distribution,
                    max_depth,
                    min_ratio,
                    no_sort,
                }
                .run(),
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
mod overlapping_arguments;
