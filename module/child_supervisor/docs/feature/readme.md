# Feature Doc Entity

### Scope

- **Purpose**: Document user-facing capabilities of the `child_supervisor` library for callers hosting a long-lived PTY child.
- **Responsibility**: Index of feature doc instances covering the hosted-session table and drained, cursor-addressed session output.
- **In Scope**: `SessionTable` and `HostedSession`, `OutputPump` and `OutputSlice`.
- **Out of Scope**: Terminal mechanics (→ `claude_pty_core/docs/feature/`), the daemon that composes this crate and the wire protocol it builds on top (→ `claude_daemon_core/docs/`), invariant constraints (shared with `claude_daemon_core` — this crate has none of its own).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Session Table](001_session_table.md) | Hosted sessions, addressed by a caller-chosen id | ✅ |
| 002 | [Session Output](002_session_output.md) | Output kept drained, retained in bounds, read by cursor | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating feature doc instances | ✅ |

**Status:** ✅ implemented · 🔄 in progress · 📋 specified, not yet built.

**These IDs are this crate's own, starting fresh at 001.** The doc instances
migrated here from `claude_daemon_core`, where they were IDs 003 and 004 — see
[`claude_daemon_core/docs/feature/readme.md`](../../../claude_daemon_core/docs/feature/readme.md)
for the retirement note at those numbers.
