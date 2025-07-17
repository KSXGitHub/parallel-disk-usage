use parallel_disk_usage::{
    data_tree::{reflection::ConversionError, DataTree, Reflection},
    size::Bytes,
};
use pretty_assertions::assert_eq;
use std::mem::transmute;

type SampleName = &'static str;
type SampleData = Bytes;
type SampleReflection = Reflection<SampleName, SampleData>;
type SampleTree = DataTree<SampleName, SampleData>;

fn valid_reflection() -> SampleReflection {
    Reflection {
        name: "root",
        size: Bytes::new(7853),
        children: vec![
            Reflection {
                name: "a",
                size: Bytes::new(78),
                children: Vec::new(),
            },
            Reflection {
                name: "b",
                size: Bytes::new(321),
                children: vec![Reflection {
                    name: "0",
                    size: Bytes::new(321),
                    children: Vec::new(),
                }],
            },
            Reflection {
                name: "c",
                size: Bytes::new(3456),
                children: vec![
                    Reflection {
                        name: "0",
                        size: Bytes::new(732),
                        children: Vec::new(),
                    },
                    Reflection {
                        name: "1",
                        size: Bytes::new(352),
                        children: Vec::new(),
                    },
                ],
            },
        ],
    }
}

fn invalid_reflection_excessive_children() -> SampleReflection {
    Reflection {
        name: "root",
        size: Bytes::new(2468),
        children: vec![
            Reflection {
                name: "a",
                size: Bytes::new(78),
                children: Vec::new(),
            },
            Reflection {
                name: "b",
                size: Bytes::new(321),
                children: vec![Reflection {
                    name: "0",
                    size: Bytes::new(321),
                    children: vec![
                        Reflection {
                            name: "abc",
                            size: Bytes::new(123),
                            children: vec![Reflection {
                                name: "xyz",
                                size: Bytes::new(4321),
                                children: Vec::new(),
                            }],
                        },
                        Reflection {
                            name: "def",
                            size: Bytes::new(456),
                            children: Vec::new(),
                        },
                    ],
                }],
            },
            Reflection {
                name: "c",
                size: Bytes::new(1084),
                children: vec![
                    Reflection {
                        name: "0",
                        size: Bytes::new(732),
                        children: Vec::new(),
                    },
                    Reflection {
                        name: "1",
                        size: Bytes::new(352),
                        children: Vec::new(),
                    },
                ],
            },
        ],
    }
}

#[test]
fn valid_conversion() {
    let actual = valid_reflection()
        .par_try_into_tree()
        .expect("create tree")
        .into_reflection();
    let expected = valid_reflection();
    assert_eq!(actual, expected);
}

#[test]
fn invalid_conversion_excessive_children() {
    let actual = invalid_reflection_excessive_children()
        .par_try_into_tree()
        .expect_err("create error");
    let expected = ConversionError::ExcessiveChildren {
        path: vec!["root", "b", "0"].into_iter().collect(),
        size: Bytes::new(321),
        child: Reflection {
            name: "def",
            size: Bytes::new(456),
            children: Vec::new(),
        },
    };
    assert_eq!(actual, expected);
}

#[test]
fn display_excessive_children() {
    let actual = invalid_reflection_excessive_children()
        .par_try_into_tree()
        .expect_err("create error")
        .to_string();
    let expected = if cfg!(unix) {
        r#"ExcessiveChildren: "root/b/0" (Bytes(321)) is less than a child named "def" (Bytes(456))"#
    } else if cfg!(windows) {
        // TODO: stop using debug format
        r#"ExcessiveChildren: "root\\b\\0" (Bytes(321)) is less than a child named "def" (Bytes(456))"#
    } else {
        eprintln!("ACTUAL: {actual}");
        panic!("This platform isn't supported!");
    };
    assert_eq!(actual, expected);
}

#[test]
fn transmute_tree_into_reflection() {
    let valid_tree = valid_reflection()
        .par_try_into_tree()
        .expect("create valid tree");
    let actual: SampleReflection = unsafe { transmute(valid_tree) };
    let expected = valid_reflection();
    assert_eq!(actual, expected);
}

#[test]
fn transmute_reflection_into_tree() {
    let actual: SampleTree = unsafe { transmute(valid_reflection()) };
    let expected = valid_reflection()
        .par_try_into_tree()
        .expect("create expected tree");
    assert_eq!(actual, expected);
}
