//! Temporary workspaces populated with sample filesystem trees.

use crate::temp::Temp;
use build_fs_tree::{Build, MergeableFileSystemTree, dir, file};
use derive_more::{AsRef, Deref};

/// Temporary workspace with sample filesystem tree.
#[derive(Debug, AsRef, Deref)]
#[as_ref(forward)]
#[deref(forward)]
pub struct SampleWorkspace(Temp);

impl Default for SampleWorkspace {
    /// Set up a temporary directory for tests.
    fn default() -> Self {
        let temp = Temp::new_dir().expect("create working directory for sample workspace");

        MergeableFileSystemTree::<&str, String>::from(dir! {
            "flat" => dir! {
                "0" => file!("")
                "1" => file!("a".repeat(100_000))
                "2" => file!("a".repeat(200_000))
                "3" => file!("a".repeat(300_000))
            }
            "nested" => dir! {
                "0" => dir! {
                    "1" => file!("a".repeat(500_000))
                }
            }
            "empty-dir" => dir! {}
        })
        .build(&temp)
        .expect("build the filesystem tree for the sample workspace");

        SampleWorkspace(temp)
    }
}

#[cfg(unix)]
mod unix_ext;
