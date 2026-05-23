# Behavior B19: Resume Flag Selects Session by UUID

### Scope

- **Purpose**: Document that `--resume`/`-r` resumes a specific prior session by UUID rather than the most recently modified one.
- **Responsibility**: Authoritative instance for behavior B19 — defines the behavior statement, certainty level, and supporting evidence.
- **In Scope**: `--resume`/`-r <session-id>` flag; UUID-based selection; override of mtime-based selection (B5).
- **Out of Scope**: Fork-session that creates a new UUID when resuming (→ [B21](021_b21_fork_session.md)); `--continue` that uses mtime selection (→ [B5](005_b5_mtime_selection.md)).

### Behavior

**Status**: 🎯 Observed | **Certainty**: 85% | **Tier**: FLAG-VFY | **Evidence**: E35, E36

`--resume <session-id>` (shorthand `-r`) selects a specific `.jsonl` file to resume by UUID rather than using the most recently modified file. This is the explicit override for B5's mtime-based selection.

The session UUID must match the filename of an existing `.jsonl` in the project's storage directory. Combined with `--fork-session`, it branches from that specific checkpoint.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E35 | B19 | Observation | `claude --help` live output | `--resume` flag entry | Help text documents `--resume` / `-r <session-id>` flag for resuming a specific prior session by UUID |
| E36 | B19 | Test | `../../tests/behavior/b19_resume_flag.rs` | `b19_resume_flag_documented_in_help` | `claude --help` output contains `--resume` flag |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [005_b5_mtime_selection.md](005_b5_mtime_selection.md) | Mtime-based selection that `--resume` overrides |
| behavior | [021_b21_fork_session.md](021_b21_fork_session.md) | `--fork-session` that branches from a resumed session |
| test | `../../tests/behavior/b19_resume_flag.rs` | Invalidation test |
