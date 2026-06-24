# Command :: 16. `.account.assign` — Fully Deregistered

`.account.assign` has been **fully deregistered** from the command registry by Feature 037 (shipped).
It produces a generic "unknown command" error — the same error as any unregistered command.
All marker-write behavior is now in `.accounts active::USER@MACHINE name::X` (Feature 064 — `assign::1` is REMOVED) — see `03_accounts.md` IT-43 through IT-46.

| # | Test | Conditions | Exit |
|---|------|-----------|------|
| IT-1 | Fully deregistered — generic "unknown command" error | `clp .account.assign name::alice@acme.com`; stdout or stderr contains generic "unknown command" error; does NOT contain migration message `"use '.accounts assign::1'"` | 1 |

**Source:** [feature/037_accounts_usage_param_unification.md AC-12](../../../../docs/feature/037_accounts_usage_param_unification.md),
[cli/command_verb/009_assign.md — Migration (Feature 037)](../../../../docs/cli/command_verb/009_assign.md#migration-feature-037)
