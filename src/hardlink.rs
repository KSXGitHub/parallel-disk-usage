// `RecordHardlink` is POSIX-exclusive, because whilst Windows does have `MetadataExt::number_of_links`, it requires Nightly.
#[cfg(unix)]
pub mod aware;
#[cfg(unix)]
pub use aware::HardlinkAware;

pub mod deduplicate;
pub mod hardlink_list;
pub mod ignorant;
pub mod link_path_list;
pub mod record;

pub use deduplicate::DeduplicateSharedSize;
pub use hardlink_list::{HardlinkList, HardlinkListReflection};
pub use ignorant::HardlinkIgnorant;
pub use link_path_list::{LinkPathList, LinkPathListReflection};
pub use record::{RecordHardlinks, RecordHardlinksArgument};
