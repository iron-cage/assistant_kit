# effort

Effort level for the current session, controlling reasoning depth vs speed tradeoff.

## Type

**CLI** — enum value

## Syntax

```
claude --effort <level>
```

## Values

| Level | Description |
|-------|-------------|
| `low` | Faster, less thorough — quick answers |
| `medium` | Balanced (default) |
| `high` | More thorough reasoning |
| `max` | Maximum effort — deepest reasoning |

## Default

`medium`

## Description

Controls how much reasoning effort Claude applies to the session. Higher effort levels use extended thinking and deeper analysis, but take longer and cost more tokens.

Use `low` for simple, repetitive tasks where speed matters. Use `max` for complex problems requiring deep analysis, security audits, or architectural decisions.

This maps to Claude's "thinking budget" or "extended thinking" feature — higher effort levels enable more thinking tokens.

## Builder API

Use `with_effort()` — Accepts an `EffortLevel` enum value (`Low`, `Medium`, `High`, `Max`).

```rust
use claude_runner_core::{ ClaudeCommand, EffortLevel };

let cmd = ClaudeCommand::new()
  .with_effort( EffortLevel::High )
  .with_message( "Audit this authentication implementation" );
```

## Examples

```bash
# Quick formatting check
claude --print --effort low "Fix indentation in this file"

# Deep code review
claude --print --effort max "Find all security vulnerabilities in src/"

# Architectural analysis
claude --print --effort high "Design a caching strategy for this API"
```

## Notes

- `max` effort can significantly increase response latency and token usage
- Effort interacts with `--max-budget-usd` — high effort + budget cap will stop early
- For automated pipelines processing many small tasks, `low` effort is most cost-efficient
