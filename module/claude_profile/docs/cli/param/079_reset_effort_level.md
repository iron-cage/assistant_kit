# Parameter: 79. `reset_effort_level::`

Removes the effort key from the store selected by `scope::`. New for Feature 035 — neither store had any effort reset/removal path exposed via the CLI before this parameter.

- **Default:** `0` — no reset; mode determined by `model::`/`reset_model::`/`effort_level::` presence
- **Constraints:** `0`, `1`, `false`, `true`
- **Purpose:** Undo a previously pinned effort level for the selected scope; restore that scope's default (fully automatic management on `scope::session`; unset on `scope::subprocess`).

**Values:**

| Value | Effect |
|-------|--------|
| `0` (default) | No-op for reset |
| `1` | Remove the effort key for the selected `scope::` — `effortLevel` from `~/.claude/settings.json` (session) or `effort` from `~/.clr/config.toml`'s user tier (subprocess); idempotent — exits 0 even if the key or file is already absent |

**Error cases:**
- `reset_effort_level::1 effort_level::VALUE` together (same scope) → exit 1; stderr: `effort_level:: and reset_effort_level::1 are mutually exclusive`

**Examples:**

```bash
clp .model reset_effort_level::1                        # session: remove effortLevel key
clp .model scope::subprocess reset_effort_level::1       # subprocess: remove effort key
clp .model model::opus reset_effort_level::1              # combine with an unrelated set in one call
```

**Notes:**
- `reset_model::` ([078_reset_model.md](078_reset_model.md)) and `reset_effort_level::` are independent — either or both may be present on a single call, and either may be combined with the *other* concept's set parameter.
- On `scope::session`, after reset `effortLevel` reverts to being fully automatically managed by `apply_model_override()` on the next `.usage`/`.account.use` call — there is no "auto" sentinel value for `effort_level::` itself; `reset_effort_level::1` is how a caller hands control back to the automatic system.
- On `scope::subprocess`, after reset `clr`'s own `--effort`/`CLR_EFFORT`/config-file resolution falls through to whatever tier is next (project `.clr.toml`, then hardcoded default) exactly as if `effort` had never been set.
- Requires a new `remove_session_effort()` helper in `claude_profile_core::account` for the `scope::session` case — see [Feature 035](../../feature/035_model_command.md) Design's No-Duplication Contract; `scope::subprocess` reuses the existing generic `toml_io::remove_user_tier()` against the `effort` key, no new low-level code needed there.
- Preserves all other keys in the target file.

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

*(ungrouped — `.model`-specific parameter)*

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.model`](../command/007_model.md) | Reset action: removes the effort key for the selected `scope::` |
