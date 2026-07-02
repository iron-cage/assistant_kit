# Parameter :: `--json-schema`

Edge case coverage for the `--json-schema` parameter. See [023_json_schema.md](../../../../docs/cli/param/023_json_schema.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--json-schema` with valid JSON → forwarded to assembled command | Behavioral Divergence |
| EC-2 | Default (no `--json-schema`) → no `--json-schema` in assembled command | Behavioral Divergence |
| EC-3 | `--json-schema` with complex schema object → forwarded verbatim | Edge Case |
| EC-4 | `--help` output contains `--json-schema` | Documentation |
| EC-5 | `--json-schema` + `--model` → both forwarded, no conflict | Interaction |
| EC-6 | `--json-schema` without message → accepted; schema in assembled command | Edge Case |
| EC-7 | `--output-style raw --json-schema` → stdout contains structured JSON (BUG-318) | Cross-Product |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-2)
- Edge Case: 2 tests
- Interaction: 1 test
- Documentation: 1 test
- Cross-Product: 1 test (EC-7)

**Total:** 7 edge cases

---

### EC-1: `--json-schema` value forwarded to assembled command

- **Given:** clean environment
- **When:** `clr --dry-run --json-schema '{"type":"object"}' "task"`
- **Then:** Assembled command contains `--json-schema` with the provided value
- **Exit:** 0
- **Source:** [--json-schema](../../../../docs/cli/param/023_json_schema.md)
- **Commands:** run, ask

---

### EC-2: Default → no `--json-schema` in assembled command

- **Given:** clean environment
- **When:** `clr --dry-run "task"`
- **Then:** Assembled command does NOT contain `--json-schema`
- **Exit:** 0
- **Source:** [--json-schema](../../../../docs/cli/param/023_json_schema.md)
- **Commands:** run, ask

---

### EC-3: Complex schema forwarded verbatim

- **Given:** clean environment
- **When:** `clr --dry-run --json-schema '{"type":"object","properties":{"n":{"type":"string"}},"required":["n"]}' "task"`
- **Then:** Assembled command contains `--json-schema` with full schema string
- **Exit:** 0
- **Source:** [--json-schema](../../../../docs/cli/param/023_json_schema.md)
- **Commands:** run, ask

---

### EC-4: `--help` lists `--json-schema`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--json-schema`
- **Exit:** 0
- **Source:** [command/02_help.md](../../../../docs/cli/command/02_help.md)
- **Commands:** run, ask

---

### EC-5: `--json-schema` + `--model` → both forwarded

- **Given:** clean environment
- **When:** `clr --dry-run --json-schema '{"type":"string"}' --model sonnet "task"`
- **Then:** Assembled command contains both `--json-schema` and `--model sonnet`
- **Exit:** 0
- **Source:** [--json-schema](../../../../docs/cli/param/023_json_schema.md)
- **Commands:** run, ask

---

### EC-6: `--json-schema` without message

- **Given:** clean environment
- **When:** `clr --dry-run --json-schema '{"type":"string"}'`
- **Then:** Exit 0; assembled command contains `--json-schema`
- **Exit:** 0
- **Source:** [--json-schema](../../../../docs/cli/param/023_json_schema.md)
- **Commands:** run, ask

---

### EC-7: `--output-style raw --json-schema` → stdout contains structured JSON (BUG-318)

- **Given:** fake claude emitting CLR JSON envelope with `"structured_output":{"x":"hello"}` and `"result":""`; `-p --max-sessions 0`; `--output-style raw`; `--json-schema '{"type":"object","properties":{"x":{"type":"string"}},"required":["x"]}'`
- **When:** `clr -p --max-sessions 0 --output-style raw --json-schema '...' "test"` with fake claude
- **Then:** Exit 0; stdout is non-empty; stdout contains `"x"` (structured output extracted from JSON envelope); raw mode + json-schema does not produce empty stdout
- **Exit:** 0
- **Source:** [--json-schema](../../../../docs/cli/param/023_json_schema.md) Known Limitation (BUG-318); [--output-style](../../../../docs/cli/param/070_output_style.md) Combinations table
- **Commands:** run, ask
