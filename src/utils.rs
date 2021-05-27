use super::os_string_display::OsStringDisplay;
use std::path::{Component::*, Path};

/// Get file name or directory name of a path.
pub fn path_name(path: &Path) -> OsStringDisplay {
    match path.components().last() {
        None | Some(CurDir) => OsStringDisplay::os_string_from("."),
        Some(RootDir) => OsStringDisplay::os_string_from("/"),
        Some(Normal(name)) => OsStringDisplay::os_string_from(name),
        Some(Prefix(prefix)) => OsStringDisplay::os_string_from(prefix.as_os_str()),
        Some(ParentDir) => OsStringDisplay::os_string_from(path),
    }
}

#[cfg(test)]
use pretty_assertions::assert_eq;
#[cfg(test)]
use std::path::PathBuf;

#[test]
fn empty() {
    let actual = path_name(&PathBuf::new());
    let expected = OsStringDisplay::os_string_from(".");
    assert_eq!(actual, expected);
}

#[test]
fn current_dir() {
    let actual = path_name(&PathBuf::from("."));
    let expected = OsStringDisplay::os_string_from(".");
    assert_eq!(actual, expected);
}

#[cfg(unix)]
#[test]
fn root_dir() {
    let actual = path_name(&PathBuf::from("/"));
    let expected = OsStringDisplay::os_string_from("/");
    assert_eq!(actual, expected);
}

#[cfg(windows)]
#[test]
fn root_dir() {
    let actual = path_name(&PathBuf::from("C:\\"));
    let expected = OsStringDisplay::os_string_from("c:");
    assert_eq!(actual, expected);
}

#[cfg(unix)]
#[test]
fn normal_relative() {
    let actual = path_name(&PathBuf::from("abc/def/ghi"));
    let expected = OsStringDisplay::os_string_from("ghi");
    assert_eq!(actual, expected);
}

#[cfg(unix)]
#[test]
fn normal_absolute() {
    let actual = path_name(&PathBuf::from("/abc/def/ghi"));
    let expected = OsStringDisplay::os_string_from("ghi");
    assert_eq!(actual, expected);
}

#[cfg(unix)]
#[test]
fn normal_trailing_separator() {
    let actual = path_name(&PathBuf::from("abc/def/ghi/"));
    let expected = OsStringDisplay::os_string_from("ghi");
    assert_eq!(actual, expected);
}

#[cfg(unix)]
#[test]
fn parent_dir() {
    let actual = path_name(&PathBuf::from(".."));
    let expected = OsStringDisplay::os_string_from("..");
    assert_eq!(actual, expected);
}

#[cfg(unix)]
#[test]
fn grandparent_dir() {
    let actual = path_name(&PathBuf::from("../.."));
    let expected = OsStringDisplay::os_string_from("../..");
    assert_eq!(actual, expected);
}
