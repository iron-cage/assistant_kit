# Feature: Describe

### Scope

- **Purpose**: Document the human-readable command inspection methods for debugging and dry-run output.
- **Responsibility**: Describe describe(), describe_env(), and describe_compact() output formats, escaping rules, and intended use.
- **In Scope**: describe() full command string, describe_env() environment variable listing, describe_compact() aligned block, escaping rules for message content.
- **Out of Scope**: Dry-run execution behavior using describe_compact() (→ `feature/002_dry_run.md`), execution methods (→ `api/001_execution_api.md`).

### Design

claude_runner_core provides three inspection methods for examining the command that would be run, without executing it:

**`describe() -> String`**

Returns a human-readable multi-line string showing the working directory change and the full command invocation:

```
cd /path/to/working/dir
claude --max-output-tokens 200000 --continue "the message"
```

The `cd` line appears only when `working_directory` is set. The second line is the `claude` binary followed by all flags and arguments in construction order.

**Escaping rules for the message field in `describe()` output:**
- Double-quotes (`"`) are escaped to `\"` for shell correctness
- Backslashes (`\`) are escaped to `\\` before quote escaping (prevents `\"` producing malformed output)
- Single quotes, `$`, newlines, and other characters are NOT escaped (human-readable only, not shell-safe)
- Working directory in the `cd` line is NOT quoted (human-readable only; actual execution uses `cmd.current_dir()` which is OS-safe)

Messages containing literal newlines embed the newline literally in `describe()` output. This is expected behavior for a human-readable representation.

**`describe_env() -> String`**

Returns `NAME=VALUE` lines for all configured environment variables that would be set on the spawned process. Float values are formatted via Rust `Display` (`to_string()`): `1.0` → `"1"`, `0.0` → `"0"` (integer representation, no decimal point for whole numbers). `describe_env()` and `build_command()` use the same formatting so they are consistent.

**`describe_compact() -> String`**

Returns a compact aligned block for concise display:

```
dir: /path/to/working/dir
cmd: claude --flags "message"
```

Label width is 4 characters (`dir:`, `cmd:`), colon-terminated with a trailing space. The `cmd:` value is the last line of `describe()` (the `claude ...` line only). When `working_directory` is `None`, the `dir:` line is omitted.

`describe_compact()` is used by dry-run mode (`with_dry_run(true)`) as the stdout value returned by `execute()`.

**Intended use:** These methods are for debugging, logging, test assertions, and dry-run preview. They are not guaranteed to produce shell-safe output for all inputs — they produce human-readable output only.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [feature/002_dry_run.md](002_dry_run.md) | Dry-run mode that uses describe_compact() as execute() output |
| doc | [api/001_execution_api.md](../api/001_execution_api.md) | execute() contract that returns describe_compact() in dry-run |
| source | `../../src/command.rs` | describe(), describe_env(), describe_compact() implementation |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | FR-21 (describe() escaping rules), FR-22 (describe_env() float formatting), FR-24 (describe_compact() format), Vocabulary (describe_compact) |
