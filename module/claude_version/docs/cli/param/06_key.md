# Parameter :: 6. `key::`

-- **Summary:** Identify the settings entry to read or write.
-- **Type:** `SettingsKey`
-- **Default:** (required)
-- **Commands:** `.settings.get`, `.settings.set`, `.config`, `.params`, `.paths`
-- **Group:** Settings Identity, Config Identity

Required for `.settings.get` and `.settings.set`. Optional for `.config`, `.params`, and `.paths` — absent means show-all mode. When present, value must not be empty.

- **Type:** [`SettingsKey`](../type/04_settings_key.md) / [`ConfigKey`](../type/07_config_key.md) / [`PathKey`](../type/09_path_key.md)
- **Default:** (required for `.settings.*`; optional for `.config`, `.params`, `.paths`)
- **Validation:** when present, must not be empty; `key::` (empty) -> exit 1

```sh
clv.settings.get key::theme
clv.settings.set key::theme value::dark
```

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.settings.get`](../command/settings.md#command-10-settingsget) | (required) | Identifies entry to read |
| 2 | [`.settings.set`](../command/settings.md#command-11-settingsset) | (required) | Identifies entry to write |
| 3 | [`.config`](../command/config.md#command-13-config) | — | Absent = show-all mode; present = get/set/unset mode |
| 4 | [`.params`](../command/params.md#command-14-params) | — | Absent = show-all; present = single-param deep-dive |
| 5 | [`.paths`](../command/paths.md#command-16-paths) | — | Absent = show-all; present = single-path lookup |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|-----------|-----------|
| 1 | [Settings Identity](../param_group/03_settings_identity.md) | Full | `value::` |
| 2 | [Config Identity](../param_group/04_config_identity.md) | Full | `value::`, `scope::`, `unset::` |

### Referenced Type

| # | Type |
|---|------|
| 1 | [`SettingsKey`](../type/04_settings_key.md) |
| 2 | [`ConfigKey`](../type/07_config_key.md) |
| 3 | [`PathKey`](../type/09_path_key.md) |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [004 Settings Management](../user_story/004_settings_management.md) | Developer (settings management) |
| 2 | [006 Config Management](../user_story/006_config_management.md) | Developer (config management) |
| 3 | [007 Params Inspection](../user_story/007_params_inspection.md) | Developer (config inspector) |
| 4 | [008 Path Discovery](../user_story/008_path_discovery.md) | Developer (path discovery and scripting) |
