# Test: `format::`

Edge case coverage for the `format::` parameter. See [params.md](../../../../docs/cli/params.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-100 | `.status format::json` → `{"version":...}` | Explicit json |
| TC-111 | `.version.show format::json` → `{"version":"..."}` | Explicit json |
| TC-121 | `.version.list format::json` → JSON array | Explicit json |
| TC-144 | `.processes format::json` → `{"processes":[...]}` | Explicit json |
| TC-167 | `.settings.show format::json` → JSON object | Explicit json |
| TC-182 | `.settings.get format::json` → `{"key":..,"value":..}` | Explicit json |
| EC-5 | `format::json` preserves bool/number types | Type Fidelity |
| TC-431 | `.version.history format::json` → version/date/summary fields | Explicit json |
| EC-1 | `.version.guard format::json dry::1` → JSON output, exit 0 | Explicit json |
| EC-2 | `format::xml` → exit 1, unknown format | Invalid |
| EC-3 | `format::JSON` (uppercase) → exit 1 | Invalid (case) |
| EC-4 | `format::` (empty) → exit 1 | Empty Value |
| TC-440 | `.version.history format::xml` → exit 1 | Invalid |
| TC-441 | `.version.history format::JSON` → exit 1 | Invalid (case) |
| EC-1 | Default (absent) → `format::text` | Default Behavior |
| EC-2 | `format::text` explicit → same as absent | Explicit text |
| EC-3 | `format::csv` → exit 1 | Invalid |
| EC-4 | `format::` only for output-returning commands | Command Scope |
| EC-5 | JSON output always starts with `{` or `[` depending on command | Structure |

## Test Coverage Summary

- Explicit json: 8 tests
- Type Fidelity: 1 test
- Invalid: 3 tests
- Invalid (case-sensitive): 2 tests
- Empty Value: 1 test
- Default Behavior: 1 test
- Explicit text: 1 test
- Command Scope: 1 test
- JSON Structure: 1 test

**Total:** 19 edge cases

**Behavioral Divergence Pair:** EC-5 (valid/expected path) ↔ EC-1 (invalid/rejected path)

---

### EC-1: `.version.guard format::json dry::1` → JSON output

- **Given:** clean environment
- **When:** `cm .version.guard format::json dry::1`
- **Then:** exit code 0; stdout starts with `{`.; JSON output
- **Exit:** 0
- **Source:** [commands.md — .version.guard](../../../../docs/cli/commands.md#command--5-versionguard)

---

### EC-2: `format::xml` → exit 1

- **Given:** clean environment
- **When:** `cm .status format::xml` (cross-cutting — applies to all format-accepting commands).
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-3: `format::JSON` (uppercase) → exit 1

- **Given:** clean environment
- **When:** `cm .status format::JSON`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-4: `format::` (empty) → exit 1

- **Given:** clean environment
- **When:** `cm .status format::`
- **Then:** exit code 1; error mentions format:: value.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-5: `format::json` preserves bool/number types

- **Given:** `HOME=<tmp>`; `settings.json` has `"flag": true` and `"count": 42`.
- **When:** `cm .settings.show format::json`
- **Then:** exit code 0; output contains unquoted `true` and `42`.; type-faithful JSON
- **Exit:** 0
- **Source:** [feature/003_settings_management.md](../../../../docs/feature/003_settings_management.md)

---

### EC-1: Default (absent) → `format::text`

- **Given:** clean environment
- **When:** `cm .status`
- **Then:** Human-readable text (not JSON).; Output does not start with `{`
- **Exit:** 0
- **Source:** [params.md — format:: default: text](../../../../docs/cli/params.md)

---

### EC-3: `format::csv` → exit 1

- **Given:** clean environment
- **When:** `cm .status format::csv`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-4: `format::` only for output-returning commands

- **Given:** clean environment
- **When:** `cm .settings.set format::json`
- **Then:** exit code 1; unknown parameter.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-5: JSON structure per command

- **Given:** clean environment
- **When:** 
- **Then:** see spec
- **Exit:** 0
