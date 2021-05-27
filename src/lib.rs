#![deny(warnings)]

mod utils;

pub mod app;
pub mod args;
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

/// The main program.
pub fn main() {
    if let Err(error) = app::App::from_env().run() {
        eprintln!("[error] {}", error);
    }
}

pub use structopt;
pub use structopt::clap;
pub use structopt_utilities;
pub use strum;
pub use zero_copy_pads;
