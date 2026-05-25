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
| `drain` (default) | Drain Top | First non-current, non-active account from drain sort order (5h_left ascending, h-exhausted sunk; tiebreak weekly desc). |

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

**Interaction with `prefer::`:** Both strategies reference weekly quota (`endurance` qualification threshold, `drain` tiebreaker) and use the `prefer::` parameter to select which weekly column to evaluate.

**Strategy comparison:**

| Dimension | `endurance` | `drain` |
|---|---|---|
| Primary sort key | qualified-first, then `weekly` desc | `5h_left` asc |
| h-exhausted handling | treated as unqualified | sunk to bottom |
| Secondary sort | within qualified: `5h_reset` asc; within unqualified: `weekly` desc | `weekly` desc |
| Qualification gate | `5h_reset ∈ [15m, 60m]` + `weekly ≥ 30%` | none |
| Uses weekly quota | yes (gate + rank) | yes (tiebreak) |
| Picks account with… | freshest 5h reset + weekly runway | least remaining session (not h-exhausted) |
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

**`drain`** — `5h_left` ascending (none h-exhausted):
a=32% < b=99% < {c,d,e,f}=100%. Within 100% group, `weekly` desc: c=3%, d=2%, e=0%=f. **Winner: a@example.com.**

Both strategies agree on a@example.com in this dataset. The footer always exposes both picks regardless of which `next::` value is active:

```
   ●   Account              Expires    ~Renews  5h Left   5h Reset   7d Left  7d(Son)  7d Reset
-  --  -------------------  ---------  -------  --------  ---------  -------  -------  --------
→  🟢  a@example.com        in 5m      ...      🟢 32%    in 33m     🟢 60%   34%      ...
   🟢  b@example.com        in 5m      ...      🟢 99%    in 33m     🟢 52%   34%      ...
✓  🟢  current@example.com  in 5m      ...      🟢 88%    in 4h 13m  🟢 73%   61%      ...
*  🟢  active@example.com   in 7h 33m  ...      🟢 99%    in 4h 33m  🟢 43%   13%      ...
   🟢  c@example.com        in 5m      ...      🟢 100%   —          🟢 19%   3%       ...
   🟡  d@example.com        in 7h 27m  ...      🟢 100%   in 4h 23m  🟡 2%    4%       ...
   🟡  e@example.com        in 7h 27m  ...      🟢 100%   in 4h 23m  🟡 4%    0%       ...
   🟡  f@example.com        in 1h 49m  ...      🟢 100%   —          🟡 2%    0%       ...

Valid: 8 / 8   ->  Next by strategy:
  endurance  a@example.com   32% session, 34% 7d left, expires in 5m
  drain      a@example.com   32% session, resets in 33m
```

(`next::drain` default — `→` on a@example.com. Both strategies agree here. Footer is identical either way.)

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
