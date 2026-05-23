# Behavior B1: Default Session Is New

### Scope

- **Purpose**: Document that `claude` defaults to a new session on each invocation, and that the `clr` wrapper inverts this default.
- **Responsibility**: Authoritative instance for behavior B1 — defines the behavior statement, certainty level, and supporting evidence.
- **In Scope**: Binary-level default (new session), `-c`/`--continue` flag as opt-in, `clr` wrapper inversion via default `-c` pass-through.
- **Out of Scope**: Physical file creation mechanics (→ [B2](002_b2_new_session_creates_file.md)); mtime-based current session selection (→ [B5](005_b5_mtime_selection.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 90% | **Tier**: VALIDATED | **Evidence**: E1, E2, E11

The `claude` binary defaults to a NEW session on every invocation. Continuation requires an explicit `-c` / `--continue` flag:

```
claude                   # starts a new session (binary default)
claude --continue        # resumes most recently modified session
claude -c "message"      # resumes + sends message (non-interactive)
claude -p "message"      # starts new session; explicit --print flag
```

The `clr` wrapper inverts this default by always passing `-c` unless `--new-session` is given. `--new-session` is a **`clr`-only flag** (absent from `claude --help`); it suppresses the wrapper's default `-c` to restore binary-default behavior (fresh start):

```
clr                      # passes -c → continues most recent session
clr --new-session        # omits -c → new .jsonl file (binary default)
```

Each session without `--continue` creates exactly one new `.jsonl` file in the project's storage directory (B2). Over time this produces a directory with one file per distinct session (B6).

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E1 | B1, B2 | Code | `../../../../module/claude_runner/src/main.rs` | line 85 | `--new-session  Start a new session (default: continues previous)` — `clr` wrapper help text; confirms wrapper default is continuation |
| E2 | B1, B4 | Code | `../../../../module/claude_runner_core/src/command.rs` | line 600 | `if self.continue_conversation { parts.push("-c") }` — `-c` is a builder option wrapping the native flag |
| E11 | B1 | Test | `../../tests/behavior/b01_default_continues.rs` | `b1_resumable_session_exists_in_real_storage` | At least one non-empty non-agent session exists in real `~/.claude/` storage — prerequisite for default continuation |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [002_b2_new_session_creates_file.md](002_b2_new_session_creates_file.md) | Physical `.jsonl` file creation per session |
| behavior | [004_b4_continue_flag.md](004_b4_continue_flag.md) | `-c`/`--continue` flag semantics |
| test | `../../tests/behavior/b01_default_continues.rs` | Invalidation test |
