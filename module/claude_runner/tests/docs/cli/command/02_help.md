# Test: `help`

Integration test planning for help output. See [command/04_help.md](../../../../docs/cli/command/04_help.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `clr --help` → help output, exit 0 | Happy Path |
| IT-2 | `clr -h` → same as `--help` | Alias |
| IT-3 | Help output lists `--system-prompt`, `--append-system-prompt`, `--no-ultrathink`, `--effort`, and `--no-effort-max` | Completeness |
| IT-4 | `--help` anywhere in argv → help wins | Override |
| IT-5 | `--help` wins even when unknown flags are present | Override |
| IT-6 | `--help` output goes to stdout; stderr is empty | Output Stream |
| IT-7 | `-h` output is byte-identical to `--help` output | Alias |
| IT-8 | Help output is stable across repeated invocations | Stability |
| EC-01 | stdout contains `"RUNNER OPTIONS:"` section header | Section Split |
| EC-02 | stdout contains `"CLAUDE CODE OPTIONS (forwarded):"` section header | Section Split |
| EC-03 | exactly 8 lines start with `"  clr "` (usage forms) | Section Split |
| EC-04 | stdout contains `"Commands:"` section | Section Split |
| EC-05 | stdout contains `--model`, `--timeout`, `--max-sessions` | Section Split |
| EC-06 | stdout does NOT contain `"\nOPTIONS:\n"` as standalone section header | Section Split |

## Test Coverage Summary

- Happy Path: 1 test
- Alias: 2 tests
- Completeness: 1 test
- Override: 2 tests
- Output Stream: 1 test
- Stability: 1 test
- Section Split: 6 tests (EC-01–EC-06, `cli_args_ext_test.rs`)

**Total:** 14 tests

---

### IT-1: `clr --help` → help output, exit 0

- **Command:** `clr --help`
- **Expected behavior:** Contains "Commands:", "RUNNER OPTIONS:", "CLAUDE CODE OPTIONS (forwarded):", known flags
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)

---

### IT-2: `clr -h` → same as `--help`

- **Command:** `clr -h`
- **Expected behavior:** Same as `clr --help`
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)

---

### IT-3: Help lists `--system-prompt`, `--append-system-prompt`, `--no-ultrathink`, `--effort`, and `--no-effort-max`

- **Command:** `clr --help`
- **Expected behavior:** Contains `--system-prompt`, `--append-system-prompt`, `--no-ultrathink`, `--effort`, and `--no-effort-max`
- **Exit:** 0
- **Source:** [--system-prompt](../../../../docs/cli/param/015_system_prompt.md), [--append-system-prompt](../../../../docs/cli/param/016_append_system_prompt.md), [--no-ultrathink](../../../../docs/cli/param/014_no_ultrathink.md), [--effort](../../../../docs/cli/param/017_effort.md), [--no-effort-max](../../../../docs/cli/param/018_no_effort_max.md)

---

### IT-4: `--help` anywhere in argv → help wins

- **Command:** `clr --model sonnet --help "Fix bug"`
- **Expected behavior:** Help output shown; not an execution
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)

---

### IT-5: `--help` wins even when unknown flags are present

- **Command:** `clr --unknown-flag --help "msg"` and `clr --help --unknown-flag "msg"`
- **Expected behavior:** Help shown; exit 0 for both orderings; unknown flags are ignored when `--help` is present
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)

---

### IT-6: `--help` output goes to stdout; stderr is empty

- **Command:** `clr --help`
- **Expected behavior:** stdout is non-empty; stderr is empty
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)

---

### IT-7: `-h` output is byte-identical to `--help` output

- **Command:** `clr -h` and `clr --help`
- **Expected behavior:** stdout of both invocations is byte-identical
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)

---

### IT-8: Help output is stable across repeated invocations

- **Command:** `clr --help` (run 3 times)
- **Expected behavior:** all 3 stdout captures are byte-identical
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)

---

### EC-01: stdout contains `"RUNNER OPTIONS:"` section header

- **Command:** `clr --help`
- **Expected behavior:** stdout contains the string `"RUNNER OPTIONS:"`
- **Exit:** 0
- **Source:** `tests/cli_args_ext_test.rs::ec01_help_contains_runner_options_section`

---

### EC-02: stdout contains `"CLAUDE CODE OPTIONS (forwarded):"` section header

- **Command:** `clr --help`
- **Expected behavior:** stdout contains the string `"CLAUDE CODE OPTIONS (forwarded):"`
- **Exit:** 0
- **Source:** `tests/cli_args_ext_test.rs::ec02_help_contains_claude_code_options_section`

---

### EC-03: exactly 8 lines start with `"  clr "` (usage forms)

- **Command:** `clr --help`
- **Expected behavior:** splitting stdout by newline and counting lines that start with `"  clr "` (two-space indent) yields exactly 8
- **Exit:** 0
- **Source:** `tests/cli_args_ext_test.rs::ec03_help_has_eight_usage_forms`

---

### EC-04: stdout contains `"Commands:"` section

- **Command:** `clr --help`
- **Expected behavior:** stdout contains the string `"Commands:"`
- **Exit:** 0
- **Source:** `tests/cli_args_ext_test.rs::ec04_help_contains_commands_section`

---

### EC-05: stdout contains `--model`, `--timeout`, `--max-sessions`

- **Command:** `clr --help`
- **Expected behavior:** stdout contains `"--model"`, `"--timeout"`, and `"--max-sessions"` (regression: rewrite preserves all option names across both groups)
- **Exit:** 0
- **Source:** `tests/cli_args_ext_test.rs::ec05_help_contains_key_flags_across_groups`

---

### EC-06: stdout does NOT contain `"\nOPTIONS:\n"` as standalone section header

- **Command:** `clr --help`
- **Expected behavior:** the old flat `OPTIONS:` section header is absent; it has been replaced by the named option groups `RUNNER OPTIONS:` and `CLAUDE CODE OPTIONS (forwarded):`
- **Exit:** 0
- **Source:** `tests/cli_args_ext_test.rs::ec06_help_no_standalone_options_header`
