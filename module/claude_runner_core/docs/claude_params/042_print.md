# print

Non-interactive mode: print response and exit. Essential for pipes and scripting.

## Type

**CLI** — boolean flag

## Syntax

```
claude --print [options] <prompt>
claude -p [options] <prompt>
```

## Default

off (interactive mode by default)

## Description

When set, Claude processes the prompt, prints the response to stdout, and exits with code 0. No TTY is allocated; stdin/stdout are suitable for pipes.

Key behaviors in `--print` mode:
- Workspace trust dialog is **skipped** — only use in trusted directories
- `--output-format`, `--fallback-model`, `--max-budget-usd`, `--no-session-persistence`, and streaming options only work with `--print`
- Exit code reflects success/failure of the Claude execution

## Builder API

Use `with_print()` — Boolean flag: when `true`, adds `-p` / `--print` to the CLI invocation.

```rust
use claude_runner_core::ClaudeCommand;

// Explicit --print flag in the command line
let cmd = ClaudeCommand::new()
  .with_print( true )
  .with_message( "Hello" );

// Note: execute() already behaves like --print by capturing stdout.
// with_print() explicitly passes the flag to the claude binary.
let output = cmd.execute()?;
println!( "{}", output.stdout );
```

## Examples

```bash
# Basic non-interactive usage
claude --print "What is 2+2?"

# Pipe output to another command
claude -p "Generate a JSON schema for a user" | jq .

# Stream JSON output
claude --print --output-format stream-json "Analyze this"
```

## Notes

- Security: the trust dialog skip means `--print` should only be used in directories you own/trust
- `execute()` in the builder API behaves like `--print` — captures stdout/stderr without TTY
- `execute_interactive()` is the opposite: attaches TTY, equivalent to running `claude` with no `--print`
