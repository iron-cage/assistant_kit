# Parameter: 66. `reset::`

Removes a preference key from `~/.clr/config.toml`'s user tier: on `.model.select`, removes `model` (reverting clr subprocess model selection to `ISOLATED_DEFAULT_MODEL`); on `.provider.select`, removes `provider` (reverting the global inference provider to `anthropic`). Present on `.model.select` and `.provider.select`.

- **Default:** `0` — no reset; mode on the host command determined by `id::` presence
- **Constraints:** `0`, `1`, `false`, `true`
- **Purpose:** Undo a previously pinned model or selected provider; restore the host command's default.

**Values:**

| Value | Effect |
|-------|--------|
| `0` (default) | No-op for reset; host command's mode determined by `id::` presence |
| `1` | Remove `model` (`.model.select`) or `provider` (`.provider.select`) from `~/.clr/config.toml`'s user tier; exits 0 even if file or key is absent (idempotent) |

**Error cases:**
- `reset::1 id::VALUE` → exit 1; stderr: `id:: and reset::1 are mutually exclusive`

**Examples:**

```bash
clp .model.select reset::1     # revert to ISOLATED_DEFAULT_MODEL
clp .provider.select reset::1  # revert to anthropic
```

**Notes:**
- `reset::1` is idempotent: running it when no preference is set, or when `~/.clr/config.toml` is absent, exits 0 without error.
- On `.model.select`, after reset `clr run/ask/isolated/refresh` uses `ISOLATED_DEFAULT_MODEL` (workspace constant in `claude_runner_core/src/isolated.rs`, currently `"opus"`).
- On `.provider.select`, after reset the effective provider is `anthropic` — the same default new accounts receive when `inference_provider::` is omitted at `.account.save` time (see [param 073](073_inference_provider.md)).
- Preserves all other keys in `~/.clr/config.toml` when removing `model`/`provider`.

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

*(ungrouped — `.model.select`/`.provider.select`-specific parameter)*

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.model.select`](../command/007_model.md) | Mode selector on `.model.select`: `reset::1` with no `id::` = reset mode |
| 2 | [`.provider.select`](../command/009_provider.md) | Mode selector on `.provider.select`: `reset::1` with no `id::` = reset mode |
