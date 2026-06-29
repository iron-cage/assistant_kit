# Algorithm: Subprocess Effort Resolution

### Purpose

Select the `--effort` flag value for isolated subprocess invocations, defaulting to `low` for all effort-capable models and no flag for Haiku or `keep`.

### Entry Point

`src/usage/subprocess.rs` — `resolve_effort(resolved_model, effort_param)`

### Decision Table

| `effort_param` | `resolved_model` | `--effort` injected |
|---|---|---|
| `"low"` | any | `low` |
| `"normal"` | any | `normal` |
| `"high"` | any | `high` |
| `"max"` | any | `max` |
| `"auto"` | `Specific("claude-opus-4-6")` | `low` |
| `"auto"` | `Specific("claude-sonnet-4-6")` | `low` |
| `"auto"` | `Specific("claude-haiku-4-5-20251001")` | *(none)* — Haiku has no extended thinking |
| `"auto"` | `KeepCurrent` | *(none)* — unknown model; avoid injecting effort flag |
| `"auto"` | other Specific | `low` (conservative default) |

### Rationale for `auto` → `low`

Isolated subprocesses run `["--print", "."]` keep-alive prompts. Low effort prevents extended thinking which would cause timeout without adding value.

### `imodel::keep` + `effort::auto` Interaction

When `imodel::keep`, no model is known at dispatch time → `effort::auto` resolves to no flag. This is the safest behavior: avoids injecting an effort flag for an unknown model.

### `imodel::haiku` + `effort::auto` Interaction

Haiku has no extended thinking support → no `--effort` flag. Explicit `effort::low/normal/high/max` with `imodel::haiku` pass through to the subprocess (Claude CLI may ignore or reject).

### Scope: Subprocess Effort Only

This algorithm governs the `--effort` flag for **subprocess invocations** (isolated keep-alive `["--print", "."]` calls). It is entirely distinct from **session effort** — the `effortLevel` field in `settings.json` that governs the interactive Claude session.

| Concept | Governs | Managed by | Storage |
|---|---|---|---|
| Subprocess effort (this algo) | `--effort` flag for isolated subprocess calls | `resolve_effort()` via `imodel::`/`effort::` params | transient — per invocation |
| Session effort | Thinking depth for the current interactive session | `apply_model_override()` + `set_session_effort()` | persisted — `settings.json "effortLevel"` |

**Session effort is model-coupled** (Fix BUG-322): `apply_model_override()` writes `"high"` when switching to Opus and `"low"` when switching back to Sonnet. Subprocess effort is independent — always defaults to `low` for keep-alive prompts regardless of session model.

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/026_subprocess_model_effort.md](../feature/026_subprocess_model_effort.md) | Full feature spec; `effort::` parameter values (AC-05 through AC-16) |
| [algorithm/001](001_touch_model_selection.md) | `resolved_model` input to this algorithm |
| [algorithm/002](002_session_model_override.md) | Session model + effort coupling (Fix BUG-322) |
| [feature/062_unified_session_config.md](../feature/062_unified_session_config.md) | `set_session_effort()` and `effortLevel` carry-forward |
