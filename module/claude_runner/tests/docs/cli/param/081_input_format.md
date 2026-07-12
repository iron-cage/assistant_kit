# Parameter :: `--input-format`

Edge case tests for the input format parameter. Tests validate value forwarding for both enum values, invalid-value rejection, and help documentation.

**Source:** [081_input_format.md](../../../../docs/cli/param/081_input_format.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `--input-format stream-json` → forwarded to assembled command | Behavioral Divergence |
| IT-1b | `--input-format text` → forwarded to assembled command | Behavioral Divergence |
| IT-2 | `--input-format badvalue` → exit 1, stderr names valid values | Validation |
| IT-3 | `--help` lists `--input-format` | Documentation |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (IT-1, IT-1b)
- Validation: 1 test (IT-2)
- Documentation: 1 test (IT-3)

**Total:** 4 edge cases

## Test Cases
---

### IT-1: `--input-format stream-json` → forwarded to assembled command

- **Given:** clean environment
- **When:** `clr --dry-run --input-format stream-json "hi"`
- **Then:** Assembled command contains `--input-format` and the value `stream-json`
- **Exit:** 0
- **Source:** [081_input_format.md](../../../../docs/cli/param/081_input_format.md)
- **Commands:** run, ask
---

### IT-1b: `--input-format text` → forwarded to assembled command

- **Given:** clean environment
- **When:** `clr --dry-run --input-format text "hi"`
- **Then:** Assembled command contains `--input-format` and the value `text`
- **Exit:** 0
- **Source:** [081_input_format.md](../../../../docs/cli/param/081_input_format.md)
- **Commands:** run, ask
---

### IT-2: `--input-format badvalue` → exit 1, stderr names valid values

- **Given:** clean environment
- **When:** `clr --input-format badvalue "hi"`
- **Then:** Exit 1; stderr names both valid values (`text`, `stream-json`)
- **Exit:** 1
- **Source:** [081_input_format.md](../../../../docs/cli/param/081_input_format.md)
- **Commands:** run, ask
---

### IT-3: `--help` lists `--input-format`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--input-format`
- **Exit:** 0
- **Source:** [command/02_help.md](../../../../docs/cli/command/02_help.md)
- **Commands:** run, ask
