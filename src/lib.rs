#![deny(warnings)]

mod utils;

#[cfg(feature = "cli")]
pub mod app;
#[cfg(feature = "cli")]
pub mod args;

/// The main program.
#[cfg(feature = "cli")]
pub fn main() {
    if let Err(error) = app::App::from_env().run() {
        eprintln!("[error] {}", error);
    }
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
pub mod os_string_display;
pub mod reporter;
pub mod runtime_error;
pub mod size;
pub mod size_getters;
pub mod status_board;
pub mod tree_builder;
pub mod visualizer;

pub use strum;
pub use zero_copy_pads;
