use crate::{data_tree::DataTree, os_string_display::OsStringDisplay, size};
use std::collections::HashMap;

/// Color information for visualizing a tree.
///
/// The `file_colors` field maps the `Display` representation of leaf/childless
/// node names (type `Name` in `Visualizer`, i.e. `OsStringDisplay`) to
/// pre-rendered ANSI escape sequence prefixes.
#[derive(Debug)]
pub struct Coloring {
    /// Maps leaf/childless node name strings to their ANSI prefix escape sequence.
    file_colors: HashMap<String, String>,
    /// ANSI prefix escape sequence for directories (nodes with children).
    dir_prefix: String,
}

/// The ANSI reset sequence.
const ANSI_RESET: &str = "\x1b[0m";

impl Coloring {
    /// Build a `Coloring` from a pruned `DataTree` by querying `LS_COLORS`.
    ///
    /// This should be called only after pruning to save CPU/IO cycles.
    #[cfg(feature = "lscolors")]
    pub fn from_tree<Size>(tree: &DataTree<OsStringDisplay, Size>) -> Self
    where
        Size: size::Size,
    {
        use lscolors::{Indicator, LsColors};

        let ls_colors = LsColors::from_env().unwrap_or_default();
        let mut file_colors = HashMap::new();
        collect_leaf_colors(tree, &ls_colors, &mut file_colors);
        let dir_prefix = ls_colors
            .style_for_indicator(Indicator::Directory)
            .map(style_to_ansi_prefix)
            .unwrap_or_default();
        Coloring {
            file_colors,
            dir_prefix,
        }
    }

    /// Get the ANSI prefix and suffix for a given name.
    ///
    /// If the name is in `file_colors`, it's a file or childless directory.
    /// Otherwise it's a directory with children — use `dir_prefix`.
    ///
    /// Returns `(prefix, suffix)` where suffix is the ANSI reset sequence.
    /// Returns `("", "")` if no style applies.
    pub fn ansi_for(&self, name: &str) -> (&str, &str) {
        let prefix = self
            .file_colors
            .get(name)
            .map(String::as_str)
            .unwrap_or(&self.dir_prefix);
        if prefix.is_empty() {
            ("", "")
        } else {
            (prefix, ANSI_RESET)
        }
    }
}

/// Recursively collect ANSI prefix strings for all childless nodes in the tree.
#[cfg(feature = "lscolors")]
fn collect_leaf_colors<Size>(
    tree: &DataTree<OsStringDisplay, Size>,
    ls_colors: &lscolors::LsColors,
    map: &mut HashMap<String, String>,
) where
    Size: size::Size,
{
    if tree.children().is_empty() {
        // Leaf node: file or childless directory
        let prefix = ls_colors
            .style_for_path(tree.name().as_os_str())
            .map(style_to_ansi_prefix)
            .unwrap_or_default();
        map.insert(tree.name().to_string(), prefix);
    } else {
        for child in tree.children() {
            collect_leaf_colors(child, ls_colors, map);
        }
    }
}

/// Convert an `lscolors::style::Style` to an ANSI escape sequence prefix string.
#[cfg(feature = "lscolors")]
fn style_to_ansi_prefix(style: &lscolors::style::Style) -> String {
    let mut codes: Vec<String> = Vec::new();

    let fs = &style.font_style;
    if fs.bold {
        codes.push("1".into());
    }
    if fs.dimmed {
        codes.push("2".into());
    }
    if fs.italic {
        codes.push("3".into());
    }
    if fs.underline {
        codes.push("4".into());
    }
    if fs.slow_blink {
        codes.push("5".into());
    }
    if fs.rapid_blink {
        codes.push("6".into());
    }
    if fs.reverse {
        codes.push("7".into());
    }
    if fs.hidden {
        codes.push("8".into());
    }
    if fs.strikethrough {
        codes.push("9".into());
    }

    if let Some(ref color) = style.foreground {
        color_to_ansi_codes(color, 30, &mut codes);
    }

    if let Some(ref color) = style.background {
        color_to_ansi_codes(color, 40, &mut codes);
    }

    if let Some(ref color) = style.underline {
        match color {
            lscolors::style::Color::Fixed(n) => {
                codes.push(format!("58;5;{n}"));
            }
            lscolors::style::Color::RGB(r, g, b) => {
                codes.push(format!("58;2;{r};{g};{b}"));
            }
            _ => {}
        }
    }

    if codes.is_empty() {
        String::new()
    } else {
        format!("\x1b[{}m", codes.join(";"))
    }
}

#[cfg(feature = "lscolors")]
fn color_to_ansi_codes(color: &lscolors::style::Color, base: u8, codes: &mut Vec<String>) {
    use lscolors::style::Color;
    match color {
        Color::Black => codes.push(format!("{}", base)),
        Color::Red => codes.push(format!("{}", base + 1)),
        Color::Green => codes.push(format!("{}", base + 2)),
        Color::Yellow => codes.push(format!("{}", base + 3)),
        Color::Blue => codes.push(format!("{}", base + 4)),
        Color::Magenta => codes.push(format!("{}", base + 5)),
        Color::Cyan => codes.push(format!("{}", base + 6)),
        Color::White => codes.push(format!("{}", base + 7)),
        Color::BrightBlack => codes.push(format!("{}", base + 60)),
        Color::BrightRed => codes.push(format!("{}", base + 61)),
        Color::BrightGreen => codes.push(format!("{}", base + 62)),
        Color::BrightYellow => codes.push(format!("{}", base + 63)),
        Color::BrightBlue => codes.push(format!("{}", base + 64)),
        Color::BrightMagenta => codes.push(format!("{}", base + 65)),
        Color::BrightCyan => codes.push(format!("{}", base + 66)),
        Color::BrightWhite => codes.push(format!("{}", base + 67)),
        Color::Fixed(n) => codes.push(format!("{};5;{n}", base + 8)),
        Color::RGB(r, g, b) => codes.push(format!("{};2;{r};{g};{b}", base + 8)),
    }
}
