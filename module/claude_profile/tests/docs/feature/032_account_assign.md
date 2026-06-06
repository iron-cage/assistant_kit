# Test: Feature 032 — Account Marker Assignment

Feature behavioral requirement test cases for `docs/feature/032_account_assign.md`. Each FT case maps to one or more acceptance criteria.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | Current machine marker written when `for::` omitted | AC-01 |
| FT-02 | Remote machine marker written via `for::bob@laptop` | AC-02 |
| FT-03 | Dry-run prints would-assign line, writes nothing | AC-03 |
| FT-04 | No `name::` emits live usage block; exits 0 | AC-04 |
| FT-05 | Unknown account exits 2 | AC-05 |
| FT-06 | `for::` without `@` exits 1 | AC-06 |
| FT-07 | Empty `for::` component exits 1 | AC-07 |
| FT-08 | Special chars in `for::` sanitized to `_` | AC-08 |
| FT-09 | Prefix resolution assigns correct account | AC-09 |
| FT-10 | Overwriting existing marker writes new content | AC-10 |
| FT-11 | `~/.claude/.credentials.json` untouched after assign | AC-11 |
| FT-12 | Dry-run output contains marker filename | AC-12 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | Assign with no `for::` writes `_active_{hostname}_{user}` for current machine | AC-01 | Marker Write |
| FT-02 | `for::bob@laptop` writes `_active_laptop_bob` = account name | AC-02 | Remote Marker |
| FT-03 | `dry::1` prints `[dry-run] would assign` line; marker file absent | AC-03 | Dry-Run |
| FT-04 | No `name::` argument emits live usage block with machine + active account info | AC-04 | Usage Block |
| FT-05 | Account not in credential store exits 2 with actionable error | AC-05 | Error Handling |
| FT-06 | `for::noslash` (no `@`) exits 1 with format error | AC-06 | Validation |
| FT-07 | `for::@laptop` (empty user) exits 1; `for::bob@` (empty machine) exits 1 | AC-07 | Validation |
| FT-08 | `for::alice@my laptop` sanitizes space → `_active_my_laptop_alice` | AC-08 | Sanitization |
| FT-09 | `name::alice` prefix resolves to `alice@corp.com` and writes marker | AC-09 | Prefix Resolution |
| FT-10 | Second assign to same `for::` overwrites marker content | AC-10 | Idempotency |
| FT-11 | `~/.claude/.credentials.json` unchanged after `.account.assign` | AC-11 | No Side Effects |
| FT-12 | `dry::1` output includes `_active_laptop_bob` when `for::bob@laptop` | AC-12 | Dry-Run Detail |

### Test Cases

#### FT-01 — Current machine marker written when `for::` omitted

**Stimulus:** `clp .account.assign name::alice@corp.com` (no `for::`)
**Setup:** `alice@corp.com.credentials.json` exists in credential store
**Expected:** `_active_{hostname}_{user}` file in credential store contains `alice@corp.com`; exit 0; stdout contains `Assigned alice@corp.com`
**Source fn:** `aa01_current_machine_marker_written` (in `tests/cli/account_assign_test.rs`)

#### FT-02 — `for::bob@laptop` writes `_active_laptop_bob`

**Stimulus:** `clp .account.assign name::alice@corp.com for::bob@laptop`
**Setup:** `alice@corp.com.credentials.json` exists; `~/.claude/.credentials.json` has different content
**Expected:** `_active_laptop_bob` in credential store = `alice@corp.com`; `~/.claude/.credentials.json` unchanged; exit 0
**Source fn:** `aa02_remote_machine_marker_written` (in `tests/cli/account_assign_test.rs`)

#### FT-03 — Dry-run prints line; writes nothing

**Stimulus:** `clp .account.assign name::alice@corp.com dry::1`
**Setup:** `alice@corp.com.credentials.json` exists; no `_active_*` files present
**Expected:** stdout = `[dry-run] would assign alice@corp.com for …`; no `_active_*` file created; exit 0
**Source fn:** `aa03_dry_run_no_write` (in `tests/cli/account_assign_test.rs`)

#### FT-04 — No `name::` emits live usage block

**Stimulus:** `clp .account.assign`
**Setup:** `alice@corp.com.credentials.json` exists; `_active_testmachine_testuser` pre-seeded = `alice@corp.com`
**Expected:** stdout contains preamble description, `Current machine:`, `Active account:` showing `alice@corp.com`, and `Ready to copy:` section with 3 example lines containing `alice@corp.com`; exit 0
**Source fn:** `aa04_no_name_emits_usage_block` (in `tests/cli/account_assign_test.rs`)

#### FT-05 — Unknown account exits 2

**Stimulus:** `clp .account.assign name::ghost@example.com`
**Setup:** credential store exists but contains no entry for `ghost@example.com`
**Expected:** exit 2; stderr contains actionable error message
**Source fn:** `aa05_unknown_account_exits_2` (in `tests/cli/account_assign_test.rs`)

#### FT-06 — `for::` without `@` exits 1

**Stimulus:** `clp .account.assign name::alice@corp.com for::badvalue`
**Setup:** `alice@corp.com.credentials.json` exists
**Expected:** exit 1; stderr contains `USER@MACHINE` format error
**Source fn:** `aa06_for_without_at_exits_1` (in `tests/cli/account_assign_test.rs`)

#### FT-07 — Empty `for::` component exits 1

**Stimulus:** `clp .account.assign name::alice@corp.com for::@laptop` and `for::bob@`
**Setup:** `alice@corp.com.credentials.json` exists
**Expected:** both exit 1; no marker file written
**Source fn:** `aa07_empty_for_component_exits_1` (in `tests/cli/account_assign_test.rs`)

#### FT-08 — Special chars sanitized

**Stimulus:** `clp .account.assign name::alice@corp.com for::alice@my laptop`
**Setup:** `alice@corp.com.credentials.json` exists
**Expected:** `_active_my_laptop_alice` written = `alice@corp.com`; exit 0
**Source fn:** `aa08_special_chars_sanitized` (in `tests/cli/account_assign_test.rs`)

#### FT-09 — Prefix resolves to full name

**Stimulus:** `clp .account.assign name::alice for::bob@laptop`
**Setup:** only `alice@corp.com.credentials.json` in store (unique prefix)
**Expected:** `_active_laptop_bob` = `alice@corp.com`; exit 0
**Source fn:** `aa09_prefix_resolution` (in `tests/cli/account_assign_test.rs`)

#### FT-10 — Overwrite existing marker

**Stimulus:** `clp .account.assign name::new@corp.com for::bob@laptop`
**Setup:** `_active_laptop_bob` already contains `old@corp.com`; both accounts in store
**Expected:** `_active_laptop_bob` = `new@corp.com`; exit 0
**Source fn:** `aa10_overwrite_existing_marker` (in `tests/cli/account_assign_test.rs`)

#### FT-11 — `~/.claude/.credentials.json` untouched

**Stimulus:** `clp .account.assign name::alice@corp.com for::bob@laptop`
**Setup:** record mtime of `~/.claude/.credentials.json` before command
**Expected:** mtime unchanged after command; exit 0
**Source fn:** `aa11_no_credentials_json_side_effect` (in `tests/cli/account_assign_test.rs`)

#### FT-12 — Dry-run output contains marker filename

**Stimulus:** `clp .account.assign name::alice@corp.com for::bob@laptop dry::1`
**Setup:** `alice@corp.com.credentials.json` exists
**Expected:** stdout contains `_active_laptop_bob`; exit 0
**Source fn:** `aa12_dry_run_shows_marker_filename` (in `tests/cli/account_assign_test.rs`)
