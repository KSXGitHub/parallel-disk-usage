pub mod progress;

pub use progress::Progress;

use super::{
    size::{Blocks, Bytes, Size},
    tree::Tree,
    tree_builder::{Info, TreeBuilder},
};
use pipe_trait::Pipe;
use std::{
    fs::{read_dir, symlink_metadata, Metadata},
    path::PathBuf,
    sync::{Arc, RwLock},
};

#[cfg(unix)]
use std::os::unix::prelude::MetadataExt;

/// Infers size from a [`Metadata`]
pub type SizeGetter<Size> = fn(&Metadata) -> Size;
/// Returns `metadata.len()`
pub const GET_APPARENT_SIZE: SizeGetter<Bytes> = |metadata| metadata.len().into();
/// Returns `metadata.blksize()` (POSIX only)
#[cfg(unix)]
pub const GET_BLOCK_SIZE: SizeGetter<Bytes> = |metadata| metadata.blksize().into();
/// Returns `metadata.blocks()` (POSIX only)
#[cfg(unix)]
pub const GET_BLOCK_COUNT: SizeGetter<Blocks> = |metadata| metadata.blocks().into();

/// Build a [`Tree`] from a directory tree
#[derive(Debug)]
pub struct FsTreeBuilder<Data, GetData, ReportProgress>
where
    Data: Size + Send + Sync,
    GetData: Fn(&Metadata) -> Data + Sync,
    ReportProgress: Fn(&Progress<Data>) + Sync,
{
    /// Root of the directory tree
    pub root: PathBuf,
    /// Returns size of an item
    pub get_data: GetData,
    /// Reports progress to external system
    pub report_progress: ReportProgress,
}

impl<Data, GetData, ReportProgress> FsTreeBuilder<Data, GetData, ReportProgress>
where
    Data: Size + Send + Sync,
    GetData: Fn(&Metadata) -> Data + Sync,
    ReportProgress: Fn(&Progress<Data>) + Sync,
{
    pub fn build(self) -> Tree<PathBuf, Data> {
        let FsTreeBuilder {
            root,
            get_data,
            report_progress,
        } = self;

        let progress = Arc::new(RwLock::new(Progress::<Data>::default()));

        macro_rules! mut_progress {
            ($field:ident $operator:tt $addend:expr) => {
                {
                    let expect_message = concat!("lock progress to mutate", stringify!($field));
                    let mut progress = progress.write().expect(expect_message);
                    progress.$field $operator $addend;
                }
                {
                    let progress = progress.read().expect("lock progress to report");
                    report_progress(&progress);
                }
            };

            ($field:ident) => {
                mut_progress!($field += 1)
            };
        }

        mut_progress!(known_items);

        let stats = match symlink_metadata(&root) {
            Err(error) => {
                eprintln!("\r[error] symlink_metadata {:?}: {}", &root, error);
                mut_progress!(errors);
                return Tree::from_children(root, Vec::new());
            }
            Ok(stats) => stats,
        };

        TreeBuilder {
            id: root,

            get_info: |path| {
                let children: Vec<_> = if stats.file_type().is_dir() {
                    match read_dir(path) {
                        Err(error) => {
                            eprintln!("\r[error] read_dir {:?}: {}", path, error);
                            mut_progress!(errors);
                            return Info::default();
                        }
                        Ok(entries) => entries,
                    }
                    .into_iter()
                    .filter_map(|entry| match entry {
                        Err(error) => {
                            eprintln!("\r[error] access entry of {:?}: {}", path, error);
                            mut_progress!(errors);
                            None
                        }
                        Ok(entry) => entry.file_name().pipe(PathBuf::from).pipe(Some),
                    })
                    .collect()
                } else {
                    Vec::new()
                };

                mut_progress!(scanned_items);

                let data = get_data(&stats);
                mut_progress!(scanned_total += data);

                Info { data, children }
            },

            join_path: |prefix, name| prefix.join(name),
        }
        .build()
    }
}
