use lscolors::{Indicator, LsColors};

/// ANSI prefix strings for each color variant, read from `LS_COLORS`.
#[derive(Debug, Clone)]
pub struct AnsiPrefixes {
    pub(crate) directory: String,
    pub(crate) normal: String,
    pub(crate) executable: String,
    pub(crate) symlink: String,
}

impl AnsiPrefixes {
    /// Initialize by reading the current environment's `LS_COLORS`.
    pub fn from_env() -> Self {
        let ls_colors = LsColors::from_env().unwrap_or_default();
        let prefix_for = |indicator: Indicator| {
            ls_colors
                .style_for_indicator(indicator)
                .map(|s| s.to_nu_ansi_term_style().prefix().to_string())
                .unwrap_or_default()
        };
        AnsiPrefixes {
            directory: prefix_for(Indicator::Directory),
            normal: prefix_for(Indicator::RegularFile),
            executable: prefix_for(Indicator::ExecutableFile),
            symlink: prefix_for(Indicator::SymbolicLink),
        }
    }
}
