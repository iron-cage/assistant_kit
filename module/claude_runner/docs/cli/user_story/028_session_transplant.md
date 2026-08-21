# User Story 028: Session Cross-Loading (Transplant)

### Scope

- **Persona**: Developer
- **Goal**: Run Claude in one project directory while resuming a session from a different project directory — either to branch work outward or to query/inject context inward.

### User Story

> As a developer working across multiple project directories,
> I want to run Claude in a target directory but continue a session from a source directory,
> so I can transplant context across projects without manually copying conversation history.

### Acceptance Criteria

- **AC-1 (Clone Outward):** `clr run --to /project-b --from /project-a "message"` causes Claude to run in `/project-b`, loading the most recent session from `/project-a`'s `CLAUDE_SESSION_DIR`. New conversation turns are written to `/project-b`'s session storage.
- **AC-2 (Inject Inward):** `clr run --from /project-b "message"` causes Claude to run in CWD, loading the most recent session from `/project-b`'s `CLAUDE_SESSION_DIR`.
- **AC-3 (No source history):** When `--from <DIR>` points to a directory with no qualifying session files, Claude starts a fresh session in the target directory (no error, no crash).
- **AC-4 (Default to CWD, first use):** `clr run --to /project-b "message"` (no `--from`), when `/project-b` has no session of its own yet, defaults the session source to CWD — Claude runs in `/project-b`, loading the most recent session from the directory `clr` was invoked in. Equivalent to explicitly passing `--from <cwd>`.
- **AC-5 (Alias `--to`):** `--to <DIR>` is accepted as an alias for `--dir <DIR>` with identical behavior.
- **AC-6 (Precedence):** `--session-dir` is deprecated and inert (claude ≥2.x ignores the override) — when both `--from` and `--session-dir` are given, `--session-dir` never suppresses the transplant, and emits a deprecation warning naming its value.
- **AC-7 (Isolation):** The source directory's session files are never modified by the cross-loaded run.
- **AC-8 (Bare invocation no-op):** `clr run "message"` (neither `--from` nor `--to`) plans no session transplant — both default to CWD, so source and target storage are identical and the self-copy guard suppresses the transplant plan. Ordinary `-c` continuation (unrelated to cross-loading) still applies independently.
- **AC-9 (Repeat use ignores CWD drift):** `clr run --to /project-b "message"` (no `--from`), called again once `/project-b` already has a session of its own (e.g. from a prior AC-4 clone), continues THAT target's own history rather than re-deriving the source from CWD — even if CWD's own most-recently-modified session has changed since the first call. Fix(BUG-541): a target directory used more than once must not have its own accumulated conversation silently orphaned by unrelated CWD drift. Applies uniformly to plain `--to`/`--dir` and to `--topic` (see [User Story 030](030_topic_creation.md) AC-001).

**Mechanism:** the runner physically copies the source session file into the target's own storage before spawn and injects bare `-c` — see [`../param/076_from.md`](../param/076_from.md) § Behavior. New turns append to the transplanted copy in target storage; the source file itself is never modified (AC-7).

### Primary Flags

| Flag | Role |
|------|------|
| `--from <DIR>` | Source directory for session lookup; defaults to the target's own storage once established (AC-9), else CWD (AC-4) |
| `--to <DIR>` | Alias for `--dir` (target directory where Claude runs); defaults to CWD when omitted |
| `--dir <DIR>` | Target directory where Claude runs |

### Examples

```sh
# Clone outward: run in project-b, load session from project-a
clr "Continue this feature in the new project" \
  --to /home/alice/project-b \
  --from /home/alice/project-a

# Inject inward: run in CWD (project-a), query session from project-b
clr "What did you implement in project-b?" \
  --from /home/alice/project-b

# --to alone: source defaults to CWD on first use; repeat calls continue
# project-b's own history instead (AC-9), regardless of CWD drift
clr "Continue this feature" --to /home/alice/project-b

# Bare invocation: no transplant (both default to CWD, self-copy guard suppresses it)
clr "Continue"
```

### Related Commands

| Command | Role |
|---------|------|
| `run` | Primary command for session cross-loading |
| `ask` | Also supports `--from`; identical to `run` |
| `topic` | Also supports `--from`; identical to `run`/`ask` — combines with topic's auto-named `--topic` target |

### Related Doc Instances

| File | Relationship |
|------|--------------|
| [`../param/076_from.md`](../param/076_from.md) | `--from` parameter spec |
| [`../param/008_dir.md`](../param/008_dir.md) | `--dir` / `--to` parameter spec |
| [`../feature/005_session_path_resolution.md`](../../feature/005_session_path_resolution.md) | Feature hub: `scope_for()` and cross-loading |
| [`../invariant/011_session_source_isolation.md`](../../invariant/011_session_source_isolation.md) | Read/write isolation invariant |

### Related User Stories

| # | Title | Relationship |
|---|-------|--------------|
| 005 | [Project-specific Execution](005_project_specific_execution.md) | `--dir` for running in specific directory |
| 007 | [Fresh Session](007_fresh_session.md) | `--new-session` takes precedence over `--from` |
| 029 | [Scope Inspection](029_scope_inspection.md) | Use `clr scope` to verify source/target paths before cross-loading |
| 030 | [Topic Creation](030_topic_creation.md) | `topic` combines `--from` cross-loading with an auto-named `--topic` target |
