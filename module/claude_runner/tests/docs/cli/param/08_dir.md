# Parameter :: `--dir`

Edge case tests for the working directory parameter. Tests validate path forwarding, missing-value rejection, and help documentation.

**Source:** [08_dir.md](../../../../docs/cli/param/08_dir.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--dir /some/path` → forwarded to assembled command | Behavioral Divergence |
| EC-2 | `--dir` without value → exit 1 | Missing Value |
| EC-3 | Default → cwd used (no `--dir` in assembled command) | Behavioral Divergence |
| EC-4 | `--dir` with non-existent path → accepted (not validated at this layer) | Permissive |
| EC-5 | `--help` lists `--dir` | Documentation |
| EC-6 | `--dir` + message → both forwarded correctly | Interaction |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-3)
- Missing Value: 1 test (EC-2)
- Permissive: 1 test (EC-4)
- Documentation: 1 test (EC-5)
- Interaction: 1 test (EC-6)

**Total:** 6 edge cases


## Test Cases
---

### EC-1: `--dir /some/path` forwarded

- **Given:** clean environment
- **When:** `clr --dry-run --dir /tmp "Fix bug"`
- **Then:** Assembled command contains `--dir /tmp`
- **Exit:** 0
- **Source:** [08_dir.md](../../../../docs/cli/param/08_dir.md)
---

### EC-2: `--dir` without value → exit 1

- **Given:** clean environment
- **When:** `clr --dir`
- **Then:** Exit 1; error about missing `--dir` value
- **Exit:** 1
- **Source:** [08_dir.md](../../../../docs/cli/param/08_dir.md)
---

### EC-3: Default → no `--dir` in assembled command

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command does NOT contain `--dir`
- **Exit:** 0
- **Source:** [08_dir.md](../../../../docs/cli/param/08_dir.md)
---

### EC-4: Non-existent path accepted without validation

- **Given:** clean environment
- **When:** `clr --dry-run --dir /no/such/path "Fix bug"`
- **Then:** Exit 0; assembled command contains `--dir /no/such/path` (no path validation at runner layer)
- **Exit:** 0
- **Source:** [08_dir.md](../../../../docs/cli/param/08_dir.md)
---

### EC-5: `--help` lists `--dir`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--dir`
- **Exit:** 0
- **Source:** [command.md](../../../../docs/cli/command.md#command--2-help)
---

### EC-6: `--dir` + message → both forwarded

- **Given:** clean environment
- **When:** `clr --dry-run --dir /workspace "Fix bug"`
- **Then:** Assembled command contains both `--dir /workspace` and the message
- **Exit:** 0
- **Source:** [08_dir.md](../../../../docs/cli/param/08_dir.md)
