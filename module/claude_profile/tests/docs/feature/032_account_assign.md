# Test: Feature 032 — Account Marker Assignment

### Scope

- **Purpose**: Test cases for the current-machine active marker assignment and unassignment.
- **Source**: `docs/feature/032_account_assign.md`
- **Covers**: AC-01 through AC-13 (AC-04 covered by Feature 065 replacement FTs — see 065_assignee_param_redesign.md)

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

- **Given:** `alice@corp.com.credentials.json` exists in credential store; `$USER`/`$HOSTNAME` set to test values.
- **When:** `clp .accounts assignee::0 name::alice@corp.com`
- **Then:** `_active_{hostname}_{user}` file in credential store contains `alice@corp.com`; stdout contains `assigned alice@corp.com for {user}@{hostname}`.
- **Exit:** 0
- **Source fn:** `ec2_assignee_zero_sentinel_assign` (in `tests/cli/account_assign_test.rs`)

### FT-02: `assignee::bob@laptop name::X` writes `_active_laptop_bob` (Feature 065)

- **Given:** `alice@corp.com.credentials.json` exists; `~/.claude/.credentials.json` has different content.
- **When:** `clp .accounts assignee::bob@laptop name::alice@corp.com` (formerly `assign::1 name::alice@corp.com for::bob@laptop`)
- **Then:** `_active_laptop_bob` in credential store = `alice@corp.com`; `~/.claude/.credentials.json` unchanged.
- **Exit:** 0
- **Source fn:** `ft01b_assignee_assign_writes_remote_marker` (in `tests/cli/account_assign_test.rs`)

### FT-03: Dry-run prints line; writes nothing

- **Given:** `alice@corp.com.credentials.json` exists; no `_active_*` files present; `$USER`/`$HOSTNAME` set to test values.
- **When:** `clp .accounts assignee::0 name::alice@corp.com dry::1`
- **Then:** stdout = `[dry-run] would assign alice@corp.com for {user}@{hostname}  →  _active_{hostname}_{user}`; no `_active_*` file created.
- **Exit:** 0
- **Source fn:** `ft03_assignee_dry_run` (in `tests/cli/account_assign_test.rs`)

### FT-04: `assignee::USER@MACHINE` (no `name::`) clears marker for that identity (Feature 065)

- **Given:** `_active_testmachine_testuser` pre-seeded = `alice@corp.com` in credential store.
- **When:** `clp .accounts assignee::testuser@testmachine` (no `name::`)
- **Then:** `_active_testmachine_testuser` cleared/removed from credential store; no credential files modified.
- **Exit:** 0
- **Source fn:** `ft02_assignee_unassign_clears_marker` (in `tests/cli/account_assign_test.rs`)

### FT-05: Unknown account exits 1 (Feature 065)

- **Given:** credential store exists but contains no entry for `ghost@example.com`.
- **When:** `clp .accounts assignee::testuser@testmachine name::ghost@example.com`
- **Then:** stderr contains actionable error message.
- **Exit:** 1
- **Source fn:** `ft04_assignee_unknown_account_exits_1` (in `tests/cli/account_assign_test.rs`)

### FT-06: Bad `assignee::` value without `@` exits 1 (Feature 065)

- **Given:** `alice@corp.com.credentials.json` exists.
- **When:** `clp .accounts assignee::badvalue name::alice@corp.com` (no `@` in value; formerly `for::badvalue`)
- **Then:** stderr contains `USER@MACHINE` format error.
- **Exit:** 1
- **Source fn:** `ec5_assignee_badvalue_exits_1` (in `tests/cli/account_assign_test.rs`)

### FT-07: Empty component in `assignee::` value exits 1 (Feature 065)

- **Given:** `alice@corp.com.credentials.json` exists.
- **When:** `clp .accounts assignee::@laptop name::alice@corp.com` and `clp .accounts assignee::bob@ name::alice@corp.com`
- **Then:** both invocations exit 1; no marker file written.
- **Exit:** 1
- **Source fn:** `ec6_assignee_empty_user_exits_1` / `ec7_assignee_empty_machine_exits_1` (in `tests/cli/account_assign_test.rs`)

### FT-08: Special chars in `assignee::` value sanitized to `_` (Feature 065)

- **Given:** `alice@corp.com.credentials.json` exists.
- **When:** `clp .accounts "assignee::alice@my laptop" name::alice@corp.com` (space in machine component)
- **Then:** `_active_my_laptop_alice` written = `alice@corp.com`.
- **Exit:** 0
- **Source fn:** `ft12a_space_in_assignee_value_sanitized` (in `tests/cli/account_assign_test.rs`)

### FT-09: Prefix resolves to full name (Feature 065)

- **Given:** only `alice@corp.com.credentials.json` in store (unique prefix).
- **When:** `clp .accounts assignee::bob@laptop name::alice` (prefix `alice` resolves to `alice@corp.com`)
- **Then:** `_active_laptop_bob` = `alice@corp.com`.
- **Exit:** 0
- **Source fn:** `aa09_prefix_resolution` (in `tests/cli/account_assign_test.rs`)

### FT-10: Overwrite existing marker (Feature 065)

- **Given:** `_active_laptop_bob` already contains `old@corp.com`; both accounts in store.
- **When:** `clp .accounts assignee::bob@laptop name::new@corp.com`
- **Then:** `_active_laptop_bob` = `new@corp.com`.
- **Exit:** 0
- **Source fn:** `ft01b_assignee_assign_writes_remote_marker` (in `tests/cli/account_assign_test.rs`)

### FT-11: `~/.claude/.credentials.json` untouched after `assignee::` assign (Feature 065)

- **Given:** record mtime of `~/.claude/.credentials.json` before command.
- **When:** `clp .accounts assignee::bob@laptop name::alice@corp.com`
- **Then:** mtime unchanged after command.
- **Exit:** 0
- **Source fn:** `ft01b_assignee_assign_writes_remote_marker` (in `tests/cli/account_assign_test.rs`)

### FT-12: Dry-run output contains marker filename (Feature 065)

- **Given:** `alice@corp.com.credentials.json` exists.
- **When:** `clp .accounts assignee::bob@laptop name::alice@corp.com dry::1`
- **Then:** stdout contains `_active_laptop_bob`.
- **Exit:** 0
- **Source fn:** `ft03_assignee_dry_run` (in `tests/cli/account_assign_test.rs`)

### FT-13: `.accounts assignee::USER@MACHINE name::X` does NOT modify `owner` field in `{name}.json` (Feature 065)

- **Given:** `alice@corp.com.credentials.json` exists; `alice@corp.com.json` has `"owner": "some@machine"`.
- **When:** `clp .accounts assignee::testuser@testmachine name::alice@corp.com`
- **Then:** `alice@corp.com.json` retains `"owner": "some@machine"` unchanged; only `_active_testmachine_testuser` marker is written.
- **Exit:** 0
- **Source fn:** `ft11_assignee_does_not_modify_owner` (in `tests/cli/account_assign_test.rs`)


