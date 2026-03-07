use super::{ChildPosition, TreeHorizontalSlice};
use crate::ls_colors::LsColors;
use derive_more::Display;
use std::{
    collections::HashMap,
    fmt,
    path::{Path, PathBuf},
};
use zero_copy_pads::Width;

/// Coloring configuration: ANSI prefix strings from the environment and a full-path-to-color map.
#[derive(Debug)]
pub struct Coloring {
    ansi_prefixes: LsColors,
    map: HashMap<PathBuf, Color>,
}

impl Coloring {
    /// Create a new [`Coloring`] from ANSI prefixes and a full-path-to-color map.
    pub fn new(ansi_prefixes: LsColors, map: HashMap<PathBuf, Color>) -> Self {
        Coloring { ansi_prefixes, map }
    }

    /// Return `(color, prefixes)` for a node, used to build a colored slice for rendering.
    pub(super) fn node_color(&self, path: &Path, has_children: bool) -> Option<(Color, &LsColors)> {
        let color = if has_children {
            Some(Color::Directory)
        } else {
            self.map.get(path).copied()
        }?;
        Some((color, &self.ansi_prefixes))
    }
}

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
    pub fn ansi_prefix(self, prefixes: &LsColors) -> AnsiPrefix<'_> {
        AnsiPrefix(prefixes.prefix_str(self))
    }
}

/// ANSI prefix wrapper for a [`Color`] variant, implements [`Display`].
#[derive(Display)]
pub struct AnsiPrefix<'a>(&'a str);

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

/// A [`TreeHorizontalSlice`] with its color applied, used for rendering.
pub(super) struct ColoredTreeHorizontalSlice<'a> {
    pub(super) slice: TreeHorizontalSlice<String>,
    pub(super) color: Color,
    pub(super) ls_colors: &'a LsColors,
}

impl fmt::Display for ColoredTreeHorizontalSlice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let TreeHorizontalSlice {
            ancestor_relative_positions,
            skeletal_component,
            name,
        } = &self.slice;
        for pos in ancestor_relative_positions {
            let connector = match pos {
                ChildPosition::Init => "│ ",
                ChildPosition::Last => "  ",
            };
            write!(f, "{connector}")?;
        }
        let prefix = self.color.ansi_prefix(self.ls_colors);
        let suffix = prefix.suffix();
        write!(f, "{skeletal_component}{prefix}{name}{suffix}")
    }
}

impl Width for ColoredTreeHorizontalSlice<'_> {
    fn width(&self) -> usize {
        self.slice.width()
    }
}
