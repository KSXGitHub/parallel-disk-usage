#![cfg(test)]
pub mod _utils;
pub use _utils::*;

use dirt::{
    fs_tree_builder::FsTreeBuilder,
    reporter::{ProgressAndErrorReporter, ProgressReport},
    size::MetricBytes,
    tree::Tree,
};
use maplit::btreeset;
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::{collections::BTreeSet, ffi::OsString, fs::metadata, sync::Mutex};

#[cfg(unix)]
use dirt::size::Blocks;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

#[test]
fn len_as_bytes() {
    let workspace = SampleWorkspace::default();
    test_sample_tree::<MetricBytes, _>(&workspace, |metadata| metadata.len());
}

#[cfg(unix)]
#[test]
fn blksize_as_bytes() {
    let workspace = SampleWorkspace::default();
    test_sample_tree::<MetricBytes, _>(&workspace, |metadata| metadata.blksize());
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
    Tree::<OsString, MetricBytes>::from(FsTreeBuilder {
        get_data: |metadata| metadata.len().into(),
        reporter: ProgressAndErrorReporter::new(
            |progress| {
                reports.lock().unwrap().insert(*progress);
            },
            |error| panic!("Unexpected call to report_error: {:?}", error),
        ),
        root: workspace.join("nested"),
        post_process_children,
    });
    macro_rules! scanned_total {
        ($(,)?) => {
            MetricBytes::from(0)
        };
        ($suffix:expr $(,)?) => {
            workspace
                .join("nested")
                .join($suffix)
                .pipe(metadata)
                .expect("get metadata")
                .len()
                .pipe(MetricBytes::from)
        };
        ($head:expr, $($tail:expr),* $(,)?) => {
            scanned_total!($head) + scanned_total!($($tail),+)
        };
    }
    let actual = reports.lock().unwrap().clone();
    dbg!(&actual);
    let expected = btreeset! {
        // begin scanning /
        ProgressReport {
            known_items: 1,
            scanned_items: 0,
            scanned_total: scanned_total!(),
            errors: 0,
        },
        // finish scanning /
        ProgressReport {
            known_items: 1,
            scanned_items: 1,
            scanned_total: scanned_total!(),
            errors: 0,
        },
        // update scanned_total
        ProgressReport {
            known_items: 1,
            scanned_items: 1,
            scanned_total: scanned_total!(""),
            errors: 0,
        },
        // begin scanning /0
        ProgressReport {
            known_items: 2,
            scanned_items: 1,
            scanned_total: scanned_total!(""),
            errors: 0,
        },
        // finish scanning /0
        ProgressReport {
            known_items: 2,
            scanned_items: 2,
            scanned_total: scanned_total!(""),
            errors: 0,
        },
        // update scanned_total
        ProgressReport {
            known_items: 2,
            scanned_items: 2,
            scanned_total: scanned_total!("", "0"),
            errors: 0,
        },
        // begin scanning /0/1
        ProgressReport {
            known_items: 3,
            scanned_items: 2,
            scanned_total: scanned_total!("", "0"),
            errors: 0,
        },
        // finish scanning /0/1
        ProgressReport {
            known_items: 3,
            scanned_items: 3,
            scanned_total: scanned_total!("", "0"),
            errors: 0,
        },
        // update scanned_total
        ProgressReport {
            known_items: 3,
            scanned_items: 3,
            scanned_total: scanned_total!("", "0", "0/1"),
            errors: 0,
        },
    };
    assert_eq!(actual, expected);
}
