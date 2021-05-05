use super::{Direction, Visualizer};
use crate::{
    measurement_system::MeasurementSystem,
    size::{Bytes, Size},
    tree::Tree,
};
use pretty_assertions::assert_eq;
use text_block_macros::text_block_fnl;

macro_rules! test_case {
    (
        $name:ident where
        tree = $tree:expr,
        max_depth = $max_depth:expr,
        max_width = $max_width:expr,
        direction = $direction:ident,
        measurement_system = $measurement_system:ident,
        expected = $expected:expr,
    ) => {
        #[test]
        fn $name() {
            let tree = $tree;
            let actual = Visualizer {
                tree: &tree,
                max_depth: $max_depth,
                max_width: $max_width,
                direction: Direction::$direction,
                measurement_system: MeasurementSystem::$measurement_system,
            }
            .to_string();
            let expected = $expected;
            eprintln!("\nACTUAL:\n{}\n", &actual);
            let actual: Vec<_> = actual.split('\n').collect();
            let expected: Vec<_> = expected.split('\n').collect();
            assert_eq!(actual, expected);
        }
    };
}

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

test_case! {
    nested_bottom_up_binary where
        tree = nested_tree::<Bytes>(
            &["a", "b", "c", "d", "e", "f"],
            4096.into(),
            "z",
            1024.into(),
        ),
        max_depth = 10,
        max_width = 150,
        direction = BottomUp,
        measurement_system = Binary,
        expected = text_block_fnl! {
            " 1K             ┌──z│                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓█████│  4%"
            " 5K           ┌─┴f  │                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓████████████████████████│ 20%"
            " 9K         ┌─┴e    │                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓██████████████████████████████████████████│ 36%"
            "13K       ┌─┴d      │                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒█████████████████████████████████████████████████████████████│ 52%"
            "17K     ┌─┴c        │                   ░░░░░░░░░░░░░░░░░░░████████████████████████████████████████████████████████████████████████████████│ 68%"
            "21K   ┌─┴b          │                   ███████████████████████████████████████████████████████████████████████████████████████████████████│ 84%"
            "25K ┌─┴a            │██████████████████████████████████████████████████████████████████████████████████████████████████████████████████████│100%"
        },
}

test_case! {
    nested_bottom_up_metric where
        tree = nested_tree::<Bytes>(
            &["a", "b", "c", "d", "e", "f"],
            4096.into(),
            "z",
            1024.into(),
        ),
        max_depth = 10,
        max_width = 150,
        direction = BottomUp,
        measurement_system = Metric,
        expected = text_block_fnl! {
            " 1K             ┌──z│                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓█████│  4%"
            " 5K           ┌─┴f  │                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓████████████████████████│ 20%"
            " 9K         ┌─┴e    │                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓██████████████████████████████████████████│ 36%"
            "13K       ┌─┴d      │                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒█████████████████████████████████████████████████████████████│ 52%"
            "17K     ┌─┴c        │                   ░░░░░░░░░░░░░░░░░░░████████████████████████████████████████████████████████████████████████████████│ 68%"
            "22K   ┌─┴b          │                   ███████████████████████████████████████████████████████████████████████████████████████████████████│ 84%"
            "26K ┌─┴a            │██████████████████████████████████████████████████████████████████████████████████████████████████████████████████████│100%"
        },
}

test_case! {
    nested_top_down_binary where
        tree = nested_tree::<Bytes>(
            &["a", "b", "c", "d", "e", "f"],
            4096.into(),
            "z",
            1024.into(),
        ),
        max_depth = 10,
        max_width = 150,
        direction = TopDown,
        measurement_system = Binary,
        expected = text_block_fnl! {
            "25K └─┬a            │██████████████████████████████████████████████████████████████████████████████████████████████████████████████████████│100%"
            "21K   └─┬b          │                   ███████████████████████████████████████████████████████████████████████████████████████████████████│ 84%"
            "17K     └─┬c        │                   ░░░░░░░░░░░░░░░░░░░████████████████████████████████████████████████████████████████████████████████│ 68%"
            "13K       └─┬d      │                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒█████████████████████████████████████████████████████████████│ 52%"
            " 9K         └─┬e    │                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓██████████████████████████████████████████│ 36%"
            " 5K           └─┬f  │                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓████████████████████████│ 20%"
            " 1K             └──z│                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓█████│  4%"
        },
}
