# Commands: Model

Unified session + subprocess model and effort management command.

---

### Command: 18. `.model`

Get or set the model and effort level for either of the two persisted model/effort stores this workspace maintains, selected by `scope::`: the Claude Code interactive session (`~/.claude/settings.json`, `scope::session`, default) or the clr subprocess-execution preference (`~/.clr/config.toml`'s user tier, `scope::subprocess`). Without any of `model::`/`effort_level::`/`reset_model::`/`reset_effort_level::`, prints both current values for the selected scope together with the resolved absolute file path. **Absorbs the former `.model.select` command** — see Command 20 below for its retirement stub.

-- **Parameters:** [`scope::`](../param/075_scope.md), [`model::`](../param/076_model_value.md), [`effort_level::`](../param/077_effort_level.md), [`reset_model::`](../param/078_reset_model.md), [`reset_effort_level::`](../param/079_reset_effort_level.md), [`format::`](../param/002_format.md)
-- **Exit:** 0 (success) | 1 (usage: unknown `scope::` value; unknown `model::`/`effort_level::` value for the selected scope; `model::`+`reset_model::1` together; `effort_level::`+`reset_effort_level::1` together; empty `model::` on `scope::subprocess`) | 2 (runtime: HOME not set)

**Syntax:**

```bash
clp .model                                              # get, scope::session (default)
clp .model scope::subprocess                            # get, scope::subprocess
clp .model model::opus                                  # set session model (shorthand)
clp .model effort_level::high                            # set session effort
clp .model reset_model::1                                # remove session model key
clp .model reset_effort_level::1                          # remove session effort key
clp .model scope::subprocess model::claude-opus-4-8      # set subprocess model (full ID)
clp .model scope::subprocess effort_level::max            # set subprocess effort
clp .model scope::subprocess reset_model::1               # remove subprocess model key
clp .model scope::subprocess reset_effort_level::1         # remove subprocess effort key
clp .model model::opus reset_effort_level::1               # combine: set model + reset effort, one call
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `scope::` | `enum` | `session` | Backing store selector: `session` (`~/.claude/settings.json`) or `subprocess` (`~/.clr/config.toml` user tier) |
| `model::` | `string` | *(omit)* | Model to write for the selected scope: `opus`/`sonnet`/`haiku`/`default` (session, shorthand) or any non-empty full model ID (subprocess) |
| `effort_level::` | `string` | *(omit)* | Effort to write for the selected scope: `low`/`normal`/`high`/`max` (session) or `low`/`medium`/`high`/`max` (subprocess — note `medium`, not `normal`) |
| `reset_model::` | `bool` | `0` | Remove the model key for the selected scope; mutually exclusive with `model::` |
| `reset_effort_level::` | `bool` | `0` | Remove the effort key for the selected scope; mutually exclusive with `effort_level::` |
| `format::` | [`OutputFormat`](../type/002_output_format.md) | `text` | Output format (get mode only) |

**Mode dispatch:**

| Any of `model::`/`effort_level::`/`reset_model::1`/`reset_effort_level::1` present? | Mode |
|---|---|
| No | get — read model + effort together for `scope::`, print with resolved absolute path |
| Yes | write — apply each present action independently against `scope::`'s store; actions may combine freely across the model/effort concepts (never within the same concept's set+reset pair) |

**Algorithm (get, 3 steps):**
1. Resolve `scope::` to its absolute path (`ClaudePaths::settings_file()` for `session`; `resolve_subprocess_config_path()` for `subprocess`)
2. Read model + effort keys for that scope: `get_session_model()`+`get_session_effort()` (session) or `toml_io::get_tiered()` against `model`/`effort` keys (subprocess)
3. Render `scope`, absolute `path`, `model`, `effort_level` together in requested `format::`; absent values print `(unset)` (text) or `null` (JSON)

**Algorithm (write, 4 steps):**
1. Reject `model::`+`reset_model::1` together, or `effort_level::`+`reset_effort_level::1` together — exit 1 naming the conflicting pair
2. Validate every present `model::`/`effort_level::` value against the selected scope's own vocabulary (see Parameters table) — exit 1 on the first invalid value with that scope's full valid-values list in stderr
3. Apply each present action independently: `model::` → `set_session_model()`/`set_user_tier(key="model")`; `reset_model::1` → `set_session_model(None)`/`remove_user_tier(key="model")`; `effort_level::` → `set_session_effort()`/`set_user_tier(key="effort")`; `reset_effort_level::1` → `remove_session_effort()`/`remove_user_tier(key="effort")`
4. Print one confirmation line per applied action, each naming the resolved absolute path and scope; exit 0

**Examples:**

```bash
clp .model
# scope: session (/home/alice/.claude/settings.json)
# model: sonnet
# effort_level: high

clp .model scope::subprocess
# scope: subprocess (/home/alice/.clr/config.toml)
# model: claude-sonnet-5
# effort_level: (unset)

clp .model model::opus
# model: opus  →  /home/alice/.claude/settings.json (session)

clp .model scope::subprocess model::claude-opus-4-8 effort_level::max
# model: claude-opus-4-8  →  /home/alice/.clr/config.toml (subprocess)
# effort_level: max  →  /home/alice/.clr/config.toml (subprocess)

clp .model format::json
# {"scope":"session","path":"/home/alice/.claude/settings.json","model":"sonnet","effort_level":"high"}

clp .model model::bad
# exit 1: model:: must be one of: opus, sonnet, haiku, default; got "bad"

clp .model scope::subprocess effort_level::normal
# exit 1: effort_level:: must be one of: low, medium, high, max; got "normal"

clp .model model::opus reset_model::1
# exit 1: model:: and reset_model::1 are mutually exclusive
```

**Notes:**
- Get mode shows the raw value stored in the selected scope's file — not any further-resolved effective value (env var override, project config for `subprocess`). For `clr`'s own full CLI-flag/env/config resolution, see `clr --help`; for the session's 4-layer equivalent, use `clv .config key::model`.
- `model::default` (session scope only) removes the `"model"` key, restoring Claude Code's built-in default model selection.
- `reset_model::1`/`reset_effort_level::1` on `scope::subprocess` are idempotent — exit 0 whether or not the key or file existed beforehand.
- `set_session_model()` is also called by `set_model::` on `.account.use`/`.usage` (Feature 034) — same write primitive, distinct command/parameter, not merged into `.model`. See Referenced Command Group below.
- A manually pinned `model::`/`effort_level::` on `scope::session` is not guaranteed durable: `apply_model_override()` (Feature 062) overwrites both keys unconditionally on the next `.usage`/`.account.use` call that reaches it, regardless of whether either value changed.
- `effort_level::` is deliberately not named `effort::` — that string is already used by `.usage`/`.account.use`/`.accounts` for an unrelated, never-persisted, differently-valued touch/refresh override (Feature 026). See Feature 035's Design section.
- `.model` appears in the "Status & info" group of `clp .help`.

### Referenced Command Group

Evaluated against `.account.use`, `.usage`, and `.models` under the strict [command_group](../command_group/readme.md) identity test (same dispatch function, same parameter set) — does not qualify against any of the three. `model_routine()` (`src/commands/model.rs:88`) has zero cross-calls with `account_use_routine()` (`src/commands/account_ops.rs:19`) or `usage_routine()` (`src/usage/api.rs:78`). The `set_session_model()` write primitive is shared at the `claude_profile_core` layer (called from `model_routine`, `account_use_routine`, and `usage_routine` alike) but that is ordinary lower-layer reuse, not evidence of shared dispatch or a shared parameter set — `.model` and `.account.use`/`.usage` remain three distinct commands with three non-overlapping parameter surfaces. See [`command_group/readme.md`](../command_group/readme.md) Evaluated, Not Qualifying for the full citation-backed analysis.

`.model` absorbing `.model.select`'s command name and parameters (this feature) is a **separate decision from, and does not itself satisfy,** the Representation Absorption Test above — `command_group/readme.md`'s own evaluation of `.model`/`.model.select` (before this merge) found zero shared handler and zero shared parameters beyond `format::`, and explicitly did not qualify them for grouping. This merge was applied anyway by direct design instruction, as a single CLI verb with an explicit `scope::` router over what remain two distinct backing stores and two distinct write paths internally — not a claim that the Representation Absorption Test's verdict was wrong. See [Feature 035](../../feature/035_model_command.md) Design section ("Why one command, `scope::`-routed, instead of two") and `command_group/readme.md`'s updated Groups table for the current, post-merge state.

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Unified Model & Effort Command](../../feature/035_model_command.md) | Full specification for this command |
| 2 | [Subprocess Model Select Command](../../feature/069_model_select_command.md) | Superseded — historical standalone design for the `scope::subprocess` half |

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

### Command: 20. `.model.select` *(removed — Feature 035; merged into `.model`)*

**Fully removed as a standalone command (Feature 035).** `.model.select`'s `id::`/`reset::` parameters and `~/.clr/config.toml` write path are absorbed into `.model` via `scope::subprocess`. Kept registered only as a stub that prints a migration message and exits 1 (mirrors the `.account.assign`/`.account.unclaim` precedent above). Use `.model scope::subprocess model::VALUE` (or `reset_model::1`) instead.

```bash
clp .model scope::subprocess model::claude-opus-4-8   # was: .model.select id::claude-opus-4-8
clp .model scope::subprocess reset_model::1             # was: .model.select reset::1
clp .model scope::subprocess                            # was: .model.select (get)
```

See [Feature 035](../../feature/035_model_command.md) for the current unified design and [Feature 069](../../feature/069_model_select_command.md) for the retired standalone design this replaces.
