pub mod app;
pub mod args;

pub use app::App;
pub use args::Args;

pub fn main() {
    App::from_env().run()
}
