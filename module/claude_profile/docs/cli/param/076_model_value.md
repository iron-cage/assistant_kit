# Parameter: 76. `model::`

Writes the model key for the store selected by `scope::`. New for Feature 035, replacing both the retired `set::` (formerly session-only, on `.model`) and `id::`'s model-pinning role (formerly subprocess-only, on the retired `.model.select`) with one scope-relative parameter.

- **Default:** *(omit)* — no model write this call; get mode if no other action parameter is present either
- **Constraints:** Depends on `scope::` — see Values
- **Purpose:** Set the interactive session model (`scope::session`) or the clr subprocess model preference (`scope::subprocess`).

**Values:**

| `scope::` | Accepted values | Written as |
|-----------|------------------|------------|
| `session` (default) | `opus`, `sonnet`, `haiku`, `default` | Mapped via `map_model_shorthand()` to a full model ID (`opus` → `claude-opus-4-8`, etc.); `default` removes the `model` key instead of writing one |
| `subprocess` | Any non-empty full model ID string (no allow-list — run `.models` to discover valid IDs) | Written verbatim to the `model` key |

**Validation:** `scope::session` validates against the closed 4-value shorthand set via `map_model_shorthand()` — unknown values are rejected. `scope::subprocess` validates only that the value is non-empty — no live API check; an invalid full model ID is accepted here and only rejected later, by the Claude API, when `clr` actually invokes it.

**Error cases:**
- `model::bad` (scope `session`) → exit 1; stderr: `model:: must be one of: opus, sonnet, haiku, default; got "bad"`
- `model::` (empty value, scope `subprocess`) → exit 1; stderr: `model:: must be a non-empty model ID`
- `model::VALUE reset_model::1` together (same scope) → exit 1; stderr: `model:: and reset_model::1 are mutually exclusive`

**Examples:**

```bash
clp .model model::opus                                  # session: shorthand
clp .model model::default                               # session: remove model key
clp .model scope::subprocess model::claude-opus-4-8     # subprocess: full ID
clp .model model::opus reset_effort_level::1             # combine with an unrelated reset in one call
```

**Notes:**
- `model::` and `effort_level::` ([077_effort_level.md](077_effort_level.md)) are independent — either, both, or neither may be present on a single call; only pairing a parameter with its own reset is rejected.
- `scope::session model::` shares `set_session_model()`/`map_model_shorthand()` with `set_model::` on `.account.use`/`.usage` (Feature 034) — same write primitive, distinct parameter name and command.
- `scope::subprocess model::` writes only `~/.clr/config.toml`'s user tier; the project-tier `.clr.toml` (if present) still takes precedence when `clr` itself resolves the effective model, exactly as under the retired `.model.select` design.
- A manually pinned `scope::session model::` value can be silently overwritten by the next automatic `apply_model_override()` call inside `.usage`/`.account.use` — see [Feature 035](../../feature/035_model_command.md) Design's clobbering caveat.

### Referenced Type

- **Fundamental Type:** `string`

### Referenced Parameter Groups

*(ungrouped — `.model`-specific parameter)*

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.model`](../command/007_model.md) | Write action: sets the model key for the selected `scope::` |
