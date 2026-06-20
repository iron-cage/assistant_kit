# Behavior B5: Current Session Selected by Mtime

### Scope

- **Purpose**: Document the inferred mechanism by which `--continue` selects the "most recent" session — filesystem modification time (mtime).
- **Responsibility**: Authoritative instance for behavior B5 — defines the behavior statement, certainty level, and supporting evidence. Certainty is capped at 60% (closed-source binary).
- **In Scope**: Mtime-based selection inference; VALIDATED† tier explanation; investigation priority.
- **Out of Scope**: UUID-based session selection (→ [B19](019_b19_resume_flag.md)); fork-session mechanics (→ [B21](021_b21_fork_session.md)).

### Behavior

**Status**: 🎯 Observed | **Certainty**: 60% | **Tier**: VALIDATED† | **Since**: pre-v1.0 | **Evidence**: E4, E15

The "current" session resumed by `--continue` is the most recently modified `.jsonl` file (mtime).

No explicit "current session pointer" metadata was found in the storage format. The most probable mechanism is filesystem mtime: `claude` reads the directory listing, sorts by modification time, and resumes the newest non-agent, non-empty `.jsonl` file.

Certainty is capped at 60% because the Claude Code binary is closed-source and this mechanism has not been confirmed by source inspection or official documentation. The test tier `VALIDATED†` reflects that distinct mtimes were confirmed to exist (feasibility proven) but that mtime is the actual selection key is unproven.

**Investigation priority:** High — can be confirmed by reading Claude Code changelog or public source if made available.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E4 | B5 | Inference | Storage observation | `~/.claude/projects/*/` | Multiple `.jsonl` files in one project; `--continue` must pick one; mtime is the only per-file ordering signal available without metadata |
| E15 | B5 | Test | `../../tests/behavior/b05_mtime_selection.rs` | `b5_real_sessions_have_distinct_mtimes` | Real project with 2+ sessions has distinct mtimes — mtime ordering is possible |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [004_b4_continue_flag.md](004_b4_continue_flag.md) | `--continue` flag that triggers this selection |
| behavior | [019_b19_resume_flag.md](019_b19_resume_flag.md) | `--resume`/`-r` as explicit UUID-based override of mtime selection |
| test | `../../tests/behavior/b05_mtime_selection.rs` | Invalidation test |
