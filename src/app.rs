use super::Args;
use structopt_utilities::StructOptUtils;

/// The main application.
pub struct App {
    /// The CLI arguments.
    args: Args,
}

impl App {
    /// Initialize the application from the environment.
    pub fn from_env() -> Self {
        App {
            args: Args::strict_from_args(),
        }
    }

    /// Run the application.
    pub fn run(self) {
        let Args { files } = self.args;

        dbg!(files);
    }
}
