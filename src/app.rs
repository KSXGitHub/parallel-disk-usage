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
        let Args { copyright, files } = self.args;

        if copyright {
            println!("Apache-2.0 © 2021 Hoàng Văn Khải <https://ksxgithub.github.io/>");
            println!("Donation: https://patreon.com/khai96_");
            return;
        }

        dbg!(files);
    }
}
