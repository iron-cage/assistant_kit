# Test: Invariant — Append-Only

Test case planning for [invariant/001_append_only.md](../../../docs/invariant/001_append_only.md). Tests validate that `JournalWriter` uses only append-mode file opens and that no existing journal content is ever modified.

**Source:** [invariant/001_append_only.md](../../../docs/invariant/001_append_only.md)
**Related:** [api/001_journal_writer.md](../../../docs/api/001_journal_writer.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | Source code: `src/writer.rs` contains no `write(true)`, `create_new(true)`, `truncate(true)`, `fs::write`, or `File::create` calls | Structural |
| IN-2 | Two sequential `append()` calls both persist — earlier line is not overwritten | Behavioral |
| IN-3 | Pre-existing file content is preserved after `append()` | Behavioral |

## Test Coverage Summary

- Structural: 1 test (IN-1)
- Behavioral: 2 tests (IN-2, IN-3)

**Total:** 3 invariant test cases

## Architectural Constraint

IN-1 is a structural test (source code scan): read `src/writer.rs` as a string and assert it does NOT contain forbidden patterns. This prevents the invariant from being silently violated in future refactors.

IN-2 and IN-3 are integration tests that use `JournalWriter` + `JournalReader` directly.

---

### IN-1: `src/writer.rs` contains only append-mode file opens

- **Given:** source file `src/writer.rs` read as a string
- **When:** scan for forbidden patterns: `write(true)`, `create_new(true)`, `truncate(true)`, `fs::write(`, `File::create(`
- **Then:** none of the forbidden patterns appear in the source; `append(true)` is present — confirms `OpenOptions::new().append(true).create(true)` pattern
- **Source:** [invariant/001_append_only.md](../../../docs/invariant/001_append_only.md) Measurement

---

### IN-2: Two sequential appends both persist

- **Given:** temporary journal dir; two distinct `EventRecord` instances with different `event_type`
- **When:** `writer.append(&ev1)` then `writer.append(&ev2)`; `reader.query(default_filter)`
- **Then:** `events.len() == 2`; `events[0].event_type == ev1.event_type`; `events[1].event_type == ev2.event_type`
- **Source:** [invariant/001_append_only.md](../../../docs/invariant/001_append_only.md) Rule: 0 non-append file operations

---

### IN-3: Pre-existing file content preserved after append

- **Given:** journal file with one pre-written event line; `JournalWriter` constructed on same dir
- **When:** `writer.append(&new_ev)` called
- **Then:** file has two lines; first line is unchanged (original event); second line is the new event
- **Source:** [invariant/001_append_only.md](../../../docs/invariant/001_append_only.md) Rule: existing content never modified
