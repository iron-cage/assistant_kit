---
id: 056
title: Add Default Flags Principle to claude_runner spec.md
status: ✅ Complete
category: documentation
created: 2026-03-31
completed: 2026-03-31
---

## Goal

Document the Default Flags Principle in `module/claude_runner/spec.md`.

## Scope

`module/claude_runner/spec.md` only.

## Done When

- New section `## Default Flags Principle` exists in spec.md with table and design rationale
- `--dangerously-skip-permissions` removed from Claude-native flags table (now implicit)
- `--no-skip-permissions` added to Runner-specific flags table
- Separation of Concerns table has default-on injection row
