# Parameter :: 11. `scope::`

-- **Summary:** Select the write target for `.config` set and unset operations.
-- **Type:** `ConfigScope`
-- **Default:** user
-- **Commands:** `.config`
-- **Group:** Config Identity

Selects which settings file is modified by `.config key::K value::V` or `.config key::K unset::1`. Has no effect on read/show operations.

- **Type:** [`ConfigScope`](../type/06_config_scope.md)
- **Default:** `user`
- **Validation:** must be `user` or `project`; any other value → exit 1

```sh
cm .config key::model value::claude-opus-4-6 scope::user     # writes ~/.claude/settings.json
cm .config key::model value::claude-haiku-4-5-20251001 scope::project  # writes {cwd}/.claude/settings.json
```

### Referenced Commands

| # | Command |
|---|---------|
| 1 | [`.config`](../command/config.md#command--13-config) |

### Referenced Parameter Groups

| # | Group |
|---|-------|
| 1 | [Config Identity](../param_group/04_config_identity.md) |

### Referenced Types

| # | Type |
|---|------|
| 1 | [`ConfigScope`](../type/06_config_scope.md) |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [004 Settings Management](../user_story/004_settings_management.md) | Developer (settings management) |
