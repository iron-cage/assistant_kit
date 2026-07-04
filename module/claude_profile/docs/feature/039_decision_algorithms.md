# Feature: Decision Algorithm Reference

### Scope

- **Purpose**: Unified reference index for the eight core decision algorithms governing model selection, quota classification, next-account recommendation, and quota approximation.
- **Responsibility**: Documents the algorithm index table linking each algorithm to its canonical `algorithm/` doc instance and entry point.
- **In Scope**: Algorithm index (algorithms 1–8); canonical entry points; `algorithm/` doc cross-references.
- **Out of Scope**: Individual algorithm logic (→ `algorithm/` doc instances).

### Algorithm Index

Each algorithm is documented in its canonical `algorithm/` doc — single source of truth. Source file paths are in each canonical doc.

| # | Algorithm | Canonical doc | Entry point |
|---|---|---|---|
| 1 | Touch model selection | [algorithm/001](../algorithm/001_touch_model_selection.md) | `subprocess.rs` `resolve_model()` |
| 2 | Session model override | [algorithm/002](../algorithm/002_session_model_override.md) | `api.rs` `apply_model_override()`, `format.rs` `recommended_model()` |
| 3 | Quota status groups | [algorithm/003](../algorithm/003_quota_status_groups.md) | `sort.rs` `status_group_of()` |
| 4 | Next-account eligibility gates | [algorithm/004](../algorithm/004_eligibility_gates.md) | `sort_next.rs` `find_first_eligible()` + `extra` closure |
| 5 | Next-account positive selection | [algorithm/005](../algorithm/005_next_account_selection.md) | `sort_next.rs` `find_next_for_strategy()` |
| 6 | Quota polynomial approximation | [algorithm/006](../algorithm/006_quota_approximation.md) | `approx.rs` `approximate_utilization()` |
| 7 | Sort strategies + `prefer_weekly` | [algorithm/007](../algorithm/007_sort_strategies.md) | `sort.rs` `sort_indices()`, `relevant_quotas()` |
| 8 | Subprocess effort resolution | [algorithm/008](../algorithm/008_subprocess_effort_resolution.md) | `subprocess.rs` `resolve_effort()` |

### Features

| File | Relationship |
|------|-------------|
| [020_usage_sort_strategies.md](020_usage_sort_strategies.md) | Sort strategies, status groups |
| [026_subprocess_model_effort.md](026_subprocess_model_effort.md) | Touch model parameter spec |
| [027_account_use_post_switch_touch.md](027_account_use_post_switch_touch.md) | Session model override |
| [036_account_ownership.md](036_account_ownership.md) | Ownership gates |
| [038_usage_strategy_rotate.md](038_usage_strategy_rotate.md) | Auto-switch |
| [040_quota_measurement_history.md](040_quota_measurement_history.md) | Measurement history and approximation |
| [061_solo_token_conservation.md](061_solo_token_conservation.md) | Solo gate predicate |
| [062_unified_session_config.md](062_unified_session_config.md) | `recommended_model()` canonical entry point |
