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
| 000_doc_structure.md | DT cases for Rule 9a structural compliance — all 33 feature docs |
| 001_account_store_init.md | FT cases for Feature 001 — Account Store Initialization |
| 002_account_save.md | FT cases for Feature 002 — Save Account |
| 003_account_list.md | FT cases for Feature 003 — Accounts |
| 004_account_use.md | FT cases for Feature 004 — Switch Account |
| 005_account_delete.md | FT cases for Feature 005 — Delete Account |
| 006_token_status.md | FT cases for Feature 006 — Token Status |
| 007_file_topology.md | FT cases for Feature 007 — File Topology |
| 008_auto_rotate.md | FT cases for Feature 008 — Auto Rotate |
| 009_token_usage.md | FT cases for Feature 009 — All-Accounts Live Quota Reporting (FR-14) |
| 010_persistent_storage.md | FT cases for Feature 010 — Persistent Storage Path |
| 011_account_status_by_name.md | FT cases for Feature 011 — Named Account Scoping |
| 012_live_credentials_status.md | FT cases for Feature 012 — Live Credentials Status |
| 013_account_limits.md | FT cases for Feature 013 — Account Rate-Limit Utilization |
| 014_rich_account_metadata.md | FT cases for Feature 014 — Rich Account Metadata |
| 015_name_shortcut_syntax.md | FT cases for Feature 015 — Account Name Shortcut Syntax |
| 016_current_account_awareness.md | FT cases for Feature 016 — Current Account Awareness |
| 017_token_refresh.md | FT cases for Feature 017 — Expired Token Refresh via Isolated Subprocess |
| 018_live_monitor.md | FT cases for Feature 018 — Live Quota Monitor Mode |
| 019_account_relogin.md | FT cases for Feature 019 — Browser Re-Authentication for Named Account |
| 020_usage_sort_strategies.md | FT cases for Feature 020 — Usage Sort Strategies |
| 021_extended_snapshot_fields.md | FT cases for Feature 021 — Extended Snapshot Fields |
| 022_org_identity_snapshot.md | FT cases for Feature 022 — Org Identity Snapshot |
| 023_next_account_strategies.md | FT cases for Feature 023 — Next Account Recommendation Strategies |
| 024_session_touch.md | FT cases for Feature 024 — Session Touch via Isolated Subprocess |
| 025_per_machine_active_marker.md | FT cases for Feature 025 — Per-Machine Active Marker |
| 026_subprocess_model_effort.md | FT cases for Feature 026 — Subprocess Model and Effort Control |
| 027_account_use_post_switch_touch.md | FT cases for Feature 027 — `.account.use` Post-Switch Touch |
| 028_usage_row_filtering.md | FT cases for Feature 028 — Usage Row Filtering and Extraction |
| 029_account_host_metadata.md | FT cases for Feature 029 — Account Host and Role Metadata |
| 030_account_renewal_override.md | FT cases for Feature 030 — Account Billing Renewal Override |
| 031_account_inspect.md | FT cases for Feature 031 — Account Inspect |
| 032_account_assign.md | FT cases for Feature 032 — Account Marker Assignment |
| 033_quota_cache.md | FT cases for Feature 033 — Quota Cache Fallback |
| 034_explicit_session_model_override.md | FT cases for Feature 034 — Explicit Session Model Override |

### Coverage Summary

| Feature | File | Cases | Status |
|---------|------|-------|--------|
| doc_structure (collection) | [000_doc_structure.md](000_doc_structure.md) | DT-01 … DT-07 | ✅ |
| 001_account_store_init | [001_account_store_init.md](001_account_store_init.md) | FT-01 … FT-04 | ✅ |
| 002_account_save | [002_account_save.md](002_account_save.md) | FT-01 … FT-11 | ✅ |
| 003_account_list | [003_account_list.md](003_account_list.md) | FT-01 … FT-19 | ✅ |
| 004_account_use | [004_account_use.md](004_account_use.md) | FT-01 … FT-09 | ✅ |
| 005_account_delete | [005_account_delete.md](005_account_delete.md) | FT-01 … FT-05 | ✅ |
| 006_token_status | [006_token_status.md](006_token_status.md) | FT-01 … FT-04 | ✅ |
| 007_file_topology | [007_file_topology.md](007_file_topology.md) | FT-01 … FT-06 | ✅ |
| 008_auto_rotate | [008_auto_rotate.md](008_auto_rotate.md) | FT-01 … FT-04 | ✅ |
| 009_token_usage | [009_token_usage.md](009_token_usage.md) | FT-01 … FT-25 | ✅ |
| 010_persistent_storage | [010_persistent_storage.md](010_persistent_storage.md) | FT-01 … FT-07 | ✅ |
| 011_account_status_by_name | [011_account_status_by_name.md](011_account_status_by_name.md) | FT-01 … FT-05 | ✅ |
| 012_live_credentials_status | [012_live_credentials_status.md](012_live_credentials_status.md) | FT-01 … FT-07 | ✅ |
| 013_account_limits | [013_account_limits.md](013_account_limits.md) | FT-01 … FT-04 | ✅ |
| 014_rich_account_metadata | [014_rich_account_metadata.md](014_rich_account_metadata.md) | FT-01 … FT-12 | ✅ |
| 015_name_shortcut_syntax | [015_name_shortcut_syntax.md](015_name_shortcut_syntax.md) | FT-01 … FT-13 | ✅ |
| 016_current_account_awareness | [016_current_account_awareness.md](016_current_account_awareness.md) | FT-01 … FT-11 | ✅ |
| 017_token_refresh | [017_token_refresh.md](017_token_refresh.md) | FT-01 … FT-17 | ✅ |
| 018_live_monitor | [018_live_monitor.md](018_live_monitor.md) | FT-01 … FT-09 | ✅ |
| 019_account_relogin | [019_account_relogin.md](019_account_relogin.md) | FT-01 … FT-09 | ✅ |
| 020_usage_sort_strategies | [020_usage_sort_strategies.md](020_usage_sort_strategies.md) | FT-01 … FT-17 | ✅ |
| 021_extended_snapshot_fields | [021_extended_snapshot_fields.md](021_extended_snapshot_fields.md) | FT-01 … FT-09 | ✅ |
| 022_org_identity_snapshot | [022_org_identity_snapshot.md](022_org_identity_snapshot.md) | FT-01 … FT-11 | ✅ |
| 023_next_account_strategies | [023_next_account_strategies.md](023_next_account_strategies.md) | FT-01 … FT-15 | ✅ |
| 024_session_touch | [024_session_touch.md](024_session_touch.md) | FT-01 … FT-17 | ✅ |
| 025_per_machine_active_marker | [025_per_machine_active_marker.md](025_per_machine_active_marker.md) | FT-01 … FT-12 | ✅ |
| 026_subprocess_model_effort | [026_subprocess_model_effort.md](026_subprocess_model_effort.md) | FT-01 … FT-21 | ✅ |
| 027_account_use_post_switch_touch | [027_account_use_post_switch_touch.md](027_account_use_post_switch_touch.md) | FT-01 … FT-19 | ✅ |
| 028_usage_row_filtering | [028_usage_row_filtering.md](028_usage_row_filtering.md) | FT-01 … FT-17 | ✅ |
| 029_account_host_metadata | [029_account_host_metadata.md](029_account_host_metadata.md) | FT-01 … FT-10 | ✅ |
| 030_account_renewal_override | [030_account_renewal_override.md](030_account_renewal_override.md) | FT-01 … FT-15 | ✅ |
| 031_account_inspect | [031_account_inspect.md](031_account_inspect.md) | FT-01 … FT-24 | ✅ |
| 032_account_assign | [032_account_assign.md](032_account_assign.md) | FT-01 … FT-12 | ✅ |
| 033_quota_cache | [033_quota_cache.md](033_quota_cache.md) | FT-01 … FT-11 | ✅ |
| 034_explicit_session_model_override | [034_explicit_session_model_override.md](034_explicit_session_model_override.md) | FT-01 … FT-11 | ✅ |

**Total:** 35 specs — 34 per-feature FT specs (behavioral) + 1 collection-level DT spec (doc structure compliance).
