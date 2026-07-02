# Test: Feature — Event Journaling

Test case planning for [feature/001_event_journaling.md](../../../docs/feature/001_event_journaling.md). Tests validate file creation, level control (full/meta/off), directory auto-creation, concurrent read safety, and stdout/stderr truncation.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | `append()` creates directory and `.jsonl` file on first call | File Creation |
| FT-2 | At `full` level, stdout and stderr fields are populated | Level Full |
| FT-3 | At `meta` level, stdout and stderr fields are None | Level Divergence |
| FT-4 | At `off` level, `append()` is never called (no-op writer) | Level Divergence |
| FT-5 | stdout field truncated at 1 MB with `\n[truncated at 1MB]` suffix | Truncation |
| FT-6 | Concurrent `JournalReader::query()` while `JournalWriter::append()` is active does not corrupt either | Concurrent Safety |

## Test Coverage Summary

- File Creation: 1 test (FT-1)
- Level Full: 1 test (FT-2)
- Level Divergence: 2 tests (FT-3, FT-4)
- Truncation: 1 test (FT-5)
- Concurrent Safety: 1 test (FT-6)

**Total:** 6 tests

## Architectural Constraint

FT-4 tests a higher-level concept (the `JournalLevel::Off` no-op) that is part of the `claude_runner` integration layer, not the `claude_journal` library itself. The library always appends when `append()` is called. FT-4 is therefore a `claude_runner` integration test; at the `claude_journal` library level it is validated by checking that calling `append()` on a fresh dir always creates a file.

FT-6 uses two threads: one calling `append()` in a tight loop (100 iterations) and the main thread calling `query()` after 50 appends. The test asserts that `query()` returns a valid set of events with no panics.

---

### FT-1: `append()` creates dir and daily file on first call

- **Given:** a temporary directory path that does not yet exist (non-existent parent subpath)
- **When:** `JournalWriter::new(dir).append(&EventRecord::new(EventType::Execution))`
- **Then:** `dir` exists; exactly one `.jsonl` file in `dir`; the file contains one non-empty line parseable as JSON
- **Source:** [feature/001_event_journaling.md](../../../docs/feature/001_event_journaling.md) AC-001, AC-010

---

### FT-2: At `full` level, stdout and stderr fields populated

- **Given:** an `EventRecord` with `fields.stdout = Some("output")` and `fields.stderr = Some("err")`
- **When:** writer appends the event; reader queries it back
- **Then:** returned event has `fields.stdout == Some("output")` and `fields.stderr == Some("err")`
- **Source:** [feature/001_event_journaling.md](../../../docs/feature/001_event_journaling.md) AC-005

---

### FT-3: At `meta` level, stdout and stderr are None

- **Given:** an `EventRecord` with `fields.stdout = None` and `fields.stderr = None` (meta-level record — no output)
- **When:** writer appends; reader queries
- **Then:** returned event has `fields.stdout == None` and `fields.stderr == None`
- **Source:** [feature/001_event_journaling.md](../../../docs/feature/001_event_journaling.md) AC-007

---

### FT-4: Writer creates file; content is always a valid JSON line

- **Given:** temporary journal dir
- **When:** `JournalWriter::new(dir).append(&EventRecord::new(EventType::Execution))`
- **Then:** file contains one line; `serde_json::from_str::<serde_json::Value>(line)` succeeds — confirms AC-002 (self-contained JSON line)
- **Source:** [feature/001_event_journaling.md](../../../docs/feature/001_event_journaling.md) AC-002

---

### FT-5: stdout truncated at 1 MB with suffix

- **Given:** an `EventRecord` with `fields.stdout = Some(s)` where `s` is 1_100_000 bytes of `'A'`
- **When:** writer appends; reader queries back
- **Then:** `fields.stdout` is `Some(t)` where `t.ends_with("\n[truncated at 1MB]")` and `t.len() <= 1_100_000`
- **Source:** [feature/001_event_journaling.md](../../../docs/feature/001_event_journaling.md) AC-006

---

### FT-6: Concurrent read during write does not corrupt

- **Given:** temporary journal dir; writer thread appending 100 events; reader on main thread
- **When:** main thread calls `query(default_filter)` after 50 appends; threads join; final `query()` called
- **Then:** no panics during concurrent access; final `query()` returns all 100 events; all events parse correctly
- **Source:** [feature/001_event_journaling.md](../../../docs/feature/001_event_journaling.md) AC-009
