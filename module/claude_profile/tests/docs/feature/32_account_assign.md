# Test: Feature 032 — Account Marker Assignment

Feature behavioral requirement test cases for `docs/feature/032_account_assign.md`. Each FT case maps to one or more acceptance criteria.

> **Feature 064 migration:** The `assign::1 name::X for::USER@MACHINE` interface is REMOVED. These FTs use the current `active::USER@MACHINE name::X` interface. The `for::` param is absorbed into the `active::` value.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | Current machine marker written (identity in `active::` value) | AC-01 |
| FT-02 | Remote machine marker written via `active::bob@laptop name::X` | AC-02 |
| FT-03 | Dry-run prints would-assign line, writes nothing | AC-03 |
| FT-04 | No `name::` unassigns marker for that identity; exits 0 | AC-10 |
| FT-05 | Unknown account exits 1 or 2 | AC-05 |
| FT-06 | Bad `active::` value without `@` exits 1 | AC-06 |
| FT-07 | Empty component in `active::` value exits 1 | AC-07 |
| FT-08 | Special chars in `active::` value sanitized to `_` | AC-08 |
| FT-09 | Prefix resolution assigns correct account | AC-09 |
| FT-10 | Overwriting existing marker writes new content | AC-10 |
| FT-11 | `~/.claude/.credentials.json` untouched after `active::` assign | AC-11 |
| FT-12 | Dry-run output contains marker filename | AC-12 |
| FT-13 | `.accounts active::USER@MACHINE` does NOT modify `owner` field in `{name}.json` (marker-only) | AC-13 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | `active::$USER@$HOSTNAME name::X` writes `_active_{hostname}_{user}` for current machine | AC-01 | Marker Write |
| FT-02 | `active::bob@laptop name::X` writes `_active_laptop_bob` = account name | AC-02 | Remote Marker |
| FT-03 | `dry::1` prints `[dry-run] would assign` line; marker file absent | AC-03 | Dry-Run |
| FT-04 | `active::USER@MACHINE` (no `name::`) clears marker for that identity; exits 0 | AC-10 | Unassign |
| FT-05 | Account not in credential store exits 1 or 2 with actionable error | AC-05 | Error Handling |
| FT-06 | `active::noslash` (no `@`) exits 1 with format error | AC-06 | Validation |
| FT-07 | `active::@laptop` (empty user) exits 1; `active::bob@` (empty machine) exits 1 | AC-07 | Validation |
| FT-08 | `active::alice@my laptop` sanitizes space → `_active_my_laptop_alice` | AC-08 | Sanitization |
| FT-09 | `name::alice` prefix resolves to `alice@corp.com` and writes marker | AC-09 | Prefix Resolution |
| FT-10 | Second assign to same identity overwrites marker content | AC-10 | Idempotency |
| FT-11 | `~/.claude/.credentials.json` unchanged after `.accounts active::` assign | AC-11 | No Side Effects |
| FT-12 | `dry::1` output includes `_active_laptop_bob` when `active::bob@laptop` | AC-12 | Dry-Run Detail |
| FT-13 | `.accounts active::USER@MACHINE name::X` does NOT modify `owner` field in `{name}.json` (marker-only write) | AC-13 | Marker-Only |

### Test Cases

### FT-01: Current machine marker written via `active::$USER@$HOSTNAME name::X` (Feature 064)

**Stimulus:** `clp .accounts active::$USER@$HOSTNAME name::alice@corp.com`
**Setup:** `alice@corp.com.credentials.json` exists in credential store
**Expected:** `_active_{hostname}_{user}` file in credential store contains `alice@corp.com`; exit 0; stdout contains `Assigned alice@corp.com`
**Source fn:** `aa01_current_machine_marker_written` (in `tests/cli/account_assign_test.rs`)

### FT-02: `active::bob@laptop name::X` writes `_active_laptop_bob` (Feature 064)

**Stimulus:** `clp .accounts active::bob@laptop name::alice@corp.com` (formerly `assign::1 name::alice@corp.com for::bob@laptop`)
**Setup:** `alice@corp.com.credentials.json` exists; `~/.claude/.credentials.json` has different content
**Expected:** `_active_laptop_bob` in credential store = `alice@corp.com`; `~/.claude/.credentials.json` unchanged; exit 0
**Source fn:** `aa02_remote_machine_marker_written` (in `tests/cli/account_assign_test.rs`)

### FT-03: Dry-run prints line; writes nothing

**Stimulus:** `clp .accounts active::$USER@$HOSTNAME name::alice@corp.com dry::1`
**Setup:** `alice@corp.com.credentials.json` exists; no `_active_*` files present
**Expected:** stdout = `[dry-run] would assign alice@corp.com for …`; no `_active_*` file created; exit 0
**Source fn:** `aa03_dry_run_no_write` (in `tests/cli/account_assign_test.rs`)

### FT-04: `active::USER@MACHINE` (no `name::`) clears marker for that identity (Feature 064)

**Stimulus:** `clp .accounts active::testuser@testmachine` (no `name::`)
**Setup:** `_active_testmachine_testuser` pre-seeded = `alice@corp.com` in credential store
**Expected:** `_active_testmachine_testuser` cleared/removed from credential store; no credential files modified; exit 0. (Formerly: no-name emitted live usage block — Feature 064 changed this to unassign.)
**Source fn:** `aa04_no_name_emits_usage_block` (in `tests/cli/account_assign_test.rs`)

### FT-05: Unknown account exits 1 or 2 (Feature 064)

**Stimulus:** `clp .accounts active::$USER@$HOSTNAME name::ghost@example.com`
**Setup:** credential store exists but contains no entry for `ghost@example.com`
**Expected:** exit 1 or 2; stderr contains actionable error message
**Source fn:** `aa05_unknown_account_exits_2` (in `tests/cli/account_assign_test.rs`)

### FT-06: Bad `active::` value without `@` exits 1 (Feature 064)

**Stimulus:** `clp .accounts active::badvalue name::alice@corp.com` (no `@` in `active::` value; formerly `assign::1 name::X for::badvalue`)
**Setup:** `alice@corp.com.credentials.json` exists
**Expected:** exit 1; stderr contains `USER@MACHINE` format error
**Source fn:** `aa06_for_without_at_exits_1` (in `tests/cli/account_assign_test.rs`)

### FT-07: Empty component in `active::` value exits 1 (Feature 064)

**Stimulus:** `clp .accounts active::@laptop name::alice@corp.com` and `clp .accounts active::bob@ name::alice@corp.com`
**Setup:** `alice@corp.com.credentials.json` exists
**Expected:** both exit 1; no marker file written
**Source fn:** `aa07_empty_for_component_exits_1` (in `tests/cli/account_assign_test.rs`)

### FT-08: Special chars in `active::` value sanitized to `_` (Feature 064)

**Stimulus:** `clp .accounts "active::alice@my laptop" name::alice@corp.com` (space in machine component)
**Setup:** `alice@corp.com.credentials.json` exists
**Expected:** `_active_my_laptop_alice` written = `alice@corp.com`; exit 0
**Source fn:** `aa08_special_chars_sanitized` (in `tests/cli/account_assign_test.rs`)

### FT-09: Prefix resolves to full name (Feature 064)

**Stimulus:** `clp .accounts active::bob@laptop name::alice` (prefix `alice` resolves to `alice@corp.com`)
**Setup:** only `alice@corp.com.credentials.json` in store (unique prefix)
**Expected:** `_active_laptop_bob` = `alice@corp.com`; exit 0
**Source fn:** `aa09_prefix_resolution` (in `tests/cli/account_assign_test.rs`)

### FT-10: Overwrite existing marker (Feature 064)

**Stimulus:** `clp .accounts active::bob@laptop name::new@corp.com`
**Setup:** `_active_laptop_bob` already contains `old@corp.com`; both accounts in store
**Expected:** `_active_laptop_bob` = `new@corp.com`; exit 0
**Source fn:** `aa10_overwrite_existing_marker` (in `tests/cli/account_assign_test.rs`)

### FT-11: `~/.claude/.credentials.json` untouched after `active::` assign (Feature 064)

**Stimulus:** `clp .accounts active::bob@laptop name::alice@corp.com`
**Setup:** record mtime of `~/.claude/.credentials.json` before command
**Expected:** mtime unchanged after command; exit 0
**Source fn:** `aa11_no_credentials_json_side_effect` (in `tests/cli/account_assign_test.rs`)

### FT-12: Dry-run output contains marker filename (Feature 064)

**Stimulus:** `clp .accounts active::bob@laptop name::alice@corp.com dry::1`
**Setup:** `alice@corp.com.credentials.json` exists
**Expected:** stdout contains `_active_laptop_bob`; exit 0
**Source fn:** `aa12_dry_run_shows_marker_filename` (in `tests/cli/account_assign_test.rs`)

### FT-13: `.accounts active::USER@MACHINE name::X` does NOT modify `owner` field in `{name}.json` (Feature 064)

**Stimulus:** `clp .accounts active::$USER@$HOSTNAME name::alice@corp.com`
**Setup:** `alice@corp.com.credentials.json` exists; `alice@corp.com.json` has `"owner": "some@machine"`
**Expected:** `alice@corp.com.json` retains `"owner": "some@machine"` unchanged; only `_active_{machine}_{user}` marker is written; exit 0
**Source fn:** `ft13_assign_does_not_modify_owner` (in `tests/cli/account_assign_test.rs`)


