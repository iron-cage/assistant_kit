# Feature 065: Assignee Param Redesign — Test Cases

**Source:** [feature/065_assignee_param_redesign.md](../../../docs/feature/065_assignee_param_redesign.md)

## Test Case Index

| ID | Test Name | AC | Category | Status |
|----|-----------|----|-----------|--------|
| FT-01 | `assignee::USER@MACHINE name::X` writes marker | AC-01 | Behavioral | ✅ |
| FT-02 | `assignee::0 name::X` assigns for current machine | AC-02 | Sentinel | ✅ |
| FT-03 | `assignee::USER@MACHINE` (no `name::`) unassigns marker | AC-03 | Behavioral | ✅ |
| FT-04 | `assignee::0` (no `name::`) unassigns current machine marker | AC-04 | Sentinel | ✅ |
| FT-05 | `assignee::USER@MACHINE name::X dry::1` previews without writing | AC-05 | Dry-run | ✅ |
| FT-06 | `assignee::0 name::X dry::1` sentinel dry-run preview | AC-06 | Dry-run | ✅ |
| FT-07 | `assignee::0 dry::1` (no `name::`) unassign sentinel dry-run | AC-07 | Dry-run | ✅ |
| FT-08 | `assignee::USER@MACHINE name::ghost` — unknown account exits 1 | AC-08 | Validation | ✅ |
| FT-09 | `assignee::badvalue` (no `@`) exits 1 | AC-09 | Validation | ✅ |
| FT-10 | `active::USER@MACHINE name::X` exits 1 with REMOVED_TOGGLE message | AC-10 | Migration | ✅ |
| FT-11 | `assignee::USER@MACHINE name::X` does NOT modify `owner` field | AC-11 | Isolation | ✅ |
| FT-12 | `assignee::` value sanitization — space→`_`; dot/hyphen preserved | AC-12 | Sanitization | ✅ |
| FT-13 | `force::1 assignee::USER@MACHINE name::X` — `force::1` silently ignored | AC-13 | No-op | ✅ |

**Total:** 13 test cases

## Test Cases

---

### FT-01: `assignee::USER@MACHINE name::X` writes marker

- **Given:** `alice@corp.com.credentials.json` exists in credential store. No existing `_active_w003_user1`.
- **When:** `clp .accounts assignee::user1@w003 name::alice@corp.com`
- **Then:** Exits 0. `{credential_store}/_active_w003_user1` contains `alice@corp.com`. stdout contains `assigned alice@corp.com for user1@w003  →  _active_w003_user1`. No credential files modified. `alice@corp.com.json` unchanged.
- **Exit:** 0
- **Maps to:** AC-01
- **Source:** [feature/065_assignee_param_redesign.md](../../../docs/feature/065_assignee_param_redesign.md)

---

### FT-02: `assignee::0 name::X` assigns for current machine

- **Given:** `alice@corp.com.credentials.json` exists. No existing `_active_{hostname}_{user}`.
- **When:** `clp .accounts assignee::0 name::alice@corp.com`
- **Then:** Exits 0. `{credential_store}/_active_{hostname}_{user}` (where `{hostname}` = `$HOSTNAME`, `{user}` = `$USER`) contains `alice@corp.com`. stdout contains `assigned alice@corp.com for {user}@{hostname}  →  _active_{hostname}_{user}`. No credential files modified.
- **Exit:** 0
- **Maps to:** AC-02
- **Source:** [feature/065_assignee_param_redesign.md](../../../docs/feature/065_assignee_param_redesign.md)

---

### FT-03: `assignee::USER@MACHINE` (no `name::`) unassigns marker

- **Given:** `{credential_store}/_active_w003_user1` exists containing `alice@corp.com`.
- **When:** `clp .accounts assignee::user1@w003` (no `name::`)
- **Then:** Exits 0. `_active_w003_user1` is cleared or deleted. stdout contains `unassigned user1@w003  →  _active_w003_user1 cleared`. No credential files modified.
- **Exit:** 0
- **Maps to:** AC-03
- **Source:** [feature/065_assignee_param_redesign.md](../../../docs/feature/065_assignee_param_redesign.md)

---

### FT-04: `assignee::0` (no `name::`) unassigns current machine marker

- **Given:** `{credential_store}/_active_{hostname}_{user}` exists (where values match `$HOSTNAME` / `$USER`).
- **When:** `clp .accounts assignee::0` (no `name::`)
- **Then:** Exits 0. `_active_{hostname}_{user}` is cleared or deleted. stdout contains `unassigned {user}@{hostname}  →  _active_{hostname}_{user} cleared`. No credential files modified.
- **Exit:** 0
- **Maps to:** AC-04
- **Source:** [feature/065_assignee_param_redesign.md](../../../docs/feature/065_assignee_param_redesign.md)

---

### FT-05: `assignee::USER@MACHINE name::X dry::1` previews without writing

- **Given:** `alice@corp.com.credentials.json` exists. No existing `_active_w003_user1`.
- **When:** `clp .accounts assignee::user1@w003 name::alice@corp.com dry::1`
- **Then:** Exits 0. stdout contains `[dry-run] would assign alice@corp.com for user1@w003  →  _active_w003_user1`. No `_active_*` file created.
- **Exit:** 0
- **Maps to:** AC-05
- **Source:** [feature/065_assignee_param_redesign.md](../../../docs/feature/065_assignee_param_redesign.md)

---

### FT-06: `assignee::0 name::X dry::1` sentinel dry-run preview

- **Given:** `alice@corp.com.credentials.json` exists. No existing `_active_{hostname}_{user}`.
- **When:** `clp .accounts assignee::0 name::alice@corp.com dry::1`
- **Then:** Exits 0. stdout contains `[dry-run] would assign alice@corp.com for {user}@{hostname}  →  _active_{hostname}_{user}`. No `_active_*` file created.
- **Exit:** 0
- **Maps to:** AC-06
- **Source:** [feature/065_assignee_param_redesign.md](../../../docs/feature/065_assignee_param_redesign.md)

---

### FT-07: `assignee::0 dry::1` (no `name::`) unassign sentinel dry-run

- **Given:** `{credential_store}/_active_{hostname}_{user}` exists containing `alice@corp.com`.
- **When:** `clp .accounts assignee::0 dry::1` (no `name::`)
- **Then:** Exits 0. stdout contains `[dry-run] would unassign {user}@{hostname}  →  _active_{hostname}_{user} cleared`. `_active_{hostname}_{user}` is NOT modified or deleted.
- **Exit:** 0
- **Maps to:** AC-07
- **Source:** [feature/065_assignee_param_redesign.md](../../../docs/feature/065_assignee_param_redesign.md)

---

### FT-08: `assignee::USER@MACHINE name::ghost` — unknown account exits 1

- **Given:** Credential store does NOT contain `ghost@example.com`.
- **When:** `clp .accounts assignee::user1@w003 name::ghost@example.com`
- **Then:** Exits 1. Error indicates account not found. No `_active_*` file written.
- **Exit:** 1
- **Maps to:** AC-08
- **Source:** [feature/065_assignee_param_redesign.md](../../../docs/feature/065_assignee_param_redesign.md)

---

### FT-09: `assignee::badvalue` (no `@`) exits 1

- **Given:** Clean environment.
- **When:** `clp .accounts assignee::badvalue name::alice@corp.com`
- **Then:** Exits 1. Error message explains `USER@MACHINE` format requirement (must include `@`) or use `"0"` sentinel. No `_active_*` file written.
- **Exit:** 1
- **Maps to:** AC-09
- **Source:** [feature/065_assignee_param_redesign.md](../../../docs/feature/065_assignee_param_redesign.md)

---

### FT-10: `active::USER@MACHINE name::X` exits 1 with REMOVED_TOGGLE message

- **Given:** Clean environment.
- **When:** `clp .accounts active::user1@w003 name::alice@corp.com`
- **Then:** Exits 1. stderr contains migration message: "REMOVED — use `assignee::USER@MACHINE name::X` (or `assignee::0 name::X` for current machine)". No files modified.
- **Exit:** 1
- **Maps to:** AC-10
- **Source:** [feature/065_assignee_param_redesign.md](../../../docs/feature/065_assignee_param_redesign.md)

---

### FT-11: `assignee::` does NOT modify `owner` field

- **Given:** `alice@corp.com.json` exists with `"owner": "other@machine"`. `alice@corp.com.credentials.json` exists.
- **When:** `clp .accounts assignee::user1@w003 name::alice@corp.com`
- **Then:** Exits 0. `{credential_store}/_active_w003_user1` written. `alice@corp.com.json` still contains `"owner": "other@machine"` — unchanged. Assignee marker write is ownership-neutral.
- **Exit:** 0
- **Maps to:** AC-11
- **Source:** [feature/065_assignee_param_redesign.md](../../../docs/feature/065_assignee_param_redesign.md)

---

### FT-12: `assignee::` value sanitization

- **Given:** `alice@corp.com.credentials.json` exists.
- **When (a):** `clp .accounts assignee::"alice@my laptop" name::alice@corp.com`
- **Then (a):** Exits 0. `_active_my_laptop_alice` written. Space in `my laptop` → `_`.
- **When (b):** `clp .accounts assignee::user1@w003.local name::alice@corp.com`
- **Then (b):** Exits 0. `_active_w003.local_user1` written. Dot is preserved verbatim.
- **Exit:** 0
- **Maps to:** AC-12
- **Source:** [feature/065_assignee_param_redesign.md](../../../docs/feature/065_assignee_param_redesign.md)

---

### FT-13: `force::1 assignee::USER@MACHINE name::X` — `force::1` silently ignored

- **Given:** `alice@corp.com.credentials.json` exists in credential store.
- **When:** `clp .accounts assignee::user1@w003 name::alice@corp.com force::1`
- **Then:** Exits 0. `{credential_store}/_active_w003_user1` written with `alice@corp.com`. `force::1` is silently ignored — `assignee::` has no ownership gate. Output identical to the same command without `force::1`.
- **Exit:** 0
- **Maps to:** AC-13
- **Source:** [feature/065_assignee_param_redesign.md](../../../docs/feature/065_assignee_param_redesign.md)
