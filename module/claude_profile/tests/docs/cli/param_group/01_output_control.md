# Test: Output Control Group

Integration and edge case coverage for the Output Control parameter group (`format::`). See [parameter_groups.md](../../../../docs/cli/parameter_groups.md#group--1-output-control) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `format::json` returns valid JSON on all supported commands | JSON Mode |
| EC-2 | `format::text` (default) produces labeled text output | Text Mode |
| EC-3 | `format::json` overrides field-presence params on `.accounts` | Interaction |
| EC-4 | `format::json` overrides field-presence params on `.credentials.status` | Interaction |
| EC-5 | `format::` param ignored by mutation commands (save, use, delete) | Non-Applicability |

### Test Coverage Summary

- JSON Mode: 1 test
- Text Mode: 1 test
- Interaction: 2 tests
- Non-Applicability: 1 test

**Total:** 5 tests

---

### EC-1: JSON Mode

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:**
  1. `clp .token.status format::json`
  2. `clp .paths format::json`
  3. `clp .usage format::json`
  4. `clp .account.limits format::json`
- **Then:** Each produces a valid JSON object or array with all fields present. All exit 0.
- **Exit:** 0
- **Source:** [parameter_groups.md — Output Control](../../../../docs/cli/parameter_groups.md#group--1-output-control)

---

### EC-2: Text Mode

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:**
  1. `clp .token.status`
  2. `clp .paths`
  3. `clp .usage`
- **Then:** Each produces labeled text output. All exit 0.
- **Exit:** 0
- **Source:** [parameter_groups.md — Output Control](../../../../docs/cli/parameter_groups.md#group--1-output-control)

---

### EC-3: Interaction — JSON overrides field-presence params on `.accounts`

- **Given:** At least one saved account with `sub::0` suppressing a text field.
- **When:** `clp .accounts sub::0 format::json`
- **Then:** Valid JSON array with `subscription_type` present despite `sub::0`. Exit 0.
- **Exit:** 0
- **Source:** [parameter_interactions.md — Interaction 2](../../../../docs/cli/parameter_interactions.md#interaction--2-formatjson-overrides-field-presence-params)

---

### EC-4: Interaction — JSON overrides field-presence params on `.credentials.status`

- **Given:** Active credentials exist. `file::0` is the default (file path suppressed in text).
- **When:** `clp .credentials.status format::json`
- **Then:** Valid JSON object with `"file"` key present despite `file::0` default. Exit 0.
- **Exit:** 0
- **Source:** [parameter_interactions.md — Interaction 2](../../../../docs/cli/parameter_interactions.md#interaction--2-formatjson-overrides-field-presence-params)

---

### EC-5: Non-Applicability

- **Given:** Active credentials exist. An account named `test_na@x.com` exists.
- **When:**
  1. `clp .account.save name::test_na@x.com format::json`
  2. `clp .account.use name::test_na@x.com format::json`
  3. `clp .account.delete name::test_na@x.com format::json`
- **Then:** Each mutation command either ignores `format::json` (producing its standard single-line confirmation) or rejects it with an error. The param does not alter mutation output format.
- **Exit:** 0
- **Source:** [parameter_groups.md — Output Control](../../../../docs/cli/parameter_groups.md#group--1-output-control)
