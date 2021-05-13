pub mod app;
pub mod args;
pub mod bytes_format;
pub mod data_tree;
pub mod fs_tree_builder;
pub mod os_string_display;
pub mod reporter;
pub mod size;
pub mod size_getters;
pub mod tree_builder;
pub mod visualizer;

/// The main program.
pub fn main() {
    app::App::from_env().run()
}

pub use structopt;
pub use structopt::clap;
pub use structopt_utilities;
pub use strum;
pub use zero_copy_pads;
