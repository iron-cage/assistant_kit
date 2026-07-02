# Parameter :: 12. `unset::`

-- **Summary:** Remove a settings key from the target scope file instead of writing a value.
-- **Type:** bool
-- **Default:** false
-- **Commands:** `.config`
-- **Group:** Config Identity

When `unset::1`, `.config key::K` deletes key K from the target scope's settings file rather than reading or writing it. Mutually exclusive with `value::`.

- **Type:** bool (0 or 1)
- **Default:** `false` (0)
- **Validation:** `unset::1` requires `key::` to be present; `unset::1` with `value::` → exit 1

```sh
clv.config key::theme unset::1            # removes "theme" from user settings
clv.config key::theme unset::1 scope::project  # removes "theme" from project settings
clv.config key::theme unset::1 dry::1     # previews removal without changing file
```

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.config`](../command/config.md#command--13-config) | false | Requires key::; mutually exclusive with value:: |

### Referenced Type

| # | Type |
|---|------|
| 1 | `bool` |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|-----------|-----------|
| 1 | [Config Identity](../param_group/04_config_identity.md) | Full | `key::`, `value::`, `scope::` |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [006 Config Management](../user_story/006_config_management.md) | Developer (config management) |

