use super::path_name;
use crate::os_string_display::OsStringDisplay;
use pretty_assertions::assert_eq;
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
    let actual = path_name(&PathBuf::from(r"C:\"));
    let expected = OsStringDisplay::os_string_from(r"C:\");
    assert_eq!(actual, expected);
}

#[cfg(windows)]
#[test]
fn prefix() {
    let actual = path_name(&PathBuf::from(r"\\prefix"));
    let expected = OsStringDisplay::os_string_from("prefix");
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
