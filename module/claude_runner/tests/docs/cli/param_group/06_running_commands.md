# Parameter Group :: Running Commands

Cross-command behavior tests for Group 6 (Running Commands): verifying that `run`, `ask`, `isolated`,
and `refresh` share the expected parameter surface and injection behaviors documented in the comparison
table.

**Source:** [param_group/06_running_commands.md](../../../../docs/cli/param_group/06_running_commands.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | `--dry-run` accepted by all 4 running commands | Cross-command |
| CC-2 | `--trace` emitted by all 4 running commands | Cross-command |
| CC-3 | `--timeout` default differs per running command | Cross-command |
| CC-4 | `isolated` and `refresh` always inject `--no-session-persistence` | Cross-command |
| CC-5 | `--journal` and `--journal-dir` accepted by all 4 running commands | Cross-command |

## Test Coverage Summary

- Cross-command: 5 tests (CC-1 through CC-5)

**Total:** 5 cross-command cases

## Test Cases

---

### CC-1: `--dry-run` accepted by all 4 running commands

- **Given:** clean environment; credentials JSON at temp path
- **When:** `clr --dry-run "task"` and `clr ask --dry-run "task"` and `clr isolated --creds <tmp> --dry-run "task"` and `clr refresh --creds <tmp> --dry-run`
- **Then:** All 4 invocations exit 0 with dry-run preview output; no subprocess is spawned by any of them
- **Exit:** 0
- **Source:** [param_group/06_running_commands.md](../../../../docs/cli/param_group/06_running_commands.md)
- **Commands:** run, ask, isolated, refresh

---

### CC-2: `--trace` emitted by all 4 running commands

- **Given:** fake `claude` script that exits 0; credentials JSON at temp path
- **When:** `clr --trace "task"` and `clr ask --trace "task"` and `clr isolated --creds <tmp> --trace "task"` and `clr refresh --creds <tmp> --trace`
- **Then:** All 4 invocations emit a `[clr]` trace prefix line to stderr before executing; exit 0
- **Exit:** 0
- **Source:** [param_group/06_running_commands.md](../../../../docs/cli/param_group/06_running_commands.md)
- **Commands:** run, ask, isolated, refresh

---

### CC-3: `--timeout` default differs per running command

- **Given:** clean environment; credentials JSON at temp path
- **When:** `clr --dry-run "task"` (run), `clr ask --dry-run "task"`, `clr isolated --creds <tmp> --dry-run "task"`, `clr refresh --creds <tmp> --dry-run` — inspect dry-run env block
- **Then:** `run` and `ask` show `timeout=3600`; `isolated` shows `timeout=30`; `refresh` shows `timeout=45`
- **Exit:** 0
- **Source:** [param_group/06_running_commands.md](../../../../docs/cli/param_group/06_running_commands.md)
- **Commands:** run, ask, isolated, refresh

---

### CC-4: `isolated` and `refresh` always inject `--no-session-persistence`

- **Given:** clean environment; credentials JSON at temp path
- **When:** `clr isolated --creds <tmp> --dry-run "task"` and `clr refresh --creds <tmp> --dry-run`; also `clr --dry-run "task"` (run, no `--no-persist`)
- **Then:** Both `isolated` and `refresh` assembled commands contain `--no-session-persistence`; plain `run` without `--no-persist` does NOT contain it
- **Exit:** 0
- **Source:** [param_group/06_running_commands.md](../../../../docs/cli/param_group/06_running_commands.md)
- **Commands:** isolated, refresh, run

---

### CC-5: `--journal` and `--journal-dir` accepted by all 4 running commands

- **Given:** fake `claude` script that exits 0; writable journal temp directory; credentials JSON at temp path
- **When:** `clr --journal full --journal-dir <tmp> "task"`, `clr ask --journal meta --journal-dir <tmp> "task"`, `clr isolated --creds <tmp> --journal off --journal-dir <tmp> "task"`, `clr refresh --creds <tmp> --journal full --journal-dir <tmp>`
- **Then:** All 4 invocations accept `--journal` and `--journal-dir` without error; `full` and `meta` modes write journal entries; `off` mode writes nothing
- **Exit:** 0
- **Source:** [param_group/06_running_commands.md](../../../../docs/cli/param_group/06_running_commands.md)
- **Commands:** run, ask, isolated, refresh
