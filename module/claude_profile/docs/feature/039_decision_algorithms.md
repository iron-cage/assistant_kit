# Feature 039 — Decision Algorithm Reference

- **Purpose**: Unified reference for the six core decision algorithms that govern model selection, quota classification, next-account recommendation, and quota approximation.
- **Cross-references**: [020](020_usage_sort_strategies.md) (sort strategies, status groups), [026](026_subprocess_model_effort.md) (touch model), [027](027_account_use_post_switch_touch.md) (session model override), [036](036_account_ownership.md) (ownership gates), [038](038_usage_strategy_rotate.md) (auto-switch), [040](040_quota_measurement_history.md) (measurement history and approximation), [061](061_solo_token_conservation.md) (solo gate predicate), [062](062_unified_session_config.md) (`recommended_model()` canonical entry point — see algorithm/002)

---

## Algorithm Index

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
