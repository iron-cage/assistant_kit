# settings — Settings Namespace Commands

> **Deprecation notice:** `.settings.show`, `.settings.get`, and `.settings.set` are deprecated in favor of the unified [`.config`](config.md) command. They remain functional but will be removed in a future version.

### Scope

- **Purpose**: Reference for settings-namespace clv commands (deprecated).
- **Responsibility**: Command syntax, parameters, exit codes, and cross-references for `.settings.show`, `.settings.get`, and `.settings.set`.
- **In Scope**: `.settings.show`, `.settings.get`, `.settings.set`.
- **Out of Scope**: Version commands (→ [version.md](version.md)), process commands (→ [processes.md](processes.md)), unified config command (→ [config.md](config.md)).

---

### Command :: 9. `.settings.show`

Print all key-value pairs from `~/.claude/settings.json`. Use this to inspect the current settings state before reading or writing individual keys.

-- **Parameters:** v::, format::
-- **Exit Codes:** 0 (success) | 2 (file unreadable or malformed JSON)

**Syntax:**

```sh
clv.settings.show [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| [`v::`](../param/04_v.md) | [`VerbosityLevel`](../type/01_verbosity_level.md) | 1 | No | Output detail level |
| [`format::`](../param/05_format.md) | [`OutputFormat`](../type/02_output_format.md) | text | No | Output format |

**Algorithm (2 steps):**
1. Read and parse `~/.claude/settings.json`; exit 2 on read or parse error.
2. Render all key-value pairs in the requested format.

**Examples:**

```sh
clv.settings.show
clv.settings.show format::json
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

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 1 | [`v::`](../param/04_v.md) |
| 2 | [`format::`](../param/05_format.md) |

### Related Commands

| # | Command | Relationship |
|---|---------|-------------|
| 1 | [`.settings.get`](#command-10-settingsget) | Reads a single key from the displayed set |
| 2 | [`.settings.set`](#command-11-settingsset) | Modifies a key from the displayed set |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [004 Settings Management](../user_story/004_settings_management.md) | Developer (settings management) |

---

**Category:** settings
**Complexity:** 2
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** Low

---

### Command :: 10. `.settings.get`

Read a single setting from `~/.claude/settings.json` by key. Exits 2 if the key is absent from the settings file.

-- **Parameters:** key::, v::, format::
-- **Exit Codes:** 0 (success) | 1 (missing key::) | 2 (key not found or file error)

**Syntax:**

```sh
clv.settings.get key::<KEY> [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| [`key::`](../param/06_key.md) | [`SettingsKey`](../type/04_settings_key.md) | — | Yes | Setting to read |
| [`v::`](../param/04_v.md) | [`VerbosityLevel`](../type/01_verbosity_level.md) | 1 | No | Output detail level |
| [`format::`](../param/05_format.md) | [`OutputFormat`](../type/02_output_format.md) | text | No | Output format |

**Algorithm (3 steps):**
1. Validate `key::` is present; exit 1 if absent.
2. Read and parse `~/.claude/settings.json`; look up the key; exit 2 on file error or missing key.
3. Render the key-value pair in the requested format.

**Examples:**

```sh
clv.settings.get key::theme
clv.settings.get key::autoUpdate format::json
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
| 2 | [Settings Identity](../param_group/03_settings_identity.md) | Partial | `value::` |

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 1 | [`key::`](../param/06_key.md) |
| 2 | [`v::`](../param/04_v.md) |
| 3 | [`format::`](../param/05_format.md) |

### Related Commands

| # | Command | Relationship |
|---|---------|-------------|
| 1 | [`.settings.show`](#command-9-settingsshow) | Shows all settings for broader context |
| 2 | [`.settings.set`](#command-11-settingsset) | Writes the key that this command reads |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [004 Settings Management](../user_story/004_settings_management.md) | Developer (settings management) |

---

**Category:** settings
**Complexity:** 5
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** Low

---

### Command :: 11. `.settings.set`

Write a single setting atomically to `~/.claude/settings.json`. The value is type-inferred: `"true"`/`"false"` → bool, integer/float → number, otherwise → string. Creates the key if absent (upsert semantics).

-- **Parameters:** key::, value::, dry::
-- **Exit Codes:** 0 (success) | 1 (missing key:: or value::) | 2 (write failure)

**Syntax:**

```sh
clv.settings.set key::<KEY> value::<VALUE> [dry::1]
```

**Parameters:**

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| [`key::`](../param/06_key.md) | [`SettingsKey`](../type/04_settings_key.md) | — | Yes | Setting to write |
| [`value::`](../param/07_value.md) | [`SettingsValue`](../type/05_settings_value.md) | — | Yes | Value to write (type-inferred) |
| [`dry::`](../param/02_dry.md) | bool | false | No | Preview without writing |

**Algorithm (4 steps):**
1. Validate both `key::` and `value::` are present; exit 1 if either absent.
2. Infer value type: `"true"`/`"false"` → bool, numeric string → number, anything else → string.
3. Atomically read-modify-write `~/.claude/settings.json` via temp-file rename (upsert semantics).
4. Print confirmation line showing the key and its new stored value.

**Examples:**

```sh
clv.settings.set key::theme value::dark
clv.settings.set key::timeout value::30       # stored as number
clv.settings.set key::autoUpdate value::true  # stored as bool
clv.settings.set key::theme value::dark dry::1
```

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|-----------|----------------|
| 1 | [Execution Control](../param_group/02_execution_control.md) | Partial | `force::` |
| 2 | [Settings Identity](../param_group/03_settings_identity.md) | Full | — |

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 1 | [`key::`](../param/06_key.md) |
| 2 | [`value::`](../param/07_value.md) |
| 3 | [`dry::`](../param/02_dry.md) |

### Related Commands

| # | Command | Relationship |
|---|---------|-------------|
| 1 | [`.settings.get`](#command-10-settingsget) | Reads the key after writing |
| 2 | [`.settings.show`](#command-9-settingsshow) | Verifies full settings after modification |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [004 Settings Management](../user_story/004_settings_management.md) | Developer (settings management) |

---

**Category:** settings
**Complexity:** 7
**API Requirement:** Write
**Idempotent:** Yes
**Risk Level:** Low
