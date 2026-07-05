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
| `unverified/` | Tasks pending the verification gate |

## Tasks Index

| Order | ID | Advisability | Value | Easiness | Safety | Priority | State | Executor | Dir | Task | Purpose |
|-------|----|-------------|-------|----------|--------|----------|-------|----------|-----|------|---------|
| 5 | 005 | 672 | 8 | 7 | 6 | 2 | 🎯 (Verified) | any | ../claude_version_core/src/ | [Adopt unused version-pinning mechanisms in lock layer](005_adopt_unused_version_pinning_mechanisms.md) | Extend lock_version() with autoUpdatesChannel, minimumVersion, env.DISABLE_UPDATES |
| 6 | 006 | 896 | 8 | 7 | 8 | 2 | ❓ (Unverified) | any | ../claude_version_core/src/ | [Instrument parameter trace on all mutating functions](unverified/006_parameter_trace_mutating_functions.md) | Add unconditional stderr trace to 10 public mutating functions |
| 7 | 007 | 864 | 9 | 6 | 8 | 2 | ❓ (Unverified) | any | src/commands/ | [Show lock-state compliance in .status](unverified/007_lock_state_status_visibility.md) | Extend .status v::2 with actual-vs-expected lock mechanism compliance |
