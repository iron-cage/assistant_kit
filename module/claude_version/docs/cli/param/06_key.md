# Parameter :: 6. `key::`

-- **Summary:** Identify the settings entry to read or write.
-- **Type:** `SettingsKey`
-- **Default:** (required)
-- **Commands:** `.settings.get`, `.settings.set`
-- **Group:** Settings Identity

Required for `.settings.get` and `.settings.set`. Missing or empty value exits 1.

- **Type:** [`SettingsKey`](../type/04_settings_key.md)
- **Default:** **(required)**
- **Validation:** must not be empty; `key::` (empty) -> exit 1

```sh
cm .settings.get key::theme
cm .settings.set key::theme value::dark
```

### Referenced Commands

| # | Command |
|---|---------|
| 1 | [`.settings.get`](../command/settings.md#command--10-settingsget) |
| 2 | [`.settings.set`](../command/settings.md#command--11-settingsset) |

### Referenced Parameter Groups

| # | Group |
|---|-------|
| 1 | [Settings Identity](../param_group/03_settings_identity.md) |

### Referenced Types

| # | Type |
|---|------|
| 1 | [`SettingsKey`](../type/04_settings_key.md) |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [004 Settings Management](../user_story/004_settings_management.md) | Developer (settings management) |
