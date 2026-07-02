# Parameter :: `--add-dir`

Edge case tests for the add-dir parameter. Tests validate value forwarding, missing-value rejection, and help documentation.

**Source:** [066_add_dir.md](../../../../docs/cli/param/066_add_dir.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--add-dir /tmp` → flag forwarded to claude | Behavioral Divergence |
| EC-2 | `--add-dir` without value → exit 1 | Missing Value |
| EC-3 | `--add-dir` at end of argv → exit 1 | Boundary Values |
| EC-4 | Any path string accepted (no validation) | Permissive |
| EC-5 | `--help` lists `--add-dir` | Documentation |
| EC-6 | Without `--add-dir` → no `--add-dir` flag in assembled command | Behavioral Divergence |
| EC-7 | `CLR_ADD_DIR=/tmp` env var → forwarded | Env Var |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-6)
- Missing Value: 1 test (EC-2)
- Boundary Values: 1 test (EC-3)
- Permissive: 1 test (EC-4)
- Documentation: 1 test (EC-5)
- Env Var: 1 test (EC-7)

**Total:** 7 edge cases

## Test Cases
---

### EC-1: `--add-dir /tmp` → forwarded to assembled command

- **Given:** clean environment
- **When:** `clr --dry-run --add-dir /tmp "Fix bug"`
- **Then:** Assembled command contains `--add-dir /tmp`
- **Exit:** 0
- **Source:** [066_add_dir.md](../../../../docs/cli/param/066_add_dir.md)
- **Commands:** run, ask
---

### EC-2: `--add-dir` without value → exit 1

- **Given:** clean environment
- **When:** `clr --add-dir`
- **Then:** Exit 1; error about missing `--add-dir` value
- **Exit:** 1
- **Source:** [066_add_dir.md](../../../../docs/cli/param/066_add_dir.md)
- **Commands:** run, ask
---

### EC-3: `--add-dir` at end of argv → exit 1

- **Given:** clean environment
- **When:** `clr "Fix bug" --add-dir`
- **Then:** Exit 1; error about missing `--add-dir` value
- **Exit:** 1
- **Source:** [066_add_dir.md](../../../../docs/cli/param/066_add_dir.md)
- **Commands:** run, ask
---

### EC-4: Any path string accepted without validation

- **Given:** clean environment
- **When:** `clr --dry-run --add-dir /nonexistent/path "Fix bug"`
- **Then:** Assembled command contains `--add-dir /nonexistent/path`; no rejection
- **Exit:** 0
- **Source:** [066_add_dir.md](../../../../docs/cli/param/066_add_dir.md)
- **Commands:** run, ask
---

### EC-5: `--help` lists `--add-dir`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--add-dir`
- **Exit:** 0
- **Source:** [command/02_help.md](../../../../docs/cli/command/02_help.md)
- **Commands:** run, ask
---

### EC-6: Without `--add-dir` → no `--add-dir` flag in assembled command

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command does NOT contain `--add-dir`
- **Exit:** 0
- **Source:** [066_add_dir.md](../../../../docs/cli/param/066_add_dir.md)
- **Commands:** run, ask
---

### EC-7: `CLR_ADD_DIR=/tmp` env var → forwarded

- **Given:** `CLR_ADD_DIR=/tmp`
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--add-dir /tmp`
- **Exit:** 0
- **Source:** [066_add_dir.md](../../../../docs/cli/param/066_add_dir.md)
- **Commands:** run, ask
