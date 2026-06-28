# Invariant: Append-Only

### Scope

- **Purpose**: Guarantee that journal files are strictly append-only, preventing concurrent-access corruption.
- **Responsibility**: State the file-open contract for `JournalWriter`, enumerate prohibited file operations, and explain why the constraint ensures reader/writer safety.
- **In Scope**: `JournalWriter` file-open mode, prohibited operations (`seek`, `truncate`, file rename), concurrent reader/writer safety.
- **Out of Scope**: Crash durability (→ `invariant/002_crash_safety.md`), schema versioning (→ `invariant/003_schema_version.md`).

### Invariant Statement

Journal files are strictly append-only. `JournalWriter` opens the daily file in append mode, writes one JSON line, and closes. No existing content is ever read, modified, or overwritten by the write path.

This invariant ensures that concurrent readers and writers cannot interfere — a reader iterating over lines will never see a modified or deleted line.

### Measurement

- **Threshold**: 0 non-append file operations in `JournalWriter` (measured by code review)
- **Method**: `grep -n "seek\|truncate\|OpenOptions.*write\|OpenOptions.*create(" src/writer.rs` must return zero matches outside the `append()` path, which uses `OpenOptions::new().append(true).create(true)`

### Violation Consequences

- Adding a `seek` or `truncate` call allows a writer to corrupt lines a reader has already partially consumed
- Opening without `O_APPEND` causes interleaved writes from concurrent processes to overwrite each other
- Renaming the daily file mid-write breaks reader iterators that hold an open file descriptor

### Sources

| File | Relationship |
|------|--------------|
| `src/writer.rs` | `JournalWriter::append()` — sole write path |
