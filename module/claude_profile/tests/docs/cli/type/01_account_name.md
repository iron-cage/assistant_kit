# Test: Type 01 — AccountName

Boundary and validation test cases for the `AccountName` type. See
[docs/cli/type/001_account_name.md](../../../../docs/cli/type/001_account_name.md) for the type
specification.

`AccountName` wraps a validated email string used as a credential store key. It enforces
email format, non-empty local part, and path-safe characters (no `/`, `\`, or `*`).

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Valid canonical email accepted | Valid (normal) |
| TC-2 | Minimal valid email accepted | Valid (boundary min) |
| TC-3 | Empty string rejected | Invalid (empty) |
| TC-4 | Input with no `@` character rejected | Invalid (format) |
| TC-5 | Local part containing `/` rejected | Invalid (path-unsafe) |
| TC-6 | Empty local part (starts with `@`) rejected | Invalid (empty local part) |

**Total:** 6 TC cases

---

### TC-1: Valid canonical email accepted

- **Given:** The string `alice@acme.com`
- **When:** `AccountName::new("alice@acme.com")` is called
- **Then:** Returns `Ok(AccountName)` with `get()` returning `"alice@acme.com"`
- **Source:** [docs/cli/type/001_account_name.md](../../../../docs/cli/type/001_account_name.md)

---

### TC-2: Minimal valid email accepted

- **Given:** The minimal-length valid email string `a@b.c`
- **When:** `AccountName::new("a@b.c")` is called
- **Then:** Returns `Ok(AccountName)` — non-empty local part, `@` present, non-empty domain
- **Source:** [docs/cli/type/001_account_name.md](../../../../docs/cli/type/001_account_name.md)

---

### TC-3: Empty string rejected

- **Given:** The empty string `""`
- **When:** `AccountName::new("")` is called
- **Then:** Returns `Err(…)` — non-empty constraint violated
- **Source:** [docs/cli/type/001_account_name.md](../../../../docs/cli/type/001_account_name.md)

---

### TC-4: Input with no `@` character rejected

- **Given:** The string `aliceatacme` (no `@`)
- **When:** `AccountName::new("aliceatacme")` is called
- **Then:** Returns `Err(…)` — email format requires exactly one `@`
- **Source:** [docs/cli/type/001_account_name.md](../../../../docs/cli/type/001_account_name.md)

---

### TC-5: Local part containing `/` rejected

- **Given:** The string `alice/bob@acme.com` (forward slash in local part)
- **When:** `AccountName::new("alice/bob@acme.com")` is called
- **Then:** Returns `Err(…)` — `/` is path-unsafe and forbidden in the local part before any
  filesystem operation
- **Source:** [docs/cli/type/001_account_name.md](../../../../docs/cli/type/001_account_name.md)

---

### TC-6: Empty local part rejected

- **Given:** The string `@acme.com` (empty local part before `@`)
- **When:** `AccountName::new("@acme.com")` is called
- **Then:** Returns `Err(…)` — local part must be non-empty
- **Source:** [docs/cli/type/001_account_name.md](../../../../docs/cli/type/001_account_name.md)
