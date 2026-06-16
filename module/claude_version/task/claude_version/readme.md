# Task Registry — claude_version

### Scope

Task work items scoped to the `claude_version` crate.

### Responsibility Table

| Path | Responsibility |
|------|----------------|
| `bug/` | Filed bug reports for claude_version |
| `unverified/` | Task files pending Verification Gate review |
| `completed/` | Task files in Completed state |
| `cancelled/` | Task files in Cancelled state |
| `actors/` | Actor registry — canonical identity records |
| `action_plan/` | Per-actor ordered action plans |

### Tasks Index

| Order | ID | Advisability | Value | Easiness | Safety | Priority | State | Executor | Dir | Task | Purpose |
|-------|----|-------------|-------|----------|--------|----------|-------|----------|-----|------|---------|
| 1 | 004 | 315 | 7 | 5 | 9 | 3 | 🎯 (Verified) | any | . | [004_test_surface_remediation.md](004_test_surface_remediation.md) | Implement pending type/user-story tests and fix 29 spec quality problems |
| 2 | 003 | 448 | 8 | 4 | 7 | 2 | 🎯 (Verified) | any | . | [003_config_command.md](003_config_command.md) | Implement `.config` command with 4-layer resolution and catalog |
| 2 | 001 | 0 | 5 | 9 | 9 | 0 | ✅ (Completed) | any | . | [001_bug001_guard_doc_comment.md](completed/001_bug001_guard_doc_comment.md) | Fix `guard_once_pinned` doc comment — `resolved` advisory semantics |
| 3 | 002 | 0 | 8 | 6 | 8 | 0 | ✅ (Completed) | any | . | [002_cli_type_test_surface.md](completed/002_cli_type_test_surface.md) | Implement 21 pending CLI type test surface functions |
