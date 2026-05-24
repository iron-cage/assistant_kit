# Parameter :: `--append-system-prompt`

Edge case coverage for the `--append-system-prompt` parameter. See [016_append_system_prompt.md](../../../../docs/cli/param/016_append_system_prompt.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--append-system-prompt "text"` → flag and value in dry-run output | Behavioral Divergence |
| EC-2 | `--append-system-prompt` without value → exit 1 | Missing Value |
| EC-3 | `--append-system-prompt ""` (empty string) → forwarded to claude | Behavioral Divergence |
| EC-4 | `--append-system-prompt` + `--system-prompt` together → both forwarded | Interaction |
| EC-5 | `--help` output contains `--append-system-prompt` | Documentation |
| EC-6 | `--append-system-prompt` value with spaces (quoted) → forwarded as single argument | Quoting |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-3)
- Missing Value: 1 test
- Interaction: 1 test
- Documentation: 1 test
- Quoting: 1 test

**Total:** 6 edge cases


---

### EC-1: `--append-system-prompt "text"` → appears in dry-run output

- **Given:** clean environment
- **When:** `clr --dry-run --append-system-prompt "Always JSON." "test"`
- **Then:** Command line contains `--append-system-prompt` and `Always JSON.`.; flag and value both present
- **Exit:** 0
- **Source:** [--append-system-prompt](../../../../docs/cli/param/016_append_system_prompt.md)
- **Commands:** run, ask

---

### EC-2: `--append-system-prompt` without value → exit 1

- **Given:** clean environment
- **When:** `clr --append-system-prompt`
- **Then:** Exit code 1; stderr contains "--append-system-prompt requires a value".; error message shown
- **Exit:** 1
- **Source:** [--append-system-prompt validation](../../../../docs/cli/param/016_append_system_prompt.md)
- **Commands:** run, ask

---

### EC-3: `--append-system-prompt ""` → forwarded without rejection

- **Given:** clean environment
- **When:** `clr --dry-run --append-system-prompt "" "test"`
- **Then:** Exit 0; command assembled (empty append forwarded).; no rejection of empty string
- **Exit:** 0
- **Source:** [--append-system-prompt](../../../../docs/cli/param/016_append_system_prompt.md)
- **Commands:** run, ask

---

### EC-4: `--append-system-prompt` + `--system-prompt` together → both forwarded

- **Given:** clean environment
- **When:** `clr --dry-run --system-prompt "Base." --append-system-prompt "Extra." "test"`
- **Then:** Output contains both `--system-prompt` and `--append-system-prompt`.; both flags present
- **Exit:** 0
- **Source:** [--system-prompt](../../../../docs/cli/param/015_system_prompt.md)
- **Commands:** run, ask

---

### EC-5: `--help` lists `--append-system-prompt`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--append-system-prompt`.; flag present in help
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)
- **Commands:** run, ask

---

### EC-6: `--append-system-prompt` value with spaces → forwarded as single argument

- **Given:** clean environment
- **When:** `clr --dry-run --append-system-prompt "Always respond in JSON." "test"`
- **Then:** `--append-system-prompt` value `Always respond in JSON.` is forwarded as a single argument (not split on spaces)
- **Exit:** 0
- **Source:** [--append-system-prompt](../../../../docs/cli/param/016_append_system_prompt.md)
- **Commands:** run, ask
