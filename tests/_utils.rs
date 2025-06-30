use build_fs_tree::{dir, file, Build, MergeableFileSystemTree};
use command_extra::CommandExtra;
use derive_more::{AsRef, Deref};
use parallel_disk_usage::{
    data_tree::{DataTree, DataTreeReflection},
    fs_tree_builder::FsTreeBuilder,
    get_size::{self, GetSize},
    os_string_display::OsStringDisplay,
    reporter::ErrorOnlyReporter,
    size,
};
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use rand::{distr::Alphanumeric, rng, Rng};
use rayon::prelude::*;
use std::{
    env::temp_dir,
    fs::{create_dir, metadata, remove_dir_all},
    io::Error,
    path::{Path, PathBuf},
    process::{Command, Output},
};

/// Default size getter method.
#[cfg(unix)]
pub const DEFAULT_GET_SIZE: get_size::GetBlockSize = get_size::GetBlockSize;
/// Default size getter method.
#[cfg(not(unix))]
pub const DEFAULT_GET_SIZE: get_size::GetApparentSize = get_size::GetApparentSize;

/// Representation of a temporary filesystem item.
///
/// **NOTE:** Delete this once https://github.com/samgiles/rs-mktemp/issues/8 is resolved.
#[derive(Debug, AsRef, Deref)]
#[as_ref(forward)]
#[deref(forward)]
pub struct Temp(PathBuf);

impl Temp {
    /// Create a temporary directory.
    pub fn new_dir() -> Result<Self, Error> {
        let path = rng()
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
    /// Delete the created temporary directory.
    fn drop(&mut self) {
        let path = &self.0;
        if let Err(error) = remove_dir_all(path) {
            eprintln!("warning: Failed to delete {path:?}: {error}");
        }
    }
}

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
                "1" => file!("a".repeat(100000))
                "2" => file!("a".repeat(200000))
                "3" => file!("a".repeat(300000))
            }
            "nested" => dir! {
                "0" => dir! {
                    "1" => file!("a".repeat(500000))
                }
            }
            "empty-dir" => dir! {}
        })
        .build(&temp)
        .expect("build the filesystem tree for the sample workspace");

        SampleWorkspace(temp)
    }
}

/// Make the snapshot of a [`TreeReflection`] testable.
///
/// The real filesystem is often messy, causing `children` to mess up its order.
/// This function makes the order of `children` deterministic by reordering them recursively.
pub fn sanitize_tree_reflection<Name, Size>(
    tree_reflection: DataTreeReflection<Name, Size>,
) -> DataTreeReflection<Name, Size>
where
    Name: Ord,
    Size: size::Size,
    DataTreeReflection<Name, Size>: Send,
{
    let DataTreeReflection {
        name,
        size,
        mut children,
    } = tree_reflection;
    children.sort_by(|left, right| left.name.cmp(&right.name));
    let children = children
        .into_par_iter()
        .map(sanitize_tree_reflection)
        .collect();
    DataTreeReflection {
        name,
        size,
        children,
    }
}

/// Test the result of tree builder on the sample workspace.
pub fn test_sample_tree<Size, SizeGetter>(root: &Path, size_getter: SizeGetter)
where
    Size: size::Size<Inner = u64> + From<u64> + Send + Sync,
    SizeGetter: GetSize<Size = Size> + Copy + Sync,
{
    let suffix_size = |suffix: &str| -> Size {
        root.join(suffix)
            .pipe(metadata)
            .unwrap_or_else(|error| panic!("get_size {suffix}: {error}"))
            .pipe(|ref metadata| size_getter.get_size(metadata))
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
            size_getter,
            reporter: ErrorOnlyReporter::new(|error| {
                panic!("Unexpected call to report_error: {error:?}")
            }),
            root: root.join(suffix),
            max_depth: 10,
        }
        .pipe(DataTree::<OsStringDisplay, Size>::from)
        .into_par_sorted(|left, right| left.name().cmp(right.name()))
        .into_reflection()
    };

    let sub = |suffix: &str| root.join(suffix).pipe(OsStringDisplay::os_string_from);

    assert_eq!(
        measure("flat"),
        sanitize_tree_reflection(DataTreeReflection {
            name: sub("flat"),
            size: suffix_size!("flat", "flat/0", "flat/1", "flat/2", "flat/3"),
            children: vec![
                DataTreeReflection {
                    name: OsStringDisplay::os_string_from("0"),
                    size: suffix_size("flat/0"),
                    children: Vec::new(),
                },
                DataTreeReflection {
                    name: OsStringDisplay::os_string_from("1"),
                    size: suffix_size("flat/1"),
                    children: Vec::new(),
                },
                DataTreeReflection {
                    name: OsStringDisplay::os_string_from("2"),
                    size: suffix_size("flat/2"),
                    children: Vec::new(),
                },
                DataTreeReflection {
                    name: OsStringDisplay::os_string_from("3"),
                    size: suffix_size("flat/3"),
                    children: Vec::new(),
                },
            ]
        }),
    );

    assert_eq!(
        measure("nested"),
        sanitize_tree_reflection(DataTreeReflection {
            name: sub("nested"),
            size: suffix_size!("nested", "nested/0", "nested/0/1"),
            children: vec![DataTreeReflection {
                name: OsStringDisplay::os_string_from("0"),
                size: suffix_size!("nested/0", "nested/0/1"),
                children: vec![DataTreeReflection {
                    name: OsStringDisplay::os_string_from("1"),
                    size: suffix_size!("nested/0/1"),
                    children: Vec::new(),
                }]
            }],
        }),
    );

    assert_eq!(
        measure("empty-dir"),
        sanitize_tree_reflection(DataTreeReflection {
            name: sub("empty-dir"),
            size: suffix_size!("empty-dir"),
            children: Vec::new(),
        }),
    );
}

/// Path to the `pdu` executable
pub const PDU: &str = env!("CARGO_BIN_EXE_pdu");

/// Representation of a `pdu` command.
#[derive(Debug, Default, Clone)]
pub struct CommandRepresentation<'a> {
    args: Vec<&'a str>,
}

impl<'a> CommandRepresentation<'a> {
    /// Add an argument.
    pub fn arg(mut self, arg: &'a str) -> Self {
        self.args.push(arg);
        self
    }
}

/// List of `pdu` commands.
#[derive(Debug, Clone, AsRef, Deref)]
pub struct CommandList<'a>(Vec<CommandRepresentation<'a>>);

impl<'a> Default for CommandList<'a> {
    /// Initialize a list with one `pdu` command.
    fn default() -> Self {
        CommandRepresentation::default()
            .pipe(|x| vec![x])
            .pipe(CommandList)
    }
}

impl<'a> CommandList<'a> {
    /// Duplicate the list with a flag argument.
    ///
    /// The resulting list would include the original list with the flag
    /// followed by the original list without the flag.
    pub fn flag_matrix(self, name: &'a str) -> Self {
        Self::assert_flag(name);
        let CommandList(list) = self;
        list.clone()
            .into_iter()
            .map(|cmd| cmd.arg(name))
            .chain(list)
            .collect::<Vec<_>>()
            .pipe(CommandList)
    }

    /// Duplicate the list with one or many option argument(s).
    ///
    /// The resulting list would include the original list with the option(s)
    /// followed by the original list without the option(s).
    pub fn option_matrix<const LEN: usize>(self, name: &'a str, values: [&'a str; LEN]) -> Self {
        Self::assert_flag(name);
        let CommandList(tail) = self;
        let mut head: Vec<_> = values
            .iter()
            .copied()
            .flat_map(|value| {
                tail.clone()
                    .into_iter()
                    .map(move |cmd| cmd.arg(name).arg(value))
            })
            .collect();
        head.extend(tail);
        CommandList(head)
    }

    /// Create a list of `pdu` [command](Command).
    pub fn commands(&'a self) -> impl Iterator<Item = Command> + 'a {
        self.iter()
            .map(|cmd| Command::new(PDU).with_args(&cmd.args))
    }

    /// Make sure a flag name has valid syntax.
    fn assert_flag(name: &str) {
        match name.len() {
            0 | 1 => panic!("{name:?} is not a valid flag"),
            2 => assert!(name.starts_with('-'), "{name:?} is not a valid flag"),
            _ => assert!(name.starts_with("--"), "{name:?} is not a valid flag"),
        }
    }
}

/// Make sure that status code is 0, print stderr if it's not empty,
/// and turn stdin into a string.
pub fn stdout_text(
    Output {
        status,
        stdout,
        stderr,
    }: Output,
) -> String {
    inspect_stderr(&stderr);
    assert!(
        status.success(),
        "progress exits with non-zero status: {status:?}",
    );
    stdout
        .pipe(String::from_utf8)
        .expect("parse stdout as UTF-8")
        .trim_end()
        .to_string()
}

/// Print stderr if it's not empty.
pub fn inspect_stderr(stderr: &[u8]) {
    let text = String::from_utf8_lossy(stderr);
    let text = text.trim();
    if !text.is_empty() {
        eprintln!("STDERR:\n{text}\n");
    }
}
