#![cfg(unix)] // This feature is not available in Windows
#![cfg(feature = "cli")]

pub mod _utils;
pub use _utils::*;

use command_extra::CommandExtra;
use parallel_disk_usage::{
    data_tree::Reflection,
    json_data::{JsonData, JsonTree},
    size::Bytes,
};
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::{
    ops::{Add, Mul},
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
    let sizes = [200_000, 220_000, 310_000, 110_000, 210_000];
    let workspace = SampleWorkspace::simple_tree_with_some_hardlinks(sizes);

    let mut tree = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
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

    let actual_size = tree.size;
    let expected_size = Bytes::new(0)
        + inode_size("main")
        + inode_size("main/sources")
        + inode_size("main/internal-hardlinks")
        + file_size("no-hardlinks.txt")
        + 2usize * file_size("one-internal-hardlink.txt")
        + 3usize * file_size("two-internal-hardlinks.txt")
        + file_size("one-external-hardlink.txt")
        + 2usize * file_size("one-internal-one-external-hardlinks.txt");
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
                    + 2usize * file_size("two-internal-hardlinks.txt")
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

    assert_eq!(tree.shared.details, None);
    assert_eq!(tree.shared.summary, None);

    let visualization = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("main")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("STDOUT:\n{visualization}");
    assert!(!visualization.contains("Hardlinks detected!"));
}

#[test]
fn multiple_hardlinks_to_a_single_file() {
    let links = 10;
    let workspace = SampleWorkspace::multiple_hardlinks_to_a_single_file(100_000, links);

    let tree = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
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

    let actual_size = tree.size;
    let expected_size = workspace
        .join("file.txt")
        .pipe_as_ref(read_apparent_size)
        .mul(links + 1)
        .add(read_apparent_size(&workspace))
        .pipe(Bytes::new);
    assert_eq!(actual_size, expected_size);

    assert_eq!(tree.shared.details, None);
    assert_eq!(tree.shared.summary, None);
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
    let expected_size: Bytes = [
        inode_size("."),
        inode_size("no-hardlinks"),
        inode_size("some-hardlinks"),
        inode_size("only-hardlinks"),
        inode_size("only-hardlinks/exclusive"),
        inode_size("only-hardlinks/mixed"),
        inode_size("only-hardlinks/external"),
        file_size * files_per_branch, // no-hardlinks/*
        file_size
            * [
                3 * files_per_branch / 8,
                2 * files_per_branch / 8,
                files_per_branch / 8,
                files_per_branch / 8,
                files_per_branch * 4 / 8,
            ]
            .into_iter()
            .sum::<usize>(), // some-hardlinks/*
        file_size * (2 * files_per_branch), // only-hardlinks/exclusive/*
        file_size * (files_per_branch / 2 + 2 * files_per_branch / 2), // only-hardlinks/mixed/*
        file_size * files_per_branch, // only-hardlinks/external/*
    ]
    .into_iter()
    .sum();
    assert_eq!(actual_size, expected_size);

    assert_eq!(tree.shared.details, None);
    assert_eq!(tree.shared.summary, None);
}

#[test]
fn hardlinks_and_non_hardlinks() {
    let files_per_branch = 2 * 4;
    let workspace =
        SampleWorkspace::complex_tree_with_shared_and_unique_files(files_per_branch, 100_000);

    let tree = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--min-ratio=0")
        .with_arg("--quantity=apparent-size")
        .with_arg("--json-output")
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

    let actual_size = tree.size;
    let expected_size = [
        inode_size("some-hardlinks"),
        file_size * files_per_branch,  // file-{index}.txt
        file_size * (2usize + 1usize), // link0-file0.txt, link1-file0.txt, link0-file1.txt
    ]
    .into_iter()
    .sum();
    assert_eq!(actual_size, expected_size);

    assert_eq!(tree.shared.details, None);
    assert_eq!(tree.shared.summary, None);

    let visualization = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("some-hardlinks")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("STDOUT:\n{visualization}");
    assert!(!visualization.contains("Hardlinks detected!"));
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

    let actual_size = tree.size;
    let expected_size =
        inode_size("only-hardlinks/exclusive") + 2usize * file_size * files_per_branch;
    assert_eq!(actual_size, expected_size);

    assert_eq!(tree.shared.details, None);
    assert_eq!(tree.shared.summary, None);

    let visualization = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("only-hardlinks/exclusive")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("STDOUT:\n{visualization}");
    assert!(!visualization.contains("Hardlinks detected!"));
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

    let actual_size = tree.size;
    let expected_size =
        inode_size("only-hardlinks/mixed") + file_size * (files_per_branch + files_per_branch / 2);
    assert_eq!(actual_size, expected_size);

    assert_eq!(tree.shared.details, None);
    assert_eq!(tree.shared.summary, None);

    let visualization = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("only-hardlinks/mixed")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("STDOUT:\n{visualization}");
    assert!(!visualization.contains("Hardlinks detected!"));
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

    let actual_size = tree.size;
    let expected_size = inode_size("only-hardlinks/external") + file_size * files_per_branch;
    assert_eq!(actual_size, expected_size);

    assert_eq!(tree.shared.details, None);
    assert_eq!(tree.shared.summary, None);

    let visualization = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("only-hardlinks/external")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("STDOUT:\n{visualization}");
    assert!(!visualization.contains("Hardlinks detected!"));
}
