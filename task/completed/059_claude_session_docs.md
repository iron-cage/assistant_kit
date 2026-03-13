---
id: 059
title: Create spec.md and readme.md for claude_session crate
status: ✅ Complete
category: documentation
created: 2026-03-31
completed: 2026-03-31
---

## Goal

Document the `claude_session` crate, which currently has no documentation
(no `spec.md`, no `readme.md`, no `tests/` directory).

## Scope

- `module/claude_session/readme.md` — CREATE
- `module/claude_session/spec.md` — CREATE

## Context

`claude_session` is a standalone stub crate (not in workspace `Cargo.toml`).
It provides session path resolution and lifecycle management:
- `get_claude_storage_path()` — map dir path to `~/.claude/projects/{escaped}`
- `check_session_exists()` — .jsonl file presence check
- `Strategy` enum — Resume/Fresh session lifecycle intent
- `SessionManager` — topic-scoped session directory lifecycle

## Done When

- `module/claude_session/readme.md` exists with Responsibility Table
- `module/claude_session/spec.md` exists with Purpose, Architecture, Public API, Constraints sections
