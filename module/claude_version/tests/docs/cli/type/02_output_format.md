# Test: `OutputFormat`

Type validation cases for the `OutputFormat` type. See [02_output_format.md](../../../../docs/cli/type/02_output_format.md) for specification.

### Scope

- **Purpose**: Type validation tests for `OutputFormat` (enum: `text` | `json`).
- **Responsibility**: Valid variant acceptance, invalid variant rejection, case-sensitivity, and default behavior.
- **Used by:** `format::` parameter
- **In Scope**: Exact-match parsing, case-sensitive rejection, unknown value rejection.
- **Out of Scope**: Output content correctness (→ `../command/`), group interactions (→ `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `format::text` → accepted, human-readable output | Valid minimum variant |
| TC-2 | `format::json` → accepted, machine-readable JSON output | Valid maximum variant |
| TC-3 | `format::TEXT` → exit 1, wrong case | Invalid: case-sensitive |
| TC-4 | `format::Json` → exit 1, wrong case | Invalid: case-sensitive |
| TC-5 | `format::xml` → exit 1, unknown variant | Invalid: above range |
| TC-6 | `format::` (empty) → exit 1 | Invalid: empty |
| TC-7 | absent `format::` → defaults to text | Default behavior |

## Test Coverage Summary

- Valid variants (text, json): 2 tests (TC-1, TC-2)
- Invalid case-sensitive: 2 tests (TC-3, TC-4)
- Invalid unknown variant: 1 test (TC-5)
- Invalid empty: 1 test (TC-6)
- Default behavior: 1 test (TC-7)

**Total:** 7 type cases

**Behavioral Divergence Pair:** TC-1 (`format::text` → human-readable labeled output) ↔ TC-2 (`format::json` → machine-readable JSON structure). Both valid; output encoding differs fundamentally.

---

### TC-1: `format::text` → accepted, human-readable

- **Given:** clean environment
- **When:** `cm .version.list format::text`
- **Then:** exit 0; output is human-readable text with labels; not JSON
- **Exit:** 0
- **Source:** [02_output_format.md — Valid values: text](../../../../docs/cli/type/02_output_format.md)

---

### TC-2: `format::json` → accepted, machine-readable JSON

- **Given:** clean environment
- **When:** `cm .version.list format::json`
- **Then:** exit 0; output parses as valid JSON; not human-readable text format
- **Exit:** 0
- **Source:** [02_output_format.md — Valid values: json](../../../../docs/cli/type/02_output_format.md)

---

### TC-3: `format::TEXT` → exit 1, wrong case

- **Given:** clean environment
- **When:** `cm .status format::TEXT`
- **Then:** exit 1; error message contains "unknown format" or "expected text or json"
- **Exit:** 1
- **Source:** [02_output_format.md — Parsing: exact string match](../../../../docs/cli/type/02_output_format.md)

---

### TC-4: `format::Json` → exit 1, wrong case

- **Given:** clean environment
- **When:** `cm .status format::Json`
- **Then:** exit 1; mixed-case rejected same as uppercase
- **Exit:** 1
- **Source:** [02_output_format.md — Parsing: Text, JSON, Json all rejected](../../../../docs/cli/type/02_output_format.md)

---

### TC-5: `format::xml` → exit 1, unknown variant

- **Given:** clean environment
- **When:** `cm .status format::xml`
- **Then:** exit 1; error message contains "unknown format 'xml': expected text or json"
- **Exit:** 1
- **Source:** [02_output_format.md — validation errors](../../../../docs/cli/type/02_output_format.md)

---

### TC-6: `format::` (empty) → exit 1

- **Given:** clean environment
- **When:** `cm .status format::`
- **Then:** exit 1; empty string is not a valid variant
- **Exit:** 1
- **Source:** [02_output_format.md — Valid values: text, json](../../../../docs/cli/type/02_output_format.md)

---

### TC-7: absent `format::` → defaults to text

- **Given:** clean environment
- **When:** `cm .version.list`
- **Then:** output is human-readable text (not JSON); default variant `text` applied
- **Exit:** 0
- **Source:** [02_output_format.md — Default: text](../../../../docs/cli/type/02_output_format.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc258_status_format_json_is_valid_json` | `integration/cross_cutting_test.rs` |
| `tc260_format_uppercase_rejected` | `integration/cross_cutting_test.rs` |
| `tc261_version_install_format_json_accepted` | `integration/cross_cutting_test.rs` |
| ⏳ `tc_output_format_text_explicit` | `cli_args_test.rs` |
| ⏳ `tc_output_format_xml_rejected` | `cli_args_test.rs` |
| ⏳ `tc_output_format_empty_rejected` | `cli_args_test.rs` |
