#![cfg(test)]
use build_fs_tree::{dir, file, Build, MergeableFileSystemTree};
use derive_more::{AsRef, Deref};
use dirt::{
    fs_tree_builder::{FsTreeBuilder, Progress},
    size::Size,
    tree::Tree,
};
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::{
    env::temp_dir,
    fs::{create_dir, metadata, remove_dir_all, Metadata},
    io::Error,
    path::{Path, PathBuf},
};

/// Representation of a temporary filesystem item.
///
/// **NOTE:** Delete this once https://github.com/samgiles/rs-mktemp/issues/8 is resolved.
#[derive(Debug, AsRef, Deref)]
pub struct Temp(PathBuf);

impl Temp {
    /// Create a temporary directory.
    pub fn new_dir() -> Result<Self, Error> {
        let path = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .map(char::from)
            .collect::<String>()
            .pipe(|name| temp_dir().join(name));
        if path.exists() {
            return Self::new_dir();
        }
        create_dir(&path)?;
        path.pipe(Temp).pipe(Ok)
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let path = &self.0;
        if let Err(error) = remove_dir_all(path) {
            eprintln!("warning: Failed to delete {:?}: {}", path, error);
        }
    }
}

/// Temporary workspace with sample filesystem tree.
#[derive(Debug, AsRef, Deref)]
pub struct SampleWorkspace(Temp);

impl Default for SampleWorkspace {
    fn default() -> Self {
        let temp = Temp::new_dir().expect("create working directory for sample workspace");

        MergeableFileSystemTree::<&str, &str>::from(dir! {
            "flat" => dir! {
                "0" => file!("")
                "1" => file!("a")
                "2" => file!("ab")
                "3" => file!("abc")
            }
            "nested" => dir! {
                "0" => dir! {
                    "1" => file!("abcdef")
                }
            }
            "empty-dir" => dir! {}
        })
        .build(&temp)
        .expect("build the filesystem tree for the sample workspace");

        SampleWorkspace(temp)
    }
}

/// Make the snapshot of a [`Tree`] testable.
///
/// The real filesystem is often messy, causing `children` to mess up its order.
/// This function makes the order of `children` deterministic by reordering them recursively.
pub fn sanitize_tree<Id, Data>(tree: Tree<Id, Data>) -> Tree<Id, Data>
where
    Id: Ord,
    Data: Size,
{
    let Tree {
        id,
        data,
        mut children,
    } = tree;
    children.sort_by(|left, right| left.id.cmp(&right.id));
    let children = children.into_iter().map(sanitize_tree).collect();
    Tree { id, data, children }
}

/// Test the result of tree builder on the sample workspace.
pub fn test_sample_tree<Data, SizeFromMetadata>(root: &Path, size_from_metadata: SizeFromMetadata)
where
    Data: Size<Inner = u64> + From<u64> + Send + Sync,
    SizeFromMetadata: Fn(&Metadata) -> u64 + Copy + Sync,
{
    let suffix_size = |suffix: &str| -> Data {
        root.join(suffix)
            .pipe(metadata)
            .unwrap_or_else(|error| panic!("get_size {}: {}", suffix, error))
            .pipe_ref(size_from_metadata)
            .into()
    };

    macro_rules! suffix_size {
        ($suffix:expr $(,)?) => {
            suffix_size($suffix)
        };
        ($head:expr, $($tail:expr),* $(,)?) => {
            suffix_size($head) + suffix_size!($($tail),*)
        };
    }

    let measure = |suffix: &str| {
        FsTreeBuilder {
            get_data: |metadata| size_from_metadata(metadata).into(),
            report_error: |error| panic!("Unexpected call to report_error: {:?}", error),
            report_progress: |_: &Progress<Data>| {},
            root: root.join(suffix),
        }
        .pipe(Tree::<PathBuf, Data>::from)
        .pipe(sanitize_tree)
    };

    assert_eq!(
        measure("flat"),
        sanitize_tree(Tree {
            id: root.join("flat"),
            data: suffix_size!("flat", "flat/0", "flat/1", "flat/2", "flat/3"),
            children: vec![
                Tree {
                    id: root.join("flat").join("0"),
                    data: suffix_size("flat/0"),
                    children: Vec::new(),
                },
                Tree {
                    id: root.join("flat").join("1"),
                    data: suffix_size("flat/1"),
                    children: Vec::new(),
                },
                Tree {
                    id: root.join("flat").join("2"),
                    data: suffix_size("flat/2"),
                    children: Vec::new(),
                },
                Tree {
                    id: root.join("flat").join("3"),
                    data: suffix_size("flat/3"),
                    children: Vec::new(),
                },
            ]
        }),
    );

    assert_eq!(
        measure("nested"),
        sanitize_tree(Tree {
            id: root.join("nested"),
            data: suffix_size!("nested", "nested/0", "nested/0/1"),
            children: vec![Tree {
                id: root.join("nested").join("0"),
                data: suffix_size!("nested/0", "nested/0/1"),
                children: vec![Tree {
                    id: root.join("nested").join("0").join("1"),
                    data: suffix_size!("nested/0/1"),
                    children: Vec::new(),
                }]
            }],
        }),
    );

    assert_eq!(
        measure("empty-dir"),
        sanitize_tree(Tree {
            id: root.join("empty-dir"),
            data: suffix_size!("empty-dir"),
            children: Vec::new(),
        }),
    );
}
