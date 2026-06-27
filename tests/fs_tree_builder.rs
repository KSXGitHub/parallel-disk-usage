pub mod _utils;
pub use _utils::*;
use parallel_disk_usage::get_size::GetApparentSize;
use parallel_disk_usage::size::Bytes;

#[cfg(unix)]
use parallel_disk_usage::get_size::{GetBlockCount, GetBlockSize};
#[cfg(unix)]
use parallel_disk_usage::size::Blocks;

#[test]
fn len_as_bytes() {
    let workspace = SampleWorkspace::default();
    test_sample_tree::<Bytes, _>(&workspace, GetApparentSize);
}

#[cfg(unix)]
#[test]
fn blocks_as_bytes() {
    let workspace = SampleWorkspace::default();
    test_sample_tree::<Bytes, _>(&workspace, GetBlockSize);
}

#[cfg(unix)]
#[test]
fn blocks_as_blocks() {
    let workspace = SampleWorkspace::default();
    test_sample_tree::<Blocks, _>(&workspace, GetBlockCount);
}
