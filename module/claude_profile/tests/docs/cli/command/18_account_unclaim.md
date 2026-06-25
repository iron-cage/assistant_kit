# Command :: 18. `.account.unclaim` — Redirect Stub

`.account.unclaim` is registered as a **redirect stub** by Feature 037 (shipped).
It exits 1 with a targeted migration hint pointing to `owner::0 name::X`.
All ownership-release behavior is now in `.accounts owner::0 name::X` (Feature 064 — `unclaim::1` is REMOVED) — see `03_accounts.md` IT-44 through IT-45.

| # | Test | Conditions | Exit |
|---|------|-----------|------|
| IT-1 | Redirect stub — targeted `owner::0` migration hint | `clp .account.unclaim name::alice@acme.com`; stderr contains `"owner::0"` migration hint; does NOT produce a generic "unknown command" error | 1 |

**Source:** [feature/037_accounts_usage_param_unification.md AC-11](../../../../docs/feature/037_accounts_usage_param_unification.md),
[cli/command_verb/011_unclaim.md — Migration (Feature 037)](../../../../docs/cli/command_verb/011_unclaim.md#migration-feature-037)
