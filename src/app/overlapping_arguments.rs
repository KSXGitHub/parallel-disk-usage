use super::host::Host;
use pipe_trait::Pipe;
use std::collections::HashSet;
use std::fs::{canonicalize, symlink_metadata};
use std::mem::take;
use std::path::PathBuf;

/// The command-line argument that the argument-resolution capabilities operate on.
pub trait ArgumentSource {
    /// The argument value that the capabilities below resolve.
    type Argument;
}

/// Capability: resolve an argument to its canonical real path.
///
/// Returns `None` when the argument cannot be resolved, for example because it
/// does not exist. This mirrors [`std::fs::canonicalize`] followed by
/// discarding the error, which is all the caller needs.
pub trait CanonicalizeArgument: ArgumentSource {
    fn canonicalize(argument: &Self::Argument) -> Option<PathBuf>;
}

/// Capability: check whether an argument refers to a real directory.
///
/// A symbolic link, even one pointing at a directory, is not a real directory
/// for this purpose, mirroring the check on [`std::fs::symlink_metadata`].
pub trait IsRealDir: ArgumentSource {
    fn is_real_dir(argument: &Self::Argument) -> bool;
}

impl ArgumentSource for Host {
    type Argument = PathBuf;
}

impl CanonicalizeArgument for Host {
    #[inline]
    fn canonicalize(argument: &Self::Argument) -> Option<PathBuf> {
        canonicalize(argument).ok()
    }
}

impl IsRealDir for Host {
    #[inline]
    fn is_real_dir(argument: &Self::Argument) -> bool {
        argument
            .pipe(symlink_metadata)
            .is_ok_and(|metadata| !metadata.is_symlink() && metadata.is_dir())
    }
}

/// Hardlinks deduplication doesn't work properly if there are more than 1 paths pointing to
/// the same tree or if a path points to a subtree of another path. Therefore, we must find
/// and remove such overlapping paths before they cause problems.
pub fn remove_overlapping_paths<Sys>(arguments: &mut Vec<Sys::Argument>)
where
    Sys: CanonicalizeArgument + IsRealDir,
{
    let to_remove = find_overlapping_paths_to_remove::<Sys>(arguments);
    remove_items_from_vec_by_indices(arguments, &to_remove);
}

/// Find overlapping paths in a list of arguments to remove and return their indices.
///
/// Prefer keeping the containing tree over the subtree (returning the index of the subtree).
///
/// Prefer keeping the first instance of the path over the later instances (returning the indices of
/// the later instances).
pub fn find_overlapping_paths_to_remove<Sys>(arguments: &[Sys::Argument]) -> HashSet<usize>
where
    Sys: CanonicalizeArgument + IsRealDir,
{
    let real_paths: Vec<_> = arguments
        .iter()
        .map(|path| {
            Sys::is_real_dir(path)
                .then(|| Sys::canonicalize(path))
                .flatten()
        })
        .collect();
    assert_eq!(arguments.len(), real_paths.len());

    let mut to_remove = HashSet::new();
    for left_index in 0..arguments.len() {
        for right_index in (left_index + 1)..arguments.len() {
            if let (Some(left), Some(right)) = (&real_paths[left_index], &real_paths[right_index]) {
                // both paths are the same, remove the second one
                if left == right {
                    to_remove.insert(right_index);
                    continue;
                }

                // `left` starts with `right` means `left` is subtree of `right`, remove `left`
                if left.starts_with(right) {
                    to_remove.insert(left_index);
                    continue;
                }

                // `right` starts with `left` means `right` is subtree of `left`, remove `right`
                if right.starts_with(left) {
                    to_remove.insert(right_index);
                    continue;
                }
            }
        }
    }
    to_remove
}

/// Remove elements from a vector by indices.
pub fn remove_items_from_vec_by_indices<Item>(vec: &mut Vec<Item>, indices: &HashSet<usize>) {
    // Optimization: If there is no element to remove then there is nothing to do.
    if indices.is_empty() {
        return;
    }

    // Optimization: If there is only 1 element to remove, shifting elements would be cheaper than reallocating a whole array.
    if indices.len() == 1 {
        let index = *indices.iter().next().unwrap();
        vec.remove(index);
        return;
    }

    // Default: If there are more than 1 element to remove, just copy the whole array without them.
    *vec = vec
        .pipe(take)
        .into_iter()
        .enumerate()
        .filter(|(index, _)| !indices.contains(index))
        .map(|(_, item)| item)
        .collect();
}

#[cfg(test)]
mod test_remove_items_from_vec_by_indices;
#[cfg(test)]
mod test_remove_overlapping_paths;
