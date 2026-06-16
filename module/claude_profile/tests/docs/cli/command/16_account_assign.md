# Command :: 16. `.account.assign` — Redirect Stub Tests

`.account.assign` was removed as a standalone working command by Feature 037. It is now a redirect
stub that exits 1 with a migration error message. All marker-write behavior moved to
`.accounts assign::1` — see `03_accounts.md` IT-43 through IT-46 for those test cases.

| # | Test | Conditions | Exit |
|---|------|-----------|------|
| IT-1 | Redirect stub exits 1 with migration message | `clp .account.assign name::alice@acme.com`; stdout or stderr contains `unknown command '.account.assign' — use '.accounts assign::1 name::X' instead` | 1 |

**Source:** [feature/037_accounts_usage_param_unification.md AC-12](../../../../docs/feature/037_accounts_usage_param_unification.md),
[cli/command_verb/009_assign.md — Migration (Feature 037)](../../../../docs/cli/command_verb/009_assign.md#migration-feature-037)
