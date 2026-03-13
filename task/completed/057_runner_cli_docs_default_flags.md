---
id: 057
title: Update claude_runner CLI docs for default-on skip-permissions
status: ✅ Complete
category: documentation
created: 2026-03-31
completed: 2026-03-31
---

## Goal

Update `docs/cli/params.md`, `parameter_groups.md`, and `commands.md` to reflect
`--dangerously-skip-permissions` becoming a silently-injected default and
`--no-skip-permissions` becoming the user-facing opt-out.

## Scope

- `module/claude_runner/docs/cli/params.md`
- `module/claude_runner/docs/cli/parameter_groups.md`
- `module/claude_runner/docs/cli/commands.md`

## Done When

- `params.md`: param 5 is `--no-skip-permissions`; group assignment updated to Runner Control
- `parameter_groups.md`: Group 1 count = 3, Group 2 count = 8; tables updated; note about automatic injection added
- `commands.md`: `run` command param table updated
