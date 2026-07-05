# Group :: 3. Settings Identity

-- **Summary:** Parameters that identify the settings entry being operated on.
-- **Parameters:** `key::`, `value::`
-- **Coherence Test:** "Does this parameter identify the settings entry?"

Both parameters specify the target of a settings read or write.

**Parameters:**

| Parameter | Type | Purpose |
|-----------|------|---------|
| [`key::`](../param/06_key.md) | [`SettingsKey`](../type/04_settings_key.md) | Entry name |
| [`value::`](../param/07_value.md) | [`SettingsValue`](../type/05_settings_value.md) | Entry value (type-inferred) |

**Partial implementors:** `.settings.get` implements `key::` only (read operation — no `value::`).

**Why NOT in this group:**
- `version::`: specifies installation target, not settings target
- `dry::`: controls execution mode, not target identification

**Typical usage:**

```sh
clv .settings.get key::theme
clv .settings.set key::theme value::dark
```

### Referenced Commands

| # | Command | Membership | Excluded Params |
|---|---------|-----------|----------------|
| 1 | [`.settings.get`](../command/settings.md#command-10-settingsget) | Partial | `value::` |
| 2 | [`.settings.set`](../command/settings.md#command-11-settingsset) | Full | — |

### Referenced Parameters

| # | Parameter | Type | Default | Role in Group |
|---|-----------|------|---------|---------------|
| 1 | [`key::`](../param/06_key.md) | [`SettingsKey`](../type/04_settings_key.md) | (required) | Entry name to read or write |
| 2 | [`value::`](../param/07_value.md) | [`SettingsValue`](../type/05_settings_value.md) | (required) | Entry value to write (type-inferred) |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [004 Settings Management](../user_story/004_settings_management.md) | Developer (settings management) |
