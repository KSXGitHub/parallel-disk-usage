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

    fn canonicalize(path: &Self::Argument) -> Result<Self::RealPath, Self::RealPathError> {
        canonicalize(path)
    }

    fn is_real_dir(path: &Self::Argument) -> bool {
        path.pipe(symlink_metadata)
            .is_ok_and(|metadata| !metadata.is_symlink() && metadata.is_dir())
    }

    fn starts_with(a: &Self::RealPath, b: &Self::RealPath) -> bool {
        a.starts_with(b)
    }
}

/// Hardlinks deduplication doesn't work properly if there are more than 1 paths pointing to
/// the same tree or if a path points to a subtree of another path. Therefore, we must find
/// and remove such duplications before they cause problem.
pub fn deduplicate_arguments<Api: self::Api>(arguments: &mut Vec<Api::Argument>) {
    let to_remove = find_argument_duplications_to_remove::<Api>(arguments);
    remove_items_from_vec_by_indices(arguments, &to_remove);
}

/// Find duplication in a list of arguments to remove and return their indices.
///
/// Prefer keeping the containing tree over the subtree (returning the index of the subtree).
///
/// Prefer keeping the first instance of the path over the later instances (returning the indices of
/// the later instances).
pub fn find_argument_duplications_to_remove<Api: self::Api>(
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

#[cfg(unix)]
#[cfg(test)]
mod tests {
    use super::{deduplicate_arguments, remove_items_from_vec_by_indices, Api};
    use maplit::hashset;
    use normalize_path::NormalizePath;
    use pipe_trait::Pipe;
    use pretty_assertions::assert_eq;
    use std::{collections::HashSet, convert::Infallible, path::PathBuf};

    const MOCKED_CURRENT_DIR: &str = "/home/user/current-dir";

    const MOCKED_SYMLINKS: &[(&str, &str)] = &[
        ("/home/user/current-dir/link-to-current-dir", "."),
        ("/home/user/current-dir/link-to-parent-dir", ".."),
        ("/home/user/current-dir/link-to-root", "/"),
        ("/home/user/current-dir/link-to-bin", "/usr/bin"),
        ("/home/user/current-dir/link-to-foo", "foo"),
        ("/home/user/current-dir/link-to-bar", "bar"),
        ("/home/user/current-dir/link-to-012", "0/1/2"),
    ];

    fn resolve_symlink(absolute_path: PathBuf) -> PathBuf {
        assert!(
            absolute_path.is_absolute(),
            "absolute_path should be absolute: {absolute_path:?}",
        );
        for &(link_path, link_target) in MOCKED_SYMLINKS {
            let link_path = PathBuf::from(link_path);
            assert!(
                link_path.is_absolute(),
                "link_path should be absolute: {link_path:?}",
            );
            let Some(parent) = link_path.parent() else {
                panic!("Cannot get parent of {link_path:?}");
            };
            if let Ok(suffix) = absolute_path.strip_prefix(&link_path) {
                return parent
                    .join(link_target)
                    .join(suffix)
                    .normalize()
                    .pipe(resolve_symlink);
            }
        }
        absolute_path
    }

    /// Mocked implementation of [`Api`] for testing purposes.
    struct MockedApi;
    impl Api for MockedApi {
        type Argument = &'static str;
        type RealPath = PathBuf;
        type RealPathError = Infallible;

        fn canonicalize(path: &Self::Argument) -> Result<Self::RealPath, Self::RealPathError> {
            MOCKED_CURRENT_DIR
                .pipe(PathBuf::from)
                .join(path)
                .normalize()
                .pipe(resolve_symlink)
                .pipe(Ok)
        }

        fn is_real_dir(path: &Self::Argument) -> bool {
            let path = MOCKED_CURRENT_DIR.pipe(PathBuf::from).join(path);
            MOCKED_SYMLINKS
                .iter()
                .all(|(link, _)| PathBuf::from(link).normalize() != path)
        }

        fn starts_with(a: &Self::RealPath, b: &Self::RealPath) -> bool {
            a.starts_with(b)
        }
    }

    #[test]
    fn find_nothing_to_remove() {
        let original = vec!["foo", "bar", "abc/def", "0/1/2"];
        let mut actual = original.clone();
        deduplicate_arguments::<MockedApi>(&mut actual);
        let expected = original;
        assert_eq!(actual, expected);
    }

    #[test]
    fn remove_duplicated_arguments() {
        let original = dbg!(vec![
            "foo",
            "bar",
            "abc/def",
            "foo",
            "0/1/2",
            "./bar",
            "./abc/./def",
        ]);
        let mut actual = original.clone();
        deduplicate_arguments::<MockedApi>(&mut actual);
        let expected = vec!["foo", "bar", "abc/def", "0/1/2"];
        assert_eq!(actual, expected);

        let original = dbg!(vec![
            "foo",
            "./bar",
            "bar",
            "./abc/./def",
            "abc/def",
            "foo",
            "0/1/2",
        ]);
        let mut actual = original.clone();
        deduplicate_arguments::<MockedApi>(&mut actual);
        let expected = vec!["foo", "./bar", "./abc/./def", "0/1/2"];
        assert_eq!(actual, expected);
    }

    #[test]
    fn remove_duplicated_sub_paths() {
        let original = vec![
            "foo/child",
            "foo",
            "bar",
            "abc/def",
            "0/1/2",
            "bar/child",
            "0/1/2/3",
        ];
        let mut actual = original.clone();
        deduplicate_arguments::<MockedApi>(&mut actual);
        let expected = vec!["foo", "bar", "abc/def", "0/1/2"];
        assert_eq!(actual, expected);
    }

    #[test]
    fn remove_all_except_current_dir() {
        let original = dbg!(vec!["foo", "bar", ".", "abc/def", "0/1/2"]);
        let mut actual = original.clone();
        deduplicate_arguments::<MockedApi>(&mut actual);
        let expected = vec!["."];
        assert_eq!(actual, expected);

        let original = dbg!(vec![
            "foo",
            "bar",
            ".",
            "abc/def",
            "0/1/2",
            MOCKED_CURRENT_DIR,
        ]);
        let mut actual = original.clone();
        deduplicate_arguments::<MockedApi>(&mut actual);
        let expected = vec!["."];
        assert_eq!(actual, expected);

        let original = dbg!(vec![
            "foo",
            "bar",
            MOCKED_CURRENT_DIR,
            ".",
            "abc/def",
            "0/1/2",
        ]);
        let mut actual = original.clone();
        deduplicate_arguments::<MockedApi>(&mut actual);
        let expected = vec![MOCKED_CURRENT_DIR];
        assert_eq!(actual, expected);
    }

    #[test]
    fn remove_all_except_parent_dir() {
        let original = dbg!(vec!["foo", "bar", "..", "abc/def", ".", "0/1/2"]);
        let mut actual = original.clone();
        deduplicate_arguments::<MockedApi>(&mut actual);
        let expected = vec![".."];
        assert_eq!(actual, expected);

        let original = dbg!(vec![
            "foo",
            "/home/user",
            "bar",
            "..",
            "abc/def",
            ".",
            "0/1/2"
        ]);
        let mut actual = original.clone();
        deduplicate_arguments::<MockedApi>(&mut actual);
        let expected = vec!["/home/user"];
        assert_eq!(actual, expected);
    }

    #[test]
    fn remove_duplicated_real_paths() {
        let original = dbg!(vec![
            "foo",
            "bar",
            "abc/def",
            "link-to-foo/child",
            "link-to-bar/a/b/c",
            "0/1/2",
        ]);
        let mut actual = original.clone();
        deduplicate_arguments::<MockedApi>(&mut actual);
        let expected = vec!["foo", "bar", "abc/def", "0/1/2"];
        assert_eq!(actual, expected);

        let original = dbg!(vec![
            "link-to-foo/child",
            "link-to-bar/a/b/c",
            "foo",
            "bar",
            "abc/def",
            "0/1/2",
        ]);
        let mut actual = original.clone();
        deduplicate_arguments::<MockedApi>(&mut actual);
        let expected = vec!["foo", "bar", "abc/def", "0/1/2"];
        assert_eq!(actual, expected);

        let original = dbg!(vec![
            "link-to-current-dir/foo",
            "foo",
            "bar",
            "abc/def",
            "link-to-current-dir/bar",
            "0/1/2",
        ]);
        let mut actual = original.clone();
        deduplicate_arguments::<MockedApi>(&mut actual);
        let expected = vec!["link-to-current-dir/foo", "bar", "abc/def", "0/1/2"];
        assert_eq!(actual, expected);
    }

    #[test]
    fn do_not_remove_symlinks() {
        let original = dbg!(vec![
            "foo",
            "bar",
            "abc/def",
            "link-to-foo",
            "link-to-bar",
            "0/1/2",
        ]);
        let mut actual = original.clone();
        deduplicate_arguments::<MockedApi>(&mut actual);
        let expected = original;
        assert_eq!(actual, expected);

        let original = dbg!(vec![
            "foo/child",
            "bar",
            "abc/def",
            "link-to-foo",
            "link-to-bar",
            "0/1/2",
        ]);
        let mut actual = original.clone();
        deduplicate_arguments::<MockedApi>(&mut actual);
        let expected = original;
        assert_eq!(actual, expected);
    }

    #[test]
    fn remove_nothing() {
        let original = vec![31, 54, 22, 81, 67, 45, 52, 20, 85, 66, 27, 84];
        let mut modified = original.clone();
        remove_items_from_vec_by_indices(&mut modified, &HashSet::new());
        assert_eq!(modified, original);
    }

    #[test]
    fn remove_single() {
        let original = vec![31, 54, 22, 81, 67, 45, 52, 20, 85, 66, 27, 84];
        let mut modified = original.clone();
        remove_items_from_vec_by_indices(&mut modified, &hashset! { 3 });
        assert_eq!(&modified[..3], &original[..3]);
        assert_eq!(&modified[3..], &original[4..]);
    }

    #[test]
    fn remove_multiple() {
        let original = vec![31, 54, 22, 81, 67, 45, 52, 20, 85, 66, 27, 84];
        let mut modified = original.clone();
        remove_items_from_vec_by_indices(&mut modified, &hashset! { 3, 4, 5, 7 });
        assert_eq!(&modified[..3], &original[..3]);
        assert_eq!(&modified[3..4], &original[6..7]);
        assert_eq!(&modified[4..], &original[8..]);
    }
}
