# Test: `.accounts`

Integration test planning for the `.accounts` command. See [command/namespace.md](../../../../docs/cli/command/001_account.md#command-3-accounts) for specification.

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
| IT-43 | `assignee::USER@MACHINE name::X` writes active marker; no other files changed (Feature 065) | Feature 065 — assignee mutation |
| IT-44 | `owner::0 name::X` clears owner field when G8 passes (Feature 064) | Feature 064 — owner mutation |
| IT-45 | `owner::0 name::X` exits 1 with ownership violation when G8 fails (Feature 064) | Feature 064 — owner mutation |
| IT-46 | `force::1 owner::0 name::X` bypasses G8 and clears owner (Feature 064) | Feature 064 — force bypass |
| IT-47 | `format::json` emits correct `owner`, `is_owned`, `renewal_at` values | TSK-324 — JSON value correctness |
| IT-48 | `format::json` emits `is_owned: false` when owner is a foreign identity | TSK-324 — JSON value correctness |
| IT-49 | `format::json` emits correct `host`, `role`, `organization_role` values | TSK-324 — JSON value correctness |
| IT-50 | `.accounts.help` shows all 6 group headers in documented order | Grouped Help — Structure |
| IT-51 | `.accounts.help` shows all 30 parameters under their correct documented group | Grouped Help — Structure |
| IT-52 | `.accounts.help` `::` delimiter aligns at the same offset across all 30 rows, spanning group boundaries | Grouped Help — Alignment |
| IT-53 | `.accounts.help` boolean parameters render bare `name::0`, never `name::0\|1` | Grouped Help — Signature Conventions |
| IT-54 | `.accounts.help` enum parameters show an uppercase value placeholder | Grouped Help — Signature Conventions |
| IT-55 | `.accounts.help` output contains no version/build banner | Grouped Help — Minimal Content |
| IT-56 | `.accounts.help` output contains no REMOVED parameter mentions | Grouped Help — Minimal Content |
| IT-57 | `.accounts.help` piped output uses plain-text group header fallback, no brackets/ANSI | Grouped Help — TTY Fallback |
| IT-58 | REMOVED_TOGGLE runtime rejection unaffected by grouped-help rendering | Grouped Help — Regression |
| IT-59 | Longest parameter name (`exclude_exhausted`) sets the shared `::` column even for the shortest name (`dry`) in a different group | Grouped Help — Alignment |
| IT-60 | Every rendered parameter belongs to exactly one group; rendered row count equals registered parameter count | Grouped Help — Exhaustiveness |
| IT-61 | Trailing `key::value` token after `.accounts.help` is silently ignored; help still renders | Grouped Help — Argv Interaction |
| IT-62 | Trailing bare (non-`::`) token after `.accounts.help` causes a usage error; help does not render | Grouped Help — Argv Interaction |
| IT-63 | Literal `.help`/`help` token elsewhere in argv takes precedence over `.accounts.help` | Grouped Help — Argv Interaction |
| IT-64 | `.accounts.help` is absent from `.help`'s own command listing | Grouped Help — Dispatch Scope |
| IT-65 | `.accounts.help` literal-token match is case-sensitive | Grouped Help — Dispatch Scope |
| IT-66 | (N/A) empty-`PARAMS` width fallback is structurally unreachable | Grouped Help — Defensive Path |

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
- TSK-324 — JSON value correctness: 3 tests (IT-47, IT-48, IT-49)
- Grouped Help — Structure: 2 tests (IT-50, IT-51)
- Grouped Help — Alignment: 2 tests (IT-52, IT-59)
- Grouped Help — Signature Conventions: 2 tests (IT-53, IT-54)
- Grouped Help — Minimal Content: 2 tests (IT-55, IT-56)
- Grouped Help — TTY Fallback: 1 test (IT-57)
- Grouped Help — Regression: 1 test (IT-58)
- Grouped Help — Exhaustiveness: 1 test (IT-60)
- Grouped Help — Argv Interaction: 3 tests (IT-61, IT-62, IT-63)
- Grouped Help — Dispatch Scope: 2 tests (IT-64, IT-65)
- Grouped Help — Defensive Path: 1 test (IT-66, N/A)

**Total:** 66 integration tests (65 active + 1 N/A)

---

### IT-1: Lists all accounts as indented key-val blocks

- **Given:** Create `~/.persistent/claude/credential/` with two credential files: `work@acme.com.credentials.json` and `personal@home.com.credentials.json`. Set `work@acme.com` as the active account via the per-machine active marker.
- **When:** `clp .accounts`
- **Then:** Output contains two indented blocks, one starting with `work@acme.com` and one with `personal@home.com`. Each block has `Owner:`, `Active:`, `Sub:`, `Tier:`, `Expires:`, `Email:` lines.; both accounts listed as indented key-val blocks with Owner column in default set
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command-3-accounts)

---

### IT-2: Active account marker

- **Given:** `work@acme.com` is active, `personal@home.com` is not. Both have credential files.
- **When:** `clp .accounts`
- **Then:** `work@acme.com` block contains `Active:  yes`; `personal@home.com` block contains `Active:  no`.; active/inactive status correctly reported per account
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command-3-accounts)

---

### IT-3: Empty account store

- **Given:** Credential store directory is absent or contains no `*.credentials.json` files.
- **When:** `clp .accounts`
- **Then:** `(no accounts configured)` with exit 0.; empty store handled gracefully with advisory message
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command-3-accounts)

---

### IT-4: Named account — scopes to single block

- **Given:** `work@acme.com` and `personal@home.com` both exist. `work@acme.com` is active.
- **When:** `clp .accounts name::work@acme.com`
- **Then:** A single block for `work@acme.com` with all default fields; no `personal@home.com` entry.; only the named account's block is shown
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command-3-accounts)

---

### IT-5: Named account — not found → exit 2

- **Given:** `work@acme.com` exists. `ghost@example.com` does NOT exist.
- **When:** `clp .accounts name::ghost@example.com`
- **Then:** Exit 2; stderr contains `not found` or `ghost@example.com`.; not-found is a runtime error, not a usage error
- **Exit:** 2
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command-3-accounts)

---

### IT-6: Named account — invalid format → exit 1

- **Given:** clean environment
- **When:** `clp .accounts name::notanemail`
- **Then:** Exit 1; stderr contains a validation error about the name format.; invalid name format is a usage error
- **Exit:** 1
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command-3-accounts)

---

### IT-7: Column suppression via `cols::` — hiding Sub and Tier

- **Given:** `work@acme.com` is the active account.
- **When:** `clp .accounts cols::-sub,-tier`
- **Then:** Blocks contain `Owner:`, `Active:`, `Expires:`, `Email:` lines but NOT `Sub:` or `Tier:` lines.; suppressed columns are absent; remaining identity-set columns present
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command-3-accounts), [feature/037_accounts_usage_param_unification.md AC-14](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-8: Legacy field-toggle params rejected — exits 1 with cols:: hint

- **Given:** Two accounts exist: `work@acme.com` (active) and `personal@home.com`.
- **When:** `clp .accounts active::0`
- **Then:** Exit 1. `active::` exits 1 with REMOVED_TOGGLE migration message pointing to `assignee::` (Feature 065); other legacy field toggles (`sub::`, `tier::`, `expires::`, `email::`, etc.) exit 1 with `cols::` migration hint (Feature 037).
- **Exit:** 1
- **Source:** [feature/037_accounts_usage_param_unification.md AC-13](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-9: JSON output

- **Given:** `work@acme.com` (active) and `personal@home.com` exist.
- **When:** `clp .accounts format::json`
- **Then:** Valid JSON array `[{...},{...}]` with each object containing `name`, `is_active`, `subscription_type`, `rate_limit_tier`, `expires_at_ms`, `email`.; valid JSON array with correct structure and active status
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command-3-accounts)

---

### IT-10: JSON output unaffected by cols:: exclusions

- **Given:** `work@acme.com` exists.
- **When:** `clp .accounts cols::-sub,-tier,-active format::json`
- **Then:** Valid JSON array where each object still contains `subscription_type`, `rate_limit_tier`, `is_active` fields despite those columns being excluded from text output.; `cols::` exclusions do not strip JSON keys
- **Exit:** 0
- **Source:** [004_parameter_interactions.md — Interaction 3](../../../../docs/cli/004_parameter_interactions.md#interaction-2-formatjson-overrides-field-presence-params)

---

### IT-11: Missing credential store

- **Given:** Remove `~/.persistent/claude/credential/` entirely (or ensure it never existed).
- **When:** `clp .accounts`
- **Then:** `(no accounts configured)` with exit 0. No error about missing directory.; absent store handled gracefully, same as empty store
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command-3-accounts)

---

### IT-12: Alphabetical ordering

- **Given:** Create accounts: `zed@acme.com`, `alice@acme.com`, `mike@acme.com` in non-alphabetical creation order.
- **When:** `clp .accounts`
- **Then:** Account blocks appear in alphabetical order: `alice@acme.com` block first, then `mike@acme.com`, then `zed@acme.com`.; accounts sorted alphabetically regardless of creation order
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command-3-accounts)

---

### IT-13: Blank line between blocks

- **Given:** Two accounts: `alice@acme.com` (active) and `alice@home.com`.
- **When:** `clp .accounts`
- **Then:** Stdout contains `\n\n` (blank-line separator between the two blocks).; blank-line separator present when multiple accounts with fields shown
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command-3-accounts)

---

### IT-14: Non-active account uses its own stored expiry

- **Given:** `alice@acme.com` is active with FAR_FUTURE_MS (valid token). `alice@home.com` is non-active with PAST_MS (expired token).
- **When:** `clp .accounts name::alice@home.com`
- **Then:** Stdout contains `expired`; does NOT contain `in ` (which would indicate leaking the active account's valid expiry).; non-active account shows own stored expiry, never leaking active account's live state
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command-3-accounts)

---

### IT-15: Missing `subscriptionType` → `Sub:     N/A`

- **Given:** Credential file contains `{"oauthAccount":{"rateLimitTier":"standard"},"expiresAt":9999999999000}` (no `subscriptionType`).
- **When:** `clp .accounts`
- **Then:** Stdout contains `Sub:     N/A`.; missing field shows `N/A` not blank
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command-3-accounts)

---

### IT-16: Missing `rateLimitTier` → `Tier:    N/A`

- **Given:** Credential file contains `{"oauthAccount":{"subscriptionType":"pro"},"expiresAt":9999999999000}` (no `rateLimitTier`).
- **When:** `clp .accounts`
- **Then:** Stdout contains `Tier:    N/A`.; missing field shows `N/A` not blank
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command-3-accounts)

---

### IT-17: `cols::+display_name` shows Display line from saved snapshot

- **Given:** `work@acme.com` with saved `{name}.json` containing `{"oauthAccount":{"displayName":"alice"}}`. Active account.
- **When:** `clp .accounts cols::+display_name`
- **Then:** Stdout contains `Display: alice`.; display name rendered from saved snapshot via cols:: addition
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command-3-accounts), [feature/037_accounts_usage_param_unification.md AC-14](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-18: `cols::+role,+billing,+model` shows corresponding lines

- **Given:** `work@acme.com` with saved `{name}.json` containing `{"role":"dev","oauthAccount":{"billingType":"stripe_subscription"},"model":"sonnet"}`.
- **When:** `clp .accounts cols::+role,+billing,+model`
- **Then:** Stdout contains `Role:    dev`, `Billing: stripe_subscription`, `Model:   sonnet`. `role` value comes from the top-level `"role"` field — not from `oauthAccount.organizationRole`.; all 3 metadata fields rendered via cols:: addition
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command-3-accounts), [feature/037_accounts_usage_param_unification.md AC-14](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-19: Account without saved metadata → N/A for non-default columns

- **Given:** `work@acme.com` with credential file only (no `{name}.json` snapshot).
- **When:** `clp .accounts cols::+display_name,+role,+billing,+model`
- **Then:** Stdout contains `Display: N/A`, `Role:    N/A`, `Billing: N/A`, `Model:   N/A`.; absent snapshots degrade gracefully
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command-3-accounts), [feature/037_accounts_usage_param_unification.md AC-14](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-20: JSON includes new metadata keys

- **Given:** `work@acme.com` with saved `{name}.json` snapshot containing display name and role.
- **When:** `clp .accounts format::json`
- **Then:** Valid JSON array where each object contains `display_name`, `role`, `billing`, `model` keys.; JSON shape includes all metadata regardless of snapshot presence
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command-3-accounts)

---

### IT-21: Non-default columns absent from default output

- **Given:** `work@acme.com` with saved `{name}.json` snapshot containing `oauthAccount` rich metadata and `model` field.
- **When:** `clp .accounts`
- **Then:** Stdout does NOT contain `Display:`, `Role:`, `Billing:`, `Model:` lines.; these columns are not in the default identity set — must be added via `cols::+display_name` etc.
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command-3-accounts), [feature/037_accounts_usage_param_unification.md AC-03](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-22: `email::` shows value from saved snapshot

- **Given:** `work@acme.com` with saved `{name}.json` containing `{"emailAddress":"work@acme.com"}`.
- **When:** `clp .accounts`
- **Then:** Stdout contains `Email:   work@acme.com`.; email address populated from saved snapshot (default-on)
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command-3-accounts)

---

### IT-23: `format::table` renders one-row-per-account table

- **Given:** `work@acme.com` (active, `subscriptionType=max`, `rateLimitTier=default_claude_max_20x`, expires far future) and `personal@home.com` (non-active, `subscriptionType=pro`, `rateLimitTier=default_claude_pro`, expires in ~5h) both exist.
- **When:** `clp .accounts format::table`
- **Then:** Stdout contains title `Accounts`, a blank line, then a header row with columns `Account`, `Sub`, `Tier`, `Expires`, `Email` (with an unlabelled flag column); `work@acme.com` row has `✓` in the flag column; `personal@home.com` row has a blank flag; both rows appear with aligned columns; field-presence params are irrelevant (table has fixed columns).
- **Exit:** 0
- **Source:** [command/001_account.md — .accounts](../../../../docs/cli/command/001_account.md#command-3-accounts)

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

### IT-43: `assignee::USER@MACHINE name::X` writes active marker; no other files changed (Feature 065)

- **Given:** `alice@acme.com` exists. Record mtime of `alice.json`, `alice.credentials.json`, `~/.claude.json`.
- **When:** `clp .accounts assignee::testuser@testmachine name::alice@acme.com` (formerly `assign::1 name::X` — Feature 037; then `active::` — Feature 064; renamed `assignee::` — Feature 065)
- **Then:** Exit 0. `_active_testmachine_testuser` in credential store contains `alice@acme.com`. mtime of `alice.json`, `alice.credentials.json`, and `~/.claude.json` unchanged.
- **Exit:** 0
- **Source:** [feature/037_accounts_usage_param_unification.md AC-08](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-44: `owner::0 name::X` clears owner field when G8 passes (Feature 064)

- **Given:** `alice@acme.com` with `alice.json` containing `"owner": "testuser@testmachine"`. Current identity = `testuser@testmachine`.
- **When:** `clp .accounts owner::0 name::alice@acme.com` (formerly `unclaim::1 name::X` — Feature 064)
- **Then:** Exit 0. `alice.json` contains `"owner": ""`. `alice.credentials.json` mtime unchanged. Active marker unchanged.
- **Exit:** 0
- **Source:** [feature/037_accounts_usage_param_unification.md AC-05](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-45: `owner::0 name::X` exits 1 with ownership violation when G8 fails (Feature 064)

- **Given:** `alice@acme.com` with `alice.json` containing `"owner": "other@remote"`. Current identity ≠ `other@remote`.
- **When:** `clp .accounts owner::0 name::alice@acme.com` (formerly `unclaim::1 name::X` — Feature 064)
- **Then:** Exit 1. stdout contains `ownership violation: this account is owned by other@remote`. `alice.json` unchanged.
- **Exit:** 1
- **Source:** [feature/037_accounts_usage_param_unification.md AC-06](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-46: `force::1 owner::0 name::X` bypasses G8 and clears owner (Feature 064)

- **Given:** `alice@acme.com` with `alice.json` containing `"owner": "other@remote"`. Current identity ≠ `other@remote`.
- **When:** `clp .accounts owner::0 name::alice@acme.com force::1` (formerly `unclaim::1 name::X force::1` — Feature 064)
- **Then:** Exit 0. G8 gate bypassed. `alice.json` contains `"owner": ""`. stdout contains `unclaimed alice@acme.com`.
- **Exit:** 0
- **Source:** [feature/037_accounts_usage_param_unification.md AC-20](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-47: `format::json` emits correct `owner`, `is_owned`, `renewal_at` values

- **Given:** `alice@acme.com` with `alice.json` containing `"owner": "testuser@testmachine"` and `"_renewal_at": "2025-08-01T00:00:00Z"`. `bob@acme.com` with no owner or renewal fields.
- **When:** `clp .accounts format::json` with current identity = `testuser@testmachine`.
- **Then:** alice JSON object has `owner: "testuser@testmachine"`, `is_owned: true`, `renewal_at: "2025-08-01T00:00:00Z"`. bob JSON object has `owner: ""`, `is_owned: true` (unowned = all-owned), `renewal_at: null`.
- **Exit:** 0
- **Source:** [feature/003_account_list.md AC-20, AC-21](../../../../docs/feature/003_account_list.md)

---

### IT-48: `format::json` emits `is_owned: false` when owner is a foreign identity

- **Given:** `alice@acme.com` with `alice.json` containing `"owner": "other@remote"`. Current identity is `local@localmachine` (USER=local, HOSTNAME=localmachine).
- **When:** `clp .accounts format::json`
- **Then:** alice JSON object has `owner: "other@remote"` and `is_owned: false` (foreign owner).
- **Exit:** 0
- **Source:** [feature/003_account_list.md AC-20](../../../../docs/feature/003_account_list.md)

---

### IT-49: `format::json` emits correct `host`, `role`, `organization_role` values

- **Given:** `test@example.com` with `{name}.json` containing `host = "work-laptop"`, `role = "developer"`, `organization_role = "admin"`.
- **When:** `clp .accounts format::json`
- **Then:** JSON object has `host: "work-laptop"`, `role: "developer"`, `organization_role: "admin"`.
- **Exit:** 0
- **Source:** [feature/003_account_list.md AC-12](../../../../docs/feature/003_account_list.md)

---

## Grouped `.accounts.help` Rendering (Task 413)

Test cases below cover the grouped, `::`-aligned `.accounts.help` rendering scheme — a presentation-layer variant of the `.accounts` command's help output, dispatched via a literal first-token bypass in `src/cli.rs` before unilang's normal registry/parser pipeline runs. See [pattern/001_grouped_help_rendering.md](../../../../docs/pattern/001_grouped_help_rendering.md) and [command/001_account.md § Help Rendering Scheme](../../../../docs/cli/command/001_account.md#command-3-accounts) for specification.

---

### IT-50: `.accounts.help` shows all 6 group headers in documented order

- **Given:** clean environment, `clp` on PATH
- **When:** `clp .accounts.help`
- **Then:** stdout contains all 6 group headers — `Core`, `Account Ownership`, `Sort Control`, `Row Filtering & Pagination`, `Display Rendering`, `Refresh & Subprocess Control` — with each header's first character offset strictly increasing (documented order, not alphabetical or registration order).
- **Exit:** 0
- **Source:** [pattern/001_grouped_help_rendering.md](../../../../docs/pattern/001_grouped_help_rendering.md), [command/001_account.md § Help Rendering Scheme](../../../../docs/cli/command/001_account.md#command-3-accounts)

---

### IT-51: `.accounts.help` shows all 30 parameters under their correct documented group

- **Given:** clean environment, `clp` on PATH
- **When:** `clp .accounts.help`
- **Then:** within each group header's text slice (from that header to the next header or end of output), every parameter documented under that group in `001_account.md § Help Rendering Scheme` appears as a line beginning with that parameter's name — e.g. `sort`, `desc`, `prefer` all appear within the `Sort Control` slice and nowhere else.
- **Exit:** 0
- **Source:** [command/001_account.md § Help Rendering Scheme](../../../../docs/cli/command/001_account.md#command-3-accounts)

---

### IT-52: `::` delimiter aligns at the same offset across all 30 rows

- **Given:** clean environment
- **When:** `clp .accounts.help`
- **Then:** the `::` delimiter appears at the identical character offset on all 30 parameter rows, including rows in different group blocks — the alignment column is computed once, globally, not reset per group.
- **Exit:** 0
- **Source:** [pattern/001_grouped_help_rendering.md](../../../../docs/pattern/001_grouped_help_rendering.md) (Solution point 3)

---

### IT-53: boolean parameters render bare (`name::0`, never `name::0|1`)

- **Given:** clean environment
- **When:** `clp .accounts.help`
- **Then:** every boolean parameter (`dry`, `trace`, `force`, `refresh`, `touch`, `desc`, `only_active`, `only_next`, `only_valid`, `exclude_exhausted`, `abs`, `no_color`, `live`) renders its signature as `name::0`; stdout contains no `0|1` alternation anywhere.
- **Exit:** 0
- **Source:** [pattern/001_grouped_help_rendering.md](../../../../docs/pattern/001_grouped_help_rendering.md) (Solution point 4)

---

### IT-54: enum parameters show an uppercase value placeholder

- **Given:** clean environment
- **When:** `clp .accounts.help`
- **Then:** `imodel::MODEL`, `effort::EFFORT`, `set_model::MODEL`, `format::FORMAT`, `sort::SORT`, `prefer::PREFER` each appear verbatim in the signature column; actual enum values (e.g. `opus`, `sonnet`) appear only in the description column, never in the signature.
- **Exit:** 0
- **Source:** [pattern/001_grouped_help_rendering.md](../../../../docs/pattern/001_grouped_help_rendering.md) (Solution point 5)

---

### IT-55: no version/build banner

- **Given:** clean environment
- **When:** `clp .accounts.help`
- **Then:** stdout contains no case-insensitive occurrence of `version`.
- **Exit:** 0
- **Source:** [pattern/001_grouped_help_rendering.md](../../../../docs/pattern/001_grouped_help_rendering.md) (Solution point 6)

---

### IT-56: no mention of REMOVED parameters

- **Given:** clean environment
- **When:** `clp .accounts.help`
- **Then:** stdout contains none of `assign::`, `for::`, `unclaim::`, `active::` — no footer count, no reveal flag; removed parameters are simply invisible from `.help` text.
- **Exit:** 0
- **Source:** [command/001_account.md § Help Rendering Scheme](../../../../docs/cli/command/001_account.md#command-3-accounts)

---

### IT-57: piped output uses plain-text group header fallback

- **Given:** stdout is piped (non-TTY), clean environment
- **When:** `clp .accounts.help | cat`
- **Then:** group headers render as plain text with a single trailing colon (e.g. `Core:`); output contains no `[`/`]` bracket punctuation and no ANSI escape sequences.
- **Exit:** 0
- **Source:** [pattern/001_grouped_help_rendering.md](../../../../docs/pattern/001_grouped_help_rendering.md) (Solution point 2)

---

### IT-58: REMOVED_TOGGLE runtime rejection unaffected by grouped-help rendering

- **Given:** clean environment
- **When:** `clp .account.rotate` and, separately, `clp .accounts active::0`
- **Then:** both still exit 1 with their existing deprecation/migration error text — the grouped-help rendering change is presentation-only and did not alter runtime dispatch for removed or deprecated tokens.
- **Exit:** 1
- **Source:** [feature/037_accounts_usage_param_unification.md](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-59: longest parameter name sets the shared alignment column (boundary)

- **Given:** clean environment. Among the 30 rendered parameters, `exclude_exhausted` (17 characters) is the longest name and renders under `Row Filtering & Pagination`; `dry` (3 characters) is one of the shortest and renders under the unrelated `Core` group.
- **When:** `clp .accounts.help`
- **Then:** the `::` offset on the `dry` row equals the `::` offset on the `exclude_exhausted` row — the single longest name across all 30 parameters sets the shared alignment column, even for the shortest name in a different group.
- **Exit:** 0
- **Source:** `src/commands/accounts_help.rs` (`let width = PARAMS.iter().map(|p| p.name.len()).max()...`, computed once before per-group rendering) — the global-vs-per-group boundary itself is not a standalone claim in `001_account.md`, only implied by "aligned across all 32 rows".

---

### IT-60: every rendered parameter belongs to exactly one group; row count matches the registry

- **Given:** clean environment. `.accounts`'s registered parameter set (per `src/registry.rs`) is the ground truth for what `.accounts.help` must render.
- **When:** `clp .accounts.help`
- **Then:** every rendered parameter name maps to exactly one of the 6 documented groups (no line appears outside any group block); the count of rendered parameter rows equals the count of currently-registered `.accounts` parameters — not a value hardcoded independently of the registry.
- **Exit:** 0
- **Source:** [command/001_account.md § Help Rendering Scheme](../../../../docs/cli/command/001_account.md#command-3-accounts), `src/registry.rs`

---

### IT-61: trailing `key::value` token after `.accounts.help` is silently ignored

- **Given:** clean environment
- **When:** `clp .accounts.help name::alice@acme.com`
- **Then:** stdout is byte-identical to bare `clp .accounts.help` — the trailing `name::value` token is silently ignored because `.accounts.help` is matched as the literal first token before unilang parsing or registry dispatch begins.
- **Exit:** 0
- **Source:** `src/cli.rs` (`tokens.first().map(String::as_str) == Some(".accounts.help")` precedes parser/registry dispatch)

---

### IT-62: trailing bare (non-`::`) token after `.accounts.help` errors

- **Given:** clean environment
- **When:** `clp .accounts.help foo`
- **Then:** exits 1 with an `expected param::value syntax, got: 'foo'` error; the grouped help text does NOT render — `.accounts.help` is not a member of `POSITIONAL_NAME_COMMANDS`, so a bare trailing token is rejected by the adapter before the literal-token dispatch is ever reached.
- **Exit:** 1
- **Source:** `src/adapter.rs` (`POSITIONAL_NAME_COMMANDS`, `split_first_colons` rejection path)

---

### IT-63: a literal `.help`/`help` token elsewhere in argv takes precedence

- **Given:** clean environment
- **When:** `clp .accounts.help .help` — and, separately, `clp .help .accounts.help`
- **Then:** the generic flat `.help` listing renders (identical to bare `clp .help`), NOT the grouped `.accounts.help` rendering, regardless of token order — `needs_help` detection (`.help` or bare `help` anywhere in argv) is evaluated before the `.accounts.help` first-token check and returns early.
- **Exit:** 0
- **Source:** `src/adapter.rs` (Step 1b: `argv.iter().any(|a| a == ".help" || a == "help")`), `src/cli.rs` (`needs_help` short-circuit precedes the `.accounts.help` check)

---

### IT-64: `.accounts.help` is absent from `.help`'s own command listing

- **Given:** clean environment
- **When:** `clp .help`
- **Then:** stdout does not list `.accounts.help` as a command entry — it is a pre-registry dispatch bypass in `cli.rs`, never passed to `build_registry()`, so it cannot appear in unilang's auto-generated command listing.
- **Exit:** 0
- **Source:** `src/cli.rs`; [02_help.md IT-1](02_help.md), [02_help.md IT-2](02_help.md)

---

### IT-65: `.accounts.help` literal-token match is case-sensitive

- **Given:** clean environment
- **When:** `clp .Accounts.Help`
- **Then:** does not match the `.accounts.help` literal-token check (Rust `==` string comparison is case-sensitive); falls through to normal unilang dispatch and is rejected as an unrecognized command.
- **Exit:** 1
- **Source:** `src/cli.rs` (`tokens.first().map(String::as_str) == Some(".accounts.help")`)

---

### IT-66: empty-`PARAMS` width fallback (`unwrap_or(0)`) is structurally unreachable

> **N/A** — `PARAMS` is a fixed 30-entry compile-time `const` array in `src/commands/accounts_help.rs`; no CLI invocation can produce an empty parameter set for `print_accounts_help()` to render, so the `.max().unwrap_or(0)` fallback on the width computation can never actually execute from any observable CLI entry point.
> Becomes testable when: no committed task — would require exposing the width computation as a unit-testable function parameterized over an arbitrary parameter slice, which is out of this feature's scope.

- **Given:** `print_accounts_help()`'s width computation would need `PARAMS` to be empty for the `unwrap_or(0)` branch to execute.
- **When:** N/A — `PARAMS` cannot be empty; it is a hardcoded compile-time const with 30 literal entries, not a runtime-populated collection.
- **Then:** N/A — no observable CLI behavior exercises this branch.
- **Exit:** N/A
- **Source:** `src/commands/accounts_help.rs` (`const PARAMS : [ ParamSpec ; 30 ]`)
