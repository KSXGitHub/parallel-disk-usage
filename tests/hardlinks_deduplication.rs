#![cfg(unix)] // This feature is not available in Windows
#![cfg(feature = "cli")]

pub mod _utils;
pub use _utils::*;

use command_extra::CommandExtra;
use itertools::Itertools;
use normalize_path::NormalizePath;
use parallel_disk_usage::{
    bytes_format::BytesFormat,
    data_tree::Reflection,
    hardlink::{
        hardlink_list::{reflection::ReflectionEntry, Summary},
        LinkPathListReflection,
    },
    inode::InodeNumber,
    json_data::{JsonData, JsonTree},
    size::Bytes,
};
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use rayon::prelude::*;
use std::{
    collections::HashSet,
    iter,
    ops::Add,
    path::PathBuf,
    process::{Command, Stdio},
};

fn stdio(command: Command) -> Command {
    command
        .with_stdin(Stdio::null())
        .with_stdout(Stdio::piped())
        .with_stderr(Stdio::piped())
}

#[test]
fn simple_tree_with_some_hardlinks() {
    #![expect(clippy::identity_op)]

    let sizes = [200_000, 220_000, 310_000, 110_000, 210_000];
    let workspace = SampleWorkspace::simple_tree_with_some_hardlinks(sizes);

    let mut tree = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--deduplicate-hardlinks")
        .with_arg("--json-output")
        .with_arg("main")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text)
        .pipe_as_ref(serde_json::from_str::<JsonData>)
        .expect("parse stdout as JsonData")
        .body
        .pipe(JsonTree::<Bytes>::try_from)
        .expect("get tree of bytes");
    sort_reflection_by(&mut tree, |a, b| a.name.cmp(&b.name));
    let tree = tree;

    let file_size = |name: &str| {
        workspace
            .join("main/sources")
            .join(name)
            .pipe_as_ref(read_apparent_size)
            .pipe(Bytes::new)
    };

    let inode_size = |path: &str| {
        workspace
            .join(path)
            .pipe_as_ref(read_apparent_size)
            .pipe(Bytes::new)
    };

    let file_inode = |name: &str| {
        workspace
            .join("main/sources")
            .join(name)
            .pipe_as_ref(read_inode_number)
            .pipe(InodeNumber::from)
    };

    let shared_paths = |suffices: &[&str]| {
        suffices
            .iter()
            .map(|suffix| PathBuf::from("main").join(suffix))
            .collect::<HashSet<_>>()
            .pipe(LinkPathListReflection)
    };

    let actual_size = tree.size;
    let expected_size = Bytes::new(0)
        + inode_size("main")
        + inode_size("main/sources")
        + inode_size("main/internal-hardlinks")
        + file_size("no-hardlinks.txt")
        + file_size("one-internal-hardlink.txt")
        + file_size("two-internal-hardlinks.txt")
        + file_size("one-external-hardlink.txt")
        + file_size("one-internal-one-external-hardlinks.txt");
    assert_eq!(actual_size, expected_size);

    let actual_tree = &tree.tree;
    let mut expected_tree = Reflection {
        name: "main".to_string(),
        size: expected_size,
        children: vec![
            Reflection {
                name: "sources".to_string(),
                size: inode_size("main/sources")
                    + file_size("no-hardlinks.txt")
                    + file_size("one-internal-hardlink.txt")
                    + file_size("two-internal-hardlinks.txt")
                    + file_size("one-external-hardlink.txt")
                    + file_size("one-internal-one-external-hardlinks.txt"),
                children: vec![
                    Reflection {
                        name: "no-hardlinks.txt".to_string(),
                        size: file_size("no-hardlinks.txt"),
                        children: Vec::new(),
                    },
                    Reflection {
                        name: "one-internal-hardlink.txt".to_string(),
                        size: file_size("one-internal-hardlink.txt"),
                        children: Vec::new(),
                    },
                    Reflection {
                        name: "two-internal-hardlinks.txt".to_string(),
                        size: file_size("two-internal-hardlinks.txt"),
                        children: Vec::new(),
                    },
                    Reflection {
                        name: "one-external-hardlink.txt".to_string(),
                        size: file_size("one-external-hardlink.txt"),
                        children: Vec::new(),
                    },
                    Reflection {
                        name: "one-internal-one-external-hardlinks.txt".to_string(),
                        size: file_size("one-internal-one-external-hardlinks.txt"),
                        children: Vec::new(),
                    },
                ],
            },
            Reflection {
                name: "internal-hardlinks".to_string(),
                size: inode_size("main/internal-hardlinks")
                    + file_size("one-internal-hardlink.txt")
                    + file_size("two-internal-hardlinks.txt")
                    + file_size("one-internal-one-external-hardlinks.txt"),
                children: vec![
                    Reflection {
                        name: "link-0.txt".to_string(),
                        size: file_size("one-internal-hardlink.txt"),
                        children: Vec::new(),
                    },
                    Reflection {
                        name: "link-1a.txt".to_string(),
                        size: file_size("two-internal-hardlinks.txt"),
                        children: Vec::new(),
                    },
                    Reflection {
                        name: "link-1b.txt".to_string(),
                        size: file_size("two-internal-hardlinks.txt"),
                        children: Vec::new(),
                    },
                    Reflection {
                        name: "link-3a.txt".to_string(),
                        size: file_size("one-internal-one-external-hardlinks.txt"),
                        children: Vec::new(),
                    },
                ],
            },
        ],
    };
    sort_reflection_by(&mut expected_tree, |a, b| a.name.cmp(&b.name));
    assert_eq!(actual_tree, &expected_tree);

    let actual_shared_details: Vec<_> = tree
        .shared
        .details
        .as_ref()
        .expect("get details")
        .iter()
        .cloned()
        .collect();
    let expected_shared_details = [
        ReflectionEntry {
            ino: file_inode("one-internal-hardlink.txt"),
            size: file_size("one-internal-hardlink.txt"),
            links: 1 + 1,
            paths: shared_paths(&[
                "sources/one-internal-hardlink.txt",
                "internal-hardlinks/link-0.txt",
            ]),
        },
        ReflectionEntry {
            ino: file_inode("two-internal-hardlinks.txt"),
            size: file_size("two-internal-hardlinks.txt"),
            links: 1 + 2,
            paths: shared_paths(&[
                "sources/two-internal-hardlinks.txt",
                "internal-hardlinks/link-1a.txt",
                "internal-hardlinks/link-1b.txt",
            ]),
        },
        ReflectionEntry {
            ino: file_inode("one-external-hardlink.txt"),
            size: file_size("one-external-hardlink.txt"),
            links: 1 + 1,
            paths: shared_paths(&["sources/one-external-hardlink.txt"]),
        },
        ReflectionEntry {
            ino: file_inode("one-internal-one-external-hardlinks.txt"),
            size: file_size("one-internal-one-external-hardlinks.txt"),
            links: 1 + 1 + 1,
            paths: shared_paths(&[
                "sources/one-internal-one-external-hardlinks.txt",
                "internal-hardlinks/link-3a.txt",
            ]),
        },
    ]
    .into_sorted_by_key(|item| u64::from(item.ino));
    assert_eq!(actual_shared_details, expected_shared_details);

    let actual_shared_summary = tree.shared.summary;
    let expected_shared_summary = Summary::default()
        .with_inodes(0 + 1 + 1 + 1 + 1)
        .with_exclusive_inodes(0 + 1 + 1 + 0 + 0)
        .with_all_links(0 + 2 + 3 + 2 + 3)
        .with_detected_links(0 + 2 + 3 + 1 + 2)
        .with_exclusive_links(0 + 2 + 3 + 0 + 0)
        .with_shared_size(
            Bytes::new(0)
                + file_size("one-internal-hardlink.txt")
                + file_size("two-internal-hardlinks.txt")
                + file_size("one-external-hardlink.txt")
                + file_size("one-internal-one-external-hardlinks.txt"),
        )
        .with_exclusive_shared_size(
            Bytes::new(0)
                + file_size("one-internal-hardlink.txt")
                + file_size("two-internal-hardlinks.txt"),
        )
        .pipe(Some);
    assert_eq!(actual_shared_summary, expected_shared_summary);

    let visualization = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--deduplicate-hardlinks")
        .with_arg("main")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("STDOUT:\n{visualization}");
    let actual_hardlinks_summary = visualization
        .lines()
        .skip_while(|line| !line.starts_with("Hardlinks detected!"))
        .join("\n");
    let expected_hardlinks_summary = {
        use parallel_disk_usage::size::Size;
        use std::fmt::Write;
        let mut summary = String::new();
        writeln!(
            summary,
            "Hardlinks detected! Some files have links outside this tree",
        )
        .unwrap();
        writeln!(
            summary,
            "* Number of shared inodes: {total} total, {exclusive} exclusive",
            total = expected_shared_summary.unwrap().inodes,
            exclusive = expected_shared_summary.unwrap().exclusive_inodes,
        )
        .unwrap();
        writeln!(
            summary,
            "* Total number of links: {total} total, {detected} detected, {exclusive} exclusive",
            total = expected_shared_summary.unwrap().all_links,
            detected = expected_shared_summary.unwrap().detected_links,
            exclusive = expected_shared_summary.unwrap().exclusive_links,
        )
        .unwrap();
        writeln!(
            summary,
            "* Total shared size: {total} total, {exclusive} exclusive",
            total = expected_shared_summary
                .unwrap()
                .shared_size
                .display(BytesFormat::MetricUnits),
            exclusive = expected_shared_summary
                .unwrap()
                .exclusive_shared_size
                .display(BytesFormat::MetricUnits),
        )
        .unwrap();
        summary
    };
    assert_eq!(
        actual_hardlinks_summary.trim_end(),
        expected_hardlinks_summary.trim_end(),
    );
}

#[test]
fn multiple_hardlinks_to_a_single_file() {
    let links = 10;
    let workspace = SampleWorkspace::multiple_hardlinks_to_a_single_file(100_000, links);

    let tree = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--deduplicate-hardlinks")
        .with_arg("--json-output")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text)
        .pipe_as_ref(serde_json::from_str::<JsonData>)
        .expect("parse stdout as JsonData")
        .body
        .pipe(JsonTree::<Bytes>::try_from)
        .expect("get tree of bytes");

    let file_size = workspace
        .join("file.txt")
        .pipe_as_ref(read_apparent_size)
        .pipe(Bytes::new);

    let file_inode = workspace
        .join("file.txt")
        .pipe_as_ref(read_inode_number)
        .pipe(InodeNumber::from);

    let actual_size = tree.size;
    let expected_size = workspace
        .pipe_as_ref(read_apparent_size)
        .pipe(Bytes::new)
        .add(file_size);
    assert_eq!(actual_size, expected_size);

    let actual_children = tree
        .children
        .clone()
        .into_sorted_by(|a, b| a.name.cmp(&b.name));
    let expected_children: Vec<_> = {
        let links = (0..links).map(|num| format!("link.{num}"));
        let node = |name| Reflection {
            name,
            size: file_size,
            children: Vec::new(),
        };
        "file.txt"
            .to_string()
            .pipe(iter::once)
            .chain(links)
            .map(node)
            .collect()
    };
    assert_eq!(actual_children, expected_children);

    let actual_shared_details: Vec<_> = tree
        .shared
        .details
        .as_ref()
        .expect("get details")
        .iter()
        .cloned()
        .collect();
    let expected_shared_details = [ReflectionEntry {
        ino: file_inode,
        size: file_size,
        links: 1 + links,
        paths: (0..links)
            .map(|num| format!("./link.{num}"))
            .chain("./file.txt".to_string().pipe(iter::once))
            .map(PathBuf::from)
            .collect::<HashSet<_>>()
            .pipe(LinkPathListReflection),
    }];
    assert_eq!(actual_shared_details, expected_shared_details);

    let actual_shared_summary = tree.shared.summary;
    let expected_shared_summary = Summary::default()
        .with_inodes(1)
        .with_exclusive_inodes(1)
        .with_all_links(1 + links)
        .with_detected_links(1 + links as usize)
        .with_exclusive_links(1 + links as usize)
        .with_shared_size(file_size)
        .with_exclusive_shared_size(file_size)
        .pipe(Some);
    assert_eq!(actual_shared_summary, expected_shared_summary);

    let visualization = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--deduplicate-hardlinks")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("STDOUT:\n{visualization}");
    let actual_hardlinks_summary = visualization
        .lines()
        .skip_while(|line| !line.starts_with("Hardlinks detected!"))
        .join("\n");
    let expected_hardlinks_summary = {
        use parallel_disk_usage::size::Size;
        use std::fmt::Write;
        let mut summary = String::new();
        writeln!(
            summary,
            "Hardlinks detected! No files have links outside this tree",
        )
        .unwrap();
        writeln!(summary, "* Number of shared inodes: 1").unwrap();
        writeln!(summary, "* Total number of links: 11").unwrap();
        writeln!(
            summary,
            "* Total shared size: {}",
            file_size.display(BytesFormat::MetricUnits),
        )
        .unwrap();
        summary
    };
    assert_eq!(
        actual_hardlinks_summary.trim_end(),
        expected_hardlinks_summary.trim_end(),
    );
}

#[test]
fn complex_tree_with_shared_and_unique_files() {
    let files_per_branch = 2 * 3 * 4;
    let workspace =
        SampleWorkspace::complex_tree_with_shared_and_unique_files(files_per_branch, 100_000);

    let tree = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--min-ratio=0")
        .with_arg("--quantity=apparent-size")
        .with_arg("--deduplicate-hardlinks")
        .with_arg("--json-output")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text)
        .pipe_as_ref(serde_json::from_str::<JsonData>)
        .expect("parse stdout as JsonData")
        .body
        .pipe(JsonTree::<Bytes>::try_from)
        .expect("get tree of bytes");

    let file_size = workspace
        .join("no-hardlinks/file-0.txt")
        .pipe_as_ref(read_apparent_size)
        .pipe(Bytes::new);

    let inode_size = |path: &str| {
        workspace
            .join(path)
            .pipe_as_ref(read_apparent_size)
            .pipe(Bytes::new)
    };

    let actual_size = tree.size;

    // The following formula treat the first file as "real" and
    // the non-first file with the same inode as "fake" for ease
    // of reasoning.
    // It should still produce the same result as the proper
    // deduplication formula however.
    #[expect(clippy::erasing_op)]
    let expected_size: Bytes = [
        inode_size("."),
        inode_size("no-hardlinks"),
        inode_size("some-hardlinks"),
        inode_size("only-hardlinks"),
        inode_size("only-hardlinks/exclusive"),
        inode_size("only-hardlinks/mixed"),
        inode_size("only-hardlinks/external"),
        file_size * files_per_branch, // no-hardlinks/*
        file_size * files_per_branch, // some-hardlinks/*
        file_size * files_per_branch, // only-hardlinks/exclusive/*
        file_size * files_per_branch, // only-hardlinks/mixed/*
        file_size * 0usize,           // only-hardlinks/external/*
    ]
    .into_iter()
    .sum();

    assert_eq!(actual_size, expected_size);

    fn starts_with_path(item: &ReflectionEntry<Bytes>, prefix: &str) -> bool {
        item.paths
            .0
            .iter()
            .any(|path| path.normalize().starts_with(prefix))
    }

    // Files with nlink <= 1 shouldn't appear
    {
        let actual = tree
            .shared
            .details
            .as_ref()
            .expect("get details")
            .iter()
            .find(|item| item.links <= 1)
            .cloned();
        assert_eq!(actual, None);
    }

    // All entries are sorted by their inodes and have unique inodes
    {
        let actual: Vec<_> = tree
            .shared
            .details
            .as_ref()
            .expect("get details")
            .iter()
            .map(|item| item.ino)
            .map(u64::from)
            .collect();
        let expected = actual
            .clone()
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .into_sorted();
        assert_eq!(actual, expected);
    }

    // No files from no-hardlinks should appear
    {
        let actual = tree
            .shared
            .details
            .as_ref()
            .expect("get details")
            .iter()
            .find(|item| starts_with_path(item, "no-hardlinks"))
            .cloned();
        assert_eq!(actual, None);
    }

    // This file in some-hardlinks should have 2 links created for it
    {
        let actual = tree
            .shared
            .details
            .as_ref()
            .expect("get details")
            .iter()
            .find(|item| starts_with_path(item, "some-hardlinks/file-0.txt"))
            .cloned();
        let expected = Some(ReflectionEntry {
            ino: workspace
                .join("some-hardlinks/file-0.txt")
                .pipe_as_ref(read_inode_number)
                .pipe(InodeNumber::from),
            size: workspace
                .join("some-hardlinks/file-0.txt")
                .pipe_as_ref(read_apparent_size)
                .pipe(Bytes::new),
            links: 3,
            paths: ["file-0.txt", "link0-file0.txt", "link1-file0.txt"]
                .map(|name| PathBuf::from(".").join("some-hardlinks").join(name))
                .pipe(HashSet::from)
                .pipe(LinkPathListReflection),
        });
        assert_eq!(actual, expected);
    }

    // This file in some-hardlinks should have 1 link created for it
    {
        let file_index = files_per_branch / 8;
        let actual = tree
            .shared
            .details
            .as_ref()
            .expect("get details")
            .iter()
            .find(|item| starts_with_path(item, &format!("some-hardlinks/file-{file_index}.txt")))
            .cloned();
        let expected = Some(ReflectionEntry {
            ino: workspace
                .join(format!("some-hardlinks/file-{file_index}.txt"))
                .pipe_as_ref(read_inode_number)
                .pipe(InodeNumber::from),
            size: workspace
                .join(format!("some-hardlinks/file-{file_index}.txt"))
                .pipe_as_ref(read_apparent_size)
                .pipe(Bytes::new),
            links: 2,
            paths: [
                format!("file-{file_index}.txt"),
                format!("link0-file{file_index}.txt"),
            ]
            .map(|name| PathBuf::from(".").join("some-hardlinks").join(name))
            .pipe(HashSet::from)
            .pipe(LinkPathListReflection),
        });
        assert_eq!(actual, expected);
    }

    let actual_shared_summary = tree.shared.summary;
    let expected_shared_summary = {
        // The following formula treat the first file as "real" and
        // the non-first file with the same inode as "fake" for ease
        // of reasoning.
        // It should still produce the same result as the proper
        // deduplication formula however.
        let inodes = [
            0,                                               // no-hardlinks/*
            2 * files_per_branch / 8 + files_per_branch / 2, // some-hardlinks/*
            files_per_branch,                                // only-hardlinks/exclusive/*
            files_per_branch,                                // only-hardlinks/mixed/*
            0,                                               // only-hardlinks/external/*
        ]
        .into_iter()
        .sum();
        let all_links = [
            0,                                                                              // no-hardlinks/*
            3 * files_per_branch / 8 + 2 * files_per_branch / 8 + 2 * files_per_branch / 2, // some-hardlinks/*
            2 * files_per_branch, // only-hardlinks/exclusive/*
            2 * files_per_branch / 2 + 2 * files_per_branch / 2, // only-hardlinks/mixed/*
            0,                    // only-hardlinks/external/*
        ]
        .into_iter()
        .sum::<usize>() as u64;
        let shared_size = file_size * inodes;
        Summary::default()
            .with_inodes(inodes)
            .with_exclusive_inodes(inodes)
            .with_all_links(all_links)
            .with_detected_links(all_links as usize)
            .with_exclusive_links(all_links as usize)
            .with_shared_size(shared_size)
            .with_exclusive_shared_size(shared_size)
            .pipe(Some)
    };
    assert_eq!(actual_shared_summary, expected_shared_summary);
}

#[test]
fn hardlinks_and_non_hardlinks() {
    #![expect(clippy::identity_op)]

    let files_per_branch = 2 * 4;
    let workspace =
        SampleWorkspace::complex_tree_with_shared_and_unique_files(files_per_branch, 100_000);

    let tree = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--min-ratio=0")
        .with_arg("--quantity=apparent-size")
        .with_arg("--json-output")
        .with_arg("--deduplicate-hardlinks")
        .with_arg("some-hardlinks")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text)
        .pipe_as_ref(serde_json::from_str::<JsonData>)
        .expect("parse stdout as JsonData")
        .body
        .pipe(JsonTree::<Bytes>::try_from)
        .expect("get tree of bytes");

    let file_size = workspace
        .join("some-hardlinks/file-0.txt")
        .pipe_as_ref(read_apparent_size)
        .pipe(Bytes::new);

    let inode_size = |path: &str| {
        workspace
            .join(path)
            .pipe_as_ref(read_apparent_size)
            .pipe(Bytes::new)
    };

    let file_inode = |name: &str| {
        workspace
            .join("some-hardlinks")
            .join(name)
            .pipe_as_ref(read_inode_number)
            .pipe(InodeNumber::from)
    };

    let shared_paths = |file_names: &[&str]| {
        file_names
            .iter()
            .map(|file_name| PathBuf::from("some-hardlinks").join(file_name))
            .collect::<HashSet<_>>()
            .pipe(LinkPathListReflection)
    };

    let actual_size = tree.size;
    let expected_size = inode_size("some-hardlinks") + file_size * files_per_branch;
    assert_eq!(actual_size, expected_size);

    let actual_shared_details: Vec<_> = tree
        .shared
        .details
        .as_ref()
        .unwrap()
        .iter()
        .cloned()
        .collect();
    let expected_shared_details = [
        ReflectionEntry {
            ino: file_inode("file-0.txt"),
            size: file_size,
            links: 3,
            paths: shared_paths(&["file-0.txt", "link0-file0.txt", "link1-file0.txt"]),
        },
        ReflectionEntry {
            ino: file_inode("file-1.txt"),
            size: file_size,
            links: 2,
            paths: shared_paths(&["file-1.txt", "link0-file1.txt"]),
        },
        // ... file-2.txt and file-3.txt don't have hardlinks so they shouldn't appear here ...
        ReflectionEntry {
            ino: file_inode("file-4.txt"),
            size: file_size,
            links: 2,
            paths: shared_paths(&["file-4.txt"]),
        },
        ReflectionEntry {
            ino: file_inode("file-5.txt"),
            size: file_size,
            links: 2,
            paths: shared_paths(&["file-5.txt"]),
        },
        ReflectionEntry {
            ino: file_inode("file-6.txt"),
            size: file_size,
            links: 2,
            paths: shared_paths(&["file-6.txt"]),
        },
        ReflectionEntry {
            ino: file_inode("file-7.txt"),
            size: file_size,
            links: 2,
            paths: shared_paths(&["file-7.txt"]),
        },
    ]
    .into_sorted_by_key(|item| u64::from(item.ino));
    assert_eq!(actual_shared_details, expected_shared_details);

    let actual_shared_summary = tree.shared.summary;
    let expected_shared_summary = {
        let inodes = expected_shared_details.len();
        let exclusive_inodes = 2;
        Summary::default()
            .with_inodes(inodes)
            .with_exclusive_inodes(exclusive_inodes)
            .with_all_links(3 + 2 + 4 * 2)
            .with_detected_links(3 + 2 + 4 * 1)
            .with_exclusive_links(3 + 2)
            .with_shared_size(inodes * file_size)
            .with_exclusive_shared_size(exclusive_inodes * file_size)
            .pipe(Some)
    };
    assert_eq!(actual_shared_summary, expected_shared_summary);
    assert_eq!(actual_shared_summary.unwrap().inodes, files_per_branch - 2);
    assert_eq!(
        actual_shared_summary.unwrap().all_links,
        actual_shared_details
            .iter()
            .map(|item| item.links)
            .sum::<u64>(),
    );
    assert_eq!(
        actual_shared_summary.unwrap().detected_links,
        actual_shared_details
            .iter()
            .map(|item| item.paths.len())
            .sum::<usize>(),
    );
    assert_eq!(
        actual_shared_summary.unwrap().exclusive_links,
        actual_shared_details
            .iter()
            .filter(|item| item.links == item.paths.len() as u64)
            .map(|item| item.links as usize)
            .sum::<usize>(),
    );

    let visualization = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--deduplicate-hardlinks")
        .with_arg("some-hardlinks")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("STDOUT:\n{visualization}");
    let actual_hardlinks_summary = visualization
        .lines()
        .skip_while(|line| !line.starts_with("Hardlinks detected!"))
        .join("\n");
    let expected_hardlinks_summary = {
        use parallel_disk_usage::size::Size;
        use std::fmt::Write;
        let mut summary = String::new();
        writeln!(
            summary,
            "Hardlinks detected! Some files have links outside this tree",
        )
        .unwrap();
        writeln!(
            summary,
            "* Number of shared inodes: {total} total, {exclusive} exclusive",
            total = expected_shared_summary.unwrap().inodes,
            exclusive = expected_shared_summary.unwrap().exclusive_inodes,
        )
        .unwrap();
        writeln!(
            summary,
            "* Total number of links: {total} total, {detected} detected, {exclusive} exclusive",
            total = expected_shared_summary.unwrap().all_links,
            detected = expected_shared_summary.unwrap().detected_links,
            exclusive = expected_shared_summary.unwrap().exclusive_links,
        )
        .unwrap();
        writeln!(
            summary,
            "* Total shared size: {total} total, {exclusive} exclusive",
            total = expected_shared_summary
                .unwrap()
                .shared_size
                .display(BytesFormat::MetricUnits),
            exclusive = expected_shared_summary
                .unwrap()
                .exclusive_shared_size
                .display(BytesFormat::MetricUnits),
        )
        .unwrap();
        summary
    };
    assert_eq!(
        actual_hardlinks_summary.trim_end(),
        expected_hardlinks_summary.trim_end(),
    );
}

#[test]
fn exclusive_hardlinks_only() {
    let files_per_branch = 2 * 4;
    let workspace =
        SampleWorkspace::complex_tree_with_shared_and_unique_files(files_per_branch, 100_000);

    let tree = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--min-ratio=0")
        .with_arg("--quantity=apparent-size")
        .with_arg("--json-output")
        .with_arg("--deduplicate-hardlinks")
        .with_arg("only-hardlinks/exclusive")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text)
        .pipe_as_ref(serde_json::from_str::<JsonData>)
        .expect("parse stdout as JsonData")
        .body
        .pipe(JsonTree::<Bytes>::try_from)
        .expect("get tree of bytes");

    let file_size = workspace
        .join("only-hardlinks/exclusive/file-0.txt")
        .pipe_as_ref(read_apparent_size)
        .pipe(Bytes::new);

    let inode_size = |path: &str| {
        workspace
            .join(path)
            .pipe_as_ref(read_apparent_size)
            .pipe(Bytes::new)
    };

    let file_inode = |name: &str| {
        workspace
            .join("only-hardlinks/exclusive")
            .join(name)
            .pipe_as_ref(read_inode_number)
            .pipe(InodeNumber::from)
    };

    let shared_paths = |file_names: &[&str]| {
        file_names
            .iter()
            .map(|file_name| PathBuf::from("only-hardlinks/exclusive").join(file_name))
            .collect::<HashSet<_>>()
            .pipe(LinkPathListReflection)
    };

    let actual_size = tree.size;
    let expected_size = inode_size("only-hardlinks/exclusive") + file_size * files_per_branch;
    assert_eq!(actual_size, expected_size);

    let actual_shared_details: Vec<_> = tree
        .shared
        .details
        .as_ref()
        .unwrap()
        .iter()
        .cloned()
        .collect();
    let expected_shared_details = (0..files_per_branch)
        .par_bridge()
        .map(|index| ReflectionEntry {
            ino: file_inode(&format!("file-{index}.txt")),
            size: file_size,
            links: 2,
            paths: shared_paths(&[&format!("file-{index}.txt"), &format!("link-{index}.txt")]),
        })
        .collect::<Vec<_>>()
        .into_sorted_by_key(|item: &ReflectionEntry<Bytes>| u64::from(item.ino));
    assert_eq!(actual_shared_details, expected_shared_details);

    let actual_shared_summary = tree.shared.summary;
    let expected_shared_summary = Summary::default()
        .with_inodes(files_per_branch)
        .with_exclusive_inodes(files_per_branch)
        .with_all_links(2 * files_per_branch as u64)
        .with_detected_links(2 * files_per_branch)
        .with_exclusive_links(2 * files_per_branch)
        .with_shared_size(files_per_branch * file_size)
        .with_exclusive_shared_size(files_per_branch * file_size)
        .pipe(Some);
    assert_eq!(actual_shared_summary, expected_shared_summary);

    let visualization = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--deduplicate-hardlinks")
        .with_arg("only-hardlinks/exclusive")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("STDOUT:\n{visualization}");
    let actual_hardlinks_summary = visualization
        .lines()
        .skip_while(|line| !line.starts_with("Hardlinks detected!"))
        .join("\n");
    let expected_hardlinks_summary = {
        use parallel_disk_usage::size::Size;
        use std::fmt::Write;
        let mut summary = String::new();
        writeln!(
            summary,
            "Hardlinks detected! No files have links outside this tree",
        )
        .unwrap();
        writeln!(summary, "* Number of shared inodes: {files_per_branch}").unwrap();
        writeln!(summary, "* Total number of links: {}", 2 * files_per_branch).unwrap();
        writeln!(
            summary,
            "* Total shared size: {}",
            (file_size * files_per_branch).display(BytesFormat::MetricUnits),
        )
        .unwrap();
        summary
    };
    assert_eq!(
        actual_hardlinks_summary.trim_end(),
        expected_hardlinks_summary.trim_end(),
    );
}

#[test]
fn exclusive_only_and_external_only_hardlinks() {
    let files_per_branch = 2 * 4;
    let workspace =
        SampleWorkspace::complex_tree_with_shared_and_unique_files(files_per_branch, 100_000);

    let tree = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--min-ratio=0")
        .with_arg("--quantity=apparent-size")
        .with_arg("--json-output")
        .with_arg("--deduplicate-hardlinks")
        .with_arg("only-hardlinks/mixed")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text)
        .pipe_as_ref(serde_json::from_str::<JsonData>)
        .expect("parse stdout as JsonData")
        .body
        .pipe(JsonTree::<Bytes>::try_from)
        .expect("get tree of bytes");

    let file_size = workspace
        .join("only-hardlinks/mixed/link0-0.txt")
        .pipe_as_ref(read_apparent_size)
        .pipe(Bytes::new);

    let inode_size = |path: &str| {
        workspace
            .join(path)
            .pipe_as_ref(read_apparent_size)
            .pipe(Bytes::new)
    };

    let file_inode = |name: &str| {
        workspace
            .join("only-hardlinks/mixed")
            .join(name)
            .pipe_as_ref(read_inode_number)
            .pipe(InodeNumber::from)
    };

    let shared_paths = |file_names: &[&str]| {
        file_names
            .iter()
            .map(|file_name| PathBuf::from("only-hardlinks/mixed").join(file_name))
            .collect::<HashSet<_>>()
            .pipe(LinkPathListReflection)
    };

    let actual_size = tree.size;
    let expected_size = inode_size("only-hardlinks/mixed") + file_size * files_per_branch;
    assert_eq!(actual_size, expected_size);

    let actual_shared_details: Vec<_> = tree
        .shared
        .details
        .as_ref()
        .unwrap()
        .iter()
        .cloned()
        .collect();
    let expected_shared_details = iter::empty()
        .par_bridge()
        .chain(
            (0..(files_per_branch / 2))
                .par_bridge()
                .map(|index| ReflectionEntry {
                    ino: file_inode(&format!("link0-{index}.txt")),
                    size: file_size,
                    links: 2,
                    paths: shared_paths(&[&format!("link0-{index}.txt")]),
                }),
        )
        .chain(
            ((files_per_branch / 2)..files_per_branch)
                .par_bridge()
                .map(|index| ReflectionEntry {
                    ino: file_inode(&format!("link0-{index}.txt")),
                    size: file_size,
                    links: 2,
                    paths: shared_paths(&[
                        &format!("link0-{index}.txt"),
                        &format!("link1-{index}.txt"),
                    ]),
                }),
        )
        .collect::<Vec<_>>()
        .into_sorted_by_key(|item: &ReflectionEntry<Bytes>| u64::from(item.ino));
    assert_eq!(actual_shared_details, expected_shared_details);

    let actual_shared_summary = tree.shared.summary;
    let expected_shared_summary = Summary::default()
        .with_inodes(files_per_branch)
        .with_exclusive_inodes(files_per_branch / 2)
        .with_all_links(2 * files_per_branch as u64)
        .with_detected_links(files_per_branch + files_per_branch / 2)
        .with_exclusive_links(files_per_branch * 2 / 2)
        .with_shared_size(files_per_branch * file_size)
        .with_exclusive_shared_size((files_per_branch / 2) * file_size)
        .pipe(Some);
    assert_eq!(actual_shared_summary, expected_shared_summary);

    let visualization = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--deduplicate-hardlinks")
        .with_arg("only-hardlinks/mixed")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("STDOUT:\n{visualization}");
    let actual_hardlinks_summary = visualization
        .lines()
        .skip_while(|line| !line.starts_with("Hardlinks detected!"))
        .join("\n");
    let expected_hardlinks_summary = {
        use parallel_disk_usage::size::Size;
        use std::fmt::Write;
        let mut summary = String::new();
        writeln!(
            summary,
            "Hardlinks detected! Some files have links outside this tree",
        )
        .unwrap();
        writeln!(
            summary,
            "* Number of shared inodes: {total} total, {exclusive} exclusive",
            total = expected_shared_summary.unwrap().inodes,
            exclusive = expected_shared_summary.unwrap().exclusive_inodes,
        )
        .unwrap();
        writeln!(
            summary,
            "* Total number of links: {total} total, {detected} detected, {exclusive} exclusive",
            total = expected_shared_summary.unwrap().all_links,
            detected = expected_shared_summary.unwrap().detected_links,
            exclusive = expected_shared_summary.unwrap().exclusive_links,
        )
        .unwrap();
        writeln!(
            summary,
            "* Total shared size: {total} total, {exclusive} exclusive",
            total = expected_shared_summary
                .unwrap()
                .shared_size
                .display(BytesFormat::MetricUnits),
            exclusive = expected_shared_summary
                .unwrap()
                .exclusive_shared_size
                .display(BytesFormat::MetricUnits),
        )
        .unwrap();
        summary
    };
    assert_eq!(
        actual_hardlinks_summary.trim_end(),
        expected_hardlinks_summary.trim_end(),
    );
}

#[test]
fn external_hardlinks_only() {
    let files_per_branch = 2 * 4;
    let workspace =
        SampleWorkspace::complex_tree_with_shared_and_unique_files(files_per_branch, 100_000);

    let tree = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--min-ratio=0")
        .with_arg("--quantity=apparent-size")
        .with_arg("--json-output")
        .with_arg("--deduplicate-hardlinks")
        .with_arg("only-hardlinks/external")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text)
        .pipe_as_ref(serde_json::from_str::<JsonData>)
        .expect("parse stdout as JsonData")
        .body
        .pipe(JsonTree::<Bytes>::try_from)
        .expect("get tree of bytes");

    let file_size = workspace
        .join("only-hardlinks/external/linkX-0.txt")
        .pipe_as_ref(read_apparent_size)
        .pipe(Bytes::new);

    let inode_size = |path: &str| {
        workspace
            .join(path)
            .pipe_as_ref(read_apparent_size)
            .pipe(Bytes::new)
    };

    let file_inode = |name: &str| {
        workspace
            .join("only-hardlinks/external")
            .join(name)
            .pipe_as_ref(read_inode_number)
            .pipe(InodeNumber::from)
    };

    let shared_paths = |file_names: &[&str]| {
        file_names
            .iter()
            .map(|file_name| PathBuf::from("only-hardlinks/external").join(file_name))
            .collect::<HashSet<_>>()
            .pipe(LinkPathListReflection)
    };

    let actual_size = tree.size;
    let expected_size = inode_size("only-hardlinks/external") + file_size * files_per_branch;
    assert_eq!(actual_size, expected_size);

    let actual_shared_details: Vec<_> = tree
        .shared
        .details
        .as_ref()
        .unwrap()
        .iter()
        .cloned()
        .collect();
    let expected_shared_details = (0..files_per_branch)
        .par_bridge()
        .map(|index| ReflectionEntry {
            ino: file_inode(&format!("linkX-{index}.txt")),
            size: file_size,
            links: 2,
            paths: shared_paths(&[&format!("linkX-{index}.txt")]),
        })
        .collect::<Vec<_>>()
        .into_sorted_by_key(|item: &ReflectionEntry<Bytes>| u64::from(item.ino));
    assert_eq!(actual_shared_details, expected_shared_details);

    let actual_shared_summary = tree.shared.summary;
    let expected_shared_summary = Summary::default()
        .with_inodes(files_per_branch)
        .with_exclusive_inodes(0)
        .with_all_links(2 * files_per_branch as u64)
        .with_detected_links(files_per_branch)
        .with_exclusive_links(0)
        .with_shared_size(files_per_branch * file_size)
        .with_exclusive_shared_size(Bytes::new(0))
        .pipe(Some);
    assert_eq!(actual_shared_summary, expected_shared_summary);

    let visualization = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--deduplicate-hardlinks")
        .with_arg("only-hardlinks/external")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("STDOUT:\n{visualization}");
    let actual_hardlinks_summary = visualization
        .lines()
        .skip_while(|line| !line.starts_with("Hardlinks detected!"))
        .join("\n");
    let expected_hardlinks_summary = {
        use parallel_disk_usage::size::Size;
        use std::fmt::Write;
        let mut summary = String::new();
        writeln!(
            summary,
            "Hardlinks detected! All hardlinks within this tree have links without",
        )
        .unwrap();
        writeln!(summary, "* Number of shared inodes: {files_per_branch}").unwrap();
        writeln!(
            summary,
            "* Total number of links: {total} total, {detected} detected",
            total = expected_shared_summary.unwrap().all_links,
            detected = expected_shared_summary.unwrap().detected_links,
        )
        .unwrap();
        writeln!(
            summary,
            "* Total shared size: {}",
            expected_shared_summary
                .unwrap()
                .shared_size
                .display(BytesFormat::MetricUnits),
        )
        .unwrap();
        summary
    };
    assert_eq!(
        actual_hardlinks_summary.trim_end(),
        expected_hardlinks_summary.trim_end(),
    );
}
