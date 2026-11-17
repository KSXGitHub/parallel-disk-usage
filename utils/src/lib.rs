//! Test-support building blocks for parallel-disk-usage's integration tests.
//!
//! Every helper here is independent of the `parallel_disk_usage` library
//! crate, so this crate builds on its own. The helpers that do read the
//! library live in `tests/_utils.rs`, which re-exports this crate alongside
//! them, so a test only ever names `_utils`.
//!
//! The path of a program to run is taken as a parameter rather than
//! discovered, because this crate is built as an ordinary dependency and
//! therefore has no access to the `CARGO_BIN_EXE_*` variables that Cargo
//! sets for a test target.

pub mod command_list;
pub mod metadata;
pub mod process_output;
pub mod sample_workspace;
pub mod temp;

pub use command_list::{CommandList, CommandRepresentation};
pub use metadata::read_apparent_size;
#[cfg(unix)]
pub use metadata::read_inode_number;
pub use process_output::{inspect_stderr, stdout_text};
pub use sample_workspace::SampleWorkspace;
pub use temp::Temp;
