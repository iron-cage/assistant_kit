# Feature: Params Command

### Scope

- **Purpose**: Document the `.params` command that displays all Claude Code parameters with their forms, current observable values, and defaults.
- **Responsibility**: Describe the param catalog, operating modes, output format, observable vs. unobservable distinction, and acceptance criteria.
- **In Scope**: `.params` (show-all / single / filtered modes), params catalog schema, observable param resolution (env + config), CLI-only annotation, `kind::` filter, `format::` output.
- **Out of Scope**: 4-layer config resolution for write operations (→ `feature/006_config_command.md`), settings.json read/write atomics (→ `feature/003_settings_management.md`), type inference for set operations (→ `algorithm/001_settings_type_inference.md`).

### Design

**Purpose distinction from `.config`:** `.config` manages persistable state — it reads and writes settings.json files. `.params` is read-only and answers "what Claude Code parameters exist, what forms do they take, and what are their current observable values?" It is a runtime inspection tool, not a configuration management tool.

**Operating modes:** The mode is determined by the presence of `key::`:

| Mode | Parameters | Behavior |
|------|------------|----------|
| show-all | (none) | Table of all params in catalog, alphabetically sorted |
| single | `key::K` | Deep-dive for one param: all forms, current env value, current config value, default |

**`kind::` filter (show-all only):**

| Value | Params shown |
|-------|-------------|
| absent | All params in catalog |
| `config` | Only params with a config key form |
| `env` | Only params with an env var form |

**Observable vs. unobservable:** Claude Code has three param kinds relative to observability from `clv`:

| Kind | Observable? | What `.params` shows |
|------|-------------|----------------------|
| Config-key param | Yes — `clv` reads settings.json | Forms + config value (user + project) + default |
| Env-var param | Yes — `clv` reads env at startup | Forms + live env value (set or unset) + default |
| CLI-flag-only param | No — per-invocation to `claude` binary | Forms + default; marks value as `(CLI-only, unobservable)` |

**Params catalog:** A static registry (`params_catalog.rs` in `claude_version_core`) covering all Claude Code parameters that have at least one observable form. Entries cover config-key params (~21) and env-only params (~15). Each entry defines:

| Field | Description |
|-------|-------------|
| `name` | Canonical parameter name (snake_case, e.g., `bash_timeout`) |
| `cli_flag` | CLI flag syntax string, e.g., `--effort <level>` (or None) |
| `env_var` | Env var name, e.g., `CLAUDE_CODE_BASH_TIMEOUT` (or None) |
| `config_key` | Settings.json key name, e.g., `effortLevel` (or None) |
| `default` | Binary default value string (or None if truly absent) |
| `description` | One-line description of what the param controls |

**Text output (v::1, single param):**

```
model
  Forms:   CLI --model <model>  |  config model
  Env:     CLAUDE_MODEL → "claude-opus-4-6" (set)
  Config:  model = "claude-sonnet-4-6" (user)
  Default: claude-sonnet-4-6
  ───────────────────────────────────────
  Effective (via .config): claude-opus-4-6 (env)
```

**Text output (v::1, env-only param):**

```
bash_timeout
  Forms:   env CLAUDE_CODE_BASH_TIMEOUT
  Env:     CLAUDE_CODE_BASH_TIMEOUT → unset
  Default: 120000 ms
```

**Text output (v::1, CLI-only param):**

```
print
  Forms:   CLI -p / --print
  Value:   (CLI-only — not observable from clv; passed directly to claude at invocation)
  Default: off
```

**JSON output:** Array of objects, one per param, with fields: `name`, `cli_flag`, `env_var`, `config_key`, `env_value`, `config_value`, `default`, `effective`, `source`.

**Show-all text output (v::0):** One line per param: `name = value (source)`, or `name = (unset)` for absent env-only params, or `name = (CLI-only)` for CLI-only params.

**Exit codes:**

| Code | Condition |
|------|-----------|
| 0 | Success |
| 1 | Invalid `kind::` value or invalid `format::` value |
| 2 | `key::K` specified but K not found in params catalog |

### Acceptance Criteria

- **AC-01**: `clv.params` exits 0; stdout contains at least 35 parameter entries; each entry includes a source annotation or CLI-only marker.
- **AC-02**: `clv.params key::model` exits 0; output shows all three forms (CLI `--model`, env `CLAUDE_MODEL`, config `model`), the current resolved value and its source, and the default `claude-sonnet-4-6`.
- **AC-03**: `clv.params kind::config` exits 0; output contains only params that have a config key form; env-only params (`bash_timeout`, `api_key`, etc.) are absent from output.
- **AC-04**: `clv.params kind::env` exits 0; output contains only params that have an env var form; config-only params (`theme`, `voiceEnabled`, etc.) are absent from output.
- **AC-05**: `clv.params key::model` when `CLAUDE_MODEL` env var is set exits 0; output shows the env value and annotates it as `(env)` (env layer wins).
- **AC-06**: `clv.params key::bash_timeout` exits 0; output shows `CLAUDE_CODE_BASH_TIMEOUT` current env value (or `unset`) and default `120000`.
- **AC-07**: `clv.params format::json` exits 0; output is valid JSON array with `name`, `cli_flag`, `env_var`, `config_key`, `env_value`, `config_value`, `default`, `effective`, `source` fields per entry.
- **AC-08**: `clv.params key::print` exits 0; output contains the `--print` CLI flag form and a `(CLI-only, unobservable)` annotation.
- **AC-09**: `clv.params key::NONEXISTENT` exits 2.
- **AC-10**: `clv.params kind::badvalue` exits 1.
- **AC-11**: `clv.params key::model` with no env var set and no config value exits 0; output shows catalog default `claude-sonnet-4-6` with `(default)` annotation.
- **AC-12**: Show-all mode lists params in alphabetical order.

### Data Sources

| Source | Role |
|--------|------|
| `params_catalog.rs` | Static registry of all Claude Code params (forms, defaults, descriptions) |
| `config_resolve.rs` | Provides config-layer value (user + project) via 4-layer resolution |
| `std::env::var()` | Provides live env var value at command invocation time |

### Features

| File | Relationship |
|------|-------------|
| [feature/006_config_command.md](006_config_command.md) | `.config` command — manages settings.json (write); `.params` is read-only |
| [feature/003_settings_management.md](003_settings_management.md) | Settings I/O underlying config reads |

### Algorithms

| File | Relationship |
|------|-------------|
| [algorithm/002_config_resolution.md](../algorithm/002_config_resolution.md) | 4-layer resolution used for config-layer values in single-param mode |

### Sources

| File | Relationship |
|------|-------------|
| `../../../claude_version_core/src/params_catalog.rs` | Params catalog registry (ParamDef) |
| `../../src/commands/params.rs` | `.params` command handler |

### Tests

| File | Relationship |
|------|-------------|
| [tests/docs/feature/07_params_command.md](../../tests/docs/feature/07_params_command.md) | Feature test spec |
| [tests/docs/cli/command/14_params.md](../../tests/docs/cli/command/14_params.md) | Command integration test spec |
