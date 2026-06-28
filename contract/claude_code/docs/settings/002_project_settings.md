# Settings: Project Settings

### Scope

- **Purpose**: Document `.claude/settings.json` and `.claude/settings.local.json` — project-level configuration managed by Claude Code.
- **Responsibility**: Authoritative instance for project-level settings — file locations, structure, auto-management semantics, and keys valid at project scope.
- **In Scope**: `.claude/settings.json` structure; `.claude/settings.local.json` structure; permissions object; auto-management by Claude Code during sessions.
- **Out of Scope**: Global user settings (→ [001_global_settings.md](001_global_settings.md)); version lock (→ [003_version_lock.md](003_version_lock.md)).

### Structure

`.claude/settings.local.json` (auto-managed by Claude Code):

```json
{
  "permissions": {
    "allow": [
      "Bash(npm run test:*)",
      "Bash(npx eslint:*)"
    ],
    "deny": [],
    "ask": []
  },
  "outputStyle": "default"
}
```

`.claude/settings.json` (user-managed project config):

```json
{
  "model": "sonnet",
  "effortLevel": "high",
  "hooks": {
    "PreToolUse": [
      { "matcher": "Bash", "hooks": [{ "type": "command", "command": "./hooks/pre-bash.sh" }] }
    ]
  },
  "mcpServers": {
    "my-server": { "command": "node", "args": ["./mcp-server.js"] }
  },
  "allowedTools": ["Bash", "Read", "Write", "Edit"],
  "permissionMode": "acceptEdits"
}
```

### Project-Level Keys

| Key | Scope | Type | Default | Description |
|-----|-------|------|---------|-------------|
| `model` | G+P | string | binary default | Persistent model preference |
| `effortLevel` | G+P | enum | `"medium"` | Effort level: `low`/`medium`/`high`/`max` |
| `hooks` | G+P | object | `{}` | Hooks for tool use events |
| `mcpServers` | G+P | object | `{}` | Inline MCP server definitions |
| `permissionMode` | G+P | enum | `"default"` | Permission mode |
| `allowedTools` | G+P | string[] | all | Persistent tool allowlist |
| `disallowedTools` | G+P | string[] | none | Persistent tool denylist |
| `outputStyle` | G+P | string | `"default"` | Terminal output rendering style |
| `permissions` | P only | object | `{}` | Per-project tool allow/deny/ask rules (auto-managed) |

### Auto-Management

The `permissions` object in `.claude/settings.local.json` is auto-managed by Claude Code during sessions. When the user grants or denies a tool permission interactively, Claude Code writes the rule to this file. The `allow`, `deny`, and `ask` arrays contain tool pattern strings.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Settings master index: full parameter table |
| settings | [001_global_settings.md](001_global_settings.md) | Global settings (G+P keys also appear here) |
| params | `../param/readme.md` | Full parameter table with CLI flags and env vars |
