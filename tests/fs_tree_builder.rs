#![cfg(test)]
pub mod _utils;
pub use _utils::*;

use dirt::{
    fs_tree_builder::{FsTreeBuilder, Progress},
    size::Bytes,
    tree::Tree,
};
use maplit::btreeset;
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::{collections::BTreeSet, fs::metadata, path::PathBuf, sync::Mutex};

#[cfg(unix)]
use dirt::size::Blocks;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

#[test]
fn len_as_bytes() {
    let workspace = SampleWorkspace::default();
    test_sample_tree::<Bytes, _>(&workspace, |metadata| metadata.len());
}

#[cfg(unix)]
#[test]
fn blksize_as_bytes() {
    let workspace = SampleWorkspace::default();
    test_sample_tree::<Bytes, _>(&workspace, |metadata| metadata.blksize());
}

#[cfg(unix)]
#[test]
fn blocks_as_blocks() {
    let workspace = SampleWorkspace::default();
    test_sample_tree::<Blocks, _>(&workspace, |metadata| metadata.blocks());
}

#[test]
fn progress_reports() {
    let workspace = SampleWorkspace::default();
    let reports = Mutex::new(BTreeSet::new());
    Tree::<PathBuf, Bytes>::from(FsTreeBuilder {
        get_data: |metadata| metadata.len().into(),
        report_error: |error| panic!("Unexpected call to report_error: {:?}", error),
        report_progress: |progress| {
            reports.lock().unwrap().insert(*progress);
        },
        root: workspace.join("nested"),
    });
    macro_rules! scanned_total {
        ($(,)?) => {
            Bytes::from(0)
        };
        ($suffix:expr $(,)?) => {
            workspace
                .join("nested")
                .join($suffix)
                .pipe(metadata)
                .expect("get metadata")
                .len()
                .pipe(Bytes::from)
        };
        ($head:expr, $($tail:expr),* $(,)?) => {
            scanned_total!($head) + scanned_total!($($tail),+)
        };
    }
    let actual = reports.lock().unwrap().clone();
    dbg!(&actual);
    let expected = btreeset! {
        // begin scanning /
        Progress {
            known_items: 1,
            scanned_items: 0,
            scanned_total: scanned_total!(),
            errors: 0,
        },
        // finish scanning /
        Progress {
            known_items: 1,
            scanned_items: 1,
            scanned_total: scanned_total!(),
            errors: 0,
        },
        // update scanned_total
        Progress {
            known_items: 1,
            scanned_items: 1,
            scanned_total: scanned_total!(""),
            errors: 0,
        },
        // begin scanning /0
        Progress {
            known_items: 2,
            scanned_items: 1,
            scanned_total: scanned_total!(""),
            errors: 0,
        },
        // finish scanning /0
        Progress {
            known_items: 2,
            scanned_items: 2,
            scanned_total: scanned_total!(""),
            errors: 0,
        },
        // update scanned_total
        Progress {
            known_items: 2,
            scanned_items: 2,
            scanned_total: scanned_total!("", "0"),
            errors: 0,
        },
        // begin scanning /0/1
        Progress {
            known_items: 3,
            scanned_items: 2,
            scanned_total: scanned_total!("", "0"),
            errors: 0,
        },
        // finish scanning /0/1
        Progress {
            known_items: 3,
            scanned_items: 3,
            scanned_total: scanned_total!("", "0"),
            errors: 0,
        },
        // update scanned_total
        Progress {
            known_items: 3,
            scanned_items: 3,
            scanned_total: scanned_total!("", "0", "0/1"),
            errors: 0,
        },
    };
    assert_eq!(actual, expected);
}
