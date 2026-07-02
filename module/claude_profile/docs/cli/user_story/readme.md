# User Stories

Five canonical user stories covering the major personas and goals of the `clp` CLI.

| File | Responsibility |
|------|----------------|
| [001_account_rotation.md](001_account_rotation.md) | Automatic account rotation when token expires |
| [002_onboarding.md](002_onboarding.md) | Onboarding a new account to the credential store |
| [003_quota_monitoring.md](003_quota_monitoring.md) | Multi-account quota visibility and rotation planning |
| [004_scripted_automation.md](004_scripted_automation.md) | Structured output for pipeline and script integration |
| [005_credential_diagnostics.md](005_credential_diagnostics.md) | Live credential inspection for authentication troubleshooting |

**Total:** 5 user stories

### User Story Index Table

| # | Story | Persona | Referenced Commands |
|---|-------|---------|---------------------|
| 1 | [Account Rotation](001_account_rotation.md) | SWE managing multiple Max accounts | `.usage rotate::1`, `.account.use`, `.accounts` |
| 2 | [Account Onboarding](002_onboarding.md) | Developer adding or recovering an account | `.account.save`, `.accounts` (incl. `assignee::USER@MACHINE` marker path), `.account.delete`, `.account.relogin`, `.account.renewal` |
| 3 | [Quota Monitoring](003_quota_monitoring.md) | Power user maximizing available quota | `.usage`, `.account.limits` |
| 4 | [Scripted Automation](004_scripted_automation.md) | DevOps engineer in CI/CD pipelines | `.usage`, `.accounts`, `.token.status`, `.usage rotate::1` |
| 5 | [Credential Diagnostics](005_credential_diagnostics.md) | Developer troubleshooting auth failures | `.credentials.status`, `.token.status`, `.paths`, `.account.inspect` |

### See Also

- [../command/](../command/readme.md) — commands referenced by user stories
- [../param/](../param/readme.md) — parameters referenced by user stories
- [../format/](../format/readme.md) — output formats referenced by user stories
