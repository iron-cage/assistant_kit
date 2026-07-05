# JournalWriter

**Status**: Planned | **Since**: 1.3.0

### Scope

- **Purpose**: Provide a crash-safe, append-only writer for recording structured journal events to daily JSONL files.
- **Responsibility**: Documents the `JournalWriter` struct, its `new()`/`append()`/`dir()` operations, and their behavioral contract.
- **In Scope**: Directory/file creation on first write, per-event JSON serialization, and thread-safety guarantees for concurrent appends.
- **Out of Scope**: Event schema definition (→ `docs/api/003_event_type.md`), reading/querying journal data (→ `docs/api/002_journal_reader.md`).

## Description

Append-only writer that records structured events to daily JSONL files. Each `append()` call opens the current day's file (creating it and the directory if absent), serializes the `EventRecord` to a single JSON line, writes it with a trailing newline, and closes the file handle. The open-write-close pattern makes each append crash-safe — no state is held between calls.

## Interface

```rust
pub struct JournalWriter
{
  dir : std::path::PathBuf,
}

impl JournalWriter
{
  /// Create a writer targeting `dir`. Does not create the directory until first `append()`.
  pub fn new( dir : std::path::PathBuf ) -> Self;

  /// Append one event to today's journal file (`YYYY-MM-DD.jsonl`).
  ///
  /// Creates `dir` and the daily file if absent.
  /// Opens in append mode — existing content is never modified.
  /// Serializes `event` to a single JSON line terminated by `\n`.
  /// Returns `Err` on I/O failure (permission denied, disk full).
  pub fn append( &self, event : &EventRecord ) -> std::io::Result< () >;

  /// Return the configured journal directory.
  pub fn dir( &self ) -> &std::path::Path;
}
```

## Behavioral Contract

- `new()` is infallible — directory existence is not checked until `append()`
- `append()` is idempotent for the directory creation: multiple concurrent calls on a fresh directory all succeed
- The daily filename uses UTC date: `chrono::Utc::now().format("%Y-%m-%d").to_string() + ".jsonl"` (or equivalent without chrono dependency)
- Each JSON line is a complete, self-contained record — no multi-line output, no file-level header/footer
- Thread safety: `JournalWriter` is `Send + Sync`; concurrent `append()` calls from different threads serialize at the OS file-append level

## Sources

- `src/writer.rs` — implementation
- `docs/feature/001_event_journaling.md` — acceptance criteria AC-001, AC-009, AC-010
- `docs/invariant/001_append_only.md` — append-only constraint
