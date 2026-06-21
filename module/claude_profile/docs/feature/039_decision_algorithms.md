# Feature 039 — Decision Algorithm Reference

- **Purpose**: Unified reference for the six core decision algorithms that govern model selection, quota classification, next-account recommendation, and quota approximation.
- **Cross-references**: [020](020_usage_sort_strategies.md) (sort strategies, status groups), [026](026_subprocess_model_effort.md) (touch model), [027](027_account_use_post_switch_touch.md) (session model override), [036](036_account_ownership.md) (ownership gates), [038](038_usage_strategy_rotate.md) (auto-switch), [040](040_quota_measurement_history.md) (measurement history and approximation), [061](061_solo_token_conservation.md) (solo gate predicate), [062](062_unified_session_config.md) (`recommended_model()` canonical entry point for Table 2)

---

## Table 1 — Touch Model Selection

How `resolve_model()` selects the model for keep-alive touch pings.

Entry point: `subprocess.rs:29-59` (`resolve_model`).

| `seven_day_sonnet` | `resets_at` | `utilization` | Touch Model | Rationale |
|---|---|---|---|---|
| `None` | — | — | Haiku | No Sonnet tier — conserve quota |
| `Some` | `None` | any | **Sonnet** | `son_idle=true` — activate idle Sonnet window (Haiku cannot open it) |
| `Some` | `Some(...)` | < 80% | **Sonnet** | `son_available=true` — window active, > 20% remaining; use quota before window expires |
| `Some` | `Some(...)` | ≥ 80% | Haiku | `son_available=false` — Sonnet near-exhausted (≤ 20% remaining); conserve last reserves |

Utilization-aware gate at `subprocess.rs:29-59` (Fix BUG-301, TSK-311): `son_idle = son.resets_at.is_none()`, `son_available = 100.0 - son.utilization > 20.0`; selects Sonnet when `son_idle || son_available`. Falls through to Haiku when Sonnet tier absent or ≤ 20% remaining.

---

## Table 2 — Session Model Override

How `apply_model_override()` upgrades the interactive session model from Sonnet to Opus. The threshold logic is canonicalized in `recommended_model(aq)` (`format.rs`) — see [Feature 062](062_unified_session_config.md).

Entry point: `format.rs` (`recommended_model`), called by `api.rs:259-290` (`apply_model_override`) and `render.rs` footer generation.

| `seven_day_sonnet` | Sonnet left (`100 - utilization`) | Override | Rationale |
|---|---|---|---|
| `None` | — | no-op | No Sonnet tier — nothing to evaluate |
| `Some` | >= 15% | no-op | Sufficient Sonnet capacity |
| `Some` | < 15% | **Sonnet -> Opus** | Sonnet near-exhausted — preserve remaining tokens |

---

## Table 3 — Quota Status Groups

How `status_group_of()` partitions accounts into 4 fixed display groups.

Entry point: `sort.rs:31-48` (`status_group_of`).

| 5h Left | 7d Left | Group | Emoji | Position |
|---|---|---|---|---|
| > 15% | > 5% | Green | G | top |
| <= 15% | > 5% | h-exhausted | Y | 2nd |
| > 15% | <= 5% | weekly-exhausted | Y | 3rd |
| <= 15% | <= 5% (or Err) | Red | R | bottom |

Group order is fixed — `desc::` only reverses row order *within* each group, never between groups. Accounts with `result = Err(...)` are classified as Red.

Thresholds: 5h = 15% (`five_hour_left(aq) > 15.0`), 7d = 5% (`seven_day_left(aq) > 5.0`).

---

## Table 4 — Next-Account Eligibility Gates

How `find_first_eligible()` + the `extra` predicate filter candidates. An account is **skipped** when any gate fires.

Entry points: `sort_next.rs:24-35` (`find_first_eligible`), `sort_next.rs:59` (`extra` predicate).

| # | Gate | Fires when (account is skipped) | Location |
|---|---|---|---|
| 1 | Current | `is_current = true` | sort_next.rs:27 |
| 2 | Active | `is_active = true` | sort_next.rs:27 |
| 3 | Occupied | `is_occupied_elsewhere = true` | sort_next.rs:28 |
| 4 | Error | `result = Err(...)` | sort_next.rs:29 |
| 5 | h-exhausted | `five_hour.utilization >= 85%` | sort_next.rs:30 |
| 6 | Expired | `expires_at <= now` | sort_next.rs:31 |
| 7 | Weekly-exhausted | `prefer_weekly(aq, prefer) <= 5.0` | sort_next.rs:59 (extra) |
| 8 | Foreign-owned | `is_owned = false` AND `gate_ownership = true` | sort_next.rs:59 (extra) |

Gates 1-6 are in `find_first_eligible` body. Gates 7-8 are in the `extra` closure passed by `find_next_for_strategy`.

**Gate 8 context — `gate_ownership` varies by call site:**

| Call site | `gate_ownership` value | Effect |
|---|---|---|
| Auto-switch execution (`api.rs:789`) | `!params.force` | Ownership required unless `force::1` |
| Table marker render (`render.rs:53`) | `rotate && !force` | Only when rotate active and not forced |
| Footer recommendation (`render.rs:242`) | `false` (hardcoded) | Ownership never checked |

`is_owned` semantics (`types.rs:193-195`): `true` when `owner` field is empty or matches `current_identity()`; `false` when a different machine owns the account.

---

## Table 5 — Next-Account Positive Selection

How `find_next_for_strategy()` selects the **winner** from among eligible accounts.

Entry point: `sort_next.rs:46-83` (`find_next_for_strategy`).

Algorithm: `sort_indices()` -> walk sorted list -> return **first** account passing all 8 eligibility gates.

**Selection is strictly from the Green group.** Gates 5 and 7 enforce this: gate 5 (5h utilization >= 85%, i.e. 5h left <= 15%) eliminates h-exhausted and Red accounts; gate 7 (prefer_weekly <= 5.0, i.e. strategy-weighted weekly capacity <= 5%) eliminates weekly-exhausted and Red accounts. Only Green accounts (5h > 15% AND 7d > 5%) survive both gates. The 4-group sort is for display order; for switch/use selection, non-Green accounts are never eligible.

### Step 1 — Group partition (display order, selection restricted to Green)

All accounts are partitioned into 4 status groups in fixed order: Green -> h-exhausted -> weekly-exhausted -> Red. The sort strategy reorders accounts only *within* each group. Since eligibility gates restrict selection to Green, the within-Green sort order determines the winner.

### Step 2 — Within-group sort keys by strategy

| Strategy | Primary key | Direction | Secondary key | Direction | Tertiary key |
|---|---|---|---|---|---|
| `sort::name` | account name | ascending (A->Z) | — | — | — |
| `sort::renew` | `min(7d_reset_secs, sub_renewal_secs)` | ascending (soonest first) | `prefer_weekly(aq, prefer)` | ascending (lowest first) | name asc |
| `sort::renews` | `sub_renewal_secs` | ascending (soonest first) | name | ascending | — |

Key definitions:

- `7d_reset_secs` = seconds until 7d quota window resets (from `seven_day.resets_at`; `u64::MAX` if absent). Source: `sort.rs:113-116`.
- `sub_renewal_secs` = seconds until subscription billing renewal (from `renewal_at` or estimated from `org_created_at`; `u64::MAX` if absent). Source: `sort.rs:117-121`.
- `prefer_weekly` = model-aware 7d capacity via `relevant_quotas(aq, prefer).1` (`format.rs`): `prefer::any` → `min(seven_day_left, seven_day_sonnet_left)` when Sonnet present, else `seven_day_left`; `prefer::son` → `seven_day_sonnet_left` when present, else `0.0` (absent = ineligible); `prefer::opus` → `seven_day_left`.

### Step 3 — First eligible wins

Walk the sorted list from position 0. The first account that passes all 8 eligibility gates (Table 4) is the **winner** — the account marked with `->` in the table and recommended in the footer. If no account passes all gates, result is `None` (no recommendation; auto-switch returns error).

### Why `sort::renew` uses ascending `prefer_weekly` as secondary key

Lower weekly capacity = benefits most from the upcoming renewal event. An account at 10% weekly benefits more from an imminent renewal than one at 50%. This ensures the recommendation favors accounts whose renewal will have the greatest impact.

---

## Table 6 — Quota Approximation

How `approximate_utilization()` estimates quota levels when the server is unavailable (rate-limited, timeout, network error).

Entry point: `approx.rs` (`approximate_utilization`).

| Measurements (post-filter) | Degree | Method | Fallback |
|---|---|---|---|
| 0 | — | `None` | No data |
| 1 | 0 | Constant (last value) | — |
| 2 | 1 | Linear extrapolation | — |
| 3–10 | 2 | Quadratic LS (Cramer 3x3) | linear if singular |

Pre-fit: discard measurements before `window_start` (`resets_at - window_duration`: 18000s for 5h, 604800s for 7d). If `now > resets_at` → return 0.0 (window expired).

Post-fit: clamp to [0.0, 100.0]. If extrapolation > 2x measurement span → tangent-line continuation (evaluate derivative at t_max, extend linearly).

Time normalization: subtract `t_values[0]` before computing power sums to avoid f64 precision loss on large Unix timestamps.

---

### Sources

| Algorithm | Primary source | Related features |
|---|---|---|
| Touch model selection | `src/usage/subprocess.rs:29-59` | 024, 026 |
| Session model override | `src/usage/format.rs` (`recommended_model`), `src/usage/api.rs:259-290` (`apply_model_override`) | 027, 034, 062 |
| Quota status groups | `src/usage/sort.rs:22-48` | 020 |
| Eligibility gates | `src/usage/sort_next.rs:16-36, 46-83` | 020, 036, 038 |
| Positive selection | `src/usage/sort.rs:62-173`, `sort_next.rs:46-83` | 020, 038 |
| Quota approximation | `src/usage/approx.rs` | 033, 040 |
| Solo gate (fetch) | `src/usage/fetch.rs` (after G1) | 036, 061 |
| Occupied-elsewhere gate G1b (fetch) | `src/usage/fetch.rs` (after solo gate) | 036 |
| Solo gate (refresh) | `src/usage/refresh.rs` (after G2) | 036, 061 |
| Solo gate (touch) | `src/usage/touch.rs` (after G4b) | 036, 061 |
