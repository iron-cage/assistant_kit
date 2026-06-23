<!-- task_system_metadata
type: local
version: 1.0
crate: assistant
root: null
last_sync: null
-->

# Task Registry — assistant

### Scope

Task work items scoped to the `assistant` crate.

### Responsibility Table

| Path | Responsibility |
|------|----------------|
| `unverified/` | Unverified task files awaiting Verification Gate |
| `decisions.md` | Decision log for task-related decisions |

## Tasks Index

| Order | ID | Advisability | Value | Easiness | Safety | Priority | State | Executor | Dir | Task | Purpose |
|-------|----|----|---|---|---|---|---|---|---|---|---|
| 1 | 001 | — | 7 | 8 | 9 | 3 | 🎯 (Verified) | ai | . | [Aggregation Test Verification](001_aggregation_test_verification.md) | Verify aggregation.rs passes all 6 test spec cases in container |
| 2 | 002 | — | 8 | 6 | 9 | 2 | 🎯 (Verified) | ai | . | [Workspace Requirement Tests](002_workspace_requirement_tests.md) | Static analysis tests for workspace-level doc invariants |
