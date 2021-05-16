use fmt_iter::repeat;
use std::sync::atomic::{AtomicUsize, Ordering};
use zero_copy_pads::Width;

/// Control all status indicators in stderr.
pub static GLOBAL_STATUS_BOARD: StatusBoard = StatusBoard::new();

/// Control all status indicators in stderr.
#[derive(Debug)]
pub struct StatusBoard {
    line_len: AtomicUsize,
}

impl StatusBoard {
    /// Create a new [`StatusBoard`].
    const fn new() -> Self {
        StatusBoard {
            line_len: AtomicUsize::new(0),
        }
    }

    /// Get the length of the current line.
    fn get_line_len(&self) -> usize {
        self.line_len.load(Ordering::Relaxed)
    }

    /// Set the length of the current line.
    fn set_line_len(&self, value: usize) {
        self.line_len.store(value, Ordering::Relaxed);
    }

    /// Clear the line that the cursor is pointing to.
    pub fn clear_line(&self, new_line_len: usize) {
        let empty_line = repeat(' ', self.get_line_len());
        eprint!("\r{}\r", empty_line);
        self.set_line_len(new_line_len);
    }

    /// Show a temporary message.
    pub fn temporary_message(&self, message: &str) {
        self.clear_line(message.width());
        eprint!("{}", message);
    }

    /// Log a permanent message.
    pub fn permanent_message(&self, message: &str) {
        self.clear_line(0);
        eprintln!("{}", message);
    }
}
