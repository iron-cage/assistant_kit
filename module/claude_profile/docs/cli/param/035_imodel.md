# Parameter: 35. `imodel::`

Controls which Claude model is used by isolated subprocesses spawned during `touch::` and `refresh::` operations. Determines whether `--model <id>` is injected into each subprocess invocation, and which model ID to use.

- **Default:** `auto`
- **Constraints:** `auto`, `sonnet`, `opus`, `haiku`, `keep`
- **Purpose:** Preserve Sonnet quota automatically (via `auto`) or override subprocess model selection explicitly.

**Values:**

| Value | `--model` injected | Selection logic |
|-------|-------------------|-----------------|
| `auto` (default) | `claude-haiku-4-5-20251001` (default); `claude-sonnet-4-6` when `son_idle=true` OR `son_available=true` | Haiku for general session activation; Sonnet whenever Sonnet window idle (opens all dimensions simultaneously) OR window active with > 20% remaining (avoid wasting quota). Fix(BUG-289, BUG-290, BUG-301) |
| `sonnet` | `claude-sonnet-4-6` | Always Sonnet, regardless of quota |
| `opus` | `claude-opus-4-6` | Always Opus, regardless of quota |
| `haiku` | `claude-haiku-4-5-20251001` | Always Haiku — lightweight; no extended thinking (effort::auto → no --effort flag) |
| `keep` | None | No `--model` flag; Claude binary chooses the model |

**Examples:**

```text
imodel::auto     → --model claude-haiku-4-5-20251001 (default)
                   --model claude-sonnet-4-6 (when son_idle=true OR son_available=true)
imodel::sonnet   → always --model claude-sonnet-4-6
imodel::opus     → always --model claude-opus-4-6
imodel::haiku    → always --model claude-haiku-4-5-20251001
imodel::keep     → no --model flag injected
```

**Notes:**
- `auto` selects Haiku by default — conserves Sonnet and Opus quota. **Utilization-aware Sonnet gate (BUG-289/BUG-290 fix; extended BUG-301 fix):** selects Sonnet when `seven_day_sonnet` present AND either `son_idle=true` (`resets_at=None` — the 7d-Sonnet window only activates on Sonnet-family API calls; a single Sonnet touch opens all idle dimensions simultaneously) OR `son_available=true` (`(100 - utilization) > 20%` — remaining Sonnet quota must not expire unused). Falls through to Haiku when Sonnet tier absent or ≥ 80% used (≤ 20% remaining).
- On `.usage`: applies to both `touch::` and `refresh::` subprocess calls within the same invocation.
- On `.account.use`: applies to the single post-switch subprocess spawned when `touch::1` and the target account is idle.
- Has no effect when no subprocess is spawned (`.usage` with neither `touch::1` nor `refresh::1` active; `.account.use` with `touch::0` or target already active).
- Does not affect `format::json` output structure.

**See Also:** [feature/026_subprocess_model_effort.md](../../feature/026_subprocess_model_effort.md) for the full model-selection algorithm and AC criteria.

### Referenced Type

- **Fundamental Type:** `enum`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command--9-usage) | Model for touch/refresh subprocesses during quota fetch |
| 2 | [`.account.use`](../command/001_account.md#command--5-accountuse) | Model for post-switch idle activation subprocess |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Quota-preserving subprocess model during account switch |
