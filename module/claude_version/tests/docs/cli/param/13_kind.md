# Test: `kind::`

Edge case coverage for the `kind::` parameter. See [param/13_kind.md](../../../../docs/cli/param/13_kind.md) for specification.

### Scope

- **Purpose**: Edge case tests for the `kind::` parameter.
- **Responsibility**: Boundary values, invalid inputs, type violations, mode interaction, and default behavior for `kind::`.
- **Commands:** `.params`
- **In Scope**: Single-parameter edge cases, validation errors, case-sensitivity, mode-supersede behavior.
- **Out of Scope**: Command integration (→ `../command/`), group interactions (→ `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `kind::config` → only config-key params shown | Valid: config |
| EC-2 | `kind::env` → only env-var params shown | Valid: env |
| EC-3 | Absent `kind::` → all params shown | Default Behavior |
| EC-4 | `kind::invalid` → exit 1, message about valid values | Invalid: unknown |
| EC-5 | `kind::` (empty) → exit 1 | Invalid: empty |
| EC-6 | `kind::CONFIG` (uppercase) → exit 1 | Invalid: case |
| EC-7 | `kind::` ignored when `key::` present | Mode Supersede |

## Test Coverage Summary

- Valid filter: 2 tests (EC-1, EC-2)
- Default Behavior: 1 test (EC-3)
- Invalid unknown: 1 test (EC-4)
- Invalid empty: 1 test (EC-5)
- Invalid case: 1 test (EC-6)
- Mode Supersede: 1 test (EC-7)

**Total:** 7 edge cases

**Behavioral Divergence Pair:** EC-1 (`kind::config` → config params only, exit 0) ↔ EC-4 (`kind::invalid` → exit 1 with valid values message)

---

### EC-1: `kind::config` → only config-key params

- **Given:** `HOME=<tmp>` (no settings.json)
- **When:** `clv.params kind::config`
- **Then:** exit 0; stdout contains params with a config key form (e.g., `model`, `theme`); env-only params (e.g., `bash_timeout`) absent
- **Exit:** 0
- **Source:** [param/13_kind.md](../../../../docs/cli/param/13_kind.md)

---

### EC-2: `kind::env` → only env-var params

- **Given:** `HOME=<tmp>` (no settings.json)
- **When:** `clv.params kind::env`
- **Then:** exit 0; stdout contains params with an env var form (e.g., `model`, `bash_timeout`); config-only params (e.g., `theme`) absent
- **Exit:** 0
- **Source:** [param/13_kind.md](../../../../docs/cli/param/13_kind.md)

---

### EC-3: Absent `kind::` → all params

- **Given:** `HOME=<tmp>` (no settings.json)
- **When:** `clv.params` (no `kind::` parameter)
- **Then:** exit 0; output contains both config-key params and env-only params; total ≥35 entries
- **Exit:** 0
- **Source:** [param/13_kind.md](../../../../docs/cli/param/13_kind.md)

---

### EC-4: `kind::invalid` → exit 1

- **Given:** `HOME=<tmp>`
- **When:** `clv.params kind::invalid`
- **Then:** exit 1; stderr contains a message indicating valid values are `config` or `env`
- **Exit:** 1
- **Source:** [param/13_kind.md](../../../../docs/cli/param/13_kind.md)

---

### EC-5: `kind::` (empty) → exit 1

- **Given:** `HOME=<tmp>`
- **When:** `clv.params kind::`
- **Then:** exit 1; error message references `kind::` or empty value
- **Exit:** 1
- **Source:** [param/13_kind.md](../../../../docs/cli/param/13_kind.md)

---

### EC-6: `kind::CONFIG` (uppercase) → exit 1

- **Given:** `HOME=<tmp>`
- **When:** `clv.params kind::CONFIG`
- **Then:** exit 1; `kind::` is case-sensitive; `CONFIG` is not a valid variant
- **Exit:** 1
- **Source:** [type/08_param_kind.md](../../../../docs/cli/type/08_param_kind.md)

---

### EC-7: `kind::` ignored when `key::` present

- **Given:** `HOME=<tmp>`
- **When:** `clv.params key::model kind::env`
- **Then:** exit 0; output shows the single-param deep-dive for `model` (all its forms, values, default); no kind-filter applied
- **Exit:** 0
- **Source:** [004_parameter_interactions.md](../../../../docs/cli/004_parameter_interactions.md)

---

### Source Functions

| Function | File |
|----------|------|
| *(none yet — implementation pending)* | — |
