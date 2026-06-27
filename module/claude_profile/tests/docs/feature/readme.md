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
| 00_doc_structure.md | DT cases for Rule 9a structural compliance — all 47 feature docs |
| 01_account_store_init.md | FT cases for Feature 001 — Account Store Initialization |
| 02_account_save.md | FT cases for Feature 002 — Save Account |
| 03_account_list.md | FT cases for Feature 003 — Accounts |
| 04_account_use.md | FT cases for Feature 004 — Switch Account |
| 05_account_delete.md | FT cases for Feature 005 — Delete Account |
| 06_token_status.md | FT cases for Feature 006 — Token Status |
| 07_file_topology.md | FT cases for Feature 007 — File Topology |
| 08_auto_rotate.md | FT cases for Feature 008 — Auto Rotate |
| 09_token_usage.md | FT cases for Feature 009 — Token Usage Reporting |
| 10_persistent_storage.md | FT cases for Feature 010 — Persistent Storage Path |
| 11_account_status_by_name.md | FT cases for Feature 011 — Named Account Scoping |
| 12_live_credentials_status.md | FT cases for Feature 012 — Live Credentials Status |
| 13_account_limits.md | FT cases for Feature 013 — Account Rate-Limit Utilization |
| 14_rich_account_metadata.md | FT cases for Feature 014 — Rich Account Metadata |
| 15_name_shortcut_syntax.md | FT cases for Feature 015 — Account Name Shortcut Syntax |
| 16_current_account_awareness.md | FT cases for Feature 016 — Current Account Awareness |
| 17_token_refresh.md | FT cases for Feature 017 — Expired Token Refresh via Isolated Subprocess |
| 18_live_monitor.md | FT cases for Feature 018 — Live Quota Monitor Mode |
| 19_account_relogin.md | FT cases for Feature 019 — Browser Re-Authentication for Named Account |
| 20_usage_sort_strategies.md | FT cases for Feature 020 — Usage Sort Strategies |
| 21_extended_snapshot_fields.md | FT cases for Feature 021 — Extended Snapshot Fields |
| 22_org_identity_snapshot.md | FT cases for Feature 022 — Org Identity Snapshot |
| ~~23_next_account_strategies.md~~ | Feature 023 DEPRECATED — absorbed into Feature 020; no standalone test spec |
| 24_session_touch.md | FT cases for Feature 024 — Session Touch via Isolated Subprocess |
| 25_per_machine_active_marker.md | FT cases for Feature 025 — Per-Machine Active Marker |
| 26_subprocess_model_effort.md | FT cases for Feature 026 — Subprocess Model and Effort Control |
| 27_account_use_post_switch_touch.md | FT cases for Feature 027 — `.account.use` Post-Switch Touch |
| 28_usage_row_filtering.md | FT cases for Feature 028 — Usage Row Filtering and Extraction |
| 29_account_host_metadata.md | FT cases for Feature 029 — Account Host and Role Metadata |
| 30_account_renewal_override.md | FT cases for Feature 030 — Account Billing Renewal Override |
| 31_account_inspect.md | FT cases for Feature 031 — Account Inspect |
| 32_account_assign.md | FT cases for Feature 032 — Account Marker Assignment |
| 33_quota_cache.md | FT cases for Feature 033 — Quota Cache Fallback |
| 34_explicit_session_model_override.md | FT cases for Feature 034 — Explicit Session Model Override |
| 35_model_command.md | FT cases for Feature 035 — Dedicated Model Get/Set Command |
| 36_account_ownership.md | FT cases for Feature 036 — Account Ownership |
| 37_accounts_usage_param_unification.md | FT cases for Feature 037 — Accounts/Usage Parameter Set Unification |
| 38_usage_strategy_rotate.md | FT cases for Feature 038 — Usage Strategy Rotate |
| 39_decision_algorithms.md | FT cases for Feature 039 — Decision Algorithm Reference |
| 40_quota_measurement_history.md | FT cases for Feature 040 — Quota Measurement History and Polynomial Approximation |
| 61_solo_token_conservation.md | FT cases for Feature 061 — Solo Token Conservation Mode |
| 62_unified_session_config.md | FT cases for Feature 062 — Unified Session Config Recommendation |
| 63_explicit_ownership_claim.md | FT cases for Feature 063 — Explicit Ownership Claim |
| 64_active_marker_and_owner_redesign.md | FT cases for Feature 064 — Active Marker and Owner Param Redesign |
| 65_assignee_param_redesign.md | FT cases for Feature 065 — Assignee Param Redesign |
| 66_dual_source_quota_parsing.md | FT cases for Feature 066 — Dual-Source OAuth Quota Parsing |
| 67_trace_timestamps.md | FT cases for Feature 067 — Trace Timestamp Prefix |

### Coverage Summary

| Feature | File | Cases | Status |
|---------|------|-------|--------|
| doc_structure (collection) | [00_doc_structure.md](00_doc_structure.md) | DT-01 … DT-07 | ✅ |
| 001_account_store_init | [001_account_store_init.md](01_account_store_init.md) | FT-01 … FT-04 | ✅ |
| 002_account_save | [002_account_save.md](02_account_save.md) | FT-01 … FT-16 | ✅ |
| 003_account_list | [003_account_list.md](03_account_list.md) | FT-01 … FT-22 | ✅ |
| 004_account_use | [004_account_use.md](04_account_use.md) | FT-01 … FT-11 | ✅ |
| 005_account_delete | [005_account_delete.md](05_account_delete.md) | FT-01 … FT-07 | ✅ |
| 006_token_status | [006_token_status.md](06_token_status.md) | FT-01 … FT-04 | ✅ |
| 007_file_topology | [007_file_topology.md](07_file_topology.md) | FT-01 … FT-06 | ✅ |
| 008_auto_rotate | [008_auto_rotate.md](08_auto_rotate.md) | FT-01 … FT-04 | ✅ |
| 009_token_usage | [009_token_usage.md](09_token_usage.md) | FT-01 … FT-35 | ✅ |
| 010_persistent_storage | [010_persistent_storage.md](10_persistent_storage.md) | FT-01 … FT-07 | ✅ |
| 011_account_status_by_name | [011_account_status_by_name.md](11_account_status_by_name.md) | FT-01 … FT-05 | ✅ |
| 012_live_credentials_status | [012_live_credentials_status.md](12_live_credentials_status.md) | FT-01 … FT-07 | ✅ |
| 013_account_limits | [013_account_limits.md](13_account_limits.md) | FT-01 … FT-04 | ✅ |
| 014_rich_account_metadata | [014_rich_account_metadata.md](14_rich_account_metadata.md) | FT-01 … FT-12 | ✅ |
| 015_name_shortcut_syntax | [015_name_shortcut_syntax.md](15_name_shortcut_syntax.md) | FT-01 … FT-14 | ✅ |
| 016_current_account_awareness | [016_current_account_awareness.md](16_current_account_awareness.md) | FT-01 … FT-11 | ✅ |
| 017_token_refresh | [017_token_refresh.md](17_token_refresh.md) | FT-01 … FT-25 | ✅ |
| 018_live_monitor | [018_live_monitor.md](18_live_monitor.md) | FT-01 … FT-09 | ✅ |
| 019_account_relogin | [019_account_relogin.md](19_account_relogin.md) | FT-01 … FT-11 | ✅ |
| 020_usage_sort_strategies | [020_usage_sort_strategies.md](20_usage_sort_strategies.md) | FT-01 … FT-13 | ✅ |
| 021_extended_snapshot_fields | [021_extended_snapshot_fields.md](21_extended_snapshot_fields.md) | FT-01 … FT-09 | ✅ |
| 022_org_identity_snapshot | [022_org_identity_snapshot.md](22_org_identity_snapshot.md) | FT-01 … FT-11 | ✅ |
| ~~023_next_account_strategies~~ | ~~[023_next_account_strategies.md](23_next_account_strategies.md)~~ | ~~FT-01 … FT-19~~ | ⛔ DEPRECATED |
| 024_session_touch | [024_session_touch.md](24_session_touch.md) | FT-01 … FT-23 | ✅ |
| 025_per_machine_active_marker | [025_per_machine_active_marker.md](25_per_machine_active_marker.md) | FT-01 … FT-13 | ✅ |
| 026_subprocess_model_effort | [026_subprocess_model_effort.md](26_subprocess_model_effort.md) | FT-01 … FT-31 | ✅ |
| 027_account_use_post_switch_touch | [027_account_use_post_switch_touch.md](27_account_use_post_switch_touch.md) | FT-01 … FT-24 | ✅ |
| 028_usage_row_filtering | [028_usage_row_filtering.md](28_usage_row_filtering.md) | FT-01 … FT-17 | ✅ |
| 029_account_host_metadata | [029_account_host_metadata.md](29_account_host_metadata.md) | FT-01 … FT-10 | ✅ |
| 030_account_renewal_override | [030_account_renewal_override.md](30_account_renewal_override.md) | FT-01 … FT-15 | ✅ |
| 031_account_inspect | [031_account_inspect.md](31_account_inspect.md) | FT-01 … FT-31 | ✅ |
| 032_account_assign | [032_account_assign.md](32_account_assign.md) | FT-01 … FT-13 | ✅ |
| 033_quota_cache | [033_quota_cache.md](33_quota_cache.md) | FT-01 … FT-11 | ✅ |
| 034_explicit_session_model_override | [034_explicit_session_model_override.md](34_explicit_session_model_override.md) | FT-01 … FT-11 | ✅ |
| 035_model_command | [035_model_command.md](35_model_command.md) | FT-01 … FT-12 | ✅ |
| 036_account_ownership | [036_account_ownership.md](36_account_ownership.md) | FT-01 … FT-25 | ✅ |
| 037_accounts_usage_param_unification | [037_accounts_usage_param_unification.md](37_accounts_usage_param_unification.md) | FT-01 … FT-21 | ✅ |
| 038_usage_strategy_rotate | [038_usage_strategy_rotate.md](38_usage_strategy_rotate.md) | FT-01 … FT-11, CC-01 … CC-07 | ✅ |
| 039_decision_algorithms | [039_decision_algorithms.md](39_decision_algorithms.md) | FT-01 … FT-12 | ✅ |
| 040_quota_measurement_history | [040_quota_measurement_history.md](40_quota_measurement_history.md) | FT-01 … FT-18 | ✅ |
| 061_solo_token_conservation | [061_solo_token_conservation.md](61_solo_token_conservation.md) | FT-01 … FT-12 | ✅ |
| 062_unified_session_config | [062_unified_session_config.md](62_unified_session_config.md) | FT-01 … FT-13, EC-01 | ✅ |
| 063_explicit_ownership_claim | [063_explicit_ownership_claim.md](63_explicit_ownership_claim.md) | FT-01 … FT-12 | ✅ |
| 064_active_marker_and_owner_redesign | [064_active_marker_and_owner_redesign.md](64_active_marker_and_owner_redesign.md) | FT-01 … FT-19 | ✅ |
| 065_assignee_param_redesign | [065_assignee_param_redesign.md](65_assignee_param_redesign.md) | FT-01 … FT-13 | ✅ |
| 066_dual_source_quota_parsing | [066_dual_source_quota_parsing.md](66_dual_source_quota_parsing.md) | FT-01 … FT-12 | ✅ |
| 067_trace_timestamps | [067_trace_timestamps.md](67_trace_timestamps.md) | FT-01 … FT-07 | ✅ |

**Total:** 48 specs — 47 per-feature FT specs (behavioral) + 1 collection-level DT spec (doc structure compliance).

### Cross-Reference Depth

This directory is **3 levels** deep from the module root (`tests/docs/feature/`). Source cross-refs to `docs/feature/` must use `../../../docs/feature/NNN_*.md` (3-UP). Do NOT use `../../../../` (4-UP) — that overshoots into the parent of the module root. By contrast, `tests/docs/cli/command/` and `tests/docs/cli/param/` are 4 levels deep and correctly use `../../../../`.
