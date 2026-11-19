pub use _utils::*;

use into_sorted::IntoSorted;
use parallel_disk_usage::data_tree::{DataTree, DataTreeReflection};
use parallel_disk_usage::device::DeviceBoundary;
use parallel_disk_usage::fs_tree_builder::FsTreeBuilder;
use parallel_disk_usage::get_size::{self, GetSize};
use parallel_disk_usage::hardlink::HardlinkIgnorant;
use parallel_disk_usage::os_string_display::OsStringDisplay;
use parallel_disk_usage::reporter::ErrorOnlyReporter;
use parallel_disk_usage::size;
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::fs::metadata;
use std::path::Path;

#[cfg(unix)]
use std::fs::symlink_metadata;

/// Default size getter method.
#[cfg(unix)]
pub const DEFAULT_GET_SIZE: get_size::GetBlockSize = get_size::GetBlockSize;
/// Default size getter method.
#[cfg(not(unix))]
pub const DEFAULT_GET_SIZE: get_size::GetApparentSize = get_size::GetApparentSize;

/// Path to the `pdu` executable
pub const PDU: &str = env!("CARGO_BIN_EXE_pdu");

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
        children,
    } = tree_reflection;
    let children = children
        .into_sorted_by(|left, right| left.name.cmp(&right.name))
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
            device_boundary: DeviceBoundary::Cross,
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

/// Recursively sort a [`DataTreeReflection`].
pub fn sort_reflection_by<Name, Size, Order>(
    reflection: &mut DataTreeReflection<Name, Size>,
    order: Order,
) where
    Size: size::Size,
    Order:
        FnMut(&DataTreeReflection<Name, Size>, &DataTreeReflection<Name, Size>) -> Ordering + Copy,
{
    reflection.children.sort_by(order);
    for child in &mut reflection.children {
        sort_reflection_by(child, order);
    }
}

/// Read [device number](std::os::unix::fs::MetadataExt::dev) of a path.
#[cfg(unix)]
pub fn read_device_number(path: &Path) -> parallel_disk_usage::device::DeviceNumber {
    use std::os::unix::fs::MetadataExt;
    path.pipe(symlink_metadata)
        .unwrap_or_else(|error| panic!("Can't read metadata at {path:?}: {error}"))
        .dev()
        .into()
}
