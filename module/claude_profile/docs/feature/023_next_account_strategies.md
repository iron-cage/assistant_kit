# Feature: Next Account Recommendation Strategies

### Scope

- **Purpose**: Provide configurable account recommendation in `.usage` output, where the `next::` parameter selects which strategy picks the `->` Next account(s) shown in the footer.
- **Responsibility**: Documents the `next::` parameter and the 5 recommendation values (`session`, `endurance`, `drain`, `reset`, `all`), including per-strategy selection algorithms and the multi-strategy footer format.
- **In Scope**: `next::` parameter with 5 values, per-strategy recommendation algorithm, multi-strategy footer rendering when `next::all` (default), single-strategy `->` marker and footer when a specific strategy is chosen, interaction with `prefer::` for weekly quota selection.
- **Out of Scope**: Row ordering (-> 020_usage_sort_strategies.md), `->` tiebreaker internals within `session` strategy (-> 009_token_usage.md AC-09), sort strategy algorithms (-> 020_usage_sort_strategies.md), `cols::` column visibility (-> 009_token_usage.md).

### Design

`.usage` accepts a `next::` parameter controlling how the recommended next account is selected and displayed. The default is `next::all`, which shows all four strategy recommendations simultaneously in the footer. Specific values (`session`, `endurance`, `drain`, `reset`) display a single recommendation with a `->` table marker and single-line footer.

**Strategy table:**

| Value | Name | Selection algorithm |
|-------|------|---------------------|
| `session` | Best Session Composite | Lexicographic max of `(5h_left, expires_in_secs, 7d_left)` among non-current, non-active accounts with valid data and non-expired token. Current `find_recommendation()` algorithm. |
| `endurance` | Endurance Top | First non-current, non-active account from endurance sort order (qualified accounts first by weekly desc then reset asc; unqualified by 5h_left desc). |
| `drain` | Drain Top | First non-current, non-active account from drain sort order (5h_left ascending, exhausted sunk; tiebreak weekly desc). |
| `reset` | Reset Top | First non-current, non-active account from reset sort order (5h_reset ascending, exhausted sunk; tiebreak 5h_left ascending). |
| `all` (default) | All Strategies | Footer shows one recommendation per strategy; `->` marker suppressed in table. |

**Recommendation eligibility:** All strategies skip accounts that are `is_current` (user is already on that session) or `is_active` (the `_active` marker account when it differs from current). Only accounts with valid quota data and `expires_in_secs > 0` are eligible. Strategies select from all eligible accounts regardless of their composite health tier (-> 009_token_usage.md three-tier grouping) -- the tier affects table display ordering, not recommendation eligibility.

**Multi-strategy footer format (`next::all`):**

When `next::all` (default), the `->` marker is suppressed in the table body. The footer expands to show each strategy's recommendation on its own line with key qualifying metrics:

```
Valid: 7 / 8   ->  Next by strategy:
  session    alice@example.com   100% session left, expires in 7h 57m
  endurance  bob@example.com     100% session, 76% 7d left, expires in 7h 56m
  drain      carol@example.com   98% session, resets in 5m
  reset      carol@example.com   98% session, resets in 5m
```

Each footer line shows: strategy name (left-aligned, 10 chars), account name (left-aligned), key metric string. When two strategies recommend the same account, both lines appear independently (the agreement is itself useful signal).

**Single-strategy footer format (`next::session`, etc.):**

When a specific strategy is chosen, the `->` marker appears on the recommended account's table row, and the footer shows a single line:

```
Valid: 7 / 8   ->  Next: alice@example.com  (100% session left, expires in 7h 57m)
```

**Interaction with `prefer::`:** Strategies that reference weekly quota (`endurance` qualification threshold, `drain` tiebreaker) use the `prefer::` parameter to select which weekly column to evaluate. `session` always uses `7d Left` for its third tiebreaker key. `reset` does not use weekly quota.

### Acceptance Criteria

- **AC-01**: `next::all` (default) suppresses the `->` marker in the table body; no table row receives the `->` flag.
- **AC-02**: `next::all` footer shows one line per strategy (session, endurance, drain, reset) with account name and key metric.
- **AC-03**: `next::session` places `->` on the account selected by `find_recommendation()` (lexicographic `(5h_left, expires_in_secs, 7d_left)` max); footer shows single line.
- **AC-04**: `next::endurance` places `->` on the top non-current, non-active account from endurance sort order; footer shows single line.
- **AC-05**: `next::drain` places `->` on the top non-current, non-active account from drain sort order; footer shows single line.
- **AC-06**: `next::reset` places `->` on the top non-current, non-active account from reset sort order; footer shows single line.
- **AC-07**: Invalid `next::` value exits 1 with an error naming the valid values.
- **AC-08**: `next::` does not affect `format::json` output -- JSON always uses alphabetical order without recommendation markers.
- **AC-09**: Footer is omitted when 0 or 1 accounts have valid quota data (same threshold as 009_token_usage.md AC-10).
- **AC-10**: `next::all` footer strategies that recommend no account (no eligible candidates) are omitted from the footer rather than showing an empty line.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/usage.rs` | `find_recommendation()`, `render_text()` footer rendering |
| param | [cli/param/032_next.md](../cli/param/032_next.md) | `next::` parameter specification |
| doc | [009_token_usage.md](009_token_usage.md) | Base `.usage` algorithm; `session` strategy = current `find_recommendation()` |
| doc | [020_usage_sort_strategies.md](020_usage_sort_strategies.md) | Sort strategy algorithms reused by endurance/drain/reset next strategies |
| param | [cli/param/027_prefer.md](../cli/param/027_prefer.md) | `prefer::` affects weekly quota used by endurance/drain strategies |
