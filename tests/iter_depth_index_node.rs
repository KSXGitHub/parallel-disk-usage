#![cfg(test)]
pub mod _utils;
pub use _utils::*;

use dirt::{fs_tree_builder::FsTreeBuilder, reporter::ErrorOnlyReporter, size::Bytes, tree::Tree};
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::{ffi::OsString, fs::metadata, path::Path};

macro_rules! test_case {
    ( $(
        $(#[$attributes:meta])*
        $name:ident in $prefix:literal => $expected:expr;
    )* ) => { $(
        $(#[$attributes])*
        #[test]
        fn $name() {
            let workspace = SampleWorkspace::default();
            let prefix = workspace.join($prefix);
            let tree = FsTreeBuilder {
                get_data: |metadata| metadata.len().into(),
                reporter: ErrorOnlyReporter::new(|error| {
                    panic!("Unexpected call to report_error: {:?}", error)
                }),
                root: prefix.clone(),
            }
            .pipe(Tree::<OsString, Bytes>::from)
            .pipe(sanitize_tree);
            let actual: Vec<_> = tree
                .iter_depth_index_node()
                .map(|item| {
                    (
                        item.depth,
                        item.index,
                        item.name.to_str().expect("convert name to utf8"),
                        item.data.inner(),
                    )
                })
                .collect();
            assert_eq!(actual, $expected(&prefix));
        }
    )* };
}

fn get_size(prefix: &Path, suffix: &'static str) -> u64 {
    prefix.join(suffix).pipe(metadata).unwrap().len()
}

macro_rules! total_size {
    ($prefix:ident ++ $($suffix:literal)+) => {
        0 $( + get_size($prefix, $suffix) )+
    };
}

fn get_name(path: &Path) -> &'_ str {
    if let Some(name) = path.file_name() {
        name.to_str().expect("get name of a path")
    } else {
        "."
    }
}

test_case! {
    flat in "flat" => |prefix| vec![
        (0, 0, "flat", total_size! { prefix ++ "" "0" "1" "2" "3" }),
        (1, 0, "0", total_size! { prefix ++ "0" }),
        (1, 1, "1", total_size! { prefix ++ "1" }),
        (1, 2, "2", total_size! { prefix ++ "2" }),
        (1, 3, "3", total_size! { prefix ++ "3" }),
    ];

    nested in "nested" => |prefix| vec![
        (0, 0, "nested", total_size! { prefix ++ "" "0" "0/1" }),
        (1, 0, "0", total_size! { prefix ++ "0" "0/1" }),
        (2, 0, "1", total_size! { prefix ++ "0/1" }),
    ];

    empty_dir in "empty-dir" => |prefix| vec![
        (0, 0, "empty-dir", total_size! { prefix ++ "" }),
    ];

    all in "." => |prefix| vec![
        (0, 0, get_name(prefix), total_size! {
            prefix ++
            ""
            "flat"
            "flat/0"
            "flat/1"
            "flat/2"
            "flat/3"
            "nested"
            "nested/0"
            "nested/0/1"
            "empty-dir"
        }),
        (1, 0, "empty-dir", total_size! { prefix ++ "empty-dir" }),
        (1, 1, "flat", total_size! {
            prefix ++
            "flat"
            "flat/0"
            "flat/1"
            "flat/2"
            "flat/3"
        }),
        (2, 0, "0", total_size! { prefix ++ "flat/0" }),
        (2, 1, "1", total_size! { prefix ++ "flat/1" }),
        (2, 2, "2", total_size! { prefix ++ "flat/2" }),
        (2, 3, "3", total_size! { prefix ++ "flat/3" }),
        (1, 2, "nested", total_size! {
            prefix ++
            "nested"
            "nested/0"
            "nested/0/1"
        }),
        (2, 0, "0", total_size! { prefix ++ "nested/0" "nested/0/1" }),
        (3, 0, "1", total_size! { prefix ++ "nested/0/1" }),
    ];
}
