# Feature: Stdin File Piping

### Scope

- **Purpose**: Document the `stdin_file` and `stdin_content` fields on `ClaudeCommand` that pipe content as standard input to the `claude` subprocess.
- **Responsibility**: Describe the `with_stdin_file()`/`with_stdin_content()` builder methods, their effect across all four spawn methods, how they interact with dry-run mode, and the caller contract for file existence.
- **In Scope**: `stdin_file: Option<PathBuf>` field; `stdin_content: Option<Vec<u8>>` field; `with_stdin_file()`/`with_stdin_content()` builder methods; `Stdio::from(file)` piping in `execute()`, `execute_interactive()`, `spawn_piped()`, and `spawn_tty()`; dry-run behavior; file-not-found error path; `stdin_file`-over-`stdin_content` precedence.
- **Out of Scope**: How callers source the file path or raw bytes (→ `claude_runner` `--file` parameter and plain-pipe stdin forwarding, `feature/004_json_config.md`); output fence stripping (→ `claude_runner` `--strip-fences`); `spawn_control_session()` (SDK protocol, does not take piped stdin content).

### Design

`ClaudeCommand` supports an optional stdin file: when `with_stdin_file(path)` is set, the subprocess's standard input is connected to the opened file handle rather than to `/dev/null` (the default). This allows callers to pipe file content to `claude` without constructing shell pipelines.

**Builder method:**

```rust
pub fn with_stdin_file(self, path: PathBuf) -> Self
```

Sets `stdin_file = Some(path)`. Passing the same builder method twice replaces the previous value (last-write wins). Passing `None` is achieved by not calling the method (the default is `None`).

**Execution effect:**

When `stdin_file` is `Some(path)`:
1. The file at `path` is opened for reading at execution time (spawn time — not builder time, for any of the four spawn methods below).
2. On open failure, the spawn method returns `Err(...)` with a descriptive message including the path and OS error.
3. On success, the open file handle is passed to `Command::stdin(Stdio::from(file))` before spawning.
4. The subprocess reads from the file until EOF, then continues with any prompt it received via other arguments.

When `stdin_file` is `None` (default):
- The subprocess receives inherited or null stdin (no change from current behavior), unless `stdin_content` is set (see below).

**Dry-run interaction:**

When `with_dry_run(true)` is set alongside `with_stdin_file(path)`:
- `execute()` returns the dry-run `describe_compact()` output as usual.
- No file is opened; the file is not checked for existence.
- The path is included in the describe output so callers can verify the intended configuration.

**Interactive mode:**

`execute_interactive()` with `stdin_file` set behaves the same as `execute()` for the stdin connection — the file is opened and attached. This enables non-interactive stdin input with a TTY-attached stdout/stderr session.

**All four spawn methods honor `stdin_file` and `stdin_content`:** `execute()`, `execute_interactive()`, `spawn_piped()` (used for `stream-json` incremental consumption), and `spawn_tty()` (TTY passthrough) each independently check `stdin_file` first, then `stdin_content`, at their own spawn point — attaching `Stdio::from(file)` (opened path, or materialized tempfile) whenever either is set. `spawn_piped()` and `spawn_tty()` fall back to `Stdio::null()` only when neither field is set.

**Raw content variant — `stdin_content`:**

```rust
pub fn with_stdin_content(self, content: Vec<u8>) -> Self
```

Sets `stdin_content = Some(content)` — raw bytes to pipe to the subprocess's stdin, for callers that already hold the content in memory (e.g. piped stdin bytes read by `claude_runner`) rather than a filesystem path. When set and `stdin_file` is absent, each spawn method materializes the bytes into an anonymous, already-unlinked temp file (`tempfile::tempfile()` — OS-reclaimed on last-fd-close, robust against this codebase's frequent `std::process::exit()` calls which bypass `Drop`) and attaches it via `Stdio::from(file)`, identically to the `stdin_file` path. **`stdin_file` always takes priority over `stdin_content` when both are set** — the same last-write-wins semantics do not apply across the two fields; `stdin_file` wins unconditionally regardless of call order.

**Caller contract:**

- The caller is responsible for ensuring the file exists and is readable before calling a spawn method (`stdin_file`) — no such contract applies to `stdin_content`, which is caller-supplied bytes with no filesystem existence to validate.
- Relative paths are resolved against the process working directory at spawn time (affected by `with_working_directory()`).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [pattern/001_command_builder.md](../pattern/001_command_builder.md) | Builder method registration for `with_stdin_file()`/`with_stdin_content()` |
| doc | [api/001_execution_api.md](../api/001_execution_api.md) | `execute()` method contract — stdin effect documented there |
| doc | [feature/002_dry_run.md](002_dry_run.md) | Dry-run mode that suppresses file open in `execute()` |
| doc | [../../../claude_runner/docs/feature/004_json_config.md](../../../claude_runner/docs/feature/004_json_config.md) | `claude_runner` CLI layer that sources `stdin_content` from piped stdin |
| source | `../../src/command/mod.rs` | `stdin_file`/`stdin_content` fields, `execute()`/`execute_interactive()`/`spawn_piped()`/`spawn_tty()` implementations |
| source | `../../src/command/params_core.rs` | `with_stdin_file()`, `with_stdin_content()` builder methods |
