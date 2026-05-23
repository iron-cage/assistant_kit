# Behavior B6: Sessions Accumulate Without Rotation

### Scope

- **Purpose**: Document that session files accumulate indefinitely — one per independent invocation — without compaction or rotation.
- **Responsibility**: Authoritative instance for behavior B6 — defines the behavior statement, certainty level, and supporting evidence.
- **In Scope**: Accumulation pattern; no compaction or rotation; growth characteristics.
- **Out of Scope**: Per-invocation file creation mechanics (→ [B2](002_b2_new_session_creates_file.md)); storage size implications (→ [`../storage/001_projects_directory.md`](../storage/001_projects_directory.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 90% | **Tier**: VALIDATED | **Evidence**: E5, E16

Each project directory accumulates one `.jsonl` file per independent session invocation (each call without `--continue`). Session files are never compacted or rotated — they persist indefinitely.

Observed: 25 `.jsonl` files in a single `-commit/` project directory from repeated sessions. The test confirms presence of 5+ files in at least one real project (higher threshold than B2's 2+ to confirm long-term accumulation).

Implication: storage grows unbounded over time. `debug/` and old `shell-snapshots/` can be cleared to reclaim space; `projects/` cannot be safely deleted without losing conversation history.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E5 | B6 | Observation | Live storage | `~/.claude/projects/…/-commit/` | 25 `.jsonl` files observed in one project directory from repeated sessions |
| E16 | B6 | Test | `../../tests/behavior/b06_session_accumulation.rs` | `b6_sessions_accumulate_in_real_project` | Real project directory contains 5+ `.jsonl` files — confirms long-term accumulation without rotation |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [002_b2_new_session_creates_file.md](002_b2_new_session_creates_file.md) | Per-invocation file creation |
| storage | [`../storage/001_projects_directory.md`](../storage/001_projects_directory.md) | Projects directory growth characteristics |
| test | `../../tests/behavior/b06_session_accumulation.rs` | Invalidation test |
