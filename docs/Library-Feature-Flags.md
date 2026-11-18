# Feature Flags

| Feature           | Default | Enables                                                                                                                                                                         |
|-------------------|---------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `cli`             | yes     | The `pdu` binary, the `app`, `args`, `man_page`, `usage_md`, and `runtime_error` modules, the `clap`, `clap_complete`, and `clap_utilities` re-exports, and the `json` feature. |
| `json`            | no      | `Serialize` and `Deserialize` implementations, and the `serde` and `serde_json` re-exports.                                                                                     |
| `cli-completions` | no      | The `pdu-completions` binary. Implies `cli`.                                                                                                                                    |
| `man-page`        | no      | The `pdu-man-page` binary. Implies `cli`.                                                                                                                                       |
| `usage-md`        | no      | The `pdu-usage-md` binary. Implies `cli`.                                                                                                                                       |
| `ai-instructions` | no      | The `pdu-ai-instructions` binary.                                                                                                                                               |

The four binary-producing features generate the project's own documentation artifacts and are not useful to library consumers.

## Recommended configuration

Disabling default features removes `clap` and its dependency tree.

```toml
[dependencies]
parallel-disk-usage = { version = "0.24", default-features = false }
```

Add `features = ["json"]` to serialize a tree.

## Gated behind `cli`

Most of the measurement and visualization API needs no features at all. The exceptions:

* `DataTree::par_cull_insignificant_data`, the ratio filter behind `--min-ratio`. Without `cli`, express it with [`par_retain`](Library-DataTree.md#filtering-with-par_retain).
* The `app`, `args`, `man_page`, `usage_md`, and `runtime_error` modules, which implement the command-line program.

## Gated behind `json`

* `Serialize` and `Deserialize` for `Reflection`, `Bytes`, `Blocks`, `OsStringDisplay`,
  `InodeNumber`, `DeviceNumber`, `HardlinkListReflection`, `SharedLinkSummary`, and `JsonData`.
* The `parallel_disk_usage::serde` and `parallel_disk_usage::serde_json` re-exports, which expose the exact versions the crate was compiled against.

The `json_data` module itself always exists; only its derives are conditional.

## Unix-only items

Independent of any feature flag, these rely on `std::os::unix::fs::MetadataExt`:

* `get_size::GetBlockSize` and `get_size::GetBlockCount`.
* `hardlink::HardlinkAware` and the `hardlink::aware` module, and therefore all hardlink deduplication. `hardlink::HardlinkIgnorant` exists everywhere.
* `InodeNumber::get` and `DeviceNumber::get`.

Code that must build on Windows should gate these behind `#[cfg(unix)]` or use the
`hardlink::record::Do` and `hardlink::record::DoNot` aliases described in
[Hardlinks](Library-Hardlinks.md#platform-support).
