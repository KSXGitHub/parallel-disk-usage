use crate::{
    args::Fraction,
    data_tree::{DataTree, DataTreeReflection},
    fs_tree_builder::FsTreeBuilder,
    get_size::GetSize,
    json_data::{BinaryVersion, JsonData, SchemaVersion, UnitAndTree},
    os_string_display::OsStringDisplay,
    reporter::ParallelReporter,
    runtime_error::RuntimeError,
    size::Size,
    status_board::GLOBAL_STATUS_BOARD,
    visualizer::{BarAlignment, ColumnWidthDistribution, Direction, Visualizer},
};
use serde::Serialize;
use std::{io::stdout, iter::once, num::NonZeroUsize, path::PathBuf};

/// The sub program of the main application.
pub struct Sub<Data, GetData, Report>
where
    Data: Size + Into<u64> + Serialize + Send + Sync,
    Report: ParallelReporter<Data> + Sync,
    GetData: GetSize<Size = Data> + Copy + Sync,
    DataTreeReflection<String, Data>: Into<UnitAndTree>,
{
    /// List of files and/or directories.
    pub files: Vec<PathBuf>,
    /// Print JSON data instead of an ASCII chart.
    pub json_output: bool,
    /// Format to be used to [`display`](Size::display) the data.
    pub bytes_format: Data::DisplayFormat,
    /// The direction of the visualization.
    pub direction: Direction,
    /// The alignment of the bars.
    pub bar_alignment: BarAlignment,
    /// Distribution and number of characters/blocks can be placed in a line.
    pub column_width_distribution: ColumnWidthDistribution,
    /// Maximum number of levels that should be visualized.
    pub max_depth: NonZeroUsize,
    /// Returns measured quantity of the files/directories.
    pub get_data: GetData,
    /// Reports measurement progress.
    pub reporter: Report,
    /// Minimal size proportion required to appear.
    pub min_ratio: Fraction,
    /// Preserve order of entries.
    pub no_sort: bool,
}

impl<Data, GetData, Report> Sub<Data, GetData, Report>
where
    Data: Size + Into<u64> + Serialize + Send + Sync,
    Report: ParallelReporter<Data> + Sync,
    GetData: GetSize<Size = Data> + Copy + Sync,
    DataTreeReflection<String, Data>: Into<UnitAndTree>,
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
            get_data,
            reporter,
            min_ratio,
            no_sort,
        } = self;

        let mut iter = files
            .into_iter()
            .map(|root| -> DataTree<OsStringDisplay, Data> {
                FsTreeBuilder {
                    reporter: &reporter,
                    root,
                    get_data,
                }
                .into()
            });

        let data_tree = if let Some(data_tree) = iter.next() {
            data_tree
        } else {
            return Sub {
                files: vec![".".into()],
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
                Data::default(),
                children,
            )
        };

        if reporter.destroy().is_err() {
            eprintln!("[warning] Failed to destroy the thread that reports progress");
        }

        let min_ratio: f32 = min_ratio.into();
        let data_tree = {
            let mut data_tree = data_tree;
            if min_ratio > 0.0 {
                data_tree.par_cull_insignificant_data(min_ratio);
            }
            if !no_sort {
                data_tree.par_sort_by(|left, right| left.data().cmp(&right.data()).reverse());
            }
            data_tree
        };

        GLOBAL_STATUS_BOARD.clear_line(0);

        if json_output {
            let unit_and_tree: UnitAndTree = data_tree
                .into_reflection() // I really want to use std::mem::transmute here but can't.
                .par_convert_names_to_utf8() // TODO: allow non-UTF8 somehow.
                .expect("convert all names from raw string to UTF-8")
                .into();
            let json_data = JsonData {
                schema_version: SchemaVersion,
                binary_version: Some(BinaryVersion::current()),
                unit_and_tree,
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
            max_depth,
        };

        print!("{}", visualizer); // visualizer already ends with "\n", println! isn't needed here.
        Ok(())
    }
}
