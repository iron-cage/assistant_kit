# Test: Invariant — Crash Safety

Test case planning for [invariant/002_crash_safety.md](../../../docs/invariant/002_crash_safety.md). Tests validate that corrupt or partial JSONL lines are silently skipped and that preceding valid lines remain readable after a simulated crash.

**Source:** [invariant/002_crash_safety.md](../../../docs/invariant/002_crash_safety.md)
**Related:** [api/002_journal_reader.md](../../../docs/api/002_journal_reader.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | Corrupt JSONL line (malformed JSON) between two valid events → both valid events returned | Corrupt Line |
| IN-2 | Partial line (truncated mid-JSON) → skipped; preceding lines intact | Partial Write |
| IN-3 | Multiple corrupt lines each independently skipped; all valid events returned | Multi-Corrupt |
| IN-4 | Empty lines in JSONL file are skipped without error | Empty Line |

## Test Coverage Summary

- Corrupt Line: 1 test (IN-1)
- Partial Write: 1 test (IN-2)
- Multi-Corrupt: 1 test (IN-3)
- Empty Line: 1 test (IN-4)

**Total:** 4 invariant test cases

## Architectural Constraint

All tests write raw content directly to the journal file via `std::fs::write` (bypassing `JournalWriter`) to simulate crash conditions. Then `JournalReader::query(default_filter)` is called to verify recovery.

---

### IN-1: Corrupt line between valid events → valid events returned

- **Given:** journal file containing: `valid_ev1_json\n{bad json\nvalid_ev2_json\n`
- **When:** `JournalReader::open(dir).query(&JournalFilter::default())`
- **Then:** returns exactly 2 events; no panic; `events[0].event_type` matches ev1; `events[1].event_type` matches ev2
- **Source:** [invariant/002_crash_safety.md](../../../docs/invariant/002_crash_safety.md) Rule: at most 1 corrupted line per crash

---

### IN-2: Partial/truncated line → preceding lines intact

- **Given:** journal file containing: `valid_ev1_json\n{"v":1,"ts":"2026-` (truncated mid-write)
- **When:** `JournalReader::open(dir).query(&JournalFilter::default())`
- **Then:** returns exactly 1 event (ev1); the truncated partial line is skipped; no panic
- **Source:** [invariant/002_crash_safety.md](../../../docs/invariant/002_crash_safety.md) Rule: preceding lines remain intact

---

### IN-3: Multiple corrupt lines each independently skipped

- **Given:** journal file: `valid_ev1\ncorrupt1\ncorrupt2\nvalid_ev2\ncorrupt3\nvalid_ev3\n`
- **When:** `query(default_filter)`
- **Then:** returns exactly 3 events: ev1, ev2, ev3 in order; each corrupt line independently skipped
- **Source:** [invariant/002_crash_safety.md](../../../docs/invariant/002_crash_safety.md) Rule: crash-safe: skip on parse failure

---

### IN-4: Empty lines are skipped without error

- **Given:** journal file: `valid_ev1\n\n\nvalid_ev2\n` (two blank lines between events)
- **When:** `query(default_filter)`
- **Then:** returns exactly 2 events; blank lines produce no error and no spurious events
- **Source:** [invariant/002_crash_safety.md](../../../docs/invariant/002_crash_safety.md) Rule: skip on parse failure
