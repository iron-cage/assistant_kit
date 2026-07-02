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
clv.config key::model value::claude-opus-4-8 scope::user     # writes ~/.claude/settings.json
clv.config key::model value::claude-haiku-4-5-20251001 scope::project  # writes {cwd}/.claude/settings.json
```

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.config`](../command/config.md#command--13-config) | user | Applies to set/unset only; ignored for read operations |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|-----------|-----------|
| 1 | [Config Identity](../param_group/04_config_identity.md) | Full | `key::`, `value::`, `unset::` |

### Referenced Type

| # | Type |
|---|------|
| 1 | [`ConfigScope`](../type/06_config_scope.md) |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [006 Config Management](../user_story/006_config_management.md) | Developer (config management) |
