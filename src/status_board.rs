use fmt_iter::repeat;
use std::sync::atomic::{AtomicUsize, Ordering};
use zero_copy_pads::Width;

/// Control all status indicators in stderr.
pub static GLOBAL_STATUS_BOARD: StatusBoard = StatusBoard::new();

/// Control all status indicators in stderr.
#[derive(Debug)]
pub struct StatusBoard {
    line_width: AtomicUsize,
}

impl StatusBoard {
    /// Create a new [`StatusBoard`].
    const fn new() -> Self {
        StatusBoard {
            line_width: AtomicUsize::new(0),
        }
    }

    /// Get the number of characters of the current line.
    fn get_line_width(&self) -> usize {
        self.line_width.load(Ordering::Relaxed)
    }

    /// Set the number of characters of the current line.
    fn set_line_width(&self, value: usize) {
        self.line_width.store(value, Ordering::Relaxed);
    }

    /// Clear the line that the cursor is pointing to.
    pub fn clear_line(&self, new_line_width: usize) {
        let empty_line = repeat(' ', self.get_line_width());
        eprint!("\r{}\r", empty_line);
        self.set_line_width(new_line_width);
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
