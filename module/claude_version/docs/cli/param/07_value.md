# Parameter :: 7. `value::`

-- **Summary:** The value to write to a settings entry; type-inferred for JSON storage.
-- **Type:** `SettingsValue`
-- **Default:** (required)
-- **Commands:** `.settings.set`, `.config`
-- **Group:** Settings Identity, Config Identity

Required for `.settings.set`. Optional for `.config` (when present alongside `key::`, triggers set mode). Type-inferred: `"true"`/`"false"` -> JSON bool, integer/float -> JSON number, otherwise -> JSON string.

- **Type:** [`SettingsValue`](../type/05_settings_value.md)
- **Default:** **(required)**
- **Validation:** must not be empty; `value::` (empty) -> exit 1

```sh
clv.settings.set key::theme value::dark      # -> "dark" (string)
clv.settings.set key::timeout value::30      # -> 30 (number)
clv.settings.set key::autoUpdate value::true  # -> true (bool)
```

### Referenced Commands

| # | Command |
|---|---------|
| 1 | [`.settings.set`](../command/settings.md#command--11-settingsset) |
| 2 | [`.config`](../command/config.md#command--13-config) |

### Referenced Parameter Groups

| # | Group |
|---|-------|
| 1 | [Settings Identity](../param_group/03_settings_identity.md) |
| 2 | [Config Identity](../param_group/04_config_identity.md) |

### Referenced Types

| # | Type |
|---|------|
| 1 | [`SettingsValue`](../type/05_settings_value.md) |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [004 Settings Management](../user_story/004_settings_management.md) | Developer (settings management) |
| 2 | [006 Config Management](../user_story/006_config_management.md) | Developer (config management) |
