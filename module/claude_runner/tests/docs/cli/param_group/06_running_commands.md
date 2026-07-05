# Parameter Group :: Running Commands

Cross-command behavior tests for Group 6 (Running Commands): verifying that `run`, `ask`, `isolated`,
and `refresh` share the expected parameter surface and injection behaviors, and that
`--timeout`, `--trace`, `--dry-run`, `--no-compact-window`, `--journal`, and `--journal-dir`
behave consistently and interact correctly across all four commands.

**Source:** [param_group/06_running_commands.md](../../../../docs/cli/param_group/06_running_commands.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | `--dry-run` accepted by all 4 running commands | Cross-command |
| CC-2 | `--trace` emitted by all 4 running commands | Cross-command |
| CC-3 | `--timeout` default differs per running command | Cross-command |
| CC-4 | `isolated` and `refresh` always inject `--no-session-persistence` | Cross-command |
| CC-5 | `--journal` and `--journal-dir` accepted by all 4 running commands | Cross-command |
| RC-1 | `--dry-run` output matches `--trace` output for `isolated` | WYSIWYG |
| RC-2 | `--dry-run` output matches `--trace` output for `refresh` | WYSIWYG |
| RC-3 | `CLAUDE_CODE_AUTO_COMPACT_WINDOW=300000` present in all 4 commands by default | Default |
| RC-4 | `--no-compact-window` suppresses env var for `run` | Opt-Out |
| RC-5 | `--no-compact-window` suppresses env var for `isolated` | Opt-Out |
| RC-6 | `--no-compact-window` suppresses env var for `refresh` | Opt-Out |
| RC-7 | `CLR_NO_COMPACT_WINDOW=1` equivalent to `--no-compact-window` for `run` | EnvFallback |
| RC-8 | `--journal off` suppresses journal creation for `isolated` | Journaling |
| RC-9 | `--timeout 0` means unlimited for all running commands | Universal |

## Test Coverage Summary

- Cross-command: 5 tests (CC-1 through CC-5)
- WYSIWYG: 2 tests (RC-1, RC-2)
- Default: 1 test (RC-3)
- Opt-Out: 3 tests (RC-4, RC-5, RC-6)
- EnvFallback: 1 test (RC-7)
- Journaling: 1 test (RC-8)
- Universal: 1 test (RC-9)

**Total:** 14 interaction cases

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

---

### RC-1: `--dry-run` output matches `--trace` output for `isolated`

- **Given:** credentials JSON at `/tmp/rc1.creds.json` (content `{}`; no live credentials needed)
- **When (A):** `clr isolated --creds /tmp/rc1.creds.json --dry-run 2>&1`
- **When (B):** `clr isolated --creds /tmp/rc1.creds.json --trace 2>&1` (will fail after trace; ignore exit)
- **Then:** stderr output of A and B are identical lines up to the command line; both show env vars and command; no stdout for A
- **Exit A:** 0 (dry-run exits without spawning); **Exit B:** 1 (claude absent)
- **Note:** verifies R5-2: dry-run uses `emit_credential_trace` — same code path as trace (WYSIWYG)
- **Source:** [param_group/06_running_commands.md](../../../../docs/cli/param_group/06_running_commands.md), [invariant/004_trace_universality.md](../../../../docs/invariant/004_trace_universality.md)
- **Commands:** isolated

---

### RC-2: `--dry-run` output matches `--trace` output for `refresh`

- **Given:** credentials JSON at `/tmp/rc2.creds.json` (content `{}`; no live credentials needed)
- **When (A):** `clr refresh --creds /tmp/rc2.creds.json --dry-run 2>&1`
- **When (B):** `clr refresh --creds /tmp/rc2.creds.json --trace 2>&1`
- **Then:** stderr output of A and B are identical lines up to the command line; both show env vars and command
- **Exit A:** 0; **Exit B:** 1
- **Note:** same WYSIWYG verification as RC-1 for the refresh command
- **Source:** [param_group/06_running_commands.md](../../../../docs/cli/param_group/06_running_commands.md)
- **Commands:** refresh

---

### RC-3: `CLAUDE_CODE_AUTO_COMPACT_WINDOW=300000` present in all 4 commands by default

- **Given:** clean environment, no `CLR_NO_COMPACT_WINDOW`
- **When (run):** `clr --dry-run "test" 2>&1`; **When (isolated):** `clr isolated --creds /tmp/creds.json --dry-run 2>&1`; **When (refresh):** `clr refresh --creds /tmp/creds.json --dry-run 2>&1`
- **Then:** Each output contains `CLAUDE_CODE_AUTO_COMPACT_WINDOW=300000`
- **Exit:** 0
- **Note:** lim_it for isolated/refresh — requires credentials file to exist (content irrelevant for dry-run)
- **Source:** [param_group/06_running_commands.md](../../../../docs/cli/param_group/06_running_commands.md), [env_param.md](../../../../docs/cli/env_param.md)
- **Commands:** run, ask, isolated, refresh

---

### RC-4: `--no-compact-window` suppresses env var for `run`

- **Given:** clean environment
- **When:** `clr --no-compact-window --dry-run "test" 2>&1`
- **Then:** output does NOT contain `CLAUDE_CODE_AUTO_COMPACT_WINDOW`
- **Exit:** 0
- **Source:** [param_group/06_running_commands.md](../../../../docs/cli/param_group/06_running_commands.md), [param/077_no_compact_window.md](../../../../docs/cli/param/077_no_compact_window.md)
- **Commands:** run

---

### RC-5: `--no-compact-window` suppresses env var for `isolated`

- **Given:** credentials JSON at `/tmp/rc5.creds.json` (content `{}`)
- **When:** `clr isolated --creds /tmp/rc5.creds.json --no-compact-window --dry-run 2>&1`
- **Then:** output does NOT contain `CLAUDE_CODE_AUTO_COMPACT_WINDOW`
- **Exit:** 0
- **Source:** [param_group/06_running_commands.md](../../../../docs/cli/param_group/06_running_commands.md), [param/077_no_compact_window.md](../../../../docs/cli/param/077_no_compact_window.md)
- **Commands:** isolated

---

### RC-6: `--no-compact-window` suppresses env var for `refresh`

- **Given:** credentials JSON at `/tmp/rc6.creds.json` (content `{}`)
- **When:** `clr refresh --creds /tmp/rc6.creds.json --no-compact-window --dry-run 2>&1`
- **Then:** output does NOT contain `CLAUDE_CODE_AUTO_COMPACT_WINDOW`
- **Exit:** 0
- **Source:** [param_group/06_running_commands.md](../../../../docs/cli/param_group/06_running_commands.md), [param/077_no_compact_window.md](../../../../docs/cli/param/077_no_compact_window.md)
- **Commands:** refresh

---

### RC-7: `CLR_NO_COMPACT_WINDOW=1` equivalent to `--no-compact-window`

- **Given:** `CLR_NO_COMPACT_WINDOW=1` in environment
- **When:** `CLR_NO_COMPACT_WINDOW=1 clr --dry-run "test" 2>&1`
- **Then:** output does NOT contain `CLAUDE_CODE_AUTO_COMPACT_WINDOW`; identical suppression to using `--no-compact-window` flag
- **Exit:** 0
- **Source:** [param_group/06_running_commands.md](../../../../docs/cli/param_group/06_running_commands.md), [env_param.md](../../../../docs/cli/env_param.md)
- **Commands:** run

---

### RC-8: `--journal off` suppresses journal creation for `isolated`

- **Given:** credentials JSON at `/tmp/rc8.creds.json` (content `{}`); journal dir at `/tmp/rc8-journal/`
- **When:** `clr isolated --creds /tmp/rc8.creds.json --dry-run --journal off --journal-dir /tmp/rc8-journal/`
- **Then:** no `.jsonl` file written to `/tmp/rc8-journal/`; dry-run exits 0 without creating temp HOME
- **Exit:** 0
- **Source:** [param_group/06_running_commands.md](../../../../docs/cli/param_group/06_running_commands.md)
- **Commands:** isolated

---

### RC-9: `--timeout 0` means unlimited for all running commands

- **Given:** credentials file exists for isolated/refresh
- **When (isolated):** `clr isolated --creds /tmp/creds.json --dry-run --timeout 0 2>&1`; **When (refresh):** `clr refresh --creds /tmp/creds.json --dry-run --timeout 0 2>&1`; **When (run):** `clr --dry-run --timeout 0 "test" 2>&1`
- **Then:** each output shows `timeout: 0s` (unlimited); no validation error; exits 0
- **Exit:** 0
- **Note:** `--timeout 0` means unlimited (no watchdog) for all 4 commands — matches `run`/`ask` semantics
- **Source:** [param_group/06_running_commands.md](../../../../docs/cli/param_group/06_running_commands.md), [invariant/005_isolated_subprocess_defaults.md](../../../../docs/invariant/005_isolated_subprocess_defaults.md)
- **Commands:** run, isolated, refresh
