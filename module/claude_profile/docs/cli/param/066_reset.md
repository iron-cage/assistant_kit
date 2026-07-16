# Parameter: 66. `reset::`

Removes the `model` preference from `~/.clr/config.toml`'s user tier, reverting clr subprocess model selection to `ISOLATED_DEFAULT_MODEL`. Present on `.model.select` only.

- **Default:** `0` — no reset; mode on `.model.select` determined by `id::` presence
- **Constraints:** `0`, `1`, `false`, `true`
- **Purpose:** Undo a previously pinned model; restore clr to workspace default subprocess model selection.

**Values:**

| Value | Effect |
|-------|--------|
| `0` (default) | No-op for reset; `.model.select` mode determined by `id::` presence |
| `1` | Remove `model` from `~/.clr/config.toml`'s user tier; exits 0 even if file or key is absent (idempotent) |

**Error cases:**
- `reset::1 id::VALUE` → exit 1; stderr: `id:: and reset::1 are mutually exclusive`

**Examples:**

```bash
clp .model.select reset::1   # revert to ISOLATED_DEFAULT_MODEL
```

**Notes:**
- `reset::1` is idempotent: running it when no preference is set, or when `~/.clr/config.toml` is absent, exits 0 without error.
- After reset, `clr run/ask/isolated/refresh` uses `ISOLATED_DEFAULT_MODEL` (workspace constant in `claude_runner_core/src/isolated.rs`, currently `"opus"`).
- Preserves all other keys in `~/.clr/config.toml` when removing `model`.

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

*(ungrouped — `.model.select`-specific parameter)*

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.model.select`](../command/007_model.md) | Mode selector on `.model.select`: `reset::1` with no `id::` = reset mode |
