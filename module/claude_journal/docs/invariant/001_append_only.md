# Append-Only

**Status**: Planned | **Since**: 1.3.0

## Description

Journal files are strictly append-only. `JournalWriter` opens the daily file in append mode, writes one JSON line, and closes. No existing content is ever read, modified, or overwritten by the write path. This invariant ensures that concurrent readers and writers cannot interfere — a reader iterating over lines will never see a modified or deleted line.

## Measurement

- **Threshold**: 0 non-append file operations in `JournalWriter` (measured by code review — no `seek`, `truncate`, `write` without `O_APPEND`, or `rename` of journal files)
- **Method**: `grep -n "seek\|truncate\|OpenOptions.*write\|OpenOptions.*create(" src/writer.rs` must return zero matches outside the `append()` path, which uses `OpenOptions::new().append(true).create(true)`

## Sources

- `src/writer.rs` — `JournalWriter::append()`
