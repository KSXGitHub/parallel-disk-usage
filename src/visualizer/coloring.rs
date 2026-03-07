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

    /// Look up the color for a node identified by its path components and whether it has children,
    /// then wrap the given [`TreeHorizontalSlice`] in the appropriate colored or colorless variant.
    fn maybe_colored_tree_slice(
        &self,
        path_components: &[&'a OsStr],
        has_children: bool,
        slice: TreeHorizontalSlice<String>,
    ) -> MaybeColoredTreeHorizontalSlice<'_> {
        let color = if has_children {
            Some(Color::Directory)
        } else {
            self.map.get(path_components).copied()
        };
        match color {
            Some(color) => MaybeColoredTreeHorizontalSlice::Colorful(ColoredTreeHorizontalSlice {
                slice,
                color,
                ls_colors: &self.ls_colors,
            }),
            None => MaybeColoredTreeHorizontalSlice::Colorless(slice),
        }
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
    fn ansi_prefix(self, prefixes: &LsColors) -> AnsiPrefix<'_> {
        AnsiPrefix(prefixes.prefix_str(self))
    }
}

/// ANSI prefix wrapper for a [`Color`] variant, implements [`Display`].
#[derive(Display)]
struct AnsiPrefix<'a>(&'a str);

impl AnsiPrefix<'_> {
    /// Returns the reset suffix to emit after this prefix, or `""` if no prefix.
    fn suffix(&self) -> &'static str {
        if self.0.is_empty() {
            ""
        } else {
            "\x1b[0m"
        }
    }
}

/// A [`TreeHorizontalSlice`] with its color applied, used for rendering.
pub(super) struct ColoredTreeHorizontalSlice<'a> {
    slice: TreeHorizontalSlice<String>,
    color: Color,
    ls_colors: &'a LsColors,
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

/// Wrap a [`TreeHorizontalSlice`] with color if coloring is available, otherwise return it as-is.
///
/// Path components are only constructed when coloring is enabled, avoiding
/// unnecessary allocation in the common no-color case.
pub(super) fn maybe_colored_slice<'a, 'b>(
    coloring: Option<&'b Coloring<'a>>,
    ancestors: impl Iterator<Item = &'a OsStr>,
    name: &'a OsStr,
    has_children: bool,
    slice: TreeHorizontalSlice<String>,
) -> MaybeColoredTreeHorizontalSlice<'b> {
    match coloring {
        Some(coloring) => {
            let path_components: Vec<&OsStr> = ancestors.chain(std::iter::once(name)).collect();
            coloring.maybe_colored_tree_slice(&path_components, has_children, slice)
        }
        None => MaybeColoredTreeHorizontalSlice::Colorless(slice),
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
