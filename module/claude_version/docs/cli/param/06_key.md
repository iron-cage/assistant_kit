# Parameter :: 6. `key::`

-- **Summary:** Identify the settings entry to read or write.
-- **Type:** `SettingsKey`
-- **Default:** (required)
-- **Commands:** `.settings.get`, `.settings.set`, `.config`
-- **Group:** Settings Identity, Config Identity

Required for `.settings.get` and `.settings.set`. Optional for `.config` — absent means show-all mode. When present, value must not be empty.

- **Type:** [`SettingsKey`](../type/04_settings_key.md) / [`ConfigKey`](../type/07_config_key.md)
- **Default:** (required for `.settings.*`; optional for `.config`)
- **Validation:** when present, must not be empty; `key::` (empty) -> exit 1

```sh
clv.settings.get key::theme
clv.settings.set key::theme value::dark
```

### Referenced Commands

| # | Command |
|---|---------|
| 1 | [`.settings.get`](../command/settings.md#command--10-settingsget) |
| 2 | [`.settings.set`](../command/settings.md#command--11-settingsset) |
| 3 | [`.config`](../command/config.md#command--13-config) |

### Referenced Parameter Groups

| # | Group |
|---|-------|
| 1 | [Settings Identity](../param_group/03_settings_identity.md) |
| 2 | [Config Identity](../param_group/04_config_identity.md) |

### Referenced Types

| # | Type |
|---|------|
| 1 | [`SettingsKey`](../type/04_settings_key.md) |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [004 Settings Management](../user_story/004_settings_management.md) | Developer (settings management) |
| 2 | [006 Config Management](../user_story/006_config_management.md) | Developer (config management) |
