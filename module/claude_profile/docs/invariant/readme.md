# Invariant Doc Entity

### Scope

- **Purpose**: Defines the non-functional constraints that `claude_profile` must maintain at all times.
- **Responsibility**: Documents all quality invariants with their statements, enforcement mechanisms, and violation consequences.
- **In Scope**: invariant/001 through invariant/012 — all non-functional constraints and architectural guarantees for claude_profile.
- **Out of Scope**: Functional requirements (→ feature/), CLI design constraints (→ cli/).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| — | [procedure](procedure.md) | Workflow for maintaining invariant instances | ✅ |
| 001 | [Zero Third-Party Dependencies](001_zero_third_party_deps.md) | Library path must have zero third-party crates.io dependencies | ✅ |
| 002 | [Cross-Platform Compatibility](002_cross_platform.md) | All path operations work correctly on Linux, macOS, and Windows | ✅ |
| 003 | [Clear Error Messages](003_clear_errors.md) | All errors name the relevant resource and state corrective action | ✅ |
| 004 | [No Process Execution](004_no_process_execution.md) | `std::process::Command` is forbidden anywhere in the library | ✅ |
| 005 | [Atomic Account Switching](005_atomic_switching.md) | Account switching uses write-then-rename to prevent credential corruption | ✅ |
| 006 | [Parameters Default to Active Context](006_param_defaults.md) | Every parameter must have a default unless requiring an explicit value is absolutely necessary | ✅ |
| 007 | [JSON Storage Format](007_json_storage_format.md) | All `.json` files written to disk use `serde_json::to_string_pretty` + trailing `\n` | ✅ |
| 008 | [Single Token Refresh Entry Point](008_single_token_refresh_entry.md) | All token refresh goes through `refresh_account_token()`; no direct `run_isolated()` calls | ✅ |
| 009 | [Container-Only Test Execution](009_container_only_test_execution.md) | All tests must execute inside the runbox container; host-native execution is a hard error | ✅ |
| 010 | [Floating-Point Comparison vs. Display Consistency](010_floating_point_comparison_vs_display_consistency.md) | Classification/branch decisions must never diverge from the rounded display of the same value | ✅ |
| 011 | [Shared Predicate Consistency](011_shared_predicate_consistency.md) | Multi-field predicates (e.g. `billing_type=="none"` + `result`) must be evaluated identically at every call site | ✅ |
| 012 | [Label-Selection Requires Co-Occurrence Coverage](012_label_selection_requires_cooccurrence_coverage.md) | Label-selection functions must be tested against every co-occurring condition combination, not each in isolation | ✅ |
