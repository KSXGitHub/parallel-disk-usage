pub mod _utils;
pub use _utils::*;

use parallel_disk_usage::size::Bytes;

#[cfg(unix)]
use parallel_disk_usage::size::Blocks;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

#[test]
fn len_as_bytes() {
    let workspace = SampleWorkspace::default();
    test_sample_tree::<Bytes, _>(&workspace, |metadata| metadata.len());
}

#[cfg(unix)]
#[test]
fn blocks_as_bytes() {
    let workspace = SampleWorkspace::default();
    test_sample_tree::<Bytes, _>(&workspace, |metadata| metadata.blocks() * 512);
}

#[cfg(unix)]
#[test]
fn blocks_as_blocks() {
    let workspace = SampleWorkspace::default();
    test_sample_tree::<Blocks, _>(&workspace, |metadata| metadata.blocks());
}
