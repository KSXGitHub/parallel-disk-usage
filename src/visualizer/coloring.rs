/// The coloring to apply to a node name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// Color as a directory.
    Directory,
    /// Color as a regular file.
    Normal,
    /// Color as an executable file.
    Executable,
    /// Color as a symbolic link.
    Symlink,
}
