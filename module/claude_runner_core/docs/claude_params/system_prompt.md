# system_prompt

Replace the default system prompt for the session.

## Type

**CLI** — string value

## Syntax

```
claude --system-prompt "<prompt>"
```

## Default

None (uses Claude Code's built-in default system prompt)

## Description

Completely replaces the default system prompt Claude Code uses for the session. The default system prompt defines Claude's role as a coding assistant, permitted tools, and behavior guidelines. Using `--system-prompt` overrides it entirely with the provided text.

To keep the default prompt and add to it, use `--append-system-prompt` instead.

## Builder API

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_system_prompt( "You are a strict code reviewer. Only respond with issues found." );
```

Builder method: `with_system_prompt(prompt: impl Into<String>)` — adds `--system-prompt <value>` to CLI args.

## Examples

```bash
# Custom role
claude --system-prompt "You are a senior Rust engineer. Be terse and precise." \
  --print "Review this implementation"

# Minimal prompt for controlled output
claude --print \
  --system-prompt "Respond only with valid JSON. No explanation." \
  "List the top 3 sorting algorithms"
```

## Notes

- Completely replaces the default prompt — Claude loses its built-in coding assistant persona and tool instructions
- For adding context without losing defaults, use `--append-system-prompt`
- Does not persist across sessions; applies only to this invocation
- Long prompts should be passed via a file: `--system-prompt "$(cat system.txt)"`
