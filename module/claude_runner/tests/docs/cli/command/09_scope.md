# Test: `scope`

Integration test planning for the `scope` command. See [command/09_scope.md](../../../../docs/cli/command/09_scope.md) for specification.

`scope` prints all 6 `CLAUDE_*` path variables for a given directory (default: CWD) in `key=value` format. Tests verify that all 6 variables are printed, that `--dir` overrides the target, that env var overrides are respected, that `CLAUDE_SESSION_FILE` is empty when no session exists, and that error cases return exit 1.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `clr scope` exits 0 | Happy path |
| IT-2 | Stdout contains all 6 `CLAUDE_*` variable names | Content |
| IT-3 | `clr scope --dir /path` prints vars for that directory | Content |
| IT-4 | `CLAUDE_SESSION_FILE=` is empty when no session history | Content |
| IT-5 | `CLAUDE_HOME` override reflected in output | Env override |
| IT-6 | `clr scope --help` exits 0 | Help flag |
| IT-7 | `clr scope -h` exits 0 | Help flag |
| IT-8 | `clr --help` mentions `scope` command | Help listing |
| IT-9 | `clr scope --dir /nonexistent` exits 1 | Error rejection |

## Test Coverage Summary

- Happy path: 1 test (IT-1)
- Content: 3 tests (IT-2, IT-3, IT-4)
- Env override: 1 test (IT-5)
- Help flag: 2 tests (IT-6, IT-7)
- Help listing: 1 test (IT-8)
- Error rejection: 1 test (IT-9)

**Total:** 9 tests

---

### IT-1: `clr scope` exits 0

- **Command:** `clr scope`
- **Expected behavior:** exit 0; stdout is non-empty (6 variable lines printed)
- **Exit:** 0
- **Source:** [command/09_scope.md](../../../../docs/cli/command/09_scope.md)

---

### IT-2: Stdout contains all 6 `CLAUDE_*` variable names

- **Command:** `clr scope`
- **Expected behavior:** exit 0; stdout contains `CLAUDE_HOME`, `CLAUDE_PROJECTS_DIR`, `CLAUDE_SESSION_DIR`, `CLAUDE_MEMORY_DIR`, `CLAUDE_MEMORY_FILE`, `CLAUDE_SESSION_FILE`
- **Exit:** 0
- **Source:** [command/09_scope.md](../../../../docs/cli/command/09_scope.md)

---

### IT-3: `clr scope --dir /path` prints vars for that directory

- **Command:** `clr scope --dir /tmp`
- **Expected behavior:** exit 0; `CLAUDE_SESSION_DIR` line contains a path derived from `/tmp` (encoded `/tmp` segment visible in value)
- **Exit:** 0
- **Source:** [command/09_scope.md](../../../../docs/cli/command/09_scope.md)

---

### IT-4: `CLAUDE_SESSION_FILE=` empty when no session exists

- **Command:** `clr scope --dir /tmp/scope_it4_no_session` (directory with no Claude session storage)
- **Expected behavior:** exit 0; line `CLAUDE_SESSION_FILE=` has empty value (nothing after `=`)
- **Exit:** 0
- **Source:** [command/09_scope.md](../../../../docs/cli/command/09_scope.md)

---

### IT-5: `CLAUDE_HOME` override reflected in output

- **Command:** `CLAUDE_HOME=/tmp/scope_it5_home clr scope --dir /tmp`
- **Expected behavior:** exit 0; `CLAUDE_HOME=/tmp/scope_it5_home`; `CLAUDE_PROJECTS_DIR` starts with `/tmp/scope_it5_home`
- **Exit:** 0
- **Source:** [command/09_scope.md](../../../../docs/cli/command/09_scope.md)

---

### IT-6: `--help` flag

- **Command:** `clr scope --help`
- **Expected behavior:** exit 0; stdout contains `scope` and `--dir`
- **Exit:** 0
- **Source:** [command/09_scope.md](../../../../docs/cli/command/09_scope.md)

---

### IT-7: `-h` short flag

- **Command:** `clr scope -h`
- **Expected behavior:** exit 0; stdout contains `scope`
- **Exit:** 0
- **Source:** [command/09_scope.md](../../../../docs/cli/command/09_scope.md)

---

### IT-8: `clr --help` mentions `scope` command

- **Command:** `clr --help`
- **Expected behavior:** exit 0; stdout contains `scope`
- **Exit:** 0
- **Source:** [command/09_scope.md](../../../../docs/cli/command/09_scope.md)

---

### IT-9: Nonexistent `--dir` path exits 1

- **Command:** `clr scope --dir /tmp/nonexistent_scope_path_it9`
- **Expected behavior:** exit 1; stderr contains error referencing the path or "does not exist"
- **Exit:** 1
- **Source:** [command/09_scope.md](../../../../docs/cli/command/09_scope.md)
