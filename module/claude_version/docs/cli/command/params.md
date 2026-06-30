# params — Params Inspection Command

### Scope

- **Purpose**: Reference for the `.params` clv command.
- **Responsibility**: Command syntax, parameters, exit codes, examples, and cross-references for `.params`.
- **In Scope**: `.params` (show-all / single / filtered modes).
- **Out of Scope**: Deprecated `.settings.*` commands (→ [settings.md](settings.md)), config management (→ [config.md](config.md)), version commands (→ [version.md](version.md)).

---

### Command :: 14. `.params`

Inspect Claude Code parameters: show their forms (CLI flag, env var, config key), current observable values (env vars read at invocation, settings.json resolved values), and defaults. Read-only — does not modify any settings or state.

The operating mode is determined by whether `key::` is provided:

| Mode | Parameters | Behavior |
|------|------------|----------|
| show-all | (none) | Table of all catalog params, sorted alphabetically |
| single | `key::K` | Deep-dive for one param: all forms, current values, source |

-- **Parameters:** key::, kind::, format::, v::
-- **Exit Codes:** 0 (success) | 1 (invalid kind:: or format:: value) | 2 (key:: specified but unknown to params catalog)

**Syntax:**

```sh
clv.params [key::K] [kind::KIND] [format::FMT] [v::N]
```

**Parameters:**

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| [`key::`](../param/06_key.md) | string | — | No | Specific param name for single-param mode |
| [`kind::`](../param/13_kind.md) | [`ParamKind`](../type/08_param_kind.md) | — | No | Filter show-all to one param kind only |
| [`format::`](../param/05_format.md) | [`OutputFormat`](../type/02_output_format.md) | text | No | Output format |
| [`v::`](../param/04_v.md) | [`VerbosityLevel`](../type/01_verbosity_level.md) | 1 | No | Detail level: 0=values only, 1=forms+values, 2=with descriptions |

**`kind::` values:**

| Value | Effect |
|-------|--------|
| absent | Show all params in catalog |
| `config` | Show only params with a config key form (settings.json params) |
| `env` | Show only params with an env var form |

**Algorithm (show-all, 3 steps):**
1. Load params catalog. If `kind::` is provided, filter catalog to the matching param kind; exit 1 if `kind::` value is not `config` or `env`.
2. For each param in alphabetical order: read current env var value (if env form exists); resolve current config value via 4-layer resolution (if config key form exists).
3. Render params table in requested format and verbosity.

**Algorithm (single-param, 4 steps):**
1. Look up `key::K` in params catalog; exit 2 if not found.
2. Read current env var value if this param has an env form.
3. Resolve current config value via 4-layer resolution if this param has a config key form.
4. Render full param detail block in requested format.

**Examples:**

```sh
# Show all known Claude Code params with their current observable values
clv.params

# Deep-dive for one param: all forms, current env, current config, default, effective
clv.params key::model

# Show only settings.json config params
clv.params kind::config

# Show only env-var params and their current env values
clv.params kind::env

# Machine-readable output
clv.params format::json
clv.params key::model format::json

# Minimal output — values only, no form annotations
clv.params v::0
```

**Sample text output (v::1, `clv.params key::model`):**

```
model
  Forms:   CLI --model <model>  |  env CLAUDE_MODEL  |  config model
  Env:     CLAUDE_MODEL → "claude-opus-4-6" (set)
  Config:  model = (absent in user config)
  Default: claude-sonnet-4-6
  ────────────────────────────────────────────────────
  Effective (via .config): claude-opus-4-6 (env)
```

**Sample text output (v::1, `clv.params key::bash_timeout`):**

```
bash_timeout
  Forms:   env CLAUDE_CODE_BASH_TIMEOUT
  Env:     CLAUDE_CODE_BASH_TIMEOUT → unset
  Default: 120000
```

**Sample text output (v::1, `clv.params key::print`):**

```
print
  Forms:   CLI -p / --print
  Value:   (CLI-only — unobservable from clv; passed directly to claude at invocation)
  Default: off
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
| 1 | [`key::`](../param/06_key.md) |
| 2 | [`format::`](../param/05_format.md) |
| 3 | [`v::`](../param/04_v.md) |
| 4 | [`kind::`](../param/13_kind.md) |

### Related Commands

| # | Command | Relationship |
|---|---------|-------------|
| 1 | [`.config`](config.md) | Manages persistable settings.json state (read/write); `.params` is read-only inspection |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [007 Params Inspection](../user_story/007_params_inspection.md) | Developer (param discovery and current-value inspection) |

---

**Category:** params
**Complexity:** 8
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** None (read-only)
