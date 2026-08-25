# Parameter :: `journal_dir`

Edge case tests for the `journal_dir` parameter. Tests validate the
3-level resolution order: CLI parameter, environment variable, default.

**Source:** [param/21_journal_dir.md](../../../../docs/cli/param/21_journal_dir.md)

## Test Case Index

| ID | Test Name | Category | Status | Implemented as |
|----|-----------|----------|--------|----------------|
| EC-1 | Absent, no env var -> defaults to `~/.clr/journal/` | Default | ⏳ | — |
| EC-2 | `CLR_JOURNAL_DIR` env var set, param absent -> env var value used | Resolution Order | ⏳ | — |
| EC-3 | Both param and env var set -> param takes priority | Resolution Order | ⏳ | — |
| EC-4 | Empty `HOME` never resolves the journal relative to cwd | Default | ✅ | `viewer_integration_test.rs::ec25_empty_home_does_not_resolve_relative_journal` |
| EC-5 | `journal_dir::` is the key read; it is not spelled `dir::` | Name Collision | ✅ | `viewer_integration_test.rs::ec27_dir_param_filters_by_event_working_directory` |

## Test Coverage Summary

- Default: 2 tests (EC-1, EC-4)
- Resolution Order: 2 tests (EC-2, EC-3)
- Name Collision: 1 test (EC-5)

**Total:** 5 edge cases (2 executable)

Every other test in the crate supplies `journal_dir::` explicitly, which
short-circuits resolution at tier 1 — so tiers 2 and 3 are exercised only by
the cases that deliberately withhold it (EC-4, and EC-1/EC-2/EC-3 when
implemented).

## Test Cases

---

### EC-1: Absent, no env var -> defaults to `~/.clr/journal/`

- **Given:** `CLR_JOURNAL_DIR` is unset; `~/.clr/journal/` exists
- **When:** `clj .list`
- **Then:** exit 0; events are read from `~/.clr/journal/`
- **Exit:** 0
- **Source:** [param/21_journal_dir.md](../../../../docs/cli/param/21_journal_dir.md)

---

### EC-2: `CLR_JOURNAL_DIR` env var set, param absent -> env var value used

- **Given:** `CLR_JOURNAL_DIR=/var/log/clr` is set; `/var/log/clr` exists
- **When:** `clj .list`
- **Then:** exit 0; events are read from `/var/log/clr`
- **Exit:** 0
- **Source:** [param/21_journal_dir.md](../../../../docs/cli/param/21_journal_dir.md)

---

### EC-3: Both param and env var set -> param takes priority

- **Given:** `CLR_JOURNAL_DIR=/var/log/clr` is set; `/tmp/test_journal` also exists
- **When:** `clj .list journal_dir::/tmp/test_journal`
- **Then:** exit 0; events are read from `/tmp/test_journal`, not `/var/log/clr`
- **Exit:** 0
- **Source:** [param/21_journal_dir.md](../../../../docs/cli/param/21_journal_dir.md)

---

### EC-4: Empty `HOME` never resolves the journal relative to cwd

- **Given:** `HOME=""` (set but empty), `CLR_JOURNAL_DIR` unset, no `journal_dir::`, and a journal at `<cwd>/.clr/journal` holding a uniquely-named probe event
- **When:** `clj .list` run with cwd set to that directory's parent
- **Then:** exit 0; the probe event is *not* shown — `PathBuf::from("").join(".clr")` is relative, so an unguarded empty `HOME` would silently read the cwd-relative journal. A positive control reads the same fixture via explicit `journal_dir::` first, so the absence assertion cannot pass through a broken fixture
- **Exit:** 0
- **Source:** [param/21_journal_dir.md](../../../../docs/cli/param/21_journal_dir.md)

---

### EC-5: `journal_dir::` is the key read; it is not spelled `dir::`

- **Given:** a temp journal whose events record their own working directories
- **When:** `clj .list journal_dir::<tmpdir> dir::/work/beta`
- **Then:** exit 0; the journal is read from `<tmpdir>` and the output is narrowed by the event-directory filter. The two keys carry different values in one invocation, so a build that resolved the journal from `dir::` would read a nonexistent path and print nothing
- **Exit:** 0
- **Source:** [param/21_journal_dir.md](../../../../docs/cli/param/21_journal_dir.md), [param/07_dir.md](../../../../docs/cli/param/07_dir.md)
