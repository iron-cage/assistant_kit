# Parameter Group :: Global

Interaction tests for the Global group: `port`, `bind`, `open`, `journal_dir`,
`no_color`, `refresh`. Tests validate cross-command resolution order and
format-scoped effects.

**Source:** [param_group/05_global.md](../../../../docs/cli/param_group/05_global.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | CLI `journal_dir` param takes precedence over `CLR_JOURNAL_DIR` env var | Resolution Order |
| CC-2 | `CLR_JOURNAL_DIR` env var used when `journal_dir` param absent | Resolution Order |
| CC-3 | `NO_COLOR` env var disables color without `no_color::1` param | Env Var Interaction |
| CC-4 | `no_color::1` with `format::json` -> no effect | Format Scoping |

## Test Coverage Summary

- Resolution Order: 2 tests (CC-1, CC-2)
- Env Var Interaction: 1 test (CC-3)
- Format Scoping: 1 test (CC-4)

**Total:** 4 corner cases

## Test Cases
---

### CC-1: CLI `journal_dir` param takes precedence over `CLR_JOURNAL_DIR` env var

- **Given:** `CLR_JOURNAL_DIR` set to `/tmp/env_journal`; a different directory `/tmp/cli_journal` also exists with events
- **When:** `clj .list journal_dir::/tmp/cli_journal` (env var still set)
- **Then:** events are read from `/tmp/cli_journal`, not `/tmp/env_journal`
- **Exit:** 0
- **Source:** [param_group/05_global.md](../../../../docs/cli/param_group/05_global.md)
---

### CC-2: `CLR_JOURNAL_DIR` env var used when `journal_dir` param absent

- **Given:** `CLR_JOURNAL_DIR` set to `/tmp/env_journal`, containing events; no `journal_dir` CLI param given
- **When:** `clj .list`
- **Then:** events are read from `/tmp/env_journal`, not the `~/.clr/journal/` default
- **Exit:** 0
- **Source:** [param_group/05_global.md](../../../../docs/cli/param_group/05_global.md)
---

### CC-3: `NO_COLOR` env var disables color without `no_color::1` param

- **Given:** `NO_COLOR` environment variable set; `no_color` CLI param not given
- **When:** `clj .list format::table`
- **Then:** output contains no ANSI color escape sequences
- **Exit:** 0
- **Source:** [param_group/05_global.md](../../../../docs/cli/param_group/05_global.md)
---

### CC-4: `no_color::1` with `format::json` -> no effect

- **Given:** clean environment
- **When:** `clj .list format::json no_color::1`
- **Then:** output is standard JSON; `no_color` has no observable effect since JSON never carries color codes
- **Exit:** 0
- **Source:** [param_group/05_global.md](../../../../docs/cli/param_group/05_global.md)
