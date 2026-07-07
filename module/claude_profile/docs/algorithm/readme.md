# Algorithm Doc Entity

### Scope

- **Purpose**: Canonical reference for all decision algorithms embedded in `claude_profile` — model selection, quota classification, account recommendation, and approximation.
- **Responsibility**: Each instance documents one algorithm: its inputs, decision table, output, and primary source location. Feature docs reference these instances rather than duplicating the logic.
- **In Scope**: Algorithms that appear in multiple features or are complex enough to warrant isolation from their host feature doc.
- **Out of Scope**: Simple per-feature rules that have no cross-feature applicability; subprocess invocation mechanics (→ `subprocess/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| — | [procedure](procedure.md) | Workflow for maintaining algorithm instances | ✅ |
| 001 | [Touch Model Selection](001_touch_model_selection.md) | Select subprocess model from quota state and configuration | ✅ |
| 002 | [Session Model Override](002_session_model_override.md) | Override session model based on quota headroom and account state | ✅ |
| 003 | [Quota Status Groups](003_quota_status_groups.md) | Classify quota usage percentage into discrete status bands | ✅ |
| 004 | [Next-Account Eligibility Gates](004_eligibility_gates.md) | Apply eligibility gates to filter candidate accounts for next-account selection | ✅ |
| 005 | [Next-Account Positive Selection](005_next_account_selection.md) | Select best eligible account by sort strategy from candidate list | ✅ |
| 006 | [Quota Polynomial Approximation](006_quota_approximation.md) | Approximate quota utilization via polynomial regression on measurement history | ✅ |
| 007 | [Sort Strategies](007_sort_strategies.md) | Order account list by configurable sort strategies | ✅ |
| 008 | [Subprocess Effort Resolution](008_subprocess_effort_resolution.md) | Resolve subprocess effort level from explicit param and account state | ✅ |
| 009 | [OAuth Usage Response Dual-Source Parsing](009_oauth_usage_response_migration.md) | Parse OAuth usage response from named-field or limits-array format | ✅ |
| 010 | [Renewal Date Computation](010_renewal_date_computation.md) | Compute seconds until next billing renewal from exact or estimated billing day | ✅ |
| 011 | [Rounding-Boundary Classification Hazards](011_rounding_boundary_classification_hazards.md) | Document classification hazards when color/branch decisions and rounded display share the same raw float | ✅ |
| 012 | [Refresh Trace Reason Classification](012_refresh_trace_reason_classification.md) | Select a single human-readable reason string from ownership, cache, occupancy, and result state | ✅ |
