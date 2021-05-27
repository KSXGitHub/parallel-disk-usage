use super::os_string_display::OsStringDisplay;
use std::path::{Component::*, Path};

/// Get file name or directory name of a path.
pub fn path_name(path: &Path) -> OsStringDisplay {
    match path.components().last() {
        None | Some(CurDir) => OsStringDisplay::os_string_from("."),
        Some(Normal(name)) => OsStringDisplay::os_string_from(name),
        Some(Prefix(prefix)) => OsStringDisplay::os_string_from(prefix.as_os_str()),
        Some(RootDir) | Some(ParentDir) => OsStringDisplay::os_string_from(path),
    }
}

#[cfg(test)]
mod test_path_name;
