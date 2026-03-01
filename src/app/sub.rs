use crate::{
    args::{Depth, Fraction},
    data_tree::DataTree,
    fs_tree_builder::build_data_tree_from_fs,
    get_size::GetSize,
    hardlink::{DeduplicateSharedSize, HardlinkIgnorant, RecordHardlinks},
    json_data::{BinaryVersion, JsonData, JsonDataBody, JsonShared, JsonTree, SchemaVersion},
    os_string_display::OsStringDisplay,
    reporter::ParallelReporter,
    runtime_error::RuntimeError,
    size,
    status_board::GLOBAL_STATUS_BOARD,
    visualizer::{BarAlignment, ColumnWidthDistribution, Direction, Visualizer},
};
use bon::builder;
use pipe_trait::Pipe;
use serde::Serialize;
use std::{io::stdout, iter::once, path::PathBuf};

/// Start building a sub program.
///
/// The returned object is a builder that asks for various fields via setter methods.
///
/// Once all required fields have been set, call [`SubBuilder::run`] to execute the sub program.
#[builder(
    finish_fn(
        name = run,
        doc {
            /// Run the sub program.
        }
    )
)]
pub fn sub<Size, SizeGetter, HardlinksHandler, Report>(
    /// List of files and/or directories.
    files: Vec<PathBuf>,
    /// Print JSON data instead of an ASCII chart.
    #[builder(required)]
    json_output: Option<JsonOutputParam>,
    /// Format to be used to [`display`](size::Size::display) the sizes returned by `size_getter`.
    bytes_format: Size::DisplayFormat,
    /// The direction of the visualization.
    direction: Direction,
    /// The alignment of the bars.
    bar_alignment: BarAlignment,
    /// Distribution and number of characters/blocks can be placed in a line.
    column_width_distribution: ColumnWidthDistribution,
    /// Maximum number of levels that should be visualized.
    max_depth: Depth,
    /// [Get the size](GetSize) of files/directories.
    size_getter: SizeGetter,
    /// Handle to detect, record, and deduplicate hardlinks.
    hardlinks_handler: HardlinksHandler,
    /// Reports measurement progress.
    reporter: Report,
    /// Minimal size proportion required to appear.
    min_ratio: Fraction,
    /// Preserve order of entries.
    no_sort: bool,
) -> Result<(), RuntimeError>
where
    Size: size::Size + Into<u64> + Serialize + Send + Sync,
    Report: ParallelReporter<Size> + Sync,
    SizeGetter: GetSize<Size = Size> + Copy + Sync,
    HardlinksHandler: RecordHardlinks<Size, Report> + HardlinkSubroutines<Size> + Sync,
    JsonTree<Size>: Into<JsonDataBody>,
{
    let max_depth_u64 = max_depth.get();

    let mut iter = files
        .into_iter()
        .map(|root| -> DataTree<OsStringDisplay, Size> {
            build_data_tree_from_fs()
                .reporter(&reporter)
                .root(root)
                .size_getter(size_getter)
                .hardlinks_recorder(&hardlinks_handler)
                .max_depth(max_depth_u64)
                .call()
        });

    let data_tree = if let Some(data_tree) = iter.next() {
        data_tree
    } else {
        return sub()
            .files(vec![".".into()])
            .json_output(json_output)
            .bytes_format(bytes_format)
            .direction(direction)
            .bar_alignment(bar_alignment)
            .column_width_distribution(column_width_distribution)
            .max_depth(max_depth)
            .size_getter(size_getter)
            .hardlinks_handler(hardlinks_handler)
            .reporter(reporter)
            .min_ratio(min_ratio)
            .no_sort(no_sort)
            .run();
    };

    let only_one_arg = iter.len() == 0; // ExactSizeIterator::is_empty is unstable
    let data_tree = if only_one_arg {
        data_tree
    } else {
        let children: Vec<_> = once(data_tree).chain(iter).collect();

        // This name is for hardlinks deduplication to work correctly as empty string is considered to be the start of any path.
        // It would be changed into "(total)" later.
        let fake_root_name = OsStringDisplay::os_string_from("");

        DataTree::dir(fake_root_name, Size::default(), children)
            .into_par_retained(|_, depth| depth + 1 < max_depth_u64)
    };

    if reporter.destroy().is_err() {
        eprintln!("[warning] Failed to destroy the thread that reports progress");
    }

    let min_ratio: f32 = min_ratio.into();
    let (data_tree, deduplication_record) = {
        let mut data_tree = data_tree;
        if min_ratio > 0.0 {
            data_tree.par_cull_insignificant_data(min_ratio);
        }
        if !no_sort {
            data_tree.par_sort_by(|left, right| left.size().cmp(&right.size()).reverse());
        }
        let deduplication_record = hardlinks_handler.deduplicate(&mut data_tree);
        if !only_one_arg {
            assert_eq!(data_tree.name().as_os_str().to_str(), Some(""));
            *data_tree.name_mut() = OsStringDisplay::os_string_from("(total)");
        }
        (data_tree, deduplication_record)
    };

    GLOBAL_STATUS_BOARD.clear_line(0);

    if let Some(json_output) = json_output {
        let JsonOutputParam {
            shared_details,
            shared_summary,
        } = json_output;
        let tree = data_tree
            .into_reflection() // I really want to use std::mem::transmute here but can't.
            .par_convert_names_to_utf8() // TODO: allow non-UTF8 somehow.
            .expect("convert all names from raw string to UTF-8");

        let deduplication_result = if !shared_details && !shared_summary {
            Ok(JsonShared::default())
        } else {
            // `try` expression would be extremely useful right now but it sadly requires nightly
            || -> Result<_, RuntimeError> {
                let mut shared = deduplication_record
                    .map_err(HardlinksHandler::convert_error)?
                    .pipe(HardlinksHandler::json_report)?
                    .unwrap_or_default();
                if !shared_details {
                    shared.details = None;
                }
                if !shared_summary {
                    shared.summary = None;
                }
                Ok(shared)
            }()
        };

        // errors caused by failing deduplication shouldn't prevent the JSON data from being printed
        let (shared, deduplication_result) = match deduplication_result {
            Ok(shared) => (shared, Ok(())),
            Err(error) => (JsonShared::default(), Err(error)),
        };

        let json_tree = JsonTree { tree, shared };
        let json_data = JsonData {
            schema_version: SchemaVersion,
            binary_version: Some(BinaryVersion::current()),
            body: json_tree.into(),
        };

        return serde_json::to_writer(stdout(), &json_data)
            .map_err(RuntimeError::SerializationFailure)
            .or(deduplication_result);
    }

    let visualizer = Visualizer {
        data_tree: &data_tree,
        bytes_format,
        direction,
        bar_alignment,
        column_width_distribution,
    };

    print!("{visualizer}"); // visualizer already ends with "\n", println! isn't needed here.

    let deduplication_record = deduplication_record.map_err(HardlinksHandler::convert_error)?;
    HardlinksHandler::print_report(deduplication_record, bytes_format)?;

    Ok(())
}

/// Value to pass to [`sub`] to decide how much details should be
/// put in the output JSON object.
#[derive(Debug, Clone, Copy)]
pub struct JsonOutputParam {
    /// Whether to include `.shared.details` in the JSON output.
    pub shared_details: bool,
    /// Whether to include `.shared.summary` in the JSON output.
    pub shared_summary: bool,
}

impl JsonOutputParam {
    /// Infer from the CLI flags.
    pub(super) fn from_cli_flags(
        output_json: bool,
        omit_shared_details: bool,
        omit_shared_summary: bool,
    ) -> Option<Self> {
        output_json.then_some(JsonOutputParam {
            shared_details: !omit_shared_details,
            shared_summary: !omit_shared_summary,
        })
    }
}

/// Subroutines used by [`sub`] to deduplicate sizes of detected hardlinks and report about it.
pub trait HardlinkSubroutines<Size: size::Size>: DeduplicateSharedSize<Size> {
    /// Convert the error to runtime error.
    fn convert_error(error: Self::Error) -> RuntimeError;
    /// Handle the report.
    fn print_report(
        report: Self::Report,
        bytes_format: Size::DisplayFormat,
    ) -> Result<(), RuntimeError>;
    /// Create a JSON serializable object from the report.
    fn json_report(report: Self::Report) -> Result<Option<JsonShared<Size>>, RuntimeError>;
}

impl<Size> HardlinkSubroutines<Size> for HardlinkIgnorant
where
    DataTree<OsStringDisplay, Size>: Send,
    Size: size::Size + Sync,
{
    #[inline]
    fn convert_error(error: Self::Error) -> RuntimeError {
        match error {}
    }

    #[inline]
    fn print_report((): Self::Report, _: Size::DisplayFormat) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline]
    fn json_report((): Self::Report) -> Result<Option<JsonShared<Size>>, RuntimeError> {
        Ok(None)
    }
}

#[cfg(unix)]
mod unix_ext;
