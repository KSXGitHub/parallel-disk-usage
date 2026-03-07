use crate::AnsiPrefixes;
use std::{collections::HashMap, fmt, hash::Hash};

/// The coloring to apply to a node name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// Color as a directory.
    Directory,
    /// Color as a regular file.
    Normal,
    /// Color as an executable file.
    Executable,
    /// Color as a symbolic link.
    Symlink,
}

impl Color {
    /// Get the ANSI prefix for this color from the given prefix table.
    pub fn ansi_prefix(self, prefixes: &AnsiPrefixes) -> AnsiPrefix<'_> {
        AnsiPrefix(match self {
            Color::Directory => &prefixes.directory,
            Color::Normal => &prefixes.normal,
            Color::Executable => &prefixes.executable,
            Color::Symlink => &prefixes.symlink,
        })
    }
}

/// ANSI prefix wrapper for a [`Color`] variant, implements [`fmt::Display`].
pub struct AnsiPrefix<'a>(pub(crate) &'a str);

impl fmt::Display for AnsiPrefix<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl AnsiPrefix<'_> {
    /// Returns the reset suffix to emit after this prefix, or `""` if no prefix.
    pub fn suffix(&self) -> &'static str {
        if self.0.is_empty() {
            ""
        } else {
            "\x1b[0m"
        }
    }
}

/// Coloring configuration: ANSI prefix strings from the environment and a name-to-color map.
#[derive(Debug)]
pub struct Coloring<Name> {
    /// ANSI prefix strings read from `LS_COLORS`.
    pub(crate) ansi_prefixes: AnsiPrefixes,
    /// Map from node name to color.
    pub(crate) map: HashMap<Name, Color>,
}

impl<Name: Hash + Eq> Coloring<Name> {
    /// Create a new [`Coloring`] from ANSI prefixes and a name-to-color map.
    pub fn new(ansi_prefixes: AnsiPrefixes, map: HashMap<Name, Color>) -> Self {
        Coloring { ansi_prefixes, map }
    }
}
