pub mod error_report;

pub use error_report::ErrorReport;

use super::{
    progress_report::{Event, ProgressReport},
    size::{Blocks, Bytes, Size},
    tree::Tree,
    tree_builder::{Info, TreeBuilder},
};
use error_report::Operation::*;
use pipe_trait::Pipe;
use std::{
    fs::{read_dir, symlink_metadata, Metadata},
    path::PathBuf,
};

#[cfg(unix)]
use std::os::unix::prelude::MetadataExt;

/// Infers size from a [`Metadata`].
pub type SizeGetter<Size> = fn(&Metadata) -> Size;
/// Returns `metadata.len()`.
pub const GET_APPARENT_SIZE: SizeGetter<Bytes> = |metadata| metadata.len().into();
/// Returns `metadata.blksize()` (POSIX only).
#[cfg(unix)]
pub const GET_BLOCK_SIZE: SizeGetter<Bytes> = |metadata| metadata.blksize().into();
/// Returns `metadata.blocks()` (POSIX only).
#[cfg(unix)]
pub const GET_BLOCK_COUNT: SizeGetter<Blocks> = |metadata| metadata.blocks().into();

/// Build a [`Tree`] from a directory tree using [`From`] or [`Into`].
#[derive(Debug)]
pub struct FsTreeBuilder<Data, GetData, ReportProgress, ReportError>
where
    Data: Size + Send + Sync,
    GetData: Fn(&Metadata) -> Data + Sync,
    ReportProgress: ProgressReport<Data> + Sync,
    ReportError: for<'r> Fn(&ErrorReport<'r>) + Sync,
{
    /// Root of the directory tree.
    pub root: PathBuf,
    /// Returns size of an item.
    pub get_data: GetData,
    /// Reports progress to external system.
    pub report_progress: ReportProgress,
    /// Reports error to external system.
    pub report_error: ReportError,
}

impl<Data, GetData, ReportProgress, ReportError>
    From<FsTreeBuilder<Data, GetData, ReportProgress, ReportError>> for Tree<PathBuf, Data>
where
    Data: Size + Send + Sync,
    GetData: Fn(&Metadata) -> Data + Sync,
    ReportProgress: ProgressReport<Data> + Sync,
    ReportError: for<'r> Fn(&ErrorReport<'r>) + Sync,
{
    fn from(builder: FsTreeBuilder<Data, GetData, ReportProgress, ReportError>) -> Self {
        let FsTreeBuilder {
            root,
            get_data,
            report_progress,
            report_error,
        } = builder;

        TreeBuilder::<PathBuf, Data, _, _> {
            id: root,

            get_info: |path| {
                report_progress.report(Event::BeginScanning);

                let stats = match symlink_metadata(&path) {
                    Err(error) => {
                        report_error(&ErrorReport {
                            operation: SymlinkMetadata,
                            path,
                            error,
                        });
                        report_progress.report(Event::EncounterError);
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
                            report_error(&ErrorReport {
                                operation: ReadDirectory,
                                path,
                                error,
                            });
                            report_progress.report(Event::EncounterError);
                            return Info::default();
                        }
                        Ok(entries) => entries,
                    }
                    .into_iter()
                    .filter_map(|entry| match entry {
                        Err(error) => {
                            report_error(&ErrorReport {
                                operation: AccessEntry,
                                path,
                                error,
                            });
                            report_progress.report(Event::EncounterError);
                            None
                        }
                        Ok(entry) => entry.file_name().pipe(PathBuf::from).pipe(Some),
                    })
                    .collect()
                } else {
                    Vec::new()
                };

                report_progress.report(Event::FinishScanning);

                let data = get_data(&stats);
                report_progress.report(Event::ReceiveData(data));

                Info { data, children }
            },

            join_path: |prefix, name| prefix.join(name),
        }
        .into()
    }
}
