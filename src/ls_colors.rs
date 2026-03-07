use crate::visualizer::coloring::Color;
use lscolors::{Indicator, LsColors as LsColorsCrate};

/// ANSI color prefix strings for terminal output, initialized from the `LS_COLORS` environment
/// variable.
#[derive(Debug, Clone)]
pub struct LsColors {
    directory: String,
    normal: String,
    executable: String,
    symlink: String,
}

impl LsColors {
    /// Initialize by reading the current environment's `LS_COLORS`.
    pub fn from_env() -> Self {
        let ls_colors = LsColorsCrate::from_env().unwrap_or_default();
        let prefix_for = |indicator: Indicator| {
            ls_colors
                .style_for_indicator(indicator)
                .map(|s| s.to_nu_ansi_term_style().prefix().to_string())
                .unwrap_or_default()
        };
        LsColors {
            directory: prefix_for(Indicator::Directory),
            normal: prefix_for(Indicator::RegularFile),
            executable: prefix_for(Indicator::ExecutableFile),
            symlink: prefix_for(Indicator::SymbolicLink),
        }
    }

    /// Return the ANSI prefix string for the given [`Color`] variant.
    pub(crate) fn prefix_str(&self, color: Color) -> &str {
        match color {
            Color::Directory => &self.directory,
            Color::Normal => &self.normal,
            Color::Executable => &self.executable,
            Color::Symlink => &self.symlink,
        }
    }
}
