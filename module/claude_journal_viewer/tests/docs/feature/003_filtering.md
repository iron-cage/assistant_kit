# Test: Feature — Filtering

Test case planning for [feature/003_filtering.md](../../../docs/feature/003_filtering.md). Tests validate empty filter (match-all), AND combination, time filter parsing, substring model matching, exact exit code matching, invalid input errors, and limit behavior.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | Empty filter matches all events | Match-All |
| FT-2 | `type::execution command::ask` combines as AND — only ask-execution events | AND Combination |
| FT-3 | `since::1h` parses as "last 60 minutes" and excludes older events | Time Filter |
| FT-4 | `model::opus` uses substring match: includes `claude-opus-4-6`, excludes `claude-sonnet-4-6` | Substring Match |
| FT-5 | `exit::0` matches only exit-0 events; `exit::2` matches only exit-2 events | Exact Exit Match |
| FT-6 | Invalid `since::abc` → error message on stderr; exit 1 | Invalid Duration |
| FT-7 | Invalid `type::bogus` → error message on stderr; exit 1 | Invalid Type |
| FT-8 | `limit::3` caps output to 3 events regardless of total matches | Limit |

## Test Coverage Summary

- Match-All: 1 test (FT-1)
- AND Combination: 1 test (FT-2)
- Time Filter: 1 test (FT-3)
- Substring Match: 1 test (FT-4)
- Exact Exit Match: 1 test (FT-5)
- Invalid Duration: 1 test (FT-6)
- Invalid Type: 1 test (FT-7)
- Limit: 1 test (FT-8)

**Total:** 8 tests

## Architectural Constraint

All tests use `clj .list --journal-dir <dir>` (or the applicable viewing command) as the CLI under test. The journal is pre-populated with controlled events written by `claude_journal::JournalWriter`.

FT-3 uses an event with `ts = "2000-01-01T00:00:00.000Z"` to represent "definitely old"; the filter `since::1h` must exclude it.

FT-4 requires two events: one with `fields.model = Some("claude-opus-4-6")` and one with `fields.model = Some("claude-sonnet-4-6")`.

---

### FT-1: Empty filter matches all events

- **Given:** journal with 5 events of mixed types
- **When:** `clj .list --journal-dir <dir>` (no filter params)
- **Then:** exit 0; output contains all 5 events
- **Source:** [feature/003_filtering.md](../../../docs/feature/003_filtering.md) AC-001

---

### FT-2: AND combination: `type::execution command::ask`

- **Given:** journal with: 3 execution/ask events, 2 execution/run events, 2 retry events
- **When:** `clj .list type::execution command::ask --journal-dir <dir>`
- **Then:** exit 0; output shows exactly 3 events; no retry or run-execution events visible
- **Source:** [feature/003_filtering.md](../../../docs/feature/003_filtering.md) AC-002

---

### FT-3: `since::1h` excludes events older than 60 minutes

- **Given:** journal with one old event (`ts = "2000-01-01T00:00:00.000Z"`) and one recent event
- **When:** `clj .list since::1h --journal-dir <dir>`
- **Then:** exit 0; output contains exactly 1 event (the recent one); old event absent
- **Source:** [feature/003_filtering.md](../../../docs/feature/003_filtering.md) AC-003

---

### FT-4: `model::opus` uses substring match

- **Given:** journal with event A (`model = "claude-opus-4-6"`) and event B (`model = "claude-sonnet-4-6"`)
- **When:** `clj .list model::opus --journal-dir <dir>`
- **Then:** exit 0; event A present; event B absent
- **Source:** [feature/003_filtering.md](../../../docs/feature/003_filtering.md) AC-004

---

### FT-5: `exit::0` and `exit::2` use exact matching

- **Given:** journal with exit-0 event, exit-1 event, exit-2 event
- **When:** `clj .list exit::0 --journal-dir <dir>` → 1 event; `clj .list exit::2 --journal-dir <dir>` → 1 event
- **Then:** each query returns exactly 1 event matching the given exit code
- **Source:** [feature/003_filtering.md](../../../docs/feature/003_filtering.md) AC-005

---

### FT-6: Invalid duration `since::abc` → error and exit 1

- **Given:** any journal dir
- **When:** `clj .list since::abc --journal-dir <dir>`
- **Then:** exit 1; stderr contains an error message about invalid duration format
- **Source:** [feature/003_filtering.md](../../../docs/feature/003_filtering.md) AC-006

---

### FT-7: Invalid type `type::bogus` → error and exit 1

- **Given:** any journal dir
- **When:** `clj .list type::bogus --journal-dir <dir>`
- **Then:** exit 1; stderr contains an error message about unknown event type
- **Source:** [feature/003_filtering.md](../../../docs/feature/003_filtering.md) AC-007

---

### FT-8: `limit::3` caps output to 3 events

- **Given:** journal with 10 events
- **When:** `clj .list limit::3 --journal-dir <dir>`
- **Then:** exit 0; output contains exactly 3 events
- **Source:** [feature/003_filtering.md](../../../docs/feature/003_filtering.md) AC-008
