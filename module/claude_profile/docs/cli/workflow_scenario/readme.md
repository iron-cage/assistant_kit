# Workflow Scenarios

Common usage patterns showing how `clp` CLI commands compose for real operational tasks.

| File | Responsibility |
|------|----------------|
| [01_account_rotation.md](01_account_rotation.md) | Switch to fresh account on token expiry |
| [02_onboarding.md](02_onboarding.md) | Save credentials when onboarding a new account |
| [03_scripted_health_check.md](03_scripted_health_check.md) | JSON pipeline integration for CI/CD health checks |
| [04_account_cleanup.md](04_account_cleanup.md) | Remove stale accounts with dry-run preview |
| [05_diagnostics.md](05_diagnostics.md) | Collect environment info for troubleshooting |
| [06_dry_run_preview.md](06_dry_run_preview.md) | Preview all mutation operations before executing |
| [07_fresh_installation.md](07_fresh_installation.md) | Inspect live credentials before account store exists |
| [08_live_quota_dashboard.md](08_live_quota_dashboard.md) | Continuous ambient quota monitoring display |
| [09_quota_auto_refresh.md](09_quota_auto_refresh.md) | Quota fetch with silent expired-token recovery |
| [10_account_relogin_recovery.md](10_account_relogin_recovery.md) | Browser re-authentication when refreshToken is dead |

**Total:** 10 scenarios
