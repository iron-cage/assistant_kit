# Inspect all Claude Code parameters and their current values

**Persona:** developer (config inspector)
**Goal:** Discover all Claude Code parameters, see their current effective values and sources, and identify which are persistently configurable vs. per-invocation only.
**Benefit:** Diagnoses unexpected behavior from env var or config overrides; reveals all configuration options in one view.
**Priority:** Medium

### Acceptance Criteria

- [ ] `clv.params` shows all known catalog parameters with source annotations in a single view.
- [ ] `clv.params kind::config` shows only parameters with a settings.json config key form.
- [ ] `clv.params kind::env` shows only parameters with an env var form, with current env values.
- [ ] `clv.params key::model` shows all forms (CLI, env, config key), current env value, current config value, default, and effective value with source.
- [ ] `clv.params key::bash_timeout` shows env-only form with no config key entry.
- [ ] `clv.params format::json` outputs valid parseable JSON with the same data as text mode.
- [ ] `clv.params v::0` outputs values only with no form annotations.
- [ ] `clv.params v::2` outputs full descriptions alongside each parameter.
- [ ] `clv.params kind::invalid` exits 1 with a message indicating valid values are `config` or `env`.
- [ ] `clv.params key::nonexistent_param` exits 2 indicating the key is not in the params catalog.

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.params`](../command/params.md#command-14-params) | Primary command for parameter discovery and inspection |

### Referenced Formats

| # | Format | Role |
|---|--------|------|
| 1 | [text](../format/01_text.md) | Default human-readable param catalog output |
| 2 | [json](../format/02_json.md) | Machine-readable param export for scripting |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Output Control](../param_group/01_output_control.md) | Controls verbosity and format of params output |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`key::`](../param/06_key.md) | Select specific param for single-param mode |
| 2 | [`format::`](../param/05_format.md) | Selects text or JSON rendering |
| 3 | [`v::`](../param/04_v.md) | Controls output detail level |
| 4 | [`kind::`](../param/13_kind.md) | Filters show-all to config or env params only |

### Workflow Steps

**Step 1 — Browse the full parameter catalog:**

```bash
clv .params
# bash_timeout   env: CLAUDE_CODE_BASH_TIMEOUT   default: 120000          (env)
# model          cli: --model  env: CLAUDE_MODEL  config: model  default: claude-sonnet-5
# theme          config: theme                    current: dark            (user)
# ...  (35+ entries)
```

**Step 2 — Filter to config-key parameters only:**

```bash
clv .params kind::config
# model   config: model   current: claude-sonnet-5  (catalog default)
# theme   config: theme   current: dark             (user)
# ...
```

**Step 3 — Inspect a single parameter in depth:**

```bash
clv .params key::model
# CLI:     --model
# Env:     CLAUDE_MODEL   (unset)
# Config:  model          claude-sonnet-5  (catalog default)
# Default: claude-sonnet-5
```

**Step 4 — Export the full catalog as JSON for tooling:**

```bash
clv .params format::json
# [{"name":"model","cli":"--model","env":"CLAUDE_MODEL","config":"model",...}, ...]
```
