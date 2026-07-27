# Behavior S8: Session Identity Is Controlled by Typed `Options` Fields Mapping 1:1 to CLI Flags

### Scope

- **Purpose**: Document that the SDK exposes session continuation/identity as typed `Options` struct fields, and that each field corresponds to an already-documented `claude_code` CLI flag.
- **Responsibility**: Authoritative instance for behavior S8.
- **In Scope**: `resume`, `sessionId`, `continue`, `forkSession` fields and their CLI-flag counterparts.
- **Out of Scope**: Per-field type/default detail (→ [`../param/`](../param/readme.md) instances 007–010); the CLI flags themselves, already fully documented in the sibling crate.

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 90% | **Since**: SDK GA | **Evidence**: E2

Four `Options` fields map directly onto CLI flags `contract/claude_code` already documents as behaviors B4, B19, B20, B21:

| `Options` field | Type | CLI equivalent | `claude_code` behavior |
|---|---|---|---|
| `continue` | `boolean` (default `false`) | `-c` / `--continue` | [B4](../../../claude_code/docs/behavior/004_b4_continue_flag.md) |
| `resume` | `string` (default `undefined`) | `-r` / `--resume <session-id>` | [B19](../../../claude_code/docs/behavior/019_b19_resume_flag.md) |
| `sessionId` | `string` (default: auto-generated) | `--session-id <uuid>` | [B20](../../../claude_code/docs/behavior/020_b20_session_id_flag.md) |
| `forkSession` | `boolean` (default `false`) | `--fork-session` | [B21](../../../claude_code/docs/behavior/021_b21_fork_session.md) |

This is a direct, field-for-flag re-exposure rather than a redesign — the SDK's session sample code (official "Sessions" example) captures `session_id`/`sessionId` from the first `query()`'s `SystemMessage`/`"system"` init message, then passes it back as `options.resume` on a second `query()` call to continue with full context, which is the exact same "list a session ID, then feed it back in on the next invocation" shape `claude_runner_core` already implements for the CLI (`--session-id` then `-c`/`-r`). No new session-continuation *semantics* were found in the SDK beyond what the CLI flags already provide — only a typed-struct surface instead of argv strings, which is friendlier to construct programmatically but not more expressive.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E2 | S8 | Doc | `https://code.claude.com/docs/en/agent-sdk/typescript` | `Options` interface; official "Sessions" tab example | `continue`, `resume`, `sessionId`, `forkSession` field definitions; session-capture-then-resume code sample |

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| entity | [readme.md](readme.md) | Master index |
| param | [../param/007_resume.md](../param/007_resume.md) | `resume` field detail |
| param | [../param/008_session_id.md](../param/008_session_id.md) | `sessionId` field detail |
| param | [../param/009_continue.md](../param/009_continue.md) | `continue` field detail |
| param | [../param/010_fork_session.md](../param/010_fork_session.md) | `forkSession` field detail |
| doc | [`../../../claude_code/docs/behavior/004_b4_continue_flag.md`](../../../claude_code/docs/behavior/004_b4_continue_flag.md) | CLI-level counterpart to `continue` |
| doc | [`../../../claude_code/docs/behavior/019_b19_resume_flag.md`](../../../claude_code/docs/behavior/019_b19_resume_flag.md) | CLI-level counterpart to `resume` |
| doc | [`../../../claude_code/docs/behavior/020_b20_session_id_flag.md`](../../../claude_code/docs/behavior/020_b20_session_id_flag.md) | CLI-level counterpart to `sessionId` |
| doc | [`../../../claude_code/docs/behavior/021_b21_fork_session.md`](../../../claude_code/docs/behavior/021_b21_fork_session.md) | CLI-level counterpart to `forkSession` |
