# Test: Invariant 003 — Clear Error Messages

Property assertion cases for `docs/invariant/003_clear_errors.md`. Verifies that error messages
produced by `claude_profile` are actionable — they name the relevant resource and state what
went wrong rather than emitting generic failures.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | Missing account error includes the account name | Invariant holds (normal) |
| IN-2 | Missing credentials file error includes the expected file path | Invariant holds (boundary) |

**Total:** 2 IN cases

---

### IN-1: Missing account error includes the account name

- **Given:** The credential store exists but does not contain an account named `ghost@example.com`
- **When:** Any account-scoped operation (e.g., `account::load("ghost@example.com", …)`) is
  attempted for that account
- **Then:** The returned error message contains the account name `ghost@example.com`; it does not
  say a generic string such as "not found" or "operation failed" without naming the account
- **Source:** [docs/invariant/003_clear_errors.md](../../../docs/invariant/003_clear_errors.md)

---

### IN-2: Missing credentials file error includes the expected file path

- **Given:** The `~/.claude/.credentials.json` file does not exist on disk
- **When:** An operation that reads live credentials is attempted (e.g., `credentials::load(paths)`)
- **Then:** The returned error message contains the expected file path
  (`~/.claude/.credentials.json` or its resolved equivalent); it does not say "error reading file"
  without identifying which file
- **Source:** [docs/invariant/003_clear_errors.md](../../../docs/invariant/003_clear_errors.md)
