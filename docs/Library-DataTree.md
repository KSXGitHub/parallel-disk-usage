# DataTree

`data_tree::DataTree<Name, Size>` is the measured tree. Every other part of the library produces one, transforms one, or renders one.

```rust
pub struct DataTree<Name, Size: size::Size> {
	name: Name,
	size: Size,
	children: Vec<Self>,
}
```

The fields are private, which guarantees the tree's invariant: **a node's `size` is always greater than or equal to the sum of its children's sizes.** The constructors maintain it, and
`Reflection::par_try_into_tree` verifies it when converting untrusted data.

## Construction

Trees are normally built by [`FsTreeBuilder` or `TreeBuilder`](Library-Building-Trees.md). The direct constructors serve tests and adapters.

| Constructor                                        | Description                                                                              |
|----------------------------------------------------|------------------------------------------------------------------------------------------|
| `DataTree::file(name, size)`                       | A leaf node of the given size.                                                           |
| `DataTree::dir(name, inode_size, children)`        | A directory whose total is `inode_size` plus the sum of its children.                    |
| `DataTree::fixed_size_dir_constructor(inode_size)` | Returns a `Fn(Name, Vec<Self>) -> Self` applying the same inode size to every directory. |

`dir` takes the directory's own size, not its total. The total is computed for you.

```rust
use parallel_disk_usage::data_tree::DataTree;
use parallel_disk_usage::size::Bytes;

let dir = DataTree::< & str, Bytes>::fixed_size_dir_constructor(Bytes::new(4096));

let tree = dir("root", vec![
	DataTree::file("a.txt", Bytes::new(1024)),
	dir("nested", vec![DataTree::file("b.txt", Bytes::new(2048))]),
]);

assert_eq!(tree.size().inner(), 4096 + 1024 + 4096 + 2048);
```

## Inspection

| Method       | Returns          |
|--------------|------------------|
| `name()`     | `&Name`          |
| `name_mut()` | `&mut Name`      |
| `size()`     | `Size` (by copy) |
| `children()` | `&Vec<Self>`     |

Nothing mutates size or children in place. To restructure a tree, use the parallel transformations below or round-trip through a [`Reflection`](#reflection).

Walking the tree is ordinary recursion over `children()`:

```rust
fn print_tree<Name: std::fmt::Display, Size: parallel_disk_usage::size::Size>(
	tree: &parallel_disk_usage::data_tree::DataTree<Name, Size>,
	depth: usize,
) {
	println!("{:indent$}{} {:?}", "", tree.name(), tree.size(), indent = depth * 2);
	for child in tree.children() {
		print_tree(child, depth + 1);
	}
}
```

## Sorting

Sorting is recursive and parallel. The comparator receives two sibling subtrees.

| Method                     | Description                    |
|----------------------------|--------------------------------|
| `par_sort_by(compare)`     | Sorts every level in place.    |
| `into_par_sorted(compare)` | The consuming, chainable form. |

`pdu` sorts descending by size:

```rust
let tree = tree.into_par_sorted( | left, right| left.size().cmp( & right.size()).reverse());
```

The comparator must be `Copy + Sync`. The sort is unstable, so equal-sized siblings may land in any order; add a tiebreaker on `name()` for deterministic output.

## Filtering with `par_retain`

| Method                         | Description                                                               |
|--------------------------------|---------------------------------------------------------------------------|
| `par_retain(predicate)`        | Recursively drops every descendant for which `predicate` returns `false`. |
| `into_par_retained(predicate)` | The consuming, chainable form.                                            |

The predicate is `Fn(&Self, u64) -> bool`, where the second argument is the depth of the node's *parent*, starting at `0` for the root's children.

Dropping a node drops its whole subtree, and surviving ancestors keep their original sizes. That is deliberate: the chart still shows a directory as large when its contents are too small to list.

Trim to a fixed depth:

```rust
let tree = tree.into_par_retained( | _, depth| depth < 3);
```

Drop anything under one mebibyte:

```rust
use parallel_disk_usage::size::Bytes;

let tree = tree.into_par_retained( | node, _ | node.size() > = Bytes::new(1024 * 1024));
```

### `par_cull_insignificant_data`

*(feature: `cli`)* Drops every descendant smaller than `min_ratio` of the root's size, implementing
`--min-ratio`. A ratio that is zero, negative, or NaN is a no-op. Without the `cli` feature:

```rust
let minimal = tree.size().inner() as f32 * min_ratio;
tree.par_retain( | node, _ | node.size().inner() as f32 > = minimal);
```

## Deduplicating hardlinks

A file with several links inside the tree is counted once per link. Correcting that is the job of
`DeduplicateSharedSize::deduplicate`. See [Hardlinks](Library-Hardlinks.md).

## Reflection

`data_tree::Reflection<Name, Size>`, also exported as `DataTreeReflection`, is a structurally identical mirror of `DataTree` with public fields.

```rust
pub struct Reflection<Name, Size: size::Size> {
	pub name: Name,
	pub size: Size,
	pub children: Vec<Self>,
}
```

It is the serializable form of a tree, and it lets tests construct trees literally, including invalid ones.

### Converting

| Direction                  | API                                        | Notes                                                  |
|----------------------------|--------------------------------------------|--------------------------------------------------------|
| `DataTree` to `Reflection` | `into_reflection()`, or `Reflection::from` | Always succeeds.                                       |
| `Reflection` to `DataTree` | `par_try_into_tree()`                      | Validates the size invariant in parallel and may fail. |

`par_try_into_tree` returns `ConversionError::ExcessiveChildren` when a node is smaller than one of its children. The error carries the `path` from the root as a `VecDeque<Name>`, the parent's
`size`, and the offending `child`, and implements `Display`.

```rust
use parallel_disk_usage::data_tree::reflection::{ConversionError, Reflection};
use parallel_disk_usage::size::Bytes;

let reflection = Reflection {
name: "root".to_string(),
size: Bytes::new(100),
children: vec![Reflection {
	name: "child".to_string(),
	size: Bytes::new(200),
	children: Vec::new(),
}],
};

match reflection.par_try_into_tree() {
Ok(_) => unreachable ! ("the child is larger than its parent"),
Err(ConversionError::ExcessiveChildren { path, .. }) => {
assert_eq ! (path.len(), 1);
}
}
```

`ConversionError` is `#[non_exhaustive]`; add a trailing `_ => {}` arm to keep compiling across versions.

### Transforming

| Method                        | Description                                                                                                                                   |
|-------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------|
| `par_try_map(transform)`      | Applies `Fn(Name, Size) -> Result<(TargetName, TargetSize), Error>` to every node in parallel. Changes the name type, the size unit, or both. |
| `par_convert_names_to_utf8()` | Converts `OsString`-like names to `String`, returning the first offending name as the error. `pdu` runs this before emitting JSON.            |

```rust
let reflection = data_tree
.into_reflection()
.par_convert_names_to_utf8()
.expect("all names are valid UTF-8");
```

### Serialization

*(feature: `json`)* `Reflection` derives `Serialize` and `Deserialize` with
`rename_all = "kebab-case"`. `DataTree` deliberately does not, so deserialization cannot produce a tree that violates the invariant. Deserialize into a reflection, then validate with
`par_try_into_tree`.

For the full document format of `pdu --json-output`, see [JSON Data](Library-JSON-Data.md).
