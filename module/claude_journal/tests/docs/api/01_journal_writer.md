# Test: API — JournalWriter

Test case planning for [api/001_journal_writer.md](../../../docs/api/001_journal_writer.md). Tests validate `JournalWriter::new()` infallibility, `append()` directory/file creation, open-write-close crash safety, concurrent thread safety, and `dir()` accessor.

**Source:** [api/001_journal_writer.md](../../../docs/api/001_journal_writer.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| AP-1 | `new()` is infallible — accepts a non-existent path without error | Constructor |
| AP-2 | `append()` creates the journal directory on first call | Dir Creation |
| AP-3 | `append()` creates today's `.jsonl` file and appends one JSON line | File Append |
| AP-4 | Multiple concurrent `append()` calls from N threads all succeed | Thread Safety |
| AP-5 | `dir()` returns the exact path passed to `new()` | Accessor |
| AP-6 | Appending to a file that already contains content adds a new line without disturbing existing content | Append Safety |

## Test Coverage Summary

- Constructor: 1 test (AP-1)
- Dir Creation: 1 test (AP-2)
- File Append: 1 test (AP-3)
- Thread Safety: 1 test (AP-4)
- Accessor: 1 test (AP-5)
- Append Safety: 1 test (AP-6)

**Total:** 6 tests

## Architectural Constraint

AP-4 uses `Arc<JournalWriter>` shared across 4 threads, each appending 25 events for 100 total. The test verifies that the final `query()` returns all 100 events and that no two JSON lines are interleaved (each line individually parses as valid JSON).

---

### AP-1: `new()` accepts a non-existent path without panicking

- **Given:** a path to a directory that does not yet exist
- **When:** `JournalWriter::new(non_existent_path)` — no directory creation expected at construction
- **Then:** returns a `JournalWriter` without error; the directory still does not exist (creation is deferred to `append()`)
- **Source:** [api/001_journal_writer.md](../../../docs/api/001_journal_writer.md) Behavioral Contract: `new()` is infallible

---

### AP-2: `append()` creates the journal directory if absent

- **Given:** `JournalWriter::new(dir)` where `dir` does not exist
- **When:** `writer.append(&EventRecord::new(EventType::Execution))`
- **Then:** `dir.exists() == true` after the call
- **Source:** [api/001_journal_writer.md](../../../docs/api/001_journal_writer.md) Behavioral Contract: directory creation

---

### AP-3: `append()` creates daily file and writes one JSON line

- **Given:** temporary journal dir
- **When:** `writer.append(&ev)` with one event
- **Then:** one `.jsonl` file exists in `dir`; file contains exactly one non-empty line; the line is valid JSON containing `"v":1` and a `"type"` field
- **Source:** [api/001_journal_writer.md](../../../docs/api/001_journal_writer.md) Interface: `append()` doc

---

### AP-4: Concurrent `append()` from multiple threads all succeed

- **Given:** `Arc<JournalWriter>` shared across 4 threads; each thread appends 25 events (100 total)
- **When:** all threads finish; `JournalReader::query(default_filter)` called
- **Then:** `events.len() == 100`; every line in the raw file individually parses as valid JSON (no interleaved partial lines)
- **Source:** [api/001_journal_writer.md](../../../docs/api/001_journal_writer.md) Behavioral Contract: `Send + Sync`

---

### AP-5: `dir()` returns the configured path

- **Given:** `JournalWriter::new(PathBuf::from("/tmp/test_journal"))`
- **When:** `writer.dir()`
- **Then:** returns `Path` equal to `/tmp/test_journal`
- **Source:** [api/001_journal_writer.md](../../../docs/api/001_journal_writer.md) Interface: `dir()` method

---

### AP-6: Second `append()` adds a new line without disturbing first

- **Given:** journal file with one pre-written event (ev1)
- **When:** `writer.append(&ev2)` (second event, different `event_type`)
- **Then:** file has exactly 2 lines; `serde_json::from_str(line1)` round-trips to ev1; `serde_json::from_str(line2)` round-trips to ev2
- **Source:** [api/001_journal_writer.md](../../../docs/api/001_journal_writer.md) Behavioral Contract: append-only, existing content not modified
