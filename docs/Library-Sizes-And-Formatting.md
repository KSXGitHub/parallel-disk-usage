# Sizes and Formatting

Three modules decide how big a file is and how that number is printed: `size` defines the units,
`get_size` reads a unit off a file, and `bytes_format` renders it.

## The `Size` trait

`size::Size` is the bound every measurement unit satisfies. It requires the usual arithmetic and ordering operations plus multiplication by the unsigned integer types, so trees can be summed, compared, and scaled.

```rust
pub trait Size: /* Debug + Default + Copy + Ord + Add + Sub + Sum + ... */ {
	type Inner: From<Self> + Into<Self> + Mul<Self, Output = Self>;
	type DisplayFormat: Copy;
	type DisplayOutput: Display;
	fn display(self, input: Self::DisplayFormat) -> Self::DisplayOutput;
}
```

`Inner` is the underlying primitive, `u64` for both built-in units. `DisplayFormat` is the configuration rendering needs. There is no reason to implement the trait yourself unless you are inventing a unit.

## `Bytes` and `Blocks`

Both are newtypes over `u64` with the same small API.

|                 | `Bytes`                | `Blocks`                               |
|-----------------|------------------------|----------------------------------------|
| Meaning         | A count of bytes.      | A count of 512-byte filesystem blocks. |
| `DisplayFormat` | `BytesFormat`          | `()`                                   |
| `DisplayOutput` | `bytes_format::Output` | `u64`                                  |

```rust
use parallel_disk_usage::size::{Bytes, Size};
use parallel_disk_usage::bytes_format::BytesFormat;

let size = Bytes::new(1_500_000);
assert_eq!(size.inner(), 1_500_000);
assert_eq!(size.display(BytesFormat::MetricUnits).to_string(), "1.5M");
```

`new` and `inner` are `const fn`. `From<u64>` and `Into<u64>` are derived. Addition, subtraction,
`Sum`, and multiplication by `u8` through `u64` and `usize` are all available, which is what lets the tree arithmetic and hardlink deduplication be written directly.

## `GetSize`

`get_size::GetSize` maps a `std::fs::Metadata` to a size. It decides whether `pdu` reports apparent size or on-disk usage.

```rust
pub trait GetSize {
	type Size;
	fn get_size(&self, metadata: &Metadata) -> Self::Size;
}
```

| Implementation    | `Size`   | Returns                                        | Platform  |
|-------------------|----------|------------------------------------------------|-----------|
| `GetApparentSize` | `Bytes`  | `metadata.len()`, the logical file length.     | All       |
| `GetBlockSize`    | `Bytes`  | `metadata.blocks() * 512`, the space occupied. | Unix only |
| `GetBlockCount`   | `Blocks` | `metadata.blocks()`.                           | Unix only |

The difference shows up in sparse files and in files whose tail does not fill a block.
`GetApparentSize` reports what an application would read; `GetBlockSize` reports what the filesystem spends. The block-based implementations need
`std::os::unix::fs::MetadataExt`, so cross-platform code must gate them behind `#[cfg(unix)]`.

All three are zero-sized types.

### Writing your own

```rust
use parallel_disk_usage::get_size::GetSize;
use parallel_disk_usage::size::Blocks;
use std::fs::Metadata;

#[derive(Clone, Copy)]
struct CountFiles;

impl GetSize for CountFiles {
	type Size = Blocks;
	fn get_size(&self, metadata: &Metadata) -> Self::Size {
		if metadata.is_dir() { Blocks::new(0) } else { Blocks::new(1) }
	}
}
```

`FsTreeBuilder` requires the getter to be `Sync`; the CLI's `Sub` additionally requires `Copy`. Keep implementations stateless.

## Formatting bytes

`bytes_format::BytesFormat` is the `DisplayFormat` of `Bytes` and mirrors `--bytes-format`.

| Variant       | Behaviour               | 1,500,000 renders as |
|---------------|-------------------------|----------------------|
| `PlainNumber` | The raw count, no unit. | `1500000`            |
| `MetricUnits` | Powers of 1000.         | `1.5M`               |
| `BinaryUnits` | Powers of 1024.         | `1.4M`               |

`BytesFormat::format(bytes) -> Output` performs the conversion and is what `Bytes::display` calls.

```rust
use parallel_disk_usage::bytes_format::BytesFormat;

assert_eq!(BytesFormat::MetricUnits.format(1_500_000).to_string(), "1.5M");
assert_eq!(BytesFormat::PlainNumber.format(1_500_000).to_string(), "1500000");
```

### Underlying pieces

* `bytes_format::Formatter` performs the scaling. It takes a scale base; the constants
  `formatter::METRIC` and `formatter::BINARY` cover the standard two, built on
  `scale_base::METRIC` (1000) and `scale_base::BINARY` (1024).
* `Formatter::parse_value(value)` returns a `ParsedValue`: `Small { value }` below the scale base, otherwise `Big { coefficient, unit, scale, exponent }` with a unit of `K`, `M`, `G`, `T`, or `P`.
* `bytes_format::Output` is the `Display` type, either `PlainNumber(u64)` or `Units(ParsedValue)`.

`ParsedValue` renders `Big` to one decimal place and pads `Small` with trailing spaces so a column of values lines up. Use `Formatter` directly for the components rather than the string:

```rust
use parallel_disk_usage::bytes_format::{ParsedValue, formatter::BINARY};

match BINARY.parse_value(3_221_225_472) {
ParsedValue::Big { coefficient, unit, exponent, .. } => {
assert_eq ! (unit, 'G');
assert_eq !(exponent, 3);
assert ! ((coefficient - 3.0).abs() < f32::EPSILON);
}
ParsedValue::Small { .. } => unreachable ! (),
}
```

## Choosing a unit at run time

`Size` is a trait and the builders are generic over it, so the unit is fixed at compile time within any one call. A program deciding at run time ends up with one code path per unit. `pdu` dispatches once in `app::Sub` and then runs fully monomorphized code. Mirror that structure; `Size` is not object-safe, so boxing is not an option.
