# Parameter :: `assignee::`

Edge case tests for the `assignee::` parameter (Feature 065). Renamed from `active::` (Feature 064). A `Kind::String` mutation param where the value is either a `USER@MACHINE` target identity or the sentinel `"0"` (= current machine, expands to `$USER@$HOSTNAME`). Controls active-account marker assign/unassign operations.

**Source:** [params.md#parameter--63-assignee](../../../../docs/cli/param/063_assignee.md)

**Behavioral Divergence Pair:** EC-1 ↔ EC-5 — `assignee::user@host name::X` with valid `USER@MACHINE` format writes the marker file and exits 0; `assignee::badvalue` without `@` exits 1 with a format error before any marker write.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `assignee::user@host name::X` writes `_active_host_user = X` | Behavioral |
| EC-2 | `assignee::0 name::X` expands to current machine, writes marker | Sentinel |
| EC-3 | `assignee::user@host` (no `name::`) clears `_active_host_user` | Behavioral |
| EC-4 | `assignee::0` (no `name::`) clears current machine marker | Sentinel |
| EC-5 | `assignee::badvalue` (no `@`, not `"0"`) exits 1 | Validation |
| EC-6 | `assignee::@host` (empty user component) exits 1 | Validation |
| EC-7 | `assignee::user@` (empty machine component) exits 1 | Validation |
| EC-8 | `assignee::user@host name::X dry::1` previews without writing | Dry-run |
| EC-9 | `assignee::0 name::X dry::1` sentinel dry-run preview | Dry-run |
| EC-10 | `assignee::0 dry::1` (no `name::`) sentinel unassign dry-run | Dry-run |
| EC-11 | `assignee::user@host name::unknown` exits 1 (account not in store) | Validation |
| EC-12 | Space in machine component sanitized to `_` | Sanitization |
| EC-13 | Dot and hyphen in machine component preserved | Sanitization |
| EC-14 | `assignee::` absent — no marker write (default omit) | Default |
| EC-15 | `assignee::user@host name::X` does NOT modify `owner` field | Isolation |
| EC-16 | `force::1 assignee::user@host name::X` — `force::1` silently ignored | No-op |
| EC-17 | `active::user@host name::X` exits 1 — REMOVED_TOGGLE migration message | Migration |
| EC-18 | `assignee::user@host` (no `name::`) when marker absent — no-op exit 0 | Behavioral |
| EC-19 | Multiple `@` in value — `assignee::alice@corp.com@laptop` splits on first `@` | Sanitization |

## Test Coverage Summary

- Behavioral: 3 tests (EC-1, EC-3, EC-18)
- Sentinel: 2 tests (EC-2, EC-4)
- Validation: 4 tests (EC-5, EC-6, EC-7, EC-11)
- Sanitization: 3 tests (EC-12, EC-13, EC-19)
- Dry-run: 3 tests (EC-8, EC-9, EC-10)
- Default: 1 test (EC-14)
- Isolation: 1 test (EC-15)
- No-op: 1 test (EC-16)
- Migration: 1 test (EC-17)
- Multiple-at: 1 test (EC-19)

**Total:** 19 edge cases

## Test Cases

---

### EC-1: `assignee::user1@w003 name::X` writes `_active_w003_user1 = X`

- **Given:** `alice@corp.com.credentials.json` exists in credential store. No existing `_active_w003_user1` marker.
- **When:** `clp .accounts assignee::user1@w003 name::alice@corp.com`
- **Then:** Exits 0. `{credential_store}/_active_w003_user1` contains `alice@corp.com`. No other files modified (credentials, `{name}.json`, `~/.claude.json` all unchanged).
- **Exit:** 0
- **Source:** [params.md#parameter--63-assignee](../../../../docs/cli/param/063_assignee.md)

---

### EC-2: `assignee::0 name::X` expands to current machine, writes marker

- **Given:** `alice@corp.com.credentials.json` exists in credential store. No existing `_active_{hostname}_{user}`.
- **When:** `clp .accounts assignee::0 name::alice@corp.com`
- **Then:** Exits 0. `{credential_store}/_active_{hostname}_{user}` (where `{hostname}` = `$HOSTNAME`, `{user}` = `$USER`) contains `alice@corp.com`. stdout contains the expanded identity (not the literal `"0"`). No credential files modified.
- **Exit:** 0
- **Source:** [params.md#parameter--63-assignee](../../../../docs/cli/param/063_assignee.md)

---

### EC-3: `assignee::user1@w003` (no `name::`) clears `_active_w003_user1`

- **Given:** `{credential_store}/_active_w003_user1` exists containing `alice@corp.com`.
- **When:** `clp .accounts assignee::user1@w003` (no `name::`)
- **Then:** Exits 0. `_active_w003_user1` is cleared or deleted. No credential files modified.
- **Exit:** 0
- **Source:** [params.md#parameter--63-assignee](../../../../docs/cli/param/063_assignee.md)

---

### EC-4: `assignee::0` (no `name::`) clears current machine marker

- **Given:** `{credential_store}/_active_{hostname}_{user}` exists containing `alice@corp.com`.
- **When:** `clp .accounts assignee::0` (no `name::`)
- **Then:** Exits 0. `_active_{hostname}_{user}` is cleared or deleted. stdout contains the expanded identity (not the literal `"0"`). No credential files modified.
- **Exit:** 0
- **Source:** [params.md#parameter--63-assignee](../../../../docs/cli/param/063_assignee.md)

---

### EC-5: `assignee::badvalue` (no `@`, not `"0"`) exits 1

- **Given:** Clean environment.
- **When:** `clp .accounts assignee::badvalue name::alice@corp.com`
- **Then:** Exits 1. Error message explains `USER@MACHINE` format (must include `@`) or use `assignee::0` for current machine. No `_active_*` file written.
- **Exit:** 1
- **Source:** [params.md#parameter--63-assignee](../../../../docs/cli/param/063_assignee.md)

---

### EC-6: `assignee::@w003` (empty user component) exits 1

- **Given:** Clean environment.
- **When:** `clp .accounts assignee::@w003 name::alice@corp.com`
- **Then:** Exits 1. Error about empty user component. No `_active_*` file written.
- **Exit:** 1
- **Source:** [params.md#parameter--63-assignee](../../../../docs/cli/param/063_assignee.md)

---

### EC-7: `assignee::user1@` (empty machine component) exits 1

- **Given:** Clean environment.
- **When:** `clp .accounts assignee::user1@ name::alice@corp.com`
- **Then:** Exits 1. Error about empty machine component. No `_active_*` file written.
- **Exit:** 1
- **Source:** [params.md#parameter--63-assignee](../../../../docs/cli/param/063_assignee.md)

---

### EC-8: `assignee::user1@w003 name::X dry::1` previews without writing

- **Given:** `alice@corp.com.credentials.json` exists. No existing `_active_w003_user1`.
- **When:** `clp .accounts assignee::user1@w003 name::alice@corp.com dry::1`
- **Then:** Exits 0. stdout contains `[dry-run]` preview message. No `_active_*` file created or modified.
- **Exit:** 0
- **Source:** [params.md#parameter--63-assignee](../../../../docs/cli/param/063_assignee.md)

---

### EC-9: `assignee::0 name::X dry::1` sentinel dry-run preview

- **Given:** `alice@corp.com.credentials.json` exists. No existing `_active_{hostname}_{user}`.
- **When:** `clp .accounts assignee::0 name::alice@corp.com dry::1`
- **Then:** Exits 0. stdout contains `[dry-run]` preview message showing expanded identity `{user}@{hostname}`. No `_active_*` file created or modified.
- **Exit:** 0
- **Source:** [params.md#parameter--63-assignee](../../../../docs/cli/param/063_assignee.md)

---

### EC-10: `assignee::0 dry::1` (no `name::`) sentinel unassign dry-run

- **Given:** `{credential_store}/_active_{hostname}_{user}` exists containing `alice@corp.com`.
- **When:** `clp .accounts assignee::0 dry::1` (no `name::`)
- **Then:** Exits 0. stdout contains `[dry-run] would unassign {user}@{hostname}  →  _active_{hostname}_{user} cleared`. `_active_{hostname}_{user}` is NOT modified or deleted.
- **Exit:** 0
- **Source:** [params.md#parameter--63-assignee](../../../../docs/cli/param/063_assignee.md)

---

### EC-11: `assignee::user1@w003 name::unknown` exits 1 — account not found

- **Given:** Credential store does NOT contain `ghost@example.com`.
- **When:** `clp .accounts assignee::user1@w003 name::ghost@example.com`
- **Then:** Exits 1. Error indicates account not found. No `_active_*` file created.
- **Exit:** 1
- **Source:** [params.md#parameter--63-assignee](../../../../docs/cli/param/063_assignee.md)

---

### EC-12: Space in machine component sanitized to `_`

- **Given:** `alice@corp.com.credentials.json` exists in credential store.
- **When:** `clp .accounts assignee::"alice@my laptop" name::alice@corp.com`
- **Then:** Exits 0. `{credential_store}/_active_my_laptop_alice` contains `alice@corp.com`. Space in `my laptop` replaced with `_`.
- **Exit:** 0
- **Source:** [params.md#parameter--63-assignee](../../../../docs/cli/param/063_assignee.md)

---

### EC-13: Dot and hyphen in machine component preserved

- **Given:** `alice@corp.com.credentials.json` exists in credential store.
- **When:** `clp .accounts assignee::user1@w003.local name::alice@corp.com`
- **Then:** Exits 0. `{credential_store}/_active_w003.local_user1` contains `alice@corp.com`. Dot is kept verbatim.
- **Exit:** 0
- **Source:** [params.md#parameter--63-assignee](../../../../docs/cli/param/063_assignee.md)

---

### EC-14: `assignee::` absent — no marker write

- **Given:** `alice@corp.com.credentials.json` exists.
- **When:** `clp .accounts name::alice@corp.com` (no `assignee::`)
- **Then:** Exits 0. No `_active_*` marker file written. Normal `.accounts` listing output.
- **Exit:** 0
- **Source:** [params.md#parameter--63-assignee](../../../../docs/cli/param/063_assignee.md)

---

### EC-15: `assignee::user@host name::X` does NOT modify `owner` field

- **Given:** `alice@corp.com.json` exists with `"owner": "other@machine"`.
- **When:** `clp .accounts assignee::user1@w003 name::alice@corp.com`
- **Then:** Exits 0. `{credential_store}/_active_w003_user1` written. `alice@corp.com.json` still contains `"owner": "other@machine"` — unchanged. `assignee::` is marker-only.
- **Exit:** 0
- **Source:** [params.md#parameter--63-assignee](../../../../docs/cli/param/063_assignee.md)

---

### EC-16: `force::1 assignee::user@host name::X` — `force::1` silently ignored

- **Given:** `alice@corp.com.credentials.json` exists in credential store.
- **When:** `clp .accounts assignee::user1@w003 name::alice@corp.com force::1`
- **Then:** Exits 0. `{credential_store}/_active_w003_user1` written with `alice@corp.com`. `force::1` is silently ignored — `assignee::` has no ownership gate. Output identical to the same command without `force::1`.
- **Exit:** 0
- **Source:** [params.md#parameter--63-assignee](../../../../docs/cli/param/063_assignee.md)

---

### EC-17: `active::user@host name::X` exits 1 — REMOVED_TOGGLE migration message

- **Given:** `alice@corp.com.credentials.json` exists in credential store.
- **When:** `clp .accounts active::user1@w003 name::alice@corp.com`
- **Then:** Exits 1. stderr contains migration message referencing `assignee::`. No `_active_*` file written.
- **Exit:** 1
- **Note:** `active::` is a REMOVED_TOGGLE (Feature 065). The migration message directs users to `assignee::USER@MACHINE name::X` (or `assignee::0 name::X` for current machine).
- **Source:** [params.md#parameter--63-assignee](../../../../docs/cli/param/063_assignee.md)

---

### EC-18: `assignee::user@host` (no `name::`) when marker absent — no-op exit 0

- **Given:** Credential store exists but contains no `_active_testmachine_testuser` marker file.
- **When:** `clp .accounts assignee::testuser@testmachine` (no `name::`)
- **Then:** Exits 0. stdout contains `unassigned`. No new `_active_*` file is created — the absent marker stays absent.
- **Exit:** 0
- **Source:** [params.md#parameter--63-assignee](../../../../docs/cli/param/063_assignee.md)

---

### EC-19: Multiple `@` in value — splits on first `@`

- **Given:** `alice@corp.com.credentials.json` exists in credential store.
- **When:** `clp .accounts assignee::alice@corp.com@laptop name::alice@corp.com`
- **Then:** Exits 0. `split_once('@')` splits on the FIRST `@`; user = `"alice"`, machine = `"corp.com@laptop"`. After sanitization (`@` → `_`): marker = `_active_corp.com_laptop_alice`. `{credential_store}/_active_corp.com_laptop_alice` contains `alice@corp.com`.
- **Exit:** 0
- **Note:** Implemented as `ec8_multiple_at_splits_on_first` (historically numbered before this spec was aligned).
- **Source:** [params.md#parameter--63-assignee](../../../../docs/cli/param/063_assignee.md)
