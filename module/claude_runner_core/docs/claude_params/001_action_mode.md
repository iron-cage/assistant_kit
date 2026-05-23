# action_mode

Action mode for tool execution — controls how Claude handles tool approval.

## Type

**Env** — environment variable

## Environment Variable

```
CLAUDE_CODE_ACTION_MODE=<mode>
```

## Values

| Value | Description |
|-------|-------------|
| `Ask` | Ask for permission on sensitive operations (default) |
| `Allow` | Allow all operations without asking |
| `Deny` | Deny all tool operations |

## Default

`Ask` (inherits standard)

## Description

Controls the default behavior when Claude Code encounters a tool call that requires approval. This maps to the `ActionMode` enum in the `claude_runner_core` type system.

- **`Ask`**: The standard interactive mode — Claude prompts before executing potentially destructive operations.
- **`Allow`**: All tools execute without approval. Similar to `auto_approve_tools=true`.
- **`Deny`**: All tool calls are denied. Claude can only respond with text.

In automation contexts, `Allow` provides the most autonomous behavior. `Deny` is useful for text-only sessions where tool execution should be prevented.

## Builder API

```rust
use claude_runner_core::{ ClaudeCommand, ActionMode };

// Default: Ask
let cmd = ClaudeCommand::new();

// Allow all tools
let cmd = ClaudeCommand::new()
  .with_action_mode( ActionMode::Allow );

// Deny all tools (text only)
let cmd = ClaudeCommand::new()
  .with_action_mode( ActionMode::Deny );
```

Builder method: `with_action_mode(mode: ActionMode)` — sets `CLAUDE_CODE_ACTION_MODE` to the mode's string representation.

## Examples

```bash
# Allow all tools for automation
CLAUDE_CODE_ACTION_MODE=Allow claude --print "Refactor these files"

# Deny all tools for text-only analysis
CLAUDE_CODE_ACTION_MODE=Deny claude --print "Explain this algorithm"
```

## Notes

- Case-sensitive: must be exactly `Ask`, `Allow`, or `Deny` (capital first letter)
- The `Deny` mode is essentially `--tools ""` but via environment variable
- `Allow` + `CLAUDE_CODE_AUTO_APPROVE_TOOLS=true` achieves the same effect in different ways; prefer one approach for clarity
