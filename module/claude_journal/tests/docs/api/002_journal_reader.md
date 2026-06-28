# Test: API — JournalReader

Test case planning for [api/002_journal_reader.md](../../../docs/api/002_journal_reader.md). Tests validate `JournalReader::open()` infallibility, `query()` ordering, filter application, limit behavior, skip-on-corrupt, and metadata accessors.

**Source:** [api/002_journal_reader.md](../../../docs/api/002_journal_reader.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| AP-1 | `open()` accepts a non-existent directory; `query()` returns empty | Constructor |
| AP-2 | `query(default_filter)` returns all events in insertion order, oldest first | Query Order |
| AP-3 | `JournalFilter::since = Some(Duration)` excludes events older than cutoff | Filter Since |
| AP-4 | `JournalFilter::limit = Some(N)` caps results to N events | Filter Limit |
| AP-5 | Corrupt lines in JSONL are skipped; surrounding valid events returned | Corrupt Skip |
| AP-6 | `file_count()`, `total_bytes()`, `oldest_date()`, `newest_date()` reflect journal state | Accessors |

## Test Coverage Summary

- Constructor: 1 test (AP-1)
- Query Order: 1 test (AP-2)
- Filter Since: 1 test (AP-3)
- Filter Limit: 1 test (AP-4)
- Corrupt Skip: 1 test (AP-5)
- Accessors: 1 test (AP-6)

**Total:** 6 tests

---

### AP-1: `open()` on non-existent dir; `query()` returns empty

- **Given:** path to a directory that does not exist
- **When:** `JournalReader::open(missing_path).query(&JournalFilter::default())`
- **Then:** no panic; returns an empty `Vec<EventRecord>`
- **Source:** [api/002_journal_reader.md](../../../docs/api/002_journal_reader.md) Behavioral Contract: `open()` is infallible; missing dir → empty results

---

### AP-2: `query()` returns events in insertion order

- **Given:** 5 events written with distinct exit codes 0..5 via `JournalWriter`
- **When:** `reader.query(&JournalFilter::default())`
- **Then:** `events.len() == 5`; `events[i].fields.exit_code == Some(i as i32)` for i in 0..5 (oldest first = insertion order)
- **Source:** [api/002_journal_reader.md](../../../docs/api/002_journal_reader.md) Behavioral Contract: events in append order

---

### AP-3: `JournalFilter::since` excludes old events

- **Given:** journal with one event having `ts = "2000-01-01T00:00:00.000Z"` (old) and one with current `ts`
- **When:** `query(&JournalFilter { since: Some(Duration::from_secs(300)), ..Default::default() })`
- **Then:** returns exactly 1 event (the recent one); the old event is excluded
- **Source:** [api/002_journal_reader.md](../../../docs/api/002_journal_reader.md) Behavioral Contract: `since` filter

---

### AP-4: `JournalFilter::limit` caps result count

- **Given:** 10 events written to the journal
- **When:** `query(&JournalFilter { limit: Some(3), ..Default::default() })`
- **Then:** returns exactly 3 events (the first 3 in insertion order)
- **Source:** [api/002_journal_reader.md](../../../docs/api/002_journal_reader.md) Behavioral Contract: `limit` cap

---

### AP-5: Corrupt JSONL lines are skipped

- **Given:** journal file written directly with pattern: `valid_json\n{corrupt\nvalid_json\n`
- **When:** `reader.query(&JournalFilter::default())`
- **Then:** returns exactly 2 events; no panic; corrupt line silently skipped
- **Source:** [api/002_journal_reader.md](../../../docs/api/002_journal_reader.md) Behavioral Contract: skip on parse failure

---

### AP-6: Metadata accessors reflect actual journal state

- **Given:** journal dir with two files: `2023-01-01.jsonl` (older) and `2026-06-27.jsonl` (today)
- **When:** `reader.file_count()`, `reader.oldest_date()`, `reader.newest_date()`, `reader.total_bytes()`
- **Then:** `file_count() == 2`; `oldest_date() == Some("2023-01-01")`; `newest_date() == Some("2026-06-27")`; `total_bytes() > 0`
- **Source:** [api/002_journal_reader.md](../../../docs/api/002_journal_reader.md) Interface: `file_count()`, `total_bytes()`, `oldest_date()`, `newest_date()`
