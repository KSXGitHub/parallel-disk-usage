use crate::{data_tree::DataTree, size::MetricBytes};
use itertools::Itertools;
use pretty_assertions::assert_eq;

type SampleName = String;
type SampleData = MetricBytes;
type SampleTree = DataTree<SampleName, SampleData>;

fn dir<const INODE_SIZE: u64>(name: &'static str, children: Vec<SampleTree>) -> SampleTree {
    SampleTree::dir(name.to_string(), INODE_SIZE.into(), children)
}

fn file(name: &'static str, size: u64) -> SampleTree {
    SampleTree::file(name.to_string(), size.into())
}

#[test]
fn typical_case() {
    let dir = dir::<4069>;
    let actual = dir(
        "root",
        vec![
            dir("empty directory", vec![]),
            dir("directory of one empty file", vec![file("empty file", 0)]),
            dir(
                "directory of one small file",
                vec![file("small file", 1024)],
            ),
            dir(
                "directory of multiple small files",
                vec![
                    file("a", 90),
                    file("b", 90),
                    file("c", 90),
                    file("d", 90),
                    file("e", 90),
                    file("f", 90),
                    file("g", 90),
                    file("h", 90),
                    file("i", 90),
                    file("j", 90),
                    file("k", 90),
                ],
            ),
            dir(
                "directory of multiple files of various sizes",
                vec![
                    file("a", 90),
                    file("b", 90),
                    file("c", 90),
                    file("d", 90),
                    file("e", 90),
                    file("f", 90),
                    file("g", 90),
                    file("h", 90),
                    file("i", 90),
                    file("j", 90),
                    file("A", 4321),
                    file("B", 4321),
                    file("C", 4321),
                    file("D", 4321),
                    file("E", 4321),
                    file("F", 4321),
                ],
            ),
        ],
    )
    .into_par_sorted(|left, right| left.name().cmp(right.name()))
    .par_partial_reduce_insignificant_data(0.05, |param| {
        param
            .reduced_children
            .iter()
            .map(SampleTree::name)
            .join(", ")
    })
    .into_reflection();
    let expected = dir(
        "root",
        vec![
            dir(
                "directory of multiple files of various sizes",
                vec![
                    file("A", 4321),
                    file("B", 4321),
                    file("C", 4321),
                    file("D", 4321),
                    file("E", 4321),
                    file("F", 4321),
                    file("a, b, c, d, e, f, g, h, i, j", 10 * 90),
                ],
            ),
            file("directory of multiple small files", 4069 + 11 * 90),
            dir("directory of one empty file", vec![file("empty file", 0)]),
            dir(
                "directory of one small file",
                vec![file("small file", 1024)],
            ),
            file("empty directory", 4069),
        ],
    )
    .into_reflection();
    assert_eq!(actual, expected);
}
