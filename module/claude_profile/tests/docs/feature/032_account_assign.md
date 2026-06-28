# Test: Feature 032 — Account Marker Assignment

Feature behavioral requirement test cases for `docs/feature/032_account_assign.md`. Each FT case maps to one or more acceptance criteria.

> **Feature 065 migration:** The `assign::1 name::X for::USER@MACHINE` interface is REMOVED (Feature 064). The `active::USER@MACHINE name::X` interface introduced by Feature 064 is also REMOVED (Feature 065). These FTs use the current `assignee::USER@MACHINE name::X` interface; `assignee::0` is the current-machine sentinel. `active::` now exits 1 with a REMOVED_TOGGLE migration message.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | Current machine marker written (identity in `assignee::` value) | AC-01 |
| FT-02 | Remote machine marker written via `assignee::bob@laptop name::X` | AC-02 |
| FT-03 | Dry-run prints would-assign line, writes nothing | AC-03 |
| FT-04 | No `name::` unassigns marker for that identity; exits 0 | AC-10 |
| FT-05 | Unknown account exits 1 or 2 | AC-05 |
| FT-06 | Bad `assignee::` value without `@` exits 1 | AC-06 |
| FT-07 | Empty component in `assignee::` value exits 1 | AC-07 |
| FT-08 | Special chars in `assignee::` value sanitized to `_` | AC-08 |
| FT-09 | Prefix resolution assigns correct account | AC-09 |
| FT-10 | Overwriting existing marker writes new content | AC-10 |
| FT-11 | `~/.claude/.credentials.json` untouched after `assignee::` assign | AC-11 |
| FT-12 | Dry-run output contains marker filename | AC-12 |
| FT-13 | `.accounts assignee::USER@MACHINE` does NOT modify `owner` field in `{name}.json` (marker-only) | AC-13 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | `assignee::0 name::X` writes `_active_{hostname}_{user}` for current machine (sentinel) | AC-01 | Marker Write |
| FT-02 | `assignee::bob@laptop name::X` writes `_active_laptop_bob` = account name | AC-02 | Remote Marker |
| FT-03 | `dry::1` prints `[dry-run] would assign` line; marker file absent | AC-03 | Dry-Run |
| FT-04 | `assignee::USER@MACHINE` (no `name::`) clears marker for that identity; exits 0 | AC-10 | Unassign |
| FT-05 | Account not in credential store exits 1 or 2 with actionable error | AC-05 | Error Handling |
| FT-06 | `assignee::noslash` (no `@`) exits 1 with format error | AC-06 | Validation |
| FT-07 | `assignee::@laptop` (empty user) exits 1; `assignee::bob@` (empty machine) exits 1 | AC-07 | Validation |
| FT-08 | `assignee::"alice@my laptop"` sanitizes space → `_active_my_laptop_alice` | AC-08 | Sanitization |
| FT-09 | `name::alice` prefix resolves to `alice@corp.com` and writes marker | AC-09 | Prefix Resolution |
| FT-10 | Second assign to same identity overwrites marker content | AC-10 | Idempotency |
| FT-11 | `~/.claude/.credentials.json` unchanged after `.accounts assignee::` assign | AC-11 | No Side Effects |
| FT-12 | `dry::1` output includes `_active_laptop_bob` when `assignee::bob@laptop` | AC-12 | Dry-Run Detail |
| FT-13 | `.accounts assignee::USER@MACHINE name::X` does NOT modify `owner` field in `{name}.json` (marker-only write) | AC-13 | Marker-Only |

### Test Cases

### FT-01: Current machine marker written via `assignee::0 name::X` (Feature 065)

**Stimulus:** `clp .accounts assignee::0 name::alice@corp.com`
**Setup:** `alice@corp.com.credentials.json` exists in credential store; `$USER`/`$HOSTNAME` set to test values
**Expected:** `_active_{hostname}_{user}` file in credential store contains `alice@corp.com`; exit 0; stdout contains `assigned alice@corp.com for {user}@{hostname}`
**Source fn:** `ec2_assignee_zero_sentinel_assign` (in `tests/cli/account_assign_test.rs`)

### FT-02: `assignee::bob@laptop name::X` writes `_active_laptop_bob` (Feature 065)

**Stimulus:** `clp .accounts assignee::bob@laptop name::alice@corp.com` (formerly `assign::1 name::alice@corp.com for::bob@laptop`)
**Setup:** `alice@corp.com.credentials.json` exists; `~/.claude/.credentials.json` has different content
**Expected:** `_active_laptop_bob` in credential store = `alice@corp.com`; `~/.claude/.credentials.json` unchanged; exit 0
**Source fn:** `ft01b_assignee_assign_writes_remote_marker` (in `tests/cli/account_assign_test.rs`)

### FT-03: Dry-run prints line; writes nothing

**Stimulus:** `clp .accounts assignee::0 name::alice@corp.com dry::1`
**Setup:** `alice@corp.com.credentials.json` exists; no `_active_*` files present; `$USER`/`$HOSTNAME` set to test values
**Expected:** stdout = `[dry-run] would assign alice@corp.com for {user}@{hostname}  →  _active_{hostname}_{user}`; no `_active_*` file created; exit 0
**Source fn:** `ft03_assignee_dry_run` (in `tests/cli/account_assign_test.rs`)

### FT-04: `assignee::USER@MACHINE` (no `name::`) clears marker for that identity (Feature 065)

**Stimulus:** `clp .accounts assignee::testuser@testmachine` (no `name::`)
**Setup:** `_active_testmachine_testuser` pre-seeded = `alice@corp.com` in credential store
**Expected:** `_active_testmachine_testuser` cleared/removed from credential store; no credential files modified; exit 0.
**Source fn:** `ft02_assignee_unassign_clears_marker` (in `tests/cli/account_assign_test.rs`)

### FT-05: Unknown account exits 1 (Feature 065)

**Stimulus:** `clp .accounts assignee::testuser@testmachine name::ghost@example.com`
**Setup:** credential store exists but contains no entry for `ghost@example.com`
**Expected:** exit 1; stderr contains actionable error message
**Source fn:** `ft04_assignee_unknown_account_exits_1` (in `tests/cli/account_assign_test.rs`)

### FT-06: Bad `assignee::` value without `@` exits 1 (Feature 065)

**Stimulus:** `clp .accounts assignee::badvalue name::alice@corp.com` (no `@` in value; formerly `for::badvalue`)
**Setup:** `alice@corp.com.credentials.json` exists
**Expected:** exit 1; stderr contains `USER@MACHINE` format error
**Source fn:** `ec5_assignee_badvalue_exits_1` (in `tests/cli/account_assign_test.rs`)

### FT-07: Empty component in `assignee::` value exits 1 (Feature 065)

**Stimulus:** `clp .accounts assignee::@laptop name::alice@corp.com` and `clp .accounts assignee::bob@ name::alice@corp.com`
**Setup:** `alice@corp.com.credentials.json` exists
**Expected:** both exit 1; no marker file written
**Source fn:** `ec6_assignee_empty_user_exits_1` / `ec7_assignee_empty_machine_exits_1` (in `tests/cli/account_assign_test.rs`)

### FT-08: Special chars in `assignee::` value sanitized to `_` (Feature 065)

**Stimulus:** `clp .accounts "assignee::alice@my laptop" name::alice@corp.com` (space in machine component)
**Setup:** `alice@corp.com.credentials.json` exists
**Expected:** `_active_my_laptop_alice` written = `alice@corp.com`; exit 0
**Source fn:** `ft12a_space_in_assignee_value_sanitized` (in `tests/cli/account_assign_test.rs`)

### FT-09: Prefix resolves to full name (Feature 065)

**Stimulus:** `clp .accounts assignee::bob@laptop name::alice` (prefix `alice` resolves to `alice@corp.com`)
**Setup:** only `alice@corp.com.credentials.json` in store (unique prefix)
**Expected:** `_active_laptop_bob` = `alice@corp.com`; exit 0
**Source fn:** `aa09_prefix_resolution` (in `tests/cli/account_assign_test.rs`)

### FT-10: Overwrite existing marker (Feature 065)

**Stimulus:** `clp .accounts assignee::bob@laptop name::new@corp.com`
**Setup:** `_active_laptop_bob` already contains `old@corp.com`; both accounts in store
**Expected:** `_active_laptop_bob` = `new@corp.com`; exit 0
**Source fn:** `ft01b_assignee_assign_writes_remote_marker` (in `tests/cli/account_assign_test.rs`)

### FT-11: `~/.claude/.credentials.json` untouched after `assignee::` assign (Feature 065)

**Stimulus:** `clp .accounts assignee::bob@laptop name::alice@corp.com`
**Setup:** record mtime of `~/.claude/.credentials.json` before command
**Expected:** mtime unchanged after command; exit 0
**Source fn:** `ft01b_assignee_assign_writes_remote_marker` (in `tests/cli/account_assign_test.rs`)

### FT-12: Dry-run output contains marker filename (Feature 065)

**Stimulus:** `clp .accounts assignee::bob@laptop name::alice@corp.com dry::1`
**Setup:** `alice@corp.com.credentials.json` exists
**Expected:** stdout contains `_active_laptop_bob`; exit 0
**Source fn:** `ft03_assignee_dry_run` (in `tests/cli/account_assign_test.rs`)

### FT-13: `.accounts assignee::USER@MACHINE name::X` does NOT modify `owner` field in `{name}.json` (Feature 065)

**Stimulus:** `clp .accounts assignee::testuser@testmachine name::alice@corp.com`
**Setup:** `alice@corp.com.credentials.json` exists; `alice@corp.com.json` has `"owner": "some@machine"`
**Expected:** `alice@corp.com.json` retains `"owner": "some@machine"` unchanged; only `_active_testmachine_testuser` marker is written; exit 0
**Source fn:** `ft11_assignee_does_not_modify_owner` (in `tests/cli/account_assign_test.rs`)


