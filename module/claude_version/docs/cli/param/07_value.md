# Parameter :: 7. `value::`

-- **Summary:** The value to write to a settings entry; type-inferred for JSON storage.
-- **Type:** `SettingsValue`
-- **Default:** (required)
-- **Commands:** `.settings.set`
-- **Group:** Settings Identity

Required for `.settings.set`. Type-inferred: `"true"`/`"false"` -> JSON bool,
integer/float -> JSON number, otherwise -> JSON string.

- **Type:** [`SettingsValue`](../type/05_settings_value.md)
- **Default:** **(required)**
- **Validation:** must not be empty; `value::` (empty) -> exit 1

```sh
cm .settings.set key::theme value::dark      # -> "dark" (string)
cm .settings.set key::timeout value::30      # -> 30 (number)
cm .settings.set key::autoUpdate value::true  # -> true (bool)
```

### Referenced Commands

| # | Command |
|---|---------|
| 1 | [`.settings.set`](../command/settings.md#command--11-settingsset) |

### Referenced Parameter Groups

| # | Group |
|---|-------|
| 1 | [Settings Identity](../param_group/03_settings_identity.md) |

### Referenced Types

| # | Type |
|---|------|
| 1 | [`SettingsValue`](../type/05_settings_value.md) |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [004 Settings Management](../user_story/004_settings_management.md) | Developer (settings management) |
