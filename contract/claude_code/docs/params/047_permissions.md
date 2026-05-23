# permissions

Per-project tool permission rules — allowlist, denylist, and ask-list for tool calls.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | — |
| Config Key | `permissions` |

### Type

object

### Default

`{}`

### Description

Defines per-project tool permission rules as three lists. Written by Claude Code when the user grants or denies a tool permission during a session. Stored in project-level `.claude/settings.json` or `.claude/settings.local.json`; not typically set in the user-global `~/.claude/settings.json`.

Each list contains pattern strings of the form `"Tool(pattern)"` where `Tool` is the tool name (e.g. `Bash`, `Read`, `Write`) and `pattern` is a glob or prefix matched against the tool call argument.

| Field | Description |
|-------|-------------|
| `allow` | Tool calls matching these patterns are auto-approved without prompting |
| `deny` | Tool calls matching these patterns are auto-rejected without prompting |
| `ask` | Tool calls matching these patterns always prompt regardless of other settings |

Example:

```json
{
  "permissions": {
    "allow": [
      "Bash(npm run test:*)",
      "Bash(npx eslint:*)",
      "Bash(cat:*)"
    ],
    "deny": [],
    "ask": []
  }
}
```

### Notes

- Typically auto-managed by Claude Code when the user clicks "Always allow" or "Always deny" during a session
- Project-level rules apply only within that project directory hierarchy
- Patterns are matched prefix/glob style against the tool call input
- An empty `{}` value means all tool calls use the default permission mode

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [permission_mode.md](permission_mode.md) | Session-level permission mode |
| doc | [allowed_tools.md](allowed_tools.md) | CLI-level tool allowlist |
