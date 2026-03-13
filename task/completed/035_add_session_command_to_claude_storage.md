# Add .session Command to claude_storage CLI

## Status

✅ **Complete** (2026-03-22)

## Goal

Add a `.session` command to the `claude_storage` CLI that checks whether a
directory has conversation history in `~/.claude/projects/`. Uses the new
`claude_storage_core::continuation::check_continuation()` API (added in task-033).

## In Scope

- Add `.session` command to `claude_storage` CLI
- Delegate to `claude_storage_core::check_continuation(path)`
- Default path: current working directory
- Accept optional `path::` parameter for explicit directory
- Update `claude_storage/spec.md` commands table
- Add tests in `claude_storage/tests/`

## Out of Scope

- `.session.clear` (destructive; deferred pending separate design)
- Changes to `claude_storage_core`

## Acceptance Criteria

```
# Check current directory
claude_storage .session
→ "has history" or "no history"

# Check specific directory
claude_storage .session path::/home/user/project/-debug
→ "has history" or "no history"
```

## Background

After task-033 moved continuation detection to `claude_storage_core`, the
natural user-facing surface for this capability is the `claude_storage` CLI.
Users can check whether a working directory has conversation history before
deciding to run Claude with `-c` flag.

## Outcomes

- `claude_storage/src/cli/mod.rs` — `session_routine()` added; delegates to `claude_storage_core::check_continuation(path)`
- `claude_storage/src/main.rs` — `.session` command registered via `".session" => cli::session_routine`
- `claude_storage/spec.md` — `.session` command added to commands table
- `claude_storage/tests/` — tests covering `.session` with and without `path::` parameter
- Acceptance criteria from task file verified: bare `.session` and `path::` form both work correctly
