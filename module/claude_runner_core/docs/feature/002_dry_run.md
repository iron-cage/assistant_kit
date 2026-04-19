# Feature: Dry Run

### Scope

- **Purpose**: Document the dry-run mode that previews commands without spawning any process.
- **Responsibility**: Describe dry-run activation, execute() behavior under dry-run, describe_compact() output format, and the no-spawn guarantee.
- **In Scope**: with_dry_run(true) flag, execute() dry-run return value, execute_interactive() dry-run return value, describe_compact() format.
- **Out of Scope**: describe() and describe_env() inspection methods (→ `feature/003_describe.md`), normal execution flow (→ `feature/001_execution_control.md`).

### Design

When `with_dry_run(true)` is set on the builder, both execution methods behave as no-ops that return a preview of the command that would have been run:

**`execute()` in dry-run mode** returns `Ok(ExecutionOutput)` with:
- `stdout`: the output of `describe_compact()`
- `stderr`: empty string
- `exit_code`: 0

No process is spawned.

**`execute_interactive()` in dry-run mode** returns an exit status of 0 immediately. No process is spawned.

**`describe_compact()` output format:**

```
dir: /path/to/working/dir
cmd: claude --flags "message"
```

The label width is exactly 4 characters (`dir:`, `cmd:`), colon-terminated with a trailing space. The `cmd:` value is the last line of `describe()` — the `claude ...` invocation only, never the `cd` line. When `working_directory` is `None`, the `dir:` line is omitted entirely.

**Use cases:** Dry-run is useful for:
- Validating command construction before running expensive operations
- Logging the intended command for debugging
- Testing builder configurations in unit tests without spawning a process

**No-spawn guarantee:** Dry-run is enforced in the `execute()` implementation — the process-spawn code path is bypassed entirely. Callers cannot accidentally spawn a process when dry-run is enabled.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [api/001_execution_api.md](../api/001_execution_api.md) | execute() contract including dry-run return value |
| doc | [feature/001_execution_control.md](001_execution_control.md) | Normal execution modes that dry-run overrides |
| doc | [feature/003_describe.md](003_describe.md) | describe() and describe_compact() inspection output |
| source | `../../src/command.rs` | dry_run field, execute() dry-run branch, describe_compact() |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | FR-24, Vocabulary (Dry-run mode, describe_compact()), FR-21 escaping rules for describe() used in describe_compact() |
