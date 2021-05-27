use super::{
    data_tree::DataTree,
    os_string_display::OsStringDisplay,
    reporter::{error_report::Operation::*, ErrorReport, Event, Reporter},
    size::Size,
    tree_builder::{Info, TreeBuilder},
    utils::path_name,
};
use pipe_trait::Pipe;
use std::{
    fs::{read_dir, symlink_metadata, Metadata},
    path::PathBuf,
};

/// Build a [`DataTree`] from a directory tree using [`From`] or [`Into`].
#[derive(Debug)]
pub struct FsTreeBuilder<Data, GetData, Report>
where
    Data: Size + Send + Sync,
    GetData: Fn(&Metadata) -> Data + Sync,
    Report: Reporter<Data> + Sync,
{
    /// Root of the directory tree.
    pub root: PathBuf,
    /// Returns size of an item.
    pub get_data: GetData,
    /// Reports progress to external system.
    pub reporter: Report,
}

impl<Data, GetData, Report> From<FsTreeBuilder<Data, GetData, Report>>
    for DataTree<OsStringDisplay, Data>
where
    Data: Size + Send + Sync,
    GetData: Fn(&Metadata) -> Data + Sync,
    Report: Reporter<Data> + Sync,
{
    fn from(builder: FsTreeBuilder<Data, GetData, Report>) -> Self {
        let FsTreeBuilder {
            root,
            get_data,
            reporter,
        } = builder;

        TreeBuilder::<PathBuf, OsStringDisplay, Data, _, _> {
            name: path_name(&root),

            path: root,

            get_info: |path| {
                let stats = match symlink_metadata(&path) {
                    Err(error) => {
                        reporter.report(Event::EncounterError(ErrorReport {
                            operation: SymlinkMetadata,
                            path,
                            error,
                        }));
                        return Info {
                            data: Data::default(),
                            children: Vec::new(),
                        };
                    }
                    Ok(stats) => stats,
                };

                let children: Vec<_> = if stats.file_type().is_dir() {
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
                    .into_iter()
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

                let data = get_data(&stats);
                reporter.report(Event::ReceiveData(data));

                Info { data, children }
            },

            join_path: |prefix, name| prefix.join(&name.0),
        }
        .into()
    }
}
