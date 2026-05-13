use super::DataTree;
use crate::size;
use assert_cmp::debug_assert_op;
use rayon::prelude::*;
use std::{ffi::OsStr, path::Path};

impl<Name, Size> DataTree<Name, Size>
where
    Self: Send,
    Name: AsRef<OsStr>,
    Size: size::Size + Sync,
{
    /// Reduce the size of the directories that have hardlinks.
    #[cfg_attr(not(unix), expect(unused))]
    pub(crate) fn par_deduplicate_hardlinks(&mut self, hardlink_info: &[(Size, Vec<&Path>)]) {
        if hardlink_info.is_empty() {
            return;
        }

        let prefix = self.name().as_ref();
        let sub_hardlink_info: Vec<(Size, Vec<&Path>)> = hardlink_info
            .iter()
            .filter(|(_, link_paths)| link_paths.len() > 1)
            .map(|(size, link_paths)| {
                let link_suffices: Vec<&Path> = link_paths
                    .iter()
                    .map(|link_path| link_path.strip_prefix(prefix))
                    .filter_map(Result::ok)
                    .collect();
                (*size, link_suffices)
            })
            .filter(|(_, link_paths)| link_paths.len() > 1)
            .collect();

        for (size, link_suffices) in &sub_hardlink_info {
            let number_of_links = link_suffices.len();
            debug_assert_op!(number_of_links > 1);
            self.size -= *size * (number_of_links - 1);
        }

        self.children
            .par_iter_mut()
            .for_each(|child| child.par_deduplicate_hardlinks(&sub_hardlink_info))
    }
}
