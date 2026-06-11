# Test: `for::` Parameter (Account Assign Target Identity)

Edge case coverage for the `for::` `USER@MACHINE` composite target identity parameter on
`.account.assign`. See [param/053_for.md](../../../../docs/cli/param/053_for.md) for specification.

The parameter splits on the **first** `@`: left → user component, right → machine component.
Each component is sanitized independently (alphanumeric, `-`, `.` kept; else `_`).

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `for::bob@laptop` writes `_active_laptop_bob` | Behavioral Divergence |
| EC-2 | `for::` omitted — current machine default used | Behavioral Divergence |
| EC-3 | `for::badvalue` (no `@`) exits 1 | Validation |
| EC-4 | `for::@laptop` (empty user component) exits 1 | Validation |
| EC-5 | `for::bob@` (empty machine component) exits 1 | Validation |
| EC-6 | Space in machine component sanitized to `_` | Sanitization |
| EC-7 | Dot and hyphen in machine component preserved | Sanitization |
| EC-8 | Multiple `@` in value — split on first only | Split Semantics |

---

### EC-1: `for::bob@laptop` writes `_active_laptop_bob`

- **Given:** `alice@corp.com.credentials.json` exists in credential store.
- **When:** `clp .account.assign name::alice@corp.com for::bob@laptop`
- **Then:** Exits 0. `{credential_store}/_active_laptop_bob` contains `alice@corp.com`.
- **Exit:** 0
- **Source fn:** `aa02_remote_machine_marker_written`
- **Source:** [param/053_for.md](../../../../docs/cli/param/053_for.md)

---

### EC-2: `for::` omitted — current machine default used

- **Given:** `alice@corp.com.credentials.json` exists in credential store.
- **When:** `clp .account.assign name::alice@corp.com` (no `for::`)
- **Then:** Exits 0. `{credential_store}/_active_{hostname}_{user}` (as returned by `active_marker_filename()`) contains `alice@corp.com`. No other `_active_*` file is created.
- **Exit:** 0
- **Source fn:** `aa01_current_machine_marker_written`
- **Source:** [param/053_for.md](../../../../docs/cli/param/053_for.md)

---

### EC-3: `for::badvalue` (no `@`) exits 1

- **Given:** `alice@corp.com.credentials.json` exists in credential store.
- **When:** `clp .account.assign name::alice@corp.com for::badvalue`
- **Then:** Exits 1. Stderr contains an error message explaining `USER@MACHINE` format (must include `@`). No `_active_*` file is written.
- **Exit:** 1
- **Source fn:** `aa06_for_without_at_exits_1`
- **Source:** [param/053_for.md](../../../../docs/cli/param/053_for.md)

---

### EC-4: `for::@laptop` (empty user component) exits 1

- **Given:** `alice@corp.com.credentials.json` exists in credential store.
- **When:** `clp .account.assign name::alice@corp.com for::@laptop`
- **Then:** Exits 1. Stderr contains an error about empty user component. No `_active_*` file is written.
- **Exit:** 1
- **Source fn:** `aa07_empty_for_component_exits_1`
- **Source:** [param/053_for.md](../../../../docs/cli/param/053_for.md)

---

### EC-5: `for::bob@` (empty machine component) exits 1

- **Given:** `alice@corp.com.credentials.json` exists in credential store.
- **When:** `clp .account.assign name::alice@corp.com for::bob@`
- **Then:** Exits 1. Stderr contains an error about empty machine component. No `_active_*` file is written.
- **Exit:** 1
- **Source fn:** `aa07_empty_for_component_exits_1`
- **Source:** [param/053_for.md](../../../../docs/cli/param/053_for.md)

---

### EC-6: Space in machine component sanitized to `_`

- **Given:** `alice@corp.com.credentials.json` exists in credential store.
- **When:** `clp .account.assign name::alice@corp.com for::alice@my laptop`
- **Then:** Exits 0. `{credential_store}/_active_my_laptop_alice` contains `alice@corp.com`. The space in `my laptop` is replaced with `_` during sanitization.
- **Exit:** 0
- **Source fn:** `aa08_special_chars_sanitized`
- **Source:** [param/053_for.md](../../../../docs/cli/param/053_for.md)

---

### EC-7: Dot and hyphen in machine component preserved

- **Given:** `alice@corp.com.credentials.json` exists in credential store.
- **When:** `clp .account.assign name::alice@corp.com for::user1@w003.local`
- **Then:** Exits 0. `{credential_store}/_active_w003.local_user1` contains `alice@corp.com`. Dot is kept verbatim in the machine component (`.` is in the allowed set).
- **Exit:** 0
- **Source fn:** `ec7_dot_hyphen_in_machine_preserved`
- **Source:** [param/053_for.md](../../../../docs/cli/param/053_for.md)

---

### EC-8: Multiple `@` in value — split on first only

- **Given:** `alice@corp.com.credentials.json` exists in credential store.
- **When:** `clp .account.assign name::alice@corp.com for::alice@corp.com@laptop`
- **Then:** Exits 0. Split on first `@`: user component = `alice`, machine component = `corp.com@laptop` (sanitized to `corp.com_laptop`). Written filename = `_active_corp.com_laptop_alice`.
- **Exit:** 0
- **Note:** This is the result of splitting on the **first** `@`. If the operator intends to target machine `laptop` with username `alice@corp.com`, they would need to use a different separator convention; the current spec does not support email-format usernames via `for::`.
- **Source fn:** `ec8_multiple_at_splits_on_first`
- **Source:** [param/053_for.md](../../../../docs/cli/param/053_for.md)
