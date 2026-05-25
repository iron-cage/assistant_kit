# Parameter :: 36. `effort::`

Controls the effort level (`--effort` flag) injected into isolated subprocesses spawned during `touch::` and `refresh::` operations. Default is the maximum effort level available for the selected model.

- **Type:** `enum`
- **Default:** `auto`
- **Constraints:** `auto`, `high`, `max`
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Set subprocess effort level; `auto` uses the maximum supported by the resolved model (Sonnet max = `high`, Opus max = `max`).
- **Group:** [Fetch Behavior](../param_group/003_fetch_behavior.md)

**Values:**

| Value | `--effort` injected | Note |
|-------|---------------------|------|
| `auto` (default) | `high` (Sonnet) or `max` (Opus) | Maximum for the resolved model; no flag when `imodel::keep` |
| `high` | `--effort high` always | Works on both Sonnet and Opus |
| `max` | `--effort max` always | Opus-capable models only; may downgrade silently on Sonnet |

**Examples:**

```text
effort::auto     → high for sonnet subprocesses, max for opus subprocesses (default)
effort::high     → always --effort high
effort::max      → always --effort max
```

**Notes:**
- `auto` derives from the `imodel::` resolution: Sonnet → `high`; Opus → `max`; `imodel::keep` → no effort flag (model unknown at dispatch time).
- Applies to both `touch::` and `refresh::` subprocess calls within the same `.usage` invocation.
- Has no effect when neither `touch::1` nor `refresh::1` is active (no subprocesses are spawned).
- Does not affect `format::json` output structure.

**See Also:** [feature/026_subprocess_model_effort.md](../../feature/026_subprocess_model_effort.md) for the full effort-resolution algorithm and AC criteria.
