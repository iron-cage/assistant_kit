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
| 009_token_usage.md | FT cases for Feature 009 — All-Accounts Live Quota Reporting (FR-14) |
| 017_token_refresh.md | FT cases for Feature 017 — Expired Token Refresh via Isolated Subprocess |
| 018_live_monitor.md | FT cases for Feature 018 — Live Quota Monitor Mode |
| 020_usage_sort_strategies.md | FT cases for Feature 020 — Usage Sort Strategies |
| 021_extended_snapshot_fields.md | FT cases for Feature 021 — Extended Snapshot Fields |
| 022_org_identity_snapshot.md | FT cases for Feature 022 — Org Identity Snapshot |
| 023_next_account_strategies.md | FT cases for Feature 023 — Next Account Recommendation Strategies |
| 024_session_touch.md | FT cases for Feature 024 — Session Touch via Isolated Subprocess |
| 025_per_machine_active_marker.md | FT cases for Feature 025 — Per-Machine Active Marker |
| 026_subprocess_model_effort.md | FT cases for Feature 026 — Subprocess Model and Effort Control |

### Coverage Summary

| Feature | File | Cases | Status |
|---------|------|-------|--------|
| 009_token_usage | [009_token_usage.md](009_token_usage.md) | FT-01 … FT-16 | ✅ |
| 017_token_refresh | [017_token_refresh.md](017_token_refresh.md) | FT-01 … FT-16 | ✅ |
| 018_live_monitor | [018_live_monitor.md](018_live_monitor.md) | FT-01 … FT-09 | ✅ |
| 020_usage_sort_strategies | [020_usage_sort_strategies.md](020_usage_sort_strategies.md) | FT-01 … FT-17 | ✅ |
| 021_extended_snapshot_fields | [021_extended_snapshot_fields.md](021_extended_snapshot_fields.md) | FT-01 … FT-09 | ✅ |
| 022_org_identity_snapshot | [022_org_identity_snapshot.md](022_org_identity_snapshot.md) | FT-01 … FT-11 | ✅ |
| 023_next_account_strategies | [023_next_account_strategies.md](023_next_account_strategies.md) | FT-01 … FT-08 | ✅ |
| 024_session_touch | [024_session_touch.md](024_session_touch.md) | FT-01 … FT-14 | ✅ (FT-13, FT-14 ⏳) |
| 025_per_machine_active_marker | [025_per_machine_active_marker.md](025_per_machine_active_marker.md) | FT-01 … FT-08 | ✅ (FT-08 ⏳) |
| 026_subprocess_model_effort | [026_subprocess_model_effort.md](026_subprocess_model_effort.md) | FT-01 … FT-17 | ✅ |

**Total:** 10 feature specs; 16 of 26 feature instances not yet covered by dedicated FT specs.
