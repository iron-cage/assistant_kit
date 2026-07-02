# Parameter: 66. `reset::`

Removes the `subprocess_model` preference from `~/.clr/prefs.json`, reverting clr subprocess model selection to `ISOLATED_DEFAULT_MODEL`. Present on `.model.select` only.

- **Default:** `0` — no reset; mode on `.model.select` determined by `id::` presence
- **Constraints:** `0`, `1`, `false`, `true`
- **Purpose:** Undo a previously pinned model; restore clr to workspace default subprocess model selection.

**Values:**

| Value | Effect |
|-------|--------|
| `0` (default) | No-op for reset; `.model.select` mode determined by `id::` presence |
| `1` | Remove `subprocess_model` from `~/.clr/prefs.json`; exits 0 even if file or key is absent (idempotent) |

**Error cases:**
- `reset::1 id::VALUE` → exit 1; stderr: `id:: and reset::1 are mutually exclusive`

**Examples:**

```bash
clp .model.select reset::1   # revert to ISOLATED_DEFAULT_MODEL
```

**Notes:**
- `reset::1` is idempotent: running it when no preference is set, or when `~/.clr/prefs.json` is absent, exits 0 without error.
- After reset, `clr run/ask/isolated/refresh` uses `ISOLATED_DEFAULT_MODEL` (workspace constant in `claude_runner_core/src/isolated.rs`, currently `claude-opus-4-8`).
- Preserves all other keys in `~/.clr/prefs.json` when removing `subprocess_model`.

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

*(ungrouped — `.model.select`-specific parameter)*

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.model.select`](../command/007_model.md) | Mode selector on `.model.select`: `reset::1` with no `id::` = reset mode |
