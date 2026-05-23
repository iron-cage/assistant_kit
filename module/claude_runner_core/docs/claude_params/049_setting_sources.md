# setting_sources

Control which setting sources are loaded for the session.

## Type

**CLI** — comma-separated list

## Syntax

```
claude --setting-sources <sources>
```

## Values

| Source | Description |
|--------|-------------|
| `user` | User-level settings from `~/.claude/settings.json` |
| `project` | Project-level settings from `.claude/settings.json` |
| `local` | Local settings from `.claude/settings.local.json` |

## Default

All sources (`user,project,local`)

## Description

By default, Claude Code loads settings from three hierarchical sources. This flag restricts which sources are loaded, allowing precise control over the configuration context.

Use cases:
- CI environments where project settings should be the only source (ignore user-level customizations)
- Testing with only local settings to avoid project config interference
- Reproducible builds where the exact settings must be controlled

Multiple sources can be combined as a comma-separated list.

## Builder API

Use `with_setting_sources()` — Accepts a comma-separated source filter string.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_setting_sources( "global,project" )
  .with_message( "Use only global and project settings" );
```

## Examples

```bash
# Only project settings (ignore user customizations in CI)
claude --setting-sources project --print "Run CI task"

# User + project, but not local
claude --setting-sources "user,project" --print "Standard session"

# Only user settings (ignore project config)
claude --setting-sources user --print "Personal session"
```

## Notes

- `--settings` flag adds an additional settings source on top of what `--setting-sources` loads
- Order of precedence within loaded sources: local > project > user
- Using `--setting-sources project` in CI avoids developer `~/.claude/settings.json` leaking in
