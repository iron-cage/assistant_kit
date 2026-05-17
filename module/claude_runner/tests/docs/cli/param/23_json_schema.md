# Parameter :: `--json-schema`

Edge case coverage for the `--json-schema` parameter. See [23_json_schema.md](../../../../docs/cli/param/23_json_schema.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--json-schema` with valid JSON → forwarded to assembled command | Behavioral Divergence |
| EC-2 | Default (no `--json-schema`) → no `--json-schema` in assembled command | Behavioral Divergence |
| EC-3 | `--json-schema` with complex schema object → forwarded verbatim | Edge Case |
| EC-4 | `--help` output contains `--json-schema` | Documentation |
| EC-5 | `--json-schema` + `--model` → both forwarded, no conflict | Interaction |
| EC-6 | `--json-schema` without message → accepted; schema in assembled command | Edge Case |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-2)
- Edge Case: 2 tests
- Interaction: 1 test
- Documentation: 1 test

**Total:** 6 edge cases

---

### EC-1: `--json-schema` value forwarded to assembled command

- **Given:** clean environment
- **When:** `clr --dry-run --json-schema '{"type":"object"}' "task"`
- **Then:** Assembled command contains `--json-schema` with the provided value
- **Exit:** 0
- **Source:** [--json-schema](../../../../docs/cli/param/23_json_schema.md)

---

### EC-2: Default → no `--json-schema` in assembled command

- **Given:** clean environment
- **When:** `clr --dry-run "task"`
- **Then:** Assembled command does NOT contain `--json-schema`
- **Exit:** 0
- **Source:** [--json-schema](../../../../docs/cli/param/23_json_schema.md)

---

### EC-3: Complex schema forwarded verbatim

- **Given:** clean environment
- **When:** `clr --dry-run --json-schema '{"type":"object","properties":{"n":{"type":"string"}},"required":["n"]}' "task"`
- **Then:** Assembled command contains `--json-schema` with full schema string
- **Exit:** 0
- **Source:** [--json-schema](../../../../docs/cli/param/23_json_schema.md)

---

### EC-4: `--help` lists `--json-schema`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--json-schema`
- **Exit:** 0
- **Source:** [command.md — help](../../../../docs/cli/command.md#command--2-help)

---

### EC-5: `--json-schema` + `--model` → both forwarded

- **Given:** clean environment
- **When:** `clr --dry-run --json-schema '{"type":"string"}' --model sonnet "task"`
- **Then:** Assembled command contains both `--json-schema` and `--model sonnet`
- **Exit:** 0
- **Source:** [--json-schema](../../../../docs/cli/param/23_json_schema.md)

---

### EC-6: `--json-schema` without message

- **Given:** clean environment
- **When:** `clr --dry-run --json-schema '{"type":"string"}'`
- **Then:** Exit 0; assembled command contains `--json-schema`
- **Exit:** 0
- **Source:** [--json-schema](../../../../docs/cli/param/23_json_schema.md)
