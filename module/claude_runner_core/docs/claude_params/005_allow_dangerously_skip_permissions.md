# allow_dangerously_skip_permissions

Enable bypassing all permission checks as an opt-in option, without it being the default behavior.

## Type

**CLI** — boolean flag

## Syntax

```
claude --allow-dangerously-skip-permissions
```

## Default

off

## Description

Makes skip-permissions available as a capability Claude can utilize during the session, but does not force it for every tool call. This is a softer version of `--dangerously-skip-permissions`.

In practice, with `--allow-dangerously-skip-permissions`, Claude may skip permission prompts when it determines they are unnecessary, while still prompting for genuinely sensitive operations.

Recommended for sandboxed environments where you want automation-friendly behavior without fully disabling all safety checks.

Compare with `--dangerously-skip-permissions` which bypasses permissions unconditionally for all operations.

## Builder API

Use `with_allow_dangerously_skip_permissions()` — Boolean flag.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_allow_dangerously_skip_permissions( true )
  .with_message( "Run with permissions option enabled" );
```

## Examples

```bash
# Sandbox environment: allow skip but don't force it
claude --allow-dangerously-skip-permissions --print "Update all test files"

# Combined with permission mode for fine-grained control
claude --allow-dangerously-skip-permissions --permission-mode acceptEdits \
  --print "Apply these formatting changes"
```

## Notes

- Less aggressive than `--dangerously-skip-permissions`
- Both flags are only recommended for sandboxes with no internet access
- Official claude docs describe this as enabling the bypass "as an option" vs the unconditional bypass
