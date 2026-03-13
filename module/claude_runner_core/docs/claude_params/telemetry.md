# telemetry

Enable or disable usage telemetry data collection.

## Type

**Env** — environment variable

## Environment Variable

```
CLAUDE_CODE_TELEMETRY=<bool>
```

Values: `true` / `false`

## Default

`false` (in `claude_runner_core` builder)

Standard claude default: `true`

## Description

Controls whether Claude Code sends usage telemetry to Anthropic. Telemetry includes usage patterns, feature adoption metrics, and performance data.

The `claude_runner_core` builder defaults to `false` (telemetry disabled), unlike the standard `true`. Automation contexts typically:
- Process sensitive codebases that shouldn't generate telemetry
- Run in air-gapped or restricted network environments
- Execute at high frequency where telemetry overhead matters
- Require explicit data governance for what leaves the environment

Set to `true` to opt into telemetry in automation contexts where it's appropriate.

## Builder API

```rust
use claude_runner_core::ClaudeCommand;

// Default: telemetry disabled
let cmd = ClaudeCommand::new();

// Explicitly enable
let cmd = ClaudeCommand::new()
  .with_telemetry( true );
```

Builder method: `with_telemetry(telemetry: bool)` — sets `CLAUDE_CODE_TELEMETRY`.

## Examples

```bash
# Disable telemetry (default in builder)
CLAUDE_CODE_TELEMETRY=false claude --print "Process this private codebase"

# Enable telemetry
CLAUDE_CODE_TELEMETRY=true claude --print "Public project analysis"
```

## Notes

- `claude_runner_core` default (`false`) vs standard claude default (`true`): respects privacy in automation
- Telemetry is non-identifying usage data; it doesn't include the content of prompts or files
- For enterprise deployments, this may need to align with organizational data policies
