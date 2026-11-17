//! Render the man page of the `pdu` command.
//!
//! `generate-completions.sh` runs this tool and writes the result to
//! `exports/pdu.1`.

use parallel_disk_usage::man_page::render_man_page;

fn main() {
    print!("{}", render_man_page());
}
