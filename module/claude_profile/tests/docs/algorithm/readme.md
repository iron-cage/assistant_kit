# Algorithm Doc Entity

Test-lens documentation for `docs/algorithm/`. Each spec file covers one algorithm and documents AC-N cases for algorithmic correctness verification. Tests live in `tests/` of the crate that owns the algorithm.

### Scope

- **Purpose**: AC-N test cases for decision algorithms in `claude_profile` and `claude_quota`.
- **Responsibility**: Index of per-algorithm correctness spec files; case prefix `AC-`.
- **In Scope**: Algorithms from `docs/algorithm/` that have been validated or are under active work.
- **Out of Scope**: Feature behavioral tests (→ `feature/`), CLI command integration tests (→ `cli/command/`).

### Overview Table

| Spec | Algorithm | Status |
|------|-----------|--------|
| 001_touch_model_selection.md | AC spec for algorithm 001 — Touch Model Selection | ✅ |
| 002_session_model_override.md | AC spec for algorithm 002 — Session Model Override | ✅ |
| 003_quota_status_groups.md | AC spec for algorithm 003 — Quota Status Groups | ✅ |
| 004_eligibility_gates.md | AC spec for algorithm 004 — Next-Account Eligibility Gates | ✅ |
| 005_next_account_selection.md | AC spec for algorithm 005 — Next-Account Positive Selection | ✅ |
| 006_quota_approximation.md | AC spec for algorithm 006 — Quota Polynomial Approximation | ✅ |
| 007_sort_strategies.md | AC spec for algorithm 007 — Sort Strategies | ✅ |
| 008_subprocess_effort_resolution.md | AC spec for algorithm 008 — Subprocess Effort Resolution | ✅ |
| 009_oauth_usage_response_migration.md | AC spec for algorithm 009 — OAuth Usage Response Dual-Source Parsing | ✅ |
