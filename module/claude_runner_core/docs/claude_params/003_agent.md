# agent

Override the agent used for the current session.

## Type

**CLI** — string value

## Syntax

```
claude --agent <agent-name>
```

## Default

Default agent (as configured in settings)

## Description

Selects a named agent configuration for the session, overriding the default agent setting. Agents define Claude's behavior, tools, and persona for specific use cases.

Agents are defined via:
- Global config (`~/.claude/settings.json` `agents` section)
- `--agents` flag for per-invocation custom agents

The agent name must correspond to a configured agent. Use `claude agents` to list available agents.

## Builder API

Use `with_agent()` — Accepts a string agent name.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_agent( "code-review" )
  .with_message( "Review this PR" );
```

## Examples

```bash
# Use a configured "reviewer" agent
claude --agent reviewer --print "Review this PR"

# List available agents
claude agents

# Use agent with custom prompt
claude --agent security-auditor --print "Check for SQL injection"
```

## Notes

- See `--agents` for defining custom agents inline per-invocation
- Agents can have custom system prompts, tool restrictions, and model settings
- If the specified agent doesn't exist, claude will error
