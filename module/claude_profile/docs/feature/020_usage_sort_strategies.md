# Feature: Usage Sort Strategies

### Scope

- **Purpose**: Provide configurable row ordering and account recommendation in `.usage` output, optimized for distinct operational workflows.
- **Responsibility**: Documents the `sort::`, `desc::`, and `prefer::` parameters on `.usage`, including the 3 sort strategies (`name`, `renew`, `renews`), the `renew` default, the four-group status partition, and the footer recommendation.
- **In Scope**: Sort strategies (`name`, `renew`, `renews`), direction control (`desc::`), model preference for weekly quota selection (`prefer::`), context-sensitive `desc::` defaults per strategy, four-group status partition (🟢 Green → 🟡 h-exhausted → 🟡 weekly-exhausted → 🔴 Red, applied before sort within each group), `renew` as the default strategy, footer recommendation driven by `sort::` (top eligible account shown in footer's `Next (strategy):` line), single-strategy footer.
- **Out of Scope**: Row rendering (→ 009_token_usage.md), `.account.rotate` selection (→ 008_auto_rotate.md), `live::` monitor loop mechanics (→ 018_live_monitor.md).

### Design

`.usage` accepts a `sort::` parameter to control row ordering and the footer recommendation. The default (`sort::renew`) puts accounts with the soonest quota renewal event at the top — maximizing throughput by consuming quota that will be replenished first. Alphabetical ordering (`sort::name`) is available for positional stability, especially in `live::1` monitor mode. `sort::` is a single parameter — no separate `next::` parameter exists.

**Four-group status partition:** Regardless of the chosen sort strategy, accounts are first partitioned into four status groups (see [dictionary](../cli/002_dictionary.md#status-groups)): 🟢 Green (both available) → 🟡 h-exhausted (5h exhausted, 7d available) → 🟡 weekly-exhausted (5h available, 7d exhausted) → 🔴 Red (both exhausted or error). Group order is fixed — sort strategy applies within each group only. `desc::1` reverses row order within each group but never changes group order. This ensures healthy accounts always appear above exhausted or errored accounts, regardless of sort direction or strategy.

**The `prefer::` parameter** determines which weekly quota column is used by the `sort::renew` within-group tiebreak and the recommendation eligibility gate. It does **not** affect the four-group status partition — group membership always uses raw `7d Left` (AC-12).

| Value | Weekly column used | When |
|-------|-------------------|------|
| `any` (default) | `min(7d Left, 7d(Son))` | Conservative — whichever limit is more constrained |
| `opus` | `7d Left` | User intends to run Opus — only overall weekly quota matters |
| `sonnet` | `7d(Son)` | User intends to run Sonnet — Sonnet-specific weekly cap matters |

**The `desc::` parameter** controls sort direction. Each strategy has a context-sensitive default:

| Strategy | `desc::` default | Natural direction |
|----------|-----------------|-------------------|
| `name` | `0` (ascending) | A→Z reading order |
| `renew` | `0` (ascending) | Soonest quota event on top |
| `renews` | `0` (ascending) | Soonest billing renewal on top |

#### Strategy 1: `sort::name`

Alphabetical by account name, ascending. Stable positional layout across refreshes — useful for `live::1` where rows jumping every cycle would be disorienting.

#### Strategy 2: `sort::renew`

**Goal:** Use accounts whose next quota renewal event fires soonest — either a weekly (7d) window reset or a subscription billing cycle.

**Algorithm:**
1. **h-exhausted** accounts (`5h Left` ≤ 15%): sunk to bottom of their status group.
2. Remaining accounts sorted by `min(7d Reset, ~Renews)` countdown ascending (soonest renewal event first; absent timers → placed at end of non-exhausted).
3. Tiebreak: `7d Left` (prefer-aware, via `prefer::`) ascending (among same renewal time, drain more weekly-depleted first); final tiebreak: alphabetical name.

**Rationale:** Both the weekly window reset and the subscription billing cycle replenish quota. Whichever fires first is the more relevant event for deciding which account to consume now — it can be freely drained because its budget will be replenished soonest.

#### Strategy 3: `sort::renews`

**Goal:** Order accounts by subscription renewal timer so accounts whose billing cycle renews soonest appear first.

**Algorithm:**
1. Sort all accounts by `renewal_secs(aq.renewal_at, aq.org_created_at, now)` ascending.
2. Accounts without subscription data (no `renewal_at` and no `org_created_at` billing day) score `u64::MAX` and appear last.
3. Tiebreak: alphabetical name.

**Rationale:** Useful for seeing which accounts will have their full weekly quota restored soonest through a billing cycle.

### Recommendation

`sort::` drives the footer recommendation — the top eligible account in the sort order is shown in the footer's `Next (<strategy>)` line. No separate `next::` parameter exists. The flag column shows `✓`, `*`, `@`, or blank.

**Eligibility:** non-current, non-active, non-occupied, not h-exhausted (`5h Left > 15%`), not weekly-exhausted (`prefer_weekly > 5.0`), valid quota data, `expires_in_secs > 0`. When no eligible account exists, the footer recommendation line is omitted.

**Footer:** The footer has two `·`-delimited, column-aligned lines (omitted when 0 or 1 valid accounts): (1) `Current · <name> · <model>/<effort> · N/N` — the `✓` account, session model/effort from `settings.json`, valid/total count; (2) `Next (<strategy>) · <name> · <model>/<effort> · <metric>` — the recommendation for the active strategy. When `session_effort` is absent from `settings.json`, the effort suffix and slash are omitted from the Next line. The metric uses the same `→ Next` format as the table column — `in {duration} +7d` (7d reset is soonest) or `in {duration} $ren` / `~in {duration} $ren` (renewal is soonest). The model label shows the session model after switching: `opus` when the account's `seven_day_sonnet` exists and `sonnet_left < 15%` (override fires, via `recommended_model()`); `sonnet` otherwise. Column padding aligns `·` delimiters vertically across both lines. See [009_token_usage.md AC-10](009_token_usage.md) and [062_unified_session_config.md](062_unified_session_config.md).

### Acceptance Criteria

- **AC-01**: `sort::renew` (default) sorts rows by `min(7d Reset, ~Renews)` countdown ascending within each status group; tiebreak is `prefer_weekly` ascending; final tiebreak is alphabetical name. `sort::name` sorts alphabetically. When `sort::` is omitted, `renew` is used.
- **AC-02**: `sort::renews` sorts by subscription renewal timer ascending; accounts without subscription data are placed last; tiebreak is alphabetical name.
- **AC-03**: `desc::1` reverses the sort direction within each status group; `desc::0` uses the strategy's natural direction. The four-group status partition (🟢 → 🟡 h-exhausted → 🟡 weekly-exhausted → 🔴) is never reversed by `desc::`.
- **AC-04**: Each strategy has a context-sensitive `desc::` default: `name`→`0`, `renew`→`0`, `renews`→`0`.
- **AC-05**: `prefer::any` (default) uses `min(7d Left, 7d(Son))` when Sonnet tier present, else raw `7d Left`; `prefer::opus` uses raw `7d Left`; `prefer::sonnet` uses `7d(Son)` when present, else `0.0` (absent Sonnet tier = ineligible for recommendation).
- **AC-06**: `prefer::` affects `sort::renew` (secondary tiebreak key).
- **AC-07**: Invalid `sort::` value exits 1 with an error naming the valid values (`name`, `renew`, `renews`).
- **AC-08**: Invalid `prefer::` value exits 1 with an error naming the valid values.
- **AC-09**: `sort::` drives both row ordering and the footer recommendation. The top eligible account in the active sort order is shown in the footer's `Next (<strategy>)` line. The flag column shows `✓`, `*`, `@`, or blank — no `→` marker. The footer has two `·`-delimited lines: `Current` (identifying the `✓` account, with `model/effort` from `settings.json`) and `Next (<strategy>)` (recommendation, with `model/effort` where effort is the carry-forward session effort — omitted when `session_effort` is absent from `settings.json`). No separate `next::` parameter exists.
- **AC-10**: `sort::` and `desc::` work correctly with `live::1` — sort order is stable within each refresh cycle. Status group order is fixed across cycles.
- **AC-11**: `format::json` output is NOT affected by `sort::` or `desc::` — `render_json` preserves the input slice order without re-sorting (alphabetical in practice; stable schema for pipeline consumers).
- **AC-12**: Four-group status partition is applied universally before any sort strategy. Accounts are partitioned into: 🟢 Green (`5h Left > 15%` and `7d Left > 5%`), 🟡 h-exhausted (`5h Left ≤ 15%` and `7d Left > 5%`), 🟡 weekly-exhausted (`5h Left > 15%` and `7d Left ≤ 5%`), 🔴 Red (both exhausted or error). Sort strategy applies within each status group. See [dictionary](../cli/002_dictionary.md#status-groups).

### Features

| File | Relationship |
|------|--------------|
| [009_token_usage.md](009_token_usage.md) | Base `.usage` algorithm; `●` composite status emoji; per-column emoji |
| [018_live_monitor.md](018_live_monitor.md) | `live::1` continuous mode |
| [024_session_touch.md](024_session_touch.md) | Session touch — activates idle accounts |
| [062_unified_session_config.md](062_unified_session_config.md) | Adds `/{session_effort}` suffix to the footer `Next` line recommendation display |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/025_sort.md](../cli/param/025_sort.md) | `sort::` parameter specification |
| [cli/param/026_desc.md](../cli/param/026_desc.md) | `desc::` parameter specification |
| [cli/param/027_prefer.md](../cli/param/027_prefer.md) | `prefer::` parameter specification |

### Bugs

| File | Relationship |
|------|--------------|
| `task/claude_profile/bug/259_sort_non_deterministic_when_all_keys_tied.md` | BUG-259 ✅ Fixed: `sort_indices` all `sort_by` closures missing final name tiebreaker — non-deterministic row order when all numeric keys tie |
| `task/claude_profile/bug/299_status_group_of_prefer_weekly_boundary.md` | BUG-299 ✅ Fixed: `status_group_of()` used `prefer_weekly` for group boundary — fix: `sort.rs:35` changed to `seven_day_left( aq ) > 5.0`; `prefer` param removed from signature (TSK-301) |

### Sources

| File | Relationship |
|------|--------------|
| `src/usage/sort.rs`, `src/usage/sort_next.rs`, `src/usage/params.rs` | Sort implementation; recommendation selection; strategy parsing |
