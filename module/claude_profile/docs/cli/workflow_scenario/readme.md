# Workflow Scenarios

> **Deprecated.** This directory was eliminated in `cli_doc.rulebook` v1.7. Workflow scenarios have been superseded by [User Stories](../user_story/readme.md), which provide the same persona-and-goal coverage under the canonical cli_doc schema. No new files should be added here. Existing files are retained for reference only.
>
> **Migration map:**
>
> | Scenario | Migrated to User Story |
> |----------|------------------------|
> | [001_account_rotation.md](001_account_rotation.md) | [001_account_rotation.md](../user_story/001_account_rotation.md) |
> | [002_onboarding.md](002_onboarding.md) | [002_onboarding.md](../user_story/002_onboarding.md) |
> | [003_scripted_health_check.md](003_scripted_health_check.md) | [004_scripted_automation.md](../user_story/004_scripted_automation.md) |
> | [004_account_cleanup.md](004_account_cleanup.md) | [002_onboarding.md](../user_story/002_onboarding.md) (delete path) |
> | [005_diagnostics.md](005_diagnostics.md) | [005_credential_diagnostics.md](../user_story/005_credential_diagnostics.md) |
> | [006_dry_run_preview.md](006_dry_run_preview.md) | No direct equivalent — covered inline in command docs |
> | [007_fresh_installation.md](007_fresh_installation.md) | [005_credential_diagnostics.md](../user_story/005_credential_diagnostics.md) |
> | [008_live_quota_dashboard.md](008_live_quota_dashboard.md) | [003_quota_monitoring.md](../user_story/003_quota_monitoring.md) |
> | [009_quota_auto_refresh.md](009_quota_auto_refresh.md) | [003_quota_monitoring.md](../user_story/003_quota_monitoring.md) |
> | [010_account_relogin_recovery.md](010_account_relogin_recovery.md) | [002_onboarding.md](../user_story/002_onboarding.md) (relogin path) |

Common usage patterns showing how `clp` CLI commands compose for real operational tasks.

| File | Responsibility |
|------|----------------|
| [001_account_rotation.md](001_account_rotation.md) | Switch to fresh account on token expiry |
| [002_onboarding.md](002_onboarding.md) | Save credentials when onboarding a new account |
| [003_scripted_health_check.md](003_scripted_health_check.md) | JSON pipeline integration for CI/CD health checks |
| [004_account_cleanup.md](004_account_cleanup.md) | Remove stale accounts with dry-run preview |
| [005_diagnostics.md](005_diagnostics.md) | Collect environment info for troubleshooting |
| [006_dry_run_preview.md](006_dry_run_preview.md) | Preview all mutation operations before executing |
| [007_fresh_installation.md](007_fresh_installation.md) | Inspect live credentials before account store exists |
| [008_live_quota_dashboard.md](008_live_quota_dashboard.md) | Continuous ambient quota monitoring display |
| [009_quota_auto_refresh.md](009_quota_auto_refresh.md) | Quota fetch with silent expired-token recovery |
| [010_account_relogin_recovery.md](010_account_relogin_recovery.md) | Browser re-authentication when refreshToken is dead |

**Total:** 10 scenarios
