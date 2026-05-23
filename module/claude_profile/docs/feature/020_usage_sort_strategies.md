# Feature: Usage Sort Strategies

### Scope

- **Purpose**: Provide configurable row ordering in `.usage` output, optimized for distinct operational workflows — long-running agent sessions, draining low-quota accounts, and exploiting upcoming quota resets.
- **Responsibility**: Documents the `sort::`, `desc::`, and `prefer::` parameters on `.usage`, including the 3 heuristic sort strategies and the `name` default.
- **In Scope**: Sort strategies (`name`, `endurance`, `drain`, `reset`), direction control (`desc::`), model preference for weekly quota selection (`prefer::`), context-sensitive `desc::` defaults per strategy, 5% exhaustion floor for `drain`/`reset` (same threshold as the `🟡` status emoji — see `009_token_usage.md` AC-18).
- **Out of Scope**: Row rendering (→ 009_token_usage.md), `→ Next` recommendation algorithm (→ 009_token_usage.md, TSK-176), `.account.rotate` selection (→ 008_auto_rotate.md), `live::` monitor loop mechanics (→ 018_live_monitor.md).

### Design

`.usage` accepts a `sort::` parameter to control row ordering. The default (`sort::name`) preserves alphabetical ordering for positional stability, especially in `live::1` monitor mode. Three heuristic strategies are available for single-shot decision-making.

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
| `reset` | `0` (ascending) | Soonest reset on top |

#### Strategy 1: `sort::name` (default)

Alphabetical by account name, ascending. Stable positional layout across refreshes — essential for `live::1` where rows jumping every cycle would be disorienting.

#### Strategy 2: `sort::endurance`

**Goal:** Find accounts that can sustain a 5h+ uninterrupted session.

**Algorithm:**
1. Classify each account as **qualified** or **unqualified**:
   - Qualified: `5h Reset` is 15–60 minutes away AND `weekly(prefer)` ≥ 30%.
   - All others: unqualified.
2. Qualified accounts are ranked first. Within qualified: highest `weekly(prefer)` → soonest `5h Reset`.
3. Unqualified accounts follow, sorted by `5h Left` descending.

**Rationale:** An account whose 5h window resets in 15–60 minutes will soon have 100% fresh session quota. Combined with ≥30% weekly runway, it can sustain a full 5-hour agent run without hitting any limit. The 15-minute floor avoids accounts that reset imminently (race condition with session start).

#### Strategy 3: `sort::drain`

**Goal:** Use up almost-exhausted accounts first, preserving fresh accounts for later.

**Algorithm:**
1. Accounts with `5h Left` ≤ 5%: marked **exhausted**, sunk to bottom.
2. Remaining accounts sorted by `5h Left` ascending (lowest first — drain these).
3. Tiebreak: highest `weekly(prefer)` (more weekly runway among equally low session accounts).

**Rationale:** When actively working at a workstation, draining low-quota accounts first avoids wasting session quota that would expire at reset. Fresh accounts are preserved for future sessions.

#### Strategy 4: `sort::reset`

**Goal:** Use accounts whose session quota refills soonest.

**Algorithm:**
1. Accounts with `5h Left` ≤ 5%: marked **exhausted**, sunk to bottom.
2. Remaining accounts sorted by `5h Reset` ascending (soonest reset first).
3. Tiebreak: `5h Left` ascending (among similar reset times, drain lower ones first).

**Rationale:** An account whose quota resets in 16 minutes can be freely drained — even if fully exhausted, it refills soon. This maximizes throughput by consuming quota that would be wasted at reset.

### Acceptance Criteria

- **AC-01**: `sort::name` (default) sorts rows alphabetically by account name, ascending. Identical to current behavior when `sort::` is omitted.
- **AC-02**: `sort::endurance` ranks qualified accounts (5h Reset 15–60 min, weekly(prefer) ≥ 30%) above unqualified accounts; within qualified, highest weekly first then soonest reset.
- **AC-03**: `sort::drain` sorts by `5h Left` ascending; accounts with ≤ 5% `5h Left` are sunk to the bottom.
- **AC-04**: `sort::reset` sorts by `5h Reset` ascending; accounts with ≤ 5% `5h Left` are sunk to the bottom.
- **AC-05**: `desc::1` reverses the sort direction; `desc::0` uses the strategy's natural direction.
- **AC-06**: Each strategy has a context-sensitive `desc::` default: `name`→`0`, `endurance`→`1`, `drain`→`0`, `reset`→`0`.
- **AC-07**: `prefer::any` (default) uses `min(7d Left, 7d(Son))` as weekly quota; `prefer::opus` uses `7d Left`; `prefer::sonnet` uses `7d(Son)`.
- **AC-08**: `prefer::` affects all strategies that reference weekly availability (endurance qualification, drain/reset tiebreaking).
- **AC-09**: Invalid `sort::` value exits 1 with an error naming the valid values.
- **AC-10**: Invalid `prefer::` value exits 1 with an error naming the valid values.
- **AC-11**: `sort::` and `desc::` do not affect the `→ Next` recommendation marker or footer — those use the recommendation algorithm from 009_token_usage.md.
- **AC-12**: `sort::` and `desc::` work correctly with `live::1` — sort order is stable within each refresh cycle.
- **AC-13**: `format::json` output is NOT affected by `sort::` or `desc::` — JSON array order remains alphabetical (stable schema for pipeline consumers).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/usage.rs` | Sort implementation in `render_text()` |
| param | [cli/param/025_sort.md](../cli/param/025_sort.md) | `sort::` parameter specification |
| param | [cli/param/026_desc.md](../cli/param/026_desc.md) | `desc::` parameter specification |
| param | [cli/param/027_prefer.md](../cli/param/027_prefer.md) | `prefer::` parameter specification |
| doc | [009_token_usage.md](009_token_usage.md) | Base `.usage` algorithm; `→ Next` recommendation; `●` status emoji (5% threshold shared) |
| doc | [018_live_monitor.md](018_live_monitor.md) | `live::1` continuous mode |
| doc | [008_auto_rotate.md](008_auto_rotate.md) | `.account.rotate` — different algorithm (highest `expires_at_ms`) |
| task | [TSK-176](../../../../task/claude_profile/176_fix_recommendation_tiebreaker.md) | `→ Next` tiebreaker fix (related but independent) |
