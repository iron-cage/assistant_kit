# Feature 064: Active Marker and Owner Param Redesign — Test Cases

### Scope

- **Purpose**: Test cases for the `active::`/`owner::0` param redesign, including REMOVED_TOGGLE migration messages for `assign::`/`unclaim::`.
- **Source**: `docs/feature/064_active_marker_and_owner_redesign.md`
- **Covers**: AC-01 through AC-19 (FT-01..04, FT-13, FT-14, FT-18, FT-19 partially superseded by Feature 065 — see 065_assignee_param_redesign.md)

**Source:** [feature/064_active_marker_and_owner_redesign.md](../../../docs/feature/064_active_marker_and_owner_redesign.md)

> **Partially superseded (Feature 065):** FTs referencing `active::USER@MACHINE` (FT-01..04, FT-13, FT-14, FT-18, FT-19) now apply via `assignee::USER@MACHINE` in the live codebase. The `owner::0` FTs (FT-08..12, FT-15..17) remain current. See [065_assignee_param_redesign.md](065_assignee_param_redesign.md) for replacement test cases.

## Test Case Index

| ID | Test Name | AC | Category | Status |
|----|-----------|----|-----------|----|
| FT-01 | `active::USER@MACHINE name::X` writes marker | AC-01 | Behavioral | ✅ |
| FT-02 | `active::USER@MACHINE` (no `name::`) unassigns marker | AC-02 | Behavioral | ✅ |
| FT-03 | `active::USER@MACHINE name::X dry::1` previews without writing | AC-03 | Dry-run | ✅ |
| FT-04 | `active::USER@MACHINE name::ghost` — unknown account exits 1 | AC-04 | Validation | ✅ |
| FT-05 | `assign::1 name::X` exits 1 with REMOVED_TOGGLE migration message | AC-05 | Migration | ✅ |
| FT-06 | `assign::1 name::X for::bob@laptop` exits 1 (both REMOVED) | AC-06 | Migration | ✅ |
| FT-07 | `unclaim::1 name::X` exits 1 with REMOVED_TOGGLE migration message | AC-07 | Migration | ✅ |
| FT-08 | `owner::0 name::X` clears ownership via `write_owner(name, store, "")` | AC-08 | Behavioral | ✅ |
| FT-09 | `owner::0` alone — batch-clears all owned accounts in filter | AC-09 | Batch | ✅ |
| FT-10 | `owner::0 name::X,Y,Z` — batch-clear comma-list | AC-10 | Batch | ✅ |
| FT-11 | `owner::USER@MACHINE name::X,Y,Z` — batch-set comma-list | AC-11 | Batch | ✅ |
| FT-12 | `owner::0 name::X force::1` bypasses G8 | AC-12 | Gate Bypass | ✅ |
| FT-13 | `active::` value sanitization — space→`_`; dot/hyphen preserved | AC-13 | Sanitization | ✅ |
| FT-14 | `active::USER@MACHINE name::X` does NOT modify `owner` field | AC-14 | Isolation | ✅ |
| FT-15 | `owner::` with empty value exits 1 (empty ≠ sentinel "0") | AC-15 | Validation | ✅ |
| FT-16 | `owner::0 name::X dry::1` prints `[dry-run]`; no file written | AC-16 | Dry-run | ✅ |
| FT-17 | `owner::0 name::X force::1 dry::1` bypasses G8 + dry-run | AC-17 | Dry-run + Gate | ✅ |
| FT-18 | `active::0 name::X` exits 1 — value `"0"` rejected (no `@`) | AC-18 | Validation | ✅ |
| FT-19 | `active::USER@MACHINE dry::1` (no `name::`) unassign dry-run preview | AC-19 | Dry-run | ✅ |

**Total:** 19 test cases

## Test Cases

---

### FT-01: `active::USER@MACHINE name::X` writes marker

- **Given:** `alice@corp.com.credentials.json` exists in credential store. No existing `_active_w003_user1`.
- **When:** `clp .accounts active::user1@w003 name::alice@corp.com`
- **Then:** Exits 0. `{credential_store}/_active_w003_user1` contains `alice@corp.com`. stdout contains `assigned alice@corp.com for user1@w003  →  _active_w003_user1`. No credential files modified. `alice@corp.com.json` unchanged.
- **Exit:** 0
- **Maps to:** AC-01
- **Source:** [feature/064_active_marker_and_owner_redesign.md](../../../docs/feature/064_active_marker_and_owner_redesign.md)

---

### FT-02: `active::USER@MACHINE` (no `name::`) unassigns marker

- **Given:** `{credential_store}/_active_w003_user1` exists containing `alice@corp.com`.
- **When:** `clp .accounts active::user1@w003` (no `name::`)
- **Then:** Exits 0. `_active_w003_user1` is cleared or deleted. stdout contains `unassigned user1@w003  →  _active_w003_user1 cleared`. No credential files modified.
- **Exit:** 0
- **Maps to:** AC-02
- **Source:** [feature/064_active_marker_and_owner_redesign.md](../../../docs/feature/064_active_marker_and_owner_redesign.md)

---

### FT-03: `active::USER@MACHINE name::X dry::1` previews without writing

- **Given:** `alice@corp.com.credentials.json` exists. No existing `_active_w003_user1`.
- **When:** `clp .accounts active::user1@w003 name::alice@corp.com dry::1`
- **Then:** Exits 0. stdout contains `[dry-run] would assign alice@corp.com for user1@w003  →  _active_w003_user1`. No `_active_*` file created.
- **Exit:** 0
- **Maps to:** AC-03
- **Source:** [feature/064_active_marker_and_owner_redesign.md](../../../docs/feature/064_active_marker_and_owner_redesign.md)

---

### FT-04: Unknown account exits 1

- **Given:** Credential store does NOT contain `ghost@example.com`.
- **When:** `clp .accounts active::user1@w003 name::ghost@example.com`
- **Then:** Exits 1. Error indicates account not found. No `_active_*` file written.
- **Exit:** 1
- **Maps to:** AC-04
- **Source:** [feature/064_active_marker_and_owner_redesign.md](../../../docs/feature/064_active_marker_and_owner_redesign.md)

---

### FT-05: `assign::1` REMOVED_TOGGLE exits 1

- **Given:** Clean environment.
- **When:** `clp .accounts assign::1 name::alice@corp.com`
- **Then:** Exits 1. stderr contains migration message: "REMOVED — use `active::USER@MACHINE name::X` instead". No files modified.
- **Exit:** 1
- **Maps to:** AC-05
- **Source:** [feature/064_active_marker_and_owner_redesign.md](../../../docs/feature/064_active_marker_and_owner_redesign.md)

---

### FT-06: `assign::1 for::` both REMOVED exits 1

- **Given:** Clean environment.
- **When:** `clp .accounts assign::1 name::alice@corp.com for::bob@laptop`
- **Then:** Exits 1. REMOVED_TOGGLE fires for one or both params. No files modified.
- **Exit:** 1
- **Maps to:** AC-06
- **Source:** [feature/064_active_marker_and_owner_redesign.md](../../../docs/feature/064_active_marker_and_owner_redesign.md)

---

### FT-07: `unclaim::1` REMOVED_TOGGLE exits 1

- **Given:** Clean environment.
- **When:** `clp .accounts unclaim::1 name::alice@corp.com`
- **Then:** Exits 1. stderr contains migration message: "REMOVED — use `owner::0 name::X` instead (or `owner::0` alone to batch-clear)". No files modified.
- **Exit:** 1
- **Maps to:** AC-07
- **Source:** [feature/064_active_marker_and_owner_redesign.md](../../../docs/feature/064_active_marker_and_owner_redesign.md)

---

### FT-08: `owner::0 name::X` clears ownership

- **Given:** `alice@corp.com.json` exists with `"owner": "user1@w003"`. Caller identity is `user1@w003` (G8 passes).
- **When:** `clp .accounts owner::0 name::alice@corp.com`
- **Then:** Exits 0. `alice@corp.com.json` contains `"owner": ""` via `write_owner(name, store, "")`. stdout contains `unclaimed alice@corp.com`. Credentials file unchanged.
- **Exit:** 0
- **Maps to:** AC-08
- **Source:** [feature/064_active_marker_and_owner_redesign.md](../../../docs/feature/064_active_marker_and_owner_redesign.md)

---

### FT-09: `owner::0` alone batch-clears all owned accounts

- **Given:** Credential store has accounts A (owned by caller), B (unowned), C (owned by caller). Current filter returns all three.
- **When:** `clp .accounts owner::0` (no `name::`)
- **Then:** Exits 0. A and C have `owner: ""` written (G8 passes for each). B is skipped with a "skip" message (not owned). No credential files modified.
- **Exit:** 0
- **Maps to:** AC-09
- **Source:** [feature/064_active_marker_and_owner_redesign.md](../../../docs/feature/064_active_marker_and_owner_redesign.md)

---

### FT-10: `owner::0 name::X,Y,Z` batch-clear via comma-list

- **Given:** `alice@corp.com.json`, `bob@corp.com.json`, `charlie@corp.com.json` all exist with caller as owner.
- **When:** `clp .accounts owner::0 name::alice@corp.com,bob@corp.com,charlie@corp.com`
- **Then:** Exits 0. All three `.json` files contain `"owner": ""`. G8 evaluated independently per account.
- **Exit:** 0
- **Maps to:** AC-10
- **Source:** [feature/064_active_marker_and_owner_redesign.md](../../../docs/feature/064_active_marker_and_owner_redesign.md)

---

### FT-11: `owner::USER@MACHINE name::X,Y,Z` batch-set via comma-list

- **Given:** `alice@corp.com.json`, `bob@corp.com.json`, `charlie@corp.com.json` all exist. All unowned (G8 passes for each).
- **When:** `clp .accounts owner::user1@w003 name::alice@corp.com,bob@corp.com,charlie@corp.com`
- **Then:** Exits 0. All three `.json` files contain `"owner": "user1@w003"`. G8 evaluated independently per account. stdout contains `owned {name} by user1@w003` for each.
- **Exit:** 0
- **Maps to:** AC-11
- **Source:** [feature/064_active_marker_and_owner_redesign.md](../../../docs/feature/064_active_marker_and_owner_redesign.md)

---

### FT-12: `owner::0 name::X force::1` bypasses G8

- **Given:** `alice@corp.com.json` has `"owner": "other@remote"`. Caller identity is `user1@w003` (G8 would block without force).
- **When:** `clp .accounts owner::0 name::alice@corp.com force::1`
- **Then:** Exits 0. `alice@corp.com.json` contains `"owner": ""`. G8 bypassed by `force::1`. stdout contains `unclaimed alice@corp.com`.
- **Exit:** 0
- **Maps to:** AC-12
- **Source:** [feature/064_active_marker_and_owner_redesign.md](../../../docs/feature/064_active_marker_and_owner_redesign.md)

---

### FT-13: `active::` value sanitization

- **Given:** `alice@corp.com.credentials.json` exists.
- **When (a):** `clp .accounts active::"alice@my laptop" name::alice@corp.com`
- **Then (a):** Exits 0. `_active_my_laptop_alice` written. Space in `my laptop` → `_`.
- **When (b):** `clp .accounts active::user1@w003.local name::alice@corp.com`
- **Then (b):** Exits 0. `_active_w003.local_user1` written. Dot is preserved verbatim.
- **Exit:** 0
- **Maps to:** AC-13
- **Source:** [feature/064_active_marker_and_owner_redesign.md](../../../docs/feature/064_active_marker_and_owner_redesign.md)

---

### FT-14: `active::` does NOT modify `owner` field

- **Given:** `alice@corp.com.json` exists with `"owner": "other@machine"`. `alice@corp.com.credentials.json` exists.
- **When:** `clp .accounts active::user1@w003 name::alice@corp.com`
- **Then:** Exits 0. `{credential_store}/_active_w003_user1` written. `alice@corp.com.json` still contains `"owner": "other@machine"` — unchanged. Active marker write is ownership-neutral.
- **Exit:** 0
- **Maps to:** AC-14
- **Source:** [feature/064_active_marker_and_owner_redesign.md](../../../docs/feature/064_active_marker_and_owner_redesign.md)

---

### FT-15: `owner::` empty value exits 1

- **Given:** Clean environment.
- **When:** `clp .accounts owner:: name::alice@corp.com` (empty value, not sentinel "0")
- **Then:** Exits 1. Error message directs user to `owner::0` for ownership release. No files written. Empty string ≠ sentinel `"0"`.
- **Exit:** 1
- **Maps to:** AC-15
- **Source:** [feature/064_active_marker_and_owner_redesign.md](../../../docs/feature/064_active_marker_and_owner_redesign.md)

---

### FT-16: `owner::0 name::X dry::1` dry-run preview

- **Given:** `alice@corp.com.json` exists with `"owner": "user1@w003"`. Caller identity is `user1@w003` (G8 passes).
- **When:** `clp .accounts owner::0 name::alice@corp.com dry::1`
- **Then:** Exits 0. stdout contains `[dry-run] would clear owner of alice@corp.com`. `alice@corp.com.json` is NOT modified — still contains `"owner": "user1@w003"`. G8 still evaluated before dry-run check.
- **Exit:** 0
- **Maps to:** AC-16
- **Source:** [feature/064_active_marker_and_owner_redesign.md](../../../docs/feature/064_active_marker_and_owner_redesign.md)

---

### FT-17: `owner::0 name::X force::1 dry::1` bypass G8 and dry-run

- **Given:** `alice@corp.com.json` has `"owner": "other@remote"`. Caller identity is `user1@w003` (G8 would block without force).
- **When:** `clp .accounts owner::0 name::alice@corp.com force::1 dry::1`
- **Then:** Exits 0. stdout contains `[dry-run] would clear owner of alice@corp.com`. G8 bypassed by `force::1`. No files written — dry-run suppresses actual write.
- **Exit:** 0
- **Maps to:** AC-17
- **Source:** [feature/064_active_marker_and_owner_redesign.md](../../../docs/feature/064_active_marker_and_owner_redesign.md)

---

### FT-18: `active::0 name::X` exits 1 — value `"0"` rejected

- **Given:** `alice@corp.com.credentials.json` exists in credential store.
- **When:** `clp .accounts active::0 name::alice@corp.com`
- **Then:** Exits 1. `"0"` contains no `@` and is rejected by `USER@MACHINE` format validation. Error message indicates invalid format. No `_active_*` file written.
- **Exit:** 1
- **Maps to:** AC-18
- **Source:** [feature/064_active_marker_and_owner_redesign.md](../../../docs/feature/064_active_marker_and_owner_redesign.md)

---

### FT-19: `active::USER@MACHINE dry::1` (no `name::`) unassign dry-run preview

- **Given:** `{credential_store}/_active_w003_user1` exists containing `alice@corp.com`.
- **When:** `clp .accounts active::user1@w003 dry::1` (no `name::`)
- **Then:** Exits 0. stdout contains `[dry-run] would unassign user1@w003  →  _active_w003_user1 cleared`. `_active_w003_user1` is NOT modified or deleted.
- **Exit:** 0
- **Maps to:** AC-19
- **Source:** [feature/064_active_marker_and_owner_redesign.md](../../../docs/feature/064_active_marker_and_owner_redesign.md)
