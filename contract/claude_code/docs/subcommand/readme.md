# Claude Code: Subcommands

All subcommands exposed by the `claude` binary beyond the default interactive/print session mode.

### Scope

- **Purpose**: Authoritative reference for every subcommand the `claude` binary provides.
- **Responsibility**: Master table and per-subcommand detail files.
- **In Scope**: All 9 subcommands — agents, auth, auto-mode, doctor, install, mcp, plugin, setup-token, update.
- **Out of Scope**: The default session mode (→ `../params/`); builder-API (→ `module/claude_runner_core/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| readme.md | Master subcommand table (this file) |
| 001_agents.md | `claude agents` — list configured agents |
| 002_auth.md | `claude auth` — manage authentication |
| 003_auto_mode.md | `claude auto-mode` — inspect auto mode classifier |
| 004_doctor.md | `claude doctor` — check auto-updater health |
| 005_install.md | `claude install` — install native build |
| 006_mcp.md | `claude mcp` — configure MCP servers |
| 007_plugin.md | `claude plugin` — manage plugins |
| 008_setup_token.md | `claude setup-token` — set up auth token |
| 009_update.md | `claude update` — check for and install updates |

### Subcommand Table

| # | Subcommand | Sub-subcommands | Since | Description |
|---|------------|-----------------|-------|-------------|
| 1 | [agents](001_agents.md) | — | v1.0.60 | List configured agents with optional `--setting-sources` filter |
| 2 | [auth](002_auth.md) | `login`, `logout`, `status` | pre-v1.0 | Manage authentication — sign in, sign out, show status |
| 3 | [auto-mode](003_auto_mode.md) | `config`, `defaults` | v2.1.158 | Inspect auto mode classifier configuration and defaults |
| 4 | [doctor](004_doctor.md) | — | v2.0.12 | Check health of Claude Code auto-updater |
| 5 | [install](005_install.md) | — | pre-v1.0 | Install Claude Code native build (`stable`, `latest`, or specific version) |
| 6 | [mcp](006_mcp.md) | `add`, `add-from-claude-desktop`, `add-json`, `get`, `list`, `remove`, `reset-project-choices`, `serve` | pre-v1.0 | Configure and manage MCP servers |
| 7 | [plugin](007_plugin.md) | `install`, `uninstall`, `update`, `enable`, `disable`, `list`, `marketplace`, `validate` | v2.0.12 | Manage Claude Code plugins |
| 8 | [setup-token](008_setup_token.md) | — | pre-v1.0 | Set up a long-lived authentication token (requires Claude subscription) |
| 9 | [update](009_update.md) | — | pre-v1.0 | Check for updates and install if available |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [`../params/readme.md`](../params/readme.md) | CLI parameter specifications |
| doc | [`../tool/readme.md`](../tool/readme.md) | Built-in tools available in sessions |
