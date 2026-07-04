# Commands: Model

Session model management command.

---

### Command: 18. `.model`

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

### Referenced Parameter Groups

| # | Group | Parameters Used |
|---|-------|-----------------|
| 1 | [Output Control](../param_group/001_output_control.md) | `format::` |

### Referenced Formats

| # | Format | Trigger |
|---|--------|---------|
| 1 | [text](../format/001_text.md) | `format::text` (default) |
| 2 | [json](../format/002_json.md) | `format::json` |

---

### Command: 20. `.model.select`

Get, set, or reset the subprocess model preference in `~/.clr/prefs.json`. Subprocess model controls which Claude model `clr run`, `clr ask`, `clr isolated`, and `clr refresh` use. Without parameters, prints the current pinned model. With `id::`, writes the preference. With `reset::1`, removes it.

-- **Parameters:** [`id::`](../param/064_id.md), [`reset::`](../param/066_reset.md), [`format::`](../param/002_format.md)
-- **Exit:** 0 (success) | 1 (usage: empty `id::`, or `id::` and `reset::1` together) | 2 (runtime: HOME not set)

**Syntax:**

```bash
clp .model.select                         # get
clp .model.select id::claude-opus-4-8     # set
clp .model.select reset::1                # reset to ISOLATED_DEFAULT_MODEL
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `id::` | `string` | *(omit)* | Full model ID to pin; activates set mode; non-empty required |
| `reset::` | `bool` | `0` | Remove `subprocess_model` key from `~/.clr/prefs.json`; idempotent |
| `format::` | [`OutputFormat`](../type/002_output_format.md) | `text` | Output format (get mode only) |

**Mode dispatch:**

| `id::` | `reset::` | Mode |
|--------|-----------|------|
| absent | `0` (default) | get — read `subprocess_model` from `~/.clr/prefs.json` |
| present | `0` (default) | set — validate non-empty, write to `~/.clr/prefs.json` |
| absent | `1` | reset — remove `subprocess_model` key; create or preserve file |
| present | `1` | error — exit 1; stderr: `id:: and reset::1 are mutually exclusive` |

**Algorithm (get, 2 steps):**
1. Read `~/.clr/prefs.json`; extract `subprocess_model` field; returns `None` when absent or file missing
2. Render `"model.select: VALUE"` (or `"model.select: (unset)"`) in requested `format::`

**Algorithm (set, 3 steps):**
1. Validate `id::VALUE` is non-empty — exit 1 on empty with `id:: must be a non-empty model ID` in stderr
2. Read `~/.clr/prefs.json` (or start with `{}`); set `subprocess_model = VALUE`; write back (create dir if needed)
3. Print `"model.select: VALUE (pinned)"` to stdout; exit 0

**Algorithm (reset, 3 steps):**
1. If `~/.clr/prefs.json` absent — print `"model.select: (reset to default)"` and exit 0 (idempotent)
2. Read file; remove `subprocess_model` key; preserve all other keys; write back
3. Print `"model.select: (reset to default)"` to stdout; exit 0

**Examples:**

```bash
clp .model.select
# model.select: claude-opus-4-8

clp .model.select id::claude-opus-4-8
# model.select: claude-opus-4-8 (pinned)

clp .model.select reset::1
# model.select: (reset to default)

clp .model.select format::json
# {"subprocess_model":"claude-opus-4-8"}

clp .model.select id::
# exit 1: id:: must be a non-empty model ID

clp .model.select id::claude-opus-4-8 reset::1
# exit 1: id:: and reset::1 are mutually exclusive
```

**Notes:**
- `.model.select` governs only the subprocess model for `clr run/ask/isolated/refresh`. For the interactive Claude Code session model, use `clp .model set::opus` instead.
- Run `clp .models` first to discover available full model IDs.
- After reset, `clr` uses `ISOLATED_DEFAULT_MODEL` (currently `"opus"`, defined in `claude_runner_core/src/isolated.rs`).
- `.model.select` appears in the "Status & info" group of `clp .help`.

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Model Select Command](../../feature/069_model_select_command.md) | Full specification for this command |

### Referenced Schema

| # | Schema | Role |
|---|--------|------|
| 1 | [CLR Preferences (`~/.clr/prefs.json`)](../../schema/008_clr_prefs_json.md) | Storage for `subprocess_model` preference |

### Referenced Formats

| # | Format | Trigger |
|---|--------|---------|
| 1 | [text](../format/001_text.md) | `format::text` (default) |
| 2 | [json](../format/002_json.md) | `format::json` |
