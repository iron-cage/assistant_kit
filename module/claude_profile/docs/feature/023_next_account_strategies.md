# Feature: Next Account Recommendation Strategies

### Scope

- **Purpose**: Provide configurable account recommendation in `.usage` output, where the `next::` parameter selects which strategy places the `→` marker on the recommended next account in the table.
- **Responsibility**: Documents the `next::` parameter and its 2 values (`endurance`, `drain`), per-strategy selection algorithms, the always-visible multi-strategy footer, and the `→` table marker.
- **In Scope**: `next::` parameter with 2 values, per-strategy recommendation algorithm, always-visible multi-strategy footer (2 strategy lines, rendered unconditionally when ≥2 valid accounts), `→` table marker on the account selected by the active `next::` strategy, interaction with `prefer::` for weekly quota selection.
- **Out of Scope**: Row ordering (-> 020_usage_sort_strategies.md), sort strategy algorithms (-> 020_usage_sort_strategies.md), `cols::` column visibility (-> 009_token_usage.md).

### Design

`.usage` accepts a `next::` parameter controlling which account receives the `→` marker in the table body — the recommended next account to switch to. The footer always shows both strategy recommendations simultaneously, providing full operational context regardless of `next::` value. Default is `next::drain`.

**Strategy table:**

| Value | Name | Selection algorithm |
|-------|------|---------------------|
| `endurance` | Endurance Top | First non-current, non-active account from endurance sort order (qualified accounts first by weekly desc then reset asc; unqualified by 5h_left desc, tiebreak weekly desc). |
| `drain` (default) | Drain Top | First non-current, non-active account from drain sort order (`prefer_weekly` ascending, h-exhausted sunk; tiebreak `5h_left` asc). |

**Recommendation eligibility:** All strategies skip accounts that are `is_current` (user is already on that session) or `is_active` (the active marker account when it differs from current). Only accounts with valid quota data and `expires_in_secs > 0` are eligible. Strategies select from all eligible accounts regardless of their composite health tier (-> 009_token_usage.md three-tier grouping) -- the tier affects table display ordering, not recommendation eligibility.

**Footer format (always shown when ≥2 valid accounts):**

The footer shows each strategy's recommendation on its own line with key qualifying metrics, regardless of the `next::` value:

```
Valid: 7 / 8   ->  Next by strategy:
  endurance  bob@example.com     100% session, 76% 7d left, expires in 7h 56m
  drain      carol@example.com   98% session, resets in 5m
```

Each footer line shows: strategy name (left-aligned, 10 chars), account name (left-aligned), key metric string. When both strategies recommend the same account, both lines appear independently (the agreement is itself useful signal). Strategy lines for which no eligible account exists are omitted rather than showing an empty line.

**`→` table marker:**

The account selected by the active `next::` strategy receives the `→` flag in the table body (flag column priority: `✓` > `*` > `→` > blank). When no eligible candidate exists for the selected strategy, no `→` is placed on any row. The `→` marker and the footer are independent — the footer always shows both strategy recommendations; the marker shows only the winner for the chosen strategy.

**Interaction with `prefer::`:** Both strategies reference weekly quota (`endurance` qualification threshold, `drain` primary sort key) and use the `prefer::` parameter to select which weekly column to evaluate.

**Strategy comparison:**

| Dimension | `endurance` | `drain` |
|---|---|---|
| Primary sort key | qualified-first, then `weekly` desc | `prefer_weekly` asc (lowest 7d Left first) |
| h-exhausted handling | treated as unqualified | sunk to bottom |
| Secondary sort | within qualified: `5h_reset` asc; within unqualified: `weekly` desc | `5h_left` asc |
| Qualification gate | `5h_reset ∈ [15m, 60m]` + `weekly ≥ 30%` | none |
| Uses weekly quota | yes (gate + rank) | yes (primary sort key) |
| Picks account with… | freshest 5h reset + weekly runway | least weekly quota remaining (not h-exhausted) |
| Best for | starting a long 5h+ agent run | active workstation rotation |

### Worked Example

Eight accounts, two ineligible (`✓` current, `*` active-but-not-current), six eligible candidates. `prefer::any` (default), `sort::drain` (default).

**Eligible candidates:**

| Account | 5h Left | Expires | 7d Left | 7d(Son) | weekly(any)¹ | 5h Reset |
|---------|---------|---------|---------|---------|--------------|----------|
| a@example.com | 32% | 5m | 60% | 34% | 34% | 33m |
| b@example.com | 99% | 5m | 52% | 34% | 34% | 33m |
| c@example.com | 100% | 5m | 19% | 3% | 3% | — |
| d@example.com | 100% | 7h 27m | 2% | 4% | 2% | 4h 23m |
| e@example.com | 100% | 7h 27m | 4% | 0% | 0% | 4h 23m |
| f@example.com | 100% | 1h 49m | 2% | 0% | 0% | — |

¹ `weekly(any)` = `min(7d Left, 7d(Son))`

**`endurance`** — qualify on `5h_reset ∈ [15m, 60m]` AND `weekly(any) ≥ 30%`:
a: reset=33m ✓, weekly=34% ✓ → qualified. b: reset=33m ✓, weekly=34% ✓ → qualified. c/d/e/f: weekly < 30% → unqualified.
Within qualified: weekly=34% tied, reset=33m tied → alphabetical: a before b. **Winner: a@example.com.**

**`drain`** — `prefer_weekly` ascending (none h-exhausted; `prefer::any`):
weekly(any): e=0%=f < d=2% < c=3% < active=13% (skip: is_active) < a=34%=b (tiebreak `5h_left`: a=32% < b=99%) < current=61% (skip: is_current). First eligible: **Winner: e@example.com.**

Strategies disagree in this dataset: endurance picks a@example.com (qualified, most reset/weekly runway); drain picks e@example.com (lowest weekly quota = 0%). The footer always exposes both picks regardless of which `next::` value is active:

```
   ●   Account              5h Left   5h Reset   7d Left  7d(Son)  7d Reset  Expires    ~Renews
-  --  -------------------  --------  ---------  -------  -------  --------  ---------  -------
   🟢  c@example.com        🟢 100%   —          🟢 19%   3%       ...       in 5m      ...
*  🟢  active@example.com   🟢 99%    in 4h 33m  🟢 43%   13%      ...       in 7h 33m  ...
   🟢  a@example.com        🟢 32%    in 33m     🟢 60%   34%      ...       in 5m      ...
   🟢  b@example.com        🟢 99%    in 33m     🟢 52%   34%      ...       in 5m      ...
✓  🟢  current@example.com  🟢 88%    in 4h 13m  🟢 73%   61%      ...       in 5m      ...
→  🟡  e@example.com        🟢 100%   in 4h 23m  🟡 4%    0%       ...       in 7h 27m  ...
   🟡  f@example.com        🟢 100%   —          🟡 2%    0%       ...       in 1h 49m  ...
   🟡  d@example.com        🟢 100%   in 4h 23m  🟡 2%    4%       ...       in 7h 27m  ...

Valid: 8 / 8   ->  Next by strategy:
  endurance  a@example.com   32% session, 34% 7d left, expires in 5m
  drain      e@example.com   100% session, resets in 4h 23m
```

(`next::drain` default — `→` on e@example.com. Strategies disagree here: endurance recommends the qualified a@example.com while drain targets the lowest-weekly e@example.com.)

### Acceptance Criteria

- **AC-01**: The footer always shows one recommendation line per strategy (endurance, drain) with account name and key metric, regardless of the `next::` parameter value. The footer is never suppressed by a `next::` value choice.
- **AC-02**: Exactly one account receives the `→` flag in the table body — the account selected by the active `next::` strategy. No `→` is placed when no eligible candidate exists for that strategy.
- **AC-03**: `next::endurance` places `→` on the top non-current, non-active account from endurance sort order.
- **AC-04**: `next::drain` (default) places `→` on the top non-current, non-active account from drain sort order.
- **AC-05**: Invalid `next::` value exits 1 with an error naming the valid values (`endurance`, `drain`).
- **AC-06**: `next::` does not affect `format::json` output — JSON always uses alphabetical order without recommendation markers.
- **AC-07**: Footer is omitted when 0 or 1 accounts have valid quota data (same threshold as 009_token_usage.md AC-10).
- **AC-08**: Footer strategy lines for which no eligible account exists are omitted from the footer rather than showing an empty line.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/usage.rs` | `find_next_for_strategy()`, `render_text()` footer rendering |
| param | [cli/param/032_next.md](../cli/param/032_next.md) | `next::` parameter specification |
| doc | [009_token_usage.md](009_token_usage.md) | Base `.usage` algorithm |
| doc | [020_usage_sort_strategies.md](020_usage_sort_strategies.md) | Sort strategy algorithms reused by endurance/drain next strategies |
| param | [cli/param/027_prefer.md](../cli/param/027_prefer.md) | `prefer::` affects weekly quota used by endurance/drain strategies |
