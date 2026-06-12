# Test: Feature 011 — Named Account Scoping

Feature behavioral requirement test cases for `docs/feature/011_account_status_by_name.md` (FR-16). Each FT case maps to one or more acceptance criteria.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | `name::` scopes output to one block; exit 0 | AC-01 |
| FT-02 | Unknown `name::` → exit 2 with not-found error | AC-02 |
| FT-03 | Invalid `name::` characters → exit 1 | AC-03 |
| FT-04 | Without `name::` lists all accounts (backward compatible) | AC-04 |
| FT-05 | Absent or empty `subscriptionType`/`rateLimitTier` → shown as `N/A` | AC-05 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | `name::alice@acme.com` → single block, exit 0 | AC-01 | Named Scoping |
| FT-02 | `name::ghost@example.com` → exit 2 not-found | AC-02 | Error Handling |
| FT-03 | `name::notanemail` → exit 1 validation error | AC-03 | Error Handling |
| FT-04 | No `name::` → all accounts listed | AC-04 | Default Behavior |
| FT-05 | Empty subscription/tier fields → `N/A` displayed | AC-05 | N/A Normalization |

**Total:** 5 FT cases

---

### FT-01: `name::alice@acme.com` → single block, exit 0

- **Given:** `alice@acme.com` exists in the credential store.
- **When:** `clp .accounts name::alice@acme.com`
- **Then:** Output shows exactly one indented key-val block for `alice@acme.com`. Exit 0.
- **Exit:** 0
- **Source fn:** `acc04_name_scopes_to_single_block`
- **Source:** [011_account_status_by_name.md AC-01](../../../docs/feature/011_account_status_by_name.md)

---

### FT-02: `name::ghost@example.com` → exit 2 not-found

- **Given:** No account named `ghost@example.com` exists in the store.
- **When:** `clp .accounts name::ghost@example.com`
- **Then:** Exits 2 with a not-found error message.
- **Exit:** 2
- **Source fn:** `acc05_name_not_found_exits_2`
- **Source:** [011_account_status_by_name.md AC-02](../../../docs/feature/011_account_status_by_name.md)

---

### FT-03: `name::notanemail` → exit 1 validation error

- **Given:** The `name::` value contains path-unsafe characters (e.g. `notanemail` with slash, or passes email validation failure).
- **When:** `clp .accounts name::a/b@c.com`
- **Then:** Exits 1 with a validation error.
- **Exit:** 1
- **Source fn:** `acc06_name_invalid_exits_1`
- **Source:** [011_account_status_by_name.md AC-03](../../../docs/feature/011_account_status_by_name.md)

---

### FT-04: No `name::` → all accounts listed

- **Given:** Multiple accounts in the store.
- **When:** `clp .accounts` (no `name::` parameter)
- **Then:** All accounts are listed in alphabetical order. Backward-compatible with the full listing behavior.
- **Exit:** 0
- **Source fn:** `acc01_lists_accounts_as_indented_blocks`
- **Source:** [011_account_status_by_name.md AC-04](../../../docs/feature/011_account_status_by_name.md)

---

### FT-05: Empty subscription/tier fields → `N/A` displayed

- **Given:** An account whose credentials file has empty-string `subscriptionType` and absent `rateLimitTier`.
- **When:** `clp .accounts name::that_account@example.com`
- **Then:** `Sub:` and `Tier:` lines show `N/A`, never blank.
- **Exit:** 0
- **Source fn:** `acc15_missing_sub_field_shows_na`, `acc16_missing_tier_field_shows_na`
- **Source:** [011_account_status_by_name.md AC-05](../../../docs/feature/011_account_status_by_name.md)
