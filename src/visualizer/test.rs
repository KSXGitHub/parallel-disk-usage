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
        $(#[$attributes:meta])*
        $name:ident where
        tree = $tree:expr,
        max_depth = $max_depth:expr,
        max_width = $max_width:expr,
        direction = $direction:ident,
        measurement_system = $measurement_system:ident,
        expected = $expected:expr,
    ) => {
        $(#[$attributes])*
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
    typical_bottom_up_metric where
        tree = typical_tree::<Bytes>(4096.into(), 1),
        max_depth = 10,
        max_width = 150,
        direction = BottomUp,
        measurement_system = Metric,
        expected = text_block_fnl! {
            " 52B   ┌──bar                                   │                                                                                          │  0%"
            "  3K   ├──foo                                   │                                                                                  ████████│  9%"
            "  4K   ├──empty dir                             │                                                                             █████████████│ 15%"
            " 45B   │   ┌──hello                             │                                                               ░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒│  0%"
            " 54B   │   ├──world                             │                                                               ░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒│  0%"
            "  4K   │ ┌─┴world                               │                                                               ░░░░░░░░░░░░░██████████████│ 15%"
            "  8K   ├─┴hello                                 │                                                               ███████████████████████████│ 30%"
            "475B   │   ┌──file with a really long name      │                                                              ░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒██│  2%"
            "  5K   │ ┌─┴subdirectory with a really long name│                                                              ░░░░░░░░░░░░░███████████████│ 16%"
            "  9K   ├─┴directory with a really long name     │                                                              ████████████████████████████│ 31%"
            " 28K ┌─┴root                                    │██████████████████████████████████████████████████████████████████████████████████████████│100%"
        },
}

test_case! {
    typical_top_down_binary where
        tree = typical_tree::<Bytes>(4096.into(), 1),
        max_depth = 10,
        max_width = 150,
        direction = TopDown,
        measurement_system = Binary,
        expected = text_block_fnl! {
            " 27K └─┬root                                    │██████████████████████████████████████████████████████████████████████████████████████████│100%"
            "  8K   ├─┬directory with a really long name     │                                                              ████████████████████████████│ 31%"
            "  4K   │ └─┬subdirectory with a really long name│                                                              ░░░░░░░░░░░░░███████████████│ 16%"
            "475B   │   └──file with a really long name      │                                                              ░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒██│  2%"
            "  8K   ├─┬hello                                 │                                                               ███████████████████████████│ 30%"
            "  4K   │ └─┬world                               │                                                               ░░░░░░░░░░░░░██████████████│ 15%"
            " 54B   │   ├──world                             │                                                               ░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒│  0%"
            " 45B   │   └──hello                             │                                                               ░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒│  0%"
            "  4K   ├──empty dir                             │                                                                             █████████████│ 15%"
            "  2K   ├──foo                                   │                                                                                  ████████│  9%"
            " 52B   └──bar                                   │                                                                                          │  0%"
        },
}

test_case! {
    typical_short_max_width where
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
    typical_even_shorter_max_width where
        tree = typical_tree::<Bytes>(4096.into(), 1),
        max_depth = 10,
        max_width = 50,
        direction = BottomUp,
        measurement_system = Binary,
        expected = text_block_fnl! {
            " 52B   ┌──bar  │                       │  0%"
            "  2K   ├──foo  │                     ██│  9%"
            "  4K   ├──em...│                    ███│ 15%"
            "  8K   ├──hello│                ███████│ 30%"
            "  8K   ├──di...│                ███████│ 31%"
            " 27K ┌─┴root   │███████████████████████│100%"
        },
}

test_case! {
    typical_binary_tebi_scale where
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
    typical_metric_tebi_scale where
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
    nested_bottom_up_blocks where
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
    nested_bottom_up_binary_many_names_short_max_width where
        tree = nested_tree::<Bytes>(
            &[
                "a",
                "ab",
                "abc",
                "abcd",
                "abcde",
                "abcdef",
                "abcdefg",
                "abcdefgh",
                "abcdefghi",
                "abcdefghij",
                "abcdefghijk",
                "abcdefghijkl",
            ],
            4096.into(),
            "xyz",
            1024.into(),
        ),
        max_depth = 10,
        max_width = 90,
        direction = BottomUp,
        measurement_system = Binary,
        expected = text_block_fnl! {
            "17K                 ┌──ab...│    ░░░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓█████████████████│ 35%"
            "21K               ┌─┴abcd...│    ░░░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓█████████████████████│ 43%"
            "25K             ┌─┴abcdefg  │    ░░░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓██████████████████████████│ 51%"
            "29K           ┌─┴abcdef     │    ░░░░▒▒▒▒▓▓▓▓▓▓▓▓██████████████████████████████│ 59%"
            "33K         ┌─┴abcde        │    ░░░░▒▒▒▒▓▓▓▓██████████████████████████████████│ 67%"
            "37K       ┌─┴abcd           │    ░░░░▒▒▒▒██████████████████████████████████████│ 76%"
            "41K     ┌─┴abc              │    ░░░░██████████████████████████████████████████│ 84%"
            "45K   ┌─┴ab                 │    ██████████████████████████████████████████████│ 92%"
            "49K ┌─┴a                    │██████████████████████████████████████████████████│100%"
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

fn long_and_short_names<Data>() -> Tree<&'static str, Data>
where
    Data: Size + Ord + From<u64> + Send,
{
    let dir = Tree::<&'static str, Data>::fixed_size_dir_constructor(1.into());
    let file = |name: &'static str, size: u64| Tree::file(name, Data::from(size));
    dir(
        "root",
        vec![
            file("a", 1),
            file("file with a long name 1", 2),
            file("b", 3),
            file("file with a long name 2", 4),
            dir(
                "c",
                vec![
                    file("a", 1),
                    file("file with a long name 1", 2),
                    file("b", 3),
                    file("file with a long name 2", 4),
                    file("weight", 5),
                ],
            ),
            dir(
                "directory with a long name 1",
                vec![
                    file("a", 1),
                    file("file with a long name 1", 2),
                    file("b", 3),
                    file("file with a long name 2", 4),
                    file("weight", 6),
                ],
            ),
            dir(
                "d",
                vec![
                    file("a", 1),
                    file("file with a long name 1", 2),
                    file("b", 3),
                    file("file with a long name 2", 4),
                    file("weight", 7),
                ],
            ),
            dir(
                "directory with a long name 2",
                vec![
                    file("a", 1),
                    file("file with a long name 1", 2),
                    file("b", 3),
                    file("file with a long name 2", 4),
                    file("weight", 8),
                ],
            ),
        ],
    )
    .into_par_sorted(order_tree)
}

test_case! {
    long_and_short_names_sufficient_max_width where
        tree = long_and_short_names::<Blocks>(),
        max_depth = 10,
        max_width = 150,
        direction = BottomUp,
        measurement_system = Binary,
        expected = text_block_fnl! {
            " 1   ┌──a                           │                                                                                                     █│  1%"
            " 2   ├──file with a long name 1     │                                                                                                   ███│  2%"
            " 3   ├──b                           │                                                                                                  ████│  4%"
            " 4   ├──file with a long name 2     │                                                                                                 █████│  5%"
            " 1   │ ┌──a                         │                                                                                  ░░░░░░░░░░░░░░░░░░░█│  1%"
            " 2   │ ├──file with a long name 1   │                                                                                  ░░░░░░░░░░░░░░░░░███│  2%"
            " 3   │ ├──b                         │                                                                                  ░░░░░░░░░░░░░░░░████│  4%"
            " 4   │ ├──file with a long name 2   │                                                                                  ░░░░░░░░░░░░░░░█████│  5%"
            " 5   │ ├──weight                    │                                                                                  ░░░░░░░░░░░░░░██████│  6%"
            "16   ├─┴c                           │                                                                                  ████████████████████│ 20%"
            " 1   │ ┌──a                         │                                                                                 ░░░░░░░░░░░░░░░░░░░░█│  1%"
            " 2   │ ├──file with a long name 1   │                                                                                 ░░░░░░░░░░░░░░░░░░███│  2%"
            " 3   │ ├──b                         │                                                                                 ░░░░░░░░░░░░░░░░░████│  4%"
            " 4   │ ├──file with a long name 2   │                                                                                 ░░░░░░░░░░░░░░░░█████│  5%"
            " 6   │ ├──weight                    │                                                                                 ░░░░░░░░░░░░░████████│  7%"
            "17   ├─┴directory with a long name 1│                                                                                 █████████████████████│ 21%"
            " 1   │ ┌──a                         │                                                                               ░░░░░░░░░░░░░░░░░░░░░░█│  1%"
            " 2   │ ├──file with a long name 1   │                                                                               ░░░░░░░░░░░░░░░░░░░░███│  2%"
            " 3   │ ├──b                         │                                                                               ░░░░░░░░░░░░░░░░░░░████│  4%"
            " 4   │ ├──file with a long name 2   │                                                                               ░░░░░░░░░░░░░░░░░░█████│  5%"
            " 7   │ ├──weight                    │                                                                               ░░░░░░░░░░░░░░█████████│  9%"
            "18   ├─┴d                           │                                                                               ███████████████████████│ 22%"
            " 1   │ ┌──a                         │                                                                              ░░░░░░░░░░░░░░░░░░░░░░░█│  1%"
            " 2   │ ├──file with a long name 1   │                                                                              ░░░░░░░░░░░░░░░░░░░░░███│  2%"
            " 3   │ ├──b                         │                                                                              ░░░░░░░░░░░░░░░░░░░░████│  4%"
            " 4   │ ├──file with a long name 2   │                                                                              ░░░░░░░░░░░░░░░░░░░█████│  5%"
            " 8   │ ├──weight                    │                                                                              ░░░░░░░░░░░░░░██████████│ 10%"
            "19   ├─┴directory with a long name 2│                                                                              ████████████████████████│ 23%"
            "81 ┌─┴root                          │██████████████████████████████████████████████████████████████████████████████████████████████████████│100%"
        },
}

test_case! {
    remaining_siblings_properly_connect_when_some_amongst_them_disappear where
        tree = long_and_short_names::<Blocks>(),
        max_depth = 10,
        max_width = 50,
        direction = BottomUp,
        measurement_system = Binary,
        expected = text_block_fnl! {
            " 1   ┌──a    │                         │  1%"
            " 2   ├──fi...│                        █│  2%"
            " 3   ├──b    │                        █│  4%"
            " 4   ├──fi...│                        █│  5%"
            " 1   │ ┌──a  │                    ░░░░░│  1%"
            " 3   │ ├──b  │                    ░░░░█│  4%"
            "16   ├─┴c    │                    █████│ 20%"
            " 1   │ ┌──a  │                    ░░░░░│  1%"
            " 3   │ ├──b  │                    ░░░░█│  4%"
            "17   ├─┴di...│                    █████│ 21%"
            " 1   │ ┌──a  │                   ░░░░░░│  1%"
            " 3   │ ├──b  │                   ░░░░░█│  4%"
            "18   ├─┴d    │                   ██████│ 22%"
            " 1   │ ┌──a  │                   ░░░░░░│  1%"
            " 3   │ ├──b  │                   ░░░░░█│  4%"
            "19   ├─┴di...│                   ██████│ 23%"
            "81 ┌─┴root   │█████████████████████████│100%"
        },
}

test_case! {
    children_of_disappeared_nodes_also_disappear where
        tree = long_and_short_names::<Blocks>(),
        max_depth = 10,
        max_width = 44,
        direction = BottomUp,
        measurement_system = Binary,
        expected = text_block_fnl! {
            " 1   ┌──a  │                     │  1%"
            " 3   ├──b  │                    █│  4%"
            " 1   │ ┌──a│                 ░░░░│  1%"
            " 3   │ ├──b│                 ░░░█│  4%"
            "16   ├─┴c  │                 ████│ 20%"
            " 1   │ ┌──a│                ░░░░░│  1%"
            " 3   │ ├──b│                ░░░░█│  4%"
            "18   ├─┴d  │                █████│ 22%"
            "81 ┌─┴root │█████████████████████│100%"
        },
}

test_case! {
    remaining_siblings_properly_connect_when_some_amongst_them_disappear_top_down where
        tree = long_and_short_names::<Blocks>(),
        max_depth = 10,
        max_width = 50,
        direction = TopDown,
        measurement_system = Binary,
        expected = text_block_fnl! {
            "81 └─┬root   │█████████████████████████│100%"
            "19   ├─┬di...│                   ██████│ 23%"
            " 3   │ ├──b  │                   ░░░░░█│  4%"
            " 1   │ └──a  │                   ░░░░░░│  1%"
            "18   ├─┬d    │                   ██████│ 22%"
            " 3   │ ├──b  │                   ░░░░░█│  4%"
            " 1   │ └──a  │                   ░░░░░░│  1%"
            "17   ├─┬di...│                    █████│ 21%"
            " 3   │ ├──b  │                    ░░░░█│  4%"
            " 1   │ └──a  │                    ░░░░░│  1%"
            "16   ├─┬c    │                    █████│ 20%"
            " 3   │ ├──b  │                    ░░░░█│  4%"
            " 1   │ └──a  │                    ░░░░░│  1%"
            " 4   ├──fi...│                        █│  5%"
            " 3   ├──b    │                        █│  4%"
            " 2   ├──fi...│                        █│  2%"
            " 1   └──a    │                         │  1%"
        },
}

test_case! {
    children_of_disappeared_nodes_also_disappear_top_down where
        tree = long_and_short_names::<Blocks>(),
        max_depth = 10,
        max_width = 44,
        direction = TopDown,
        measurement_system = Binary,
        expected = text_block_fnl! {
            "81 └─┬root │█████████████████████│100%"
            "18   ├─┬d  │                █████│ 22%"
            " 3   │ ├──b│                ░░░░█│  4%"
            " 1   │ └──a│                ░░░░░│  1%"
            "16   ├─┬c  │                 ████│ 20%"
            " 3   │ ├──b│                 ░░░█│  4%"
            " 1   │ └──a│                 ░░░░│  1%"
            " 3   ├──b  │                    █│  4%"
            " 1   └──a  │                     │  1%"
        },
}

fn big_tree_with_long_names<Data>() -> Tree<&'static str, Data>
where
    Data: Size + Ord + From<u64> + Send,
{
    let dir = Tree::<&'static str, Data>::fixed_size_dir_constructor(4069.into());
    let file = |name: &'static str, size: u64| Tree::file(name, Data::from(size));
    let mut short_file_names = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r",
        "s", "t", "u", "v", "w", "x", "y", "z", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J",
        "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
    ]
    .iter();
    let mut short_file = || {
        let name = short_file_names.next().expect("access short file name");
        file(name, 750)
    };
    dir(
        "root",
        vec![
            dir(
                "sub 1",
                vec![
                    file("first file with a long name", 999),
                    dir(
                        "sub 1.1",
                        vec![
                            file("second file with a long name", 7766),
                            short_file(),
                            short_file(),
                            dir(
                                "first sub directory with a long name",
                                vec![file("abc", 123), file("def", 456), short_file()],
                            ),
                            dir(
                                "second sub directory with a long name",
                                vec![
                                    file("abcdefghi", 1234),
                                    file("ihgfedbca", 4321),
                                    file("abc abc abc", 1212),
                                    file("third file with a long name", 4545),
                                    short_file(),
                                    short_file(),
                                    short_file(),
                                ],
                            ),
                        ],
                    ),
                    dir(
                        "sub 1.2",
                        vec![
                            short_file(),
                            dir(
                                "third sub directory with a long name",
                                vec![dir(
                                    "forth sub directory with a long name",
                                    vec![file("forth file with a long name", 3647), short_file()],
                                )],
                            ),
                            dir(
                                "fifth sub directory with a long name",
                                vec![
                                    short_file(),
                                    short_file(),
                                    dir(
                                        "sixth sub directory with a long name",
                                        vec![
                                            file("fifth file with a long name", 364),
                                            short_file(),
                                            short_file(),
                                            short_file(),
                                        ],
                                    ),
                                ],
                            ),
                            dir(
                                "sixth sub directory with a long name",
                                vec![
                                    short_file(),
                                    dir(
                                        "seventh sub directory with a long name",
                                        vec![
                                            file("sixth file with a long name", 6565),
                                            file("seventh file with a long name", 555),
                                            short_file(),
                                        ],
                                    ),
                                    dir(
                                        "eighth sub directory with a long name",
                                        vec![
                                            file("eighth file with a long name", 444),
                                            short_file(),
                                            short_file(),
                                            short_file(),
                                        ],
                                    ),
                                    file("ninth file with a long name", 777),
                                ],
                            ),
                        ],
                    ),
                ],
            ),
            dir(
                "sub 2",
                vec![
                    dir(
                        "sub 2.1",
                        vec![
                            short_file(),
                            dir(
                                "ninth sub directory with a long name",
                                vec![
                                    file("tenth file with a long name", 88888),
                                    short_file(),
                                    short_file(),
                                    dir(
                                        "tenth sub directory with a long name",
                                        vec![
                                            file("eleventh file with a long name", 44444),
                                            short_file(),
                                            short_file(),
                                            short_file(),
                                        ],
                                    ),
                                ],
                            ),
                        ],
                    ),
                    dir(
                        "sub 2.2",
                        vec![
                            short_file(),
                            dir(
                                "eleventh sub directory with a long name",
                                vec![
                                    file("twelfth file with a long name", 453),
                                    file("thirteenth file with a long name", 352),
                                    short_file(),
                                    dir(
                                        "twelfth sub directory with a long name",
                                        vec![
                                            file("fourteenth file with a long name", 128),
                                            short_file(),
                                        ],
                                    ),
                                    dir(
                                        "thirteenth sub directory with a long name",
                                        vec![
                                            file("fifteenth file with a long name", 128),
                                            file("sixteenth file with a long name", 256),
                                            file("seventeenth file with a long name", 512),
                                            short_file(),
                                        ],
                                    ),
                                ],
                            ),
                            dir(
                                "fourteenth sub directory with a long name",
                                vec![
                                    file("eighteenth file with a long name", 542),
                                    file("eighty-first file with a long name", 357),
                                    short_file(),
                                    short_file(),
                                    short_file(),
                                    dir(
                                        "twelfth sub directory with a long name",
                                        vec![
                                            file("eighty-second file with a long name", 222),
                                            short_file(),
                                            short_file(),
                                        ],
                                    ),
                                    dir(
                                        "fifteenth sub directory with a long name",
                                        vec![
                                            file("eighty-third file with a long name", 333),
                                            file("eighty-fourth file with a long name", 344),
                                            file("eighty-seventh file with a long name", 444),
                                            short_file(),
                                        ],
                                    ),
                                ],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    )
    .into_par_sorted(order_tree)
}

test_case! {
    big_tree_with_long_names_short_max_width where
        tree = big_tree_with_long_names::<Bytes>(),
        max_depth = 100,
        max_width = 67,
        direction = BottomUp,
        measurement_system = Binary,
        expected = text_block_fnl! {
            "999B     ┌──first ...│                       ░░░░░░░░░░░│  0%"
            "750B     │ ┌──b      │                       ░░░░░░░▒▒▒▒│  0%"
            "750B     │ ├──a      │                       ░░░░░░░▒▒▒▒│  0%"
            "123B     │ │ ┌──abc  │                       ░░░░░░░▒▒▒▓│  0%"
            "456B     │ │ ├──def  │                       ░░░░░░░▒▒▒▓│  0%"
            "750B     │ │ ├──c    │                       ░░░░░░░▒▒▒▓│  0%"
            "  5K     │ ├─┴firs...│                       ░░░░░░░▒▒▒█│  2%"
            "  8K     │ ├──seco...│                       ░░░░░░░▒▒▒█│  3%"
            "750B     │ │ ┌──f    │                       ░░░░░░░▒▒▓▓│  0%"
            "750B     │ │ ├──e    │                       ░░░░░░░▒▒▓▓│  0%"
            "750B     │ │ ├──d    │                       ░░░░░░░▒▒▓▓│  0%"
            "  1K     │ │ ├──ab...│                       ░░░░░░░▒▒▓▓│  0%"
            "  1K     │ │ ├──ab...│                       ░░░░░░░▒▒▓▓│  0%"
            "  4K     │ │ ├──ih...│                       ░░░░░░░▒▒▓█│  1%"
            "  4K     │ │ ├──th...│                       ░░░░░░░▒▒▓█│  2%"
            " 17K     │ ├─┴seco...│                       ░░░░░░░▒▒██│  6%"
            " 36K     ├─┴sub 1.1  │                       ░░░░░░░████│ 12%"
            "750B     │ ┌──g      │                       ░░░░░▒▒▒▒▒▒│  0%"
            "750B     │ │ ┌──j    │                       ░░░░░▒▒▒▒▒▓│  0%"
            "750B     │ │ ├──i    │                       ░░░░░▒▒▒▒▒▓│  0%"
            "750B     │ │ │ ┌──m  │                       ░░░░░▒▒▒▒▒▓│  0%"
            "750B     │ │ │ ├──l  │                       ░░░░░▒▒▒▒▒▓│  0%"
            "750B     │ │ │ ├──k  │                       ░░░░░▒▒▒▒▒▓│  0%"
            "  7K     │ │ ├─┴si...│                       ░░░░░▒▒▒▒▒█│  2%"
            " 12K     │ ├─┴fift...│                       ░░░░░▒▒▒▒▒█│  4%"
            "750B     │ │   ┌──h  │                       ░░░░░▒▒▒▒▒▓│  0%"
            "  8K     │ │ ┌─┴fo...│                       ░░░░░▒▒▒▒▒█│  3%"
            " 12K     │ ├─┴thir...│                       ░░░░░▒▒▒▒▒█│  4%"
            "750B     │ │ ┌──n    │                       ░░░░░▒▒▒▓▓▓│  0%"
            "777B     │ │ ├──ni...│                       ░░░░░▒▒▒▓▓▓│  0%"
            "750B     │ │ │ ┌──r  │                       ░░░░░▒▒▒▓▓▓│  0%"
            "750B     │ │ │ ├──q  │                       ░░░░░▒▒▒▓▓▓│  0%"
            "750B     │ │ │ ├──p  │                       ░░░░░▒▒▒▓▓▓│  0%"
            "  7K     │ │ ├─┴ei...│                       ░░░░░▒▒▒▓▓█│  2%"
            "750B     │ │ │ ┌──o  │                       ░░░░░▒▒▒▓▓▓│  0%"
            " 12K     │ │ ├─┴se...│                       ░░░░░▒▒▒▓▓█│  4%"
            " 24K     │ ├─┴sixt...│                       ░░░░░▒▒▒███│  8%"
            " 53K     ├─┴sub 1.2  │                       ░░░░░██████│ 18%"
            " 93K   ┌─┴sub 1      │                       ███████████│ 32%"
            "750B   │   ┌──y      │            ░░░░░░░░░░░░░░░░░▒▒▒▒▒│  0%"
            "352B   │   │ ┌──th...│            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "453B   │   │ ├──tw...│            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "750B   │   │ ├──z    │            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "750B   │   │ │ ┌──A  │            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "  5K   │   │ ├─┴tw...│            ░░░░░░░░░░░░░░░░░▒▒▒▓█│  2%"
            "750B   │   │ │ ┌──B  │            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "  6K   │   │ ├─┴th...│            ░░░░░░░░░░░░░░░░░▒▒▒▓█│  2%"
            " 16K   │   ├─┴elev...│            ░░░░░░░░░░░░░░░░░▒▒▒██│  6%"
            "357B   │   │ ┌──ei...│            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "542B   │   │ ├──ei...│            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "750B   │   │ ├──E    │            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "750B   │   │ ├──D    │            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "750B   │   │ ├──C    │            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "750B   │   │ │ ┌──G  │            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "750B   │   │ │ ├──F  │            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "  6K   │   │ ├─┴tw...│            ░░░░░░░░░░░░░░░░░▒▒▒▓█│  2%"
            "750B   │   │ │ ┌──H  │            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "  6K   │   │ ├─┴fi...│            ░░░░░░░░░░░░░░░░░▒▒▒▓█│  2%"
            " 19K   │   ├─┴four...│            ░░░░░░░░░░░░░░░░░▒▒▒██│  6%"
            " 39K   │ ┌─┴sub 2.2  │            ░░░░░░░░░░░░░░░░░█████│ 14%"
            "750B   │ │ ┌──s      │            ░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒│  0%"
            "750B   │ │ │ ┌──u    │            ░░░░░▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│  0%"
            "750B   │ │ │ ├──t    │            ░░░░░▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│  0%"
            "750B   │ │ │ │ ┌──x  │            ░░░░░▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│  0%"
            "750B   │ │ │ │ ├──w  │            ░░░░░▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│  0%"
            "750B   │ │ │ │ ├──v  │            ░░░░░▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│  0%"
            " 50K   │ │ │ ├─┴te...│            ░░░░░▓▓▓▓▓▓▓▓▓▓▓██████│ 17%"
            " 87K   │ │ │ ├──te...│            ░░░░░▓▓▓▓▓▓▓██████████│ 30%"
            "142K   │ │ ├─┴nint...│            ░░░░░█████████████████│ 49%"
            "147K   │ ├─┴sub 2.1  │            ░░░░░█████████████████│ 51%"
            "190K   ├─┴sub 2      │            ██████████████████████│ 66%"
            "287K ┌─┴root         │██████████████████████████████████│100%"
        },
}
