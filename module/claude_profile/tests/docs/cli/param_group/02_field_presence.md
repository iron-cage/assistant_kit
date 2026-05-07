# Test: Field Presence Group

Integration and edge case coverage for the Field Presence parameter group. See [parameter_groups.md](../../../../docs/cli/parameter_groups.md#group--2-field-presence) for specification.

Both `.accounts` and `.credentials.status` are Full Implementors for their own field sets. Four parameters (`sub::`, `tier::`, `expires::`, `org::`) are shared between both commands.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Default: all on-by-default fields appear in `.accounts` output | Default On |
| EC-2 | Default: all on-by-default fields appear in `.credentials.status` output | Default On |
| EC-3 | Single field disabled — only that line suppressed | Single Suppression |
| EC-4 | All on-by-default fields disabled — only opt-in fields absent | Full Suppression |
| EC-5 | All 6 opt-in fields (`file`, `saved`, `display_name`, `role`, `billing`, `model`) appear when enabled | Opt-In Fields |
| EC-6 | Shared params (`sub::`, `tier::`, `expires::`, `org::`) behave identically on both commands | Cross-Command Consistency |
| EC-1 | `format::json` overrides field-presence params — all keys in JSON | Interaction |
| EC-2 | `active::0` suppresses `Active:` in `.accounts` but has no effect on `.credentials.status` | Command Specificity |
| EC-3 | `account::0` suppresses `Account:` in `.credentials.status` but has no effect on `.accounts` | Command Specificity |
| EC-4 | `token::` and `email::` not accepted by `.accounts` | Non-Applicability |
| EC-5 | Field-presence params do not affect exit codes | Exit Code Preservation |

### Test Coverage Summary

- Default On: 2 tests
- Single Suppression: 1 test
- Full Suppression: 1 test
- Opt-In Fields: 1 test
- Cross-Command Consistency: 1 test
- Interaction: 1 test
- Command Specificity: 2 tests
- Non-Applicability: 1 test
- Exit Code Preservation: 1 test

**Total:** 11 tests (6 integration, 5 edge cases)

---

### EC-1: Default fields in `.accounts`

- **Given:** At least one account exists in the credential store.
- **When:** `clp .accounts`
- **Then:** Each account block contains `Active:`, `Sub:`, `Tier:`, `Expires:`, `Org:` lines.; all five on-by-default fields present without any params
- **Exit:** 0
- **Source:** [parameter_groups.md — Field Presence](../../../../docs/cli/parameter_groups.md#group--2-field-presence)

---

### EC-2: Default fields in `.credentials.status`

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:** `clp .credentials.status`
- **Then:** Output contains `Account:`, `Sub:`, `Tier:`, `Token:`, `Expires:`, `Email:`, `Org:` lines.; all seven on-by-default fields present without any params
- **Exit:** 0
- **Source:** [parameter_groups.md — Field Presence](../../../../docs/cli/parameter_groups.md#group--2-field-presence)

---

### EC-3: Single field suppression

- **Given:** Active account exists.
- **When:** `clp .accounts sub::0`
- **Then:** Account block contains `Active:`, `Tier:`, `Expires:`, `Org:` but NOT `Sub:`.; only `Sub:` line absent; all other on-by-default fields remain
- **Exit:** 0
- **Source:** [parameter_groups.md — Field Presence](../../../../docs/cli/parameter_groups.md#group--2-field-presence)

---

### EC-4: All on-by-default fields suppressed

- **Given:** Two accounts exist.
- **When:** `clp .accounts active::0 sub::0 tier::0 expires::0 org::0`
- **Then:** Only account name lines (unindented), no field lines, no blank separators.; bare name list when all on-by-default fields disabled
- **Exit:** 0
- **Source:** [parameter_groups.md — Field Presence](../../../../docs/cli/parameter_groups.md#group--2-field-presence)

---

### EC-5: Opt-in fields appear when enabled

- **Given:** Active credentials exist with `~/.claude.json` containing `displayName`, `organizationRole`, `billingType` and `~/.claude/settings.json` containing `model`. At least one account saved in credential store.
- **When:** `clp .credentials.status file::1 saved::1 display_name::1 role::1 billing::1 model::1`
- **Then:** All seven default-on fields plus `File:`, `Saved:`, `Display:`, `Role:`, `Billing:`, and `Model:` lines.; all 6 opt-in fields appear when explicitly enabled
- **Exit:** 0
- **Source:** [parameter_groups.md — Field Presence](../../../../docs/cli/parameter_groups.md#group--2-field-presence)

---

### EC-6: Shared params consistent across both commands

- **Given:** Active account and credentials exist.
- **When:**
  1. `clp .accounts sub::0 tier::0 expires::0 org::0`
  2. `clp .credentials.status sub::0 tier::0 expires::0 org::0`
- **Then:** 1. `.accounts` blocks contain `Active:` but NOT `Sub:`, `Tier:`, `Expires:`, `Org:`
2. `.credentials.status` contains `Account:`, `Token:`, `Email:` but NOT `Sub:`, `Tier:`, `Expires:`, `Org:`; for both; shared params suppress the same fields on both commands
- **Exit:** 0
- **Source:** [parameter_groups.md — Field Presence](../../../../docs/cli/parameter_groups.md#group--2-field-presence)

---

### EC-1: `format::json` overrides field-presence params

- **Given:** Active account and credentials exist.
- **When:** `clp .accounts sub::0 tier::0 format::json`
- **Then:** Valid JSON array where each object still contains `subscription_type` and `rate_limit_tier`.; field-presence params do not strip JSON keys
- **Exit:** 0
- **Source:** [parameter_interactions.md — Interaction 3](../../../../docs/cli/parameter_interactions.md#interaction--3-formatjson-overrides-field-presence-params)

---

### EC-2: `active::` is `.accounts`-specific — no effect on `.credentials.status`

- **Given:** Active account and credentials exist.
- **When:**
  1. `clp .accounts active::0` — should suppress `Active:` line
  2. `clp .credentials.status active::0` — should fail (unknown param) or be ignored
- **Then:** `active::` applies to `.accounts` only; `.credentials.status` rejects it
- **Exit:** 0
- **Source:** [parameter_groups.md — Field Presence](../../../../docs/cli/parameter_groups.md#group--2-field-presence)

---

### EC-3: `account::` is `.credentials.status`-specific — no effect on `.accounts`

- **Given:** Active account and credentials exist.
- **When:**
  1. `clp .credentials.status account::0` — should suppress `Account:` line
  2. `clp .accounts account::0` — should fail (unknown param)
- **Then:** `account::` applies to `.credentials.status` only; `.accounts` rejects it
- **Exit:** 0
- **Source:** [parameter_groups.md — Field Presence](../../../../docs/cli/parameter_groups.md#group--2-field-presence)

---

### EC-4: `.credentials.status`-only params rejected by `.accounts`

- **Given:** Active account exists.
- **When:**
  1. `clp .accounts token::0`
  2. `clp .accounts email::0`
- **Then:** Exit 1 for both with an unrecognised-parameter error.; `.credentials.status`-only params rejected by `.accounts`
- **Exit:** 1
- **Source:** [parameter_groups.md — Field Presence](../../../../docs/cli/parameter_groups.md#group--2-field-presence)

---

### EC-5: Field-presence params do not affect exit codes

- **Given:** Remove or hide the credential store so `.accounts` has no data.
- **When:** `clp .accounts active::0 sub::0 tier::0 expires::0 org::0`
- **Then:** Exit 0 with `(no accounts configured)` — the empty-store case is not an error.; field-presence params do not affect exit code semantics
- **Exit:** 0
- **Source:** [parameter_groups.md — Field Presence](../../../../docs/cli/parameter_groups.md#group--2-field-presence)
