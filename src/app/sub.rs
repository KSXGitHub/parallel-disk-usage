use crate::{
    args::Fraction,
    data_tree::DataTree,
    fs_tree_builder::FsTreeBuilder,
    get_size::GetSize,
    hook,
    json_data::{BinaryVersion, JsonData, JsonTree, SchemaVersion, UnitAndTree},
    os_string_display::OsStringDisplay,
    reporter::ParallelReporter,
    runtime_error::RuntimeError,
    size,
    status_board::GLOBAL_STATUS_BOARD,
    visualizer::{BarAlignment, ColumnWidthDistribution, Direction, Visualizer},
};
use serde::Serialize;
use std::{io::stdout, iter::once, num::NonZeroU64, path::PathBuf};

/// The sub program of the main application.
pub struct Sub<Size, SizeGetter, Hook, Report>
where
    Report: ParallelReporter<Size> + Sync,
    Size: size::Size + Into<u64> + Serialize + Send + Sync,
    SizeGetter: GetSize<Size = Size> + Copy + Sync,
    Hook: hook::Hook<Size, Report> + DeduplicateHardlinkSizes<Size> + Copy + Sync,
    JsonTree<Size>: Into<UnitAndTree>,
{
    /// List of files and/or directories.
    pub files: Vec<PathBuf>,
    /// Print JSON data instead of an ASCII chart.
    pub json_output: bool,
    /// Format to be used to [`display`](size::Size::display) the sizes returned by [`size_getter`](Self::size_getter).
    pub bytes_format: Size::DisplayFormat,
    /// The direction of the visualization.
    pub direction: Direction,
    /// The alignment of the bars.
    pub bar_alignment: BarAlignment,
    /// Distribution and number of characters/blocks can be placed in a line.
    pub column_width_distribution: ColumnWidthDistribution,
    /// Maximum number of levels that should be visualized.
    pub max_depth: NonZeroU64,
    /// [Get the size](GetSize) of files/directories.
    pub size_getter: SizeGetter,
    /// Hook to run after [`Self::size_getter`].
    pub hook: Hook,
    /// Record of detected hardlinks.
    pub hardlink_record: Hook::HardlinkRecord,
    /// Reports measurement progress.
    pub reporter: Report,
    /// Minimal size proportion required to appear.
    pub min_ratio: Fraction,
    /// Preserve order of entries.
    pub no_sort: bool,
}

impl<Size, SizeGetter, Hook, Report> Sub<Size, SizeGetter, Hook, Report>
where
    Size: size::Size + Into<u64> + Serialize + Send + Sync,
    Report: ParallelReporter<Size> + Sync,
    SizeGetter: GetSize<Size = Size> + Copy + Sync,
    Hook: hook::Hook<Size, Report> + DeduplicateHardlinkSizes<Size> + Copy + Sync,
    JsonTree<Size>: Into<UnitAndTree>,
{
    /// Run the sub program.
    pub fn run(self) -> Result<(), RuntimeError> {
        let Sub {
            files,
            json_output,
            bytes_format,
            direction,
            bar_alignment,
            column_width_distribution,
            max_depth,
            size_getter,
            hook,
            hardlink_record,
            reporter,
            min_ratio,
            no_sort,
        } = self;

        let max_depth = max_depth.get();

        let mut iter = files
            .into_iter()
            .map(|root| -> DataTree<OsStringDisplay, Size> {
                FsTreeBuilder {
                    reporter: &reporter,
                    root,
                    size_getter,
                    hook,
                    max_depth,
                }
                .into()
            });

        let data_tree = if let Some(data_tree) = iter.next() {
            data_tree
        } else {
            return Sub {
                files: vec![".".into()],
                hook,
                hardlink_record,
                reporter,
                ..self
            }
            .run();
        };

        // ExactSizeIterator::is_empty is unstable
        let data_tree = if iter.len() == 0 {
            data_tree
        } else {
            let children: Vec<_> = once(data_tree).chain(iter).collect();
            DataTree::dir(
                OsStringDisplay::os_string_from("(total)"),
                Size::default(),
                children,
            )
            .into_par_retained(|_, depth| depth + 1 < max_depth)
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
            let deduplication_record =
                Hook::deduplicate_hardlink_sizes(&mut data_tree, hardlink_record);
            (data_tree, deduplication_record)
        };

        GLOBAL_STATUS_BOARD.clear_line(0);

        if json_output {
            let data = data_tree
                .into_reflection() // I really want to use std::mem::transmute here but can't.
                .par_convert_names_to_utf8() // TODO: allow non-UTF8 somehow.
                .expect("convert all names from raw string to UTF-8");
            let json_tree = JsonTree {
                data,
                shared_inodes: None, // TODO: somehow get data from `deduplication_record` above
            };
            let json_data = JsonData {
                schema_version: SchemaVersion,
                binary_version: Some(BinaryVersion::current()),
                unit_and_tree: json_tree.into(),
            };
            return serde_json::to_writer(stdout(), &json_data)
                .map_err(RuntimeError::SerializationFailure);
        }

        let visualizer = Visualizer {
            data_tree: &data_tree,
            bytes_format,
            direction,
            bar_alignment,
            column_width_distribution,
        };

        print!("{visualizer}"); // visualizer already ends with "\n", println! isn't needed here.
        Hook::report_deduplication_results(deduplication_record?, bytes_format)?;
        Ok(())
    }
}

/// Subroutines used by [`Sub`] to deduplicate sizes of detected hardlinks and report about it.
pub trait DeduplicateHardlinkSizes<Size: size::Size> {
    /// Record of detected hardlinks.
    type HardlinkRecord;
    /// Report created by [`DeduplicateHardlinkSizes::deduplicate_hardlink_sizes`].
    type DeduplicationReport;
    /// Deduplicate the sizes of detected hardlinks and return a report object.
    fn deduplicate_hardlink_sizes(
        data_tree: &mut DataTree<OsStringDisplay, Size>,
        record: Self::HardlinkRecord,
    ) -> Result<Self::DeduplicationReport, RuntimeError>;
    /// Handle the report.
    fn report_deduplication_results(
        report: Self::DeduplicationReport,
        bytes_format: Size::DisplayFormat,
    ) -> Result<(), RuntimeError>;
}

#[cfg(unix)]
impl<'a, Size> DeduplicateHardlinkSizes<Size> for hook::RecordHardlink<'a, Size>
where
    DataTree<OsStringDisplay, Size>: Send,
    Size: size::Size + Sync,
{
    type HardlinkRecord = &'a crate::hardlink::HardlinkList<Size>;
    type DeduplicationReport = &'a crate::hardlink::HardlinkList<Size>;

    fn deduplicate_hardlink_sizes(
        data_tree: &mut DataTree<OsStringDisplay, Size>,
        record: Self::HardlinkRecord,
    ) -> Result<Self::DeduplicationReport, RuntimeError> {
        use crate::hardlink::LinkPathList;
        use std::path::Path;
        let hardlink_info: Box<[(Size, LinkPathList)]> = record
            .iter()
            .map(|values| (*values.size(), values.links().clone()))
            .collect();
        let hardlink_info: Box<[(Size, Vec<&Path>)]> = hardlink_info
            .iter()
            .map(|(size, paths)| (*size, paths.iter().map(AsRef::as_ref).collect()))
            .collect();
        data_tree.par_deduplicate_hardlinks(&hardlink_info);
        Ok(record)
    }

    fn report_deduplication_results(
        report: Self::DeduplicationReport,
        bytes_format: Size::DisplayFormat,
    ) -> Result<(), RuntimeError> {
        let (inodes, links, size): (usize, usize, Size) = report
            .iter()
            .filter_map(|values| {
                let size = values.size();
                let links = values.links().len();
                (links > 1).then_some(())?;
                Some((*size, links))
            })
            .fold(
                (0, 0, Size::default()),
                |(inodes, total_links, total_size), (size, links)| {
                    (inodes + 1, total_links + links, total_size + size)
                },
            );
        if inodes > 0 {
            let size = size.display(bytes_format);
            println!("Detected {links} hardlinks for {inodes} unique files (total: {size})");
        }
        Ok(())
    }
}

impl<Size> DeduplicateHardlinkSizes<Size> for hook::DoNothing
where
    DataTree<OsStringDisplay, Size>: Send,
    Size: size::Size + Sync,
{
    type HardlinkRecord = ();
    type DeduplicationReport = ();

    fn deduplicate_hardlink_sizes(
        _: &mut DataTree<OsStringDisplay, Size>,
        _: Self::HardlinkRecord,
    ) -> Result<Self::DeduplicationReport, RuntimeError> {
        Ok(())
    }

    fn report_deduplication_results(
        (): Self::DeduplicationReport,
        _: Size::DisplayFormat,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }
}
