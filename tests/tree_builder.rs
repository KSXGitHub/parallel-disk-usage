use build_fs_tree::{dir, file, FileSystemTree};
use derive_more::From;
use parallel_disk_usage::{
    data_tree::{DataTree, DataTreeReflection},
    size::Bytes,
    tree_builder::{Info, TreeBuilder},
};
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;

type SampleData = Bytes;
type SampleName = String;
const SAMPLE_SEPARATOR: char = '/';
const SAMPLE_DIR_SIZE: SampleData = Bytes::new(5);

#[derive(Debug, From)]
struct SampleTree(FileSystemTree<String, &'static str>);

const fn len(text: &str) -> SampleData {
    SampleData::new(text.len() as u64)
}

impl SampleTree {
    fn create_sample() -> Self {
        SampleTree::from(dir! {
            "flat" => dir! {
                "0" => file!("")
                "1" => file!("a")
                "2" => file!("ab")
                "3" => file!("abc")
            }
            "nested" => dir! {
                "0" => dir! {
                    "1" => file!("abcdef")
                }
            }
            "empty-dir" => dir! {}
        })
    }

    fn tree(&self, root: &'static str) -> DataTree<SampleName, SampleData> {
        TreeBuilder::builder()
            .path(root.to_string())
            .name(root.to_string())
            .get_info(|path| {
                let path: Vec<_> = path
                    .split(SAMPLE_SEPARATOR)
                    .map(ToString::to_string)
                    .collect();
                let mut path = path.iter();
                match self.0.path(&mut path) {
                    Some(FileSystemTree::File(content)) => Info::from((len(content), Vec::new())),
                    Some(FileSystemTree::Directory(content)) => Info::from((
                        SAMPLE_DIR_SIZE,
                        content.keys().map(ToString::to_string).collect(),
                    )),
                    None => panic!("Path does not exist"),
                }
            })
            .join_path(|prefix, name| format!("{prefix}{SAMPLE_SEPARATOR}{name}"))
            .max_depth(10)
            .build()
            .pipe(DataTree::from)
            .into_par_sorted(|left, right| left.name().as_str().cmp(right.name().as_str()))
    }
}

#[test]
fn flat() {
    let actual = SampleTree::create_sample().tree("flat").into_reflection();
    let expected = DataTreeReflection {
        name: "flat".to_string(),
        size: len("") + len("a") + len("ab") + len("abc") + SAMPLE_DIR_SIZE,
        children: vec![
            DataTreeReflection {
                name: "0".to_string(),
                size: len(""),
                children: Vec::new(),
            },
            DataTreeReflection {
                name: "1".to_string(),
                size: len("a"),
                children: Vec::new(),
            },
            DataTreeReflection {
                name: "2".to_string(),
                size: len("ab"),
                children: Vec::new(),
            },
            DataTreeReflection {
                name: "3".to_string(),
                size: len("abc"),
                children: Vec::new(),
            },
        ],
    };
    assert_eq!(actual, expected);
}

#[test]
fn nested() {
    let actual = SampleTree::create_sample().tree("nested").into_reflection();
    let expected = DataTreeReflection {
        name: "nested".to_string(),
        size: len("abcdef") + SAMPLE_DIR_SIZE + SAMPLE_DIR_SIZE,
        children: vec![DataTreeReflection {
            name: "0".to_string(),
            size: len("abcdef") + SAMPLE_DIR_SIZE,
            children: vec![DataTreeReflection {
                name: "1".to_string(),
                size: len("abcdef"),
                children: Vec::new(),
            }],
        }],
    };
    assert_eq!(actual, expected);
}

#[test]
fn empty_dir() {
    let actual = SampleTree::create_sample()
        .tree("empty-dir")
        .into_reflection();
    let expected = DataTreeReflection {
        name: "empty-dir".to_string(),
        size: SAMPLE_DIR_SIZE,
        children: Vec::new(),
    };
    assert_eq!(actual, expected);
}
