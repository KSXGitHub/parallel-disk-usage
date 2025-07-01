use super::{
    data_tree::DataTree,
    get_size::GetSize,
    os_string_display::OsStringDisplay,
    reporter::{error_report::Operation::*, ErrorReport, Event, Reporter},
    size,
    tree_builder::{Info, TreeBuilder},
};
use pipe_trait::Pipe;
use std::{
    fs::{read_dir, symlink_metadata},
    path::PathBuf,
};

/// Build a [`DataTree`] from a directory tree using [`From`] or [`Into`].
///
/// **Example:**
///
/// ```no_run
/// # use parallel_disk_usage::fs_tree_builder::FsTreeBuilder;
/// use parallel_disk_usage::{
///     data_tree::DataTree,
///     get_size::GetApparentSize,
///     os_string_display::OsStringDisplay,
///     reporter::{ErrorOnlyReporter, ErrorReport},
///     size::Bytes,
/// };
/// let builder = FsTreeBuilder {
///     root: std::env::current_dir().unwrap(),
///     size_getter: GetApparentSize,
///     reporter: ErrorOnlyReporter::new(ErrorReport::SILENT),
///     max_depth: 10,
/// };
/// let data_tree: DataTree<OsStringDisplay, Bytes> = builder.into();
/// ```
#[derive(Debug)]
pub struct FsTreeBuilder<Size, SizeGetter, Report>
where
    Report: Reporter<Size> + Sync,
    Size: size::Size + Send + Sync,
    SizeGetter: GetSize<Size = Size> + Sync,
{
    /// Root of the directory tree.
    pub root: PathBuf,
    /// Returns size of an item.
    pub size_getter: SizeGetter,
    /// Reports progress to external system.
    pub reporter: Report,
    /// Deepest level of descendent display in the graph. The sizes beyond the max depth still count toward total.
    pub max_depth: u64,
}

impl<Size, SizeGetter, Report> From<FsTreeBuilder<Size, SizeGetter, Report>>
    for DataTree<OsStringDisplay, Size>
where
    Report: Reporter<Size> + Sync,
    Size: size::Size + Send + Sync,
    SizeGetter: GetSize<Size = Size> + Sync,
{
    /// Create a [`DataTree`] from an [`FsTreeBuilder`].
    fn from(builder: FsTreeBuilder<Size, SizeGetter, Report>) -> Self {
        let FsTreeBuilder {
            root,
            size_getter,
            reporter,
            max_depth,
        } = builder;

        TreeBuilder::<PathBuf, OsStringDisplay, Size, _, _> {
            name: OsStringDisplay::os_string_from(&root),

            path: root,

            get_info: |path| {
                let (is_dir, size) = match symlink_metadata(path) {
                    Err(error) => {
                        reporter.report(Event::EncounterError(ErrorReport {
                            operation: SymlinkMetadata,
                            path,
                            error,
                        }));
                        return Info {
                            size: Size::default(),
                            children: Vec::new(),
                        };
                    }
                    Ok(stats) => {
                        // `stats` should be dropped ASAP to avoid piling up kernel memory usage
                        let is_dir = stats.is_dir();
                        let size = size_getter.get_size(&stats);
                        reporter.report(Event::ReceiveData(size));
                        (is_dir, size)
                    }
                };

                let children: Vec<_> = if is_dir {
                    match read_dir(path) {
                        Err(error) => {
                            reporter.report(Event::EncounterError(ErrorReport {
                                operation: ReadDirectory,
                                path,
                                error,
                            }));
                            return Info::default();
                        }
                        Ok(entries) => entries,
                    }
                    .filter_map(|entry| match entry {
                        Err(error) => {
                            reporter.report(Event::EncounterError(ErrorReport {
                                operation: AccessEntry,
                                path,
                                error,
                            }));
                            None
                        }
                        Ok(entry) => entry.file_name().pipe(OsStringDisplay::from).pipe(Some),
                    })
                    .collect()
                } else {
                    Vec::new()
                };

                Info { size, children }
            },

            join_path: |prefix, name| prefix.join(&name.0),

            max_depth,
        }
        .into()
    }
}
