//! Bug Reproducer (BUG-529): `sort_rows()` had no secondary tie-break key, so
//! rows exactly tied on the chosen `sort_by` metric came back in a
//! non-deterministic order across repeated invocations of the identical
//! command against identical, unchanged data.
//!
//! ## Root Cause
//!
//! `build_rollup()` groups sessions into rows via an internal
//! `HashMap<String, RollupRow>`, then hands `HashMap::into_values()`'s output
//! to `sort_rows()`, which sorts with `slice::sort_by` — a *stable* sort, but
//! one whose stability only preserves whatever order the elements already
//! carried going in. `HashMap`'s default `RandomState` hasher reseeds every
//! process, so for rows that tie exactly on `sort_by`'s comparator (every
//! `match` arm returns `Ordering::Equal`), the final displayed order was
//! effectively `HashMap` iteration order — different on every fresh `clg`
//! invocation even though the underlying session data never changed.
//! Confirmed empirically: 5 separate manual `clg .rollup scope::global
//! group::project` invocations against 6 identically-totaled synthetic
//! sessions produced 5 distinct row orderings.
//!
//! ## Why Not Caught
//!
//! Every existing `sort_by_*` test in `rollup_test.rs` (e.g.
//! `sort_total_desc_orders_largest_first`, `sort_by_input_uses_input_metric`)
//! deliberately uses DISTINCT metric values across rows specifically to prove
//! which field drives ordering — none construct rows that are genuinely tied
//! on the metric under test, so the untested gap (what happens on an exact
//! tie) was never exercised.
//!
//! ## Fix Applied
//!
//! `sort_rows()` now appends `.then_with(|| a.group.cmp(&b.group))` after the
//! `order`-adjusted primary comparison — an unconditional, always-ascending
//! secondary key (never reversed by `order::desc`, since the tie-break is a
//! display-stability concern, not part of the user's requested direction).
//! Group labels are unique per row within one `build_rollup()` call (each row
//! is keyed by a distinct group string), so this secondary key always
//! produces a total, deterministic order with no further ties possible.
//!
//! ## Prevention
//!
//! These tests lock in that rows tied on the primary `sort_by` metric always
//! come back in ascending-group-label order, regardless of `order::` and
//! regardless of the rows' pre-sort (`HashMap`-derived) order — the exact
//! property that was previously left to chance.
//!
//! ## Pitfall
//!
//! `HashMap::into_values()` feeding a stable sort is a trap: "stable" sounds
//! like a determinism guarantee, but it only relocates the non-determinism
//! from "which order are equal elements sorted into" to "which order did
//! they arrive in" — and a `HashMap`'s arrival order is itself
//! process-randomized by design. Any pipeline that sources a `Vec` from a
//! `HashMap` and then sorts it must add an explicit secondary key whenever
//! ties are possible and a *total* (not just partial-by-the-primary-metric)
//! order is part of the actual contract — stability alone never supplies one.

use claude_storage_core::
{
  GroupKey, SortKey, SortOrder, RollupInput, RollupParams, build_rollup,
};
use claude_storage_core::SessionStats;

/// Build a `RollupInput` whose only relevant field is `input` (used to drive
/// `SortKey::Total` ties) under a distinct `group` label.
fn tied_input( group : &str, input_tokens : u64 ) -> RollupInput
{
  let mut stats = SessionStats::new( group.to_string() );
  stats.assistant_entries = 1;
  stats.total_input_tokens = input_tokens;

  RollupInput { session_id : group.to_string(), project_label : group.to_string(), stats }
}

fn params( order : SortOrder ) -> RollupParams
{
  RollupParams { group_by : GroupKey::Project, sort_by : SortKey::Total, order, model_filter : None, limit : 0 }
}

/// Test 6 exactly-tied rows, constructed in reverse-alphabetical input order,
/// come back in ascending-group order under `SortOrder::Desc`.
///
/// ## Coverage
/// Six `RollupInput`s all totaling `1000`, with group labels deliberately
/// inserted in the OPPOSITE of the expected output order (`zulu` first,
/// `uniform` last) — before the fix, this pre-sort `HashMap`-derived order
/// (or some other non-alphabetical arrangement) could surface unchanged,
/// since a stable sort over all-`Equal` comparisons is a no-op.
///
/// ## Validation Strategy
/// Six same-total inputs; `sort_by::Total order::Desc`; assert the output
/// group order is exactly ascending alphabetical, never the reverse-inserted
/// order and never any other arrangement.
#[ test ]
fn tied_rows_break_ties_by_ascending_group_under_desc()
{
  let entries = vec!
  [
    tied_input( "zulu", 1000 ),
    tied_input( "yankee", 1000 ),
    tied_input( "xray", 1000 ),
    tied_input( "whiskey", 1000 ),
    tied_input( "victor", 1000 ),
    tied_input( "uniform", 1000 ),
  ];
  let rows = build_rollup( &entries, &params( SortOrder::Desc ) );

  let order : Vec< &str > = rows.iter().map( | r | r.group.as_str() ).collect();
  assert_eq!
  (
    order,
    vec![ "uniform", "victor", "whiskey", "xray", "yankee", "zulu" ],
    "BUG-529: rows tied on total must break ties by ascending group label, regardless of insertion order; got: {order:?}"
  );
}

/// Test the same tie-break holds under `SortOrder::Asc` too — the secondary
/// key is never reversed by `order::`, only the primary metric is.
///
/// ## Coverage
/// Same 6 tied rows as the `Desc` test, `order::Asc` instead — since every
/// row's primary comparison is `Equal` either way, `Asc` vs `Desc` must make
/// no observable difference to a fully-tied set: both must yield the same
/// ascending-group order.
///
/// ## Validation Strategy
/// Six same-total inputs; `sort_by::Total order::Asc`; assert identical
/// ascending-group output to the `Desc` case.
#[ test ]
fn tied_rows_break_ties_by_ascending_group_under_asc()
{
  let entries = vec!
  [
    tied_input( "zulu", 1000 ),
    tied_input( "yankee", 1000 ),
    tied_input( "xray", 1000 ),
    tied_input( "whiskey", 1000 ),
    tied_input( "victor", 1000 ),
    tied_input( "uniform", 1000 ),
  ];
  let rows = build_rollup( &entries, &params( SortOrder::Asc ) );

  let order : Vec< &str > = rows.iter().map( | r | r.group.as_str() ).collect();
  assert_eq!
  (
    order,
    vec![ "uniform", "victor", "whiskey", "xray", "yankee", "zulu" ],
    "BUG-529: the tie-break must be order::-independent (always ascending group), even under order::asc; got: {order:?}"
  );
}

/// Test a mixed set — some rows genuinely tied, one row distinctly larger —
/// sorts the distinct row correctly by total while the tied subgroup still
/// breaks ties by group label.
///
/// ## Coverage
/// `"big"` (total 5000) must sort strictly ahead of the two tied rows
/// (`"bravo"`/`"alpha"`, total 1000 each) under `Desc`; the tied pair must
/// still land in ascending-group order (`alpha` before `bravo`) — proving the
/// secondary key activates ONLY among genuine ties and never overrides a real
/// primary-metric difference.
///
/// ## Validation Strategy
/// Three inputs (5000/1000/1000); `sort_by::Total order::Desc`; assert exact
/// 3-element order `["big", "alpha", "bravo"]`.
#[ test ]
fn tie_break_only_activates_among_genuinely_tied_rows()
{
  let entries = vec!
  [
    tied_input( "bravo", 1000 ),
    tied_input( "big", 5000 ),
    tied_input( "alpha", 1000 ),
  ];
  let rows = build_rollup( &entries, &params( SortOrder::Desc ) );

  let order : Vec< &str > = rows.iter().map( | r | r.group.as_str() ).collect();
  assert_eq!
  (
    order,
    vec![ "big", "alpha", "bravo" ],
    "BUG-529: the distinct-total row must still rank by total; only the genuinely tied pair breaks ties by group; got: {order:?}"
  );
}
