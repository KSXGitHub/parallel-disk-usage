use super::{
    data_tree::DataTree,
    get_size::GetSize,
    hardlink::{RecordHardlinks, RecordHardlinksArgument},
    os_string_display::OsStringDisplay,
    reporter::{error_report::Operation::*, ErrorReport, Event, Reporter},
    size,
    tree_builder::{Info, TreeBuilder},
};
use device_id::get_device_id;
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
///     hardlink::HardlinkIgnorant,
/// };
/// let builder = FsTreeBuilder {
///     root: std::env::current_dir().unwrap(),
///     hardlinks_recorder: &HardlinkIgnorant,
///     size_getter: GetApparentSize,
///     reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
///     one_file_system: false,
///     max_depth: 10,
/// };
/// let data_tree: DataTree<OsStringDisplay, Bytes> = builder.into();
/// ```
#[derive(Debug)]
pub struct FsTreeBuilder<'a, Size, SizeGetter, HardlinksRecorder, Report>
where
    Report: Reporter<Size> + Sync + ?Sized,
    Size: size::Size + Send + Sync,
    SizeGetter: GetSize<Size = Size> + Sync,
    HardlinksRecorder: RecordHardlinks<Size, Report> + Sync + ?Sized,
{
    /// Root of the directory tree.
    pub root: PathBuf,
    /// Returns size of an item.
    pub size_getter: SizeGetter,
    /// Handle to detect and record hardlinks.
    pub hardlinks_recorder: &'a HardlinksRecorder,
    /// Reports progress to external system.
    pub reporter: &'a Report,
    /// Skip directories on different filesystems.
    pub one_file_system: bool,
    /// Deepest level of descendant display in the graph. The sizes beyond the max depth still count toward total.
    pub max_depth: u64,
}

impl<'a, Size, SizeGetter, HardlinksRecorder, Report>
    From<FsTreeBuilder<'a, Size, SizeGetter, HardlinksRecorder, Report>>
    for DataTree<OsStringDisplay, Size>
where
    Report: Reporter<Size> + Sync + ?Sized,
    Size: size::Size + Send + Sync,
    SizeGetter: GetSize<Size = Size> + Sync,
    HardlinksRecorder: RecordHardlinks<Size, Report> + Sync + ?Sized,
{
    /// Create a [`DataTree`] from an [`FsTreeBuilder`].
    fn from(builder: FsTreeBuilder<Size, SizeGetter, HardlinksRecorder, Report>) -> Self {
        let FsTreeBuilder {
            root,
            size_getter,
            hardlinks_recorder,
            reporter,
            one_file_system,
            max_depth,
        } = builder;

        // `root` would be inspected multiple times, but its impact on performance is insignificant
        // before the (usually) massive fs tree `root` contains.
        let root_dev = if one_file_system {
            match symlink_metadata(&root) {
                Err(error) => {
                    reporter.report(Event::EncounterError(ErrorReport {
                        operation: SymlinkMetadata,
                        path: &root,
                        error,
                    }));
                    return DataTree::file(OsStringDisplay::os_string_from(&root), Size::default());
                }
                Ok(stats) => Some(get_device_id(&stats)),
            }
        } else {
            None
        };

        TreeBuilder::<PathBuf, OsStringDisplay, Size, _, _> {
            name: OsStringDisplay::os_string_from(&root),

            path: root,

            get_info: |path| {
                let (is_dir, size, same_device) = match symlink_metadata(path) {
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
                        let same_device =
                            root_dev.is_none_or(|root_dev| get_device_id(&stats) == root_dev);
                        let size = size_getter.get_size(&stats);
                        reporter.report(Event::ReceiveData(size));
                        hardlinks_recorder
                            .record_hardlinks(RecordHardlinksArgument::new(
                                path, &stats, size, reporter,
                            ))
                            .ok(); // ignore the error for now
                        (is_dir, size, same_device)
                    }
                };

                let children: Vec<_> = if is_dir && same_device {
                    match read_dir(path) {
                        Err(error) => {
                            reporter.report(Event::EncounterError(ErrorReport {
                                operation: ReadDirectory,
                                path,
                                error,
                            }));
                            return Info {
                                size,
                                children: Vec::new(),
                            };
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

mod device_id;
