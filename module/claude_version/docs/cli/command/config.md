# config — Config Namespace Command

### Scope

- **Purpose**: Reference for the `.config` clvcommand.
- **Responsibility**: Command syntax, parameters, exit codes, examples, and cross-references for `.config`.
- **In Scope**: `.config` (show-all / get / set / unset modes).
- **Out of Scope**: Deprecated `.settings.*` commands (→ [settings.md](settings.md)), version commands (→ [version.md](version.md)), process commands (→ [processes.md](processes.md)).

---

### Command :: 13. `.config`

Inspect or modify Claude Code settings with 4-layer effective-value resolution: env var → project config → user config → catalog default.

The operating mode is determined by the parameter combination:

| Mode | Parameters | Behavior |
|------|------------|----------|
| show-all | (none) | Print all resolved settings across all layers |
| get | `key::K` | Print resolved value for key K with source layer |
| set | `key::K value::V` | Write K→V to target scope (type-inferred) |
| unset | `key::K unset::1` | Delete key K from target scope |

-- **Parameters:** key::, value::, scope::, format::, v::, dry::, unset::
-- **Exit Codes:** 0 (success) | 1 (invalid params, bad combination) | 2 (write failure, HOME unset)

**Syntax:**

```sh
clv.config [key::K] [value::V] [scope::SCOPE] [format::FMT] [v::N] [dry::1] [unset::1]
```

**Parameters:**

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| [`key::`](../param/06_key.md) | [`SettingsKey`](../type/04_settings_key.md) | — | No | Setting key to read or write |
| [`value::`](../param/07_value.md) | [`SettingsValue`](../type/05_settings_value.md) | — | No | Value to write (type-inferred) |
| [`scope::`](../param/11_scope.md) | [`ConfigScope`](../type/06_config_scope.md) | user | No | Write target: user or project |
| [`format::`](../param/05_format.md) | [`OutputFormat`](../type/02_output_format.md) | text | No | Output format |
| [`v::`](../param/04_v.md) | [`VerbosityLevel`](../type/01_verbosity_level.md) | 1 | No | Output detail level |
| [`dry::`](../param/02_dry.md) | bool | false | No | Preview without writing |
| [`unset::`](../param/12_unset.md) | bool | false | No | Delete the key from target scope |

**Invalid combinations:**

| Combination | Error |
|-------------|-------|
| `value::V` without `key::K` | exit 1: `key:: is required when value:: is provided` |
| `unset::1` without `key::K` | exit 1: `key:: is required when unset::1` |
| `value::V` and `unset::1` together | exit 1: `value:: and unset:: are mutually exclusive` |
| `scope::project` without `key::K value::V` or `key::K unset::1` | exit 1: `scope:: only applies to write operations` |

**Algorithm (4 steps):**
1. Determine operating mode from parameter combination: show-all (no key), get (key only), set (key+value), unset (key+unset::1); exit 1 on invalid combination.
2. **show-all / get**: Resolve effective value(s) by querying all 4 layers in priority order (env var → project config → user config → catalog default); annotate each value with its source layer.
3. **set**: Infer value type (`"true"`/`"false"` → bool, numeric → number, else → string); atomically write key→value to target scope file via temp-file rename. **unset**: Delete the key from target scope file.
4. Render result (all resolved settings, single value with source annotation, or write confirmation) in the requested format.

**Examples:**

```sh
# Show all resolved settings with source annotations
clv.config

# Get the effective value of a key
clv.config key::model
clv.config key::theme format::json

# Set a value in user settings (default scope)
clv.config key::theme value::dark
clv.config key::model value::claude-opus-4-6

# Set a value in project settings
clv.config key::model value::claude-haiku-4-5-20251001 scope::project

# Preview a write (no file change)
clv.config key::theme value::dark dry::1

# Remove a key from user settings
clv.config key::theme unset::1
```

### Referenced Formats

| # | Format | Role |
|---|--------|------|
| 1 | [text](../format/01_text.md) | Default human-readable output |
| 2 | [json](../format/02_json.md) | Machine-readable structured output |

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|-----------|----------------|
| 1 | [Output Control](../param_group/01_output_control.md) | Partial | `count::` |
| 2 | [Execution Control](../param_group/02_execution_control.md) | Partial | `force::` |
| 4 | [Config Identity](../param_group/04_config_identity.md) | Full | — |

### Related Commands

| # | Command | Relationship |
|---|---------|-------------|
| 1 | [`.settings.show`](settings.md#command--9-settings-show) | Deprecated predecessor (show-all mode) |
| 2 | [`.settings.get`](settings.md#command--10-settings-get) | Deprecated predecessor (get mode) |
| 3 | [`.settings.set`](settings.md#command--11-settings-set) | Deprecated predecessor (set mode) |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 4 | [004 Settings Management](../user_story/004_settings_management.md) | Developer (settings management) |

---

**Category:** config
**Complexity:** 12
**API Requirement:** None
**Idempotent:** Yes (set/unset are idempotent on the same key+value)
**Risk Level:** Low
