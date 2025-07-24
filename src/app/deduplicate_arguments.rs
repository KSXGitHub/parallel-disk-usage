use pipe_trait::Pipe;
use std::{collections::HashSet, mem::take};

/// Hardlinks deduplication doesn't work properly if there are more than 1 paths pointing to
/// the same tree or if a path points to a subtree of another path. Therefore, we must find
/// and remove such duplications before they cause problem.
pub fn deduplicate_arguments<'a, Argument, Canonicalize, StartsWith, RealPath, CanonicalizeError>(
    arguments: &'a mut Vec<Argument>,
    canonicalize: Canonicalize,
    starts_with: StartsWith,
) where
    Canonicalize: for<'r> FnMut(&Argument) -> Result<RealPath, CanonicalizeError>,
    StartsWith: for<'r> FnMut(&'r RealPath, &'r RealPath) -> bool,
    RealPath: Eq,
{
    let to_remove = find_argument_duplications_to_remove(arguments, canonicalize, starts_with);
    remove_items_from_vec_by_indices(arguments, &to_remove);
}

/// Find duplication in a list of arguments to remove and return their indices.
///
/// Prefer keeping the containing tree over the subtree (returning the index of the subtree).
///
/// Prefer keeping the first instance of the path over the later instances (returning the indices of
/// the later instances).
pub fn find_argument_duplications_to_remove<
    Argument,
    Canonicalize,
    StartsWith,
    RealPath,
    CanonicalizeError,
>(
    arguments: &[Argument],
    canonicalize: Canonicalize,
    mut starts_with: StartsWith,
) -> HashSet<usize>
where
    Canonicalize: for<'r> FnMut(&Argument) -> Result<RealPath, CanonicalizeError>,
    StartsWith: for<'r> FnMut(&'r RealPath, &'r RealPath) -> bool,
    RealPath: Eq,
{
    let real_paths: Vec<_> = arguments.iter().map(canonicalize).collect();
    assert_eq!(arguments.len(), real_paths.len());

    let mut to_remove = HashSet::new();
    for left_index in 0..arguments.len() {
        for right_index in (left_index + 1)..arguments.len() {
            if let (Ok(left), Ok(right)) = (&real_paths[left_index], &real_paths[right_index]) {
                // both paths are the same, remove the second one
                if left == right {
                    to_remove.insert(right_index);
                    continue;
                }

                // `left` starts with `right` means `left` is subtree of `right`, remove `left`
                if starts_with(left, right) {
                    to_remove.insert(left_index);
                    continue;
                }

                // `right` starts with `left` means `right` is subtree of `left`, remove `right`
                if starts_with(right, left) {
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

    // Optimization: If there is only 1 element to remove, shifting elements would be cheaper than reallocate a whole array.
    if indices.len() == 1 {
        let index = *indices.iter().next().unwrap();
        vec.remove(index);
        return;
    }

    // Default: If there are more than 1 elements to remove, just copy the whole array without them.
    *vec = vec
        .pipe(take)
        .into_iter()
        .enumerate()
        .filter(|(index, _)| !indices.contains(index))
        .map(|(_, item)| item)
        .collect();
}
