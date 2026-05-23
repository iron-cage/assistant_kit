# hooks

Configures automated shell commands that execute at tool-use lifecycle events.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | — |
| Config Key | `hooks` |

### Type

object

### Default

`{}`

### Description

Defines shell commands to execute at tool-use lifecycle events. The top-level keys are event names; each value is an array of matcher objects. Supported events: `PreToolUse`, `PostToolUse`, `UserPromptSubmit`. Each matcher has a `matcher` field (tool name, e.g. `"Bash"`) and a `hooks` array of `{ "type": "command", "command": "<shell cmd>" }` entries. Hooks defined in `~/.claude/settings.json` are user-global and apply to every session. Hooks in a project-level `.claude/settings.json` apply only to that project. Hook output is captured; exit code non-zero aborts the tool call (for `PreToolUse`).

Example:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [{ "type": "command", "command": "/home/user/.claude/hooks/check.sh" }]
      }
    ]
  }
}
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
