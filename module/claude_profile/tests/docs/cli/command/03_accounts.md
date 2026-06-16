# Test: `.accounts`

Integration test planning for the `.accounts` command. See [command/namespace.md](../../../../docs/cli/command/001_account.md#command--3-accounts) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Lists all accounts as indented key-val blocks | Basic Invocation |
| IT-2 | Active account shows `Active:  yes`; others show `Active:  no` | Output Format |
| IT-3 | Empty account store → `(no accounts configured)`, exit 0 | Empty State |
| IT-4 | `name::EMAIL` scopes output to single account block | Named Account |
| IT-5 | `name::EMAIL` not in store → exit 2 | Named Account / Error |
| IT-6 | `name::notanemail` (invalid) → exit 1 | Named Account / Validation |
| IT-7 | `sub::0 tier::0` suppresses those lines from all blocks | Field Presence |
| IT-8 | All field params off → bare name per line | Field Presence |
| IT-9 | `format::json` returns valid JSON array | Output Format |
| IT-10 | `format::json` ignores field-presence params — all fields in JSON | Interaction |
| IT-11 | Credential store missing → `(no accounts configured)`, exit 0 | Edge Case |
| IT-12 | Accounts listed alphabetically by name | Output Order |
| IT-13 | Multiple accounts with fields → blank line between each block | Output Format |
| IT-14 | Non-active account shows its own stored expiry, not active's | Bug Reproducer |
| IT-15 | Missing `subscriptionType` in credential file → `Sub:     N/A` | Edge Case |
| IT-16 | Missing `rateLimitTier` in credential file → `Tier:    N/A` | Edge Case |
| IT-17 | `display_name::1` shows `Display:` line from saved `{name}.json` snapshot | Rich Metadata |
| IT-18 | `role::1 billing::1 model::1` shows corresponding lines from saved snapshots | Rich Metadata |
| IT-19 | Account without saved metadata snapshots → `N/A` for all 4 new fields | Rich Metadata / Edge Case |
| IT-20 | `format::json` includes `display_name`, `role`, `billing`, `model` keys | Rich Metadata / JSON |
| IT-21 | New metadata fields absent by default (opt-in) | Rich Metadata / Default |
| IT-22 | `email::` shows value from saved `{name}.json` snapshot (default-on) | Rich Metadata / Email |
| IT-23 | `format::table` renders one-row-per-account table with flag, Account, Sub, Tier, Expires, Email columns | Table Format |
| IT-24 | Positional bare arg `alice@acme.com` (no `name::`) shows single account | Positional Syntax |
| IT-25 | Prefix `alice` resolves to `alice@acme.com` and shows that account | Prefix Resolution |
| IT-26 | Account matching live `accessToken` shows `Current:  yes`; others show `Current:  no` | Current Account |
| IT-27 | `Current:` line suppressed when `~/.claude/.credentials.json` is unreadable | Current Account |
| IT-28 | `current::0` suppresses `Current:` line; `format::json` includes `is_current` field | Current Account |
| IT-29 | `cols::+uuid` shows `ID:` line from `taggedId` in saved `{name}.json` snapshot | Extended Snapshot |
| IT-30 | `cols::+capabilities` shows `Capabilities:` line as comma-separated list | Extended Snapshot |
| IT-31 | Empty capabilities array in snapshot → `Capabilities: N/A` | Extended Snapshot / Edge Case |
| IT-32 | Account without `{name}.json` snapshot → `ID: N/A`, `Capabilities: N/A` | Extended Snapshot / Edge Case |
| IT-33 | `cols::+org_uuid` shows `Org ID:` line from saved `{name}.json` | Org Identity |
| IT-34 | `cols::+org_name` shows `Org:` line from saved `{name}.json` | Org Identity |
| IT-35 | Account without org fields in `{name}.json` → `Org ID: N/A`, `Org: N/A` | Org Identity / Edge Case |
| IT-36 | `format::json` includes `tagged_id`, `capabilities`, `organization_uuid`, `organization_name` | Extended Snapshot / JSON |
| IT-37 | Extended snapshot columns absent from default output | Extended Snapshot / Default |
| IT-38 | `cols::+host` shows `Host:` line from saved `{name}.json` | Host Metadata |
| IT-39 | `cols::+role` shows user-defined role label from saved `{name}.json` | Host Metadata |
| IT-40 | Owner column present by default; `—` when unowned | Feature 037 — Owner Column |
| IT-41 | `cols::-owner` hides Owner column | Feature 037 — Owner Column |
| IT-42 | Unknown parameter exits 1 | Feature 037 — Param Validation |
| IT-43 | `assign::1 name::X` writes active marker; no other files changed | Feature 037 — assign mutation |
| IT-44 | `unclaim::1 name::X` clears owner field when G8 passes | Feature 037 — unclaim mutation |
| IT-45 | `unclaim::1 name::X` exits 1 with ownership violation when G8 fails | Feature 037 — unclaim mutation |
| IT-46 | `force::1 unclaim::1 name::X` bypasses G8 and clears owner | Feature 037 — force bypass |

### Test Coverage Summary

- Basic Invocation: 1 test (IT-1)
- Output Format: 3 tests (IT-2, IT-9, IT-13)
- Empty State: 1 test (IT-3)
- Named Account: 3 tests (IT-4, IT-5, IT-6)
- Column Control (cols::): 2 tests (IT-7, IT-10)
- Legacy Param Rejection: 1 test (IT-8)
- Edge Case: 4 tests (IT-11, IT-15, IT-16, IT-19)
- Output Order: 1 test (IT-12)
- Bug Reproducer: 1 test (IT-14)
- Rich Metadata: 4 tests (IT-17, IT-18, IT-20, IT-21)
- Email Display: 1 test (IT-22)
- Table Format: 1 test (IT-23)
- Positional Syntax: 1 test (IT-24)
- Prefix Resolution: 1 test (IT-25)
- Current Account: 3 tests (IT-26, IT-27, IT-28)
- Extended Snapshot: 4 tests (IT-29, IT-30, IT-31, IT-32)
- Org Identity: 3 tests (IT-33, IT-34, IT-35)
- Extended Snapshot / JSON: 1 test (IT-36)
- Extended Snapshot / Default: 1 test (IT-37)
- Host Metadata: 2 tests (IT-38, IT-39)
- Feature 037 — Owner Column: 2 tests (IT-40, IT-41)
- Feature 037 — Param Validation: 1 test (IT-42)
- Feature 037 — assign mutation: 1 test (IT-43)
- Feature 037 — unclaim mutation: 2 tests (IT-44, IT-45)
- Feature 037 — force bypass: 1 test (IT-46)

**Total:** 46 integration tests

---

### IT-1: Lists all accounts as indented key-val blocks

- **Given:** Create `~/.persistent/claude/credential/` with two credential files: `work@acme.com.credentials.json` and `personal@home.com.credentials.json`. Set `work@acme.com` as the active account via the per-machine active marker.
- **When:** `clp .accounts`
- **Then:** Output contains two indented blocks, one starting with `work@acme.com` and one with `personal@home.com`. Each block has `Owner:`, `Active:`, `Sub:`, `Tier:`, `Expires:`, `Email:` lines.; both accounts listed as indented key-val blocks with Owner column in default set
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command--3-accounts)

---

### IT-2: Active account marker

- **Given:** `work@acme.com` is active, `personal@home.com` is not. Both have credential files.
- **When:** `clp .accounts`
- **Then:** `work@acme.com` block contains `Active:  yes`; `personal@home.com` block contains `Active:  no`.; active/inactive status correctly reported per account
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command--3-accounts)

---

### IT-3: Empty account store

- **Given:** Credential store directory is absent or contains no `*.credentials.json` files.
- **When:** `clp .accounts`
- **Then:** `(no accounts configured)` with exit 0.; empty store handled gracefully with advisory message
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command--3-accounts)

---

### IT-4: Named account — scopes to single block

- **Given:** `work@acme.com` and `personal@home.com` both exist. `work@acme.com` is active.
- **When:** `clp .accounts name::work@acme.com`
- **Then:** A single block for `work@acme.com` with all default fields; no `personal@home.com` entry.; only the named account's block is shown
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command--3-accounts)

---

### IT-5: Named account — not found → exit 2

- **Given:** `work@acme.com` exists. `ghost@example.com` does NOT exist.
- **When:** `clp .accounts name::ghost@example.com`
- **Then:** Exit 2; stderr contains `not found` or `ghost@example.com`.; not-found is a runtime error, not a usage error
- **Exit:** 2
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command--3-accounts)

---

### IT-6: Named account — invalid format → exit 1

- **Given:** clean environment
- **When:** `clp .accounts name::notanemail`
- **Then:** Exit 1; stderr contains a validation error about the name format.; invalid name format is a usage error
- **Exit:** 1
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command--3-accounts)

---

### IT-7: Column suppression via `cols::` — hiding Sub and Tier

- **Given:** `work@acme.com` is the active account.
- **When:** `clp .accounts cols::-sub,-tier`
- **Then:** Blocks contain `Owner:`, `Active:`, `Expires:`, `Email:` lines but NOT `Sub:` or `Tier:` lines.; suppressed columns are absent; remaining identity-set columns present
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command--3-accounts), [feature/037_accounts_usage_param_unification.md AC-14](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-8: Legacy field-toggle params rejected — exits 1 with cols:: hint

- **Given:** Two accounts exist: `work@acme.com` (active) and `personal@home.com`.
- **When:** `clp .accounts active::0`
- **Then:** Exit 1. Error message references `cols::` syntax.; individual field toggle params removed in Feature 037 — `active::`, `sub::`, `tier::`, `expires::`, `email::`, etc. all rejected
- **Exit:** 1
- **Source:** [feature/037_accounts_usage_param_unification.md AC-13](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-9: JSON output

- **Given:** `work@acme.com` (active) and `personal@home.com` exist.
- **When:** `clp .accounts format::json`
- **Then:** Valid JSON array `[{...},{...}]` with each object containing `name`, `is_active`, `subscription_type`, `rate_limit_tier`, `expires_at_ms`, `email`.; valid JSON array with correct structure and active status
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command--3-accounts)

---

### IT-10: JSON output unaffected by cols:: exclusions

- **Given:** `work@acme.com` exists.
- **When:** `clp .accounts cols::-sub,-tier,-active format::json`
- **Then:** Valid JSON array where each object still contains `subscription_type`, `rate_limit_tier`, `is_active` fields despite those columns being excluded from text output.; `cols::` exclusions do not strip JSON keys
- **Exit:** 0
- **Source:** [004_parameter_interactions.md — Interaction 3](../../../../docs/cli/004_parameter_interactions.md#interaction--3-formatjson-overrides-field-presence-params)

---

### IT-11: Missing credential store

- **Given:** Remove `~/.persistent/claude/credential/` entirely (or ensure it never existed).
- **When:** `clp .accounts`
- **Then:** `(no accounts configured)` with exit 0. No error about missing directory.; absent store handled gracefully, same as empty store
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command--3-accounts)

---

### IT-12: Alphabetical ordering

- **Given:** Create accounts: `zed@acme.com`, `alice@acme.com`, `mike@acme.com` in non-alphabetical creation order.
- **When:** `clp .accounts`
- **Then:** Account blocks appear in alphabetical order: `alice@acme.com` block first, then `mike@acme.com`, then `zed@acme.com`.; accounts sorted alphabetically regardless of creation order
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command--3-accounts)

---

### IT-13: Blank line between blocks

- **Given:** Two accounts: `alice@acme.com` (active) and `alice@home.com`.
- **When:** `clp .accounts`
- **Then:** Stdout contains `\n\n` (blank-line separator between the two blocks).; blank-line separator present when multiple accounts with fields shown
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command--3-accounts)

---

### IT-14: Non-active account uses its own stored expiry

- **Given:** `alice@acme.com` is active with FAR_FUTURE_MS (valid token). `alice@home.com` is non-active with PAST_MS (expired token).
- **When:** `clp .accounts name::alice@home.com`
- **Then:** Stdout contains `expired`; does NOT contain `in ` (which would indicate leaking the active account's valid expiry).; non-active account shows own stored expiry, never leaking active account's live state
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command--3-accounts)

---

### IT-15: Missing `subscriptionType` → `Sub:     N/A`

- **Given:** Credential file contains `{"oauthAccount":{"rateLimitTier":"standard"},"expiresAt":9999999999000}` (no `subscriptionType`).
- **When:** `clp .accounts`
- **Then:** Stdout contains `Sub:     N/A`.; missing field shows `N/A` not blank
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command--3-accounts)

---

### IT-16: Missing `rateLimitTier` → `Tier:    N/A`

- **Given:** Credential file contains `{"oauthAccount":{"subscriptionType":"pro"},"expiresAt":9999999999000}` (no `rateLimitTier`).
- **When:** `clp .accounts`
- **Then:** Stdout contains `Tier:    N/A`.; missing field shows `N/A` not blank
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command--3-accounts)

---

### IT-17: `cols::+display_name` shows Display line from saved snapshot

- **Given:** `work@acme.com` with saved `{name}.json` containing `{"oauthAccount":{"displayName":"alice"}}`. Active account.
- **When:** `clp .accounts cols::+display_name`
- **Then:** Stdout contains `Display: alice`.; display name rendered from saved snapshot via cols:: addition
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command--3-accounts), [feature/037_accounts_usage_param_unification.md AC-14](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-18: `cols::+role,+billing,+model` shows corresponding lines

- **Given:** `work@acme.com` with saved `{name}.json` containing `{"oauthAccount":{"organizationRole":"admin","billingType":"stripe_subscription"}, "model":"sonnet"}`.
- **When:** `clp .accounts cols::+role,+billing,+model`
- **Then:** Stdout contains `Role:    admin`, `Billing: stripe_subscription`, `Model:   sonnet`.; all 3 metadata fields rendered via cols:: addition
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command--3-accounts), [feature/037_accounts_usage_param_unification.md AC-14](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-19: Account without saved metadata → N/A for non-default columns

- **Given:** `work@acme.com` with credential file only (no `{name}.json` snapshot).
- **When:** `clp .accounts cols::+display_name,+role,+billing,+model`
- **Then:** Stdout contains `Display: N/A`, `Role:    N/A`, `Billing: N/A`, `Model:   N/A`.; absent snapshots degrade gracefully
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command--3-accounts), [feature/037_accounts_usage_param_unification.md AC-14](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-20: JSON includes new metadata keys

- **Given:** `work@acme.com` with saved `{name}.json` snapshot containing display name and role.
- **When:** `clp .accounts format::json`
- **Then:** Valid JSON array where each object contains `display_name`, `role`, `billing`, `model` keys.; JSON shape includes all metadata regardless of snapshot presence
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command--3-accounts)

---

### IT-21: Non-default columns absent from default output

- **Given:** `work@acme.com` with saved `{name}.json` snapshot containing `oauthAccount` rich metadata and `model` field.
- **When:** `clp .accounts`
- **Then:** Stdout does NOT contain `Display:`, `Role:`, `Billing:`, `Model:` lines.; these columns are not in the default identity set — must be added via `cols::+display_name` etc.
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command--3-accounts), [feature/037_accounts_usage_param_unification.md AC-03](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-22: `email::` shows value from saved snapshot

- **Given:** `work@acme.com` with saved `{name}.json` containing `{"emailAddress":"work@acme.com"}`.
- **When:** `clp .accounts`
- **Then:** Stdout contains `Email:   work@acme.com`.; email address populated from saved snapshot (default-on)
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command--3-accounts)

---

### IT-23: `format::table` renders one-row-per-account table

- **Given:** `work@acme.com` (active, `subscriptionType=max`, `rateLimitTier=default_claude_max_20x`, expires far future) and `personal@home.com` (non-active, `subscriptionType=pro`, `rateLimitTier=default_claude_pro`, expires in ~5h) both exist.
- **When:** `clp .accounts format::table`
- **Then:** Stdout contains title `Accounts`, a blank line, then a header row with columns `Account`, `Sub`, `Tier`, `Expires`, `Email` (with an unlabelled flag column); `work@acme.com` row has `✓` in the flag column; `personal@home.com` row has a blank flag; both rows appear with aligned columns; field-presence params are irrelevant (table has fixed columns).
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command--3-accounts)

---

### IT-24: Positional bare arg shows single account

- **Given:** Two accounts saved: `work@acme.com` (active) and `alice@acme.com`.
- **When:** `clp .accounts alice@acme.com` (no `name::` prefix)
- **Then:** Exits 0; output identical to `clp .accounts name::alice@acme.com`; shows only the `alice@acme.com` indented block.
- **Exit:** 0
- **Source:** [015_name_shortcut_syntax.md AC-03](../../../../docs/feature/015_name_shortcut_syntax.md)

---

### IT-25: Prefix resolves to single account

- **Given:** Two accounts saved: `alice@acme.com` and `work@acme.com` (active).
- **When:** `clp .accounts alice` (prefix form, no `@`)
- **Then:** Exits 0; output identical to `clp .accounts name::alice@acme.com`; shows only the `alice@acme.com` indented block.
- **Exit:** 0
- **Source:** [015_name_shortcut_syntax.md AC-05](../../../../docs/feature/015_name_shortcut_syntax.md)

---

### IT-26: Current account shows `Current:  yes`; others show `Current:  no`

- **Given:** Two accounts: `alice@acme.com` (active) and `work@acme.com`. The live `~/.claude/.credentials.json` has an `accessToken` matching `work@acme.com`'s stored token.
- **When:** `clp .accounts`
- **Then:** `work@acme.com` block contains `Current: yes`; `alice@acme.com` block contains `Current: no`. Both blocks also show `Active:  no` / `Active:  yes` respectively.
- **Exit:** 0
- **Source:** [016_current_account_awareness.md AC-01](../../../../docs/feature/016_current_account_awareness.md)

---

### IT-27: `Current:` line suppressed when credentials file unreadable

- **Given:** Two accounts saved. `~/.claude/.credentials.json` is absent (or has no read permission).
- **When:** `clp .accounts`
- **Then:** Output contains `Active:`, `Sub:`, `Tier:`, `Expires:`, `Email:` lines for each account but does NOT contain any `Current:` line.
- **Exit:** 0
- **Source:** [016_current_account_awareness.md AC-02](../../../../docs/feature/016_current_account_awareness.md)

---

### IT-28: `cols::-current` suppresses Current line; JSON includes `is_current`

- **Given:** Two accounts saved. Live `~/.claude/.credentials.json` matches one of them.
- **When (a):** `clp .accounts cols::-current` → stdout must NOT contain `Current:` line.
- **When (b):** `clp .accounts format::json` → valid JSON array; each object contains `is_current` boolean field.
- **Exit:** 0
- **Source:** [016_current_account_awareness.md AC-03, AC-04](../../../../docs/feature/016_current_account_awareness.md), [feature/037_accounts_usage_param_unification.md AC-14](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-29: `cols::+uuid` shows `ID:` line from taggedId

- **Given:** `work@acme.com` saved with `{credential_store}/work@acme.com.json` containing `{"oauthAccount":{"taggedId":"user_abc123"}}`.
- **When:** `clp .accounts cols::+uuid`
- **Then:** Output block for `work@acme.com` contains `ID:      user_abc123`.
- **Exit:** 0
- **Source:** [021_extended_snapshot_fields.md AC-03](../../../../docs/feature/021_extended_snapshot_fields.md), [feature/037_accounts_usage_param_unification.md AC-14](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-30: `cols::+capabilities` shows `Capabilities:` as comma-separated list

- **Given:** `work@acme.com` saved with `{credential_store}/work@acme.com.json` containing `{"oauthAccount":{"capabilities":["claude_code","pro"]}}`.
- **When:** `clp .accounts cols::+capabilities`
- **Then:** Output block contains `Capabilities: claude_code, pro`.
- **Exit:** 0
- **Source:** [021_extended_snapshot_fields.md AC-04](../../../../docs/feature/021_extended_snapshot_fields.md), [feature/037_accounts_usage_param_unification.md AC-14](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-31: Empty capabilities array → `Capabilities: N/A`

- **Given:** `work@acme.com` saved with `{credential_store}/work@acme.com.json` containing `{"oauthAccount":{"capabilities":[]}}`.
- **When:** `clp .accounts cols::+capabilities`
- **Then:** Output block contains `Capabilities: N/A`.
- **Exit:** 0
- **Source:** [021_extended_snapshot_fields.md AC-09](../../../../docs/feature/021_extended_snapshot_fields.md), [feature/037_accounts_usage_param_unification.md AC-14](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-32: Account without `{name}.json` snapshot → N/A for uuid and capabilities

- **Given:** `work@acme.com` with credential file only — no `{credential_store}/work@acme.com.json` snapshot.
- **When:** `clp .accounts cols::+uuid,+capabilities`
- **Then:** Output block contains `ID:      N/A` and `Capabilities: N/A`.
- **Exit:** 0
- **Source:** [021_extended_snapshot_fields.md AC-07](../../../../docs/feature/021_extended_snapshot_fields.md), [feature/037_accounts_usage_param_unification.md AC-14](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-33: `cols::+org_uuid` shows `Org ID:` from `{name}.json`

- **Given:** `work@acme.com` saved with `{credential_store}/work@acme.com.json` containing `{"organization_uuid":"org-xyz-789","organization_name":"Acme Corp"}`.
- **When:** `clp .accounts cols::+org_uuid`
- **Then:** Output block contains `Org ID:  org-xyz-789`.
- **Exit:** 0
- **Source:** [022_org_identity_snapshot.md AC-05](../../../../docs/feature/022_org_identity_snapshot.md), [feature/037_accounts_usage_param_unification.md AC-14](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-34: `cols::+org_name` shows `Org:` from `{name}.json`

- **Given:** `work@acme.com` saved with `{credential_store}/work@acme.com.json` containing `{"organization_uuid":"org-xyz-789","organization_name":"Acme Corp"}`.
- **When:** `clp .accounts cols::+org_name`
- **Then:** Output block contains `Org:     Acme Corp`.
- **Exit:** 0
- **Source:** [022_org_identity_snapshot.md AC-06](../../../../docs/feature/022_org_identity_snapshot.md), [feature/037_accounts_usage_param_unification.md AC-14](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-35: Account without org fields in `{name}.json` → N/A for org fields

- **Given:** `work@acme.com` saved with credential file only — no org identity fields in `{credential_store}/work@acme.com.json` snapshot.
- **When:** `clp .accounts cols::+org_uuid,+org_name`
- **Then:** Output block contains `Org ID:  N/A` and `Org:     N/A`.
- **Exit:** 0
- **Source:** [022_org_identity_snapshot.md AC-05, AC-06](../../../../docs/feature/022_org_identity_snapshot.md), [feature/037_accounts_usage_param_unification.md AC-14](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-36: `format::json` includes extended snapshot and org fields

- **Given:** `work@acme.com` saved with `{name}.json` snapshot (taggedId="user_abc123", capabilities=["claude_code"], organization_uuid="org-xyz", organization_name="Acme").
- **When:** `clp .accounts format::json`
- **Then:** Valid JSON array where each object contains `tagged_id`, `capabilities`, `organization_uuid`, `organization_name` keys; `work@acme.com` has non-null values for all four.
- **Exit:** 0
- **Source:** [021_extended_snapshot_fields.md AC-06](../../../../docs/feature/021_extended_snapshot_fields.md), [022_org_identity_snapshot.md AC-09](../../../../docs/feature/022_org_identity_snapshot.md)

---

### IT-37: Extended params absent by default

- **Given:** `work@acme.com` saved with `{name}.json` containing taggedId, capabilities, and org fields.
- **When:** `clp .accounts` (no extended params)
- **Then:** Stdout does NOT contain `ID:`, `Capabilities:`, `Org ID:`, `Org:` lines. Only default-on fields present.
- **Exit:** 0
- **Source:** [021_extended_snapshot_fields.md AC-05](../../../../docs/feature/021_extended_snapshot_fields.md), [022_org_identity_snapshot.md](../../../../docs/feature/022_org_identity_snapshot.md) Design §New field-presence params

---

### IT-38: `cols::+host` shows Host line from `{name}.json`

- **Given:** `work@acme.com` saved with `{credential_store}/work@acme.com.json` containing `{"host":"workstation","role":"dev"}`.
- **When:** `clp .accounts cols::+host`
- **Then:** Output block for `work@acme.com` contains `Host:    workstation`. No `Host:` line appears in default output without `cols::+host`.
- **Exit:** 0
- **Source:** [feature/029_account_host_metadata.md FT-08](../../../../docs/feature/029_account_host_metadata.md), [feature/037_accounts_usage_param_unification.md AC-14](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-39: `cols::+role` shows user-defined role label from `{name}.json`

- **Given:** `work@acme.com` saved with `{credential_store}/work@acme.com.json` containing `{"host":"workstation","role":"dev"}`.
- **When:** `clp .accounts cols::+role`
- **Then:** Output block for `work@acme.com` contains `Role:    dev`. Label is sourced from the `role` field in `{name}.json` (written by `.account.save role::`) — not from `organizationRole` in the `oauthAccount` subtree.
- **Exit:** 0
- **Source:** [feature/029_account_host_metadata.md FT-08](../../../../docs/feature/029_account_host_metadata.md), [feature/037_accounts_usage_param_unification.md AC-14](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-40: Owner column present by default; `—` when unowned

- **Given:** `alice@acme.com` with `alice.json` containing `"owner": "testuser@testmachine"`. `bob@acme.com` with `bob.json` containing `"owner": ""`.
- **When:** `clp .accounts`
- **Then:** `alice@acme.com` block contains `Owner:   testuser@testmachine`. `bob@acme.com` block contains `Owner:   —`. Owner column is in the default identity set — no `cols::` modifier required.
- **Exit:** 0
- **Source:** [feature/037_accounts_usage_param_unification.md AC-19](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-41: `cols::-owner` hides Owner column

- **Given:** `alice@acme.com` with `alice.json` containing `"owner": "testuser@testmachine"`.
- **When:** `clp .accounts cols::-owner`
- **Then:** Stdout does NOT contain `Owner:` line in any account block.
- **Exit:** 0
- **Source:** [feature/037_accounts_usage_param_unification.md AC-19](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-42: Unknown parameter exits 1

- **Given:** `work@acme.com` exists.
- **When:** `clp .accounts unknown_param::1`
- **Then:** Exit 1. Error message references the unknown parameter name.
- **Exit:** 1
- **Source:** [feature/037_accounts_usage_param_unification.md AC-01](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-43: `assign::1 name::X` writes active marker; no other files changed

- **Given:** `alice@acme.com` exists. Record mtime of `alice.json`, `alice.credentials.json`, `~/.claude.json`.
- **When:** `clp .accounts assign::1 name::alice@acme.com`
- **Then:** Exit 0. `_active_{machine}_{user}` in credential store contains `alice@acme.com`. mtime of `alice.json`, `alice.credentials.json`, and `~/.claude.json` unchanged.
- **Exit:** 0
- **Source:** [feature/037_accounts_usage_param_unification.md AC-08](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-44: `unclaim::1 name::X` clears owner field when G8 passes

- **Given:** `alice@acme.com` with `alice.json` containing `"owner": "testuser@testmachine"`. Current identity = `testuser@testmachine`.
- **When:** `clp .accounts unclaim::1 name::alice@acme.com`
- **Then:** Exit 0. `alice.json` contains `"owner": ""`. `alice.credentials.json` mtime unchanged. Active marker unchanged.
- **Exit:** 0
- **Source:** [feature/037_accounts_usage_param_unification.md AC-05](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-45: `unclaim::1 name::X` exits 1 with ownership violation when G8 fails

- **Given:** `alice@acme.com` with `alice.json` containing `"owner": "other@remote"`. Current identity ≠ `other@remote`.
- **When:** `clp .accounts unclaim::1 name::alice@acme.com`
- **Then:** Exit 1. stdout contains `ownership violation: this account is owned by other@remote`. `alice.json` unchanged.
- **Exit:** 1
- **Source:** [feature/037_accounts_usage_param_unification.md AC-06](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-46: `force::1 unclaim::1 name::X` bypasses G8 and clears owner

- **Given:** `alice@acme.com` with `alice.json` containing `"owner": "other@remote"`. Current identity ≠ `other@remote`.
- **When:** `clp .accounts unclaim::1 name::alice@acme.com force::1`
- **Then:** Exit 0. G8 gate bypassed. `alice.json` contains `"owner": ""`. stdout contains `unclaimed alice@acme.com`.
- **Exit:** 0
- **Source:** [feature/037_accounts_usage_param_unification.md AC-20](../../../../docs/feature/037_accounts_usage_param_unification.md)
