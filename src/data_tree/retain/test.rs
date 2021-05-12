use crate::{data_tree::DataTree, size::MetricBytes};
use pretty_assertions::assert_eq;
use std::{cmp::Ordering, ops::Not};

type SampleName = String;
type SampleData = MetricBytes;
type SampleTree = DataTree<SampleName, SampleData>;

fn dir<const INODE_SIZE: u64>(name: &'static str, children: Vec<SampleTree>) -> SampleTree {
    SampleTree::dir(name.to_string(), INODE_SIZE.into(), children)
}

fn file(name: &'static str, size: u64) -> SampleTree {
    SampleTree::file(name.to_string(), size.into())
}

fn culled_dir<const INODE_SIZE: u64>(
    name: &'static str,
    culled_size: u64,
    children: Vec<SampleTree>,
) -> SampleTree {
    SampleTree::dir(
        name.to_string(),
        (INODE_SIZE + culled_size).into(),
        children,
    )
}

fn order_tree(left: &SampleTree, right: &SampleTree) -> Ordering {
    left.name().cmp(right.name())
}

#[test]
fn typical_case() {
    let dir = dir::<4069>;
    let culled_dir = culled_dir::<4069>;
    let actual = dir(
        "root",
        vec![
            dir("empty directory", vec![]),
            dir("directory of one empty file", vec![file("empty file", 0)]),
            dir(
                "directory of one small file",
                vec![file("small file", 4321)],
            ),
            dir("directory of one big file", vec![file("big file", 54321)]),
            dir(
                "directory of multiple small files",
                vec![
                    file("a", 4321),
                    file("b", 4321),
                    file("c", 4321),
                    file("d", 4321),
                    file("e", 4321),
                    file("f", 4321),
                    file("g", 4321),
                    file("h", 4321),
                    file("i", 4321),
                    file("j", 4321),
                    file("k", 4321),
                ],
            ),
            dir(
                "directory of multiple files of various sizes",
                vec![
                    file("a", 4321),
                    file("b", 4321),
                    file("c", 4321),
                    file("d", 4321),
                    file("e", 4321),
                    file("f", 4321),
                    file("g", 4321),
                    file("h", 4321),
                    file("i", 4321),
                    file("j", 4321),
                    file("A", 54321),
                    file("B", 54321),
                    file("C", 54321),
                    file("D", 54321),
                    file("E", 54321),
                    file("F", 54321),
                ],
            ),
        ],
    )
    .into_par_sorted(order_tree)
    .into_insignificant_data_par_culled(0.05)
    .into_reflection();
    let expected = culled_dir(
        "root",
        4069 + 4069 + (4069 + 4321),
        vec![
            culled_dir(
                "directory of multiple files of various sizes",
                10 * 4321,
                vec![
                    file("A", 54321),
                    file("B", 54321),
                    file("C", 54321),
                    file("D", 54321),
                    file("E", 54321),
                    file("F", 54321),
                ],
            ),
            file("directory of multiple small files", 4069 + 11 * 4321),
            dir("directory of one big file", vec![file("big file", 54321)]),
        ],
    )
    .into_reflection();
    assert_eq!(actual, expected);
}

#[test]
fn edge_cases() {
    let dir = dir::<4069>;
    let culled_dir = culled_dir::<4069>;
    let actual = dir(
        "root",
        vec![
            dir(
                "reduce half",
                vec![
                    file("!abc", 123),
                    file("abc", 321),
                    file("!def", 456),
                    file("def", 654),
                    file("!ghi", 789),
                    file("ghi", 987),
                ],
            ),
            dir(
                "reduce all",
                vec![file("!abc", 123), file("!def", 456), file("!ghi", 789)],
            ),
            dir(
                "reduce none",
                vec![file("abc", 321), file("def", 654), file("ghi", 987)],
            ),
            dir(
                "reduce one",
                vec![
                    file("abc", 321),
                    file("def", 654),
                    file("!def", 456),
                    file("ghi", 987),
                ],
            ),
            dir(
                "reduce all but one",
                vec![
                    file("!abc", 123),
                    file("!def", 456),
                    file("!ghi", 789),
                    file("def", 654),
                ],
            ),
        ],
    )
    .into_par_retained(|descendant| descendant.name().starts_with('!').not())
    .into_reflection();
    let expected = dir(
        "root",
        vec![
            culled_dir(
                "reduce half",
                123 + 456 + 789,
                vec![file("abc", 321), file("def", 654), file("ghi", 987)],
            ),
            file("reduce all", 4069 + 123 + 456 + 789),
            dir(
                "reduce none",
                vec![file("abc", 321), file("def", 654), file("ghi", 987)],
            ),
            culled_dir(
                "reduce one",
                456,
                vec![file("abc", 321), file("def", 654), file("ghi", 987)],
            ),
            culled_dir(
                "reduce all but one",
                123 + 456 + 789,
                vec![file("def", 654)],
            ),
        ],
    )
    .into_reflection();
    assert_eq!(actual, expected);
}
