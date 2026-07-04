# Test: Feature 014 — Rich Account Metadata

### Scope

- **Purpose**: Test cases for rich account metadata capture and display.
- **Source**: `docs/feature/014_rich_account_metadata.md`
- **Covers**: AC-01 through AC-12

Feature behavioral requirement test cases for `docs/feature/014_rich_account_metadata.md` (FR-20). Each FT case maps to one or more acceptance criteria.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | `display_name::1` shows `Display:` line on `.credentials.status` | AC-01 |
| FT-02 | `role::1` shows `Role:` line on `.credentials.status` | AC-02 |
| FT-03 | `billing::1` shows `Billing:` line on `.credentials.status` | AC-03 |
| FT-04 | `model::1` shows `Model:` line on `.credentials.status` | AC-04 |
| FT-05 | All four params default to 0 — absent from default output | AC-05 |
| FT-06 | `format::json` includes all four new keys regardless of params | AC-06 |
| FT-07 | Absent `~/.claude.json` → oauthAccount fields show `N/A` | AC-07 |
| FT-08 | Absent `~/.claude/settings.json` → `model` shows `N/A` | AC-08 |
| FT-09 | `display_name::1` on `.accounts` shows `Display:` from snapshot | AC-09 |
| FT-10 | `role::1 billing::1 model::1` on `.accounts` shows fields per account | AC-10 |
| FT-11 | No snapshot on disk → `N/A` for all four fields | AC-11 |
| FT-12 | `.accounts format::json` includes all four keys per account object | AC-12 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | `display_name::1` on `.credentials.status` → `Display:` line | AC-01 | Opt-In Fields |
| FT-02 | `role::1` on `.credentials.status` → `Role:` line | AC-02 | Opt-In Fields |
| FT-03 | `billing::1` on `.credentials.status` → `Billing:` line | AC-03 | Opt-In Fields |
| FT-04 | `model::1` on `.credentials.status` → `Model:` line | AC-04 | Opt-In Fields |
| FT-05 | New params absent by default from `.credentials.status` output | AC-05 | Default Behavior |
| FT-06 | `format::json` includes extended keys on `.credentials.status` | AC-06 | JSON Format |
| FT-07 | Absent `~/.claude.json` → all three oauthAccount fields `N/A` | AC-07 | N/A Handling |
| FT-08 | Absent `settings.json` → `model` `N/A` | AC-08 | N/A Handling |
| FT-09 | `.accounts display_name::1` → `Display:` per account | AC-09 | Accounts Integration |
| FT-10 | `.accounts role::1 billing::1 model::1` → per-account lines | AC-10 | Accounts Integration |
| FT-11 | No `{name}.json` snapshot → `N/A` for all new fields | AC-11 | N/A Handling |
| FT-12 | `.accounts format::json` includes 4 new keys per object | AC-12 | JSON Format |

**Total:** 12 FT cases

---

### FT-01: `display_name::1` on `.credentials.status` → `Display:` line

- **Given:** `~/.claude.json` contains `oauthAccount.displayName`.
- **When:** `clp .credentials.status display_name::1`
- **Then:** Output includes `Display: {displayName}` line.
- **Exit:** 0
- **Source fn:** `cred08_display_name_opt_in`
- **Source:** [014_rich_account_metadata.md AC-01](../../../docs/feature/014_rich_account_metadata.md)

---

### FT-02: `role::1` on `.credentials.status` → `Role:` line

- **Given:** `~/.claude.json` contains `oauthAccount.organizationRole`.
- **When:** `clp .credentials.status role::1`
- **Then:** Output includes `Role: {organizationRole}` line.
- **Exit:** 0
- **Source fn:** `cred09_role_opt_in`
- **Source:** [014_rich_account_metadata.md AC-02](../../../docs/feature/014_rich_account_metadata.md)

---

### FT-03: `billing::1` on `.credentials.status` → `Billing:` line

- **Given:** `~/.claude.json` contains `oauthAccount.billingType`.
- **When:** `clp .credentials.status billing::1`
- **Then:** Output includes `Billing: {billingType}` line.
- **Exit:** 0
- **Source fn:** `cred10_billing_opt_in`
- **Source:** [014_rich_account_metadata.md AC-03](../../../docs/feature/014_rich_account_metadata.md)

---

### FT-04: `model::1` on `.credentials.status` → `Model:` line

- **Given:** `~/.claude/settings.json` contains a `model` field.
- **When:** `clp .credentials.status model::1`
- **Then:** Output includes `Model: {model}` line.
- **Exit:** 0
- **Source fn:** `cred11_model_opt_in`
- **Source:** [014_rich_account_metadata.md AC-04](../../../docs/feature/014_rich_account_metadata.md)

---

### FT-05: New params absent by default from `.credentials.status` output

- **Given:** `~/.claude.json` and `~/.claude/settings.json` present with all fields.
- **When:** `clp .credentials.status` (no extra params)
- **Then:** None of `Display:`, `Role:`, `Billing:`, `Model:` lines appear in output.
- **Exit:** 0
- **Source fn:** `cred13_new_params_absent_by_default`
- **Source:** [014_rich_account_metadata.md AC-05](../../../docs/feature/014_rich_account_metadata.md)

---

### FT-06: `format::json` includes extended keys on `.credentials.status`

- **Given:** Valid credentials, `~/.claude.json`, and `~/.claude/settings.json` present.
- **When:** `clp .credentials.status format::json`
- **Then:** JSON includes `"display_name"`, `"role"`, `"billing"`, `"model"` keys regardless of whether those params were explicitly passed.
- **Exit:** 0
- **Source fn:** `cred12_json_extended_shape`
- **Source:** [014_rich_account_metadata.md AC-06](../../../docs/feature/014_rich_account_metadata.md)

---

### FT-07: Absent `~/.claude.json` → all three oauthAccount fields `N/A`

- **Given:** `~/.claude.json` does not exist. Valid credentials present.
- **When:** `clp .credentials.status display_name::1 role::1 billing::1`
- **Then:** `Display:`, `Role:`, `Billing:` all show `N/A`. No error.
- **Exit:** 0
- **Source fn:** `cred05_no_claude_json_shows_na`
- **Source:** [014_rich_account_metadata.md AC-07](../../../docs/feature/014_rich_account_metadata.md)

---

### FT-08: Absent `settings.json` → `model` `N/A`

- **Given:** `~/.claude/settings.json` does not exist. Valid credentials present.
- **When:** `clp .credentials.status model::1`
- **Then:** `Model:` shows `N/A`. No error.
- **Exit:** 0
- **Source fn:** `cred47_absent_settings_json_model_shows_na`
- **Source:** [014_rich_account_metadata.md AC-08](../../../docs/feature/014_rich_account_metadata.md)

---

### FT-09: `.accounts display_name::1` → `Display:` per account

- **Given:** An account saved with a `{name}.json` snapshot containing `displayName`.
- **When:** `clp .accounts display_name::1`
- **Then:** Each account block includes a `Display:` line populated from the snapshot.
- **Exit:** 0
- **Source fn:** `acc20_display_name_shows_from_snapshot`
- **Source:** [014_rich_account_metadata.md AC-09](../../../docs/feature/014_rich_account_metadata.md)

---

### FT-10: `.accounts role::1 billing::1 model::1` → per-account lines

- **Given:** Accounts saved with `{name}.json` snapshots containing `oauthAccount` and `model` fields.
- **When:** `clp .accounts role::1 billing::1 model::1`
- **Then:** Each account block shows `Role:`, `Billing:`, and `Model:` lines from their respective snapshots. `Model:` is `N/A` if `model` field is absent in `{name}.json`.
- **Exit:** 0
- **Source fn:** `acc21_role_billing_model_from_snapshots`
- **Source:** [014_rich_account_metadata.md AC-10](../../../docs/feature/014_rich_account_metadata.md)

---

### FT-11: No `{name}.json` snapshot → `N/A` for all new fields

- **Given:** An account with no `{name}.json` snapshot on disk.
- **When:** `clp .accounts display_name::1 role::1 billing::1 model::1`
- **Then:** All four lines show `N/A` for the affected account. No error.
- **Exit:** 0
- **Source fn:** `acc22_no_snapshot_shows_na_for_new_fields`
- **Source:** [014_rich_account_metadata.md AC-11](../../../docs/feature/014_rich_account_metadata.md)

---

### FT-12: `.accounts format::json` includes 4 new keys per object

- **Given:** One or more accounts saved, with or without snapshots.
- **When:** `clp .accounts format::json`
- **Then:** Each account object in the JSON array includes `"display_name"`, `"role"`, `"billing"`, `"model"` keys. Values are `N/A` strings when data is absent.
- **Exit:** 0
- **Source fn:** `acc23_json_includes_new_fields`
- **Source:** [014_rich_account_metadata.md AC-12](../../../docs/feature/014_rich_account_metadata.md)
