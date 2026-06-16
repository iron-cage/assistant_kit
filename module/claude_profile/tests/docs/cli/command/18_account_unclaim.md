# Command :: 18. `.account.unclaim` — Redirect Stub Tests

`.account.unclaim` was removed as a standalone working command by Feature 037. It is now a redirect
stub that exits 1 with a migration error message. All ownership-release behavior moved to
`.accounts unclaim::1` — see `03_accounts.md` IT-44 through IT-45 and `09_usage.md` for those test cases.

| # | Test | Conditions | Exit |
|---|------|-----------|------|
| IT-1 | Redirect stub exits 1 with migration message | `clp .account.unclaim name::alice@acme.com`; stdout or stderr contains `unknown command '.account.unclaim' — use '.accounts unclaim::1 name::X' instead` | 1 |

**Source:** [feature/037_accounts_usage_param_unification.md AC-11](../../../../docs/feature/037_accounts_usage_param_unification.md),
[cli/command_verb/011_unclaim.md — Migration (Feature 037)](../../../../docs/cli/command_verb/011_unclaim.md#migration-feature-037)
