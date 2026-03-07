use crate::visualizer::coloring::Color;
#[cfg(feature = "color")]
use lscolors::{self, Indicator};

/// ANSI color prefix strings for terminal output, initialized from the `LS_COLORS` environment
/// variable.
#[derive(Debug, Clone)]
pub struct LsColors {
    #[cfg(feature = "color")]
    directory: String,
    #[cfg(feature = "color")]
    normal: String,
    #[cfg(feature = "color")]
    executable: String,
    #[cfg(feature = "color")]
    symlink: String,
}

impl LsColors {
    /// Initialize by reading the current environment's `LS_COLORS`.
    pub fn from_env() -> Self {
        #[cfg(feature = "color")]
        return Self::from_ls_colors_crate(&lscolors::LsColors::from_env().unwrap_or_default());
        #[cfg(not(feature = "color"))]
        LsColors {}
    }

    /// Parse an `LS_COLORS`-format string into an [`LsColors`].
    ///
    /// Unrecognized or invalid entries are silently ignored.
    pub fn from_ls_colors_string(input: &str) -> Self {
        #[cfg(feature = "color")]
        return Self::from_ls_colors_crate(&lscolors::LsColors::from_string(input));
        #[cfg(not(feature = "color"))]
        {
            let _ = input;
            LsColors {}
        }
    }

    #[cfg(feature = "color")]
    fn from_ls_colors_crate(ls_colors: &lscolors::LsColors) -> Self {
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
        #[cfg(feature = "color")]
        return match color {
            Color::Directory => &self.directory,
            Color::Normal => &self.normal,
            Color::Executable => &self.executable,
            Color::Symlink => &self.symlink,
        };
        #[cfg(not(feature = "color"))]
        {
            let _ = color;
            ""
        }
    }
}
