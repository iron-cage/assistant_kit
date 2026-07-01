# Algorithm: OAuth Usage Response Dual-Source Parsing

### Scope

- **Purpose**: Define the dual-source parsing algorithm for `GET /api/oauth/usage` API responses.
- **Responsibility**: Documents the two-phase parsing strategy, `limits` array format, and backward/forward compatibility invariants.
- **In Scope**: `parse_oauth_usage()` algorithm; named-field Phase 1 and `limits`-array Phase 2; `scan_limits_for_kind()` logic; operational blind spots.
- **Out of Scope**: HTTP transport mechanics (→ `claude_quota/`); downstream algorithm behavior when data changes (→ algorithm/001, algorithm/002).

### Abstract

Parse per-model quota data from `GET /api/oauth/usage` using two response formats: the original named-field format (`seven_day_sonnet`, etc.) and the new `limits` array format introduced 2026-06-25. When Anthropic re-enables per-model quota entries in the `limits` array, `clp` auto-recovers without downstream code changes.

### Algorithm

#### Entry Point

`claude_quota/src/lib.rs` — `parse_oauth_usage(body: &str) -> Result<OauthUsageData, QuotaError>`

#### API Response Change (2026-06-24 → 2026-06-25)

Between measurement i13 (`2026-06-24T22:06Z`, last with Sonnet data) and measurement i10 (`2026-06-25T01:24Z`, first without), Anthropic restructured the `GET /api/oauth/usage` JSON response:

| Field | Before | After |
|-------|--------|-------|
| `seven_day_sonnet` | Object `{ "utilization": N, "resets_at": "..." }` | Always `null` |
| `seven_day_opus` | Absent | Present (object or `null`) |
| `limits` | Absent | Present (array of boundary entries) |
| `extra_usage` | Absent | Present |
| `spend` | Absent | Present |
| Codename fields | Absent | `tangelo`, `iguana_necktie`, `omelette_promotional`, `cinder_cove`, `amber_ladder` |

**New `limits` array — confirmed field shapes (2026-06-25):**

```json
[
  {
    "kind":      "session",
    "group":     "session",
    "percent":   2,
    "severity":  "normal",
    "resets_at": "2026-06-25T11:59:59",
    "scope":     null,
    "is_active": false
  },
  {
    "kind":      "weekly_all",
    "group":     "weekly",
    "percent":   18,
    "severity":  "normal",
    "resets_at": "2026-06-30T04:00:00+00:00",
    "scope":     null,
    "is_active": true
  }
]
```

**Field semantics:**

| Field | Semantics |
|-------|-----------|
| `kind` | Quota boundary type: known values `"session"` (5h window), `"weekly_all"` (7d all-model). Per-model values (`"weekly_sonnet"`, `"weekly_opus"`) expected when re-enabled. |
| `group` | Display grouping — `"session"` or `"weekly"`. |
| `percent` | **USED** percentage (0–100 integer). Semantically identical to `utilization` in the named-field format. Cast directly: `utilization = percent as f64`. |
| `severity` | `"normal"` / `"warning"` / `"critical"` — threshold state. Not consumed by `clp`. |
| `resets_at` | ISO-8601 UTC reset timestamp. Same format as the named-field `resets_at`. |
| `scope` | Currently always `null`. Expected to carry per-model identifier (`"sonnet"`, `"opus"`) when re-enabled. |
| `is_active` | `true` when this quota window is currently open; `false` for `"session"` when no 5h window is active. |

**Current state (as of 2026-06-25):** `limits` contains only `"session"` and `"weekly_all"` entries. No per-model (Sonnet/Opus) entries exist. The `seven_day_sonnet` named field is present in the response but always `null`. Per-model `limits` entries are expected to be re-enabled in a future API change.

#### Operational Blind Spots (current state)

With `seven_day_sonnet = None`, three algorithms produce suboptimal or unsafe behavior:

| Algorithm | Expected Behavior | Current Blind Spot | Risk |
|-----------|------------------|--------------------|------|
| `apply_model_override()` (`api.rs`) | Write `"opus"` when Sonnet ≥ 85% consumed | Never fires — `None` treated as absent tier; writes `"sonnet"` conservatively (Fix BUG-311) | Session stays in Sonnet after quota exhausted |
| `resolve_model(Auto)` (`subprocess.rs`) | Use Sonnet when quota window exists and available | Always returns Haiku — `None` treated as no Sonnet tier (algorithm/001 table row 5) | Sonnet quota wasted; touches use Haiku unnecessarily |
| `recommended_model()` (`format.rs`) | Return `"opus"` when Sonnet near-exhausted | Always returns `"sonnet"` (100% remaining assumed when `None`) | Footer always shows Sonnet; no rotation to Opus |

**Key risk:** Sonnet quota continues to be consumed by Claude Code even though the API no longer reports it. A user can exhaust Sonnet quota without any `clp` warning, override, or rotation.

**Proxy risk:** `7d Left` (all-model weekly quota) is NOT a reliable proxy for Sonnet quota. Observed gap: one account showed `7d Left = 82%` while Sonnet-specific quota was only `11%` remaining — a 71-point difference.

#### Dual-Source Parsing Algorithm

Implemented in `parse_oauth_usage()`. Phase 1 preserves backward compatibility; Phase 2 provides forward compatibility when per-model `limits` entries are re-enabled.

```
fn parse_oauth_usage(body: &str) -> Result<OauthUsageData, QuotaError>:

  // Guard: body must contain at least one period key (unchanged)
  if not (body contains "five_hour" or "seven_day" or "seven_day_sonnet"):
    return Err(ResponseParse("five_hour/seven_day/seven_day_sonnet"))

  // Phase 1: named-field parsing (backward compat — unchanged)
  five_hour        = parse_period(body, "five_hour")?
  seven_day        = parse_period(body, "seven_day")?
  seven_day_sonnet = parse_period(body, "seven_day_sonnet")?
  // Returns None when field is null — no error

  // Phase 2: limits-array fallback (forward compat — new)
  // Only runs when named field returned None
  if seven_day_sonnet.is_none():
    seven_day_sonnet = scan_limits_for_kind(body, ["weekly_sonnet", "sonnet"])

  return Ok(OauthUsageData { five_hour, seven_day, seven_day_sonnet })


fn scan_limits_for_kind(body: &str, kind_needles: &[&str]) -> Option<PeriodUsage>:
  // Find "limits":[ in body
  pos = body.find('"limits":')?
  after_limits = body[pos + len('"limits":')..].trim_start()
  if not after_limits.starts_with('['):
    return None

  // Walk the array: extract each {...} object block
  inner = after_limits[1..]  // skip '['
  loop:
    inner = inner.trim_start()
    if inner.starts_with(']') or inner.is_empty():
      break
    block = extract_object_block(inner)?  // brace-counting, reuse existing fn
    inner = inner[len(block)..].trim_start()
    if inner.starts_with(','):
      inner = inner[1..]

    // Check if "kind" value matches any needle
    kind_val = parse_optional_string_in_block(block, "kind")
    scope_val = parse_optional_string_in_block(block, "scope")
    matched = kind_needles.any(|n| kind_val.contains(n) || scope_val.contains(n))
    if not matched:
      continue

    // Extract percent (integer → f64) as utilization
    utilization = parse_f64_in_block(block, "percent")?
    resets_at   = parse_optional_string_in_block(block, "resets_at")
    return Some(PeriodUsage { utilization, resets_at })

  return None
```

**`percent` → `utilization` mapping:** The `limits` entries use `percent` (integer 0–100) for consumed quota. The named-field format uses `utilization` (f64, 0.0–100.0). Semantics are identical. Mapping: `utilization = percent as f64`. No scale conversion needed.

#### Parsing Invariants

- Named-field guard remains valid: the new response body still contains `"five_hour"`, `"seven_day"`, and `"seven_day_sonnet"` keys (the latter as `null`) — the guard passes.
- `parse_period()` returns `None` for `null` without error — no guard change needed.
- Phase 2 (`scan_limits_for_kind`) is additive — runs only when Phase 1 returned `None`; a `Some` from Phase 1 is never overridden.
- `OauthUsageData` struct is unchanged — all downstream consumers (`apply_model_override`, `resolve_model`, `recommended_model`) already handle `Some`/`None` and auto-recover when `seven_day_sonnet` becomes `Some` again.

### Algorithms

| File | Relationship |
|------|-------------|
| [algorithm/001_touch_model_selection.md](001_touch_model_selection.md) | Affected algorithm — `resolve_model(Auto)` uses `seven_day_sonnet`; blind spot row 2 above |
| [algorithm/002_session_model_override.md](002_session_model_override.md) | Affected algorithm — `apply_model_override()` uses `seven_day_sonnet`; blind spot row 1 above |

### Features

| File | Relationship |
|------|-------------|
| [feature/009_token_usage.md](../feature/009_token_usage.md) | `7d(Son)` column sourced from `OauthUsageData.seven_day_sonnet`; `recommended_model()` uses it; blind spot row 3 above |
| [feature/066_dual_source_quota_parsing.md](../feature/066_dual_source_quota_parsing.md) | Implementation feature spec — acceptance criteria for this algorithm |

### Sources

| File | Relationship |
|------|-------------|
| `claude_quota/src/lib.rs` | `parse_oauth_usage()`, `parse_period()`, `scan_limits_for_kind()`, `OauthUsageData`, `PeriodUsage` |
