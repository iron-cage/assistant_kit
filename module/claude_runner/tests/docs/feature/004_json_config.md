# Test: Feature — JSON Config Loading

Test case planning for [feature/004_json_config.md](../../../../docs/feature/004_json_config.md). Tests validate JSON file loading via `--args-file`, stdin JSON pipe detection, precedence ordering (CLI > JSON > CLR_* > defaults), boolean flag handling, error cases, and subcommand coverage.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| JC-1 | `--args-file` loads JSON and applies params | File Loading |
| JC-2 | CLI flag overrides JSON config value | Precedence |
| JC-3 | JSON config overrides CLR_* env var | Precedence |
| JC-4 | `CLR_ARGS_FILE` env var equivalent to `--args-file` | Env Var |
| JC-5 | Stdin JSON pipe detected when stdin not a TTY | Stdin Pipe |
| JC-6 | Invalid JSON in config file → exit 1 with parse error | Error Handling |
| JC-7 | Non-existent `--args-file` path → exit 1 with file-not-found | Error Handling |
| JC-8 | Boolean flag `true` activates flag; `false` is no-op | Boolean Handling |
| JC-9 | Unknown JSON key silently ignored; other keys applied | Forward Compat |
| JC-10 | JSON config applies to `isolated` subcommand | Subcommand |

## Test Coverage Summary

- File Loading: 1 test (JC-1)
- Precedence: 2 tests (JC-2, JC-3)
- Env Var: 1 test (JC-4)
- Stdin Pipe: 1 test (JC-5)
- Error Handling: 2 tests (JC-6, JC-7)
- Boolean Handling: 1 test (JC-8)
- Forward Compat: 1 test (JC-9)
- Subcommand: 1 test (JC-10)

**Total:** 10 tests

---

### JC-1: `--args-file` loads JSON and applies params

- **Given:** JSON file containing `{"model": "claude-haiku-4-5-20251001", "max-sessions": 0}` written to a temp path; fake claude binary in PATH
- **When:** `clr --args-file <tmp/fast.json> --dry-run "task"`
- **Then:** dry-run output contains `--model claude-haiku-4-5-20251001`; `--max-sessions` count is 0 (gate disabled); JSON params applied
- **Exit:** 0
- **Source:** [feature/004_json_config.md](../../../../docs/feature/004_json_config.md) AC-001; [cli/param/075_args_file.md](../../../../docs/cli/param/075_args_file.md)
- **Implemented by:** TBD — `json_config_test.rs`

---

### JC-2: CLI flag overrides JSON config value

- **Given:** JSON file containing `{"model": "claude-haiku-4-5-20251001"}`; CLI passes `--model claude-opus-4-6`
- **When:** `clr --args-file <tmp/fast.json> --model claude-opus-4-6 --dry-run "task"`
- **Then:** dry-run output contains `--model claude-opus-4-6` (CLI wins); JSON value for model is overridden
- **Exit:** 0
- **Source:** [feature/004_json_config.md](../../../../docs/feature/004_json_config.md) AC-003
- **Implemented by:** TBD — `json_config_test.rs`

---

### JC-3: JSON config overrides CLR_* env var

- **Given:** JSON file containing `{"model": "claude-haiku-4-5-20251001"}`; `CLR_MODEL=claude-opus-4-6` set in environment
- **When:** `CLR_MODEL=claude-opus-4-6 clr --args-file <tmp/fast.json> --dry-run "task"`
- **Then:** dry-run output contains `--model claude-haiku-4-5-20251001` (JSON wins over env var)
- **Exit:** 0
- **Source:** [feature/004_json_config.md](../../../../docs/feature/004_json_config.md) AC-004
- **Implemented by:** TBD — `json_config_test.rs`

---

### JC-4: `CLR_ARGS_FILE` env var equivalent to `--args-file`

- **Given:** JSON file at a temp path containing `{"max-sessions": 0}`; `CLR_ARGS_FILE` set to that path
- **When:** `CLR_ARGS_FILE=<tmp/fast.json> clr --dry-run "task"` (no `--args-file` on CLI)
- **Then:** dry-run output reflects `--max-sessions 0` from JSON; behavior identical to `--args-file <path>`
- **Exit:** 0
- **Source:** [feature/004_json_config.md](../../../../docs/feature/004_json_config.md) AC-005; [cli/param/075_args_file.md](../../../../docs/cli/param/075_args_file.md)
- **Implemented by:** TBD — `json_config_test.rs`

---

### JC-5: Stdin JSON pipe detected when stdin not a TTY

- **Given:** JSON input `{"max-sessions": 0}` piped to stdin; stdin is not a TTY (pipe context); no `--args-file` flag; no `--file` flag
- **When:** `echo '{"max-sessions":0}' | clr --dry-run "task"`
- **Then:** dry-run output reflects `--max-sessions 0` from stdin JSON; stdin JSON auto-detected and consumed as param source
- **Exit:** 0
- **Source:** [feature/004_json_config.md](../../../../docs/feature/004_json_config.md) AC-002
- **Implemented by:** TBD — `json_config_test.rs`

---

### JC-6: Invalid JSON in config file → exit 1 with parse error

- **Given:** File at temp path containing malformed JSON (`{model: bad}` — no quotes)
- **When:** `clr --args-file <tmp/bad.json> "task"` with fake claude binary
- **Then:** clr exits 1; stderr contains JSON parse error message; subprocess not spawned
- **Exit:** 1
- **Source:** [feature/004_json_config.md](../../../../docs/feature/004_json_config.md) AC-006
- **Implemented by:** TBD — `json_config_test.rs`

---

### JC-7: Non-existent `--args-file` path → exit 1 with file-not-found

- **Given:** `--args-file` path points to a file that does not exist
- **When:** `clr --args-file /tmp/nonexistent_clr_config_xyz.json "task"` with fake claude binary
- **Then:** clr exits 1; stderr contains file-not-found or similar IO error; subprocess not spawned
- **Exit:** 1
- **Source:** [feature/004_json_config.md](../../../../docs/feature/004_json_config.md) AC-007
- **Implemented by:** TBD — `json_config_test.rs`

---

### JC-8: Boolean flag `true` activates flag; `false` is no-op

- **Given:** JSON file containing `{"dry-run": true}`
- **When:** `clr --args-file <tmp/fast.json> "task"` with fake claude binary (no `--dry-run` on CLI)
- **Then:** clr executes in dry-run mode (prints command without spawning); boolean `true` in JSON activates the flag; JSON `false` for any other boolean param is a no-op (absent)
- **Exit:** 0
- **Source:** [feature/004_json_config.md](../../../../docs/feature/004_json_config.md) AC-008
- **Implemented by:** TBD — `json_config_test.rs`

---

### JC-9: Unknown JSON key silently ignored; other keys applied

- **Given:** JSON file containing `{"_future_param": "x", "max-sessions": 0}` (one unknown key, one known)
- **When:** `clr --args-file <tmp/fast.json> --dry-run "task"`
- **Then:** clr exits 0; no error for unknown key `_future_param`; `max-sessions: 0` is applied; forward-compatible
- **Exit:** 0
- **Source:** [feature/004_json_config.md](../../../../docs/feature/004_json_config.md) AC-009
- **Implemented by:** TBD — `json_config_test.rs`

---

### JC-10: JSON config applies to `isolated` subcommand

- **Given:** JSON file containing `{"timeout": 60}` at a temp path; `CLR_ARGS_FILE` set to that path
- **When:** `CLR_ARGS_FILE=<tmp/fast.json> clr isolated --dry-run -c /tmp/test-creds.json`
- **Then:** isolated subcommand uses timeout of 60s from JSON config; param applies cross-subcommand
- **Exit:** 0
- **Source:** [feature/004_json_config.md](../../../../docs/feature/004_json_config.md) AC-010
- **Implemented by:** TBD — `json_config_test.rs`
