# Reporters

Measurement produces two kinds of side information: progress counters and filesystem errors. The
`reporter` module delivers both through one trait, so the traversal never stops to print.

## The traits

```rust
pub trait Reporter<Size: size::Size> {
	fn report(&self, event: Event<Size>);
}

pub trait ParallelReporter<Size: size::Size>: Reporter<Size> {
	type DestructionError;
	fn destroy(self) -> Result<(), Self::DestructionError>;
}
```

`report` takes `&self` because the reporter is shared by every worker thread, so any state it accumulates must be internally synchronized, typically with atomics.

`ParallelReporter::destroy` shuts down whatever background threads the reporter owns. Call it once the tree is complete and before rendering, so a progress line does not interleave with the chart.

A blanket implementation forwards `Reporter` through shared references, which is why
`FsTreeBuilder` can hold a `&Report`.

## Events

`reporter::Event` is the message type. It is `#[non_exhaustive]`, so match with a catch-all arm.

| Variant                             | Emitted when                                                                   |
|-------------------------------------|--------------------------------------------------------------------------------|
| `ReceiveData(Size)`                 | An item's size has been measured. Once per file and directory.                 |
| `EncounterError(ErrorReport)`       | A filesystem operation failed.                                                 |
| `DetectHardlink(HardlinkDetection)` | A file with more than one link was found. Only with a hardlink-aware recorder. |

`HardlinkDetection` carries the `path`, the `stats`, the `size`, and `links`, the total number of links to the inode including this one.

## Error reports

```rust
pub struct ErrorReport<'a> {
	pub operation: Operation,
	pub path: &'a Path,
	pub error: std::io::Error,
}
```

`Operation` names the failing call and offers `name()` for a human-readable form.

| Variant           | `name()`           | Cause                                                  |
|-------------------|--------------------|--------------------------------------------------------|
| `SymlinkMetadata` | `symlink_metadata` | The entry could not be stated.                         |
| `ReadDirectory`   | `read_dir`         | A directory could not be listed.                       |
| `AccessEntry`     | `access entry`     | One entry of a listed directory could not be accessed. |

Two handlers are provided as associated constants of type `fn(ErrorReport)`:

* `ErrorReport::SILENT` ignores the report.
* `ErrorReport::TEXT` prints `[error] {operation} {path:?}: {error}` to stderr via the
  [status board](#the-status-board).

The report borrows its path, so a handler that keeps the information must copy it out.

## `ErrorOnlyReporter`

Ignores progress and forwards only errors.

```rust
use parallel_disk_usage::reporter::{ErrorOnlyReporter, ErrorReport};

let reporter = ErrorOnlyReporter::new(ErrorReport::TEXT);
```

`new` accepts any `Fn(ErrorReport)`. Its `destroy` is a no-op that cannot fail, since it owns no threads. This is the right choice whenever you do not need live progress.

## `ProgressAndErrorReporter`

Accumulates counters in atomics and spawns a thread that publishes them on a fixed interval.

```rust
use parallel_disk_usage::reporter::{ErrorReport, ProgressAndErrorReporter, ProgressReport};
use parallel_disk_usage::size::Bytes;
use std::time::Duration;

let reporter = ProgressAndErrorReporter::<Bytes, _ >::new(
ProgressReport::TEXT,
Duration::from_millis(100),
ErrorReport::TEXT,
);
```

The arguments are the progress handler, the publication interval, and the error handler. The progress handler runs on the reporter's own thread, so a slow handler delays publication but never the measurement.

`ProgressReport<Size>` holds the counters:

| Field    | Meaning                                              |
|----------|------------------------------------------------------|
| `items`  | Number of items measured.                            |
| `total`  | Sum of their sizes.                                  |
| `errors` | Number of errors encountered.                        |
| `linked` | Total number of links across all detected hardlinks. |
| `shared` | Total size of all detected hardlinks.                |

`ProgressReport::TEXT` renders these as the one-line status `pdu` shows, omitting the hardlink and error counts when they are zero. The struct also derives `with_`-prefixed setters, useful in tests.

Errors are reported before progress, so an error message never appears to describe a state newer than the counters printed after it.

### Shutting it down

`destroy` stops the thread and joins it, returning `Result<(), Box<dyn Any + Send>>`, the payload of a panic in the reporting thread. `stop_progress_reporter` signals the thread without joining. Dropping the reporter also stops it, but joining is what guarantees no further output.

```rust
use parallel_disk_usage::reporter::ParallelReporter;

if reporter.destroy().is_err() {
eprintln ! ("[warning] the progress reporting thread panicked");
}
```

## Writing a custom reporter

```rust
use parallel_disk_usage::reporter::{Event, Reporter};
use parallel_disk_usage::size::Bytes;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

#[derive(Default)]
struct Counter {
	items: AtomicU64,
	errors: AtomicU64,
}

impl Reporter<Bytes> for Counter {
	fn report(&self, event: Event<Bytes>) {
		match event {
			Event::ReceiveData(_) => { self.items.fetch_add(1, Relaxed); }
			Event::EncounterError(_) => { self.errors.fetch_add(1, Relaxed); }
			_ => {}
		}
	}
}
```

`report` runs once per measured item from many threads at once, squarely in the hot path. Keep it allocation-free and lock-free, and use `Ordering::Relaxed` for counters as the built-in reporter does.

Implement `ParallelReporter` as well if your reporter owns threads. If not, mirror
`ErrorOnlyReporter` with `type DestructionError = Infallible`.

## The status board

`status_board::GLOBAL_STATUS_BOARD` is the shared handle to stderr that keeps a transient progress line from being scrambled by permanent messages.

* `temporary_message(text)` overwrites the current line, updating the progress counter in place.
* `permanent_message(text)` clears the transient line, prints, and moves to the next line.
* `clear_line(0)` erases the transient line, which is what to call before printing a chart.

Any code writing to stderr while a `ProgressAndErrorReporter` runs should go through the status board rather than `eprintln!`.
