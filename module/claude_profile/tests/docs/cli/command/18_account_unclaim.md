# Command :: 18. `.account.unclaim` — Fully Deregistered

`.account.unclaim` has been **fully deregistered** from the command registry by Feature 037 (shipped).
It produces a generic "unknown command" error — the same error as any unregistered command.
All ownership-release behavior is in `.accounts unclaim::1` — see `03_accounts.md` IT-44 through IT-45.

| # | Test | Conditions | Exit |
|---|------|-----------|------|
| IT-1 | Fully deregistered — generic "unknown command" error | `clp .account.unclaim name::alice@acme.com`; stdout or stderr contains generic "unknown command" error; does NOT contain migration message `"use '.accounts unclaim::1'"` | 1 |

**Source:** [feature/037_accounts_usage_param_unification.md AC-11](../../../../docs/feature/037_accounts_usage_param_unification.md),
[cli/command_verb/011_unclaim.md — Migration (Feature 037)](../../../../docs/cli/command_verb/011_unclaim.md#migration-feature-037)
