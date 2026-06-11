# Test: Type 04 — AccountSelector

Boundary and resolution test cases for the `AccountSelector` type. See
[docs/cli/type/004_account_selector.md](../../../../docs/cli/type/004_account_selector.md) for
the type specification.

`AccountSelector` is a documentation concept for the adapter layer's resolution logic. It accepts
three input forms — full email, local-part prefix, and positional bare arg — and resolves each to
an `AccountName`. Ambiguous or non-matching inputs are rejected with descriptive errors.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Full email form resolves directly to AccountName | Valid (full email) |
| TC-2 | Exact local-part prefix resolves to the one matching account | Valid (prefix) |
| TC-3 | Ambiguous prefix matching multiple accounts is rejected | Invalid (ambiguous) |
| TC-4 | Non-matching prefix is rejected with "not found" error | Invalid (no match) |

**Total:** 4 TC cases

---

### TC-1: Full email form resolves directly to AccountName

- **Given:** The input `"alice@acme.com"` (contains `@`); the credential store contains an
  account with that exact email
- **When:** The adapter layer resolves the selector
- **Then:** Returns `AccountName("alice@acme.com")` directly via `AccountName::new(input)` —
  prefix scan is skipped because the input contains `@`
- **Source:** [docs/cli/type/004_account_selector.md](../../../../docs/cli/type/004_account_selector.md)

---

### TC-2: Exact local-part prefix resolves to the one matching account

- **Given:** The input `"alice"` (no `@`); the credential store contains exactly one account
  whose email local part is `alice` (e.g., `alice@acme.com`); no other saved account starts
  with `alice`
- **When:** The adapter layer resolves the selector
- **Then:** Returns `AccountName("alice@acme.com")` — exact local-part match found; no ambiguity
- **Source:** [docs/cli/type/004_account_selector.md](../../../../docs/cli/type/004_account_selector.md)

---

### TC-3: Ambiguous prefix matching multiple accounts is rejected

- **Given:** The input `"al"` (no `@`); the credential store contains at least two accounts
  whose emails start with `al` (e.g., `alice@acme.com` and `alice@home.com`)
- **When:** The adapter layer resolves the selector
- **Then:** Returns an error containing "ambiguous prefix" and listing the matching accounts;
  exits with code 1 at the CLI layer
- **Source:** [docs/cli/type/004_account_selector.md](../../../../docs/cli/type/004_account_selector.md)

---

### TC-4: Non-matching prefix rejected with not-found error

- **Given:** The input `"xyz"` (no `@`); no saved account in the credential store starts
  with `xyz`
- **When:** The adapter layer resolves the selector
- **Then:** Returns an error containing the input `xyz` and "not found"; exits with code 2 at
  the CLI layer
- **Source:** [docs/cli/type/004_account_selector.md](../../../../docs/cli/type/004_account_selector.md)
