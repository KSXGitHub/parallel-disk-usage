use super::{ChildPosition, TreeHorizontalSlice};
use crate::ls_colors::LsColors;
use derive_more::Display;
use std::{collections::HashMap, fmt, hash::Hash};
use zero_copy_pads::Width;

/// Coloring configuration: ANSI prefix strings from the environment and a name-to-color map.
#[derive(Debug)]
pub struct Coloring<Name> {
    ansi_prefixes: LsColors,
    map: HashMap<Name, Color>,
}

impl<Name: Hash + Eq> Coloring<Name> {
    /// Create a new [`Coloring`] from ANSI prefixes and a name-to-color map.
    pub fn new(ansi_prefixes: LsColors, map: HashMap<Name, Color>) -> Self {
        Coloring { ansi_prefixes, map }
    }

    /// Return `(color, prefixes)` for a node, used to build a colored slice for rendering.
    pub(crate) fn node_color(&self, name: &Name, has_children: bool) -> Option<(Color, &LsColors)> {
        let color = if has_children {
            Some(Color::Directory)
        } else {
            self.map.get(name).copied()
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
pub(crate) struct ColoredTreeHorizontalSlice<'a> {
    pub(crate) slice: TreeHorizontalSlice<String>,
    pub(crate) color: Color,
    pub(crate) ansi_prefixes: &'a LsColors,
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
        let prefix = self.color.ansi_prefix(self.ansi_prefixes);
        let suffix = prefix.suffix();
        write!(f, "{skeletal_component}{prefix}{name}{suffix}")
    }
}

impl Width for ColoredTreeHorizontalSlice<'_> {
    fn width(&self) -> usize {
        self.slice.width()
    }
}
