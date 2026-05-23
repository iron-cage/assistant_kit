# Feature: Stdin File Piping

### Scope

- **Purpose**: Document the `stdin_file` field on `ClaudeCommand` that pipes a file's content as standard input to the `claude` subprocess.
- **Responsibility**: Describe the `with_stdin_file()` builder method, its effect on `execute()` and `execute_interactive()`, how it interacts with dry-run mode, and the caller contract for file existence.
- **In Scope**: `stdin_file: Option<PathBuf>` field; `with_stdin_file()` builder method; `Stdio::from(file)` piping in `execute()`; dry-run behavior; `execute_interactive()` behavior; file-not-found error path.
- **Out of Scope**: How callers source the file path (→ `claude_runner` `--file` parameter); output fence stripping (→ `claude_runner` `--strip-fences`); other stdin modes.

### Design

`ClaudeCommand` supports an optional stdin file: when `with_stdin_file(path)` is set, the subprocess's standard input is connected to the opened file handle rather than to `/dev/null` (the default). This allows callers to pipe file content to `claude` without constructing shell pipelines.

**Builder method:**

```rust
pub fn with_stdin_file(self, path: PathBuf) -> Self
```

Sets `stdin_file = Some(path)`. Passing the same builder method twice replaces the previous value (last-write wins). Passing `None` is achieved by not calling the method (the default is `None`).

**Execution effect:**

When `stdin_file` is `Some(path)`:
1. The file at `path` is opened for reading at `execute()` call time.
2. On open failure, `execute()` returns `Err(...)` with a descriptive message including the path and OS error.
3. On success, the open file handle is passed to `Command::stdin(Stdio::from(file))` before spawning.
4. The subprocess reads from the file until EOF, then continues with any prompt it received via other arguments.

When `stdin_file` is `None` (default):
- The subprocess receives inherited or null stdin (no change from current behavior).

**Dry-run interaction:**

When `with_dry_run(true)` is set alongside `with_stdin_file(path)`:
- `execute()` returns the dry-run `describe_compact()` output as usual.
- No file is opened; the file is not checked for existence.
- The path is included in the describe output so callers can verify the intended configuration.

**Interactive mode:**

`execute_interactive()` with `stdin_file` set behaves the same as `execute()` for the stdin connection — the file is opened and attached. This enables non-interactive stdin input with a TTY-attached stdout/stderr session.

**Caller contract:**

- The caller is responsible for ensuring the file exists and is readable before calling `execute()`.
- Relative paths are resolved against the process working directory at `execute()` time (affected by `with_working_directory()`).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [pattern/001_command_builder.md](../pattern/001_command_builder.md) | Builder method registration for `with_stdin_file()` |
| doc | [api/001_execution_api.md](../api/001_execution_api.md) | `execute()` method contract — stdin effect documented there |
| doc | [feature/002_dry_run.md](002_dry_run.md) | Dry-run mode that suppresses file open in `execute()` |
| source | `../../src/command.rs` | `with_stdin_file()`, `stdin_file` field, `execute()` implementation |
