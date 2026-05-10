# Test: `.accounts`

Integration test planning for the `.accounts` command. See [commands.md](../../../../docs/cli/commands.md#command--3-accounts) for specification.

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
| IT-17 | `display_name::1` shows `Display:` line from saved `.claude.json` snapshot | Rich Metadata |
| IT-18 | `role::1 billing::1 model::1` shows corresponding lines from saved snapshots | Rich Metadata |
| IT-19 | Account without saved metadata snapshots → `N/A` for all 4 new fields | Rich Metadata / Edge Case |
| IT-20 | `format::json` includes `display_name`, `role`, `billing`, `model` keys | Rich Metadata / JSON |
| IT-21 | New metadata fields absent by default (opt-in) | Rich Metadata / Default |
| IT-22 | `email::` shows value from saved `.claude.json` snapshot (default-on) | Rich Metadata / Email |

### Test Coverage Summary

- Basic Invocation: 1 test (IT-1)
- Output Format: 3 tests (IT-2, IT-9, IT-13)
- Empty State: 1 test (IT-3)
- Named Account: 3 tests (IT-4, IT-5, IT-6)
- Field Presence: 2 tests (IT-7, IT-8)
- Interaction: 1 test (IT-10)
- Edge Case: 4 tests (IT-11, IT-15, IT-16, IT-19)
- Output Order: 1 test (IT-12)
- Bug Reproducer: 1 test (IT-14)
- Rich Metadata: 4 tests (IT-17, IT-18, IT-20, IT-21)
- Rich Metadata / Email: 1 test (IT-22)

**Total:** 22 integration tests

---

### IT-1: Lists all accounts as indented key-val blocks

- **Given:** Create `~/.persistent/claude/credential/` with two credential files: `work@acme.com.credentials.json` and `personal@home.com.credentials.json`. Set `work@acme.com` as the active account via the `_active` marker.
- **When:** `clp .accounts`
- **Then:** Output contains two indented blocks, one starting with `work@acme.com` and one with `personal@home.com`. Each block has `Active:`, `Sub:`, `Tier:`, `Expires:`, `Email:` lines.; both accounts listed as indented key-val blocks
- **Exit:** 0
- **Source:** [commands.md — .accounts](../../../../docs/cli/commands.md#command--3-accounts)

---

### IT-2: Active account marker

- **Given:** `work@acme.com` is active, `personal@home.com` is not. Both have credential files.
- **When:** `clp .accounts`
- **Then:** `work@acme.com` block contains `Active:  yes`; `personal@home.com` block contains `Active:  no`.; active/inactive status correctly reported per account
- **Exit:** 0
- **Source:** [commands.md — .accounts](../../../../docs/cli/commands.md#command--3-accounts)

---

### IT-3: Empty account store

- **Given:** Credential store directory is absent or contains no `*.credentials.json` files.
- **When:** `clp .accounts`
- **Then:** `(no accounts configured)` with exit 0.; empty store handled gracefully with advisory message
- **Exit:** 0
- **Source:** [commands.md — .accounts](../../../../docs/cli/commands.md#command--3-accounts)

---

### IT-4: Named account — scopes to single block

- **Given:** `work@acme.com` and `personal@home.com` both exist. `work@acme.com` is active.
- **When:** `clp .accounts name::work@acme.com`
- **Then:** A single block for `work@acme.com` with all default fields; no `personal@home.com` entry.; only the named account's block is shown
- **Exit:** 0
- **Source:** [commands.md — .accounts](../../../../docs/cli/commands.md#command--3-accounts)

---

### IT-5: Named account — not found → exit 2

- **Given:** `work@acme.com` exists. `ghost@example.com` does NOT exist.
- **When:** `clp .accounts name::ghost@example.com`
- **Then:** Exit 2; stderr contains `not found` or `ghost@example.com`.; not-found is a runtime error, not a usage error
- **Exit:** 2
- **Source:** [commands.md — .accounts](../../../../docs/cli/commands.md#command--3-accounts)

---

### IT-6: Named account — invalid format → exit 1

- **Given:** clean environment
- **When:** `clp .accounts name::notanemail`
- **Then:** Exit 1; stderr contains a validation error about the name format.; invalid name format is a usage error
- **Exit:** 1
- **Source:** [commands.md — .accounts](../../../../docs/cli/commands.md#command--3-accounts)

---

### IT-7: Field presence — suppressing individual lines

- **Given:** `work@acme.com` is the active account.
- **When:** `clp .accounts sub::0 tier::0`
- **Then:** Blocks contain `Active:`, `Expires:`, `Email:` lines but NOT `Sub:` or `Tier:` lines.; suppressed field lines are absent; non-suppressed lines remain
- **Exit:** 0
- **Source:** [commands.md — .accounts](../../../../docs/cli/commands.md#command--3-accounts), [parameter_groups.md — Field Presence](../../../../docs/cli/parameter_groups.md#group--2-field-presence)

---

### IT-8: All field params off — bare name list

- **Given:** Two accounts exist: `work@acme.com` (active) and `personal@home.com`.
- **When:** `clp .accounts active::0 sub::0 tier::0 expires::0 email::0`
- **Then:** Two bare name lines, no indented fields, no blank-line separators between them.; bare name list when all fields off
- **Exit:** 0
- **Source:** [commands.md — .accounts](../../../../docs/cli/commands.md#command--3-accounts)

---

### IT-9: JSON output

- **Given:** `work@acme.com` (active) and `personal@home.com` exist.
- **When:** `clp .accounts format::json`
- **Then:** Valid JSON array `[{...},{...}]` with each object containing `name`, `is_active`, `subscription_type`, `rate_limit_tier`, `expires_at_ms`, `email`.; valid JSON array with correct structure and active status
- **Exit:** 0
- **Source:** [commands.md — .accounts](../../../../docs/cli/commands.md#command--3-accounts)

---

### IT-10: JSON overrides field-presence params

- **Given:** `work@acme.com` exists.
- **When:** `clp .accounts sub::0 tier::0 active::0 format::json`
- **Then:** Valid JSON array where each object still contains `subscription_type`, `rate_limit_tier`, `is_active` fields despite the presence params being disabled.; field-presence params do not strip JSON keys
- **Exit:** 0
- **Source:** [parameter_interactions.md — Interaction 3](../../../../docs/cli/parameter_interactions.md#interaction--3-formatjson-overrides-field-presence-params)

---

### IT-11: Missing credential store

- **Given:** Remove `~/.persistent/claude/credential/` entirely (or ensure it never existed).
- **When:** `clp .accounts`
- **Then:** `(no accounts configured)` with exit 0. No error about missing directory.; absent store handled gracefully, same as empty store
- **Exit:** 0
- **Source:** [commands.md — .accounts](../../../../docs/cli/commands.md#command--3-accounts)

---

### IT-12: Alphabetical ordering

- **Given:** Create accounts: `zed@acme.com`, `alice@acme.com`, `mike@acme.com` in non-alphabetical creation order.
- **When:** `clp .accounts active::0 sub::0 tier::0 expires::0 email::0`
- **Then:** Three bare names in alphabetical order: `alice@acme.com`, `mike@acme.com`, `zed@acme.com`.; accounts sorted alphabetically regardless of creation order
- **Exit:** 0
- **Source:** [commands.md — .accounts](../../../../docs/cli/commands.md#command--3-accounts)

---

### IT-13: Blank line between blocks

- **Given:** Two accounts: `alice@acme.com` (active) and `alice@home.com`.
- **When:** `clp .accounts`
- **Then:** Stdout contains `\n\n` (blank-line separator between the two blocks).; blank-line separator present when multiple accounts with fields shown
- **Exit:** 0
- **Source:** [commands.md — .accounts](../../../../docs/cli/commands.md#command--3-accounts)

---

### IT-14: Non-active account uses its own stored expiry

- **Given:** `alice@acme.com` is active with FAR_FUTURE_MS (valid token). `alice@home.com` is non-active with PAST_MS (expired token).
- **When:** `clp .accounts name::alice@home.com`
- **Then:** Stdout contains `expired`; does NOT contain `in ` (which would indicate leaking the active account's valid expiry).; non-active account shows own stored expiry, never leaking active account's live state
- **Exit:** 0
- **Source:** [commands.md — .accounts](../../../../docs/cli/commands.md#command--3-accounts)

---

### IT-15: Missing `subscriptionType` → `Sub:     N/A`

- **Given:** Credential file contains `{"oauthAccount":{"rateLimitTier":"standard"},"expiresAt":9999999999000}` (no `subscriptionType`).
- **When:** `clp .accounts`
- **Then:** Stdout contains `Sub:     N/A`.; missing field shows `N/A` not blank
- **Exit:** 0
- **Source:** [commands.md — .accounts](../../../../docs/cli/commands.md#command--3-accounts)

---

### IT-16: Missing `rateLimitTier` → `Tier:    N/A`

- **Given:** Credential file contains `{"oauthAccount":{"subscriptionType":"pro"},"expiresAt":9999999999000}` (no `rateLimitTier`).
- **When:** `clp .accounts`
- **Then:** Stdout contains `Tier:    N/A`.; missing field shows `N/A` not blank
- **Exit:** 0
- **Source:** [commands.md — .accounts](../../../../docs/cli/commands.md#command--3-accounts)

---

### IT-17: `display_name::1` shows Display line from saved snapshot

- **Given:** `work@acme.com` with saved `.claude.json` containing `{"oauthAccount":{"displayName":"alice"}}`. Active account.
- **When:** `clp .accounts display_name::1`
- **Then:** Stdout contains `Display: alice`.; display name rendered from saved snapshot
- **Exit:** 0
- **Source:** [commands.md — .accounts](../../../../docs/cli/commands.md#command--3-accounts)

---

### IT-18: `role::1 billing::1 model::1` shows corresponding lines

- **Given:** `work@acme.com` with saved `.claude.json` containing `{"oauthAccount":{"organizationRole":"admin","billingType":"stripe_subscription"}}` and saved `.settings.json` containing `{"model":"sonnet"}`.
- **When:** `clp .accounts role::1 billing::1 model::1`
- **Then:** Stdout contains `Role:    admin`, `Billing: stripe_subscription`, `Model:   sonnet`.; all 3 metadata fields rendered
- **Exit:** 0
- **Source:** [commands.md — .accounts](../../../../docs/cli/commands.md#command--3-accounts)

---

### IT-19: Account without saved metadata → N/A for new fields

- **Given:** `work@acme.com` with credential file only (no `.claude.json` or `.settings.json` snapshots).
- **When:** `clp .accounts display_name::1 role::1 billing::1 model::1`
- **Then:** Stdout contains `Display: N/A`, `Role:    N/A`, `Billing: N/A`, `Model:   N/A`.; absent snapshots degrade gracefully
- **Exit:** 0
- **Source:** [commands.md — .accounts](../../../../docs/cli/commands.md#command--3-accounts)

---

### IT-20: JSON includes new metadata keys

- **Given:** `work@acme.com` with saved `.claude.json` snapshot containing display name and role.
- **When:** `clp .accounts format::json`
- **Then:** Valid JSON array where each object contains `display_name`, `role`, `billing`, `model` keys.; JSON shape includes all metadata regardless of snapshot presence
- **Exit:** 0
- **Source:** [commands.md — .accounts](../../../../docs/cli/commands.md#command--3-accounts)

---

### IT-21: New metadata fields absent by default

- **Given:** `work@acme.com` with saved `.claude.json` and `.settings.json` snapshots containing rich metadata.
- **When:** `clp .accounts`
- **Then:** Stdout does NOT contain `Display:`, `Role:`, `Billing:`, `Model:` lines.; opt-in fields absent by default
- **Exit:** 0
- **Source:** [commands.md — .accounts](../../../../docs/cli/commands.md#command--3-accounts)

---

### IT-22: `email::` shows value from saved snapshot

- **Given:** `work@acme.com` with saved `.claude.json` containing `{"emailAddress":"work@acme.com"}`.
- **When:** `clp .accounts`
- **Then:** Stdout contains `Email:   work@acme.com`.; email address populated from saved snapshot (default-on)
- **Exit:** 0
- **Source:** [commands.md — .accounts](../../../../docs/cli/commands.md#command--3-accounts)
