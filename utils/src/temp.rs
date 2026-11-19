use derive_more::{AsRef, Deref};
use pipe_trait::Pipe;
use rand::distr::Alphanumeric;
use rand::{RngExt, rng};
use std::env::temp_dir;
use std::fs::{create_dir, remove_dir_all};
use std::io::Error;
use std::path::PathBuf;

/// Representation of a temporary filesystem item.
///
/// **NOTE:** Delete this once <https://github.com/samgiles/rs-mktemp/issues/8> is resolved.
#[derive(Debug, AsRef, Deref)]
#[as_ref(forward)]
#[deref(forward)]
pub struct Temp(PathBuf);

impl Temp {
    /// Create a temporary directory.
    pub fn new_dir() -> Result<Self, Error> {
        let path = rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .map(char::from)
            .collect::<String>()
            .pipe(|name| temp_dir().join(name));
        if path.exists() {
            return Self::new_dir();
        }
        create_dir(&path)?;
        path.pipe(Temp).pipe(Ok)
    }
}

impl Drop for Temp {
    /// Delete the created temporary directory.
    fn drop(&mut self) {
        let path = &self.0;
        if let Err(error) = remove_dir_all(path) {
            eprintln!("warning: Failed to delete {path:?}: {error}");
        }
    }
}
