# Behavior B21: Fork Session Branches Without Modifying Original

### Scope

- **Purpose**: Document that `--fork-session` creates a new session UUID when resuming, preserving the original session unchanged.
- **Responsibility**: Authoritative instance for behavior B21 — defines the behavior statement, certainty level, and supporting evidence.
- **In Scope**: `--fork-session` flag; new UUID creation; history copy; original session preservation.
- **Out of Scope**: `--resume`/`-r` that selects by UUID (→ [B19](019_b19_resume_flag.md)); `--session-id` that assigns deterministic UUID (→ [B20](020_b20_session_id_flag.md)).

### Behavior

**Status**: 🎯 Observed | **Certainty**: 80% | **Tier**: FLAG-VFY | **Evidence**: E39, E40

When resuming a session (`--resume` or `--continue`), `--fork-session` creates a new session UUID rather than appending to the original file. The resumed history is copied into the new `.jsonl`, leaving the source session untouched.

This is the mechanism for checkpoint branching: explore alternative conversation paths from a known-good state without polluting the original session file.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E39 | B21 | Observation | `claude --help` live output | `--fork-session` flag entry | Help text documents `--fork-session` flag for branching from a prior session without modifying the original |
| E40 | B21 | Test | `../../tests/behavior/b21_fork_session_flag.rs` | `b21_fork_session_flag_documented_in_help` | `claude --help` output contains `--fork-session` flag |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [019_b19_resume_flag.md](019_b19_resume_flag.md) | `--resume`/`-r` that is typically combined with `--fork-session` |
| behavior | [020_b20_session_id_flag.md](020_b20_session_id_flag.md) | `--session-id` for deterministic UUID after fork |
| test | `../../tests/behavior/b21_fork_session_flag.rs` | Invalidation test |
