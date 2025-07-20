use build_fs_tree::{dir, file, Build, MergeableFileSystemTree};
use command_extra::CommandExtra;
use derive_more::{AsRef, Deref};
use parallel_disk_usage::{
    data_tree::{DataTree, DataTreeReflection},
    fs_tree_builder::FsTreeBuilder,
    get_size::{self, GetSize},
    hardlink::HardlinkIgnorant,
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
    fs::{create_dir, metadata, remove_dir_all, symlink_metadata},
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

/// POSIX-exclusive functions
#[cfg(unix)]
impl SampleWorkspace {
    /// Set up a temporary directory for tests.
    ///
    /// This directory would have a single file being hard-linked multiple times.
    pub fn multiple_hardlinks_to_a_single_file(bytes: usize, links: u64) -> Self {
        use std::fs::{hard_link, write as write_file};
        let temp = Temp::new_dir().expect("create working directory for sample workspace");

        let file_path = temp.join("file.txt");
        write_file(&file_path, "a".repeat(bytes)).expect("create file.txt");

        for num in 0..links {
            hard_link(&file_path, temp.join(format!("link.{num}")))
                .unwrap_or_else(|error| panic!("Failed to create 'link.{num}': {error}"));
        }

        SampleWorkspace(temp)
    }

    /// Set up a temporary directory for tests.
    ///
    /// The tree in this tests have a diverse types of files, both shared (hardlinks)
    /// and unique (non-hardlinks).
    pub fn complex_tree_with_shared_and_unique_files(
        files_per_branch: usize,
        bytes_per_file: usize,
    ) -> Self {
        use std::fs::{create_dir_all, hard_link, write as write_file};

        let whole = files_per_branch;
        let half = files_per_branch / 2;
        let quarter = files_per_branch / 4;
        let half_quarter = files_per_branch / 8;
        let temp = Temp::new_dir().expect("create working directory for sample workspace");

        temp.join("no-hardlinks")
            .pipe(create_dir_all)
            .expect("create no-hardlinks");
        temp.join("some-hardlinks")
            .pipe(create_dir_all)
            .expect("create some-hardlinks");
        temp.join("only-hardlinks/exclusive")
            .pipe(create_dir_all)
            .expect("create only-hardlinks/exclusive");
        temp.join("only-hardlinks/mixed")
            .pipe(create_dir_all)
            .expect("create only-hardlinks/mixed");
        temp.join("only-hardlinks/external")
            .pipe(create_dir_all)
            .expect("create only-hardlinks/external");

        // Create files in no-hardlinks.
        // There will be no files with nlink > 1.
        (0..files_per_branch).par_bridge().for_each(|index| {
            let file_name = format!("file-{index}.txt");
            let file_path = temp.join("no-hardlinks").join(file_name);
            if let Err(error) = write_file(&file_path, "a".repeat(bytes_per_file)) {
                panic!("Failed to write {bytes_per_file} bytes into {file_path:?}: {error}");
            }
        });

        // Create files in some-hardlinks.
        // Let's divide the files into 8 equal groups.
        // Each file in the first group will have 2 exclusive links.
        // Each file in the second group will have 1 exclusive link.
        // Each file in the third and fourth groups will have no links.
        // Each file in the remaining groups is PLANNED to have 1 external link from only-hardlinks/mixed.
        (0..whole).par_bridge().for_each(|file_index| {
            let file_name = format!("file-{file_index}.txt");
            let file_path = temp.join("some-hardlinks").join(file_name);
            if let Err(error) = write_file(&file_path, "a".repeat(bytes_per_file)) {
                panic!("Failed to write {bytes_per_file} bytes into {file_path:?}: {error}");
            }

            let link_count =
                ((file_index < quarter) as usize) + ((file_index < half_quarter) as usize);

            for link_index in 0..link_count {
                let link_name = format!("link{link_index}-file{file_index}.txt");
                let link_path = temp.join("some-hardlinks").join(link_name);
                if let Err(error) = hard_link(&file_path, &link_path) {
                    panic!("Failed to link {file_path:?} to {link_path:?}: {error}");
                }
            }
        });

        // Create files in only-hardlinks/exclusive.
        // Each file in this directory will have 1 exclusive link.
        (0..whole).par_bridge().for_each(|index| {
            let file_name = format!("file-{index}.txt");
            let file_path = temp.join("only-hardlinks/exclusive").join(file_name);
            if let Err(error) = write_file(&file_path, "a".repeat(bytes_per_file)) {
                panic!("Failed to write {bytes_per_file} bytes into {file_path:?}: {error}");
            }
            let link_name = format!("link-{index}.txt");
            let link_path = temp.join("only-hardlinks/exclusive").join(link_name);
            if let Err(error) = hard_link(&file_path, &link_path) {
                panic!("Failed to link {file_path:?} to {link_path:?}: {error}");
            }
        });

        // Create links in only-hardlinks/mixed.
        // Let's divide the PLANNED links into 2 equal groups.
        // Each link in the first group is PLANNED to share with only-hardlinks/external.
        // Each link in the second group is exclusive.
        (half..whole).par_bridge().for_each(|index| {
            let file_name = format!("link0-{index}.txt");
            let file_path = temp.join("only-hardlinks/mixed").join(file_name);
            if let Err(error) = write_file(&file_path, "a".repeat(bytes_per_file)) {
                panic!("Failed to write {bytes_per_file} bytes to {file_path:?}: {error}");
            }

            let link_name = format!("link1-{index}.txt");
            let link_path = temp.join("only-hardlinks/mixed").join(link_name);
            if let Err(error) = hard_link(&file_path, &link_path) {
                panic!("Failed to link {file_path:?} to {link_path:?}: {error}");
            }
        });

        // Create links in only-hardlinks/external
        // Let's divide the links into 2 equal groups.
        // The first group will share with only-hardlinks/mixed.
        // The second group will share with some-hardlinks.
        (0..whole).par_bridge().for_each(|index| {
            let link_name = format!("linkX-{index}.txt");
            let link_path = temp.join("only-hardlinks/external").join(link_name);

            let file_path = if index <= half {
                let file_name = format!("link0-{index}.txt"); // file name from only-hardlinks/mixed
                let file_path = temp.join("only-hardlinks/mixed").join(file_name);
                if let Err(error) = write_file(&file_path, "a".repeat(bytes_per_file)) {
                    panic!("Failed to write {bytes_per_file} bytes to {file_path:?}: {error}");
                }
                file_path
            } else {
                let file_name = format!("file-{index}.txt"); // file name from some-hardlinks
                temp.join("some-hardlinks").join(file_name)
            };

            if let Err(error) = hard_link(&file_path, &link_path) {
                panic!("Failed to link {file_path:?} to {link_path:?}: {error}");
            }
        });

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
            hardlinks_recorder: &HardlinkIgnorant,
            reporter: &ErrorOnlyReporter::new(|error| {
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

/// Read [apparent size](std::fs::Metadata::len) of a path.
pub fn read_apparent_size(path: &Path) -> u64 {
    path.pipe(symlink_metadata)
        .unwrap_or_else(|error| panic!("Can't read metadata at {path:?}: {error}"))
        .len()
}
