# Type :: `JsonSchemaText`

Validation tests for the `JsonSchemaText` semantic type (String: JSON object text). Tests
verify that schema values are accepted and forwarded verbatim to the claude subprocess;
structural validation is deferred to claude, not enforced by the runner.

**Source:** [type.md](../../../../docs/cli/type.md#type--10-jsonschematext)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Simple object schema → accepted and forwarded | Valid Input |
| TC-2 | Empty string → forwarded (validation deferred to claude) | Edge Case |
| TC-3 | Complex nested schema → forwarded verbatim | Valid Input |
| TC-4 | Schema from file via command substitution → forwarded | Valid Input |

## Test Coverage Summary

- Valid Input: 3 tests (TC-1, TC-3, TC-4)
- Edge Case: 1 test (TC-2)

**Total:** 4 test cases

## Test Cases

---

### TC-1: Simple object schema → accepted and forwarded

- **Given:** clean environment
- **When:** `clr --dry-run --json-schema '{"type":"object"}' "task"`
- **Then:** Exit 0; assembled command contains `--json-schema` with the value `{"type":"object"}`
- **Exit:** 0
- **Source:** [type.md — JsonSchemaText](../../../../docs/cli/type.md#type--10-jsonschematext)

---

### TC-2: Empty string → forwarded without parse error

- **Given:** clean environment
- **When:** `clr --dry-run --json-schema "" "task"`
- **Then:** Exit 0; assembled command contains `--json-schema ""`; clr does not reject the empty string — validation is deferred to claude
- **Exit:** 0
- **Source:** [type.md — JsonSchemaText](../../../../docs/cli/type.md#type--10-jsonschematext)

---

### TC-3: Complex nested schema → forwarded verbatim

- **Given:** clean environment
- **When:** `clr --dry-run --json-schema '{"type":"object","properties":{"name":{"type":"string"},"count":{"type":"integer"}},"required":["name"]}' "task"`
- **Then:** Exit 0; assembled command contains `--json-schema` with the full nested JSON string, unchanged
- **Exit:** 0
- **Source:** [type.md — JsonSchemaText](../../../../docs/cli/type.md#type--10-jsonschematext)

---

### TC-4: Schema from file via shell substitution → forwarded

- **Given:** a JSON schema file exists at `/tmp/tc4_schema.json` containing `{"type":"string"}`
- **When:** `clr --dry-run --json-schema "$(cat /tmp/tc4_schema.json)" "task"`
- **Then:** Exit 0; assembled command contains `--json-schema` with the file contents as the value
- **Exit:** 0
- **Source:** [type.md — JsonSchemaText](../../../../docs/cli/type.md#type--10-jsonschematext)
