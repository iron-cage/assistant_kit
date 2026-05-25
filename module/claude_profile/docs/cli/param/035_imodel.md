# Parameter :: 35. `imodel::`

Controls which Claude model is used by isolated subprocesses spawned during `touch::` and `refresh::` operations. Determines whether `--model <id>` is injected into each subprocess invocation, and which model ID to use.

- **Type:** `enum`
- **Default:** `auto`
- **Constraints:** `auto`, `sonnet`, `opus`, `keep`
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Preserve Sonnet quota automatically (via `auto`) or override subprocess model selection explicitly.
- **Group:** [Fetch Behavior](../param_group/003_fetch_behavior.md)

**Values:**

| Value | `--model` injected | Selection logic |
|-------|-------------------|-----------------|
| `auto` (default) | Per-account | `claude-sonnet-4-6` if account's `7d(Son) ≥ 30%`; `claude-opus-4-6` otherwise |
| `sonnet` | `claude-sonnet-4-6` | Always Sonnet, regardless of quota |
| `opus` | `claude-opus-4-6` | Always Opus, regardless of quota |
| `keep` | None | No `--model` flag; Claude binary chooses the model |

**Examples:**

```text
imodel::auto     → per-account: sonnet when 7d(Son)≥30%, opus when <30% (default)
imodel::sonnet   → always --model claude-sonnet-4-6
imodel::opus     → always --model claude-opus-4-6
imodel::keep     → no --model flag injected
```

**Notes:**
- `auto` reads `7d(Son)` from the already-fetched quota data — no extra API call is made.
- Fallback when `7d(Son)` is unavailable (e.g., `refresh::` accounts with failed quota fetch): `claude-opus-4-6`. Auth-error accounts have authentication failures, not quota exhaustion; Opus is the appropriate conservative choice.
- Applies to both `touch::` and `refresh::` subprocess calls within the same `.usage` invocation.
- Has no effect when neither `touch::1` nor `refresh::1` is active (no subprocesses are spawned).
- Does not affect `format::json` output structure.

**See Also:** [feature/026_subprocess_model_effort.md](../../feature/026_subprocess_model_effort.md) for the full model-selection algorithm and AC criteria.
