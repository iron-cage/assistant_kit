# Behavior B22: No-Session-Persistence Disables Disk Writes

### Scope

- **Purpose**: Document that `--no-session-persistence` disables session disk writes and requires `--print` mode.
- **Responsibility**: Authoritative instance for behavior B22 — defines the behavior statement, certainty level, and supporting evidence.
- **In Scope**: `--no-session-persistence` flag; no `.jsonl` file creation; `--print` mode requirement; ephemeral use cases.
- **Out of Scope**: `CLAUDE_CODE_SESSION_DIR` for redirecting (not disabling) storage (→ [B23](023_b23_session_dir_override.md)); `--print` mode semantics (→ [B3](003_b3_print_orthogonal.md)).

### Behavior

**Status**: 🎯 Observed | **Certainty**: 85% | **Tier**: FLAG-VFY | **Since**: pre-v1.0 | **Evidence**: E41, E42

`--no-session-persistence` disables session disk writes; no `.jsonl` file is created and the session cannot be resumed. Only works with `--print` mode (non-interactive), since interactive mode requires session persistence for the terminal UI.

Useful for:
- Ephemeral CI queries where session history must not be written to disk
- Privacy-sensitive contexts where conversation data should not persist
- Testing/automation where session accumulation is unwanted

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E41 | B22 | Observation | `claude --help` live output | `--no-session-persistence` flag entry | Help text documents `--no-session-persistence` flag; notes it disables `.jsonl` creation and works only with `--print` mode |
| E42 | B22 | Test | `../../tests/behavior/b22_no_session_persistence_flag.rs` | `b22_no_session_persistence_flag_documented_in_help` | `claude --help` output contains `--no-session-persistence` flag |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [003_b3_print_orthogonal.md](003_b3_print_orthogonal.md) | `--print` mode (required by `--no-session-persistence`) |
| behavior | [023_b23_session_dir_override.md](023_b23_session_dir_override.md) | `CLAUDE_CODE_SESSION_DIR` (redirects rather than disables) |
| test | `../../tests/behavior/b22_no_session_persistence_flag.rs` | Invalidation test |
