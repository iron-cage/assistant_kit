# claude_version Tasks

<!-- task_system_metadata
type: root
registry_prefix: null
next_id: 8
-->

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `readme.md` | Task index and registry for the module |
| `decisions.md` | Design decision log for open questions |
| `procedure.md` | Task lifecycle procedures |
| `unverified/` | Tasks pending the verification gate |
| `verifying/` | Tasks undergoing verification gate review |
| `executing/` | Tasks being actively executed |
| `validating/` | Tasks whose output is being validated |
| `completed/` | Closed completed tasks |
| `cancelled/` | Closed cancelled tasks |
| `actors/` | Actor registry with canonical names and roles |
| `action_plan/` | Ordered execution plan across actors |

## Tasks Index

| Order | ID | Advisability | Value | Easiness | Safety | Priority | State | Executor | Dir | Task | Purpose |
|-------|----|-------------|-------|----------|--------|----------|-------|----------|-----|------|---------|
| 1 | 001 | 0 | 8 | 9 | 5 | 0 | ✅ (Closed) | any | tests/cli/ | [Verify kind_param and format tests in container](completed/001_verify_kind_format_tests.md) | Run 15 new test functions in container and fix any failures |
| 2 | 002 | 0 | 8 | 9 | 5 | 0 | ✅ (Closed) | any | tests/cli/ | [Verify .params and story 007 tests in container](completed/002_verify_params_story007_tests.md) | Run 24 new test functions in container and fix any failures |
| 3 | 003 | 0 | 8 | 9 | 5 | 0 | ✅ (Closed) | any | tests/cli/ | [Verify config command and params feature tests in container](completed/003_verify_config_feature_tests.md) | Run 53 new test functions in container and fix any failures |
| 4 | 004 | 0 | 7 | 8 | 5 | 0 | ✅ (Closed) | any | src/commands/ | [Implement .runtime_files CLI command](completed/004_implement_runtime_files_command.md) | Add .runtime_files command per l0_gov Runtime File Discovery Mandate |
| 5 | 005 | 672 | 8 | 7 | 6 | 2 | 🎯 (Verified) | any | ../claude_version_core/src/ | [Adopt unused version-pinning mechanisms in lock layer](005_adopt_unused_version_pinning_mechanisms.md) | Extend lock_version() with autoUpdatesChannel, minimumVersion, env.DISABLE_UPDATES |
| 6 | 006 | 896 | 8 | 7 | 8 | 2 | ❓ (Unverified) | any | ../claude_version_core/src/ | [Instrument parameter trace on all mutating functions](unverified/006_parameter_trace_mutating_functions.md) | Add unconditional stderr trace to 10 public mutating functions |
| 7 | 007 | 864 | 9 | 6 | 8 | 2 | ❓ (Unverified) | any | src/commands/ | [Show lock-state compliance in .status](unverified/007_lock_state_status_visibility.md) | Extend .status v::2 with actual-vs-expected lock mechanism compliance |
