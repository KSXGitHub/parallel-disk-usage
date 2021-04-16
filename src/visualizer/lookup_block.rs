/// Lookup visualization block according to distance from parent.
///
/// The closer the child, the bolder the block.
pub const fn lookup_block(level: u8) -> char {
    match level {
        0 => '█',
        1 => '▓',
        2 => '▒',
        3 => '░',
        _ => ' ',
    }
}
