use super::{Direction, Visualizer};
use crate::{
    measurement_system::MeasurementSystem,
    size::{Bytes, Size},
    tree::Tree,
};
use pretty_assertions::assert_eq;
use text_block_macros::text_block_fnl;

fn nested_tree<Data: Size>(
    dir_names: &[&'static str],
    size_per_dir: Data,
    file_name: &'static str,
    file_size: Data,
) -> Tree<&'static str, Data> {
    if let Some((head, tail)) = dir_names.split_first() {
        let child = nested_tree(tail, size_per_dir, file_name, file_size);
        Tree::dir(*head, size_per_dir, vec![child])
    } else {
        Tree::file(file_name, file_size)
    }
}

#[test]
fn nested() {
    let tree = nested_tree::<Bytes>(
        &["a", "b", "c", "d", "e", "f"],
        4096.into(),
        "z",
        1024.into(),
    );
    let actual = Visualizer {
        tree: &tree,
        direction: Direction::BottomUp,
        max_depth: 10,
        max_width: 150,
        measurement_system: MeasurementSystem::Binary,
    }
    .to_string();
    let expected = text_block_fnl! {
        " 1K             ┌──z│                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓█████│  4%"
        " 5K           ┌─┴f  │                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓████████████████████████│ 20%"
        " 9K         ┌─┴e    │                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓██████████████████████████████████████████│ 36%"
        "13K       ┌─┴d      │                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒█████████████████████████████████████████████████████████████│ 52%"
        "17K     ┌─┴c        │                   ░░░░░░░░░░░░░░░░░░░████████████████████████████████████████████████████████████████████████████████│ 68%"
        "21K   ┌─┴b          │                   ███████████████████████████████████████████████████████████████████████████████████████████████████│ 84%"
        "25K ┌─┴a            │██████████████████████████████████████████████████████████████████████████████████████████████████████████████████████│100%"
    };
    eprintln!("\nACTUAL:\n{}\n", &actual);
    assert_eq!(actual, expected);
}
