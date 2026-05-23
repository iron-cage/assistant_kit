# Workflow Scenarios

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
