# Test: Output Control Group

Integration and edge case coverage for the Output Control parameter group (`format::`). See [parameter_groups.md](../../../../docs/cli/param_group/readme.md#group--1-output-control) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | `format::json` returns valid JSON on all supported commands | JSON Mode |
| CC-2 | `format::text` (default) produces labeled text output | Text Mode |
| CC-3 | `format::json` overrides field-presence params on `.accounts` | Interaction |
| CC-4 | `format::json` overrides field-presence params on `.credentials.status` | Interaction |
| CC-5 | `format::` param ignored by mutation commands (save, use, delete) | Non-Applicability |
| CC-6 | `format::table` on `.accounts` renders table; rejected on other commands | Table Mode |

### Test Coverage Summary

- JSON Mode: 1 test
- Text Mode: 1 test
- Interaction: 2 tests
- Non-Applicability: 1 test
- Table Mode: 1 test

**Total:** 6 tests

---

### CC-1: JSON Mode

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:**
  1. `clp .token.status format::json`
  2. `clp .paths format::json`
  3. `clp .usage format::json`
  4. `clp .account.limits format::json`
  5. `clp .accounts format::json`
  6. `clp .credentials.status format::json`
  7. `clp .account.inspect format::json`
- **Then:** Each produces a valid JSON object or array with all fields present. All exit 0.
- **Exit:** 0
- **Source:** [parameter_groups.md — Output Control](../../../../docs/cli/param_group/readme.md#group--1-output-control)

---

### CC-2: Text Mode

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:**
  1. `clp .token.status`
  2. `clp .paths`
  3. `clp .usage`
- **Then:** Each produces labeled text output. All exit 0.
- **Exit:** 0
- **Source:** [parameter_groups.md — Output Control](../../../../docs/cli/param_group/readme.md#group--1-output-control)

---

### CC-3: Interaction — JSON overrides field-presence params on `.accounts`

- **Given:** At least one saved account with `sub::0` suppressing a text field.
- **When:** `clp .accounts sub::0 format::json`
- **Then:** Valid JSON array with `subscription_type` present despite `sub::0`. Exit 0.
- **Exit:** 0
- **Source:** [004_parameter_interactions.md — Interaction 2](../../../../docs/cli/004_parameter_interactions.md#interaction--2-formatjson-overrides-field-presence-params)

---

### CC-4: Interaction — JSON overrides field-presence params on `.credentials.status`

- **Given:** Active credentials exist. `file::0` is the default (file path suppressed in text).
- **When:** `clp .credentials.status format::json`
- **Then:** Valid JSON object with `"file"` key present despite `file::0` default. Exit 0.
- **Exit:** 0
- **Source:** [004_parameter_interactions.md — Interaction 2](../../../../docs/cli/004_parameter_interactions.md#interaction--2-formatjson-overrides-field-presence-params)

---

### CC-5: Non-Applicability

- **Given:** Active credentials exist. An account named `test_na@x.com` exists.
- **When:**
  1. `clp .account.save name::test_na@x.com format::json`
  2. `clp .account.use name::test_na@x.com format::json`
  3. `clp .account.delete name::test_na@x.com format::json`
- **Then:** Each mutation command either ignores `format::json` (producing its standard single-line confirmation) or rejects it with an error. The param does not alter mutation output format.
- **Exit:** 0
- **Source:** [parameter_groups.md — Output Control](../../../../docs/cli/param_group/readme.md#group--1-output-control)

---

### CC-6: Table Mode

- **Given:** At least one saved account exists. Active credentials present.
- **When:**
  1. `clp .accounts format::table`
  2. `clp .token.status format::table`
- **Then:** `.accounts format::table` exits 0 and produces a titled, aligned table with columns Account, Sub, Tier, Expires, Email (with flag column). `.token.status format::table` exits 1 with an error indicating table is not supported.
- **Exit:** 0 (`.accounts`), 1 (`.token.status`)
- **Source:** [commands.md — .accounts](../../../../docs/cli/command/001_account.md#command--3-accounts), [004_parameter_interactions.md — Interaction 3](../../../../docs/cli/004_parameter_interactions.md#interaction--3-formattable-ignores-field-presence-params)
