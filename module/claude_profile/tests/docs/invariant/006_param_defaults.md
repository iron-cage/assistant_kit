# Test: Invariant 006 — Parameters Default to Active Context

Property assertion cases for `docs/invariant/006_param_defaults.md`. Verifies that every CLI
parameter defaults to an ambient context value when omitted, and that only the two documented
exceptions require an explicit argument.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | Account-scoped commands work without name:: when active account is set | Invariant holds (normal) |
| IN-2 | Only .account.use and .account.delete require explicit name:: | Invariant holds (boundary) |

**Total:** 2 IN cases

---

### IN-1: Account-scoped commands work without name:: when active account is set

- **Given:** A credential store with at least one saved account; the active account marker
  (`_active_{hostname}_{user}` file) points to a valid saved account
- **When:** Any account-scoped command other than `.account.use` or `.account.delete` is invoked
  without a `name::` argument (e.g., `clp .token.status`, `clp .account.limits`)
- **Then:** The command resolves to the active account and executes successfully; no "name
  required" error is produced — the ambient active account serves as the default
- **Source:** [docs/invariant/006_param_defaults.md](../../../docs/invariant/006_param_defaults.md)

---

### IN-2: Only .account.use and .account.delete require explicit name::

- **Given:** The `src/commands/` implementation tree at the current HEAD
- **When:** `grep -rn "require_nonempty_string_arg" src/` is run
- **Then:** The only call sites are within the `.account.use` and `.account.delete` command
  handlers; no other command handler uses `require_nonempty_string_arg` for the `name` argument
- **Source:** [docs/invariant/006_param_defaults.md](../../../docs/invariant/006_param_defaults.md)
