# settings

Load additional settings from a file path or inline JSON string.

## Type

**CLI** — string value (file path or JSON)

## Syntax

```
claude --settings <file-path>
claude --settings '<json-string>'
```

## Default

None (only standard config hierarchy applies)

## Description

Loads additional settings for the session from either a JSON file or an inline JSON string. These settings are merged with the standard config hierarchy (user, project, local settings).

The JSON structure matches Claude Code's settings format (`~/.claude/settings.json`).

Use cases:
- Per-invocation settings overrides without modifying config files
- Loading environment-specific settings (dev vs prod)
- Scripted sessions with non-standard configurations
- CI/CD pipelines with custom tool permissions

## Builder API

Use `with_settings()` — Accepts a file path or inline JSON string.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_settings( "/home/user/.claude/settings.json" )
  .with_message( "Load custom settings" );
```

## Examples

```bash
# Load from file
claude --settings ~/.config/claude/ci-settings.json --print "CI task"

# Inline JSON
claude --settings '{"permissions":{"allow":["Bash","Read"]}}' \
  --print "Limited tools session"

# Environment-specific settings
claude --settings "$(cat environments/prod.settings.json)" --print "Production check"
```

## Notes

- Inline JSON must be valid and properly shell-quoted
- Settings are merged with the config hierarchy; see `--setting-sources` to control what's loaded
- The exact settings schema matches `~/.claude/settings.json`
