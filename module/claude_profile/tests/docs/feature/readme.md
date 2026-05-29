# Feature Tests

Feature behavioral requirement test cases for `claude_profile`. Each file covers one feature doc instance from `docs/feature/` and maps its acceptance criteria to `FT-N` test cases.

### Scope

- **Purpose**: Document FT-N test cases for each feature behavioral requirement.
- **Responsibility**: Index of per-feature test case planning files covering AC coverage.
- **In Scope**: All feature doc instances in `docs/feature/` that have been validated or are under active work.
- **Out of Scope**: CLI command tests (→ `cli/command/`), parameter edge cases (→ `cli/param/`), automated test implementations (→ `tests/cli/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 002_account_save.md | FT cases for Feature 002 — Save Account |
| 009_token_usage.md | FT cases for Feature 009 — All-Accounts Live Quota Reporting (FR-14) |
| 017_token_refresh.md | FT cases for Feature 017 — Expired Token Refresh via Isolated Subprocess |
| 018_live_monitor.md | FT cases for Feature 018 — Live Quota Monitor Mode |
| 020_usage_sort_strategies.md | FT cases for Feature 020 — Usage Sort Strategies |
| 021_extended_snapshot_fields.md | FT cases for Feature 021 — Extended Snapshot Fields |
| 022_org_identity_snapshot.md | FT cases for Feature 022 — Org Identity Snapshot |
| 023_next_account_strategies.md | FT cases for Feature 023 — Next Account Recommendation Strategies |
| 028_usage_row_filtering.md | FT cases for Feature 028 — Usage Row Filtering and Extraction |
| 029_account_host_metadata.md | FT cases for Feature 029 — Account Host and Role Metadata |
| 030_account_renewal_override.md | FT cases for Feature 030 — Account Billing Renewal Override |
| 024_session_touch.md | FT cases for Feature 024 — Session Touch via Isolated Subprocess |
| 025_per_machine_active_marker.md | FT cases for Feature 025 — Per-Machine Active Marker |
| 026_subprocess_model_effort.md | FT cases for Feature 026 — Subprocess Model and Effort Control |
| 027_account_use_post_switch_touch.md | FT cases for Feature 027 — `.account.use` Post-Switch Touch |

### Coverage Summary

| Feature | File | Cases | Status |
|---------|------|-------|--------|
| 002_account_save | [002_account_save.md](002_account_save.md) | FT-01 … FT-11 | ⏳ |
| 009_token_usage | [009_token_usage.md](009_token_usage.md) | FT-01 … FT-19 | ⏳ |
| 017_token_refresh | [017_token_refresh.md](017_token_refresh.md) | FT-01 … FT-17 | ✅ |
| 018_live_monitor | [018_live_monitor.md](018_live_monitor.md) | FT-01 … FT-09 | ✅ |
| 020_usage_sort_strategies | [020_usage_sort_strategies.md](020_usage_sort_strategies.md) | FT-01 … FT-17 | ✅ |
| 021_extended_snapshot_fields | [021_extended_snapshot_fields.md](021_extended_snapshot_fields.md) | FT-01 … FT-09 | ✅ |
| 022_org_identity_snapshot | [022_org_identity_snapshot.md](022_org_identity_snapshot.md) | FT-01 … FT-11 | ✅ |
| 023_next_account_strategies | [023_next_account_strategies.md](023_next_account_strategies.md) | FT-01 … FT-11 | ✅ |
| 024_session_touch | [024_session_touch.md](024_session_touch.md) | FT-01 … FT-17 | ✅ |
| 025_per_machine_active_marker | [025_per_machine_active_marker.md](025_per_machine_active_marker.md) | FT-01 … FT-10 | ✅ |
| 026_subprocess_model_effort | [026_subprocess_model_effort.md](026_subprocess_model_effort.md) | FT-01 … FT-17 | ✅ |
| 027_account_use_post_switch_touch | [027_account_use_post_switch_touch.md](027_account_use_post_switch_touch.md) | FT-01 … FT-17 | ✅ |
| 028_usage_row_filtering | [028_usage_row_filtering.md](028_usage_row_filtering.md) | FT-01 … FT-16 | ✅ |
| 029_account_host_metadata | [029_account_host_metadata.md](029_account_host_metadata.md) | FT-01 … FT-10 | ✅ |
| 030_account_renewal_override | [030_account_renewal_override.md](030_account_renewal_override.md) | FT-01 … FT-15 | ⏳ |

**Total:** 15 feature specs; 15 of 30 feature instances not yet covered by dedicated FT specs.
