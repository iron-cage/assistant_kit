# Failure Mode: CLAUDECODE Env Var Inherited from Parent

### Scope

- **Purpose**: Document that spawning `claude` from within a Claude Code session silently inherits the `CLAUDECODE` env var, changing child behavior without any visible signal.
- **Responsibility**: Explain the leak mechanism, its behavioral consequences, and the mandatory mitigation.
- **In Scope**: `CLAUDECODE` env var inheritance, `unset_claudecode` default-on behavior, `env -u CLAUDECODE` prefix in trace/dry-run output.
- **Out of Scope**: Other env vars that affect claude behavior, `CLAUDE_CODE_*` env vars (those are intentionally configured).

### Behavior

Claude Code sets `CLAUDECODE=1` in its process environment. Any child process spawned without explicit env cleanup inherits this variable. The child `claude` instance then behaves as if it is running inside Claude Code, which can alter:

- Interactive mode detection
- Permission prompting behavior
- Session isolation defaults

The failure is **silent**: no warning is emitted, no exit-code change, no log entry. The caller gets different behavior depending on whether the parent process is Claude Code or a bare terminal — making the system non-deterministic across invocation contexts.

### Mechanism

```
Claude Code process (CLAUDECODE=1)
  └── spawn claude WITHOUT env cleanup
        └── child inherits CLAUDECODE=1
              └── child behaves as if running inside Claude Code  ← silent behavior change
```

### Mitigation

`ClaudeCommand` defaults `unset_claudecode: true`, which calls `cmd.env_remove("CLAUDECODE")` before spawning. This removes the variable from the child's environment regardless of whether the parent has it set:

```rust
// Effective invocation (shown in describe() / dry-run output):
// env -u CLAUDECODE claude --dangerously-skip-permissions ...
```

To disable (rare; requires explicit justification):
```rust
ClaudeCommand::new().with_unset_claudecode(false)
```

### clr Response

`clr` handles the primary protection correctly: `ClaudeCommand::new()` defaults `unset_claudecode: true`, which calls `cmd.env_remove("CLAUDECODE")` before spawning — removing the variable regardless of whether the parent has it set. Dry-run and trace output show the `env -u CLAUDECODE` prefix to reflect this removal (WYSIWYG invariant, BUG-246).

**Gap (BUG-248):** When the user passes `--keep-claudecode`, `clr` calls `builder.with_unset_claudecode(false)` — explicitly disabling the protection. No warning is emitted when this happens AND `CLAUDECODE` is currently set in the environment. A user may not realize they are handing down a live `CLAUDECODE=1` to the child process, changing its behavior silently. The warning should fire when: `cli.keep_claudecode == true && verbosity.shows_warnings() && env::var("CLAUDECODE").is_ok()`.

Fix location: `module/claude_runner/src/cli/mod.rs` — `run_built_command()` — add warning check before dispatching to print or interactive mode. See BUG-248.

### WYSIWYG Invariant (BUG-246)

`describe()` and `describe_compact()` must show the `env -u CLAUDECODE` prefix when `unset_claudecode` is true — otherwise the dry-run output diverges from what the subprocess actually executes. This is enforced by the describe() implementation and tested in `tests/describe_test.rs` and `tests/dry_run_test.rs`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [feature/006_unset_claudecode.md](../feature/006_unset_claudecode.md) | `unset_claudecode` field — design and default-on rationale |
| doc | [feature/003_describe.md](../feature/003_describe.md) | `describe()` WYSIWYG invariant — `env -u CLAUDECODE` prefix in output |
| source | `../../src/command/mod.rs` | `build_command()` → `env_remove("CLAUDECODE")`, `describe()` → `env -u CLAUDECODE` prefix |
| test | `../../tests/describe_test.rs` | Asserts `describe()` starts with `"env -u CLAUDECODE"` |
| test | `../../tests/dry_run_test.rs` | Asserts dry-run output starts with `"env -u CLAUDECODE"` |
| source | `../../../claude_runner/src/cli/mod.rs` | `run_built_command()` — missing warning when `--keep-claudecode` disables protection (BUG-248) |
| bug | BUG-246 | `describe()` showed `claude ...` without `env -u CLAUDECODE` prefix; WYSIWYG violated |
| bug | BUG-248 | `run_built_command()` emits no warning when `--keep-claudecode` is set and `CLAUDECODE` is in env |

### Sources

| File | Notes |
|------|-------|
| BUG-246 | Root cause and fix: describe() must mirror build_command() env manipulations |
| BUG-248 | Gap: no warning emitted when --keep-claudecode disables protection with CLAUDECODE in env |
| `docs/feature/006_unset_claudecode.md` | Design rationale for default-on unset behavior |
