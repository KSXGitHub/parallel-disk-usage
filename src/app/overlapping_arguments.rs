use pipe_trait::Pipe;
use std::{
    collections::HashSet,
    fs::{canonicalize, symlink_metadata},
    io,
    mem::take,
    path::PathBuf,
};

/// Mockable APIs to interact with the system.
pub trait Api {
    type Argument;
    type RealPath: Eq;
    type RealPathError;
    fn canonicalize(path: &Self::Argument) -> Result<Self::RealPath, Self::RealPathError>;
    fn is_real_dir(path: &Self::Argument) -> bool;
    fn starts_with(a: &Self::RealPath, b: &Self::RealPath) -> bool;
}

/// Implementation of [`Api`] that interacts with the real system.
pub struct RealApi;
impl Api for RealApi {
    type Argument = PathBuf;
    type RealPath = PathBuf;
    type RealPathError = io::Error;

    #[inline]
    fn canonicalize(path: &Self::Argument) -> Result<Self::RealPath, Self::RealPathError> {
        canonicalize(path)
    }

    #[inline]
    fn is_real_dir(path: &Self::Argument) -> bool {
        path.pipe(symlink_metadata)
            .is_ok_and(|metadata| !metadata.is_symlink() && metadata.is_dir())
    }

    #[inline]
    fn starts_with(a: &Self::RealPath, b: &Self::RealPath) -> bool {
        a.starts_with(b)
    }
}

/// Hardlinks deduplication doesn't work properly if there are more than 1 paths pointing to
/// the same tree or if a path points to a subtree of another path. Therefore, we must find
/// and remove such overlapping paths before they cause problems.
pub fn remove_overlapping_paths<Api: self::Api>(arguments: &mut Vec<Api::Argument>) {
    let to_remove = find_overlapping_paths_to_remove::<Api>(arguments);
    remove_items_from_vec_by_indices(arguments, &to_remove);
}

/// Find overlapping paths in a list of arguments to remove and return their indices.
///
/// Prefer keeping the containing tree over the subtree (returning the index of the subtree).
///
/// Prefer keeping the first instance of the path over the later instances (returning the indices of
/// the later instances).
pub fn find_overlapping_paths_to_remove<Api: self::Api>(
    arguments: &[Api::Argument],
) -> HashSet<usize> {
    let real_paths: Vec<_> = arguments
        .iter()
        .map(|path| {
            Api::is_real_dir(path)
                .then(|| Api::canonicalize(path))
                .and_then(Result::ok)
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
                if Api::starts_with(left, right) {
                    to_remove.insert(left_index);
                    continue;
                }

                // `right` starts with `left` means `right` is subtree of `left`, remove `right`
                if Api::starts_with(right, left) {
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
#[cfg(unix)]
#[cfg(test)]
mod test_remove_overlapping_paths;
