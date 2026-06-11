# Parameter :: 36. `effort::`

Controls the effort level (`--effort` flag) injected into isolated subprocesses spawned during `touch::` and `refresh::` operations. Default is `low` for all models that support effort.

- **Default:** `auto`
- **Constraints:** `auto`, `low`, `normal`, `high`, `max`
- **Purpose:** Set subprocess effort level; `auto` uses `low` for any model, preventing extended thinking overhead in keep-alive subprocesses.

**Values:**

| Value | `--effort` injected | Note |
|-------|---------------------|------|
| `auto` (default) | `--effort low` for any model; no flag for Haiku or `imodel::keep` | Low effort avoids extended thinking in keep-alive subprocesses; Haiku has no extended thinking |
| `low` | `--effort low` always | Light effort; works on any model |
| `normal` | `--effort normal` always | Standard effort; works on any model |
| `high` | `--effort high` always | Works on both Sonnet and Opus |
| `max` | `--effort max` always | Opus-capable models only; may downgrade silently on Sonnet |

**Examples:**

```text
effort::auto     → low for any model; no flag for haiku or keep (default)
effort::low      → always --effort low
effort::normal   → always --effort normal
effort::high     → always --effort high
effort::max      → always --effort max
```

**Notes:**
- `auto` always injects `low` regardless of model (Sonnet or Opus); `imodel::haiku` → no effort flag (no extended thinking support); `imodel::keep` → no effort flag (model unknown at dispatch time). `low` prevents extended thinking, keeping isolated keep-alive subprocesses fast.
- On `.usage`: applies to both `touch::` and `refresh::` subprocess calls within the same invocation.
- On `.account.use`: applies to the single post-switch subprocess spawned when `touch::1` and the target account is idle.
- Has no effect when no subprocess is spawned (`.usage` with neither `touch::1` nor `refresh::1` active; `.account.use` with `touch::0` or target already active).
- Does not affect `format::json` output structure.

**See Also:** [feature/026_subprocess_model_effort.md](../../feature/026_subprocess_model_effort.md) for the full effort-resolution algorithm and AC criteria.

### Referenced Type

- **Fundamental Type:** `enum`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command--9-usage) | Effort level for touch/refresh subprocesses |
| 2 | [`.account.use`](../command/001_account.md#command--5-accountuse) | Effort level for post-switch idle activation subprocess |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Low-effort subprocess keeps keep-alive pings fast |
