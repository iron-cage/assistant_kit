# entrypoint

Classifies which wrapper or surface launched the current `claude` process.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_ENTRYPOINT` |
| Config Key | — |

### Type

enum (string)

### Default

unset — expected to be set by the launching wrapper, not by hand

### Since

≤v2.1.197 (undocumented — absent from both the official docs page and the
independent GitHub mirror; confirmed only via binary string/reference
inspection)

### Description

Read via `process.env.CLAUDE_CODE_ENTRYPOINT` and `switch`-mapped in the
installed binary to at least the following values: `"claude-vscode"`,
`"remote"`, `"remote_baku"`, `"remote_cowork"`, `"remote_desktop"`,
`"remote_mobile"`, `"claude-in-teams"`, `"sdk-cli"`, `"sdk-ts"` — mapped to
broader categories such as `"claude_code_vscode"` and `"claude_code_remote"`.

The documented `OTEL_METRICS_INCLUDE_ENTRYPOINT` setting ("include the
session entrypoint in metrics attributes") confirms "entrypoint" is a real,
stable internal concept — just not one exposed as a user-settable input in
its own right. In practice this is set by the launching wrapper (the VS Code
extension, a remote/mobile client, an SDK harness) before it execs `claude`,
not something a user or a tool-call subprocess would set directly.

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [132_claudecode.md](132_claudecode.md) | Broader spawned-subprocess marker (orthogonal axis) |
| doc | [133_child_session.md](133_child_session.md) | Nested-process marker (orthogonal axis) |
| doc | [032_ide.md](032_ide.md) | `--ide` — one concrete entrypoint this classifies |
