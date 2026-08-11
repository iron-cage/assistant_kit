# Parameter: 66. `reset::`

> **Narrowed scope** (Feature 035): `reset::` no longer appears on `.model.select` — that command is retired; its subprocess-model reset is now `reset_model::1` (with `scope::subprocess`) on the unified [`.model`](../command/007_model.md) command; see [078_reset_model.md](078_reset_model.md). `reset::` remains live and unchanged on `.provider.select`, documented below.

Removes the `provider` key from `~/.clr/config.toml`'s user tier, reverting the global inference provider to `anthropic`. Present on `.provider.select`.

- **Default:** `0` — no reset; mode on `.provider.select` determined by `id::` presence
- **Constraints:** `0` or `1` (`Kind::Integer` — only integer literals are accepted; `false`/`true` are rejected as a type mismatch before the command runs, unlike the string-typed `lock::`/`reserve::`, which silently coerce non-`"1"` values to off); any integer other than `1` behaves as `0`/no-op.
- **Purpose:** Undo a previously selected provider; restore the default.

**Values:**

| Value | Effect |
|-------|--------|
| `0` (default) | No-op for reset; `.provider.select`'s mode determined by `id::` presence |
| `1` | Remove `provider` from `~/.clr/config.toml`'s user tier; exits 0 even if file or key is absent (idempotent) |

**Error cases:**
- `reset::1 id::VALUE` → exit 1; stderr: `id:: and reset::1 are mutually exclusive`

**Examples:**

```bash
clp .provider.select reset::1  # revert to anthropic
```

**Notes:**
- `reset::1` is idempotent: running it when no preference is set, or when `~/.clr/config.toml` is absent, exits 0 without error.
- After reset the effective provider is `anthropic` — the same default new accounts receive when `inference_provider::` is omitted at `.account.save` time (see [param 073](073_inference_provider.md)).
- Preserves all other keys in `~/.clr/config.toml` when removing `provider`.
- For subprocess model reset (the former `.model.select reset::1`), see [`reset_model::`](078_reset_model.md) on the unified `.model` command.

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

*(ungrouped — `.provider.select`-specific parameter)*

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.provider.select`](../command/009_provider.md) | Mode selector on `.provider.select`: `reset::1` with no `id::` = reset mode |
