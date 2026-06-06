# Feature: CLAUDECODE Environment Variable Unsetting

### Scope

- **Purpose**: Document the `unset_claudecode` field on `ClaudeCommand` that removes the `CLAUDECODE` environment variable from the subprocess environment before spawning.
- **Responsibility**: Describe the `with_unset_claudecode()` builder method, the default-on behaviour, why it exists, and when callers need to opt out.
- **In Scope**: `unset_claudecode: bool` field; `with_unset_claudecode()` builder method; `.env_remove("CLAUDECODE")` in `build_command()`; default `true`; interaction with nested invocation; opt-out semantics.
- **Out of Scope**: The `--keep-claudecode` flag in `clr` (→ `claude_runner` `--keep-claudecode`); the `CLAUDECODE` variable's meaning to the `claude` binary (→ Anthropic Claude Code documentation); credential isolation (→ `feature/004_run_isolated.md`).

### Design

When a `clr` process is launched from inside a Claude Code session, the parent session sets `CLAUDECODE=1` in its environment. A child `claude` subprocess that inherits this variable believes it is running nested inside Claude Code, which alters its behaviour (permission handling, output format, tool availability). For automation use-cases, the subprocess should behave as a first-class Claude Code process, not as a nested agent.

**Default-on:**

`unset_claudecode` defaults to `true`. Every `ClaudeCommand::new()` therefore removes `CLAUDECODE` from the subprocess environment via `.env_remove("CLAUDECODE")` inside `build_command()`. This is the correct default for automation: the subprocess runs as a standalone Claude Code session regardless of the host environment.

**Builder method:**

```rust
pub fn with_unset_claudecode(self, unset: bool) -> Self
```

Sets `unset_claudecode = unset`. Passing `false` preserves `CLAUDECODE` in the subprocess environment — the subprocess sees whatever the parent process had.

**When to opt in (keep `CLAUDECODE`):**

Callers that deliberately want the subprocess to operate in nested-agent mode should call `with_unset_claudecode(false)`. This is rare; the default covers virtually all automation use-cases.

**WYSIWYG in `describe()` output:**

`describe()` mirrors `build_command()`'s env manipulation so trace and dry-run output is WYSIWYG:

- When `unset_claudecode` is `true` (the default): `describe()` starts with `env -u CLAUDECODE claude ...`
- When `unset_claudecode` is `false`: `describe()` starts with plain `claude ...`

This is the WYSIWYG invariant: every `env_remove()` call in `build_command()` must appear in `describe()` at the same position. Before BUG-246, `describe()` always started with `"claude"` regardless of `unset_claudecode` — the env removal was invisible in trace/dry-run output.

**`run_isolated()` interaction:**

`run_isolated()` constructs its subprocess environment independently (it sets `HOME` to a temp directory and uses `Command::env_clear()` or selective inheritance). The `unset_claudecode` field on `ClaudeCommand` does not apply to `run_isolated()` — that function manages `CLAUDECODE` removal as part of its own environment construction.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [pattern/001_command_builder.md](../pattern/001_command_builder.md) | Builder method registration for `with_unset_claudecode()` |
| doc | [api/001_execution_api.md](../api/001_execution_api.md) | `execute()` contract — env modification effect |
| doc | [feature/003_describe.md](003_describe.md) | WYSIWYG invariant: describe() mirrors build_command() env manipulations |
| doc | [feature/004_run_isolated.md](004_run_isolated.md) | Isolated subprocess which manages its own env separately |
| source | `../../src/command.rs` | `with_unset_claudecode()`, `unset_claudecode` field, `build_command()` implementation |
