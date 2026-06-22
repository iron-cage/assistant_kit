# Simplify Sort Strategies and Implement 4-Group Status Partition

## Goal

**Motivated:** `.usage` carries 7 sort strategies and a separate 3-strategy `next::` parameter. Four sort strategies are dead weight: `endurance` and `drain` (complex multi-factor heuristics with no distinct workflow need — `renew` subsumes both), `next` (meta-alias resolved at parse time), and `expires` (sorts by transient OAuth token expiry — not a quota decision axis). The `next::` parameter is entirely redundant — `sort::` now drives both row ordering and the `→` recommendation marker, making `next::` and `NextStrategy` unnecessary. The 3-tier grouping with ad-hoc sub-groups is described inconsistently across docs. Removing ~400 lines of strategy code, the entire `next::` parameter, and formalizing the 4-group status partition unblocks consistent documentation and reduces maintenance surface.

**Observable:** After completion: `SortStrategy` enum has 3 variants (`Name`, `Renew`, `Renews`); `NextStrategy` enum is removed entirely; `next::` parameter is rejected (exit 1 with "unknown parameter"); `sort::endurance`, `sort::drain`, `sort::next`, `sort::expires` each exit 1 with an error naming valid values (`name`, `renew`, `renews`); footer shows 1 recommendation line for the active `sort::` strategy; sort grouping uses 4 explicit status groups (🟢 Green → 🟡 h-exhausted → 🟡 weekly-exhausted → 🔴 Red) where 🔴 includes both-exhausted accounts (not just errors); `find_next_for_strategy()` uses the sort strategy directly — no `NextStrategy` indirection.

**Scoped:** Single deliverable — remove 4 sort strategies, remove `next::` parameter and `NextStrategy` enum entirely, implement 4-group status partition, update tests.

**Testable:** `./verb/test` passes; `clp .usage sort::endurance` exits 1; `clp .usage next::renew` exits 1 (unknown parameter); footer shows only 1 strategy line; accounts with both quotas exhausted appear in 🔴 group.

## Execution State

- **State:** ✅ (Completed)
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
- Remove `NextStrategy` enum entirely from `types.rs` (not reduce to 1 variant — `sort::` drives recommendation directly)
- Remove `next::` parameter from `UsageParams` in `types.rs` and from `parse_usage_params()` in `params.rs`
- Remove dropped strategy match arms from `sort.rs` (`sort_indices()`)
- Remove `sort::next` alias resolution from `params.rs`
- Update `SortStrategy::parse()` to reject dropped values (exit 1 with valid-values error); `next::` is rejected as unknown parameter
- Remove `find_next_for_strategy()` `NextStrategy` parameter — function uses `SortStrategy` directly
- Remove endurance/drain arms from `sort_next.rs`
- Update `render.rs` footer: 3-strategy → 1-strategy line for active `sort::` strategy
- Implement 4-group status partition in `sort.rs`: replace ad-hoc `five_hour_left > 15.0` binary partition with 4 explicit groups (🟢 5h>15% AND 7d>5%, 🟡 5h≤15% AND 7d>5%, 🟡 5h>15% AND 7d≤5%, 🔴 both-exhausted OR error)

**Test changes:**
- Remove/update tests referencing `NextStrategy` enum and deleted `SortStrategy` variants in `sort_next_tests.rs`, `render_tests.rs`, `usage_test.rs`, `sort.rs` embedded tests
- Add tests for rejected `sort::` values and for `next::` being an unknown parameter
- Add tests for 4-group status partition ordering (h-exhausted above weekly-exhausted, both-exhausted in 🔴)
- Update `tests/docs/` spec files for features 20, 23 and params 25, 32, 09 to reflect reduced strategy set and `next::` removal

## Out of Scope

- Vocabulary/terminology changes (`status group`, `h-exhausted`, `weekly-exhausted` — already done in dictionary)
- Behavioral documentation edits in `docs/` (all docs already updated to target state prior to this task; `tests/docs/` test spec updates are in scope as part of test changes)
- Changes to `prefer::` parameter semantics (remains, applies to `sort::renew` secondary key)
- Changes to `rotate::1` behavior (Feature 038, task 003)
- Changes to `cols::`, row filtering params, `live::`, `touch::`, `imodel::`
- Changes to `.accounts` command
- New sort strategies
- Changes to `desc::` semantics (still reverses within groups, never reverses group order)

## Work Procedure

1. **Read** current enum definitions in `src/usage/types.rs` — confirm `SortStrategy` and `NextStrategy` variant lists, `UsageParams` fields
2. **Red** — write failing tests: `sort::endurance` → exit 1, `next::renew` → exit 1 (unknown param), 4-group ordering (h-exhausted above weekly-exhausted, both-exhausted in 🔴), single-strategy footer
3. **Remove enums** — delete `Endurance`, `Drain`, `Next`, `Expires` from `SortStrategy`; delete `NextStrategy` enum entirely from `types.rs`; remove `next` field from `UsageParams`
4. **Fix parse** — remove `next::` from `parse_usage_params()` in `params.rs`; update `SortStrategy::parse()` to reject dropped values; remove `sort::next` alias resolution
5. **Fix sort** — remove match arms for dropped strategies in `sort_indices()` in `sort.rs`; implement 4-group status partition (group 1: 5h>15% AND 7d>5%; group 2: 5h≤15% AND 7d>5%; group 3: 5h>15% AND 7d≤5%; group 4: both-below OR error)
6. **Fix next** — refactor `find_next_for_strategy()` in `sort_next.rs` to take `SortStrategy` directly (not `NextStrategy`); remove endurance/drain arms
7. **Fix render** — update footer in `render.rs` to produce 1 recommendation line for active `sort::` strategy
8. **Fix callers** — update all call sites that pass `NextStrategy` to pass `SortStrategy` instead; update `rotate::1` to use sort strategy for recommendation
9. **Green** — run `./verb/test`; fix compilation errors from removed types across all files
10. **Update tests** — remove/update test functions in sort_next_tests.rs, render_tests.rs, usage_test.rs that reference deleted strategies/types; verify new rejection tests pass
11. **Run tests** — `./verb/test` — full green state
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
| `next::renew` | Unknown parameter | Exit 1, error: unknown parameter `next` | 8 |
| `next::endurance` | Unknown parameter | Exit 1, error: unknown parameter `next` | 9 |
| Footer output | 1-strategy footer | Single recommendation line for active `sort::` strategy | 10 |
| 🟢 above 🟡 h-exhausted | Group 1 vs group 2 | 🟢 rows always above 🟡 h-exhausted rows | 12 |
| 🟡 h-exhausted above 🟡 weekly-exhausted | Group 2 vs group 3 | h-exhausted above weekly-exhausted | 13 |
| 🟡 weekly-exhausted above 🔴 | Group 3 vs group 4 | weekly-exhausted above 🔴 | 14 |
| Both 5h ≤ 15% and 7d ≤ 5% | Group 4 membership | Account in 🔴 Red group (not 🟡) | 15 |
| `desc::1` with mixed groups | Direction + group order | Rows reversed within each group; group order unchanged | 16 |

## Validation

### Checklist

- [ ] `SortStrategy` enum has exactly 3 variants: `Name`, `Renew`, `Renews`
- [ ] `NextStrategy` enum does not exist (fully removed)
- [ ] `next::` is rejected as unknown parameter (exit 1)
- [ ] `sort::endurance`, `sort::drain`, `sort::next`, `sort::expires` each exit 1
- [ ] Footer shows exactly 1 recommendation line for active `sort::` strategy
- [ ] 4-group status partition implemented with correct ordering
- [ ] 🔴 group includes both-exhausted accounts (not just errors)
- [ ] `desc::1` reverses within groups but not group order
- [ ] All tests pass (`./verb/test`)
- [ ] No compilation warnings (`RUSTFLAGS="-D warnings"`)

### Measurements

| Metric | Expected |
|--------|----------|
| `SortStrategy` variant count | 3 |
| `NextStrategy` exists | no (removed) |
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
- `grep -rn 'NextStrategy' src/usage/` returns 0 matches
- `clp .usage sort::endurance 2>&1; echo $?` — last line is `1`
- `clp .usage next::renew 2>&1; echo $?` — last line is `1` (unknown parameter)
- `clp .usage 2>&1 | grep -c 'endurance\|drain'` — expect 0 (no footer references)

## Related Documentation

**Updated (consistent with target state):**
- `docs/cli/param/025_sort.md` — sort parameter spec: 3 strategies, 4-group table, `→` recommendation
- `docs/cli/param/026_desc.md` — desc parameter: dropped strategy defaults removed
- `docs/cli/param/027_prefer.md` — prefer parameter: dropped strategy interactions removed
- `docs/cli/param/032_next.md` — REMOVED tombstone
- `docs/cli/param/040_only_next.md` — references `sort::` instead of `next::`
- `docs/cli/param/059_rotate.md` — references `sort::` instead of `next::`
- `docs/cli/param/045_get.md` — examples updated
- `docs/cli/param/readme.md` — param 032 marked REMOVED
- `docs/cli/param_group/004_sort_control.md` — `next::` removed from group
- `docs/cli/param_group/005_display_control.md` — semantic test updated
- `docs/cli/param_group/readme.md` — `next::` removed from Sort Control listing
- `docs/cli/command/006_usage.md` — param table, footer note, cross-references
- `docs/cli/command/001_account.md` — `next::` removed from param table and list
- `docs/cli/004_parameter_interactions.md` — interaction 9 updated to `sort::`
- `docs/cli/workflow_scenario/001_account_rotation.md` — strategy examples
- `docs/cli/user_story/001_account_rotation.md` — `sort::` instead of `next::`
- `docs/cli/user_story/003_quota_monitoring.md` — `sort::` strategy references
- `docs/cli/command_verb/006_rotate.md` — `sort::` instead of `next::`
- `docs/cli/002_dictionary.md` — vocabulary: status group, h-exhausted, weekly-exhausted
- `docs/feature/020_usage_sort_strategies.md` — rewritten: 3 strategies, `→` recommendation, 4-group partition
- `docs/feature/023_next_account_strategies.md` — DEPRECATED (absorbed into feature 020)
- `docs/feature/009_token_usage.md` — 4-group partition, `sort::` recommendation, 1-strategy footer
- `docs/feature/038_usage_strategy_rotate.md` — `sort::` instead of `next::`
- `docs/feature/028_usage_row_filtering.md` — `sort::` instead of `next::`
- `docs/feature/024_session_touch.md` — cross-references updated
- `docs/feature/033_quota_cache.md` — AC-08 updated
- `docs/feature/037_accounts_usage_param_unification.md` — `next::` removed from param table
- `docs/feature/008_auto_rotate.md` — deprecation note updated
- `docs/feature/readme.md` — feature 020/023 listings
- `docs/entity/readme.md` — feature 023 and param 032 marked deprecated/removed
- `docs/doc_graph.yml` — feature 023 edges removed

**Test specs (need update in implementation):**
- `tests/docs/feature/20_usage_sort_strategies.md` — remove dropped strategy test cases
- `tests/docs/feature/23_next_account_strategies.md` — deprecate or remove
- `tests/docs/cli/param/25_sort.md` — update valid values
- `tests/docs/cli/param/32_next.md` — remove or mark as REMOVED test surface

## History

- **[2026-06-17]** `CREATED` — Simplify sort strategies to name/renew/renews, reduce next to renew-only, implement 4-group status partition replacing ad-hoc 3-tier grouping with sub-groups.
- **[2026-06-17]** `UPDATED` — Passed Verification Gate (4/4 PASS).
- **[2026-06-18]** `UPDATED` — Extended scope: `next::` parameter removed entirely (not reduced to 1 variant); `NextStrategy` enum removed; `sort::` unifies row ordering and `→` recommendation. All documentation updated to target state.
- **[2026-06-18]** `UPDATED` — Passed Verification Gate (4/4 PASS) after scope extension.
- **[2026-06-18]** `COMPLETED` — All phases executed. 64/64 container jobs green, 16/16 crates pass. Consistency pass applied to 9 doc/spec files with stale references. Deprecated/removed test specs trimmed. Task closed.

## Verification Record

| Dimension | Result | Agent |
|-----------|--------|-------|
| Scope Coherence | PASS | sonnet (ab73f5a0) |
| MOST Goal Quality | PASS | sonnet (a31fc331) |
| Value / YAGNI | PASS | sonnet (a93e72f8) — adversarial mandate |
| Implementation Readiness | PASS | sonnet (a47773aa) |
