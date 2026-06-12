# Commands :: Model

Session model management command.

---

### Command :: 17. `.model`

Get or set the Claude Code session model in `~/.claude/settings.json`. Without parameters, prints the current model. With `set::`, writes the requested model.

-- **Parameters:** [`set::`](../param/055_set.md), [`format::`](../param/002_format.md)
-- **Exit:** 0 (success) | 1 (usage: unknown `set::` value) | 2 (runtime: HOME not set)

**Syntax:**

```bash
clp .model               # get
clp .model set::opus     # set to Opus
clp .model set::default  # remove model key (Claude Code default)
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `set::` | `enum` | *(omit)* | Model to write: `opus`, `sonnet`, `haiku`, `default` |
| `format::` | [`OutputFormat`](../type/002_output_format.md) | `text` | Output format (get mode only) |

**Mode dispatch:**

| `set::` | Mode |
|---------|------|
| absent | get — read `model` from `~/.claude/settings.json` |
| present | set — validate value, write via `set_session_model()` |

**Algorithm (get, 2 steps):**
1. Call `get_session_model(paths)` — reads `"model"` from `~/.claude/settings.json`; returns `None` when absent
2. Render `"model: VALUE"` (or `"model: (unset)"`) in requested `format::`

**Algorithm (set, 3 steps):**
1. Validate `set::VALUE` via `map_model_shorthand()` — exit 1 on unknown value with all valid values named in stderr
2. Call `set_session_model(paths, model_id)` — writes (or removes) `"model"` key, preserving all other keys
3. Print `"model set: VALUE"` to stdout; exit 0

**Examples:**

```bash
clp .model
# model: sonnet

clp .model set::opus
# model set: opus

clp .model set::default
# model set: default

clp .model format::json
# {"model":"sonnet"}

clp .model set::bad
# exit 1: set:: must be one of: opus, sonnet, haiku, default; got "bad"
```

**Notes:**
- Get mode shows the raw value stored in `settings.json` — not the 4-layer resolved effective value (env var override, project config). For full resolution, use `clv .config key::model`.
- `set::default` removes the `"model"` key, restoring Claude Code's built-in default model selection.
- `set_session_model()` is shared with the `set_model::` parameter on `.account.use` and `.usage` (Feature 034). No duplication in the write path.
- `.model` appears in the "Status & info" group of `clp .help`.

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Dedicated Model Get/Set Command](../../feature/035_model_command.md) | Full specification for this command |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Lock session model without requiring a full account switch |
| 2 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Quickly inspect the active model setting |
