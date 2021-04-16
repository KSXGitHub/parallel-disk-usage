pub mod app;
pub mod args;
pub mod fs_tree_builder;
pub mod size;
pub mod tree;
pub mod tree_builder;
pub mod progress_report;

pub use app::App;
pub use args::Args;

/// The main program.
pub fn main() {
    App::from_env().run()
}
