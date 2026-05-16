# Parameter :: `--system-prompt`

Edge case coverage for the `--system-prompt` parameter. See [15_system_prompt.md](../../../../docs/cli/param/15_system_prompt.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--system-prompt "text"` → flag and value in dry-run output | Behavioral Divergence |
| EC-2 | `--system-prompt` without value → exit 1 | Missing Value |
| EC-3 | `--system-prompt ""` (empty string) → forwarded to claude | Behavioral Divergence |
| EC-4 | `--system-prompt` + `--append-system-prompt` together → both forwarded | Interaction |
| EC-5 | `--help` output contains `--system-prompt` | Documentation |
| EC-6 | `--system-prompt` value with spaces (quoted) → forwarded as single argument | Quoting |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-3)
- Missing Value: 1 test
- Interaction: 1 test
- Documentation: 1 test
- Quoting: 1 test

**Total:** 6 edge cases


---

### EC-1: `--system-prompt "text"` → appears in dry-run output

- **Given:** clean environment
- **When:** `clr --dry-run --system-prompt "Be concise." "test"`
- **Then:** Command line contains `--system-prompt` and `Be concise.`.; flag and value both present
- **Exit:** 0
- **Source:** [--system-prompt](../../../../docs/cli/param/15_system_prompt.md)

---

### EC-2: `--system-prompt` without value → exit 1

- **Given:** clean environment
- **When:** `clr --system-prompt`
- **Then:** Exit code 1; stderr contains "--system-prompt requires a value".; error message shown
- **Exit:** 1
- **Source:** [--system-prompt validation](../../../../docs/cli/param/15_system_prompt.md)

---

### EC-3: `--system-prompt ""` → forwarded without rejection

- **Given:** clean environment
- **When:** `clr --dry-run --system-prompt "" "test"`
- **Then:** Exit 0; command assembled (empty prompt forwarded).; no rejection of empty string
- **Exit:** 0
- **Source:** [--system-prompt](../../../../docs/cli/param/15_system_prompt.md)

---

### EC-4: `--system-prompt` + `--append-system-prompt` together → both forwarded

- **Given:** clean environment
- **When:** `clr --dry-run --system-prompt "Base." --append-system-prompt "Extra." "test"`
- **Then:** Output contains both `--system-prompt` and `--append-system-prompt`.; both flags present
- **Exit:** 0
- **Source:** [--append-system-prompt](../../../../docs/cli/param/16_append_system_prompt.md)

---

### EC-5: `--help` lists `--system-prompt`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--system-prompt`.; flag present in help
- **Exit:** 0
- **Source:** [command.md — help](../../../../docs/cli/command.md#command--2-help)

---

### EC-6: `--system-prompt` value with spaces → forwarded as single argument

- **Given:** clean environment
- **When:** `clr --dry-run --system-prompt "Be concise and accurate." "test"`
- **Then:** `--system-prompt` value `Be concise and accurate.` is forwarded as a single argument (not split on spaces)
- **Exit:** 0
- **Source:** [--system-prompt](../../../../docs/cli/param/15_system_prompt.md)
