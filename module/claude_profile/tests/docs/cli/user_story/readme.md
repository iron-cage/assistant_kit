# Test: user story acceptance

UA-N user acceptance test specs for clp CLI user stories. Each spec covers the
Acceptance Criteria defined in `docs/cli/user_story/`, verifying end-to-end scenario
correctness from the persona's perspective.

**UA- extension note:** UA- (User Acceptance) is a project-local element type extension not
registered in `test_surface.rulebook.md` (that file is outside the `module/claude_profile/`
package scope). This directory is the authorizing source for the UA- prefix.

### Responsibility Table

| File | Story | Persona | UA-N Cases |
|------|-------|---------|-----------|
| `001_account_rotation.md` | Automatic Account Rotation | SWE multi-account | UA-1 through UA-5 |
| `002_onboarding.md` | Account Onboarding and Lifecycle | Developer setup | UA-1 through UA-6 |
| `003_quota_monitoring.md` | Multi-Account Quota Monitoring | Power user | UA-1 through UA-5 |
| `004_scripted_automation.md` | Scripted Pipeline Automation | DevOps engineer | UA-1 through UA-4 |
| `005_credential_diagnostics.md` | Credential Diagnostics | Troubleshooting developer | UA-1 through UA-4 |

### Coverage Summary

| Story Files | Total Cases |
|-------------|-------------|
| 5 | 24 (5+6+5+4+4) |

### See Also

- [docs/cli/user_story/](../../../../docs/cli/user_story/readme.md) — user story source docs
