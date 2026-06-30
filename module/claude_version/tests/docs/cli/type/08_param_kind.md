# Test: `ParamKind`

Type compliance and validation tests for `ParamKind`. See [type/08_param_kind.md](../../../../docs/cli/type/08_param_kind.md) for specification.

### Scope

- **Purpose**: Validate ParamKind parsing, case-sensitivity enforcement, and filtering behavior.
- **Responsibility**: Valid variants, invalid inputs, default behavior, and observable output differences between kind values.
- **Commands:** `.params`
- **In Scope**: Kind string parsing, case-sensitive matching, and observable output filtering differences.
- **Out of Scope**: Per-command JSON schema structure (→ `../command/`), parameter interactions (→ `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `kind::config` → shows config-key params only | Valid: config |
| TC-2 | `kind::env` → shows env-var params only | Valid: env |
| TC-3 | Absent `kind::` → defaults to all params | Default |
| TC-4 | `kind::Config` → exit 1 (case-sensitive) | Validation: case |
| TC-5 | `kind::all` → exit 1 (unknown variant) | Validation: unknown |
| TC-6 | `kind::` (empty) → exit 1 | Validation: empty |

## Test Coverage Summary

- Valid filter: 2 tests (TC-1, TC-2)
- Default Behavior: 1 test (TC-3)
- Case sensitivity: 1 test (TC-4)
- Unknown variant: 1 test (TC-5)
- Empty value: 1 test (TC-6)

**Total:** 6 tests

**Behavioral Divergence Pair:** TC-1 (`kind::config` → config params only, exit 0) ↔ TC-2 (`kind::env` → env params only, exit 0)

---

### TC-1: `kind::config` → config-key params only

- **Given:** `HOME=<tmp>` (no settings.json)
- **When:** `clv.params kind::config`
- **Then:** exit 0; output contains only params with a `config` form; env-only params (those with `env_var` but no `config_key`) are absent
- **Exit:** 0
- **Source:** [type/08_param_kind.md — config: settings.json config key form](../../../../docs/cli/type/08_param_kind.md)

---

### TC-2: `kind::env` → env-var params only

- **Given:** `HOME=<tmp>` (no settings.json)
- **When:** `clv.params kind::env`
- **Then:** exit 0; output contains only params with an `env` form; config-only params (those with `config_key` but no `env_var`) are absent
- **Exit:** 0
- **Source:** [type/08_param_kind.md — env: env var form](../../../../docs/cli/type/08_param_kind.md)

---

### TC-3: Absent `kind::` → all params

- **Given:** `HOME=<tmp>` (no settings.json)
- **When:** `clv.params` (no `kind::`)
- **Then:** exit 0; output contains all catalog params including both config-only and env-only entries
- **Exit:** 0
- **Source:** [type/08_param_kind.md — Default: absent (no filter)](../../../../docs/cli/type/08_param_kind.md)

---

### TC-4: `kind::Config` → exit 1

- **Given:** `HOME=<tmp>`
- **When:** `clv.params kind::Config`
- **Then:** exit 1; stderr contains error message referencing unknown kind or case-sensitivity
- **Exit:** 1
- **Source:** [type/08_param_kind.md — Parsing: exact string match](../../../../docs/cli/type/08_param_kind.md)

---

### TC-5: `kind::all` → exit 1

- **Given:** `HOME=<tmp>`
- **When:** `clv.params kind::all`
- **Then:** exit 1; stderr contains "unknown kind" or similar message mentioning expected values `config` or `env`
- **Exit:** 1
- **Source:** [type/08_param_kind.md — Validation errors: expected config or env](../../../../docs/cli/type/08_param_kind.md)

---

### TC-6: `kind::` (empty) → exit 1

- **Given:** `HOME=<tmp>`
- **When:** `clv.params kind::`
- **Then:** exit 1; error message references `kind::` or empty value
- **Exit:** 1
- **Source:** [type/08_param_kind.md — Validation errors](../../../../docs/cli/type/08_param_kind.md)

---

### Source Functions

| Function | File |
|----------|------|
| *(none yet — implementation pending)* | — |
