pub mod app;
pub mod args;
pub mod size;
pub mod tree;

pub use app::App;
pub use args::Args;

pub fn main() {
    App::from_env().run()
}
