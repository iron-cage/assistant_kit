# Algorithm

### Scope

- **Purpose**: Canonical reference for all decision algorithms embedded in `claude_profile` — model selection, quota classification, account recommendation, and approximation.
- **Responsibility**: Each instance documents one algorithm: its inputs, decision table, output, and primary source location. Feature docs reference these instances rather than duplicating the logic.
- **In Scope**: Algorithms that appear in multiple features or are complex enough to warrant isolation from their host feature doc.
- **Out of Scope**: Simple per-feature rules that have no cross-feature applicability; subprocess invocation mechanics (→ `subprocess/`).

### Overview Table

| ID | Name | Entry Point | Feature Refs |
|----|------|-------------|-------------|
| 001 | [Touch Model Selection](001_touch_model_selection.md) | `subprocess.rs` `resolve_model()` | 024, 026, 027 |
| 002 | [Session Model Override](002_session_model_override.md) | `api.rs` `apply_model_override()`, `format.rs` `recommended_model()` | 009, 034, 062 |
| 003 | [Quota Status Groups](003_quota_status_groups.md) | `sort.rs` `status_group_of()` | 020, 038 |
| 004 | [Next-Account Eligibility Gates](004_eligibility_gates.md) | `sort_next.rs` `find_first_eligible()` | 020, 036, 038, 061 |
| 005 | [Next-Account Positive Selection](005_next_account_selection.md) | `sort_next.rs` `find_next_for_strategy()` | 020, 038 |
| 006 | [Quota Polynomial Approximation](006_quota_approximation.md) | `approx.rs` `approximate_utilization()` | 033, 040, 061 |
| 007 | [Sort Strategies](007_sort_strategies.md) | `sort.rs` `sort_indices()` | 020 |
| 008 | [Subprocess Effort Resolution](008_subprocess_effort_resolution.md) | `subprocess.rs` `resolve_effort()` | 026 |
