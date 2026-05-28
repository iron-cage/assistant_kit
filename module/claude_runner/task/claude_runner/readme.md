<!-- task_system_metadata
type: local
version: 1.0
crate: claude_runner
root: null
last_sync: null
-->

# Task Registry — claude_runner

### Scope

Task work items scoped to the `claude_runner` crate.

### Responsibility Table

| Path | Responsibility |
|------|----------------|
| `unverified/` | Unverified task files awaiting Verification Gate |
| `completed/` | Completed task files (all validation passed) |
| `cancelled/` | Cancelled task files (reopenable) |
| `actors/` | Actors Registry — canonical identity for all task actors |
| `action_plan/` | Per-actor action plan files |

## Tasks Index

| Order | ID | Advisability | Value | Easiness | Safety | Priority | State | Executor | Task | Purpose |
|-------|----|----|---|---|---|---|---|---|---|---|
| 1 | 001 | 0 | 8 | 6 | 9 | 0 | ✅ (Completed) | ai | [Test Surface Remediation](completed/001_test_surface_remediation.md) | Fix all 18 test surface spec violations and create missing user story coverage |
| 2 | 002 | 0 | 7 | 8 | 9 | 0 | ✅ (Completed) | ai | [US-16 CLI Discoverability Tests](completed/002_us16_cli_discoverability_tests.md) | Add 4 Rust test functions for user story 016 (CLI Discoverability) |
| 3 | 003 | 0 | 9 | 7 | 9 | 0 | ✅ (Completed) | ai | [BUG-212: run subcommand stripping](completed/003_bug_212_run_subcommand_stripping.md) | Fix `clr run` treating "run" as message argument |
| 4 | 004 | 0 | 7 | 9 | 9 | 0 | ✅ (Completed) | ai | [BUG-213: verbosity test env isolation](completed/004_bug_213_test_env_isolation.md) | Fix 4 verbosity tests failing under ambient CLR_TRACE |
