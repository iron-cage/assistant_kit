# Subcommand: agents

Manage background agents.

### Usage

```
claude agents [options]
```

### Options

Corrected against `claude agents --help` on v2.1.220 — an earlier revision of
this doc described a "list configured agents" command with 2 flags; the real
surface is the "agent view" for background/dispatched sessions, with 17:

| Flag | Description |
|------|-------------|
| `--add-dir <directory>` | Additional directory to allow tool access to in dispatched sessions (repeatable) |
| `--agent <agent>` | Default agent for sessions dispatched from agent view. Overrides the `agent` setting |
| `--all` | With `--json`: also include completed background sessions |
| `--allow-dangerously-skip-permissions` | Make bypass-permissions mode available to dispatched sessions without defaulting to it |
| `--cwd <path>` | Show only background sessions started under `<path>` |
| `--dangerously-skip-permissions` | Alias for `--permission-mode bypassPermissions` |
| `--effort <level>` | Default effort level for sessions dispatched from agent view |
| `-h`, `--help` | Display help |
| `--json` | Print active sessions (interactive and background) as a JSON array and exit (for scripting; does not require a TTY) |
| `--mcp-config <config>` | MCP server configuration to apply to dispatched sessions (repeatable) |
| `--model <model>` | Default model for sessions dispatched from agent view |
| `--permission-mode <mode>` | Default permission mode for sessions dispatched from agent view |
| `--plugin-dir <path>` | Load plugins from specified directory for the agent view and dispatched sessions (repeatable) |
| `--setting-sources <sources>` | Comma-separated list of setting sources to load (`user`, `project`, `local`) |
| `--settings <file-or-json>` | Settings file or JSON string to apply to the agent view and dispatched sessions |
| `--strict-mcp-config` | Only use MCP servers from `--mcp-config` in dispatched sessions |

### Sub-subcommands

None — a flat options surface, no `Commands:` section in `--help`.

### Description

This is the "agent view": the CLI front-end for listing and configuring
background/dispatched Claude Code sessions, not a lister of subagent-type
definitions (that's what the confusingly similarly-named global `--agent`/
`--agents` flags on the default session do — see `../param/003_agent.md`,
`../param/004_agents.md`). `--json` prints active sessions for scripting;
`--all`, `--cwd` filter what's shown. The remaining flags (`--model`,
`--effort`, `--permission-mode`, `--mcp-config`, `--plugin-dir`, `--settings`,
`--add-dir`, `--*-skip-permissions`) set defaults applied to sessions dispatched
*from* agent view — the same family of concerns as `013_attach.md`,
`014_daemon.md`, `016_logs.md`, `017_respawn.md`, `018_rm.md`, `019_stop.md`.

### Since

v1.0.60 (2025-07-24); the full options surface above dates to the "agent view"
enhancement in v2.1.139 — the pre-v2.1.139 surface is unverified.

### Verification

```bash
claude agents --help    # → 17 options, "Manage background agents"
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master subcommand table |
| doc | [013_attach.md](013_attach.md) | Attach to a background session listed here |
| doc | [014_daemon.md](014_daemon.md) | Background-session supervisor this view reads from |
| doc | [018_rm.md](018_rm.md) | Delete a background session listed here |
| doc | [../param/003_agent.md](../param/003_agent.md) | Unrelated global `--agent` flag (default session, not agent view) |
| doc | [../param/004_agents.md](../param/004_agents.md) | Unrelated global `--agents` JSON flag (default session, not agent view) |
