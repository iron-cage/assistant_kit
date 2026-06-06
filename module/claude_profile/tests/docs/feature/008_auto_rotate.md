# Test: Feature 008 — Auto Rotate

Feature behavioral requirement test cases for `docs/feature/008_auto_rotate.md` (FR-13). Each FT case maps to one or more acceptance criteria.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | Returns inactive account with highest `expires_at_ms` | AC-01 |
| FT-02 | Returns `NotFound` when no inactive accounts exist | AC-02 |
| FT-03 | After rotation, `~/.claude/.credentials.json` holds selected account | AC-03 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | Selects inactive account with highest expiry | AC-01 | Selection |
| FT-02 | `NotFound` when all accounts are active or store is empty | AC-02 | Not Found |
| FT-03 | Credentials file updated to selected account after rotation | AC-03 | Side Effects |

**Total:** 3 FT cases

---

### FT-01: Selects inactive account with highest expiry

- **Given:** Two inactive accounts in the store: one with `expires_at_ms = 1000`, one with `expires_at_ms = 9000`. One active account.
- **When:** `account::auto_rotate(credential_store, paths)` is called.
- **Then:** Returns the name of the inactive account with `expires_at_ms = 9000`. The lower-expiry inactive account is not selected.
- **Exit:** Ok(name)
- **Source fn:** `auto_rotate_picks_account_with_highest_expires_at`, `auto_rotate_returns_switched_account_name`
- **Source:** [008_auto_rotate.md AC-01](../../../../docs/feature/008_auto_rotate.md)

---

### FT-02: `NotFound` when all accounts are active or store is empty

- **Given (case A):** Only one account in the store and it is the current active account.
- **Given (case B):** No accounts in the store at all.
- **When:** `account::auto_rotate(credential_store, paths)` is called.
- **Then:** Returns `NotFound` in both cases. No panic, no credentials mutation.
- **Exit:** Err(NotFound)
- **Source fn:** `auto_rotate_fails_when_no_inactive_accounts`, `auto_rotate_fails_when_account_store_empty`
- **Source:** [008_auto_rotate.md AC-02](../../../../docs/feature/008_auto_rotate.md)

---

### FT-03: Credentials file updated to selected account after rotation

- **Given:** Two accounts; one active, one inactive. The inactive account has higher expiry.
- **When:** `account::auto_rotate(credential_store, paths)` is called.
- **Then:** `~/.claude/.credentials.json` now contains the inactive account's credentials. The `_active_{hostname}_{user}` marker is updated to the selected account name.
- **Exit:** Ok(name)
- **Source fn:** `auto_rotate_switches_to_inactive_account`
- **Source:** [008_auto_rotate.md AC-03](../../../../docs/feature/008_auto_rotate.md)
