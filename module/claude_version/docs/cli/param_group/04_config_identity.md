# Group :: 4. Config Identity

-- **Summary:** Parameters that identify the config entry being operated on and the write target.
-- **Parameters:** `key::`, `value::`, `scope::`, `unset::`
-- **Coherence Test:** "Does this parameter identify the config target or operation?"

These parameters together specify what to read or write (key, value) and where to write it (scope, unset).

**Parameters:**

| Parameter | Type | Purpose |
|-----------|------|---------|
| [`key::`](../param/06_key.md) | [`SettingsKey`](../type/04_settings_key.md) | Config key to read or write |
| [`value::`](../param/07_value.md) | [`SettingsValue`](../type/05_settings_value.md) | Value to write (type-inferred); absent = read mode |
| [`scope::`](../param/11_scope.md) | [`ConfigScope`](../type/06_config_scope.md) | Write target: user or project |
| [`unset::`](../param/12_unset.md) | bool | Delete key instead of writing |

**Mode disambiguation:**

| key:: | value:: | unset:: | Mode |
|-------|---------|---------|------|
| absent | absent | absent | show-all |
| present | absent | absent | get |
| present | present | absent | set |
| present | absent | true | unset |

**Why NOT in this group:**
- `dry::`: controls execution mode, not target identification
- `v::`: controls output verbosity, not target identification
- `format::`: controls output format, not target identification

**Typical usage:**

```sh
clv .config key::model
clv .config key::theme value::dark
clv .config key::theme value::dark scope::project
clv .config key::theme unset::1
```

### Referenced Commands

| # | Command | Membership |
|---|---------|-----------|
| 1 | [`.config`](../command/config.md#command--13-config) | Full (`key::`, `value::`, `scope::`, `unset::`) |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [004 Settings Management](../user_story/004_settings_management.md) | Developer (settings management) |
