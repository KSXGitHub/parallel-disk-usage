use super::{Direction, Visualizer};
use crate::{
    measurement_system::MeasurementSystem,
    size::{Blocks, Bytes, Size},
    tree::Tree,
};
use pretty_assertions::assert_eq;
use std::cmp::Ordering;
use text_block_macros::text_block_fnl;

fn order_tree<Name, Data: Size>(left: &Tree<Name, Data>, right: &Tree<Name, Data>) -> Ordering {
    left.data().cmp(&right.data()).reverse()
}

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

fn typical_tree<Data>(size_per_dir: Data, file_size_factor: u64) -> Tree<&'static str, Data>
where
    Data: Size + Ord + From<u64> + Send,
{
    let dir = Tree::<&'static str, Data>::fixed_size_dir_constructor(size_per_dir);
    let file =
        |name: &'static str, size: u64| Tree::file(name, Data::from(size * file_size_factor));
    dir(
        "root",
        vec![
            file("foo", 2530),
            file("bar", 52),
            dir(
                "hello",
                vec![dir("world", vec![file("hello", 45), file("world", 54)])],
            ),
            dir("empty dir", vec![]),
            dir(
                "directory with a really long name",
                vec![dir(
                    "subdirectory with a really long name",
                    vec![file("file with a really long name", 475)],
                )],
            ),
        ],
    )
    .into_par_sorted(order_tree)
}

test_case! {
    typical_bottom_up_binary where
        tree = typical_tree::<Bytes>(4096.into(), 1),
        max_depth = 10,
        max_width = 150,
        direction = BottomUp,
        measurement_system = Binary,
        expected = text_block_fnl! {
            " 52B   ┌──bar                                   │                                                                                          │  0%"
            "  2K   ├──foo                                   │                                                                                  ████████│  9%"
            "  4K   ├──empty dir                             │                                                                             █████████████│ 15%"
            " 45B   │   ┌──hello                             │                                                               ░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒│  0%"
            " 54B   │   ├──world                             │                                                               ░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒│  0%"
            "  4K   │ ┌─┴world                               │                                                               ░░░░░░░░░░░░░██████████████│ 15%"
            "  8K   ├─┴hello                                 │                                                               ███████████████████████████│ 30%"
            "475B   │   ┌──file with a really long name      │                                                              ░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒██│  2%"
            "  4K   │ ┌─┴subdirectory with a really long name│                                                              ░░░░░░░░░░░░░███████████████│ 16%"
            "  8K   ├─┴directory with a really long name     │                                                              ████████████████████████████│ 31%"
            " 27K ┌─┴root                                    │██████████████████████████████████████████████████████████████████████████████████████████│100%"
        },
}

test_case! {
    typical_bottom_up_binary_short_max_width where
        tree = typical_tree::<Bytes>(4096.into(), 1),
        max_depth = 10,
        max_width = 90,
        direction = BottomUp,
        measurement_system = Binary,
        expected = text_block_fnl! {
            " 52B   ┌──bar                │                                                 │  0%"
            "  2K   ├──foo                │                                             ████│  9%"
            "  4K   ├──empty dir          │                                          ███████│ 15%"
            " 45B   │   ┌──hello          │                                  ░░░░░░░░▒▒▒▒▒▒▒│  0%"
            " 54B   │   ├──world          │                                  ░░░░░░░░▒▒▒▒▒▒▒│  0%"
            "  4K   │ ┌─┴world            │                                  ░░░░░░░░███████│ 15%"
            "  8K   ├─┴hello              │                                  ███████████████│ 30%"
            "475B   │   ┌──file with a ...│                                  ░░░░░░░▒▒▒▒▒▒▒█│  2%"
            "  4K   │ ┌─┴subdirectory w...│                                  ░░░░░░░████████│ 16%"
            "  8K   ├─┴directory with a...│                                  ███████████████│ 31%"
            " 27K ┌─┴root                 │█████████████████████████████████████████████████│100%"
        },
}

test_case! {
    typical_bottom_up_binary_tebi_scale where
        tree = typical_tree::<Bytes>(4096.into(), 1 << 40),
        max_depth = 10,
        max_width = 150,
        direction = BottomUp,
        measurement_system = Binary,
        expected = text_block_fnl! {
            "  4K   ┌──empty dir                             │                                                                                          │  0%"
            " 52T   ├──bar                                   │                                                                                         █│  2%"
            " 45T   │   ┌──hello                             │                                                                                       ▒▒█│  1%"
            " 54T   │   ├──world                             │                                                                                       ▒██│  2%"
            " 99T   │ ┌─┴world                               │                                                                                       ███│  3%"
            " 99T   ├─┴hello                                 │                                                                                       ███│  3%"
            "475T   │   ┌──file with a really long name      │                                                                            ██████████████│ 15%"
            "475T   │ ┌─┴subdirectory with a really long name│                                                                            ██████████████│ 15%"
            "475T   ├─┴directory with a really long name     │                                                                            ██████████████│ 15%"
            "  2P   ├──foo                                   │                  ████████████████████████████████████████████████████████████████████████│ 80%"
            "  3P ┌─┴root                                    │██████████████████████████████████████████████████████████████████████████████████████████│100%"
        },
}

test_case! {
    typical_bottom_up_metric_tebi_scale where
        tree = typical_tree::<Bytes>(4096.into(), 1 << 40),
        max_depth = 10,
        max_width = 150,
        direction = BottomUp,
        measurement_system = Metric,
        expected = text_block_fnl! {
            "  4K   ┌──empty dir                             │                                                                                          │  0%"
            " 57T   ├──bar                                   │                                                                                         █│  2%"
            " 49T   │   ┌──hello                             │                                                                                       ▒▒█│  1%"
            " 59T   │   ├──world                             │                                                                                       ▒██│  2%"
            "109T   │ ┌─┴world                               │                                                                                       ███│  3%"
            "109T   ├─┴hello                                 │                                                                                       ███│  3%"
            "522T   │   ┌──file with a really long name      │                                                                            ██████████████│ 15%"
            "522T   │ ┌─┴subdirectory with a really long name│                                                                            ██████████████│ 15%"
            "522T   ├─┴directory with a really long name     │                                                                            ██████████████│ 15%"
            "  3P   ├──foo                                   │                  ████████████████████████████████████████████████████████████████████████│ 80%"
            "  3P ┌─┴root                                    │██████████████████████████████████████████████████████████████████████████████████████████│100%"
        },
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

test_case! {
    nested_bottom_up_binary_blocks where
        tree = nested_tree::<Blocks>(
            &["a", "b", "c", "d", "e", "f"],
            8.into(),
            "z",
            2.into(),
        ),
        max_depth = 10,
        max_width = 150,
        direction = BottomUp,
        measurement_system = Binary,
        expected = text_block_fnl! {
            " 2             ┌──z│                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓█████│  4%"
            "10           ┌─┴f  │                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓████████████████████████│ 20%"
            "18         ┌─┴e    │                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓███████████████████████████████████████████│ 36%"
            "26       ┌─┴d      │                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒██████████████████████████████████████████████████████████████│ 52%"
            "34     ┌─┴c        │                   ░░░░░░░░░░░░░░░░░░░█████████████████████████████████████████████████████████████████████████████████│ 68%"
            "42   ┌─┴b          │                   ████████████████████████████████████████████████████████████████████████████████████████████████████│ 84%"
            "50 ┌─┴a            │███████████████████████████████████████████████████████████████████████████████████████████████████████████████████████│100%"
        },
}

test_case! {
    nested_bottom_up_binary_long_names_short_max_width where
        tree = nested_tree::<Bytes>(
            &[
                "directory with a long name",
                "child directory with a long name",
                "grandchild directory with a long name",
                "great-grandchild directory with a long name",
                "great-great-grandchild directory with a long name",
            ],
            4096.into(),
            "file with a long name",
            1024.into(),
        ),
        max_depth = 10,
        max_width = 100,
        direction = BottomUp,
        measurement_system = Binary,
        expected = text_block_fnl! {
            " 1K           ┌──file with a...│           ░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓███│  5%"
            " 5K         ┌─┴great-great-g...│           ░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓██████████████│ 24%"
            " 9K       ┌─┴great-grandchil...│           ░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒████████████████████████│ 43%"
            "13K     ┌─┴grandchild direct...│           ░░░░░░░░░░░███████████████████████████████████│ 62%"
            "17K   ┌─┴child directory wit...│           ██████████████████████████████████████████████│ 81%"
            "21K ┌─┴directory with a long...│█████████████████████████████████████████████████████████│100%"
        },
}

test_case! {
    nested_bottom_up_binary_tebi_scale where
        tree = nested_tree::<Bytes>(
            &["a", "b", "c", "d", "e", "f"],
            (4 << 40).into(),
            "z",
            (1 << 40).into(),
        ),
        max_depth = 10,
        max_width = 150,
        direction = BottomUp,
        measurement_system = Binary,
        expected = text_block_fnl! {
            " 1T             ┌──z│                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓█████│  4%"
            " 5T           ┌─┴f  │                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓████████████████████████│ 20%"
            " 9T         ┌─┴e    │                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓██████████████████████████████████████████│ 36%"
            "13T       ┌─┴d      │                   ░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒█████████████████████████████████████████████████████████████│ 52%"
            "17T     ┌─┴c        │                   ░░░░░░░░░░░░░░░░░░░████████████████████████████████████████████████████████████████████████████████│ 68%"
            "21T   ┌─┴b          │                   ███████████████████████████████████████████████████████████████████████████████████████████████████│ 84%"
            "25T ┌─┴a            │██████████████████████████████████████████████████████████████████████████████████████████████████████████████████████│100%"
        },
}
