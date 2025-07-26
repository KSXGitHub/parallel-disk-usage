use super::{remove_overlapping_paths, Api};
use normalize_path::NormalizePath;
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::{convert::Infallible, path::PathBuf};

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
fn remove_nothing() {
    let original = vec!["foo", "bar", "abc/def", "0/1/2"];
    let mut actual = original.clone();
    remove_overlapping_paths::<MockedApi>(&mut actual);
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
    remove_overlapping_paths::<MockedApi>(&mut actual);
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
    remove_overlapping_paths::<MockedApi>(&mut actual);
    let expected = vec!["foo", "./bar", "./abc/./def", "0/1/2"];
    assert_eq!(actual, expected);
}

#[test]
fn remove_overlapping_sub_paths() {
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
    remove_overlapping_paths::<MockedApi>(&mut actual);
    let expected = vec!["foo", "bar", "abc/def", "0/1/2"];
    assert_eq!(actual, expected);
}

#[test]
fn remove_all_except_current_dir() {
    let original = dbg!(vec!["foo", "bar", ".", "abc/def", "0/1/2"]);
    let mut actual = original.clone();
    remove_overlapping_paths::<MockedApi>(&mut actual);
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
    remove_overlapping_paths::<MockedApi>(&mut actual);
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
    remove_overlapping_paths::<MockedApi>(&mut actual);
    let expected = vec![MOCKED_CURRENT_DIR];
    assert_eq!(actual, expected);
}

#[test]
fn remove_all_except_parent_dir() {
    let original = dbg!(vec!["foo", "bar", "..", "abc/def", ".", "0/1/2"]);
    let mut actual = original.clone();
    remove_overlapping_paths::<MockedApi>(&mut actual);
    let expected = vec![".."];
    assert_eq!(actual, expected);

    let original = dbg!(vec![
        "foo",
        "/home/user",
        "bar",
        "..",
        "abc/def",
        ".",
        "0/1/2",
    ]);
    let mut actual = original.clone();
    remove_overlapping_paths::<MockedApi>(&mut actual);
    let expected = vec!["/home/user"];
    assert_eq!(actual, expected);
}

#[test]
fn remove_overlapping_real_paths() {
    let original = dbg!(vec![
        "foo",
        "bar",
        "abc/def",
        "link-to-foo/child",
        "link-to-bar/a/b/c",
        "0/1/2",
    ]);
    let mut actual = original.clone();
    remove_overlapping_paths::<MockedApi>(&mut actual);
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
    remove_overlapping_paths::<MockedApi>(&mut actual);
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
    remove_overlapping_paths::<MockedApi>(&mut actual);
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
    remove_overlapping_paths::<MockedApi>(&mut actual);
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
    remove_overlapping_paths::<MockedApi>(&mut actual);
    let expected = original;
    assert_eq!(actual, expected);
}
