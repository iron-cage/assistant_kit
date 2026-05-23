# Behavior B20: Session-ID Flag Assigns Deterministic UUID

### Scope

- **Purpose**: Document that `--session-id <uuid>` assigns a caller-supplied deterministic UUID to the current session.
- **Responsibility**: Authoritative instance for behavior B20 — defines the behavior statement, certainty level, and supporting evidence.
- **In Scope**: `--session-id <uuid>` flag; deterministic UUID assignment; interaction with `--resume` and `--fork-session`.
- **Out of Scope**: UUID-based resume of existing session (→ [B19](019_b19_resume_flag.md)); fork-session mechanics (→ [B21](021_b21_fork_session.md)).

### Behavior

**Status**: 🎯 Observed | **Certainty**: 80% | **Tier**: FLAG-VFY | **Evidence**: E37, E38

By default Claude Code generates a random UUIDv4 for each new session. `--session-id <uuid>` overrides this to a caller-supplied UUID. Useful for reproducible automation where session identity must be deterministic (e.g., linking Claude invocations to external tracking systems).

If the supplied UUID already matches an existing `.jsonl` file, behavior depends on other flags:
- With `--resume`: appends to the existing session
- With `--fork-session`: branches into a new UUID

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E37 | B20 | Observation | `claude --help` live output | `--session-id` flag entry | Help text documents `--session-id <uuid>` flag for assigning a deterministic UUID to the current session |
| E38 | B20 | Test | `../../tests/behavior/b20_session_id_flag.rs` | `b20_session_id_flag_documented_in_help` | `claude --help` output contains `--session-id` flag |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [019_b19_resume_flag.md](019_b19_resume_flag.md) | `--resume`/`-r` resume by UUID |
| behavior | [021_b21_fork_session.md](021_b21_fork_session.md) | `--fork-session` branching |
| test | `../../tests/behavior/b20_session_id_flag.rs` | Invalidation test |
