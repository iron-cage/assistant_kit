# Simplify Sort Strategies and Implement 4-Group Status Partition

## Goal

**Motivated:** `.usage` carries 7 sort strategies and a 3-strategy `next::` parameter. Four sort strategies are dead weight: `endurance` and `drain` (complex multi-factor heuristics with no distinct workflow need — `renew` subsumes both), `next` (meta-alias resolved at parse time), and `expires` (sorts by transient OAuth token expiry — not a quota decision axis). The `next::` parameter still offers `endurance`/`drain` which reference the same dead strategies. The 3-tier grouping with ad-hoc sub-groups is described inconsistently across docs. Removing ~400 lines of strategy code and formalizing the 4-group status partition unblocks consistent documentation and reduces maintenance surface.

**Observable:** After completion: `SortStrategy` enum has 3 variants (`Name`, `Renew`, `Renews`); `NextStrategy` enum has 1 variant (`Renew`); `sort::endurance`, `sort::drain`, `sort::next`, `sort::expires` each exit 1 with an error naming valid values; `next::endurance`, `next::drain` each exit 1; footer shows 1 recommendation (renew only); sort grouping uses 4 explicit status groups (🟢 Green → 🟡 h-exhausted → 🟡 weekly-exhausted → 🔴 Red) where 🔴 includes both-exhausted accounts (not just errors).

**Scoped:** Single deliverable — remove 4 sort strategies + 2 next strategies from `src/usage/`, implement 4-group status partition, update tests and consequential docs.

**Testable:** `./verb/test` passes; `clp .usage sort::endurance` exits 1; `clp .usage next::drain` exits 1; footer shows only `renew` line; accounts with both quotas exhausted appear in 🔴 group.

## Execution State

- **State:** 🎯 (Verified)
- **Priority:** 3
- **Dir:** module/claude_profile
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **Closes:** null
- **Executor Type:** any
- **Validated By:** null
- **Validation Date:** null

## In Scope

**Source changes (`src/usage/`):**
- Remove `SortStrategy::{Endurance, Drain, Next, Expires}` from `types.rs`
- Remove `NextStrategy::{Endurance, Drain}` from `types.rs`
- Remove dropped strategy match arms from `sort.rs` (`sort_indices()`)
- Remove `sort::next` alias resolution from `params.rs`
- Update `SortStrategy::parse()` and `NextStrategy::parse()` to reject dropped values (exit 1 with valid-values error)
- Remove endurance/drain arms from `sort_next.rs` (`find_next_for_strategy()`)
- Update `render.rs` footer: 3-strategy → 1-strategy (renew only)
- Implement 4-group status partition in `sort.rs`: replace ad-hoc `five_hour_left > 15.0` binary partition with 4 explicit groups (🟢 5h>15% AND 7d>5%, 🟡 5h≤15% AND 7d>5%, 🟡 5h>15% AND 7d≤5%, 🔴 both-exhausted OR error)

**Test changes:**
- Remove/update tests referencing deleted enum variants in `sort_next_tests.rs`, `render_tests.rs`, `usage_test.rs`, `sort.rs` embedded tests
- Add tests for rejected `sort::` and `next::` values
- Add tests for 4-group status partition ordering (h-exhausted above weekly-exhausted, both-exhausted in 🔴)

**Consequential doc updates (coupled to code removal):**
- Feature 020: remove strategy sections for endurance/drain/next/expires; update ACs
- Feature 023: reduce to renew-only next strategy; update ACs
- Feature 009: update footer examples to 1-strategy
- Param 026_desc: remove desc defaults for dropped strategies
- Param 032_next: reduce to single value `renew`
- Param 027_prefer: remove interactions with dropped strategies
- Command 006_usage: update parameter table, syntax examples, footer example
- Test spec docs: 20, 23, 25, 32, 09 — remove test cases for dropped strategies

## Out of Scope

- Vocabulary/terminology changes (`status group`, `h-exhausted`, `weekly-exhausted` — already done in dictionary, sort param doc, grouping terminology)
- Changes to `prefer::` parameter semantics (remains, applies to `sort::renew` secondary key)
- Changes to `rotate::1` behavior (Feature 038, task 003)
- Changes to `cols::`, row filtering params, `live::`, `touch::`, `imodel::`
- Changes to `.accounts` command
- New sort strategies or next strategies
- Changes to `desc::` semantics (still reverses within groups, never reverses group order)

## Work Procedure

1. **Read** current enum definitions in `src/usage/types.rs` — confirm `SortStrategy` and `NextStrategy` variant lists
2. **Red** — write failing tests for rejected sort/next values (`sort::endurance` → exit 1, `next::drain` → exit 1, etc.) and 4-group ordering (h-exhausted above weekly-exhausted, both-exhausted in 🔴)
3. **Remove enums** — delete `Endurance`, `Drain`, `Next`, `Expires` from `SortStrategy`; delete `Endurance`, `Drain` from `NextStrategy` in `types.rs`
4. **Fix parse** — update `SortStrategy::parse()` and `NextStrategy::parse()` in `params.rs` to reject dropped values; remove `sort::next` alias resolution
5. **Fix sort** — remove match arms for dropped strategies in `sort_indices()` in `sort.rs`; implement 4-group status partition (group 1: 5h>15% AND 7d>5%; group 2: 5h≤15% AND 7d>5%; group 3: 5h>15% AND 7d≤5%; group 4: both-below OR error)
6. **Fix next** — remove endurance/drain arms from `find_next_for_strategy()` in `sort_next.rs`
7. **Fix render** — update footer in `render.rs` to produce 1 recommendation line (renew only)
8. **Green** — run `./verb/test`; fix compilation errors from removed variants across all files
9. **Update tests** — remove/update test functions in sort_next_tests.rs, render_tests.rs, usage_test.rs that reference deleted strategies; verify new rejection tests pass
10. **Run tests** — `./verb/test` — full green state
11. **Update docs** — feature 020 (remove 4 strategy sections + ACs), feature 023 (renew-only), param 026_desc/032_next/027_prefer, command 006_usage parameter table + examples + footer
12. **Update test specs** — tests/docs/ for features 20, 23 and params 25, 32, 09
13. **Final verification** — `./verb/test` — confirm full green
14. **Update state** — set task state to ⏳ In Progress → ✅ Completed

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior | Criterion |
|---------------|-------------------|-------------------|-----------|
| `sort::name` | Valid strategy | Exit 0, rows alphabetical | 1 |
| `sort::renew` | Valid strategy (default) | Exit 0, soonest quota event first | 2 |
| `sort::renews` | Valid strategy | Exit 0, soonest billing renewal first | 3 |
| `sort::endurance` | Rejected strategy | Exit 1, error names valid values (`name`, `renew`, `renews`) | 4 |
| `sort::drain` | Rejected strategy | Exit 1, error names valid values | 5 |
| `sort::next` | Rejected strategy | Exit 1, error names valid values | 6 |
| `sort::expires` | Rejected strategy | Exit 1, error names valid values | 7 |
| `next::renew` | Valid next (sole value) | Exit 0, `→` on renew winner | 8 |
| `next::endurance` | Rejected next | Exit 1, error names valid values (`renew`) | 9 |
| `next::drain` | Rejected next | Exit 1, error names valid values | 10 |
| Footer output | 1-strategy footer | Only `renew` recommendation line shown | 11 |
| 🟢 above 🟡 h-exhausted | Group 1 vs group 2 | 🟢 rows always above 🟡 h-exhausted rows | 12 |
| 🟡 h-exhausted above 🟡 weekly-exhausted | Group 2 vs group 3 | h-exhausted above weekly-exhausted | 13 |
| 🟡 weekly-exhausted above 🔴 | Group 3 vs group 4 | weekly-exhausted above 🔴 | 14 |
| Both 5h ≤ 15% and 7d ≤ 5% | Group 4 membership | Account in 🔴 Red group (not 🟡) | 15 |
| `desc::1` with mixed groups | Direction + group order | Rows reversed within each group; group order unchanged | 16 |

## Validation

### Checklist

- [ ] `SortStrategy` enum has exactly 3 variants: `Name`, `Renew`, `Renews`
- [ ] `NextStrategy` enum has exactly 1 variant: `Renew`
- [ ] `sort::endurance`, `sort::drain`, `sort::next`, `sort::expires` each exit 1
- [ ] `next::endurance`, `next::drain` each exit 1
- [ ] Footer shows exactly 1 recommendation line (renew)
- [ ] 4-group status partition implemented with correct ordering
- [ ] 🔴 group includes both-exhausted accounts (not just errors)
- [ ] `desc::1` reverses within groups but not group order
- [ ] All tests pass (`./verb/test`)
- [ ] No compilation warnings (`RUSTFLAGS="-D warnings"`)
- [ ] Feature 020 has no sections for endurance/drain/next/expires
- [ ] Feature 023 describes renew-only next strategy
- [ ] Param 032_next lists only `renew` as valid value

### Measurements

| Metric | Expected |
|--------|----------|
| `SortStrategy` variant count | 3 |
| `NextStrategy` variant count | 1 |
| Footer recommendation lines per `.usage` invocation | 1 |
| Status groups in sort partition | 4 |
| `grep -c 'Endurance\|Drain' src/usage/types.rs` | 0 |

### Invariants

- I1: Green accounts always appear above yellow in sorted output
- I2: Yellow always above red in sorted output
- I3: h-exhausted always above weekly-exhausted in sorted output
- I4: `desc::` never reverses group order
- I5: Account name is the final tiebreaker in all strategies (Fix(BUG-259))

### Anti-faking checks

- `grep -rn 'SortStrategy::Endurance\|SortStrategy::Drain\|SortStrategy::Next\|SortStrategy::Expires' src/usage/` returns 0 matches
- `grep -rn 'NextStrategy::Endurance\|NextStrategy::Drain' src/usage/` returns 0 matches
- `clp .usage sort::endurance 2>&1; echo $?` — last line is `1`
- `clp .usage next::drain 2>&1; echo $?` — last line is `1`
- `clp .usage 2>&1 | grep -c 'endurance\|drain'` — expect 0 (no footer references)

## Related Documentation

- `docs/cli/param/025_sort.md` — sort parameter spec (already updated: 4-group table, 3 strategies only)
- `docs/cli/002_dictionary.md` — vocabulary (already updated: status group, h-exhausted, weekly-exhausted definitions)
- `docs/feature/020_usage_sort_strategies.md` — sort strategy feature (grouping updated; strategy sections still present — update in step 11)
- `docs/feature/023_next_account_strategies.md` — next strategies (still references 3 strategies — update in step 11)
- `docs/feature/009_token_usage.md` — base usage feature (AC-24/AC-26 updated; footer example stale)
- `docs/cli/command/006_usage.md` — usage command spec (grouping updated; param table/examples stale)
- `docs/cli/param/026_desc.md` — desc parameter (still lists dropped strategy defaults)
- `docs/cli/param/032_next.md` — next parameter (still lists 3 values)
- `docs/cli/param/027_prefer.md` — prefer parameter (interactions with dropped strategies)
- `tests/docs/feature/20_usage_sort_strategies.md` — sort strategy test spec
- `tests/docs/feature/23_next_account_strategies.md` — next strategy test spec
- `tests/docs/cli/param/25_sort.md` — sort param test spec
- `tests/docs/cli/param/32_next.md` — next param test spec

## History

- **[2026-06-17]** `CREATED` — Simplify sort strategies to name/renew/renews, reduce next to renew-only, implement 4-group status partition replacing ad-hoc 3-tier grouping with sub-groups.
- **[2026-06-17]** `UPDATED` — Passed Verification Gate (4/4 PASS).

## Verification Record

| Dimension | Result | Agent |
|-----------|--------|-------|
| Scope Coherence | PASS | sonnet (a255b1ff) |
| MOST Goal Quality | PASS | sonnet (ac589eeb) |
| Value / YAGNI | PASS | sonnet (ac755573) — adversarial mandate |
| Implementation Readiness | PASS | sonnet (a30f49f8) |
