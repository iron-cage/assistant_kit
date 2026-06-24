# Parameter :: `active::`

Edge case tests for the repurposed `active::` parameter (Feature 064). Previously a `bool` field-presence toggle; now a `Kind::String` mutation param where the value is a `USER@MACHINE` target identity for active-account marker assign/unassign operations.

**Source:** [params.md#parameter--13-active](../../../../docs/cli/param/013_active.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `active::user@host name::X` writes `_active_host_user = X` | Behavioral |
| EC-2 | `active::user@host` (no `name::`) clears `_active_host_user` | Behavioral |
| EC-3 | `active::badvalue` (no `@`) exits 1 | Validation |
| EC-4 | `active::@host` (empty user component) exits 1 | Validation |
| EC-5 | `active::user@` (empty machine component) exits 1 | Validation |
| EC-6 | Space in machine component sanitized to `_` | Sanitization |
| EC-7 | Dot and hyphen in machine component preserved | Sanitization |
| EC-8 | `active::user@host name::X dry::1` previews without writing | Dry-run |
| EC-9 | `active::user@host name::unknown` exits 1 (account not in store) | Validation |
| EC-10 | `active::` absent — no marker write (default omit) | Default |
| EC-11 | `active::user@host` does NOT modify `owner` field | Isolation |
| EC-12 | `active::0 name::X` exits 1 — `"0"` is not a valid `USER@MACHINE` | Validation |
| EC-13 | `force::1 active::user@host name::X` — `force::1` silently ignored; marker written | No-op |

## Test Coverage Summary

- Behavioral: 2 tests (EC-1, EC-2)
- Validation: 5 tests (EC-3, EC-4, EC-5, EC-9, EC-12)
- Sanitization: 2 tests (EC-6, EC-7)
- Dry-run: 1 test (EC-8)
- Default: 1 test (EC-10)
- Isolation: 1 test (EC-11)
- No-op: 1 test (EC-13)

**Total:** 13 edge cases

## Test Cases

---

### EC-1: `active::user1@w003 name::X` writes `_active_w003_user1 = X`

- **Given:** `alice@corp.com.credentials.json` exists in credential store. No existing `_active_w003_user1` marker.
- **When:** `clp .accounts active::user1@w003 name::alice@corp.com`
- **Then:** Exits 0. `{credential_store}/_active_w003_user1` contains `alice@corp.com`. No other files modified (credentials, `{name}.json`, `~/.claude.json` all unchanged).
- **Exit:** 0
- **Source:** [params.md#parameter--13-active](../../../../docs/cli/param/013_active.md)

---

### EC-2: `active::user1@w003` (no `name::`) clears `_active_w003_user1`

- **Given:** `{credential_store}/_active_w003_user1` exists containing `alice@corp.com`.
- **When:** `clp .accounts active::user1@w003` (no `name::`)
- **Then:** Exits 0. `_active_w003_user1` is cleared or deleted. No credential files modified.
- **Exit:** 0
- **Source:** [params.md#parameter--13-active](../../../../docs/cli/param/013_active.md)

---

### EC-3: `active::badvalue` (no `@`) exits 1

- **Given:** Clean environment.
- **When:** `clp .accounts active::badvalue name::alice@corp.com`
- **Then:** Exits 1. Error message explains `USER@MACHINE` format (must include `@`). No `_active_*` file written.
- **Exit:** 1
- **Source:** [params.md#parameter--13-active](../../../../docs/cli/param/013_active.md)

---

### EC-4: `active::@w003` (empty user component) exits 1

- **Given:** Clean environment.
- **When:** `clp .accounts active::@w003 name::alice@corp.com`
- **Then:** Exits 1. Error about empty user component. No `_active_*` file written.
- **Exit:** 1
- **Source:** [params.md#parameter--13-active](../../../../docs/cli/param/013_active.md)

---

### EC-5: `active::user1@` (empty machine component) exits 1

- **Given:** Clean environment.
- **When:** `clp .accounts active::user1@ name::alice@corp.com`
- **Then:** Exits 1. Error about empty machine component. No `_active_*` file written.
- **Exit:** 1
- **Source:** [params.md#parameter--13-active](../../../../docs/cli/param/013_active.md)

---

### EC-6: Space in machine component sanitized to `_`

- **Given:** `alice@corp.com.credentials.json` exists in credential store.
- **When:** `clp .accounts active::"alice@my laptop" name::alice@corp.com`
- **Then:** Exits 0. `{credential_store}/_active_my_laptop_alice` contains `alice@corp.com`. Space in `my laptop` replaced with `_`.
- **Exit:** 0
- **Source:** [params.md#parameter--13-active](../../../../docs/cli/param/013_active.md)

---

### EC-7: Dot and hyphen in machine component preserved

- **Given:** `alice@corp.com.credentials.json` exists in credential store.
- **When:** `clp .accounts active::user1@w003.local name::alice@corp.com`
- **Then:** Exits 0. `{credential_store}/_active_w003.local_user1` contains `alice@corp.com`. Dot is kept verbatim.
- **Exit:** 0
- **Source:** [params.md#parameter--13-active](../../../../docs/cli/param/013_active.md)

---

### EC-8: `active::user1@w003 name::X dry::1` previews without writing

- **Given:** `alice@corp.com.credentials.json` exists. No existing `_active_w003_user1`.
- **When:** `clp .accounts active::user1@w003 name::alice@corp.com dry::1`
- **Then:** Exits 0. stdout contains `[dry-run]` preview message. No `_active_*` file created or modified.
- **Exit:** 0
- **Source:** [params.md#parameter--13-active](../../../../docs/cli/param/013_active.md)

---

### EC-9: `active::user1@w003 name::unknown` exits 1 — account not found

- **Given:** Credential store does NOT contain `ghost@example.com`.
- **When:** `clp .accounts active::user1@w003 name::ghost@example.com`
- **Then:** Exits 1. Error indicates account not found. No `_active_*` file created.
- **Exit:** 1
- **Source:** [params.md#parameter--13-active](../../../../docs/cli/param/013_active.md)

---

### EC-10: `active::` absent — no marker write

- **Given:** `alice@corp.com.credentials.json` exists.
- **When:** `clp .accounts name::alice@corp.com` (no `active::`)
- **Then:** Exits 0. No `_active_*` marker file written. Normal `.accounts` listing output.
- **Exit:** 0
- **Source:** [params.md#parameter--13-active](../../../../docs/cli/param/013_active.md)

---

### EC-11: `active::user@host name::X` does NOT modify `owner` field

- **Given:** `alice@corp.com.json` exists with `"owner": "other@machine"`.
- **When:** `clp .accounts active::user1@w003 name::alice@corp.com`
- **Then:** Exits 0. `{credential_store}/_active_w003_user1` written. `alice@corp.com.json` still contains `"owner": "other@machine"` — unchanged. `active::` is marker-only.
- **Exit:** 0
- **Source:** [params.md#parameter--13-active](../../../../docs/cli/param/013_active.md)

---

### EC-12: `active::0 name::X` exits 1 — `"0"` rejected

- **Given:** `alice@corp.com.credentials.json` exists in credential store.
- **When:** `clp .accounts active::0 name::alice@corp.com`
- **Then:** Exits 1. `"0"` contains no `@` — rejected by `USER@MACHINE` format validation. Error message indicates invalid `active::` format. No `_active_*` file written.
- **Exit:** 1
- **Note:** `"0"` is a sentinel for `owner::0` (ownership release), NOT for `active::`. The error should direct the user to `owner::0` or `active::USER@MACHINE` (without `name::`) for unassign.
- **Source:** [params.md#parameter--13-active](../../../../docs/cli/param/013_active.md)

---

### EC-13: `force::1 active::user@host name::X` — `force::1` silently ignored

- **Given:** `alice@corp.com.credentials.json` exists in credential store.
- **When:** `clp .accounts active::user1@w003 name::alice@corp.com force::1`
- **Then:** Exits 0. `{credential_store}/_active_w003_user1` written with `alice@corp.com`. `force::1` is silently ignored — `active::` has no ownership gate, so `force::` has no effect. Output identical to the same command without `force::1`.
- **Exit:** 0
- **Source:** [params.md#parameter--13-active](../../../../docs/cli/param/013_active.md)
