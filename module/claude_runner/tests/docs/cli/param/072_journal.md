# Parameter :: `--journal` (run/ask/isolated/refresh)

Edge case coverage for the `--journal` parameter. See [072_journal.md](../../../../docs/cli/param/072_journal.md) for specification.

**Scope note:** `--journal` selects the journaling level: `full` (default, all fields including stdout/stderr), `meta` (metadata only, no output), or `off` (no journaling). Accepted by `run`, `ask`, `isolated`, and `refresh`.

- **Commands:** run, ask, isolated, refresh

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-01 | `--journal full` — event written with stdout field | Behavioral Divergence |
| EC-02 | `--journal meta` — event written without stdout/stderr | Behavioral Divergence |
| EC-03 | `--journal off` — no event created | Off Level |
| EC-04 | Default (no flag) — journals at `full` (same as explicit `--journal full`) | Default |
| EC-05 | `CLR_JOURNAL=full` env var — same as `--journal full` | Env Var |
| EC-06 | `CLR_JOURNAL=off` env var — no event created | Env Var Off |
| EC-07 | `--journal full` + `CLR_JOURNAL=off` → CLI flag wins | CLI Wins |
| EC-08 | Invalid value `--journal verbose` → exit 1; stderr shows enum error | Validation |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-01, EC-02)
- Off Level: 1 test (EC-03)
- Default: 1 test (EC-04)
- Env Var: 1 test (EC-05)
- Env Var Off: 1 test (EC-06)
- CLI Wins: 1 test (EC-07)
- Validation: 1 test (EC-08)

**Total:** 8 test cases

## Architectural Constraint

All tests that check event content (EC-01, EC-02, EC-04, EC-05, EC-07) require:
- A fake `claude` subprocess that exits 0 and prints a known string to stdout
- A temporary directory as `--journal-dir` so events land in an isolated path, not `~/.clr/journal/`
- Reading the journal JSONL file after `clr` exits and deserializing the last line

EC-03 and EC-06 assert the absence of any `.jsonl` file in the temp dir.

---

### EC-01: `--journal full` → event has stdout field

- **Given:** temporary journal dir; fake claude prints `"hello"`; `--max-sessions 0`
- **When:** `clr -p --max-sessions 0 --journal full --journal-dir <tmpdir> "task"`
- **Then:** journal `YYYY-MM-DD.jsonl` contains one line; `fields.stdout == Some("hello")`; `fields.exit_code == Some(0)`
- **Exit:** 0
- **Source:** [072_journal.md](../../../../docs/cli/param/072_journal.md) — `full` level

---

### EC-02: `--journal meta` → event has no stdout/stderr

- **Given:** temporary journal dir; fake claude prints `"hello"`; `--max-sessions 0`
- **When:** `clr -p --max-sessions 0 --journal meta --journal-dir <tmpdir> "task"`
- **Then:** journal line exists; `fields.stdout == None`; `fields.stderr == None`; `fields.exit_code == Some(0)` (metadata present)
- **Exit:** 0
- **Source:** [072_journal.md](../../../../docs/cli/param/072_journal.md) — `meta` level

---

### EC-03: `--journal off` → no journal event

- **Given:** temporary journal dir; fake claude exits 0
- **When:** `clr -p --max-sessions 0 --journal off --journal-dir <tmpdir> "task"`
- **Then:** `<tmpdir>` has no `.jsonl` files after execution
- **Exit:** 0
- **Source:** [072_journal.md](../../../../docs/cli/param/072_journal.md) — `off` level

---

### EC-04: Default (no `--journal`) journals at `full`

- **Given:** temporary journal dir; fake claude prints `"result"`
- **When:** `clr -p --max-sessions 0 --journal-dir <tmpdir> "task"` (no --journal flag)
- **Then:** journal line exists; `fields.stdout == Some("result")` — confirms default is `full`
- **Exit:** 0
- **Source:** [072_journal.md](../../../../docs/cli/param/072_journal.md) — Default: full

---

### EC-05: `CLR_JOURNAL=full` env var activates full level

- **Given:** temporary journal dir; fake claude prints `"data"`; env `CLR_JOURNAL=full`
- **When:** `clr -p --max-sessions 0 --journal-dir <tmpdir> "task"` with env var set
- **Then:** journal line exists; `fields.stdout == Some("data")`
- **Exit:** 0
- **Source:** [072_journal.md](../../../../docs/cli/param/072_journal.md) — Env var

---

### EC-06: `CLR_JOURNAL=off` env var disables journaling

- **Given:** temporary journal dir; env `CLR_JOURNAL=off`
- **When:** `clr -p --max-sessions 0 --journal-dir <tmpdir> "task"` with env var set
- **Then:** `<tmpdir>` has no `.jsonl` files
- **Exit:** 0
- **Source:** [072_journal.md](../../../../docs/cli/param/072_journal.md) — Env var off

---

### EC-07: CLI `--journal full` wins over `CLR_JOURNAL=off`

- **Given:** temporary journal dir; fake claude prints `"ok"`; env `CLR_JOURNAL=off`
- **When:** `clr -p --max-sessions 0 --journal full --journal-dir <tmpdir> "task"`
- **Then:** journal line exists; `fields.stdout == Some("ok")` — CLI flag overrides env var
- **Exit:** 0
- **Source:** [072_journal.md](../../../../docs/cli/param/072_journal.md) — CLI > env resolution

---

### EC-08: Invalid `--journal verbose` → validation error

- **Given:** no fake subprocess needed (parse error fires before execution)
- **When:** `clr --journal verbose "task"`
- **Then:** exit 1; stderr contains text indicating invalid journal level or unrecognized value
- **Exit:** 1
- **Source:** [072_journal.md](../../../../docs/cli/param/072_journal.md) — Type: JournalLevel enum
