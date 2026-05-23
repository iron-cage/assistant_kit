# append_system_prompt

Append additional instructions to the default system prompt.

## Type

**CLI** — string value

## Syntax

```
claude --append-system-prompt "<prompt>"
```

## Default

None

## Description

Appends the given text to Claude Code's default system prompt without replacing it. The default prompt's coding assistant behavior, tool instructions, and guidelines are preserved — the appended text is added at the end.

Use this to inject project-specific context, additional constraints, or persona tweaks while keeping Claude's standard capabilities intact.

Compare with `--system-prompt` which replaces the default entirely.

## Builder API

Use `with_append_system_prompt()` — Accepts a string appended to the default system prompt.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_append_system_prompt( "Always cite line numbers when referencing code." )
  .with_message( "Review this code" );
```

## Examples

```bash
# Add project context
claude --append-system-prompt "This project uses Rust 2021 edition with no_std. Avoid stdlib." \
  --print "How should I handle errors?"

# Add output constraints
claude --append-system-prompt "Never use markdown. Plain text only." \
  --print "Summarize this module"

# Add persona
claude --append-system-prompt "You are reviewing code for OWASP Top 10 vulnerabilities." \
  --print "Review src/auth.rs"
```

## Notes

- Appended text appears after the default prompt; exact position may vary across claude versions
- Useful for per-project instructions without needing a custom system prompt file
- Does not persist; only applies to this invocation
