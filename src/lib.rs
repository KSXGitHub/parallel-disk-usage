#![deny(warnings)]

#[cfg(feature = "json")]
pub use serde;
#[cfg(feature = "json")]
pub use serde_json;

#[cfg(feature = "cli")]
pub mod app;
#[cfg(feature = "cli")]
pub mod args;
#[cfg(feature = "cli")]
pub mod runtime_error;

/// The main program.
#[cfg(feature = "cli")]
pub fn main() -> std::process::ExitCode {
    if let Err(error) = app::App::from_env().run() {
        eprintln!("[error] {}", error);
        return std::process::ExitCode::FAILURE;
    }
    std::process::ExitCode::SUCCESS
}

#[cfg(feature = "cli")]
pub use structopt;
#[cfg(feature = "cli")]
pub use structopt::clap;
#[cfg(feature = "cli")]
pub use structopt_utilities;

pub mod bytes_format;
pub mod data_tree;
pub mod fs_tree_builder;
pub mod json_data;
pub mod os_string_display;
pub mod reporter;
pub mod size;
pub mod size_getters;
pub mod status_board;
pub mod tree_builder;
pub mod visualizer;

pub use strum;
pub use zero_copy_pads;
