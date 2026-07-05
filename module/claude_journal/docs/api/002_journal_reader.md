# JournalReader

**Status**: Planned | **Since**: 1.3.0

### Scope

- **Purpose**: Provide a read-side API for querying and tailing journal events across daily JSONL files.
- **Responsibility**: Documents the `JournalReader`/`JournalFilter` types, their `open()`/`query()`/`tail()` and metadata operations, and their behavioral contract.
- **In Scope**: Chronological file iteration, AND-combined filter matching, blocking tail semantics, and parse-failure tolerance.
- **Out of Scope**: Event schema definition (→ `docs/api/003_event_type.md`), writing journal events (→ `docs/api/001_journal_writer.md`).

## Description

Read-side API for querying and tailing journal events. Opens a journal directory and iterates over daily JSONL files in chronological order, applying a `JournalFilter` to select matching events. Supports both batch query (returns all matching events) and streaming tail (watches for new events).

## Interface

```rust
pub struct JournalReader
{
  dir : std::path::PathBuf,
}

pub struct JournalFilter
{
  pub since : Option< std::time::Duration >,
  pub until : Option< std::time::SystemTime >,
  pub event_type : Option< EventType >,
  pub command : Option< String >,
  pub exit_code : Option< i32 >,
  pub model : Option< String >,
  pub dir : Option< String >,
  pub creds : Option< String >,
  pub limit : Option< usize >,
}

impl JournalReader
{
  /// Open a journal directory for reading.
  pub fn open( dir : std::path::PathBuf ) -> Self;

  /// Return all events matching `filter`, newest last.
  ///
  /// Reads only daily files within the `since..until` date range.
  /// Skips lines that fail JSON parse (crash-safety tolerance).
  /// Applies all non-None filter fields as AND conditions.
  /// Stops after `filter.limit` matches if set.
  pub fn query( &self, filter : &JournalFilter ) -> Vec< EventRecord >;

  /// Stream new events matching `filter` as they are appended.
  ///
  /// Polls the current day's file for new lines at ~500ms intervals.
  /// Rolls over to the next day's file at UTC midnight.
  /// Skips lines that fail JSON parse.
  pub fn tail( &self, filter : &JournalFilter ) -> impl Iterator< Item = EventRecord >;

  /// Count of `.jsonl` files in the journal directory.
  pub fn file_count( &self ) -> usize;

  /// Total bytes across all `.jsonl` files.
  pub fn total_bytes( &self ) -> u64;

  /// Date of the oldest journal file (from filename, not content).
  pub fn oldest_date( &self ) -> Option< String >;

  /// Date of the newest journal file (from filename, not content).
  pub fn newest_date( &self ) -> Option< String >;
}
```

## Behavioral Contract

- `open()` is infallible — missing directory is detected on first `query()`/`tail()` (returns empty/no events)
- `query()` reads files in date order (oldest first); events within a file are in append order (oldest first)
- `tail()` is a blocking iterator — it yields events as they appear and does not return until dropped
- Filter fields are AND-combined: all non-None fields must match for an event to be included
- `model` and `dir` filters use substring matching (case-sensitive)
- `command` and `event_type` filters use exact matching
- Lines that fail JSON parse are silently skipped (crash-safety tolerance per `docs/invariant/002_crash_safety.md`)

## Sources

- `src/reader.rs` — implementation
- `docs/invariant/002_crash_safety.md` — skip-on-parse-failure behavior
