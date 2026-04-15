# Test: `format::`

Edge case coverage for the `format::` parameter. See [params.md](../../params.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-100 | `.status format::json` → `{"version":...}` | Explicit json |
| TC-111 | `.version.show format::json` → `{"version":"..."}` | Explicit json |
| TC-121 | `.version.list format::json` → JSON array | Explicit json |
| TC-144 | `.processes format::json` → `{"processes":[...]}` | Explicit json |
| TC-167 | `.settings.show format::json` → JSON object | Explicit json |
| TC-182 | `.settings.get format::json` → `{"key":..,"value":..}` | Explicit json |
| TC-241 | `format::json` preserves bool/number types | Type Fidelity |
| TC-431 | `.version.history format::json` → version/date/summary fields | Explicit json |
| TC-242 | `format::xml` → exit 1, unknown format | Invalid |
| TC-243 | `format::JSON` (uppercase) → exit 1 | Invalid (case) |
| TC-244 | `format::` (empty) → exit 1 | Empty Value |
| TC-440 | `.version.history format::xml` → exit 1 | Invalid |
| TC-441 | `.version.history format::JSON` → exit 1 | Invalid (case) |
| EC-1 | Default (absent) → `format::text` | Default Behavior |
| EC-2 | `format::text` explicit → same as absent | Explicit text |
| EC-3 | `format::csv` → exit 1 | Invalid |
| EC-4 | `format::` only for output-returning commands | Command Scope |
| EC-5 | JSON output always starts with `{` or `[` depending on command | Structure |

## Test Coverage Summary

- Explicit json: 7 tests
- Type Fidelity: 1 test
- Invalid: 3 tests
- Invalid (case-sensitive): 2 tests
- Empty Value: 1 test
- Default Behavior: 1 test
- Explicit text: 1 test
- Command Scope: 1 test
- JSON Structure: 1 test

**Total:** 18 edge cases

---

### TC-242: `format::xml` → exit 1

**Goal:** Unrecognized format values are rejected at argument validation.
**Setup:** None.
**Command:** `cm .status format::xml` (cross-cutting — applies to all format-accepting commands).
**Expected Output:** exit code 1.
**Pass Criteria:** Exit 1.
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md)

---

### TC-243: `format::JSON` (uppercase) → exit 1

**Goal:** Format matching is case-sensitive.
**Setup:** None.
**Command:** `cm .status format::JSON`
**Expected Output:** exit code 1.
**Pass Criteria:** Exit 1.
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md)

---

### TC-244: `format::` (empty) → exit 1

**Goal:** Empty format string is not a valid value.
**Setup:** None.
**Command:** `cm .status format::`
**Expected Output:** exit code 1; error mentions format:: value.
**Pass Criteria:** Exit 1.
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md)

---

### TC-241: `format::json` preserves bool/number types

**Goal:** JSON output faithfully represents native types from settings, not re-quotes them.
**Setup:** `HOME=<tmp>`; `settings.json` has `"flag": true` and `"count": 42`.
**Command:** `cm .settings.show format::json`
**Expected Output:** exit code 0; output contains unquoted `true` and `42`.
**Pass Criteria:** Exit 0; type-faithful JSON.
**Source:** [feature/003_settings_management.md](../../../feature/003_settings_management.md)

---

### EC-1: Default (absent) → `format::text`

**Goal:** Omitting `format::` defaults to human-readable text output.
**Setup:** None.
**Command:** `cm .status`
**Expected Output:** Human-readable text (not JSON).
**Pass Criteria:** Output does not start with `{`.
**Source:** [params.md — format:: default: text](../../params.md)

---

### EC-3: `format::csv` → exit 1

**Goal:** Any value other than "text" or "json" is rejected.
**Setup:** None.
**Command:** `cm .status format::csv`
**Expected Output:** exit code 1.
**Pass Criteria:** Exit 1.
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md)

---

### EC-4: `format::` only for output-returning commands

**Goal:** `.processes.kill` and `.settings.set` do not declare `format::` and reject it.
**Setup:** None.
**Command:** `cm .processes.kill format::json`
**Expected Output:** exit code 1; unknown parameter.
**Pass Criteria:** Exit 1.
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md)

---

### EC-5: JSON structure per command

| Command | JSON top-level type | Key |
|---------|--------------------|----|
| `.status` | object `{}` | version, processes, account |
| `.version.show` | object `{}` | version |
| `.version.list` | array `[]` or object | aliases or list |
| `.processes` | object `{}` | processes (array) |
| `.settings.show` | object `{}` | all settings keys |
| `.settings.get` | object `{}` | key, value |
| `.version.history` | array `[]` | version, date, summary per element |
