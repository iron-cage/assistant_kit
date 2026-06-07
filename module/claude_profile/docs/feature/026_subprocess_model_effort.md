# Feature: Subprocess Model and Effort Control

### Scope

- **Purpose**: Allow `.usage` and `.account.use` to configure which Claude model and effort level are used by isolated subprocesses spawned during `touch::` and `refresh::` operations, with a per-account automatic selection mode based on remaining weekly Sonnet quota.
- **Responsibility**: Documents the `imodel::` and `effort::` parameters, the `auto` model-selection algorithm (20% `7d(Son)` threshold), the effort resolution rule (`low` for all models to avoid extended thinking overhead), and the interaction with `IsolatedModel` in `claude_runner_core`.
- **In Scope**: `imodel::` parameter with 5 values (`auto`, `sonnet`, `opus`, `keep`, `haiku`); `effort::` parameter with 5 values (`auto`, `high`, `max`, `low`, `normal`); `auto` model-selection logic reading per-account `7d(Son)` from already-fetched quota data; `auto` effort resolution (`low` for all models that support effort; no flag for haiku or keep — no extended thinking overhead in isolated subprocesses); fallback rules when `7d(Son)` is unavailable; application to `touch::` and `refresh::` subprocess calls on `.usage`, and to the single post-switch subprocess on `.account.use`; no effect on `format::json` output.
- **Out of Scope**: `run_isolated()` internals (-> `claude_runner_core/src/isolated.rs`); `IsolatedModel` type definition (-> `claude_runner_core`); subprocess timeout (-> 024_session_touch.md, 017_token_refresh.md); endurance qualification (-> 020_usage_sort_strategies.md).

### Design

`.usage` and `.account.use` accept `imodel::` and `effort::` parameters that control the Claude binary invocation for isolated subprocesses. On `.usage`, these apply to both `touch::` subprocess calls (session activation) and `refresh::` subprocess calls (OAuth token refresh on auth error). On `.account.use`, they apply to the single post-switch subprocess spawned when `touch::1` and the target account is idle.

**`imodel::` — isolated subprocess model:**

| Value | Model injected via `--model` | When to use |
|-------|------------------------------|-------------|
| `auto` (default) | Per-account: `claude-sonnet-4-6` if account's `7d(Son) ≥ 20%`, else `claude-opus-4-6` | Automatically preserves Sonnet quota when running low |
| `sonnet` | `claude-sonnet-4-6` always | Force Sonnet regardless of quota state |
| `opus` | `claude-opus-4-6` always | Force Opus regardless of quota state |
| `haiku` | `claude-haiku-4-5-20251001` always | Force Haiku — lightweight model; note: no extended thinking support |
| `keep` | No `--model` flag injected | Preserve whatever model the Claude binary would normally select |

**`auto` model-selection algorithm:**

For each account being touched or refreshed, the model is chosen from the already-fetched quota data (available before the subprocess is spawned):

```
fn resolve_model(account_quota, imodel_param) -> IsolatedModel:
    if imodel_param == "sonnet":
        return Specific("claude-sonnet-4-6")
    if imodel_param == "opus":
        return Specific("claude-opus-4-6")
    if imodel_param == "haiku":
        return Specific("claude-haiku-4-5-20251001")
    if imodel_param == "keep":
        return KeepCurrent
    // auto:
    sonnet_pct = account_quota.seven_day_sonnet_left_pct  // 100.0 - utilization
    if sonnet_pct is Some(pct) AND pct >= 20.0:
        return Specific("claude-sonnet-4-6")   // plenty of Sonnet headroom
    else:
        return Specific("claude-opus-4-6")     // Sonnet low or unavailable; use Opus
```

**Fallback when `7d(Son)` is unavailable** (accounts whose quota fetch failed — only relevant for `refresh::` subprocesses on auth-error accounts): fall back to `claude-opus-4-6`. When quota data is absent, Sonnet headroom cannot be confirmed ≥20%; Opus is the conservative safe choice. The `else` branch of the algorithm handles both the `<20%` and `unavailable` cases uniformly.

**`effort::` — isolated subprocess effort level:**

| Value | `--effort` flag injected | Note |
|-------|--------------------------|------|
| `auto` (default) | `--effort low` for any model that supports effort; no flag for Haiku or `keep` | Low effort avoids extended thinking in keep-alive subprocesses; Haiku has no extended thinking |
| `low` | `--effort low` always | Light effort; works on any model |
| `normal` | `--effort normal` always | Standard effort; works on any model |
| `high` | `--effort high` always | Works on both Sonnet and Opus |
| `max` | `--effort max` always | Opus-capable models only; using with Sonnet may downgrade silently |

**`auto` effort resolution:**

```
fn resolve_effort(resolved_model, effort_param) -> Option<&str>:
    if effort_param == "low":
        return Some("low")
    if effort_param == "normal":
        return Some("normal")
    if effort_param == "high":
        return Some("high")
    if effort_param == "max":
        return Some("max")
    // auto:
    match resolved_model:
        Specific("claude-opus-4-6")            => Some("low")
        Specific("claude-sonnet-4-6")          => Some("low")
        Specific("claude-haiku-4-5-20251001")  => None   // Haiku has no extended thinking
        KeepCurrent                            => None   // unknown model; inject no effort flag
        _                                      => Some("low")   // conservative default
```

**`imodel::keep` + `effort::auto` interaction:** When `imodel::keep`, no model is known at dispatch time; `effort::auto` resolves to no `--effort` flag (safest: avoids injecting an effort flag for an unknown model; `low` is safe for all models but the model identity is needed to confirm effort support).

**`imodel::haiku` + `effort::auto` interaction:** Haiku has no extended thinking support; `effort::auto` resolves to no `--effort` flag. Explicit `effort::low`, `effort::normal`, `effort::high`, or `effort::max` with `imodel::haiku` pass through as-is — the flag is forwarded to the subprocess; Claude CLI may ignore or reject it if haiku does not support that effort level.

**Subprocess argument construction** (in `usage.rs`, before `run_isolated()` call):

```
let resolved_model = resolve_model(account_quota, imodel_param);
let effort_opt = resolve_effort(&resolved_model, effort_param);

let mut args = vec!["--print".to_string(), ".".to_string()];
// --model prepended by run_isolated() via IsolatedModel
// --effort prepended here if present:
if let Some(effort) = effort_opt {
    args.insert(0, effort.to_string());
    args.insert(0, "--effort".to_string());
}
```

**Subprocess applicability:**
- `.usage` `touch::` subprocesses — model and effort injected for each active-window extension call
- `.usage` `refresh::` subprocesses — model and effort injected for each auth-error retry call
- `.account.use` post-switch subprocess — model and effort injected for the single idle-account activation call

**No effect on `format::json`:** `imodel::` and `effort::` control subprocess invocation, not output rendering. JSON output structure is unchanged regardless of these parameter values.

**Layer assignment:** `auto` resolution logic lives in `claude_profile/src/usage/subprocess.rs` — it reads per-account `7d(Son)` from the already-fetched quota data and resolves to a concrete `IsolatedModel` variant before calling `run_isolated()`. `claude_runner_core` always receives a concrete `IsolatedModel` (no quota awareness added to that crate).

### Acceptance Criteria

- **AC-01**: `imodel::auto` (default) selects `claude-sonnet-4-6` for accounts whose `7d(Son) ≥ 20%` and `claude-opus-4-6` for accounts whose `7d(Son) < 20%` or is unavailable.
- **AC-02**: `imodel::sonnet` always injects `--model claude-sonnet-4-6` into subprocess args regardless of quota state.
- **AC-03**: `imodel::opus` always injects `--model claude-opus-4-6` into subprocess args regardless of quota state.
- **AC-04**: `imodel::keep` injects no `--model` flag; `IsolatedModel::KeepCurrent` is passed to `run_isolated()`.
- **AC-05**: `effort::auto` (default) injects `--effort low` for Sonnet and Opus; injects no `--effort` flag when `imodel::keep` or `imodel::haiku`. Rationale: isolated subprocesses run `--print .` keep-alive prompts; low effort prevents extended thinking which would cause timeout.
- **AC-06**: `effort::high` always injects `--effort high` regardless of model.
- **AC-07**: `effort::max` always injects `--effort max` regardless of model.
- **AC-08**: On `.usage`: `imodel::` and `effort::` apply to both `touch::` and `refresh::` subprocess calls within the same invocation. On `.account.use`: they apply to the single post-switch subprocess when `touch::1` and the target account is idle.
- **AC-09**: `imodel::` and `effort::` have no effect on `format::json` output structure.
- **AC-10**: Invalid `imodel::` value exits 1 with an error naming the valid values (`auto`, `sonnet`, `opus`, `haiku`, `keep`).
- **AC-11**: Invalid `effort::` value exits 1 with an error naming the valid values (`auto`, `low`, `normal`, `high`, `max`).
- **AC-12**: `imodel::` and `effort::` parameters appear in `.usage --help` output with their default values (`auto`).
- **AC-13**: `imodel::haiku` always injects `--model claude-haiku-4-5-20251001` into subprocess args regardless of quota state. `imodel::auto` never selects Haiku — the auto algorithm selects only between Sonnet and Opus based on `7d(Son)`.
- **AC-14**: `effort::auto` with a Haiku subprocess (`imodel::haiku` or any path where resolved model is `Specific("claude-haiku-4-5-20251001")`) injects no `--effort` flag. Haiku has no extended thinking support.
- **AC-15**: `effort::low` always injects `--effort low` regardless of model.
- **AC-16**: `effort::normal` always injects `--effort normal` regardless of model.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/usage/subprocess.rs` | `resolve_model()`, `resolve_effort()`, subprocess arg construction |
| source | `src/lib.rs` | `imodel::` and `effort::` parameter registration via `register_commands()` |
| dep | `claude_runner_core` | `IsolatedModel`, `run_isolated()` — subprocess execution |
| doc | [024_session_touch.md](024_session_touch.md) | `touch::` subprocess trigger conditions; `imodel::`/`effort::` apply here |
| doc | [017_token_refresh.md](017_token_refresh.md) | `refresh::` subprocess trigger conditions; `imodel::`/`effort::` apply here |
| param | [cli/param/035_imodel.md](../cli/param/035_imodel.md) | `imodel::` parameter specification |
| param | [cli/param/036_effort.md](../cli/param/036_effort.md) | `effort::` parameter specification |
| dep | [claude_runner_core/src/isolated.rs](../../../../claude_runner_core/src/isolated.rs) | `IsolatedModel` enum definition and `run_isolated()` signature |
| doc | [027_account_use_post_switch_touch.md](027_account_use_post_switch_touch.md) | `.account.use` post-switch subprocess — also uses `resolve_model()` and `resolve_effort()` |
