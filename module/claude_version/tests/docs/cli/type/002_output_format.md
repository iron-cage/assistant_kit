# Test: `OutputFormat`

Type compliance and validation tests for `OutputFormat`. See [type/02_output_format.md](../../../../docs/cli/type/02_output_format.md) for specification.

### Scope

- **Purpose**: Validate OutputFormat parsing, case-sensitivity enforcement, and output structure.
- **Responsibility**: Valid variants, invalid inputs, default behavior, and observable output differences between formats.
- **Commands:** `.status`, `.version.show`, `.version.install`, `.version.list`, `.version.guard`, `.version.history`, `.processes`, `.processes.kill`, `.settings.show`, `.settings.get`
- **In Scope**: Format string parsing, case-sensitive matching, and observable output format differences.
- **Out of Scope**: Per-command JSON schema structure (→ `../command/`), parameter interactions (→ `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `format::text` → human-readable labeled output | Valid: text |
| TC-2 | `format::json` → valid JSON object output | Valid: json |
| EC-1 | Absent `format::` → defaults to text | Default |
| EC-2 | `format::TEXT` → exit 1 (case-sensitive) | Validation: case |
| EC-3 | `format::Json` → exit 1 (case-sensitive) | Validation: case |
| EC-4 | `format::xml` → exit 1 (unknown variant) | Validation: unknown |
| EC-5 | `format::` (empty) → exit 1 | Validation: empty |

## Test Coverage Summary

- Valid format: 2 tests (TC-1, TC-2)
- Default Behavior: 1 test (EC-1)
- Case sensitivity: 2 tests (EC-2, EC-3)
- Unknown variant: 1 test (EC-4)
- Empty value: 1 test (EC-5)

**Total:** 7 tests

**Behavioral Divergence Pair:** TC-1 (`cm .status format::text` → labeled `key: value` pairs) ↔ TC-2 (`cm .status format::json` → valid JSON object with machine-readable structure)

---

### TC-1: `format::text` → human-readable output

- **Given:** clean environment with Claude Code installed
- **When:** `cm .status format::text`
- **Then:** output is human-readable labeled text (e.g., "version: X.Y.Z"); not parseable as JSON
- **Exit:** 0
- **Source:** [type/02_output_format.md — text: human-readable](../../../../docs/cli/type/02_output_format.md)

---

### TC-2: `format::json` → valid JSON output

- **Given:** clean environment with Claude Code installed
- **When:** `cm .status format::json`
- **Then:** output is valid JSON (parseable by `jq .`); contains a top-level object with version field
- **Exit:** 0
- **Source:** [type/02_output_format.md — json: machine-readable](../../../../docs/cli/type/02_output_format.md)

---

### EC-1: Absent `format::` → defaults to text

- **Given:** clean environment
- **When:** `cm .status` (no `format::` parameter)
- **Then:** output matches `format::text` output; not JSON
- **Exit:** 0
- **Source:** [type/02_output_format.md — Default: text](../../../../docs/cli/type/02_output_format.md)

---

### EC-2: `format::TEXT` → exit 1

- **Given:** clean environment
- **When:** `cm .status format::TEXT`
- **Then:** exit code 1; stderr contains error message referencing unknown format or case-sensitivity
- **Exit:** 1
- **Source:** [type/02_output_format.md — Parsing: exact string match](../../../../docs/cli/type/02_output_format.md)

---

### EC-3: `format::Json` → exit 1

- **Given:** clean environment
- **When:** `cm .status format::Json`
- **Then:** exit code 1; stderr contains "unknown format" or similar message
- **Exit:** 1
- **Source:** [type/02_output_format.md — Validation errors: unknown format](../../../../docs/cli/type/02_output_format.md)

---

### EC-4: `format::xml` → exit 1

- **Given:** clean environment
- **When:** `cm .status format::xml`
- **Then:** exit code 1; stderr contains "unknown format" message mentioning expected values
- **Exit:** 1
- **Source:** [type/02_output_format.md — Validation errors: expected text or json](../../../../docs/cli/type/02_output_format.md)

---

### EC-5: `format::` (empty) → exit 1

- **Given:** clean environment
- **When:** `cm .status format::`
- **Then:** exit code 1; error message references `format::` or unknown format
- **Exit:** 1
- **Source:** [type/02_output_format.md — Validation errors](../../../../docs/cli/type/02_output_format.md)

---

### Source Functions

Pending implementation. See [task 176](../../../../../task/claude_version/unverified/176_cli_type_test_surface.md).
