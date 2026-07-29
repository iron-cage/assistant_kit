# Algorithm: OAuth Usage Response Dual-Source Parsing

### Scope

- **Purpose**: Define the dual-source parsing algorithm for `GET /api/oauth/usage` API responses.
- **Responsibility**: Documents the two-phase parsing strategy, `limits` array format, and backward/forward compatibility invariants.
- **In Scope**: `parse_oauth_usage()` algorithm; named-field Phase 1 and `limits`-array Phase 2; `scan_limits_for_kind()` logic; operational blind spots.
- **Out of Scope**: HTTP transport mechanics (→ `claude_quota/`); downstream algorithm behavior when data changes (→ algorithm/001, algorithm/002).

### Abstract

Parse per-model quota data from `GET /api/oauth/usage` using two response formats: the original named-field format (`seven_day_sonnet`, etc.) and the new `limits` array format introduced 2026-06-25. When Anthropic re-enables a per-model `limits` entry shaped the way this algorithm anticipated (a flat string `scope`), `clp` auto-recovers without downstream code changes — **but a live-verified 2026-07-28 observation shows the actual re-enabled shape uses a nested `scope` object instead, which the current recovery path does not match.** See § Known Limitation below.

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
| `scope` | Was always `null` as of 2026-06-25 (this table's original observation). **Superseded 2026-07-28**: a 3rd `kind` value (`"weekly_scoped"`) has since been observed with a non-null `scope` — but as an **object** (`{"model": {"id", "display_name"}, "surface"}`), not the flat per-model string (`"sonnet"`, `"opus"`) this row originally predicted. See § Further API Drift and § Known Limitation below. |
| `is_active` | `true` when this quota window is currently open; `false` for `"session"` when no 5h window is active. |

**Current state (as of 2026-06-25):** `limits` contains only `"session"` and `"weekly_all"` entries. No per-model (Sonnet/Opus) entries exist. The `seven_day_sonnet` named field is present in the response but always `null`. Per-model `limits` entries are expected to be re-enabled in a future API change.

**Further change (observed 2026-07-28):** live calls against 2 independent accounts (structurally cross-verified via full key-set comparison) show the response has evolved beyond the 2026-06-25 shape described above. See § Further API Drift (observed 2026-07-28) immediately below.

#### Further API Drift (observed 2026-07-28)

Live-verified via direct calls to `GET /api/oauth/usage` against 2 independent accounts. Full wire-level detail lives in `contract/claude_code/docs/endpoint/001_oauth_usage.md` (single source of truth for field shapes) — this section summarizes only what changed relative to the 2026-06-25 baseline above and why it matters to this algorithm.

| Area | Change since 2026-06-25 |
|------|--------------------------|
| `five_hour` / `seven_day` | Gained `limit_dollars`, `used_dollars`, `remaining_dollars` — all `null` in every response observed; reserved fields, not yet consumed by any parser |
| `extra_usage` | Gained `decimal_places`, `user_disabled` (real per-account bool), `spend_limit_reached`, `credits_ever_enabled` (real per-account bool), `daily`, `weekly` |
| Codename fields | 6th field `nimbus_quill` joined the previously-known 5 (`tangelo`, `iguana_necktie`, `omelette_promotional`, `cinder_cove`, `amber_ladder`) — still always `null` |
| `spend` | New top-level object — full shape in the contract doc. Not consumed by `clp`. |
| `member_dashboard_available` | New top-level bool. Not consumed by `clp`. |
| `seven_day_omelette` | Changed from a zeroed object (`{"utilization":0.0,"resets_at":null}`) to plain `null` |
| `limits[]` | Gained a 3rd `kind`: `"weekly_scoped"` — `scope` is a **nested object**, not the flat string this doc's field-semantics table originally predicted. See § Known Limitation. |

**Relevance filter:** of these, only the `limits[]` `"weekly_scoped"` addition affects this algorithm's parsing logic — everything else (dollar fields, `spend`, `member_dashboard_available`, new codenames, `extra_usage` growth) is either reserved/unconsumed or belongs to a different feature's concern (billing/credits UI, not quota parsing). The `"weekly_scoped"` addition is exactly the kind of forward-compatibility event Phase 2 (`scan_limits_for_kind`) was designed to auto-recover from — see § Known Limitation for why it currently does not.

#### Operational Blind Spots (current state)

With `seven_day_sonnet = None`, three algorithms produce suboptimal or unsafe behavior:

| Algorithm | Expected Behavior | Current Blind Spot | Risk |
|-----------|------------------|--------------------|------|
| `apply_model_override()` (`api.rs`) | Write `"opus"` when Sonnet ≥ 85% consumed | Never fires — `None` treated as absent tier; writes `"sonnet"` conservatively (Fix BUG-311) | Session stays in Sonnet after quota exhausted |
| `resolve_model(Auto)` (`subprocess.rs`) | Use Sonnet when quota window exists and available | Always returns Haiku — `None` treated as no Sonnet tier (algorithm/001 table row 5) | Sonnet quota wasted; touches use Haiku unnecessarily |
| `recommended_model()` (`format.rs`) | Return `"opus"` when Sonnet near-exhausted | Always returns `"sonnet"` (100% remaining assumed when `None`) | Footer always shows Sonnet; no rotation to Opus |

**Key risk:** Sonnet quota continues to be consumed by Claude Code even though the API no longer reports it. A user can exhaust Sonnet quota without any `clp` warning, override, or rotation.

**Proxy risk:** `7d Left` (all-model weekly quota) is NOT a reliable proxy for Sonnet quota. Observed gap: one account showed `7d Left = 82%` while Sonnet-specific quota was only `11%` remaining — a 71-point difference.

**Note (2026-07-28):** the new `"weekly_scoped"` entry (§ Further API Drift above) is scoped to `display_name: "Fable"`, not Sonnet or Opus — it does not resolve any of the 3 blind spots in the table above even where the parser could read it. Sonnet-specific quota remains unreported by name in every response observed to date.

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

#### Known Limitation: `scan_limits_for_kind()` Cannot Match Object-Shaped `scope`

**Status:** confirmed via direct source read (`claude_quota/src/lib.rs`), not inferred. Live-verified 2026-07-28. Not yet fixed — documented here per explicit scope decision to avoid opening a full bug-fix cycle for this pass.

**The gap:** `parse_optional_string_in_block()` (`claude_quota/src/lib.rs:435-452`) parses a JSON value textually and returns `None` for anything that isn't `null` or a quoted string:

```rust
fn parse_optional_string_in_block( block : &str, key : &str ) -> Option< String >
{
  let needle    = format!( "\"{key}\":" );
  let after_key = block.find( needle.as_str() ).map( |p| &block[ p + needle.len() .. ] )?;
  let value     = after_key.trim_start();
  if value.starts_with( "null" ) { return None; }
  if let Some( inner ) = value.strip_prefix( '"' )
  {
    let end = inner.find( '"' )?;
    return Some( inner[ ..end ].to_string() );
  }
  None  // <-- falls through here when value starts with '{' (an object)
}
```

`scan_limits_for_kind()` (`claude_quota/src/lib.rs:454-495`) calls this for the `scope` field and defaults a parse failure to an empty string:

```rust
let scope_val = parse_optional_string_in_block( block, "scope" ).unwrap_or_default();
let matched   = kind_needles.iter().any( |n| kind_val.contains( *n ) || scope_val.contains( *n ) );
```

For the live-observed `"weekly_scoped"` entry, `scope` is `{"model": {"id": null, "display_name": "Fable"}, "surface": null}` — an object, so `parse_optional_string_in_block` returns `None`, `unwrap_or_default()` produces `""`, and `"".contains(needle)` is always `false`. `kind_val` for this entry is the literal string `"weekly_scoped"`, which also does not `contains()` any of Phase 2's current needles (`"weekly_sonnet"`, `"sonnet"`) — so this specific live entry does not currently match and is correctly ignored (it is Fable-scoped, not Sonnet-scoped; see the Operational Blind Spots note above).

**Why this matters beyond the current observation:** Phase 2 exists specifically so that a *future* per-model re-enablement (e.g. `"weekly_sonnet"` or a `scope` value containing `"sonnet"`) is auto-recovered without a code change — that is this algorithm's stated forward-compatibility promise (§ Abstract). The 2026-07-28 observation is the first live evidence of *how* Anthropic actually shapes a scoped `limits` entry when they add one: via a nested `scope` object, not a flat string. If a future Sonnet/Opus re-enablement follows this same pattern — `kind` staying generic (e.g. `"weekly_scoped"`) with the model identifier living inside `scope.model.display_name` instead of in a `kind`-string Phase 2 can substring-match — Phase 2 will silently fail to recover it: no error, no warning, just a continued `None` for `seven_day_sonnet`, indistinguishable from the already-known blind spot it was built to eventually resolve.

**Not yet fixed because:** resolving this properly requires deciding how to match on a structured `scope.model.display_name` (or `.id`) value rather than a substring scan over flattened text — a real design change to `scan_limits_for_kind()`'s matching strategy, not a one-line patch. Out of scope for this documentation pass per explicit user direction; tracked here so the gap is traceable and discoverable rather than silently latent.

**Cross-references:** `contract/claude_code/docs/endpoint/001_oauth_usage.md` § Response (`limits[]` field table) and § Field Semantics (`limits[].scope`) both link forward to this entry.

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
| `claude_quota/src/lib.rs` | `parse_oauth_usage()`, `parse_period()`, `scan_limits_for_kind()`, `parse_optional_string_in_block()`, `OauthUsageData`, `PeriodUsage` |
| `../../../../contract/claude_code/docs/endpoint/001_oauth_usage.md` | Full wire-contract schema for `GET /api/oauth/usage` — single source of truth for field shapes; this doc covers only the parsing algorithm and its known limitation |
