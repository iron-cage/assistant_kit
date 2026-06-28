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
| FT-9 | `--journal-dir <cli>` + `CLR_JOURNAL_DIR=<env>` → file in CLI dir (CLI wins) | Precedence |
| FT-10 | Gate wait event emitted when `wait_for_session_slot()` blocks | Gate Emission |
| FT-11 | Validation retry event emitted on expect-strategy retry | Validation Emission |
| FT-12 | `--dry-run` does NOT create journal directory (BUG-319) | Side Effect Isolation |
| FT-13 | `--journal bogus` CLI flag → exit 1 with error | Validation |
| FT-14 | `--journal Full` (wrong case) → exit 1 | Validation |
| FT-15 | `--journal` missing value → exit 1 | Validation |
| FT-16 | `--journal full --journal meta` (last wins) → meta-level | Duplicate Handling |
| FT-17 | `--journal off --journal-dir <dir>` → no JSONL | Off Precedence |
| FT-18 | `CLR_JOURNAL=off` + `CLR_JOURNAL_DIR=<dir>` → no JSONL | Off Precedence |

## Test Coverage Summary

- Level Default: 1 test (FT-1)
- Level Divergence: 2 tests (FT-2, FT-3)
- Default Behavior: 1 test (FT-4)
- Directory Override: 1 test (FT-5)
- Env Var: 1 test (FT-6)
- Error Isolation: 1 test (FT-7)
- Truncation: 1 test (FT-8)
- Precedence: 1 test (FT-9)
- Gate Emission: 1 test (FT-10)
- Validation Emission: 1 test (FT-11)
- Side Effect Isolation: 1 test (FT-12)
- Validation: 3 tests (FT-13, FT-14, FT-15)
- Duplicate Handling: 1 test (FT-16)
- Off Precedence: 2 tests (FT-17, FT-18)

**Total:** 18 tests

> **Implementation note:** The actual test file uses EC-N identifiers (EC-1..EC-22)
> mapped to integration test scenarios. FT-N here is the spec-level identifier.
> CLI-wins-over-env is implemented as `ec14_journal_dir_cli_wins_over_env`;
> truncation marker as `ec15_stdout_over_1mb_has_truncation_marker`;
> gate_wait emission as `ec11_gate_wait_event_emitted_when_gate_blocks`;
> validation_retry emission as `ec12_validation_retry_event_emitted_on_expect_mismatch`.

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

---

### FT-9: `--journal-dir` CLI flag wins over `CLR_JOURNAL_DIR` env var

- **Given:** two temporary directories (`cli_dir`, `env_dir`); fake claude exits 0; `CLR_JOURNAL_DIR=<env_dir>` set
- **When:** `clr -p --max-sessions 0 --journal-dir <cli_dir> "task"` with `CLR_JOURNAL_DIR=<env_dir>`
- **Then:** JSONL file appears in `<cli_dir>`; `<env_dir>` contains no `.jsonl` files (CLI flag takes precedence)
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../../docs/feature/002_journaling_integration.md) design — "Resolution: CLI > env > default"

---

### FT-10: Gate wait event emitted when `wait_for_session_slot()` blocks

- **Given:** ELF fake `claude` binary holding 1 gate slot for ~3 s; separate script fake for the actual subprocess; temporary journal dir
- **When:** `clr -p --max-sessions 1 --journal full --journal-dir <tmpdir> "x"` with `_CLR_GATE_POLL_SECS=1`
- **Then:** JSONL contains a line with `"type":"gate_wait"` and `"gate_outcome":"acquired"`; `clr` exits 0 once gate releases
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../../docs/feature/002_journaling_integration.md) AC-009

---

### FT-11: Validation retry event emitted on expect-strategy retry

- **Given:** counter-script fake `claude` (first call prints `WRONG`, second prints `RIGHT`); temporary journal dir
- **When:** `clr -p --max-sessions 0 --expect right --expect-strategy retry --retry-on-validation 1 --validation-delay 0 --journal full --journal-dir <tmpdir> "x"`
- **Then:** JSONL contains a line with `"type":"validation_retry"` (emitted before the re-attempt); second attempt matches `"right"`; `clr` exits 0
- **Exit:** 0
- **Source:** [feature/002_journaling_integration.md](../../../../docs/feature/002_journaling_integration.md) AC-013

---

### FT-12: `--dry-run` does NOT create journal directory (BUG-319)

- **Given:** parent temp dir with a non-existent subdirectory `must_not_exist`
- **When:** `clr --dry-run --journal-dir <parent>/must_not_exist "test"`
- **Then:** `<parent>/must_not_exist` does NOT exist on disk; dry-run output shown on stdout
- **Exit:** 0
- **Source:** BUG-319 regression guard

---

### FT-13: `--journal bogus` CLI flag exits 1

- **Given:** no special setup
- **When:** `clr --dry-run --journal bogus "test"`
- **Then:** exit 1; stderr contains `--journal` and `bogus`
- **Exit:** 1
- **Source:** [param/072_journal.md](../../../../docs/cli/param/072_journal.md) — valid values: full, meta, off

---

### FT-14: `--journal Full` (case-sensitive) exits 1

- **Given:** no special setup
- **When:** `clr --dry-run --journal Full "test"` (also: FULL, Meta, META, Off, OFF)
- **Then:** exit 1 for each case variant; only lowercase accepted
- **Exit:** 1
- **Source:** [param/072_journal.md](../../../../docs/cli/param/072_journal.md) — enum values are lowercase only

---

### FT-15: `--journal` missing value exits 1

- **Given:** no special setup
- **When:** `clr --dry-run --journal` (no following value)
- **Then:** exit 1; stderr mentions `--journal` or `requires a value`
- **Exit:** 1
- **Source:** parse.rs `next_value()` guard

---

### FT-16: `--journal full --journal meta` (last wins) → meta-level

- **Given:** temp journal dir; fake claude exits 0 with output
- **When:** `clr -p --max-sessions 0 --journal full --journal meta --journal-dir <tmpdir> "x"`
- **Then:** JSONL contains execution event; `stdout` field absent (meta-level wins)
- **Exit:** 0
- **Source:** Standard last-wins flag semantics

---

### FT-17: `--journal off --journal-dir <dir>` → no JSONL

- **Given:** parent temp dir with non-existent subdirectory; fake claude exits 0
- **When:** `clr -p --max-sessions 0 --journal off --journal-dir <parent>/should_not_appear "x"`
- **Then:** `<parent>/should_not_appear` does NOT exist (off short-circuits before dir creation)
- **Exit:** 0
- **Source:** resolve_journal_writer() early return on "off"

---

### FT-18: `CLR_JOURNAL=off` + `CLR_JOURNAL_DIR=<dir>` → no JSONL

- **Given:** parent temp dir with non-existent subdirectory; fake claude exits 0; `CLR_JOURNAL=off` + `CLR_JOURNAL_DIR=<parent>/env_off_should_not_appear`
- **When:** `clr -p --max-sessions 0 "x"` with env vars set
- **Then:** `<parent>/env_off_should_not_appear` does NOT exist
- **Exit:** 0
- **Source:** env var precedence + resolve_journal_writer() early return on "off"
