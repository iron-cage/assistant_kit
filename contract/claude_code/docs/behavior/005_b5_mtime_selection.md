# Behavior B5: Continue Session Selection Rule

### Scope

- **Purpose**: Document which session `--continue` resumes — the documented candidate filter, and the still-unconfirmed ordering key within that filtered set.
- **Responsibility**: Authoritative instance for behavior B5 — defines the behavior statement, certainty level, and supporting evidence. Splits the confirmed candidate-set rule from the inferred ordering key.
- **In Scope**: Officially documented exclusion filter; the interactive vs `-p` filter difference; mtime-as-ordering-key inference; VALIDATED† tier explanation.
- **Out of Scope**: UUID-based session selection (→ [B19](019_b19_resume_flag.md)); fork-session mechanics (→ [B21](021_b21_fork_session.md)); storage path encoding (→ [B9](009_b9_storage_path_encoding.md)).

### Behavior

**Status**: ✅ Confirmed (candidate filter) / ❓ Uncertain (ordering key) | **Certainty**: 95% filter, 55% mtime key | **Tier**: VALIDATED† | **Since**: pre-v1.0 | **Evidence**: E4, E15, E71

This behavior has two separable claims. Official documentation settles the first and is silent on the second.

**1. Candidate filter — ✅ Confirmed (95%).** `claude --continue` resumes the most recent *interactive* session in the current directory. It is not a plain "newest `.jsonl` in the directory" rule: Claude Code excludes specific session classes from the candidate set (E71):

| Session class | Excluded from `claude --continue` | Excluded from `claude -p --continue` |
|---------------|-----------------------------------|--------------------------------------|
| Background sessions | Yes | Yes |
| Sessions created by `claude -p` or the Agent SDK | Yes | No — included |
| Sessions whose first prompt was `/loop` | Yes | No — included |

The interactive and print-mode filters therefore differ: `-p --continue` widens the candidate set to everything except background sessions. Running `/loop` later in a conversation does not exclude the session — only a `/loop` first prompt does.

**2. Ordering key — ❓ Uncertain (55%).** Within the filtered candidate set, "most recent" is not defined by official documentation in terms of any concrete on-disk field. Filesystem mtime remains the most probable key, since no "current session pointer" metadata exists in the storage format and mtime is the only per-file ordering signal available without parsing every transcript. Last-activity time parsed from transcript entries is an equally consistent alternative that the available evidence does not rule out — the session picker displays "time since last activity", which would be derivable either way.

Certainty on the ordering key stays below 60% because the binary is closed-source and official documentation describes the *selection outcome* rather than the *sort key*. The `VALIDATED†` tier reflects that distinct mtimes were confirmed to exist (feasibility proven) while mtime-as-selection-key is unproven.

**Known test gap:** `b05_mtime_selection.rs` asserts only that distinct mtimes exist. It does not exercise the documented exclusion filter — a regression that made `--continue` pick a `-p` or background session would not turn this test RED.

**Superseded claim:** before this revision, B5 stated flatly that `--continue` resumes "the most recently modified `.jsonl` file (mtime)". That statement is incorrect as written — it omits the exclusion filter that official documentation specifies.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E4 | B5 | Inference | Storage observation | `~/.claude/projects/*/` | Multiple `.jsonl` files in one project; `--continue` must pick one; mtime is the only per-file ordering signal available without metadata |
| E15 | B5 | Test | `../../tests/behavior/b05_mtime_selection.rs` | `b5_real_sessions_have_distinct_mtimes` | Real project with 2+ sessions has distinct mtimes — mtime ordering is possible |
| E71 | B5 | Doc | Official Claude Code documentation (code.claude.com/docs/en/sessions § Resume a session) | `--continue` row and following paragraph | "`claude --continue` — Resumes the most recent interactive session in the current directory." And: "Claude Code leaves sessions created with `claude -p` or the Agent SDK out of the session picker and out of `claude --continue`… With `claude --continue`, Claude Code also skips background sessions and sessions whose first prompt was `/loop`. When you run `claude -p --continue`, Claude Code includes `-p`, SDK, and `/loop` sessions and still skips background sessions." No sort key is stated. |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [004_b4_continue_flag.md](004_b4_continue_flag.md) | `--continue` flag that triggers this selection |
| behavior | [019_b19_resume_flag.md](019_b19_resume_flag.md) | `--resume`/`-r` as explicit UUID-based override of mtime selection |
| test | `../../tests/behavior/b05_mtime_selection.rs` | Invalidation test |
