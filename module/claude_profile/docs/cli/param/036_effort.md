# Parameter :: 36. `effort::`

Controls the effort level (`--effort` flag) injected into isolated subprocesses spawned during `touch::` and `refresh::` operations. Default is the maximum effort level available for the selected model.

- **Type:** `enum`
- **Default:** `auto`
- **Constraints:** `auto`, `low`, `normal`, `high`, `max`
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage), [`.account.use`](../command/001_account.md#command--5-accountuse)
- **Purpose:** Set subprocess effort level; `auto` uses the maximum supported by the resolved model (Sonnet max = `high`, Opus max = `max`).
- **Group:** [Fetch Behavior](../param_group/003_fetch_behavior.md)

**Values:**

| Value | `--effort` injected | Note |
|-------|---------------------|------|
| `auto` (default) | `high` (Sonnet) or `max` (Opus); no flag for Haiku or `imodel::keep` | Maximum for the resolved model; Haiku has no extended thinking |
| `low` | `--effort low` always | Light effort; works on any model |
| `normal` | `--effort normal` always | Standard effort; works on any model |
| `high` | `--effort high` always | Works on both Sonnet and Opus |
| `max` | `--effort max` always | Opus-capable models only; may downgrade silently on Sonnet |

**Examples:**

```text
effort::auto     → high for sonnet, max for opus, no flag for haiku or keep (default)
effort::low      → always --effort low
effort::normal   → always --effort normal
effort::high     → always --effort high
effort::max      → always --effort max
```

**Notes:**
- `auto` derives from the `imodel::` resolution: Sonnet → `high`; Opus → `max`; `imodel::haiku` → no effort flag (no extended thinking support); `imodel::keep` → no effort flag (model unknown at dispatch time).
- On `.usage`: applies to both `touch::` and `refresh::` subprocess calls within the same invocation.
- On `.account.use`: applies to the single post-switch subprocess spawned when `touch::1` and the target account is idle.
- Has no effect when no subprocess is spawned (`.usage` with neither `touch::1` nor `refresh::1` active; `.account.use` with `touch::0` or target already active).
- Does not affect `format::json` output structure.

**See Also:** [feature/026_subprocess_model_effort.md](../../feature/026_subprocess_model_effort.md) for the full effort-resolution algorithm and AC criteria.
