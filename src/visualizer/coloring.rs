use super::{ChildPosition, TreeHorizontalSlice};
use crate::ls_colors::LsColors;
use derive_more::Display;
use std::{collections::HashMap, ffi::OsStr, fmt};
use zero_copy_pads::Width;

/// Coloring configuration: ANSI prefix strings from the environment and a full-path-to-color map.
#[derive(Debug)]
pub struct Coloring<'a> {
    ls_colors: LsColors,
    map: HashMap<Vec<&'a OsStr>, Color>,
}

impl<'a> Coloring<'a> {
    /// Create a new [`Coloring`] from LS_COLORS prefixes and a path-components-to-color map.
    pub fn new(ls_colors: LsColors, map: HashMap<Vec<&'a OsStr>, Color>) -> Self {
        Coloring { ls_colors, map }
    }

    /// Return `(color, ls_colors)` for a node, used to build a colored slice for rendering.
    pub(super) fn node_color(
        &self,
        path_components: &[&'a OsStr],
        has_children: bool,
    ) -> Option<(Color, &LsColors)> {
        let color = if has_children {
            Some(Color::Directory)
        } else {
            self.map.get(path_components).copied()
        }?;
        Some((color, &self.ls_colors))
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
    pub(super) fn ansi_prefix(self, prefixes: &LsColors) -> AnsiPrefix<'_> {
        AnsiPrefix(prefixes.prefix_str(self))
    }
}

/// ANSI prefix wrapper for a [`Color`] variant, implements [`Display`].
#[derive(Display)]
pub struct AnsiPrefix<'a>(&'a str);

impl AnsiPrefix<'_> {
    /// Returns the reset suffix to emit after this prefix, or `""` if no prefix.
    pub(super) fn suffix(&self) -> &'static str {
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

/// Either a [`TreeHorizontalSlice`] (colorless) or a [`ColoredTreeHorizontalSlice`] (colorful).
#[derive(Display)]
pub(super) enum MaybeColoredTreeHorizontalSlice<'a> {
    Colorless(TreeHorizontalSlice<String>),
    Colorful(ColoredTreeHorizontalSlice<'a>),
}

impl Width for MaybeColoredTreeHorizontalSlice<'_> {
    fn width(&self) -> usize {
        match self {
            MaybeColoredTreeHorizontalSlice::Colorless(slice) => slice.width(),
            MaybeColoredTreeHorizontalSlice::Colorful(slice) => slice.width(),
        }
    }
}
