pub mod app;
pub mod args;
pub mod size;
pub mod tree;
pub mod tree_builder;

pub use app::App;
pub use args::Args;

pub fn main() {
    App::from_env().run()
}
