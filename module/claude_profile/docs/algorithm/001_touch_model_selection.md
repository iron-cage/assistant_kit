# Algorithm: Touch Model Selection

### Purpose

Select the Claude model for isolated subprocess keep-alive pings (`touch::`, `refresh::`, post-switch touch). Defaults to Haiku to conserve quota; upgrades to Sonnet when Sonnet quota exists and would otherwise go unused.

### Entry Point

`src/usage/subprocess.rs:29-59` — `resolve_model(account_quota, imodel_param)`

### Decision Table

| `imodel_param` | `seven_day_sonnet` | `resets_at` | `100 - utilization` | Selected Model |
|---|---|---|---|---|
| `"sonnet"` | — | — | — | `claude-sonnet-4-6` (forced) |
| `"opus"` | — | — | — | `claude-opus-4-6` (forced) |
| `"haiku"` | — | — | — | `claude-haiku-4-5-20251001` (forced) |
| `"keep"` | — | — | — | `KeepCurrent` (no `--model` flag) |
| `"auto"` | `None` | — | — | `claude-haiku-4-5-20251001` (no Sonnet tier) |
| `"auto"` | `Some` | `None` | any | `claude-sonnet-4-6` (`son_idle=true` — Haiku cannot open the idle window) |
| `"auto"` | `Some` | `Some` | > 20% | `claude-sonnet-4-6` (`son_available=true` — avoid wasting quota as window expires) |
| `"auto"` | `Some` | `Some` | ≤ 20% | `claude-haiku-4-5-20251001` (Sonnet near-exhausted — conserve reserves) |

### Pseudocode

```
fn resolve_model(aq, imodel_param):
  match imodel_param:
    "sonnet" → Specific("claude-sonnet-4-6")
    "opus"   → Specific("claude-opus-4-6")
    "haiku"  → Specific("claude-haiku-4-5-20251001")
    "keep"   → KeepCurrent
    "auto"   →
      if aq.result is Ok(data) and data.seven_day_sonnet is Some(son):
        son_idle      = son.resets_at is None
        son_available = (100.0 - son.utilization) > 20.0
        if son_idle or son_available:
          return Specific("claude-sonnet-4-6")
      Specific("claude-haiku-4-5-20251001")
```

### Bug History

- **BUG-289 / BUG-290:** over-constrained gate (`five_h_running AND d7_running AND son_idle`) caused two-touch warm-up; simplified to `son_idle` alone.
- **BUG-301 (Fix TSK-311):** binary `son_idle` gate ignored utilization; added `son_available = (100 - utilization) > 20.0`.

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/026_subprocess_model_effort.md](../feature/026_subprocess_model_effort.md) | Full feature spec, `imodel::` parameter values, AC |
| [algorithm/008](008_subprocess_effort_resolution.md) | Companion effort resolution algorithm |
| [feature/024_session_touch.md](../feature/024_session_touch.md) | Touch subprocess trigger conditions |
| [feature/017_token_refresh.md](../feature/017_token_refresh.md) | Refresh subprocess also uses `resolve_model()` |
