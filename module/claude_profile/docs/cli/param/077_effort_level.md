# Parameter: 77. `effort_level::`

Writes the effort key for the store selected by `scope::`. New for Feature 035 — neither store had a direct CLI-level effort setter before this parameter; `scope::session`'s `effortLevel` key was previously written only as an automatic side effect of `apply_model_override()`, and `scope::subprocess`'s `effort` key had no clp reader or writer at all.

- **Default:** *(omit)* — no effort write this call; get mode if no other action parameter is present either
- **Constraints:** Depends on `scope::` — see Values
- **Purpose:** Set the interactive session effort level (`scope::session`) or the clr subprocess effort preference (`scope::subprocess`).

**Values:**

| `scope::` | Accepted values | Written as |
|-----------|------------------|------------|
| `session` (default) | `low`, `normal`, `high`, `max` | Written verbatim to the `effortLevel` key |
| `subprocess` | `low`, `medium`, `high`, `max` — **note `medium`, not `normal`** | Written verbatim to the `effort` key |

**Validation:** Plain allow-list check against the four values for the selected scope — no shorthand-to-value mapping table exists or is needed (unlike `model::`, these are the literal stored values). The two scopes use genuinely different vocabularies (`normal` vs. `medium` in the third position) — passing the wrong scope's value is a validation error, not silently accepted.

**Error cases:**
- `effort_level::bad` (scope `session`) → exit 1; stderr: `effort_level:: must be one of: low, normal, high, max; got "bad"`
- `effort_level::normal` (scope `subprocess`) → exit 1; stderr: `effort_level:: must be one of: low, medium, high, max; got "normal"`
- `effort_level::VALUE reset_effort_level::1` together (same scope) → exit 1; stderr: `effort_level:: and reset_effort_level::1 are mutually exclusive`

**Examples:**

```bash
clp .model effort_level::high                             # session
clp .model scope::subprocess effort_level::medium          # subprocess
clp .model scope::subprocess model::claude-opus-4-8 effort_level::max   # combine with model:: in one call
```

**Notes:**
- **Not named `effort::`.** `.usage`/`.account.use`/`.accounts` already register a parameter literally named `effort::` ([036_effort.md](036_effort.md)) for an unrelated, never-persisted, per-invocation touch/refresh override (`auto`/`low`/`normal`/`high`/`max`, Feature 026). This parameter is deliberately named `effort_level::` — matching the literal `effortLevel` JSON key on the session side — to avoid the same word meaning two unrelated things depending on which command it's attached to. See [Feature 035](../../feature/035_model_command.md) Design's Naming/Vocabulary note.
- `model::` and `effort_level::` are independent — either, both, or neither may be present on a single call; only pairing a parameter with its own reset is rejected.
- `scope::session effort_level::` shares `set_session_effort()` with the automatic write path in `apply_model_override()` (Feature 062) — same write primitive; a manual pin here can still be silently overwritten by the next automatic rotation call, exactly like `model::`. See [Feature 035](../../feature/035_model_command.md) Design's clobbering caveat.
- `scope::subprocess effort_level::` values match `claude_runner_core::types::EffortLevel`'s four variants exactly (`Low`/`Medium`/`High`/`Max`) — this is the vocabulary `clr`'s own `--effort` flag and `CLR_EFFORT` env var also use.

### Referenced Type

- **Fundamental Type:** `string`

### Referenced Parameter Groups

*(ungrouped — `.model`-specific parameter)*

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.model`](../command/007_model.md) | Write action: sets the effort key for the selected `scope::` |
