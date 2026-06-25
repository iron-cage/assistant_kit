# Command :: 16. `.account.assign` — Redirect Stub

`.account.assign` is registered as a **redirect stub** by Feature 037 (shipped).
It exits 1 with a targeted migration hint pointing to `assignee::USER@MACHINE name::X`.
All marker-write behavior is now in `.accounts assignee::USER@MACHINE name::X` (or `assignee::0 name::X` for current machine) (Feature 065 — `assign::1` and `active::` are both REMOVED) — see `03_accounts.md` IT-43 through IT-46.

| # | Test | Conditions | Exit |
|---|------|-----------|------|
| IT-1 | Redirect stub — targeted `assignee::` migration hint | `clp .account.assign name::alice@acme.com`; stderr contains `"assignee::"` migration hint; does NOT produce a generic "unknown command" error | 1 |

**Source:** [feature/037_accounts_usage_param_unification.md AC-12](../../../../docs/feature/037_accounts_usage_param_unification.md),
[cli/command_verb/009_assign.md — Migration (Feature 037)](../../../../docs/cli/command_verb/009_assign.md#migration-feature-037)
