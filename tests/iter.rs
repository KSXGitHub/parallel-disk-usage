#![cfg(test)]
pub mod _utils;
pub use _utils::*;

use dirt::{fs_tree_builder::FsTreeBuilder, reporter::ErrorOnlyReporter, size::Bytes, tree::Tree};
use maplit::btreeset;
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::{collections::BTreeSet, ffi::OsString, fs::metadata, path::Path};

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
            let tree = Tree::<OsString, Bytes>::from(FsTreeBuilder {
                get_data: |metadata| metadata.len().into(),
                reporter: ErrorOnlyReporter::new(|error| {
                    panic!("Unexpected call to report_error: {:?}", error)
                }),
                root: prefix.clone(),
            });
            let actual: BTreeSet<_> = tree
                .iter_path()
                .map(|item| {
                    (
                        item.path
                            .iter()
                            .map(|x| x.to_str().expect("convert path item to utf8"))
                            .collect::<Vec<_>>(),
                        item.name
                            .to_str()
                            .expect("convert name to utf8"),
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
    flat in "flat" => |prefix| btreeset![
        (vec![], "flat", total_size! { prefix ++ "" "0" "1" "2" "3" }),
        (vec!["flat"], "0", total_size! { prefix ++ "0" }),
        (vec!["flat"], "1", total_size! { prefix ++ "1" }),
        (vec!["flat"], "2", total_size! { prefix ++ "2" }),
        (vec!["flat"], "3", total_size! { prefix ++ "3" }),
    ];

    nested in "nested" => |prefix| btreeset![
        (vec![], "nested", total_size! { prefix ++ "" "0" "0/1" }),
        (vec!["nested"], "0", total_size! { prefix ++ "0" "0/1" }),
        (vec!["nested", "0"], "1", total_size! { prefix ++ "0/1" }),
    ];

    all in "." => |prefix| btreeset![
        (vec![], get_name(prefix), total_size! {
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
        (vec![get_name(prefix)], "flat", total_size! {
            prefix ++
            "flat"
            "flat/0"
            "flat/1"
            "flat/2"
            "flat/3"
        }),
        (vec![get_name(prefix), "flat"], "0", total_size! { prefix ++ "flat/0" }),
        (vec![get_name(prefix), "flat"], "1", total_size! { prefix ++ "flat/1" }),
        (vec![get_name(prefix), "flat"], "2", total_size! { prefix ++ "flat/2" }),
        (vec![get_name(prefix), "flat"], "3", total_size! { prefix ++ "flat/3" }),
        (vec![get_name(prefix)], "nested", total_size! {
            prefix ++
            "nested"
            "nested/0"
            "nested/0/1"
        }),
        (vec![get_name(prefix), "nested"], "0", total_size! { prefix ++ "nested/0" "nested/0/1" }),
        (vec![get_name(prefix), "nested", "0"], "1", total_size! { prefix ++ "nested/0/1" }),
        (vec![get_name(prefix)], "empty-dir", total_size! { prefix ++ "empty-dir" }),
    ];
}
