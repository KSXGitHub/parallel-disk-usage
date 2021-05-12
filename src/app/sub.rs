use crate::{
    args::Fraction,
    data_tree::DataTree,
    fs_tree_builder::FsTreeBuilder,
    os_string_display::OsStringDisplay,
    reporter::Reporter,
    size::Size,
    visualizer::{ColumnWidthDistribution, Direction, Visualizer},
};
use pipe_trait::Pipe;
use std::{ffi::OsString, fs::Metadata, iter::once, num::NonZeroUsize, path::PathBuf};

/// The sub program of the main application.
pub struct Sub<Data, GetData, Report, PostProcessChildren>
where
    Data: Size + Into<u64> + Send + Sync,
    Report: Reporter<Data> + Copy + Sync,
    GetData: Fn(&Metadata) -> Data + Copy + Sync,
    PostProcessChildren: Fn(&mut Vec<DataTree<OsStringDisplay, Data>>) + Copy + Send + Sync,
{
    /// List of files and/or directories.
    pub files: Vec<PathBuf>,
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
}

impl<Data, GetData, Report, PostProcessChildren> Sub<Data, GetData, Report, PostProcessChildren>
where
    Data: Size + Into<u64> + Send + Sync,
    Report: Reporter<Data> + Copy + Sync,
    GetData: Fn(&Metadata) -> Data + Copy + Sync,
    PostProcessChildren: Fn(&mut Vec<DataTree<OsStringDisplay, Data>>) + Copy + Send + Sync,
{
    /// Run the sub program.
    pub fn run(self) {
        let Sub {
            files,
            direction,
            column_width_distribution,
            max_depth,
            get_data,
            reporter,
            post_process_children,
            minimal_ratio,
        } = self;

        let mut iter = files
            .into_iter()
            .map(|root| -> DataTree<OsStringDisplay, Data> {
                FsTreeBuilder {
                    root,
                    get_data,
                    reporter,
                    post_process_children,
                }
                .into()
            });

        let data_tree = if let Some(data_tree) = iter.next() {
            data_tree
        } else {
            return Sub {
                files: vec![".".into()],
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

        let minimal_ratio: f32 = minimal_ratio.into();
        let data_tree = if minimal_ratio > 0.0 {
            data_tree.par_partial_reduce_insignificant_data(minimal_ratio, |param| {
                debug_assert!(!param.reduced_children.is_empty());
                if param.reduced_children.len() == 1 {
                    param.reduced_children.first().unwrap().name().clone()
                } else {
                    "(⋅⋅⋅)".pipe(OsString::from).into()
                }
            })
        } else {
            data_tree
        };

        let visualizer = Visualizer {
            data_tree: &data_tree,
            direction,
            column_width_distribution,
            max_depth,
        };

        println!("{}", visualizer);
    }
}
