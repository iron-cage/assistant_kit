# Parameter :: `--args-file`

Edge case tests for the `--args-file` parameter that loads clr parameters from a JSON file.
Covers file loading, env var fallback, dry-run transparency, boolean handling, and error paths.

**Source:** [075_args_file.md](../../../../docs/cli/param/075_args_file.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| AF-1 | `--args-file` flag accepted; JSON params loaded | Acceptance |
| AF-2 | `CLR_ARGS_FILE` env var fallback when no CLI flag | Env Var |
| AF-3 | `--args-file` + `--dry-run` shows JSON-sourced params in preview | Dry-Run |
| AF-4 | Boolean `true` in JSON activates flag | Boolean |
| AF-5 | Unknown JSON key silently ignored | Forward Compat |
| AF-6 | `--args-file` path does not exist → exit 1 | Error |

## Test Coverage Summary

- Acceptance: 1 test (AF-1)
- Env Var: 1 test (AF-2)
- Dry-Run: 1 test (AF-3)
- Boolean: 1 test (AF-4)
- Forward Compat: 1 test (AF-5)
- Error: 1 test (AF-6)

**Total:** 6 test cases

## Test Cases

---

### AF-1: `--args-file` flag accepted; JSON params loaded

- **Given:** JSON file `{"max-sessions": 0}` at a temp path; fake claude binary
- **When:** `clr --args-file <tmp/config.json> --dry-run "task"`
- **Then:** Exit 0; dry-run output reflects max-sessions 0 from the JSON file
- **Exit:** 0
- **Source:** [075_args_file.md](../../../../docs/cli/param/075_args_file.md)
- **Commands:** run, ask

---

### AF-2: `CLR_ARGS_FILE` env var fallback when no CLI flag

- **Given:** JSON file at temp path; `CLR_ARGS_FILE=<tmp/config.json>` set; no `--args-file` on CLI
- **When:** `CLR_ARGS_FILE=<tmp/config.json> clr --dry-run "task"`
- **Then:** Exit 0; JSON params applied; env var triggers same loading behavior as `--args-file`
- **Exit:** 0
- **Source:** [075_args_file.md](../../../../docs/cli/param/075_args_file.md)
- **Commands:** run, ask

---

### AF-3: `--args-file` + `--dry-run` shows JSON-sourced params in preview

- **Given:** JSON file `{"model": "claude-haiku-4-5-20251001"}` at temp path
- **When:** `clr --args-file <tmp/config.json> --dry-run "task"`
- **Then:** Exit 0; dry-run stdout includes `--model claude-haiku-4-5-20251001` from JSON; merged param set visible
- **Exit:** 0
- **Source:** [075_args_file.md](../../../../docs/cli/param/075_args_file.md)
- **Commands:** run, ask

---

### AF-4: Boolean `true` in JSON activates flag

- **Given:** JSON file `{"dry-run": true}` at temp path; no `--dry-run` on CLI
- **When:** `clr --args-file <tmp/config.json> "task"` with fake claude
- **Then:** Exit 0; clr runs in dry-run mode (command printed, no subprocess); JSON boolean `true` triggers flag
- **Exit:** 0
- **Source:** [075_args_file.md](../../../../docs/cli/param/075_args_file.md)
- **Commands:** run, ask

---

### AF-5: Unknown JSON key silently ignored

- **Given:** JSON file `{"_unknown_future_key": 42, "max-sessions": 0}` at temp path
- **When:** `clr --args-file <tmp/config.json> --dry-run "task"`
- **Then:** Exit 0; no error for unknown key; max-sessions 0 applied normally; forward-compatible behavior
- **Exit:** 0
- **Source:** [075_args_file.md](../../../../docs/cli/param/075_args_file.md)
- **Commands:** run, ask

---

### AF-6: `--args-file` path does not exist → exit 1

- **Given:** `--args-file` references `/tmp/nonexistent_args_file_test_xyz.json` (guaranteed absent)
- **When:** `clr --args-file /tmp/nonexistent_args_file_test_xyz.json "task"`
- **Then:** Exit 1; stderr contains file-not-found error; subprocess not spawned
- **Exit:** 1
- **Source:** [075_args_file.md](../../../../docs/cli/param/075_args_file.md)
- **Commands:** run, ask
