use crate::args::Args;
use clap::CommandFactory;
use clap_mangen::Man;
use std::io;

/// Renders the man page for `pdu` as a string.
pub fn render_man_page() -> io::Result<String> {
    let command = Args::command();
    let man = Man::new(command);
    let mut buffer = Vec::new();
    man.render(&mut buffer)?;
    let content = String::from_utf8(buffer)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok(content.replace("\n.SH EXTRA\n", "\n.SH EXAMPLES\n"))
}
