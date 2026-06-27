# Subcommand: agents

List configured agents.

### Usage

```
claude agents [options]
```

### Options

| Flag | Description |
|------|-------------|
| `--setting-sources <sources>` | Comma-separated list of setting sources to load (`user`, `project`, `local`) |
| `-h`, `--help` | Display help |

### Sub-subcommands

None.

### Description

Lists all agents configured for the current session context. Agents can be
defined via `--agents` JSON flag, `~/.claude/agents/` directory, or project-level
settings. The `--setting-sources` filter controls which configuration layers are
consulted.

### Since

v1.0.60 (2025-07-24); enhanced with agent view in v2.1.139

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master subcommand table |
| doc | [../params/003_agent.md](../params/003_agent.md) | `--agent` override parameter |
| doc | [../params/004_agents.md](../params/004_agents.md) | `--agents` JSON definitions |
| doc | [../tool/007_agent.md](../tool/007_agent.md) | Agent tool that uses these agents |
