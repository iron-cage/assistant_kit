# Parameter: 78. `reset_model::`

Removes the model key from the store selected by `scope::`. New for Feature 035, replacing the retired `.model.select`'s `reset::1` (which only ever targeted the subprocess store) with a scope-relative parameter that also covers the session store for the first time.

- **Default:** `0` — no reset; mode determined by `model::`/`effort_level::`/`reset_effort_level::` presence
- **Constraints:** `0` or `1` (`Kind::Integer` — only integer literals are accepted; `false`/`true` are rejected as a type mismatch before the command runs, unlike the string-typed `lock::`/`reserve::`, which silently coerce non-`"1"` values to off); any integer other than `1` behaves as `0`/no-op.
- **Purpose:** Undo a previously pinned model for the selected scope; restore that scope's default.

**Values:**

| Value | Effect |
|-------|--------|
| `0` (default) | No-op for reset |
| `1` | Remove the model key for the selected `scope::` — `model` from `~/.claude/settings.json` (session) or `~/.clr/config.toml`'s user tier (subprocess); idempotent — exits 0 even if the key or file is already absent |

**Error cases:**
- `reset_model::1 model::VALUE` together (same scope) → exit 1; stderr: `model:: and reset_model::1 are mutually exclusive`

**Examples:**

```bash
clp .model reset_model::1                        # session: remove model key
clp .model scope::subprocess reset_model::1       # subprocess: revert to ISOLATED_DEFAULT_MODEL
clp .model reset_model::1 effort_level::high       # combine with an unrelated set in one call
```

**Notes:**
- `reset_model::1` and `reset_effort_level::1` ([079_reset_effort_level.md](079_reset_effort_level.md)) are independent — either or both may be present on a single call, and either may be combined with the *other* concept's set parameter (e.g. `reset_model::1 effort_level::high`).
- On `scope::subprocess`, after reset `clr run/ask/isolated` uses `ISOLATED_DEFAULT_MODEL` (workspace constant in `claude_runner_core/src/isolated.rs`, currently `"opus"`) — identical post-reset behavior to the retired `.model.select reset::1`.
- On `scope::session`, after reset Claude Code falls back to its own built-in default model selection — identical post-reset behavior to the retired `.model set::default`.
- Preserves all other keys in the target file.

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

*(ungrouped — `.model`-specific parameter)*

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.model`](../command/007_model.md) | Reset action: removes the model key for the selected `scope::` |
