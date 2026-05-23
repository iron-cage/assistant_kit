# dry_run

Builder-only mode: return the full command description as stdout without spawning a process.

## Type

**Builder** — boolean flag (not a `claude` binary parameter)

## Syntax

This parameter has no CLI flag. It is a `ClaudeCommand` builder feature only.

## Default

false (process is spawned normally)

## Description

When `dry_run` is `true`, calling `execute()` returns `describe_compact()` as stdout with exit code 0 — no process is spawned. Calling `execute_interactive()` returns a success `ExitStatus` immediately.

Key use cases:
- **Inspect before run**: verify the exact `claude` invocation that would be generated
- **Testing**: unit-test command construction without spawning a real process
- **Logging**: capture the command line for audit trails or debug output
- **CI dry-runs**: validate command arguments without consuming API budget

The output of `execute()` in dry-run mode is identical to `describe_compact()` — one line containing the full CLI invocation (no `cd` prefix even when `working_directory` is set).

## Builder API

Use `with_dry_run()` to enable or disable dry-run mode:

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_message( "Hello" )
  .with_working_directory( "/tmp/work" )
  .with_dry_run( true );

// Returns "claude --message 'Hello'" as stdout — no process spawned
let output = cmd.execute()?;
println!( "{}", output.stdout );
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Related

- `describe_compact()` — returns the same one-line command string synchronously
- `describe()` — multi-line form including `cd /dir` prefix when working directory is set
- `describe_env()` — shows environment variables that would be set

## Notes

- `dry_run` does NOT affect `describe()` or `describe_env()` — those always return regardless of mode
- `execute()` in dry-run returns the compact form (no `cd` prefix), not the full multi-line `describe()` output
- On Windows, `execute_interactive()` dry-run runs a no-op `cmd /C` command to produce a valid `ExitStatus`
