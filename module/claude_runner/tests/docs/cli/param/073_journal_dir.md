# Parameter :: `--journal-dir` (run/ask/isolated/refresh)

Edge case coverage for the `--journal-dir` parameter. See [073_journal_dir.md](../../../../docs/cli/param/073_journal_dir.md) for specification.

**Scope note:** `--journal-dir` overrides the journal directory path. Resolution order: CLI flag > `CLR_JOURNAL_DIR` env var > `~/.clr/journal/` default. The directory is created automatically on first append.

- **Commands:** run, ask, isolated, refresh

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-01 | `--journal-dir <tmpdir>` — event written to specified path | Behavioral Divergence |
| EC-02 | No `--journal-dir` + no `CLR_JOURNAL_DIR` → uses `~/.clr/journal/` default | Behavioral Divergence |
| EC-03 | `CLR_JOURNAL_DIR=<tmpdir>` env var — event written to env var path | Env Var |
| EC-04 | `--journal-dir <a>` + `CLR_JOURNAL_DIR=<b>` → CLI flag wins | CLI Wins |
| EC-05 | Directory created automatically if it does not exist | Auto Create |
| EC-06 | `--journal off --journal-dir <tmpdir>` — no event written (off level wins) | Level Interaction |
| EC-07 | Read-only `--journal-dir` path → journal write fails; clr exits 0 | Error Isolation |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-01, EC-02)
- Env Var: 1 test (EC-03)
- CLI Wins: 1 test (EC-04)
- Auto Create: 1 test (EC-05)
- Level Interaction: 1 test (EC-06)
- Error Isolation: 1 test (EC-07)

**Total:** 7 test cases

## Architectural Constraint

EC-02 reads the actual `~/.clr/journal/` directory to verify a file was created. This test must clean up any file it creates in the default location or skip in environments where `$HOME` is not writable.

EC-05 deliberately passes a path inside a freshly created temp dir (`<tmpdir>/deep/nested/`) that does not yet exist at test start.

EC-07 requires `std::fs::set_permissions` to make `<tmpdir>` read-only before running `clr`.

---

### EC-01: `--journal-dir <path>` writes events to that path

- **Given:** two distinct temporary directories `dir_a`, `dir_b`; fake claude exits 0; `--max-sessions 0`
- **When:** `clr -p --max-sessions 0 --journal-dir <dir_a> "task"`
- **Then:** `<dir_a>/YYYY-MM-DD.jsonl` exists with one event line; `<dir_b>` has no `.jsonl` files
- **Exit:** 0
- **Source:** [073_journal_dir.md](../../../../docs/cli/param/073_journal_dir.md) — CLI override

---

### EC-02: Default journal directory is `~/.clr/journal/`

- **Given:** no `--journal-dir` flag; no `CLR_JOURNAL_DIR` env var; fake claude exits 0
- **When:** `clr -p --max-sessions 0 "task"`
- **Then:** `$HOME/.clr/journal/YYYY-MM-DD.jsonl` exists (or is extended with new line)
- **Exit:** 0
- **Source:** [073_journal_dir.md](../../../../docs/cli/param/073_journal_dir.md) — Default: ~/.clr/journal/

---

### EC-03: `CLR_JOURNAL_DIR` env var sets the journal directory

- **Given:** temporary directory `dir_a`; env `CLR_JOURNAL_DIR=<dir_a>`; fake claude exits 0
- **When:** `clr -p --max-sessions 0 "task"` with `CLR_JOURNAL_DIR` set
- **Then:** `<dir_a>/YYYY-MM-DD.jsonl` exists with one event line
- **Exit:** 0
- **Source:** [073_journal_dir.md](../../../../docs/cli/param/073_journal_dir.md) — Env var

---

### EC-04: CLI `--journal-dir` wins over `CLR_JOURNAL_DIR`

- **Given:** two temporary directories `dir_a`, `dir_b`; env `CLR_JOURNAL_DIR=<dir_b>`; fake claude exits 0
- **When:** `clr -p --max-sessions 0 --journal-dir <dir_a> "task"`
- **Then:** `<dir_a>/YYYY-MM-DD.jsonl` exists; `<dir_b>` has no `.jsonl` files — CLI flag overrides env var
- **Exit:** 0
- **Source:** [073_journal_dir.md](../../../../docs/cli/param/073_journal_dir.md) — Resolution order

---

### EC-05: Directory created automatically if absent

- **Given:** temporary root `<tmpdir>`; target path `<tmpdir>/deep/nested/` does not exist
- **When:** `clr -p --max-sessions 0 --journal-dir <tmpdir>/deep/nested "task"`
- **Then:** `<tmpdir>/deep/nested/YYYY-MM-DD.jsonl` exists after execution
- **Exit:** 0
- **Source:** [073_journal_dir.md](../../../../docs/cli/param/073_journal_dir.md) — "created if absent"

---

### EC-06: `--journal off` with `--journal-dir` → no event written

- **Given:** temporary journal dir; fake claude exits 0
- **When:** `clr -p --max-sessions 0 --journal off --journal-dir <tmpdir> "task"`
- **Then:** `<tmpdir>` has no `.jsonl` files — off level suppresses all writes regardless of dir
- **Exit:** 0
- **Source:** [073_journal_dir.md](../../../../docs/cli/param/073_journal_dir.md) — Level interaction

---

### EC-07: Read-only journal dir → write fails; clr exits 0

- **Given:** temporary directory made read-only (`0o555`); fake claude exits 0
- **When:** `clr -p --max-sessions 0 --journal-dir <readonly_dir> "task"`
- **Then:** exit 0 — journal write failure does not change clr exit code (best-effort journaling)
- **Exit:** 0
- **Source:** [073_journal_dir.md](../../../../docs/cli/param/073_journal_dir.md) — "Error handling: best-effort"
