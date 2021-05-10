pub mod app;
pub mod args;
pub mod fs_tree_builder;
pub mod measurement_system;
pub mod reporter;
pub mod size;
pub mod size_getters;
pub mod tree;
pub mod tree_builder;
pub mod visualizer;

pub use app::App;
pub use args::Args;

/// The main program.
pub fn main() {
    App::from_env().run()
}
