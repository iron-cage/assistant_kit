# Test: Feature 003 — Accounts

Feature behavioral requirement test cases for `docs/feature/003_account_list.md` (FR-8). Each FT case maps to one or more acceptance criteria.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | Empty or absent credential store → advisory message, exit 0 | AC-01 |
| FT-02 | Each entry shows standard fields as indented key-val blocks | AC-02 |
| FT-03 | Active account marked `Active: yes`; all others `Active: no` | AC-03 |
| FT-04 | `format::json` returns a valid JSON array | AC-04 |
| FT-05 | `name::` scopes to one account; exit 2 not found; exit 1 invalid | AC-05 |
| FT-06 | `cols::` column suppression removes lines; JSON ignores `cols::` settings | AC-06 |
| FT-07 | All default-on columns removed via `cols::` → bare name lines, no blank separators | AC-07 |
| FT-08 | Accounts listed alphabetically by name | AC-08 |
| FT-09 | `cols::+display_name` shows `Display:` line from snapshot | AC-09 |
| FT-10 | `cols::+host,+role,+billing,+model` show lines from snapshots | AC-10 |
| FT-11 | Absent metadata files → `N/A` for affected fields | AC-11 |
| FT-12 | `format::json` includes extended fields for every account object | AC-12 |
| FT-13 | `Current: yes` for token-matched account; `Current: no` for others | AC-13 |
| FT-14 | `cols::-current` suppresses `Current:` line; absent creds also suppresses | AC-14 |
| FT-15 | `format::json` includes `is_current` boolean per account | AC-15 |
| FT-16 | `cols::+uuid` shows `ID:` line from snapshot; `N/A` when absent | AC-16 |
| FT-17 | `cols::+capabilities` shows `Capabilities:` line; `N/A` when absent | AC-17 |
| FT-18 | `cols::+org_uuid` shows `Org ID:` from `{name}.json`; `N/A` when absent | AC-18 |
| FT-19 | `cols::+org_name` shows `Org:` from `{name}.json`; `N/A` when absent | AC-19 |
| FT-20 | `format::json` includes `owner` and `is_owned` fields per account | AC-20 |
| FT-21 | `format::json` includes `renewal_at` field; absent when not set | AC-21 |
| FT-22 | `format::json` emits correct `host`, `role`, `organization_role` values | AC-12 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | Empty or absent store shows advisory, exits 0 | AC-01 | Error Handling |
| FT-02 | Entry format: indented key-val blocks, blank separators, expiresAt | AC-02 | Output Format |
| FT-03 | Active account shows `Active: yes`; inactive shows `Active: no` | AC-03 | Active Marker |
| FT-04 | `format::json` produces valid JSON array; empty store → `[]` | AC-04 | JSON Format |
| FT-05 | `name::` scopes output; exit 2 unknown; exit 1 invalid characters | AC-05 | Name Scoping |
| FT-06 | `cols::` column suppression removes lines; JSON ignores `cols::` settings | AC-06 | Column Control |
| FT-07 | All default-on columns removed via `cols::` → bare name lines, no blank separators | AC-07 | Column Control |
| FT-08 | Accounts listed alphabetically | AC-08 | Ordering |
| FT-09 | `cols::+display_name` shows `Display:` line; absent by default | AC-09 | Opt-In Fields |
| FT-10 | `cols::+host,+role,+billing,+model` show lines; absent by default | AC-10 | Opt-In Fields |
| FT-11 | Missing credential/snapshot data shows `N/A` for absent fields | AC-11 | N/A Handling |
| FT-12 | `format::json` includes extended fields on every object | AC-12 | JSON Fields |
| FT-13 | Live credential match → `Current: yes` on matched; `no` on others | AC-13 | Current Account |
| FT-14 | `cols::-current` suppresses `Current:` line; unreadable creds also suppresses | AC-14 | Current Account |
| FT-15 | `format::json` includes `is_current` boolean per account | AC-15 | JSON Fields |
| FT-16 | `cols::+uuid` shows `ID:` from snapshot; absent by default; `N/A` if no snapshot | AC-16 | Opt-In Fields |
| FT-17 | `cols::+capabilities` shows `Capabilities:` list; absent by default; `N/A` if absent | AC-17 | Opt-In Fields |
| FT-18 | `cols::+org_uuid` shows `Org ID:` from `{name}.json`; `N/A` if absent | AC-18 | Opt-In Fields |
| FT-19 | `cols::+org_name` shows `Org:` from `{name}.json`; `N/A` if absent | AC-19 | Opt-In Fields |
| FT-20 | `format::json` includes `owner` (string) and `is_owned` (bool) per account | AC-20 | JSON Fields |
| FT-21 | `format::json` includes `renewal_at`; absent/null when not set | AC-21 | JSON Fields |
| FT-22 | `format::json` emits correct `host`, `role`, `organization_role` field values | AC-12 | JSON Fields |

**Total:** 22 FT cases

---

### FT-01: Empty or absent store shows advisory, exits 0

- **Given:** No credential store directory exists, or the store directory is present but contains no `.credentials.json` files.
- **When:** `clp .accounts`
- **Then:** Output contains an advisory message (e.g., `(no accounts configured)`). Exit 0.
- **Exit:** 0
- **Source fn:** `acc03_empty_store_shows_advisory`, `acc11_missing_store_shows_advisory`
- **Source:** [003_account_list.md AC-01](../../../docs/feature/003_account_list.md)

---

### FT-02: Entry format: indented key-val blocks, blank separators, expiresAt

- **Given:** Two accounts are saved: `alice@acme.com` (non-active) and `work@acme.com` (active). `alice` has a valid `expiresAt`; `work` has no `expiresAt` field in its credentials.
- **When:** `clp .accounts`
- **Then:** Each account block has an email header line followed by indented `Key:  value` lines. A blank line separates consecutive blocks. A single account has no trailing blank line. The non-active account shows its own stored `expiresAt` value, not the active account's. An absent `expiresAt` is shown as `expired` (not an error). `Email:` line is shown from the saved `{name}.json` snapshot.
- **Exit:** 0
- **Source fn:** `acc01_lists_accounts_as_indented_blocks`, `acc13_blank_line_between_blocks`, `acc14_nonactive_shows_own_stored_expires`, `acc18_single_account_no_trailing_blank`, `acc19_missing_expires_at_shows_expired`, `acc25_email_reads_from_snapshot`
- **Source:** [003_account_list.md AC-02](../../../docs/feature/003_account_list.md)

---

### FT-03: Active account shows `Active: yes`; inactive shows `Active: no`

- **Given:** Two accounts stored; `alice@acme.com` is the active account per the per-machine active marker.
- **When:** `clp .accounts`
- **Then:** `alice@acme.com`'s block shows `Active:  yes`; the other account shows `Active:  no`.
- **Exit:** 0
- **Source fn:** `acc02_active_shows_yes_inactive_shows_no`
- **Source:** [003_account_list.md AC-03](../../../docs/feature/003_account_list.md)

---

### FT-04: `format::json` produces valid JSON array; empty store → `[]`

- **Given:** Accounts are saved in the credential store (or the store is absent/empty).
- **When (non-empty):** `clp .accounts format::json`
- **Then:** Output is a valid JSON array; each element is an object with `name`, `is_active`, `subscription_type`, `rate_limit_tier`, `expires_at_ms`, `email` keys (and extended keys per AC-12/AC-15).
- **When (empty):** `clp .accounts format::json` with no credential store.
- **Then:** Output is `[]`.
- **Exit:** 0
- **Source fn:** `acc09_json_format_array`, `acc17_json_format_empty_store`
- **Source:** [003_account_list.md AC-04](../../../docs/feature/003_account_list.md)

---

### FT-05: `name::` scopes output; exit 2 unknown; exit 1 invalid characters

- **Given:** Credential store contains `alice@acme.com.credentials.json`.
- **When:** `clp .accounts name::alice@acme.com` (exact email)
- **Then:** Output shows only the block for `alice@acme.com`. Exit 0.
- **When:** `clp .accounts name::alice` (prefix)
- **Then:** Prefix resolves to `alice@acme.com`; only that block shown. Exit 0.
- **When:** `clp .accounts alice@acme.com` (positional bare argument)
- **Then:** Shows only `alice@acme.com`'s block. Exit 0.
- **When:** `clp .accounts name::ghost@example.com` (valid but absent)
- **Then:** Exit 2 with not-found message.
- **When:** `clp .accounts name::a/b@c.com` (path-unsafe characters)
- **Then:** Exit 1.
- **Exit:** 0 / 1 / 2
- **Source fn:** `acc04_name_scopes_to_single_block`, `acc05_name_not_found_exits_2`, `acc06_name_invalid_exits_1`, `acc29_accounts_positional_bare_arg`, `acc30_accounts_prefix_resolves`
- **Source:** [003_account_list.md AC-05](../../../docs/feature/003_account_list.md)

---

### FT-06: `cols::` column suppression removes lines; JSON ignores `cols::` settings

- **Given:** An account with all standard fields populated.
- **When (text):** `clp .accounts cols::-sub,-tier`
- **Then:** `Sub:` and `Tier:` lines are absent from the block; other standard fields remain present.
- **When (json):** `clp .accounts cols::-sub,-tier format::json`
- **Then:** JSON output still includes all fields; `cols::` exclusions apply to text output only.
- **Exit:** 0
- **Source fn:** `acc07_field_presence_suppresses_lines`, `acc10_json_ignores_field_presence`
- **Source:** [003_account_list.md AC-06](../../../docs/feature/003_account_list.md)

---

### FT-07: All default-on columns removed via `cols::` → bare name lines, no blank separators

- **Given:** Two accounts in the store.
- **When:** `clp .accounts cols::-active,-owner,-current,-sub,-tier,-expires,-email`
- **Then:** Output contains only bare account name lines (no indentation, no `Key: value` pairs). No blank-line separators between accounts.
- **Exit:** 0
- **Source fn:** `acc08_all_fields_off_bare_names`
- **Source:** [003_account_list.md AC-07](../../../docs/feature/003_account_list.md)

---

### FT-08: Accounts listed alphabetically

- **Given:** Three accounts added in non-alphabetical order: `charlie@x.com`, `alice@x.com`, `bob@x.com`.
- **When:** `clp .accounts`
- **Then:** Output lists `alice@x.com`, then `bob@x.com`, then `charlie@x.com` (alphabetical by name).
- **Exit:** 0
- **Source fn:** `acc12_sorted_alphabetically`
- **Source:** [003_account_list.md AC-08](../../../docs/feature/003_account_list.md)

---

### FT-09: `cols::+display_name` shows `Display:` line; absent by default

- **Given:** An account with a saved `{name}.json` snapshot containing `oauthAccount.displayName = "Alice"`.
- **When (opt-in):** `clp .accounts cols::+display_name`
- **Then:** Block includes `Display:  Alice`.
- **When (default):** `clp .accounts` (no opt-in)
- **Then:** Block does NOT contain a `Display:` line.
- **Exit:** 0
- **Source fn:** `acc20_display_name_shows_from_snapshot`, `acc24_new_fields_absent_by_default`
- **Source:** [003_account_list.md AC-09](../../../docs/feature/003_account_list.md)

---

### FT-10: `cols::+host,+role,+billing,+model` show lines; absent by default

- **Given:** An account with `{name}.json` containing `host = "alice@workstation"`, `role = "work"` (user-defined label), `billingType = "stripe_subscription"`, and `model = "claude-opus-4-6"`.
- **When (opt-in):** `clp .accounts cols::+host,+role,+billing,+model`
- **Then:** Block includes `Host:  alice@workstation`, `Role:  work`, `Billing:  stripe_subscription`, `Model:  claude-opus-4-6`.
- **When (default):** `clp .accounts`
- **Then:** None of those four lines appear in the block.
- **Exit:** 0
- **Source fn:** `acc21_role_billing_model_from_snapshots`, `acc24_new_fields_absent_by_default`, `mre_324_role_toggle_shows_user_label`
- **Source:** [003_account_list.md AC-10](../../../docs/feature/003_account_list.md)

---

### FT-11: Missing credential/snapshot data shows `N/A` for absent fields

- **Given:** An account with a valid `{name}.credentials.json` but no `subscriptionType` or `rateLimitTier` fields; no `{name}.json` snapshot at all.
- **When:** `clp .accounts cols::+display_name,+host,+role,+billing,+model,+uuid,+capabilities`
- **Then:** `Sub:  N/A`, `Tier:  N/A`, `Email:  N/A`, `Display:  N/A`, `Host:  N/A`, `Role:  N/A`, `Billing:  N/A`, `Model:  N/A`, `ID:  N/A`, `Capabilities:  N/A`.
- **Exit:** 0
- **Source fn:** `acc15_missing_sub_field_shows_na`, `acc16_missing_tier_field_shows_na`, `acc22_no_snapshot_shows_na_for_new_fields`, `acc41_no_snapshot_uuid_capabilities_na`, `mre_324_host_role_na_when_metadata_absent`
- **Source:** [003_account_list.md AC-11](../../../docs/feature/003_account_list.md)

---

### FT-12: `format::json` includes extended fields on every account object

- **Given:** An account with snapshots for email, display_name, organization_role, workspace_uuid, workspace_name, role, host, billing, model, tagged_id, capabilities, organization_uuid, organization_name, owner, renewal_at.
- **When:** `clp .accounts format::json`
- **Then:** Each JSON object contains keys `email`, `display_name`, `role`, `billing`, `model`, `tagged_id`, `capabilities`, `organization_uuid`, `organization_name`, `organization_role`, `workspace_uuid`, `workspace_name`, `host`, `owner`, `is_owned`, `renewal_at` — regardless of `cols::` column settings.
- **Exit:** 0
- **Source fn:** `acc23_json_includes_new_fields`, `acc37_json_includes_tagged_id`, `acc40_json_includes_capabilities`, `acc45_json_includes_org_uuid`, `mre_324_json_output_keys`
- **Source:** [003_account_list.md AC-12](../../../docs/feature/003_account_list.md)

---

### FT-13: Live credential match → `Current: yes` on matched; `no` on others

- **Given:** Two accounts: `alice@acme.com` and `work@acme.com`. `~/.claude/.credentials.json` contains `work@acme.com`'s `accessToken`.
- **When:** `clp .accounts`
- **Then:** `work@acme.com`'s block shows `Current:  yes`; `alice@acme.com`'s block shows `Current:  no`.
- **Exit:** 0
- **Source fn:** `acc31_accounts_shows_current_yes_no`
- **Source:** [003_account_list.md AC-13](../../../docs/feature/003_account_list.md)

---

### FT-14: `cols::-current` suppresses `Current:` line; unreadable creds also suppresses

- **Given:** `~/.claude/.credentials.json` is absent (or the live credentials are unreadable).
- **When (unreadable):** `clp .accounts`
- **Then:** `Current:` line is absent from all blocks (no live credential file → cannot compare).
- **When (explicit toggle):** `clp .accounts cols::-current` with a live credentials file present
- **Then:** `Current:` line is absent from all blocks.
- **Exit:** 0
- **Source fn:** `acc32_accounts_suppresses_current_when_creds_absent`, `acc33_accounts_current_param_and_json`
- **Source:** [003_account_list.md AC-14](../../../docs/feature/003_account_list.md)

---

### FT-15: `format::json` includes `is_current` boolean per account

- **Given:** Two accounts; live `~/.claude/.credentials.json` matches one.
- **When:** `clp .accounts format::json`
- **Then:** Each JSON object contains `is_current: true` or `is_current: false` accordingly.
- **Exit:** 0
- **Source fn:** `acc33_accounts_current_param_and_json`
- **Source:** [003_account_list.md AC-15](../../../docs/feature/003_account_list.md)

---

### FT-16: `cols::+uuid` shows `ID:` from snapshot; absent by default; `N/A` if no snapshot

- **Given:** One account with `{name}.json` containing `oauthAccount.taggedId = "user_01abc"`. One account with no snapshot.
- **When (opt-in):** `clp .accounts cols::+uuid`
- **Then:** Account with snapshot shows `ID:  user_01abc`; account without snapshot shows `ID:  N/A`.
- **When (default):** `clp .accounts`
- **Then:** No `ID:` line in any block.
- **Exit:** 0
- **Source fn:** `acc35_uuid_shows_id_from_snapshot`, `acc36_uuid_absent_by_default`, `acc41_no_snapshot_uuid_capabilities_na`
- **Source:** [003_account_list.md AC-16](../../../docs/feature/003_account_list.md)

---

### FT-17: `cols::+capabilities` shows `Capabilities:` list; absent by default; `N/A` if absent

- **Given:** One account with `{name}.json` containing `capabilities = ["claude_max", "chat"]`. One account with no snapshot.
- **When (opt-in):** `clp .accounts cols::+capabilities`
- **Then:** Account with snapshot shows `Capabilities:  claude_max, chat`; account without snapshot shows `Capabilities:  N/A`.
- **When (default):** `clp .accounts`
- **Then:** No `Capabilities:` line in any block.
- **Exit:** 0
- **Source fn:** `acc38_capabilities_shows_list_from_snapshot`, `acc39_capabilities_absent_by_default`, `acc41_no_snapshot_uuid_capabilities_na`
- **Source:** [003_account_list.md AC-17](../../../docs/feature/003_account_list.md)

---

### FT-18: `cols::+org_uuid` shows `Org ID:` from `{name}.json`; `N/A` if absent

- **Given:** One account with `{name}.json` containing `organization_uuid = "aaaa-bbbb"`. One account with no org identity fields.
- **When (opt-in):** `clp .accounts cols::+org_uuid`
- **Then:** Account with `{name}.json` shows `Org ID:  aaaa-bbbb`; account without shows `Org ID:  N/A`.
- **When (default):** `clp .accounts`
- **Then:** No `Org ID:` line in any block.
- **Exit:** 0
- **Source fn:** `acc42_org_uuid_shows_from_roles_json`, `acc43_org_uuid_absent_by_default`, `acc44_org_uuid_missing_roles_json_na`
- **Source:** [003_account_list.md AC-18](../../../docs/feature/003_account_list.md)

---

### FT-19: `cols::+org_name` shows `Org:` from `{name}.json`; `N/A` if absent

- **Given:** One account with `{name}.json` containing `organization_name = "Acme Corp"`. One account with no org identity fields.
- **When (opt-in):** `clp .accounts cols::+org_name`
- **Then:** Account with `{name}.json` shows `Org:  Acme Corp`; account without shows `Org:  N/A`.
- **When (default):** `clp .accounts`
- **Then:** No `Org:` line in any block.
- **Exit:** 0
- **Source fn:** `acc46_org_name_shows_from_roles_json`, `acc47_org_name_absent_by_default`, `acc48_org_name_missing_roles_json_na`
- **Source:** [003_account_list.md AC-19](../../../docs/feature/003_account_list.md)

---

### FT-20: `format::json` includes `owner` and `is_owned` fields per account

- **Given:** Account A has `{name}.json` with `owner = "w003_user1"` (matches current machine identity). Account B has no `owner` field.
- **When:** `clp .accounts format::json`
- **Then:** Account A JSON object contains `owner: "w003_user1"` and `is_owned: true`. Account B JSON object contains `owner: ""` and `is_owned: true` (unowned = owned by all).
- **Exit:** 0
- **Source fn:** `mre_324_json_output_keys` (key presence), `mre_324_json_owner_is_owned_values` (matching owner → true, unowned → true), `mre_324_json_is_owned_false_for_foreign_owner` (foreign owner → false)
- **Source:** [003_account_list.md AC-20](../../../docs/feature/003_account_list.md)

---

### FT-21: `format::json` includes `renewal_at`; absent/null when not set

- **Given:** Account A has `{name}.json` with `_renewal_at = "2025-08-01T00:00:00Z"`. Account B has no `_renewal_at` field.
- **When:** `clp .accounts format::json`
- **Then:** Account A JSON object contains `renewal_at: "2025-08-01T00:00:00Z"`. Account B JSON object has `renewal_at` absent or `null`.
- **Exit:** 0
- **Source fn:** `mre_324_json_output_keys` (key presence), `mre_324_json_renewal_at_values` (value-level)
- **Source:** [003_account_list.md AC-21](../../../docs/feature/003_account_list.md)

---

### FT-22: `format::json` emits correct `host`, `role`, `organization_role` field values

- **Given:** `test@example.com` with `{name}.json` containing `host = "work-laptop"` (user-defined host label), `role = "developer"` (user-defined role label), `organization_role = "admin"` (org role from Roles API). These three fields come from distinct JSON keys.
- **When:** `clp .accounts format::json`
- **Then:** JSON object has `host: "work-laptop"`, `role: "developer"`, `organization_role: "admin"`. No cross-contamination between user role label and org role.
- **Exit:** 0
- **Source fn:** `mre_324_json_host_role_org_role_values`
- **Source:** [003_account_list.md AC-12](../../../docs/feature/003_account_list.md)
