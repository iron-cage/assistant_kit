# Test: Feature — CLI Viewing

### Scope

- **Purpose**: FT- test cases verifying CLI command dispatch, listing, tailing, stats, export, status, and prune behaviors.
- **Responsibility**: Acceptance criteria confirming all 8 viewer commands are registered and produce correct output against a populated journal.
- **In Scope**: Command registration/dispatch, `.list` default and filtered output, `.tail` streaming, `.stats` aggregation, `.export`, `.status`, `.prune`.
- **Out of Scope**: HTTP server and web UI (-> `002_web_viewing.md`), detailed filter semantics (-> `003_filtering.md`), read-only invariant (-> `../invariant/001_read_only.md`).

Test case planning for [feature/001_cli_viewing.md](../../../docs/feature/001_cli_viewing.md). Tests validate command dispatch, default `.list` output, filtering, `.tail` streaming, `.stats` aggregation, and `.export` file output.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | All 8 commands registered — dispatch exits 0 with known output for `--help` | Command Registration |
| FT-2 | `.list` with no params shows ≤50 most recent events in table format | Default List |
| FT-3 | `.list since::1h type::execution` returns only execution events from last hour | Filtered List |
| FT-4 | `.tail` emits new events as they are appended; terminates on SIGTERM | Tail Streaming |
| FT-5 | `.stats` without params shows daily aggregates for last 7 days | Stats Aggregation |
| FT-6 | `.export format::csv output::/tmp/events.csv` writes filtered events to file | Export |
| FT-7 | `.status` reports file count, total bytes, oldest/newest dates, journal dir | Status |
| FT-8 | `.prune keep::30d` deletes files older than 30 days and reports count | Prune |

## Test Coverage Summary

- Command Registration: 1 test (FT-1)
- Default List: 1 test (FT-2)
- Filtered List: 1 test (FT-3)
- Tail Streaming: 1 test (FT-4)
- Stats Aggregation: 1 test (FT-5)
- Export: 1 test (FT-6)
- Status: 1 test (FT-7)
- Prune: 1 test (FT-8)

**Total:** 8 tests

## Architectural Constraint

FT-2, FT-3, FT-5, FT-6 require a pre-populated journal directory created via `claude_journal::JournalWriter`. Events need a mix of types and timestamps to exercise filtering.

FT-4 requires background thread writing; the test spawns `clj .tail --journal-dir <tmpdir>` as a subprocess, appends one event from another thread, reads stdout with a timeout, and asserts the event appears.

FT-8 requires synthetic journal files with known dates (past files) to verify prune deletes the right ones.

---

### FT-1: All 8 commands registered and dispatch correctly

- **Given:** journal dir with some events
- **When:** `clj --help` (or invoking each command with `--help`)
- **Then:** exit 0; output references `.list`, `.tail`, `.stats`, `.search`, `.serve`, `.prune`, `.status`, `.export`
- **Source:** [feature/001_cli_viewing.md](../../../docs/feature/001_cli_viewing.md) AC-001

---

### FT-2: `.list` with no params shows ≤50 most recent events

- **Given:** journal with 60 events
- **When:** `clj .list --journal-dir <dir>`
- **Then:** exit 0; output contains exactly 50 rows (most recent 50); table columns include TIME, CMD or TYPE column
- **Source:** [feature/001_cli_viewing.md](../../../docs/feature/001_cli_viewing.md) AC-002

---

### FT-3: `.list` with `since::` and `type::` filters

- **Given:** journal with 10 execution events (recent) and 5 retry events (recent) and 10 old execution events
- **When:** `clj .list since::1h type::execution --journal-dir <dir>`
- **Then:** exit 0; output shows only the 10 recent execution events; retry events absent; old execution events absent
- **Source:** [feature/001_cli_viewing.md](../../../docs/feature/001_cli_viewing.md) AC-003

---

### FT-4: `.tail` emits new events as they appear

- **Given:** journal dir; `clj .tail --journal-dir <dir>` started as background subprocess
- **When:** `JournalWriter::append(&new_ev)` from another thread 500ms after tail starts
- **Then:** within 3s, the new event line appears in tail's stdout; tail does not exit on its own
- **Source:** [feature/001_cli_viewing.md](../../../docs/feature/001_cli_viewing.md) AC-004

---

### FT-5: `.stats` shows daily aggregates

- **Given:** journal with events spread across 3 different UTC days (synthetic dates)
- **When:** `clj .stats --journal-dir <dir>`
- **Then:** exit 0; output contains one row per day; each row shows event count and/or cost aggregates
- **Source:** [feature/001_cli_viewing.md](../../../docs/feature/001_cli_viewing.md) AC-005

---

### FT-6: `.export format::csv` writes to file

- **Given:** journal with 5 events; output file path `/tmp/clj_test_events.csv`
- **When:** `clj .export format::csv output::/tmp/clj_test_events.csv --journal-dir <dir>`
- **Then:** exit 0; `/tmp/clj_test_events.csv` exists; contains at least one data row and one header row
- **Source:** [feature/001_cli_viewing.md](../../../docs/feature/001_cli_viewing.md) AC-009

---

### FT-7: `.status` reports journal health

- **Given:** journal dir with 3 `.jsonl` files
- **When:** `clj .status --journal-dir <dir>`
- **Then:** exit 0; stdout contains file count `3`; shows directory path; shows oldest/newest dates
- **Source:** [feature/001_cli_viewing.md](../../../docs/feature/001_cli_viewing.md) AC-008

---

### FT-8: `.prune keep::30d` deletes old files

- **Given:** journal dir with `2020-01-01.jsonl` (old) and today's file
- **When:** `clj .prune keep::30d --journal-dir <dir>`
- **Then:** exit 0 after confirmation; `2020-01-01.jsonl` deleted; today's file present; stdout reports 1 file deleted
- **Source:** [feature/001_cli_viewing.md](../../../docs/feature/001_cli_viewing.md) AC-007
