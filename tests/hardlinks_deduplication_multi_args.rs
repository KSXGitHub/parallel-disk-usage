#![cfg(unix)] // This feature is not available in Windows
#![cfg(feature = "cli")]

pub mod _utils;
pub use _utils::*;

use command_extra::CommandExtra;
use into_sorted::IntoSorted;
use itertools::Itertools;
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
use std::{
    collections::HashSet,
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
        .with_arg("main/sources")
        .with_arg("main/internal-hardlinks")
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
        + inode_size("main/sources")
        + inode_size("main/internal-hardlinks")
        + file_size("no-hardlinks.txt")
        + file_size("one-internal-hardlink.txt")
        + file_size("two-internal-hardlinks.txt")
        + file_size("one-external-hardlink.txt")
        + file_size("one-internal-one-external-hardlinks.txt");
    assert_eq!(actual_size, expected_size);

    let actual_tree = &tree.tree;
    let expected_tree = {
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
            .expect("get tree of bytes")
            .tree;
        sort_reflection_by(&mut tree, |a, b| a.name.cmp(&b.name));
        tree.name = "(total)".to_string();
        tree.size = expected_size;
        for child in &mut tree.children {
            let name = match child.name.as_str() {
                "sources" => "main/sources",
                "internal-hardlinks" => "main/internal-hardlinks",
                name => panic!("Unexpected name: {name:?}"),
            };
            child.name = name.to_string();
        }
        tree
    };
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
        .with_arg("main/sources")
        .with_arg("main/internal-hardlinks")
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
    let args = ["file.txt", "link.3", "link.5"];
    let workspace = SampleWorkspace::multiple_hardlinks_to_a_single_file(100_000, links);

    let tree = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--deduplicate-hardlinks")
        .with_arg("--json-output")
        .with_args(args)
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
    let expected_size = file_size;
    assert_eq!(actual_size, expected_size);

    let actual_children = tree
        .children
        .clone()
        .into_sorted_by(|a, b| a.name.cmp(&b.name));
    let expected_children = args.map(|name| Reflection {
        name: name.to_string(),
        size: file_size,
        children: Vec::new(),
    });
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
        paths: args
            .map(PathBuf::from)
            .pipe(HashSet::from)
            .pipe(LinkPathListReflection),
    }];
    assert_eq!(actual_shared_details, expected_shared_details);

    let actual_shared_summary = tree.shared.summary;
    let expected_shared_summary = Summary::default()
        .with_inodes(1)
        .with_exclusive_inodes(0)
        .with_all_links(1 + links)
        .with_detected_links(args.len())
        .with_exclusive_links(0)
        .with_shared_size(file_size)
        .with_exclusive_shared_size(Bytes::new(0))
        .pipe(Some);
    assert_eq!(actual_shared_summary, expected_shared_summary);

    let visualization = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--deduplicate-hardlinks")
        .with_args(args)
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
        writeln!(summary, "* Number of shared inodes: 1").unwrap();
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
fn multiple_duplicated_arguments() {
    #![expect(clippy::identity_op)]

    let sizes = [200_000, 220_000, 310_000, 110_000, 210_000];
    let workspace = SampleWorkspace::simple_tree_with_some_symlinks_and_hardlinks(sizes);

    let mut tree = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--deduplicate-hardlinks")
        .with_arg("--json-output")
        .with_arg("main/sources") // expected to be kept
        .with_arg("main/main-itself/sources") // expected to be removed
        .with_arg("workspace-itself/main/parent-of-main/main-mirror/internal-hardlinks") // expected to be kept
        .with_arg("main/internal-hardlinks") // expected to be removed
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
            .map(PathBuf::from)
            .collect::<HashSet<_>>()
            .pipe(LinkPathListReflection)
    };

    let actual_size = tree.size;
    let expected_size = Bytes::new(0)
        + inode_size("main/sources")
        + inode_size("main/internal-hardlinks")
        + file_size("no-hardlinks.txt")
        + file_size("one-internal-hardlink.txt")
        + file_size("two-internal-hardlinks.txt")
        + file_size("one-external-hardlink.txt")
        + file_size("one-internal-one-external-hardlinks.txt");
    assert_eq!(actual_size, expected_size);

    let actual_tree = &tree.tree;
    let expected_tree = {
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
            .expect("get tree of bytes")
            .tree;
        tree.name = "(total)".to_string();
        tree.size = expected_size;
        for child in &mut tree.children {
            let name = match child.name.as_str() {
                "sources" => "main/sources",
                "internal-hardlinks" => {
                    "workspace-itself/main/parent-of-main/main-mirror/internal-hardlinks"
                }
                name => panic!("Unexpected name: {name:?}"),
            };
            child.name = name.to_string();
        }
        sort_reflection_by(&mut tree, |a, b| a.name.cmp(&b.name));
        tree
    };
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
                "main/sources/one-internal-hardlink.txt",
                "workspace-itself/main/parent-of-main/main-mirror/internal-hardlinks/link-0.txt",
            ]),
        },
        ReflectionEntry {
            ino: file_inode("two-internal-hardlinks.txt"),
            size: file_size("two-internal-hardlinks.txt"),
            links: 1 + 2,
            paths: shared_paths(&[
                "main/sources/two-internal-hardlinks.txt",
                "workspace-itself/main/parent-of-main/main-mirror/internal-hardlinks/link-1a.txt",
                "workspace-itself/main/parent-of-main/main-mirror/internal-hardlinks/link-1b.txt",
            ]),
        },
        ReflectionEntry {
            ino: file_inode("one-external-hardlink.txt"),
            size: file_size("one-external-hardlink.txt"),
            links: 1 + 1,
            paths: shared_paths(&["main/sources/one-external-hardlink.txt"]),
        },
        ReflectionEntry {
            ino: file_inode("one-internal-one-external-hardlinks.txt"),
            size: file_size("one-internal-one-external-hardlinks.txt"),
            links: 1 + 1 + 1,
            paths: shared_paths(&[
                "main/sources/one-internal-one-external-hardlinks.txt",
                "workspace-itself/main/parent-of-main/main-mirror/internal-hardlinks/link-3a.txt",
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
        .with_arg("main/sources")
        .with_arg("main/internal-hardlinks")
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
