# Test: Feature — Journaling Integration

Test case planning for [feature/002_journaling_integration.md](../../../../docs/feature/002_journaling_integration.md). Tests validate journaling emission at execution boundaries, level control (full/meta/off), directory resolution, truncation, and error isolation.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | `--journal full` (default) — event written with stdout field populated | Level Default |
| FT-2 | `--journal meta` — event written without stdout/stderr fields | Level Divergence |
| FT-3 | `--journal off` — no event file created in journal dir | Level Divergence |
| FT-4 | No `--journal` flag — journals at `full` level by default (AC-004) | Default Behavior |
| FT-5 | `--journal-dir /tmp/j` — event written to `/tmp/j/YYYY-MM-DD.jsonl` | Directory Override |
| FT-6 | `CLR_JOURNAL_DIR=/tmp/j` env var — same directory effect as CLI flag | Env Var |
| FT-7 | Journal write failure (read-only path) — `clr` exit code unchanged | Error Isolation |
| FT-8 | Stdout exceeding 1 MB → field truncated with `[truncated at 1MB]` marker | Truncation |

## Test Coverage Summary

- Level Default: 1 test (FT-1)
- Level Divergence: 2 tests (FT-2, FT-3)
- Default Behavior: 1 test (FT-4)
- Directory Override: 1 test (FT-5)
- Env Var: 1 test (FT-6)
- Error Isolation: 1 test (FT-7)
- Truncation: 1 test (FT-8)

**Total:** 8 tests

## Architectural Constraint

FT-1 through FT-6 require a fake `claude` subprocess and a temporary directory as the journal path. All tests must set `--journal-dir <tmpdir>` (or `CLR_JOURNAL_DIR=<tmpdir>`) so that journal events land in an isolated temp directory, not `~/.clr/journal/`.

FT-7 requires a read-only directory created via `std::fs::set_permissions`. The test asserts that `clr` exits 0 despite the journal write failing.

FT-8 requires a fake `claude` subprocess that emits >1 MB of repeated output on stdout. The test reads the journal event and asserts the `stdout` field ends with `\n[truncated at 1MB]`.

---

### FT-1: `--journal full` → event with stdout field

- **Given:** temporary journal dir; fake claude that exits 0 and prints `"hello"`
- **When:** `clr -p --max-sessions 0 --journal full --journal-dir <tmpdir> "task"`
- **Then:** journal file `<tmpdir>/YYYY-MM-DD.jsonl` exists; last line parses as JSON; `event.fields.stdout` is `Some("hello")`; `event.event_type == EventType::Execution`
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../../docs/feature/002_journaling_integration.md) AC-001

---

### FT-2: `--journal meta` → event without stdout/stderr

- **Given:** temporary journal dir; fake claude that exits 0 and prints `"hello"`
- **When:** `clr -p --max-sessions 0 --journal meta --journal-dir <tmpdir> "task"`
- **Then:** journal file exists; last line parses as JSON; `event.fields.stdout` is `None`; `event.fields.stderr` is `None`; `event.fields.exit_code` is `Some(0)` (metadata present)
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../../docs/feature/002_journaling_integration.md) AC-002

---

### FT-3: `--journal off` → no journal event

- **Given:** temporary journal dir; fake claude that exits 0
- **When:** `clr -p --max-sessions 0 --journal off --journal-dir <tmpdir> "task"`
- **Then:** `<tmpdir>` either does not exist or contains no `.jsonl` files (journaling disabled)
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../../docs/feature/002_journaling_integration.md) AC-003

---

### FT-4: No `--journal` flag → defaults to `full`

- **Given:** temporary journal dir; fake claude that exits 0 and prints `"result"`
- **When:** `clr -p --max-sessions 0 --journal-dir <tmpdir> "task"` (no --journal flag)
- **Then:** journal file exists; last line parses as JSON; `event.fields.stdout` is `Some("result")` — confirms default level is `full`
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../../docs/feature/002_journaling_integration.md) AC-004

---

### FT-5: `--journal-dir <path>` → events written to specified path

- **Given:** two temporary directories: `dir_a` and `dir_b`; fake claude exits 0
- **When:** `clr -p --max-sessions 0 --journal-dir <dir_a> "task"`
- **Then:** `<dir_a>/YYYY-MM-DD.jsonl` exists and contains one event line; `<dir_b>` has no `.jsonl` files
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../../docs/feature/002_journaling_integration.md) AC-005

---

### FT-6: `CLR_JOURNAL_DIR` env var routes journal events

- **Given:** temporary journal dir; env var `CLR_JOURNAL_DIR=<tmpdir>`; fake claude exits 0
- **When:** `clr -p --max-sessions 0 "task"` with `CLR_JOURNAL_DIR` set (no `--journal-dir` flag)
- **Then:** `<tmpdir>/YYYY-MM-DD.jsonl` exists and contains one event line
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../../docs/feature/002_journaling_integration.md) AC-006

---

### FT-7: Journal write failure does not change exit code

- **Given:** read-only journal dir (`0o555` permissions); fake claude exits 0
- **When:** `clr -p --max-sessions 0 --journal-dir <readonly_dir> "task"`
- **Then:** exit 0 (clr exit code unchanged by journal write failure); journal event may or may not be present (best-effort)
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../../docs/feature/002_journaling_integration.md) AC-008

---

### FT-8: Stdout exceeding 1 MB is truncated in journal

- **Given:** temporary journal dir; fake claude that emits >1 MB on stdout (repeated `'A'` × 1_100_000)
- **When:** `clr -p --max-sessions 0 --journal full --journal-dir <tmpdir> "task"`
- **Then:** journal event `fields.stdout` is `Some(s)` where `s.ends_with("\n[truncated at 1MB]")` and `s.len() <= 1_100_000` (truncated to 1 MB + suffix)
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../../docs/feature/002_journaling_integration.md) AC-007
