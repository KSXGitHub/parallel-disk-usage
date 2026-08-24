# JSON Data

*(feature: `json`)*

The `json_data` module defines the document `pdu --json-output` writes and `pdu --json-input`
reads. Use it to exchange trees with the `pdu` program, or as a stable on-disk format for a measurement.

The module always exists, but its `Serialize` and `Deserialize` derives require the `json` feature. See [Feature Flags](Library-Feature-Flags.md).

## Structure

```text
JsonData
├── schema-version : SchemaVersion
├── pdu            : Option<BinaryVersion>
└── body           : JsonDataBody         (flattened)
    └── unit       : "bytes" | "blocks"   (internally tagged)
        └── JsonTree<Size>
            ├── tree   : DataTreeReflection<String, Size>   (flattened via Deref)
            └── shared : JsonShared<Size>
                ├── details : Option<HardlinkListReflection<Size>>
                └── summary : Option<SharedLinkSummary<Size>>
```

Field names are kebab-case. `JsonDataBody` is internally tagged on `unit`, so one document can carry either a byte tree or a block tree without ambiguity.

```json
{
  "schema-version": "2026-04-02",
  "pdu": "0.24.0",
  "unit": "bytes",
  "tree": {
    "name": "root",
    "size": 4096,
    "children": []
  }
}
```

## Types

| Type               | Role                                                                    |
|--------------------|-------------------------------------------------------------------------|
| `JsonData`         | The whole document.                                                     |
| `JsonDataBody`     | `Bytes(JsonTree<Bytes>)` or `Blocks(JsonTree<Blocks>)`.                 |
| `JsonTree<Size>`   | The tree plus its hardlink information. Derefs to the inner reflection. |
| `JsonShared<Size>` | Optional hardlink `details` and `summary`.                              |
| `SchemaVersion`    | A zero-sized token that validates the schema string on deserialization. |
| `BinaryVersion`    | The version of the `pdu` that produced the document.                    |

`JsonDataBody` derives `From` and `TryInto`, so `json_tree.into()` builds the body and
`body.try_into()` extracts a tree of a specific unit.

## Versioning

`SchemaVersion` serializes to the constant `json_data::SCHEMA_VERSION` and deserializes only from that exact string. Anything else produces an `InvalidSchema` error naming the offending input, so an incompatible document fails at parse time rather than producing a wrong tree.

`BinaryVersion` is informational and not validated. `BinaryVersion::current()` returns the version of the crate you compiled against, from `json_data::CURRENT_VERSION`.

## Writing

Convert the tree to a reflection with `String` names first; JSON cannot represent an arbitrary
`OsString`.

```rust
use parallel_disk_usage::json_data::{JsonData, JsonShared, JsonTree, SchemaVersion, BinaryVersion};

let tree = data_tree
.into_reflection()
.par_convert_names_to_utf8()
.expect("all names are valid UTF-8");

let json_data = JsonData {
schema_version: SchemaVersion,
binary_version: Some(BinaryVersion::current()),
body: JsonTree { tree, shared: JsonShared::default () }.into(),
};

serde_json::to_writer(std::io::stdout(), & json_data).expect("serialize the tree");
```

`par_convert_names_to_utf8` returns the first non-UTF-8 name as its error. Handle it rather than unwrapping if the scanned filesystem might contain such names.

`shared` is skipped during serialization when both `details` and `summary` are absent or empty, so a document without hardlink information has no `shared` key.

## Reading

```rust
use parallel_disk_usage::json_data::{JsonData, JsonDataBody};

let json_data: JsonData = serde_json::from_reader(std::io::stdin()).expect("parse the document");

match json_data.body {
JsonDataBody::Bytes(json_tree) => {
let data_tree = json_tree.tree.par_try_into_tree().expect("valid tree");
// render or inspect
}
JsonDataBody::Blocks(json_tree) => {
let data_tree = json_tree.tree.par_try_into_tree().expect("valid tree");
}
}
```

Deserialization yields a `Reflection`, not a `DataTree`, and the reflection is untrusted.
`par_try_into_tree` verifies that no node is smaller than its children; do not skip it on data you did not produce. See [DataTree](Library-DataTree.md#reflection).

The two arms have different `Size` types, so code handling both ends up duplicated or generic. This is the monomorphization constraint described in
[Sizes and Formatting](Library-Sizes-And-Formatting.md#choosing-a-unit-at-run-time).

## Hardlink information

Populate `JsonShared` from the record that
[`DeduplicateSharedSize::deduplicate`](Library-Hardlinks.md) returned:

```rust
use parallel_disk_usage::hardlink::hardlink_list::summary::SummarizeHardlinks;
use parallel_disk_usage::json_data::JsonShared;

let summary = record.iter().summarize_hardlinks();
let shared = JsonShared {
details: Some(record.into_reflection()),
summary: Some(summary),
};
```

Set either field to `None` to omit it, as `--omit-json-shared-details` and
`--omit-json-shared-summary` do.

## Using the crate's `serde`

The crate re-exports the exact `serde` and `serde_json` versions it was built against as
`parallel_disk_usage::serde` and `parallel_disk_usage::serde_json`. Prefer them over your own dependency entries on a version mismatch in the derived traits.
