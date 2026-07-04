# Test: Field Presence Group

Integration and edge case coverage for the Field Presence parameter group. See [parameter_groups.md](../../../../docs/cli/param_group/002_field_presence.md) for specification.

> **Feature 037/065 note:** `.accounts` field-presence toggles (`active::`, `sub::`, `tier::`, `expires::`, `email::`, etc.) were removed in Feature 037 and replaced by `cols::`. CCs that invoke these on `.accounts` (CC-4, CC-8, CC-11) describe pre-Feature-037 behavior — the tests now exit 1 instead of 0. Additionally, `active::` was further repurposed (Feature 064) and then REMOVED_TOGGLE'd (Feature 065); on `.accounts` it now exits 1 with a migration message pointing to `assignee::`. CC-8 has been updated to reflect this.

Both `.accounts` and `.credentials.status` are Full Implementors for their own field sets. Eight parameters are shared between both commands: four default-on (`sub::`, `tier::`, `expires::`, `email::`) and four opt-in (`display_name::`, `role::`, `billing::`, `model::`).

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | Default: all on-by-default fields appear in `.accounts` output | Default On |
| CC-2 | Default: all on-by-default fields appear in `.credentials.status` output | Default On |
| CC-3 | Single field disabled — only that line suppressed | Single Suppression |
| CC-4 | All on-by-default fields disabled — only opt-in fields absent | Full Suppression |
| CC-5 | All 6 opt-in fields (`file`, `saved`, `display_name`, `role`, `billing`, `model`) appear when enabled | Opt-In Fields |
| CC-6 | Shared default-on params (`sub::`, `tier::`, `expires::`, `email::`) behave identically on both commands | Cross-Command Consistency |
| CC-7 | `format::json` overrides field-presence params — all keys in JSON | Interaction |
| CC-8 | `active::0` suppresses `Active:` in `.accounts` but has no effect on `.credentials.status` | Command Specificity |
| CC-9 | `account::0` suppresses `Account:` in `.credentials.status` but has no effect on `.accounts` | Command Specificity |
| CC-10 | `token::` not accepted by `.accounts` | Non-Applicability |
| CC-11 | Field-presence params do not affect exit codes | Exit Code Preservation |
| CC-12 | Opt-in fields (`display_name::`, `role::`, `billing::`, `model::`) work on `.accounts` with saved snapshots | Opt-In on .accounts |
| CC-13 | Shared opt-in params behave consistently across both commands | Cross-Command Opt-In Consistency |

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
- Opt-In on .accounts: 1 test
- Cross-Command Opt-In Consistency: 1 test

**Total:** 13 tests (EC-1–EC-6 group integration, EC-7–EC-13 group edge cases)

---

### CC-1: Default fields in `.accounts`

- **Given:** At least one account exists in the credential store.
- **When:** `clp .accounts`
- **Then:** Each account block contains `Active:`, `Sub:`, `Tier:`, `Expires:`, `Email:` lines.; all five on-by-default fields present without any params
- **Exit:** 0
- **Source:** [parameter_groups.md — Field Presence](../../../../docs/cli/param_group/002_field_presence.md)

---

### CC-2: Default fields in `.credentials.status`

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:** `clp .credentials.status`
- **Then:** Output contains `Account:`, `Sub:`, `Tier:`, `Token:`, `Expires:`, `Email:` lines.; all six on-by-default fields present without any params
- **Exit:** 0
- **Source:** [parameter_groups.md — Field Presence](../../../../docs/cli/param_group/002_field_presence.md)

---

### CC-3: Single field suppression

- **Given:** Active account exists.
- **When:** `clp .accounts sub::0`
- **Then:** Account block contains `Active:`, `Tier:`, `Expires:`, `Email:` but NOT `Sub:`.; only `Sub:` line absent; all other on-by-default fields remain
- **Exit:** 0
- **Source:** [parameter_groups.md — Field Presence](../../../../docs/cli/param_group/002_field_presence.md)

---

### CC-4: All on-by-default fields suppressed

- **Given:** Two accounts exist.
- **When:** `clp .accounts active::0 sub::0 tier::0 expires::0 email::0`
- **Then:** Only account name lines (unindented), no field lines, no blank separators.; bare name list when all on-by-default fields disabled
- **Exit:** 0
- **Source:** [parameter_groups.md — Field Presence](../../../../docs/cli/param_group/002_field_presence.md)

---

### CC-5: Opt-in fields appear when enabled

- **Given:** Active credentials exist with `~/.claude.json` containing `displayName`, `organizationRole`, `billingType` and `~/.claude/settings.json` containing `model`. At least one account saved in credential store.
- **When:** `clp .credentials.status file::1 saved::1 display_name::1 role::1 billing::1 model::1`
- **Then:** All six default-on fields plus `File:`, `Saved:`, `Display:`, `Role:`, `Billing:`, and `Model:` lines.; all 6 opt-in fields appear when explicitly enabled
- **Exit:** 0
- **Source:** [parameter_groups.md — Field Presence](../../../../docs/cli/param_group/002_field_presence.md)

---

### CC-6: Shared default-on params consistent across both commands

- **Given:** Active account and credentials exist.
- **When:**
  1. `clp .accounts sub::0 tier::0 expires::0 email::0`
  2. `clp .credentials.status sub::0 tier::0 expires::0 email::0`
- **Then:** 1. `.accounts` blocks contain `Active:` but NOT `Sub:`, `Tier:`, `Expires:`, `Email:`
2. `.credentials.status` contains `Account:`, `Token:` but NOT `Sub:`, `Tier:`, `Expires:`, `Email:`; for both; shared params suppress the same fields on both commands
- **Exit:** 0
- **Source:** [parameter_groups.md — Field Presence](../../../../docs/cli/param_group/002_field_presence.md)

---

### CC-7: `format::json` overrides field-presence params

- **Given:** Active account and credentials exist.
- **When:** `clp .accounts sub::0 tier::0 format::json`
- **Then:** Valid JSON array where each object still contains `subscription_type` and `rate_limit_tier`.; field-presence params do not strip JSON keys
- **Exit:** 0
- **Source:** [004_parameter_interactions.md — Interaction 3](../../../../docs/cli/004_parameter_interactions.md#interaction-2-formatjson-overrides-field-presence-params)

---

### CC-8: `active::` is `.accounts`-specific — no effect on `.credentials.status`

> **Feature 065 note:** `active::` is now a REMOVED_TOGGLE on `.accounts` (Feature 065). `clp .accounts active::0` exits 1 with a migration message pointing to `assignee::` — it no longer suppresses the Active field. The command-specificity principle (that `.credentials.status` rejects `.accounts`-only params) still holds, but `active::` exits 1 on both commands for different reasons.

- **Given:** Active account and credentials exist.
- **When:**
  1. `clp .accounts active::0` — exits 1 (REMOVED_TOGGLE, Feature 065; formerly suppressed `Active:` line)
  2. `clp .credentials.status active::0` — exits 1 (unknown param)
- **Then:** Both exit 1. `active::` is a REMOVED_TOGGLE on `.accounts`; `.credentials.status` rejects it as unknown.
- **Exit:** 1 (both cases)
- **Source:** [parameter_groups.md — Field Presence](../../../../docs/cli/param_group/002_field_presence.md)

---

### CC-9: `account::` is `.credentials.status`-specific — no effect on `.accounts`

- **Given:** Active account and credentials exist.
- **When:**
  1. `clp .credentials.status account::0` — should suppress `Account:` line
  2. `clp .accounts account::0` — should fail (unknown param)
- **Then:** `account::` applies to `.credentials.status` only; `.accounts` rejects it
- **Exit:** 0
- **Source:** [parameter_groups.md — Field Presence](../../../../docs/cli/param_group/002_field_presence.md)

---

### CC-10: `.credentials.status`-only params rejected by `.accounts`

- **Given:** Active account exists.
- **When:** `clp .accounts token::0`
- **Then:** Exit 1 with an unrecognised-parameter error.; `token::` is `.credentials.status`-only and rejected by `.accounts`
- **Exit:** 1
- **Source:** [parameter_groups.md — Field Presence](../../../../docs/cli/param_group/002_field_presence.md)

---

### CC-11: Field-presence params do not affect exit codes

- **Given:** Remove or hide the credential store so `.accounts` has no data.
- **When:** `clp .accounts active::0 sub::0 tier::0 expires::0 email::0`
- **Then:** Exit 0 with `(no accounts configured)` — the empty-store case is not an error.; field-presence params do not affect exit code semantics
- **Exit:** 0
- **Source:** [parameter_groups.md — Field Presence](../../../../docs/cli/param_group/002_field_presence.md)

---

### CC-12: Opt-in fields on `.accounts` with saved snapshots

- **Given:** `work@acme.com` in credential store with saved `{name}.json` containing displayName, organizationRole, billingType, and model.
- **When:** `clp .accounts display_name::1 role::1 billing::1 model::1`
- **Then:** Account block contains `Display:`, `Role:`, `Billing:`, `Model:` lines with values from saved snapshots.; all 4 opt-in metadata fields render from per-account snapshots
- **Exit:** 0
- **Source:** [parameter_groups.md — Field Presence](../../../../docs/cli/param_group/002_field_presence.md)

---

### CC-13: Shared opt-in params consistent across both commands

- **Given:** Active credentials with live `~/.claude.json` and saved per-account `{name}.json` snapshot for same account.
- **When:**
  1. `clp .credentials.status display_name::1 role::1`
  2. `clp .accounts display_name::1 role::1`
- **Then:** Both commands show `Display:` and `Role:` lines with values from their respective data sources (live vs saved).; opt-in field params suppress/show identically on both commands
- **Exit:** 0
- **Source:** [parameter_groups.md — Field Presence](../../../../docs/cli/param_group/002_field_presence.md)
