use parallel_disk_usage::{
    bytes_format::BytesFormat::*,
    data_tree::DataTree,
    size::{self, Blocks, Bytes},
    visualizer::{BarAlignment, ColumnWidthDistribution, Direction, Visualizer},
};
use pretty_assertions::assert_eq;
use std::cmp::Ordering;
use text_block_macros::text_block_fnl;
use zero_copy_pads::Width;

fn order_tree<Name, Size: size::Size>(
    left: &DataTree<Name, Size>,
    right: &DataTree<Name, Size>,
) -> Ordering {
    left.size().cmp(&right.size()).reverse()
}

macro_rules! test_case {
    (
        $(#[$attributes:meta])*
        $name:ident where
        tree = $tree:expr,
        bytes_format = $bytes_format:expr,
        max_depth = $max_depth:expr,
        column_width_distribution = $column_width_function:ident $($column_width_arguments:literal)+,
        direction = $direction:ident,
        bar_alignment = $bar_alignment:ident,
        expected = $expected:expr,
    ) => {
        $(#[$attributes])*
        #[test]
        fn $name() {
            let mut tree = $tree;
            let column_width_distribution =
                ColumnWidthDistribution::$column_width_function($($column_width_arguments),+);
            tree.par_retain(|_, depth| depth + 1 < $max_depth);
            let actual = Visualizer::builder()
                .column_width_distribution(column_width_distribution)
                .data_tree(&tree)
                .bytes_format($bytes_format)
                .direction(Direction::$direction)
                .bar_alignment(BarAlignment::$bar_alignment)
                .build()
            .to_string();
            let expected = $expected;
            eprintln!("\nACTUAL:\n{actual}\n");

            let actual_lines: Vec<_> = actual.lines().collect();
            let expected_lines: Vec<_> = expected.lines().collect();
            assert_eq!(actual_lines, expected_lines);

            if let ColumnWidthDistribution::Total { width } = column_width_distribution {
                let actual_line_widths: Vec<_> = actual
                    .lines()
                    .map(|line| line.width())
                    .collect();
                let expected_line_widths: Vec<_> = expected
                    .lines()
                    .map(|_| width)
                    .collect();
                assert_eq!(actual_line_widths, expected_line_widths);
            }
        }
    };
}

fn typical_tree<Size>(size_per_dir: Size, file_size_factor: u64) -> DataTree<&'static str, Size>
where
    Size: size::Size + Ord + From<u64> + Send,
{
    let dir = DataTree::<&'static str, Size>::fixed_size_dir_constructor(size_per_dir);
    let file =
        |name: &'static str, size: u64| DataTree::file(name, Size::from(size * file_size_factor));
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
        bytes_format = BinaryUnits,
        max_depth = 10,
        column_width_distribution = total 90,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            " 52      ┌──bar                                   │                                  │  0%"
            "  2.5K   ├──foo                                   │                               ███│  9%"
            "  4.0K   ├──empty dir                             │                             █████│ 15%"
            " 45      │   ┌──hello                             │                        ░░░░░▒▒▒▒▒│  0%"
            " 54      │   ├──world                             │                        ░░░░░▒▒▒▒▒│  0%"
            "  4.1K   │ ┌─┴world                               │                        ░░░░░█████│ 15%"
            "  8.1K   ├─┴hello                                 │                        ██████████│ 30%"
            "475      │   ┌──file with a really long name      │                       ░░░░░▒▒▒▒▒█│  2%"
            "  4.5K   │ ┌─┴subdirectory with a really long name│                       ░░░░░██████│ 16%"
            "  8.5K   ├─┴directory with a really long name     │                       ███████████│ 31%"
            " 27.1K ┌─┴root                                    │██████████████████████████████████│100%"
        },
}

test_case! {
    typical_bottom_up_metric where
        tree = typical_tree::<Bytes>(4096.into(), 1),
        bytes_format = MetricUnits,
        max_depth = 10,
        column_width_distribution = total 90,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            " 52      ┌──bar                                   │                                  │  0%"
            "  2.5K   ├──foo                                   │                               ███│  9%"
            "  4.1K   ├──empty dir                             │                             █████│ 15%"
            " 45      │   ┌──hello                             │                        ░░░░░▒▒▒▒▒│  0%"
            " 54      │   ├──world                             │                        ░░░░░▒▒▒▒▒│  0%"
            "  4.2K   │ ┌─┴world                               │                        ░░░░░█████│ 15%"
            "  8.3K   ├─┴hello                                 │                        ██████████│ 30%"
            "475      │   ┌──file with a really long name      │                       ░░░░░▒▒▒▒▒█│  2%"
            "  4.6K   │ ┌─┴subdirectory with a really long name│                       ░░░░░██████│ 16%"
            "  8.7K   ├─┴directory with a really long name     │                       ███████████│ 31%"
            " 27.7K ┌─┴root                                    │██████████████████████████████████│100%"
        },
}

test_case! {
    typical_top_down_binary where
        tree = typical_tree::<Bytes>(4096.into(), 1),
        bytes_format = BinaryUnits,
        max_depth = 10,
        column_width_distribution = total 90,
        direction = TopDown,
        bar_alignment = Right,
        expected = text_block_fnl! {
            " 27.1K └─┬root                                    │██████████████████████████████████│100%"
            "  8.5K   ├─┬directory with a really long name     │                       ███████████│ 31%"
            "  4.5K   │ └─┬subdirectory with a really long name│                       ░░░░░██████│ 16%"
            "475      │   └──file with a really long name      │                       ░░░░░▒▒▒▒▒█│  2%"
            "  8.1K   ├─┬hello                                 │                        ██████████│ 30%"
            "  4.1K   │ └─┬world                               │                        ░░░░░█████│ 15%"
            " 54      │   ├──world                             │                        ░░░░░▒▒▒▒▒│  0%"
            " 45      │   └──hello                             │                        ░░░░░▒▒▒▒▒│  0%"
            "  4.0K   ├──empty dir                             │                             █████│ 15%"
            "  2.5K   ├──foo                                   │                               ███│  9%"
            " 52      └──bar                                   │                                  │  0%"
        },
}

test_case! {
    typical_bottom_up_binary_align_left where
        tree = typical_tree::<Bytes>(4096.into(), 1),
        bytes_format = BinaryUnits,
        max_depth = 10,
        column_width_distribution = total 90,
        direction = BottomUp,
        bar_alignment = Left,
        expected = text_block_fnl! {
            " 52      ┌──bar                                   │                                  │  0%"
            "  2.5K   ├──foo                                   │███                               │  9%"
            "  4.0K   ├──empty dir                             │█████                             │ 15%"
            " 45      │   ┌──hello                             │▒▒▒▒▒░░░░░                        │  0%"
            " 54      │   ├──world                             │▒▒▒▒▒░░░░░                        │  0%"
            "  4.1K   │ ┌─┴world                               │█████░░░░░                        │ 15%"
            "  8.1K   ├─┴hello                                 │██████████                        │ 30%"
            "475      │   ┌──file with a really long name      │█▒▒▒▒▒░░░░░                       │  2%"
            "  4.5K   │ ┌─┴subdirectory with a really long name│██████░░░░░                       │ 16%"
            "  8.5K   ├─┴directory with a really long name     │███████████                       │ 31%"
            " 27.1K ┌─┴root                                    │██████████████████████████████████│100%"
        },
}

test_case! {
    typical_narrow_tree_column where
        tree = typical_tree::<Bytes>(4096.into(), 1),
        bytes_format = BinaryUnits,
        max_depth = 10,
        column_width_distribution = components 24 49,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            " 52      ┌──bar                │                                                 │  0%"
            "  2.5K   ├──foo                │                                             ████│  9%"
            "  4.0K   ├──empty dir          │                                          ███████│ 15%"
            " 45      │   ┌──hello          │                                  ░░░░░░░░▒▒▒▒▒▒▒│  0%"
            " 54      │   ├──world          │                                  ░░░░░░░░▒▒▒▒▒▒▒│  0%"
            "  4.1K   │ ┌─┴world            │                                  ░░░░░░░░███████│ 15%"
            "  8.1K   ├─┴hello              │                                  ███████████████│ 30%"
            "475      │   ┌──file with a ...│                                  ░░░░░░░▒▒▒▒▒▒▒█│  2%"
            "  4.5K   │ ┌─┴subdirectory w...│                                  ░░░░░░░████████│ 16%"
            "  8.5K   ├─┴directory with a...│                                  ███████████████│ 31%"
            " 27.1K ┌─┴root                 │█████████████████████████████████████████████████│100%"
        },
}

test_case! {
    typical_even_shorter_max_width where
        tree = typical_tree::<Bytes>(4096.into(), 1),
        bytes_format = BinaryUnits,
        max_depth = 10,
        column_width_distribution = components 10 23,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            " 52      ┌──bar  │                       │  0%"
            "  2.5K   ├──foo  │                     ██│  9%"
            "  4.0K   ├──em...│                    ███│ 15%"
            "  8.1K   ├──hello│                ███████│ 30%"
            "  8.5K   ├──di...│                ███████│ 31%"
            " 27.1K ┌─┴root   │███████████████████████│100%"
        },
}

test_case! {
    typical_sufficient_depth where
        tree = typical_tree::<Bytes>(4096.into(), 1),
        bytes_format = BinaryUnits,
        max_depth = 4,
        column_width_distribution = total 90,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            " 52      ┌──bar                                   │                                  │  0%"
            "  2.5K   ├──foo                                   │                               ███│  9%"
            "  4.0K   ├──empty dir                             │                             █████│ 15%"
            " 45      │   ┌──hello                             │                        ░░░░░▒▒▒▒▒│  0%"
            " 54      │   ├──world                             │                        ░░░░░▒▒▒▒▒│  0%"
            "  4.1K   │ ┌─┴world                               │                        ░░░░░█████│ 15%"
            "  8.1K   ├─┴hello                                 │                        ██████████│ 30%"
            "475      │   ┌──file with a really long name      │                       ░░░░░▒▒▒▒▒█│  2%"
            "  4.5K   │ ┌─┴subdirectory with a really long name│                       ░░░░░██████│ 16%"
            "  8.5K   ├─┴directory with a really long name     │                       ███████████│ 31%"
            " 27.1K ┌─┴root                                    │██████████████████████████████████│100%"
        },
}

test_case! {
    typical_shallow where
        tree = typical_tree::<Bytes>(4096.into(), 1),
        bytes_format = BinaryUnits,
        max_depth = 3,
        column_width_distribution = total 90,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            "52      ┌──bar                                   │                                   │  0%"
            " 2.5K   ├──foo                                   │                                ███│  9%"
            " 4.0K   ├──empty dir                             │                              █████│ 15%"
            " 4.1K   │ ┌──world                               │                         ░░░░░█████│ 15%"
            " 8.1K   ├─┴hello                                 │                         ██████████│ 30%"
            " 4.5K   │ ┌──subdirectory with a really long name│                        ░░░░░██████│ 16%"
            " 8.5K   ├─┴directory with a really long name     │                        ███████████│ 31%"
            "27.1K ┌─┴root                                    │███████████████████████████████████│100%"
        },
}

test_case! {
    typical_flat where
        tree = typical_tree::<Bytes>(4096.into(), 1),
        bytes_format = BinaryUnits,
        max_depth = 2,
        column_width_distribution = total 90,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            "52      ┌──bar                              │                                        │  0%"
            " 2.5K   ├──foo                              │                                    ████│  9%"
            " 4.0K   ├──empty dir                        │                                  ██████│ 15%"
            " 8.1K   ├──hello                            │                            ████████████│ 30%"
            " 8.5K   ├──directory with a really long name│                           █████████████│ 31%"
            "27.1K ┌─┴root                               │████████████████████████████████████████│100%"
        },
}

test_case! {
    typical_root_only where
        tree = typical_tree::<Bytes>(4096.into(), 1),
        bytes_format = BinaryUnits,
        max_depth = 1,
        column_width_distribution = total 90,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            "27.1K ┌──root│███████████████████████████████████████████████████████████████████████│100%"
        },
}

test_case! {
    typical_binary_tebi_scale where
        tree = typical_tree::<Bytes>(4096.into(), 1 << 40),
        bytes_format = BinaryUnits,
        max_depth = 10,
        column_width_distribution = total 90,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            "  4.0K   ┌──empty dir                             │                                  │  0%"
            " 52.0T   ├──bar                                   │                                 █│  2%"
            " 45.0T   │   ┌──hello                             │                                 ▒│  1%"
            " 54.0T   │   ├──world                             │                                 █│  2%"
            " 99.0T   │ ┌─┴world                               │                                 █│  3%"
            " 99.0T   ├─┴hello                                 │                                 █│  3%"
            "475.0T   │   ┌──file with a really long name      │                             █████│ 15%"
            "475.0T   │ ┌─┴subdirectory with a really long name│                             █████│ 15%"
            "475.0T   ├─┴directory with a really long name     │                             █████│ 15%"
            "  2.5P   ├──foo                                   │       ███████████████████████████│ 80%"
            "  3.1P ┌─┴root                                    │██████████████████████████████████│100%"
        },
}

test_case! {
    typical_metric_tebi_scale where
        tree = typical_tree::<Bytes>(4096.into(), 1 << 40),
        bytes_format = MetricUnits,
        max_depth = 10,
        column_width_distribution = total 90,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            "  4.1K   ┌──empty dir                             │                                  │  0%"
            " 57.2T   ├──bar                                   │                                 █│  2%"
            " 49.5T   │   ┌──hello                             │                                 ▒│  1%"
            " 59.4T   │   ├──world                             │                                 █│  2%"
            "108.9T   │ ┌─┴world                               │                                 █│  3%"
            "108.9T   ├─┴hello                                 │                                 █│  3%"
            "522.3T   │   ┌──file with a really long name      │                             █████│ 15%"
            "522.3T   │ ┌─┴subdirectory with a really long name│                             █████│ 15%"
            "522.3T   ├─┴directory with a really long name     │                             █████│ 15%"
            "  2.8P   ├──foo                                   │       ███████████████████████████│ 80%"
            "  3.5P ┌─┴root                                    │██████████████████████████████████│100%"
        },
}

test_case! {
    typical_empty_files where
        tree = typical_tree::<Bytes>(4096.into(), 0),
        bytes_format = BinaryUnits,
        max_depth = 10,
        column_width_distribution = total 90,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            " 0      ┌──bar                                   │                                   │  0%"
            " 0      ├──foo                                   │                                   │  0%"
            " 4.0K   ├──empty dir                             │                             ██████│ 17%"
            " 0      │   ┌──file with a really long name      │                       ░░░░░░▒▒▒▒▒▒│  0%"
            " 4.0K   │ ┌─┴subdirectory with a really long name│                       ░░░░░░██████│ 17%"
            " 8.0K   ├─┴directory with a really long name     │                       ████████████│ 33%"
            " 0      │   ┌──world                             │                       ░░░░░░▒▒▒▒▒▒│  0%"
            " 0      │   ├──hello                             │                       ░░░░░░▒▒▒▒▒▒│  0%"
            " 4.0K   │ ┌─┴world                               │                       ░░░░░░██████│ 17%"
            " 8.0K   ├─┴hello                                 │                       ████████████│ 33%"
            "24.0K ┌─┴root                                    │███████████████████████████████████│100%"
        },
}

test_case! {
    typical_empty_files_zero_sized_inodes where
        tree = typical_tree::<Bytes>(0.into(), 0),
        bytes_format = BinaryUnits,
        max_depth = 10,
        column_width_distribution = total 90,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            "0          ┌──file with a really long name      │                                    │  0%"
            "0        ┌─┴subdirectory with a really long name│                                    │  0%"
            "0      ┌─┴directory with a really long name     │                                    │  0%"
            "0      ├──empty dir                             │                                    │  0%"
            "0      │   ┌──world                             │                                    │  0%"
            "0      │   ├──hello                             │                                    │  0%"
            "0      │ ┌─┴world                               │                                    │  0%"
            "0      ├─┴hello                                 │                                    │  0%"
            "0      ├──bar                                   │                                    │  0%"
            "0      ├──foo                                   │                                    │  0%"
            "0    ┌─┴root                                    │                                    │  0%"
        },
}

fn nested_tree<Size: size::Size>(
    dir_names: &[&'static str],
    size_per_dir: Size,
    file_name: &'static str,
    file_size: Size,
) -> DataTree<&'static str, Size> {
    if let Some((head, tail)) = dir_names.split_first() {
        let child = nested_tree(tail, size_per_dir, file_name, file_size);
        DataTree::dir(*head, size_per_dir, vec![child])
    } else {
        DataTree::file(file_name, file_size)
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
        bytes_format = BinaryUnits,
        max_depth = 10,
        column_width_distribution = total 90,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            " 1.0K             ┌──z│          ░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓██│  4%"
            " 5.0K           ┌─┴f  │          ░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓████████████│ 20%"
            " 9.0K         ┌─┴e    │          ░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓██████████████████████│ 36%"
            "13.0K       ┌─┴d      │          ░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒████████████████████████████████│ 52%"
            "17.0K     ┌─┴c        │          ░░░░░░░░░░██████████████████████████████████████████│ 68%"
            "21.0K   ┌─┴b          │          ████████████████████████████████████████████████████│ 84%"
            "25.0K ┌─┴a            │██████████████████████████████████████████████████████████████│100%"
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
        bytes_format = MetricUnits,
        max_depth = 10,
        column_width_distribution = total 90,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            " 1.0K             ┌──z│          ░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓██│  4%"
            " 5.1K           ┌─┴f  │          ░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓████████████│ 20%"
            " 9.2K         ┌─┴e    │          ░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓██████████████████████│ 36%"
            "13.3K       ┌─┴d      │          ░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒████████████████████████████████│ 52%"
            "17.4K     ┌─┴c        │          ░░░░░░░░░░██████████████████████████████████████████│ 68%"
            "21.5K   ┌─┴b          │          ████████████████████████████████████████████████████│ 84%"
            "25.6K ┌─┴a            │██████████████████████████████████████████████████████████████│100%"
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
        bytes_format = BinaryUnits,
        max_depth = 10,
        column_width_distribution = total 90,
        direction = TopDown,
        bar_alignment = Right,
        expected = text_block_fnl! {
            "25.0K └─┬a            │██████████████████████████████████████████████████████████████│100%"
            "21.0K   └─┬b          │          ████████████████████████████████████████████████████│ 84%"
            "17.0K     └─┬c        │          ░░░░░░░░░░██████████████████████████████████████████│ 68%"
            "13.0K       └─┬d      │          ░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒████████████████████████████████│ 52%"
            " 9.0K         └─┬e    │          ░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓██████████████████████│ 36%"
            " 5.0K           └─┬f  │          ░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓████████████│ 20%"
            " 1.0K             └──z│          ░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓██│  4%"
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
        bytes_format = (),
        max_depth = 10,
        column_width_distribution = total 90,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            " 2             ┌──z│          ░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓███│  4%"
            "10           ┌─┴f  │          ░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓█████████████│ 20%"
            "18         ┌─┴e    │          ░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓███████████████████████│ 36%"
            "26       ┌─┴d      │          ░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒██████████████████████████████████│ 52%"
            "34     ┌─┴c        │          ░░░░░░░░░░░████████████████████████████████████████████│ 68%"
            "42   ┌─┴b          │          ███████████████████████████████████████████████████████│ 84%"
            "50 ┌─┴a            │█████████████████████████████████████████████████████████████████│100%"
        },
}

test_case! {
    nested_bottom_up_binary_align_left where
        tree = nested_tree::<Bytes>(
            &["a", "b", "c", "d", "e", "f"],
            4096.into(),
            "z",
            1024.into(),
        ),
        bytes_format = BinaryUnits,
        max_depth = 10,
        column_width_distribution = total 90,
        direction = BottomUp,
        bar_alignment = Left,
        expected = text_block_fnl! {
            " 1.0K             ┌──z│██▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒░░░░░░░░░░          │  4%"
            " 5.0K           ┌─┴f  │████████████▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒░░░░░░░░░░          │ 20%"
            " 9.0K         ┌─┴e    │██████████████████████▓▓▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒░░░░░░░░░░          │ 36%"
            "13.0K       ┌─┴d      │████████████████████████████████▒▒▒▒▒▒▒▒▒▒░░░░░░░░░░          │ 52%"
            "17.0K     ┌─┴c        │██████████████████████████████████████████░░░░░░░░░░          │ 68%"
            "21.0K   ┌─┴b          │████████████████████████████████████████████████████          │ 84%"
            "25.0K ┌─┴a            │██████████████████████████████████████████████████████████████│100%"
        },
}

test_case! {
    nested_top_down_binary_align_left where
        tree = nested_tree::<Bytes>(
            &["a", "b", "c", "d", "e", "f"],
            4096.into(),
            "z",
            1024.into(),
        ),
        bytes_format = BinaryUnits,
        max_depth = 10,
        column_width_distribution = total 90,
        direction = TopDown,
        bar_alignment = Left,
        expected = text_block_fnl! {
            "25.0K └─┬a            │██████████████████████████████████████████████████████████████│100%"
            "21.0K   └─┬b          │████████████████████████████████████████████████████          │ 84%"
            "17.0K     └─┬c        │██████████████████████████████████████████░░░░░░░░░░          │ 68%"
            "13.0K       └─┬d      │████████████████████████████████▒▒▒▒▒▒▒▒▒▒░░░░░░░░░░          │ 52%"
            " 9.0K         └─┬e    │██████████████████████▓▓▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒░░░░░░░░░░          │ 36%"
            " 5.0K           └─┬f  │████████████▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒░░░░░░░░░░          │ 20%"
            " 1.0K             └──z│██▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒░░░░░░░░░░          │  4%"
        },
}

test_case! {
    nested_bottom_up_binary_long_names_narrow_tree_column where
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
        bytes_format = BinaryUnits,
        max_depth = 10,
        column_width_distribution = components 27 57,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            " 1.0K           ┌──file with a...│           ░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓███│  5%"
            " 5.0K         ┌─┴great-great-g...│           ░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓██████████████│ 24%"
            " 9.0K       ┌─┴great-grandchil...│           ░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒████████████████████████│ 43%"
            "13.0K     ┌─┴grandchild direct...│           ░░░░░░░░░░░███████████████████████████████████│ 62%"
            "17.0K   ┌─┴child directory wit...│           ██████████████████████████████████████████████│ 81%"
            "21.0K ┌─┴directory with a long...│█████████████████████████████████████████████████████████│100%"
        },
}

test_case! {
    nested_bottom_up_binary_many_names_narrow_tree_column where
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
        bytes_format = BinaryUnits,
        max_depth = 10,
        column_width_distribution = components 24 50,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            "17.0K                 ┌──ab...│    ░░░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓█████████████████│ 35%"
            "21.0K               ┌─┴abcd...│    ░░░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓█████████████████████│ 43%"
            "25.0K             ┌─┴abcdefg  │    ░░░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓██████████████████████████│ 51%"
            "29.0K           ┌─┴abcdef     │    ░░░░▒▒▒▒▓▓▓▓▓▓▓▓██████████████████████████████│ 59%"
            "33.0K         ┌─┴abcde        │    ░░░░▒▒▒▒▓▓▓▓██████████████████████████████████│ 67%"
            "37.0K       ┌─┴abcd           │    ░░░░▒▒▒▒██████████████████████████████████████│ 76%"
            "41.0K     ┌─┴abc              │    ░░░░██████████████████████████████████████████│ 84%"
            "45.0K   ┌─┴ab                 │    ██████████████████████████████████████████████│ 92%"
            "49.0K ┌─┴a                    │██████████████████████████████████████████████████│100%"
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
        bytes_format = BinaryUnits,
        max_depth = 10,
        column_width_distribution = total 90,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            " 1.0T             ┌──z│          ░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓██│  4%"
            " 5.0T           ┌─┴f  │          ░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓████████████│ 20%"
            " 9.0T         ┌─┴e    │          ░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓██████████████████████│ 36%"
            "13.0T       ┌─┴d      │          ░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒████████████████████████████████│ 52%"
            "17.0T     ┌─┴c        │          ░░░░░░░░░░██████████████████████████████████████████│ 68%"
            "21.0T   ┌─┴b          │          ████████████████████████████████████████████████████│ 84%"
            "25.0T ┌─┴a            │██████████████████████████████████████████████████████████████│100%"
        },
}

fn empty_dir<Size>(inode_size: Size) -> DataTree<&'static str, Size>
where
    Size: size::Size + Ord + From<u64> + Send,
{
    DataTree::dir("empty directory", inode_size, Vec::new()).into_par_sorted(order_tree)
}

test_case! {
    empty_dir_non_zero_inode where
        tree = empty_dir::<Bytes>(4069.into()),
        bytes_format = MetricUnits,
        max_depth = 10,
        column_width_distribution = total 90,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            "4.1K ┌──empty directory│█████████████████████████████████████████████████████████████│100%"
        },
}

test_case! {
    empty_dir_zero_sized_inode where
        tree = empty_dir::<Bytes>(0.into()),
        bytes_format = MetricUnits,
        max_depth = 10,
        column_width_distribution = total 90,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            "0    ┌──empty directory│                                                             │  0%"
        },
}

test_case! {
    empty_file where
        tree = DataTree::file("empty file", Bytes::from(0)),
        bytes_format = MetricUnits,
        max_depth = 10,
        column_width_distribution = total 90,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            "0    ┌──empty file│                                                                  │  0%"
        },
}

fn long_and_short_names<Size>() -> DataTree<&'static str, Size>
where
    Size: size::Size + Ord + From<u64> + Send,
{
    let dir = DataTree::<&'static str, Size>::fixed_size_dir_constructor(1.into());
    let file = |name: &'static str, size: u64| DataTree::file(name, Size::from(size));
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
    long_and_short_names_fit_tree_column where
        tree = long_and_short_names::<Blocks>(),
        bytes_format = (),
        max_depth = 10,
        column_width_distribution = components 33 25,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            " 1   ┌──a                           │                         │  1%"
            " 2   ├──file with a long name 1     │                        █│  2%"
            " 3   ├──b                           │                        █│  4%"
            " 4   ├──file with a long name 2     │                        █│  5%"
            " 1   │ ┌──a                         │                    ░░░░░│  1%"
            " 2   │ ├──file with a long name 1   │                    ░░░░█│  2%"
            " 3   │ ├──b                         │                    ░░░░█│  4%"
            " 4   │ ├──file with a long name 2   │                    ░░░░█│  5%"
            " 5   │ ├──weight                    │                    ░░░██│  6%"
            "16   ├─┴c                           │                    █████│ 20%"
            " 1   │ ┌──a                         │                    ░░░░░│  1%"
            " 2   │ ├──file with a long name 1   │                    ░░░░█│  2%"
            " 3   │ ├──b                         │                    ░░░░█│  4%"
            " 4   │ ├──file with a long name 2   │                    ░░░░█│  5%"
            " 6   │ ├──weight                    │                    ░░░██│  7%"
            "17   ├─┴directory with a long name 1│                    █████│ 21%"
            " 1   │ ┌──a                         │                   ░░░░░░│  1%"
            " 2   │ ├──file with a long name 1   │                   ░░░░░█│  2%"
            " 3   │ ├──b                         │                   ░░░░░█│  4%"
            " 4   │ ├──file with a long name 2   │                   ░░░░░█│  5%"
            " 7   │ ├──weight                    │                   ░░░░██│  9%"
            "18   ├─┴d                           │                   ██████│ 22%"
            " 1   │ ┌──a                         │                   ░░░░░░│  1%"
            " 2   │ ├──file with a long name 1   │                   ░░░░░█│  2%"
            " 3   │ ├──b                         │                   ░░░░░█│  4%"
            " 4   │ ├──file with a long name 2   │                   ░░░░░█│  5%"
            " 8   │ ├──weight                    │                   ░░░░██│ 10%"
            "19   ├─┴directory with a long name 2│                   ██████│ 23%"
            "81 ┌─┴root                          │█████████████████████████│100%"
        },
}

test_case! {
    remaining_siblings_properly_connect_when_some_amongst_them_disappear where
        tree = long_and_short_names::<Blocks>(),
        bytes_format = (),
        max_depth = 10,
        column_width_distribution = components 10 25,
        direction = BottomUp,
        bar_alignment = Right,
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
        bytes_format = (),
        max_depth = 10,
        column_width_distribution = components 8 25,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            " 1   ┌──a  │                         │  1%"
            " 3   ├──b  │                        █│  4%"
            " 1   │ ┌──a│                    ░░░░░│  1%"
            " 3   │ ├──b│                    ░░░░█│  4%"
            "16   ├─┴c  │                    █████│ 20%"
            " 1   │ ┌──a│                   ░░░░░░│  1%"
            " 3   │ ├──b│                   ░░░░░█│  4%"
            "18   ├─┴d  │                   ██████│ 22%"
            "81 ┌─┴root │█████████████████████████│100%"
        },
}

test_case! {
    remaining_siblings_properly_connect_when_some_amongst_them_disappear_top_down where
        tree = long_and_short_names::<Blocks>(),
        bytes_format = (),
        max_depth = 10,
        column_width_distribution = components 10 25,
        direction = TopDown,
        bar_alignment = Right,
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
        bytes_format = (),
        max_depth = 10,
        column_width_distribution = components 8 25,
        direction = TopDown,
        bar_alignment = Right,
        expected = text_block_fnl! {
            "81 └─┬root │█████████████████████████│100%"
            "18   ├─┬d  │                   ██████│ 22%"
            " 3   │ ├──b│                   ░░░░░█│  4%"
            " 1   │ └──a│                   ░░░░░░│  1%"
            "16   ├─┬c  │                    █████│ 20%"
            " 3   │ ├──b│                    ░░░░█│  4%"
            " 1   │ └──a│                    ░░░░░│  1%"
            " 3   ├──b  │                        █│  4%"
            " 1   └──a  │                         │  1%"
        },
}

fn tree_with_a_file_of_extremely_long_name<Size>() -> DataTree<&'static str, Size>
where
    Size: size::Size + Ord + From<u64> + Send,
{
    let dir = DataTree::<&'static str, Size>::fixed_size_dir_constructor(4069.into());
    let file = |name: &'static str, size: u64| DataTree::file(name, Size::from(size));
    dir(
        "root",
        vec![file(
            "file with a very super extraordinary extremely long name",
            4069,
        )],
    )
    .into_par_sorted(order_tree)
}

test_case! {
    width_of_tree_column_is_prioritized_before_bar_column where
        tree = tree_with_a_file_of_extremely_long_name::<Bytes>(),
        bytes_format = MetricUnits,
        max_depth = 10,
        column_width_distribution = total 85,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            "4.1K   ┌──file with a very super extraordinary extremely long name│      ███████│ 50%"
            "8.1K ┌─┴root                                                      │█████████████│100%"
        },
}

test_case! {
    bar_column_has_a_minimum_width where
        tree = tree_with_a_file_of_extremely_long_name::<Bytes>(),
        bytes_format = BinaryUnits,
        max_depth = 10,
        column_width_distribution = total 50,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            "4.0K   ┌──file with a very super ex...│   ███│ 50%"
            "7.9K ┌─┴root                          │██████│100%"
        },
}

fn big_tree_with_long_names<Size>() -> DataTree<&'static str, Size>
where
    Size: size::Size + Ord + From<u64> + Send,
{
    let dir = DataTree::<&'static str, Size>::fixed_size_dir_constructor(4069.into());
    let file = |name: &'static str, size: u64| DataTree::file(name, Size::from(size));
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
    big_tree_with_long_names_narrow_tree_column where
        tree = big_tree_with_long_names::<Bytes>(),
        bytes_format = BinaryUnits,
        max_depth = 100,
        column_width_distribution = components 16 34,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            "999        ┌──first ...│                       ░░░░░░░░░░░│  0%"
            "750        │ ┌──b      │                       ░░░░░░░▒▒▒▒│  0%"
            "750        │ ├──a      │                       ░░░░░░░▒▒▒▒│  0%"
            "123        │ │ ┌──abc  │                       ░░░░░░░▒▒▒▓│  0%"
            "456        │ │ ├──def  │                       ░░░░░░░▒▒▒▓│  0%"
            "750        │ │ ├──c    │                       ░░░░░░░▒▒▒▓│  0%"
            "  5.3K     │ ├─┴firs...│                       ░░░░░░░▒▒▒█│  2%"
            "  7.6K     │ ├──seco...│                       ░░░░░░░▒▒▒█│  3%"
            "750        │ │ ┌──f    │                       ░░░░░░░▒▒▓▓│  0%"
            "750        │ │ ├──e    │                       ░░░░░░░▒▒▓▓│  0%"
            "750        │ │ ├──d    │                       ░░░░░░░▒▒▓▓│  0%"
            "  1.2K     │ │ ├──ab...│                       ░░░░░░░▒▒▓▓│  0%"
            "  1.2K     │ │ ├──ab...│                       ░░░░░░░▒▒▓▓│  0%"
            "  4.2K     │ │ ├──ih...│                       ░░░░░░░▒▒▓█│  1%"
            "  4.4K     │ │ ├──th...│                       ░░░░░░░▒▒▓█│  2%"
            " 17.2K     │ ├─┴seco...│                       ░░░░░░░▒▒██│  6%"
            " 35.5K     ├─┴sub 1.1  │                       ░░░░░░░████│ 12%"
            "750        │ ┌──g      │                       ░░░░░▒▒▒▒▒▒│  0%"
            "750        │ │ ┌──j    │                       ░░░░░▒▒▒▒▒▓│  0%"
            "750        │ │ ├──i    │                       ░░░░░▒▒▒▒▒▓│  0%"
            "750        │ │ │ ┌──m  │                       ░░░░░▒▒▒▒▒▓│  0%"
            "750        │ │ │ ├──l  │                       ░░░░░▒▒▒▒▒▓│  0%"
            "750        │ │ │ ├──k  │                       ░░░░░▒▒▒▒▒▓│  0%"
            "  6.5K     │ │ ├─┴si...│                       ░░░░░▒▒▒▒▒█│  2%"
            " 12.0K     │ ├─┴fift...│                       ░░░░░▒▒▒▒▒█│  4%"
            "750        │ │   ┌──h  │                       ░░░░░▒▒▒▒▒▓│  0%"
            "  8.3K     │ │ ┌─┴fo...│                       ░░░░░▒▒▒▒▒█│  3%"
            " 12.2K     │ ├─┴thir...│                       ░░░░░▒▒▒▒▒█│  4%"
            "750        │ │ ┌──n    │                       ░░░░░▒▒▒▓▓▓│  0%"
            "777        │ │ ├──ni...│                       ░░░░░▒▒▒▓▓▓│  0%"
            "750        │ │ │ ┌──r  │                       ░░░░░▒▒▒▓▓▓│  0%"
            "750        │ │ │ ├──q  │                       ░░░░░▒▒▒▓▓▓│  0%"
            "750        │ │ │ ├──p  │                       ░░░░░▒▒▒▓▓▓│  0%"
            "  6.6K     │ │ ├─┴ei...│                       ░░░░░▒▒▒▓▓█│  2%"
            "750        │ │ │ ┌──o  │                       ░░░░░▒▒▒▓▓▓│  0%"
            " 11.7K     │ │ ├─┴se...│                       ░░░░░▒▒▒▓▓█│  4%"
            " 23.7K     │ ├─┴sixt...│                       ░░░░░▒▒▒███│  8%"
            " 52.6K     ├─┴sub 1.2  │                       ░░░░░██████│ 18%"
            " 93.1K   ┌─┴sub 1      │                       ███████████│ 32%"
            "750      │   ┌──y      │            ░░░░░░░░░░░░░░░░░▒▒▒▒▒│  0%"
            "352      │   │ ┌──th...│            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "453      │   │ ├──tw...│            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "750      │   │ ├──z    │            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "750      │   │ │ ┌──A  │            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "  4.8K   │   │ ├─┴tw...│            ░░░░░░░░░░░░░░░░░▒▒▒▓█│  2%"
            "750      │   │ │ ┌──B  │            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "  5.6K   │   │ ├─┴th...│            ░░░░░░░░░░░░░░░░░▒▒▒▓█│  2%"
            " 15.9K   │   ├─┴elev...│            ░░░░░░░░░░░░░░░░░▒▒▒██│  6%"
            "357      │   │ ┌──ei...│            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "542      │   │ ├──ei...│            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "750      │   │ ├──E    │            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "750      │   │ ├──D    │            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "750      │   │ ├──C    │            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "750      │   │ │ ┌──G  │            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "750      │   │ │ ├──F  │            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "  5.7K   │   │ ├─┴tw...│            ░░░░░░░░░░░░░░░░░▒▒▒▓█│  2%"
            "750      │   │ │ ┌──H  │            ░░░░░░░░░░░░░░░░░▒▒▒▓▓│  0%"
            "  5.8K   │   │ ├─┴fi...│            ░░░░░░░░░░░░░░░░░▒▒▒▓█│  2%"
            " 18.5K   │   ├─┴four...│            ░░░░░░░░░░░░░░░░░▒▒▒██│  6%"
            " 39.1K   │ ┌─┴sub 2.2  │            ░░░░░░░░░░░░░░░░░█████│ 14%"
            "750      │ │ ┌──s      │            ░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒│  0%"
            "750      │ │ │ ┌──u    │            ░░░░░▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│  0%"
            "750      │ │ │ ├──t    │            ░░░░░▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│  0%"
            "750      │ │ │ │ ┌──x  │            ░░░░░▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│  0%"
            "750      │ │ │ │ ├──w  │            ░░░░░▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│  0%"
            "750      │ │ │ │ ├──v  │            ░░░░░▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│  0%"
            " 49.6K   │ │ │ ├─┴te...│            ░░░░░▓▓▓▓▓▓▓▓▓▓▓██████│ 17%"
            " 86.8K   │ │ │ ├──te...│            ░░░░░▓▓▓▓▓▓▓██████████│ 30%"
            "141.8K   │ │ ├─┴nint...│            ░░░░░█████████████████│ 49%"
            "146.5K   │ ├─┴sub 2.1  │            ░░░░░█████████████████│ 51%"
            "189.6K   ├─┴sub 2      │            ██████████████████████│ 66%"
            "286.7K ┌─┴root         │██████████████████████████████████│100%"
        },
}

test_case! {
    big_tree_with_long_names_shallow where
        tree = big_tree_with_long_names::<Bytes>(),
        bytes_format = BinaryUnits,
        max_depth = 3,
        column_width_distribution = total 90,
        direction = BottomUp,
        bar_alignment = Right,
        expected = text_block_fnl! {
            "999        ┌──first file with a long name│                             ░░░░░░░░░░░░░░│  0%"
            " 35.5K     ├──sub 1.1                    │                             ░░░░░░░░░█████│ 12%"
            " 52.6K     ├──sub 1.2                    │                             ░░░░░░████████│ 18%"
            " 93.1K   ┌─┴sub 1                        │                             ██████████████│ 32%"
            " 39.1K   │ ┌──sub 2.2                    │               ░░░░░░░░░░░░░░░░░░░░░░██████│ 14%"
            "146.5K   │ ├──sub 2.1                    │               ░░░░░░██████████████████████│ 51%"
            "189.6K   ├─┴sub 2                        │               ████████████████████████████│ 66%"
            "286.7K ┌─┴root                           │███████████████████████████████████████████│100%"
        },
}
