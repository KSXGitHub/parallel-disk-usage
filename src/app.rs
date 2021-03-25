use super::Args;
use structopt_utilities::StructOptUtils;

pub struct App {
    args: Args,
}

impl App {
    pub fn from_env() -> Self {
        App {
            args: Args::strict_from_args(),
        }
    }

    pub fn run(self) {
        let Args { copyright, files } = self.args;

        if copyright {
            println!("Apache-2.0 © 2021 Hoàng Văn Khải");
            return;
        }

        dbg!(files);
    }
}
