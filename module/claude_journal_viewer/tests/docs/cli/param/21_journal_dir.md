# Parameter :: `journal_dir`

Edge case tests for the `journal_dir` parameter. Tests validate the
3-level resolution order: CLI parameter, environment variable, default.

**Source:** [param/21_journal_dir.md](../../../../docs/cli/param/21_journal_dir.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent, no env var -> defaults to `~/.clr/journal/` | Default |
| EC-2 | `CLR_JOURNAL_DIR` env var set, param absent -> env var value used | Resolution Order |
| EC-3 | Both param and env var set -> param takes priority | Resolution Order |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Resolution Order: 2 tests (EC-2, EC-3)

**Total:** 3 edge cases

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
