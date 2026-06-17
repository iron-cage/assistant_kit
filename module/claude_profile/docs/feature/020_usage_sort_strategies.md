# Feature: Usage Sort Strategies

### Scope

- **Purpose**: Provide configurable row ordering in `.usage` output, optimized for distinct operational workflows — long-running agent sessions, draining low-quota accounts, and exploiting upcoming quota resets.
- **Responsibility**: Documents the `sort::`, `desc::`, and `prefer::` parameters on `.usage`, including the 6 sort strategies (`name`, `endurance`, `drain`, `renew`, `expires`, `renews`), the `renew` default, and the `next` meta-strategy.
- **In Scope**: Sort strategies (`name`, `endurance`, `drain`, `renew`, `expires`, `renews`, `next`), direction control (`desc::`), model preference for weekly quota selection (`prefer::`), context-sensitive `desc::` defaults per strategy, four-group status partition (🟢 Green → 🟡 h-exhausted → 🟡 weekly-exhausted → 🔴 Red, applied before sort within each group), `renew` as the default strategy, `next` as a meta-strategy that mirrors the active `next::` algorithm so the `→` winner always appears first.
- **Out of Scope**: Row rendering (→ 009_token_usage.md), `→ Next` recommendation algorithm (→ 023_next_account_strategies.md), `.account.rotate` selection (→ 008_auto_rotate.md), `live::` monitor loop mechanics (→ 018_live_monitor.md).

### Design

`.usage` accepts a `sort::` parameter to control row ordering. The default (`sort::renew`) puts accounts with the soonest weekly quota reset at the top — maximizing throughput by consuming quota that will be replenished first. Alphabetical ordering (`sort::name`) is available for positional stability, especially in `live::1` monitor mode. Six heuristic strategies are available for single-shot decision-making, plus `sort::next` — a meta-strategy that dynamically mirrors whatever `next::` algorithm is active, guaranteeing the `→` recommended account always appears at the top of the table.

**Four-group status partition:** Regardless of the chosen sort strategy, accounts are first partitioned into four status groups (see [dictionary](../cli/002_dictionary.md#status-groups)): 🟢 Green (both available) → 🟡 h-exhausted (5h exhausted, 7d available) → 🟡 weekly-exhausted (5h available, 7d exhausted) → 🔴 Red (both exhausted or error). Group order is fixed — sort strategy applies within each group only. `desc::1` reverses row order within each group but never changes group order. This ensures healthy accounts always appear above exhausted or errored accounts, regardless of sort direction or strategy.

**The `prefer::` parameter** determines which weekly quota column is used by all strategies that reference weekly availability:

| Value | Weekly column used | When |
|-------|-------------------|------|
| `any` (default) | `min(7d Left, 7d(Son))` | Conservative — whichever limit is more constrained |
| `opus` | `7d Left` | User intends to run Opus — only overall weekly quota matters |
| `sonnet` | `7d(Son)` | User intends to run Sonnet — Sonnet-specific weekly cap matters |

**The `desc::` parameter** controls sort direction. Each strategy has a context-sensitive default:

| Strategy | `desc::` default | Natural direction |
|----------|-----------------|-------------------|
| `name` | `0` (ascending) | A→Z reading order |
| `endurance` | `1` (descending) | Best-qualified on top |
| `drain` | `0` (ascending) | Drain targets on top |
| `renew` | `0` (ascending) | Soonest reset on top |
| `expires` | `0` (ascending) | Soonest token expiry on top |
| `renews` | `0` (ascending) | Soonest billing renewal on top |

#### Strategy 1: `sort::name`

Alphabetical by account name, ascending. Stable positional layout across refreshes — useful for `live::1` where rows jumping every cycle would be disorienting.

#### Strategy 2: `sort::endurance`

**Goal:** Find accounts that can sustain a 5h+ uninterrupted session.

**Algorithm:**
1. Classify each account as **qualified** or **unqualified**:
   - Qualified: `5h Reset` is 15–60 minutes away AND `weekly(prefer)` ≥ 30%.
   - All others: unqualified.
2. Qualified accounts are ranked first. Within qualified: highest `weekly(prefer)` → soonest `5h Reset`.
3. Unqualified accounts follow, sorted by `5h Left` descending; tiebreak by highest `weekly(prefer)` first; final tiebreak: alphabetical name.

**Rationale:** An account whose 5h window resets in 15–60 minutes will soon have 100% fresh session quota. Combined with ≥30% weekly runway, it can sustain a full 5-hour agent run without hitting any limit. The 15-minute floor avoids accounts that reset imminently (race condition with session start).

#### Strategy 3: `sort::drain`

**Goal:** Use accounts with the lowest weekly (7d) quota first, preserving high-quota accounts for later.

**Algorithm:**
1. **h-exhausted** accounts (`5h Left` ≤ 15%): sunk to bottom.
2. Remaining accounts sorted by `7d Left` (prefer-aware, via `prefer::`) ascending (lowest first — drain these).
3. Tiebreak: `5h Left` ascending (among equal weekly quota, drain lower session ones first); final tiebreak: alphabetical name.

**Rationale:** Weekly quota is the scarcest resource — it doesn't reset for 7 days. Draining accounts with the lowest 7d runway first ensures the limited weekly budget is consumed before expiry, rather than targeting only session-depleted accounts (which reset every 5 hours anyway).

#### Strategy 4: `sort::renew`

**Goal:** Use accounts whose next quota renewal event fires soonest — either a weekly (7d) window reset or a subscription billing cycle.

**Algorithm:**
1. **h-exhausted** accounts (`5h Left` ≤ 15%): sunk to bottom.
2. Remaining accounts sorted by `min(7d Reset, ~Renews)` countdown ascending (soonest renewal event first; absent timers → placed at end of non-exhausted).
3. Tiebreak: `7d Left` (prefer-aware, via `prefer::`) ascending (among same renewal time, drain more weekly-depleted first); final tiebreak: alphabetical name.

**Rationale:** Both the weekly window reset and the subscription billing cycle replenish quota. Whichever fires first is the more relevant event for deciding which account to consume now — it can be freely drained because its budget will be replenished soonest.

#### Strategy 5: `sort::next`

**Goal:** Guarantee the `→` recommended account always appears at the top of the table, regardless of which `next::` strategy is active.

**Algorithm:**
1. Resolve `sort::next` to the concrete sort strategy matching the active `next::` parameter: `next::drain` → `sort::drain`; `next::endurance` → `sort::endurance`.
2. Apply the resolved strategy's full algorithm (including h-exhausted sinking and tiebreaks).

**Rationale:** Keeps sort and recommendation aligned without requiring the user to specify both `sort::drain next::drain` (or both endurance variants) separately. When `sort::next` is used, the account the footer recommends is always visible at row 1.

#### Strategy 6: `sort::expires`

**Goal:** Order accounts by token expiry time so accounts that will expire soonest appear first.

**Algorithm:**
1. Sort all accounts by `expires_at_ms` ascending (soonest expiry first).
2. Accounts with `expires_at_ms == 0` (unknown expiry) are placed at the end (scored as `u64::MAX`).
3. Tiebreak: alphabetical name.

**Rationale:** Useful for identifying which accounts need re-authentication soonest.

#### Strategy 7: `sort::renews`

**Goal:** Order accounts by subscription renewal timer so accounts whose billing cycle renews soonest appear first.

**Algorithm:**
1. Sort all accounts by `renewal_secs(aq.renewal_at, aq.org_created_at, now)` ascending.
2. Accounts without subscription data (no `renewal_at` and no `org_created_at` billing day) score `u64::MAX` and appear last.
3. Tiebreak: alphabetical name.

**Rationale:** Useful for seeing which accounts will have their full weekly quota restored soonest through a billing cycle.

### Acceptance Criteria

- **AC-01**: `sort::drain` sorts rows by `7d Left` (prefer-aware) ascending within each tier; tiebreak is `5h Left` ascending. `sort::name` sorts alphabetically. When `sort::` is omitted, `renew` is used.
- **AC-02**: `sort::endurance` ranks qualified accounts (5h Reset 15–60 min, weekly(prefer) ≥ 30%) above unqualified accounts; within qualified, highest weekly first then soonest reset; within unqualified, highest `weekly(prefer)` first as tiebreaker when session quotas are equal; final tiebreak within any sub-group: alphabetical name.
- **AC-03**: `sort::drain` sorts by `7d Left` (prefer-aware) ascending; tiebreak is `5h Left` ascending; final tiebreak is alphabetical name; h-exhausted accounts (`5h Left` ≤ 15%) are sunk to the bottom.
- **AC-04**: `sort::renew` sorts by `min(7d Reset, ~Renews)` countdown ascending (soonest renewal event first — weekly window reset or subscription billing cycle, whichever is earlier); tiebreak is `7d Left` (prefer-aware) ascending; final tiebreak is alphabetical name; h-exhausted accounts (`5h Left` ≤ 15%) are sunk to the bottom.
- **AC-05**: `desc::1` reverses the sort direction within each status group; `desc::0` uses the strategy's natural direction. The four-group status partition (🟢 → 🟡 h-exhausted → 🟡 weekly-exhausted → 🔴) is never reversed by `desc::`.
- **AC-06**: Each strategy has a context-sensitive `desc::` default: `name`→`0`, `endurance`→`1`, `drain`→`0`, `renew`→`0`, `expires`→`0`, `renews`→`0`.
- **AC-07**: `prefer::any` (default) uses `min(7d Left, 7d(Son))` as weekly quota; `prefer::opus` uses `7d Left`; `prefer::sonnet` uses `7d(Son)`.
- **AC-08**: `prefer::` affects `sort::endurance` (qualification gate ≥ 30%), `sort::drain` (primary sort key), and `sort::renew` (tiebreak). For drain: `prefer_weekly` is the primary sort key (ascending — lowest first); `5h Left` is the secondary tiebreak; alphabetical name is the final tiebreak. For renew: `min(7d Reset, ~Renews)` countdown is the primary key; `prefer_weekly` is the secondary tiebreak (ascending); alphabetical name is the final tiebreak.
- **AC-09**: Invalid `sort::` value exits 1 with an error naming the valid values (`name`, `endurance`, `drain`, `renew`, `next`, `expires`, `renews`).
- **AC-10**: Invalid `prefer::` value exits 1 with an error naming the valid values.
- **AC-11**: `sort::` and `desc::` do not affect the `→` recommendation marker or footer — those are controlled by the `next::` parameter (see 023_next_account_strategies.md). The footer always shows three strategy recommendations (renew, endurance, drain) regardless of `sort::` or `next::` values. The `next::renew`, `next::endurance`, and `next::drain` strategies reuse the same sort algorithms but select independently from the table sort order.
- **AC-12**: `sort::` and `desc::` work correctly with `live::1` — sort order is stable within each refresh cycle. Status group order is fixed across cycles.
- **AC-13**: `format::json` output is NOT affected by `sort::` or `desc::` — `render_json` preserves the input slice order without re-sorting (alphabetical in practice since `fetch_all_quota` returns accounts alphabetically; stable schema for pipeline consumers).
- **AC-14**: Four-group status partition is applied universally before any sort strategy. Accounts are partitioned into: 🟢 Green (`5h Left > 15%` and `7d Left > 5%`), 🟡 h-exhausted (`5h Left ≤ 15%` and `7d Left > 5%`), 🟡 weekly-exhausted (`5h Left > 15%` and `7d Left ≤ 5%`), 🔴 Red (both exhausted or error). Sort strategy applies within each status group. See [dictionary](../cli/002_dictionary.md#status-groups).
- **AC-15**: `sort::next` resolves to the concrete sort strategy matching the active `next::` parameter: `next::drain` → `sort::drain`; `next::endurance` → `sort::endurance`. The resolution happens at param-parse time, so `sort::next` is never stored as a distinct strategy — it is transparent to all downstream logic. The `→` recommended account always appears at row 1 when `sort::next` is used.
- **AC-16**: `sort::expires` sorts by `expires_at_ms` ascending; accounts with `expires_at_ms == 0` (unknown expiry) are placed last. Default `desc::` is `0`.
- **AC-17**: `sort::renews` sorts by subscription renewal timer ascending via `renewal_secs()`; accounts with no subscription data are placed last (scored as `u64::MAX`). Default `desc::` is `0`.

### Features

| File | Relationship |
|------|--------------|
| [008_auto_rotate.md](008_auto_rotate.md) | `.account.rotate` — different algorithm (highest `expires_at_ms`) |
| [009_token_usage.md](009_token_usage.md) | Base `.usage` algorithm; `●` composite status emoji; per-column emoji |
| [018_live_monitor.md](018_live_monitor.md) | `live::1` continuous mode |
| [023_next_account_strategies.md](023_next_account_strategies.md) | `→ Next` recommendation strategies (reuse sort algorithms) |
| [024_session_touch.md](024_session_touch.md) | Session touch — endurance qualification requires concrete `5h_reset` |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/025_sort.md](../cli/param/025_sort.md) | `sort::` parameter specification |
| [cli/param/026_desc.md](../cli/param/026_desc.md) | `desc::` parameter specification |
| [cli/param/027_prefer.md](../cli/param/027_prefer.md) | `prefer::` parameter specification |
| [cli/param/032_next.md](../cli/param/032_next.md) | `next::` parameter — reuses sort strategy algorithms for recommendation |

### Bugs

| File | Relationship |
|------|--------------|
| `task/claude_profile/bug/259_sort_non_deterministic_when_all_keys_tied.md` | BUG-259 ✅ Fixed: `sort_indices` all `sort_by` closures missing final name tiebreaker — non-deterministic row order when all numeric keys tie |

### Sources

| File | Relationship |
|------|--------------|
| `src/usage/sort.rs`, `src/usage/params.rs` | Sort implementation; strategy parsing |
