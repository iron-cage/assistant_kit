# Type :: 6. `ConfigScope`

-- **Summary:** Write target for `.config` set and unset operations.
-- **Base Type:** String enum
-- **Constraints:** exactly `user` or `project`
-- **Default:** `user`
-- **Used By:** `scope::`

Selects which settings file is the write target. Read/show operations always consult both files via the resolution chain regardless of this value.

- **Base type:** String (2 valid values)
- **Constraints:** `user` | `project`
- **Validation:** any other value → exit 1 with `"scope:: must be 'user' or 'project'"`

**Values:**

| Value | Target file |
|-------|-------------|
| `user` | `~/.claude/settings.json` |
| `project` | `{cwd}/.claude/settings.json` (creates dir + file if absent) |

```sh
clv .config key::model value::claude-opus-4-6 scope::user
clv .config key::model value::claude-haiku-4-5-20251001 scope::project
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|--------------|
| 1 | [`.config`](../command/config.md#command--13-config) | `scope::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|---------|
| 1 | [`scope::`](../param/11_scope.md) | 1 |
