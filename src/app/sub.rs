use crate::{
    args::Fraction,
    data_tree::DataTree,
    fs_tree_builder::FsTreeBuilder,
    os_string_display::OsStringDisplay,
    reporter::ParallelReporter,
    runtime_error::RuntimeError,
    size::Size,
    visualizer::{ColumnWidthDistribution, Direction, Visualizer},
};
use std::{fs::Metadata, iter::once, num::NonZeroUsize, path::PathBuf};

/// The sub program of the main application.
pub struct Sub<Data, GetData, Report, PostProcessChildren>
where
    Data: Size + Into<u64> + Send + Sync,
    Report: ParallelReporter<Data> + Sync,
    GetData: Fn(&Metadata) -> Data + Copy + Sync,
    PostProcessChildren: Fn(&mut Vec<DataTree<OsStringDisplay, Data>>) + Copy + Send + Sync,
{
    /// List of files and/or directories.
    pub files: Vec<PathBuf>,
    /// Format to be used to [`display`](Size::display) the data.
    pub bytes_format: Data::DisplayFormat,
    /// The direction of the visualization.
    pub direction: Direction,
    /// Distribution and number of characters/blocks can be placed in a line.
    pub column_width_distribution: ColumnWidthDistribution,
    /// Maximum number of levels that should be visualized.
    pub max_depth: NonZeroUsize,
    /// Returns measured quantity of the files/directories.
    pub get_data: GetData,
    /// Reports measurement progress.
    pub reporter: Report,
    /// Processes lists of children after forming.
    pub post_process_children: PostProcessChildren,
    /// Minimal size proportion required to appear.
    pub minimal_ratio: Fraction,
    /// Preserve order of entries.
    pub no_sort: bool,
}

impl<Data, GetData, Report, PostProcessChildren> Sub<Data, GetData, Report, PostProcessChildren>
where
    Data: Size + Into<u64> + Send + Sync,
    Report: ParallelReporter<Data> + Sync,
    GetData: Fn(&Metadata) -> Data + Copy + Sync,
    PostProcessChildren: Fn(&mut Vec<DataTree<OsStringDisplay, Data>>) + Copy + Send + Sync,
{
    /// Run the sub program.
    pub fn run(self) -> Result<(), RuntimeError> {
        let Sub {
            files,
            bytes_format,
            direction,
            column_width_distribution,
            max_depth,
            get_data,
            reporter,
            post_process_children,
            minimal_ratio,
            no_sort,
        } = self;

        let mut iter = files
            .into_iter()
            .map(|root| -> DataTree<OsStringDisplay, Data> {
                FsTreeBuilder {
                    reporter: &reporter,
                    root,
                    get_data,
                    post_process_children,
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

        let minimal_ratio: f32 = minimal_ratio.into();
        let data_tree = {
            let mut data_tree = data_tree;
            if minimal_ratio > 0.0 {
                data_tree.par_cull_insignificant_data(minimal_ratio);
            }
            if !no_sort {
                data_tree.par_sort_by(|left, right| left.data().cmp(&right.data()).reverse());
            }
            data_tree
        };

        let visualizer = Visualizer {
            data_tree: &data_tree,
            bytes_format,
            direction,
            column_width_distribution,
            max_depth,
        };

        eprint!("\r"); // erase progress report.
        print!("{}", visualizer); // visualizer already ends with "\n", println! isn't needed here.
        Ok(())
    }
}
