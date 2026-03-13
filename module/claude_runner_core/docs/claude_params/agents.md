# agents

Define custom agents inline as a JSON object for the current session.

## Type

**CLI** — JSON string value

## Syntax

```
claude --agents '<json>'
```

## Default

None (only configured agents available)

## Description

Defines one or more custom agents as a JSON object, available for the current session without needing to modify config files. Each agent has a name, description, and system prompt.

JSON format:
```json
{
  "agent-name": {
    "description": "What this agent does",
    "prompt": "System prompt for this agent"
  }
}
```

After defining agents with `--agents`, select one using `--agent <name>`.

## Builder API

Use `with_agents()` — Accepts a single JSON string defining multiple agents.

```rust
use claude_runner_core::ClaudeCommand;

let json = r#"[{"name":"reviewer","model":"claude-opus-4-6"}]"#;
let cmd = ClaudeCommand::new()
  .with_agents( json )
  .with_message( "Use custom agent config" );
```

## Examples

```bash
# Define and use a custom agent
claude \
  --agents '{"tester":{"description":"Writes tests","prompt":"You write comprehensive test suites. Follow TDD principles."}}' \
  --agent tester \
  --print "Write tests for src/auth.rs"

# Multiple agents defined
claude \
  --agents '{"reviewer":{"description":"Reviews","prompt":"Be critical"},"fixer":{"description":"Fixes","prompt":"Be helpful"}}' \
  --agent reviewer \
  --print "Review this"
```

## Notes

- JSON must be valid and properly shell-quoted
- Use single quotes around the JSON to avoid shell interpolation issues
- Agent names defined here override any identically-named agents in config files
- Combine with `--agent <name>` to select which defined agent to use
