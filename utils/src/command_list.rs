use command_extra::CommandExtra;
use derive_more::{AsRef, Deref};
use pipe_trait::Pipe;
use std::process::Command;

/// Representation of a command's argument list.
#[derive(Debug, Default, Clone)]
pub struct CommandRepresentation<'a> {
    args: Vec<&'a str>,
}

impl<'a> CommandRepresentation<'a> {
    /// Add an argument.
    pub fn arg(mut self, arg: &'a str) -> Self {
        self.args.push(arg);
        self
    }
}

/// List of argument lists to run a program with.
#[derive(Debug, Clone, AsRef, Deref)]
pub struct CommandList<'a>(Vec<CommandRepresentation<'a>>);

impl<'a> Default for CommandList<'a> {
    /// Initialize a list with one empty argument list.
    fn default() -> Self {
        CommandRepresentation::default()
            .pipe(|x| vec![x])
            .pipe(CommandList)
    }
}

impl<'a> CommandList<'a> {
    /// Duplicate the list with a flag argument.
    ///
    /// The resulting list would include the original list with the flag
    /// followed by the original list without the flag.
    pub fn flag_matrix(self, name: &'a str) -> Self {
        Self::assert_flag(name);
        let CommandList(list) = self;
        list.clone()
            .into_iter()
            .map(|cmd| cmd.arg(name))
            .chain(list)
            .collect::<Vec<_>>()
            .pipe(CommandList)
    }

    /// Duplicate the list with one or many option argument(s).
    ///
    /// The resulting list would include the original list with the option(s)
    /// followed by the original list without the option(s).
    pub fn option_matrix<const LEN: usize>(self, name: &'a str, values: [&'a str; LEN]) -> Self {
        Self::assert_flag(name);
        let CommandList(tail) = self;
        let mut head: Vec<_> = values
            .iter()
            .copied()
            .flat_map(|value| {
                tail.clone()
                    .into_iter()
                    .map(move |cmd| cmd.arg(name).arg(value))
            })
            .collect();
        head.extend(tail);
        CommandList(head)
    }

    /// Create a list of [commands](Command) that run `program`.
    pub fn commands(&'a self, program: &'a str) -> impl Iterator<Item = Command> + 'a {
        self.iter()
            .map(move |cmd| Command::new(program).with_args(&cmd.args))
    }

    /// Make sure a flag name has valid syntax.
    fn assert_flag(name: &str) {
        match name.len() {
            0 | 1 => panic!("{name:?} is not a valid flag"),
            2 => assert!(name.starts_with('-'), "{name:?} is not a valid flag"),
            _ => assert!(name.starts_with("--"), "{name:?} is not a valid flag"),
        }
    }
}
