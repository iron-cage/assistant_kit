//! Unit tests for `rollup::build_rollup()` — grouping, model filtering,
//! percent computation, sorting, and `limit`.
//!
//! Pure-logic tests: every fixture is a synthetic `RollupInput`/`SessionStats`
//! value, no filesystem or JSONL involved. Filesystem-level coverage (the
//! `message.id` dedup this engine consumes) lives in
//! `session_stats_dedup_bug.rs`; CLI-level coverage lives in
//! `claude_storage/tests/cli_cmd_rollup_test.rs`.

use claude_storage_core::
{
  GroupKey, SortKey, SortOrder, StringMatcher, RollupInput, RollupParams, RollupRow, build_rollup,
};
use claude_storage_core::SessionStats;

/// Build a `RollupInput` with only the fields a given test cares about;
/// everything else starts at `SessionStats::new`'s zero/`None` baseline.
// 10 independent, order-sensitive fixture fields read far more clearly as
// positional args at every call site below than behind a builder — allow
// stays scoped to this single test helper, never the crate's public API.
#[ allow( clippy::too_many_arguments ) ]
fn input(
  session_id : &str,
  project_label : &str,
  model : Option< &str >,
  input_tokens : u64,
  output_tokens : u64,
  cache_read : u64,
  cache_creation : u64,
  max_context : u64,
  first_ts : Option< &str >,
  last_ts : Option< &str >,
) -> RollupInput
{
  let mut stats = SessionStats::new( session_id.to_string() );
  stats.assistant_entries = 1;
  stats.total_input_tokens = input_tokens;
  stats.total_output_tokens = output_tokens;
  stats.total_cache_read_tokens = cache_read;
  stats.total_cache_creation_tokens = cache_creation;
  stats.max_context_tokens = max_context;
  stats.model = model.map( std::string::ToString::to_string );
  stats.first_timestamp = first_ts.map( std::string::ToString::to_string );
  stats.last_timestamp = last_ts.map( std::string::ToString::to_string );

  RollupInput { session_id : session_id.to_string(), project_label : project_label.to_string(), stats }
}

/// Default params: group by session, sort by total descending, no filter, no limit.
fn default_params() -> RollupParams
{
  RollupParams { group_by : GroupKey::Session, sort_by : SortKey::Total, order : SortOrder::Desc, model_filter : None, limit : 0 }
}

fn row_by_group< 'a >( rows : &'a [ RollupRow ], group : &str ) -> &'a RollupRow
{
  rows.iter().find( | r | r.group == group ).unwrap_or_else( || panic!( "no row for group {group}; rows: {rows:?}" ) )
}

/// Test `GroupKey::Session` produces exactly one row per input, unaggregated.
///
/// ## Purpose
/// Validates the finest-granularity grouping mode: no cross-session merging.
///
/// ## Coverage
/// Two distinct sessions in the same project yield two separate rows.
///
/// ## Validation Strategy
/// Two `RollupInput`s sharing a `project_label`; group by `Session`; assert
/// row count and per-row token totals stay unmerged.
#[ test ]
fn group_by_session_one_row_per_input()
{
  let entries = vec!
  [
    input( "sess-a", "proj-x", None, 100, 10, 0, 0, 100, None, None ),
    input( "sess-b", "proj-x", None, 200, 20, 0, 0, 200, None, None ),
  ];
  let rows = build_rollup( &entries, &default_params() );

  assert_eq!( rows.len(), 2, "one row per session; got: {rows:?}" );
  assert_eq!( row_by_group( &rows, "sess-a" ).input, 100 );
  assert_eq!( row_by_group( &rows, "sess-b" ).input, 200 );
}

/// Test `GroupKey::Project` aggregates every session under one project row.
///
/// ## Purpose
/// Validates coarser grouping: sessions sharing `project_label` merge into
/// one row with summed totals and a correct `sessions` count.
///
/// ## Coverage
/// Two sessions in `proj-x`, one in `proj-y`; assert 2 rows, correct sums.
///
/// ## Validation Strategy
/// Three `RollupInput`s; group by `Project`; assert row count, `sessions`,
/// and summed `input`/`calls`.
#[ test ]
fn group_by_project_aggregates_sessions()
{
  let entries = vec!
  [
    input( "sess-a", "proj-x", None, 100, 0, 0, 0, 0, None, None ),
    input( "sess-b", "proj-x", None, 50, 0, 0, 0, 0, None, None ),
    input( "sess-c", "proj-y", None, 9, 0, 0, 0, 0, None, None ),
  ];
  let mut params = default_params();
  params.group_by = GroupKey::Project;
  let rows = build_rollup( &entries, &params );

  assert_eq!( rows.len(), 2, "2 project rows expected; got: {rows:?}" );
  let x = row_by_group( &rows, "proj-x" );
  assert_eq!( x.sessions, 2, "proj-x must aggregate 2 sessions" );
  assert_eq!( x.input, 150, "proj-x input must sum both sessions" );
  assert_eq!( x.calls, 2, "proj-x calls must sum both sessions' assistant_entries" );
}

/// Test `GroupKey::Model` buckets by `stats.model`, with `None` under `"unknown"`.
///
/// ## Purpose
/// Validates model grouping and the documented fallback bucket for sessions
/// with no recorded model.
///
/// ## Coverage
/// Two `opus` sessions merge; one `None`-model session lands in `"unknown"`.
///
/// ## Validation Strategy
/// Three `RollupInput`s; group by `Model`; assert bucket membership.
#[ test ]
fn group_by_model_buckets_with_unknown_fallback()
{
  let entries = vec!
  [
    input( "sess-a", "proj-x", Some( "opus" ), 10, 0, 0, 0, 0, None, None ),
    input( "sess-b", "proj-x", Some( "opus" ), 20, 0, 0, 0, 0, None, None ),
    input( "sess-c", "proj-x", None, 5, 0, 0, 0, 0, None, None ),
  ];
  let mut params = default_params();
  params.group_by = GroupKey::Model;
  let rows = build_rollup( &entries, &params );

  assert_eq!( rows.len(), 2, "opus + unknown; got: {rows:?}" );
  assert_eq!( row_by_group( &rows, "opus" ).sessions, 2 );
  assert_eq!( row_by_group( &rows, "opus" ).input, 30 );
  assert_eq!( row_by_group( &rows, "unknown" ).sessions, 1 );
}

/// Test `GroupKey::Day` buckets by the `YYYY-MM-DD` prefix of `first_timestamp`.
///
/// ## Purpose
/// Validates date-bucket grouping via plain string slicing (no date-parsing
/// dependency) and the `"unknown"` fallback for a missing timestamp.
///
/// ## Coverage
/// Two same-day sessions merge; a different-day session is separate; a
/// timestamp-less session lands in `"unknown"`.
///
/// ## Validation Strategy
/// Three dated inputs plus one undated input; group by `Day`; assert buckets.
#[ test ]
fn group_by_day_buckets_by_date_prefix()
{
  let entries = vec!
  [
    input( "sess-a", "proj-x", None, 10, 0, 0, 0, 0, Some( "2026-08-19T10:00:00Z" ), None ),
    input( "sess-b", "proj-x", None, 20, 0, 0, 0, 0, Some( "2026-08-19T23:59:00Z" ), None ),
    input( "sess-c", "proj-x", None, 30, 0, 0, 0, 0, Some( "2026-08-20T00:00:01Z" ), None ),
    input( "sess-d", "proj-x", None, 40, 0, 0, 0, 0, None, None ),
  ];
  let mut params = default_params();
  params.group_by = GroupKey::Day;
  let rows = build_rollup( &entries, &params );

  assert_eq!( rows.len(), 3, "2026-08-19 + 2026-08-20 + unknown; got: {rows:?}" );
  assert_eq!( row_by_group( &rows, "2026-08-19" ).sessions, 2 );
  assert_eq!( row_by_group( &rows, "2026-08-19" ).input, 30 );
  assert_eq!( row_by_group( &rows, "2026-08-20" ).sessions, 1 );
  assert_eq!( row_by_group( &rows, "unknown" ).sessions, 1 );
}

/// Test `model_filter` drops non-matching sessions before grouping.
///
/// ## Purpose
/// Validates the filter is applied at session granularity, before any
/// aggregation — a filtered-out session must not contribute to any row, even
/// when grouped at project/day granularity.
///
/// ## Coverage
/// A session with no model at all is excluded once a filter is set (absent
/// never satisfies a set filter); a substring match is case-insensitive.
///
/// ## Validation Strategy
/// Three sessions (`opus`-model, `sonnet`-model, no model) grouped by
/// `Project`; filter `"OPUS"`; assert only the opus session's tokens survive.
#[ test ]
fn model_filter_drops_non_matching_sessions_before_grouping()
{
  let entries = vec!
  [
    input( "sess-a", "proj-x", Some( "claude-opus-5" ), 100, 0, 0, 0, 0, None, None ),
    input( "sess-b", "proj-x", Some( "claude-sonnet-5" ), 200, 0, 0, 0, 0, None, None ),
    input( "sess-c", "proj-x", None, 300, 0, 0, 0, 0, None, None ),
  ];
  let mut params = default_params();
  params.group_by = GroupKey::Project;
  params.model_filter = Some( StringMatcher::new( "OPUS" ) );
  let rows = build_rollup( &entries, &params );

  assert_eq!( rows.len(), 1, "only the opus session's project row should survive; got: {rows:?}" );
  let row = &rows[ 0 ];
  assert_eq!( row.sessions, 1, "sonnet and model-less sessions must be excluded" );
  assert_eq!( row.input, 100, "only the opus session's tokens must count" );
}

/// Test `percent` is computed against the full filtered grand total, not just
/// the rows surviving `limit`.
///
/// ## Purpose
/// Validates the documented percent semantics: a narrow `limit::` must not
/// change what "100%" means for the rows still shown.
///
/// ## Coverage
/// Three equal-sized rows (each 1/3 of the total) with `limit::1`; the one
/// surviving row must still report ~33%, not 100%.
///
/// ## Validation Strategy
/// Three sessions with equal `input`, `limit::1`; assert the single returned
/// row's `percent` is close to `33.33`, not `100.0`.
#[ test ]
fn percent_reflects_full_filtered_total_not_post_limit_rows()
{
  let entries = vec!
  [
    input( "sess-a", "proj-x", None, 100, 0, 0, 0, 0, None, None ),
    input( "sess-b", "proj-x", None, 100, 0, 0, 0, 0, None, None ),
    input( "sess-c", "proj-x", None, 100, 0, 0, 0, 0, None, None ),
  ];
  let mut params = default_params();
  params.limit = 1;
  let rows = build_rollup( &entries, &params );

  assert_eq!( rows.len(), 1, "limit::1 must cap to one row; got: {rows:?}" );
  let percent = rows[ 0 ].percent;
  assert!(
    ( percent - 33.333 ).abs() < 0.1,
    "percent must reflect the full 3-row total (~33.3%), not 100%; got: {percent}"
  );
}

/// Test `percent` is `0.0`, not `NaN`, when the grand total is `0`.
///
/// ## Purpose
/// Validates the documented zero-total guard prevents a division-by-zero
/// `NaN` from ever reaching a rendered table.
///
/// ## Coverage
/// A single all-zero-token session.
///
/// ## Validation Strategy
/// One input with every token field `0`; assert `percent == 0.0` exactly.
#[ test ]
fn percent_is_zero_not_nan_when_grand_total_is_zero()
{
  let entries = vec![ input( "sess-a", "proj-x", None, 0, 0, 0, 0, 0, None, None ) ];
  let rows = build_rollup( &entries, &default_params() );

  assert_eq!( rows.len(), 1 );
  // Exact comparison is intentional: the zero-total branch returns the `0.0`
  // literal directly (see `build_rollup`'s doc), never a computed value that
  // could carry rounding error — bit-exact equality is the correct check.
  #[ allow( clippy::float_cmp ) ]
  { assert_eq!( rows[ 0 ].percent, 0.0, "zero grand total must yield 0.0, never NaN" ); }
}

/// Test `SortKey::Total` with `SortOrder::Desc` orders the largest row first.
///
/// ## Purpose
/// Validates the default sort configuration end-to-end.
///
/// ## Coverage
/// Three sessions with distinct totals; largest-first ordering.
///
/// ## Validation Strategy
/// Three inputs with totals 10/999/50; assert output order is 999, 50, 10.
#[ test ]
fn sort_total_desc_orders_largest_first()
{
  let entries = vec!
  [
    input( "small", "proj-x", None, 10, 0, 0, 0, 0, None, None ),
    input( "big", "proj-x", None, 999, 0, 0, 0, 0, None, None ),
    input( "mid", "proj-x", None, 50, 0, 0, 0, 0, None, None ),
  ];
  let rows = build_rollup( &entries, &default_params() );

  let order : Vec< &str > = rows.iter().map( | r | r.group.as_str() ).collect();
  assert_eq!( order, vec![ "big", "mid", "small" ], "must be largest-total-first; got: {order:?}" );
}

/// Test `SortOrder::Asc` reverses the default descending order.
///
/// ## Purpose
/// Validates the `order::` parameter actually flips comparison direction
/// rather than being ignored.
///
/// ## Coverage
/// Same fixture as the descending test, opposite expected order.
///
/// ## Validation Strategy
/// Three inputs with totals 10/999/50, `order::Asc`; assert ascending order.
#[ test ]
fn sort_order_asc_reverses_direction()
{
  let entries = vec!
  [
    input( "small", "proj-x", None, 10, 0, 0, 0, 0, None, None ),
    input( "big", "proj-x", None, 999, 0, 0, 0, 0, None, None ),
    input( "mid", "proj-x", None, 50, 0, 0, 0, 0, None, None ),
  ];
  let mut params = default_params();
  params.order = SortOrder::Asc;
  let rows = build_rollup( &entries, &params );

  let order : Vec< &str > = rows.iter().map( | r | r.group.as_str() ).collect();
  assert_eq!( order, vec![ "small", "mid", "big" ], "ascending must reverse the default; got: {order:?}" );
}

/// Test `SortKey::MaxContext` sorts by the per-row running max, not by total.
///
/// ## Purpose
/// Validates a non-default sort key is honored — proves `sort_by` genuinely
/// changes the comparison basis rather than always sorting by total.
///
/// ## Coverage
/// A row with a small total but a huge single-call max context must sort
/// ahead of a row with a bigger total but a small max context.
///
/// ## Validation Strategy
/// Two sessions: one with `total=1000/max_context=100`, one with
/// `total=10/max_context=900`; sort by `MaxContext` desc; assert the
/// small-total/high-max-context row comes first.
#[ test ]
fn sort_by_max_context_uses_correct_metric()
{
  let entries = vec!
  [
    input( "big-total", "proj-x", None, 1000, 0, 0, 0, 100, None, None ),
    input( "big-window", "proj-x", None, 10, 0, 0, 0, 900, None, None ),
  ];
  let mut params = default_params();
  params.sort_by = SortKey::MaxContext;
  let rows = build_rollup( &entries, &params );

  assert_eq!( rows[ 0 ].group, "big-window", "must sort by max_context, not total; got: {rows:?}" );
}

/// Test `max_context` tracks the running max across sessions merged into one row.
///
/// ## Purpose
/// Validates the aggregation step takes the max, not the sum, across
/// contributing sessions' `max_context_tokens`.
///
/// ## Coverage
/// Two sessions in one project with `max_context` 50 and 900; the merged row
/// must report 900, not 950.
///
/// ## Validation Strategy
/// Two same-project inputs; group by `Project`; assert `max_context == 900`.
#[ test ]
fn max_context_takes_running_max_not_sum()
{
  let entries = vec!
  [
    input( "sess-a", "proj-x", None, 0, 0, 0, 0, 50, None, None ),
    input( "sess-b", "proj-x", None, 0, 0, 0, 0, 900, None, None ),
  ];
  let mut params = default_params();
  params.group_by = GroupKey::Project;
  let rows = build_rollup( &entries, &params );

  assert_eq!( rows[ 0 ].max_context, 900, "must be max(50, 900), never their sum" );
}

/// Test `limit::0` is unbounded (returns every grouped row).
///
/// ## Purpose
/// Validates the `0` sentinel documented on `RollupParams::limit`.
///
/// ## Coverage
/// Five distinct sessions, `limit::0`; all five rows survive.
///
/// ## Validation Strategy
/// Five inputs; default params (`limit: 0`); assert row count is 5.
#[ test ]
fn limit_zero_is_unbounded()
{
  let entries : Vec< RollupInput > = ( 0..5 )
    .map( | i | input( &format!( "sess-{i}" ), "proj-x", None, i, 0, 0, 0, 0, None, None ) )
    .collect();
  let rows = build_rollup( &entries, &default_params() );

  assert_eq!( rows.len(), 5, "limit::0 must return every row; got: {rows:?}" );
}

/// Test `cache()` and `total()` combine fields correctly.
///
/// ## Purpose
/// Validates the two `RollupRow` helper methods' arithmetic directly.
///
/// ## Coverage
/// `cache() == cache_read + cache_creation`; `total() == input + output + cache()`.
///
/// ## Validation Strategy
/// One session with distinct nonzero values in every token field; compute
/// both helpers and assert exact sums.
#[ test ]
fn cache_and_total_combine_fields_correctly()
{
  let entries = vec![ input( "sess-a", "proj-x", None, 100, 20, 7, 3, 0, None, None ) ];
  let rows = build_rollup( &entries, &default_params() );

  let row = &rows[ 0 ];
  assert_eq!( row.cache(), 10, "cache_read(7) + cache_creation(3)" );
  assert_eq!( row.total(), 130, "input(100) + output(20) + cache(10)" );
}

/// Test empty input returns an empty result, not an error or panic.
///
/// ## Purpose
/// Validates the zero-row edge case — `build_rollup` cannot fail (it returns
/// `Vec<RollupRow>`, not `Result`), so the only valid empty-input outcome is
/// an empty `Vec`.
///
/// ## Coverage
/// Zero-length `entries` slice under default params.
///
/// ## Validation Strategy
/// Call `build_rollup` with `&[]`; assert an empty `Vec` comes back.
#[ test ]
fn empty_input_returns_empty_output()
{
  let rows = build_rollup( &[], &default_params() );
  assert!( rows.is_empty(), "empty input must yield an empty result; got: {rows:?}" );
}

/// Test `first`/`last` widen to the earliest/latest timestamp across a
/// merged group, not first-input-wins or last-input-wins by insertion order.
///
/// ## Purpose
/// Validates the min/max timestamp-widening logic in `accumulate()`.
///
/// ## Coverage
/// Three sessions merged into one project row, with timestamps inserted out
/// of chronological order.
///
/// ## Validation Strategy
/// Three same-project inputs with out-of-order first/last timestamps; group
/// by `Project`; assert the merged row's `first`/`last` are the true min/max.
#[ test ]
fn first_last_widen_across_merged_group()
{
  let entries = vec!
  [
    input( "sess-a", "proj-x", None, 0, 0, 0, 0, 0, Some( "2026-08-19T12:00:00Z" ), Some( "2026-08-19T12:05:00Z" ) ),
    input( "sess-b", "proj-x", None, 0, 0, 0, 0, 0, Some( "2026-08-17T08:00:00Z" ), Some( "2026-08-17T08:10:00Z" ) ),
    input( "sess-c", "proj-x", None, 0, 0, 0, 0, 0, Some( "2026-08-20T00:00:00Z" ), Some( "2026-08-20T00:01:00Z" ) ),
  ];
  let mut params = default_params();
  params.group_by = GroupKey::Project;
  let rows = build_rollup( &entries, &params );

  let row = &rows[ 0 ];
  assert_eq!( row.first.as_deref(), Some( "2026-08-17T08:00:00Z" ), "first must be the true earliest" );
  assert_eq!( row.last.as_deref(), Some( "2026-08-20T00:01:00Z" ), "last must be the true latest" );
}

/// Test `SortKey::Input` orders by `row.input` alone, not `Total`.
///
/// ## Purpose
/// Guards `sort_rows`'s `Input` match arm against a copy-paste mis-wiring —
/// the underlying `input` field's correctness is already covered elsewhere
/// (`group_by_project_aggregates_sessions`); this only checks the arm picks
/// the right field.
///
/// ## Coverage
/// Two rows whose `input` order is the OPPOSITE of their `Total` order.
///
/// ## Validation Strategy
/// `"low-input"` has less input but more output than `"high-input"`; sort by
/// `Input` desc; assert `"high-input"` comes first despite a lower total.
#[ test ]
fn sort_by_input_uses_input_metric()
{
  let entries = vec!
  [
    input( "low-input", "proj-x", None, 10, 900, 0, 0, 0, None, None ),
    input( "high-input", "proj-x", None, 500, 0, 0, 0, 0, None, None ),
  ];
  let mut params = default_params();
  params.sort_by = SortKey::Input;
  let rows = build_rollup( &entries, &params );

  assert_eq!( rows[ 0 ].group, "high-input", "must sort by input, not total; got: {rows:?}" );
}

/// Test `SortKey::Output` orders by `row.output` alone, not `Total`.
///
/// ## Purpose
/// Guards `sort_rows`'s `Output` match arm against a copy-paste mis-wiring.
///
/// ## Coverage
/// Two rows whose `output` order is the OPPOSITE of their `Total` order.
///
/// ## Validation Strategy
/// `"low-output"` has less output but more input than `"high-output"`; sort
/// by `Output` desc; assert `"high-output"` comes first despite a lower total.
#[ test ]
fn sort_by_output_uses_output_metric()
{
  let entries = vec!
  [
    input( "low-output", "proj-x", None, 900, 10, 0, 0, 0, None, None ),
    input( "high-output", "proj-x", None, 0, 500, 0, 0, 0, None, None ),
  ];
  let mut params = default_params();
  params.sort_by = SortKey::Output;
  let rows = build_rollup( &entries, &params );

  assert_eq!( rows[ 0 ].group, "high-output", "must sort by output, not total; got: {rows:?}" );
}

/// Test `SortKey::Cache` orders by `row.cache()` (`cache_read + cache_creation`),
/// not `Total`.
///
/// ## Purpose
/// Guards `sort_rows`'s `Cache` match arm against a copy-paste mis-wiring.
///
/// ## Coverage
/// Two rows whose combined-cache order is the OPPOSITE of their `Total` order.
///
/// ## Validation Strategy
/// `"low-cache"` has a small cache but a large non-cache total; sort by
/// `Cache` desc; assert `"high-cache"` comes first despite a lower total.
#[ test ]
fn sort_by_cache_uses_cache_metric()
{
  let entries = vec!
  [
    input( "low-cache", "proj-x", None, 900, 0, 5, 0, 0, None, None ),
    input( "high-cache", "proj-x", None, 0, 0, 300, 200, 0, None, None ),
  ];
  let mut params = default_params();
  params.sort_by = SortKey::Cache;
  let rows = build_rollup( &entries, &params );

  assert_eq!( rows[ 0 ].group, "high-cache", "must sort by cache_read+cache_creation, not total; got: {rows:?}" );
}

/// Test `SortKey::Calls` orders by `row.calls` (summed `assistant_entries`),
/// not `Total`.
///
/// ## Purpose
/// Guards `sort_rows`'s `Calls` match arm against a copy-paste mis-wiring —
/// `calls`' own aggregation correctness is covered by
/// `group_by_project_aggregates_sessions`.
///
/// ## Coverage
/// `proj-many-calls` merges 3 sessions (3 calls); `proj-one-call` merges 1 —
/// all with identical (zero) token totals, so only `calls` can drive order.
///
/// ## Validation Strategy
/// Group by `Project`; sort by `Calls` desc; assert the 3-session project
/// sorts first.
#[ test ]
fn sort_by_calls_uses_calls_metric()
{
  let entries = vec!
  [
    input( "sess-a", "proj-one-call", None, 0, 0, 0, 0, 0, None, None ),
    input( "sess-b", "proj-many-calls", None, 0, 0, 0, 0, 0, None, None ),
    input( "sess-c", "proj-many-calls", None, 0, 0, 0, 0, 0, None, None ),
    input( "sess-d", "proj-many-calls", None, 0, 0, 0, 0, 0, None, None ),
  ];
  let mut params = default_params();
  params.group_by = GroupKey::Project;
  params.sort_by = SortKey::Calls;
  let rows = build_rollup( &entries, &params );

  assert_eq!( rows[ 0 ].group, "proj-many-calls", "must sort by calls (3 vs 1); got: {rows:?}" );
}

/// Test `SortKey::Sessions` orders by `row.sessions`, not `Total` or `Calls`.
///
/// ## Purpose
/// Guards `sort_rows`'s `Sessions` match arm against a copy-paste mis-wiring.
///
/// ## Coverage
/// `proj-many-sessions` merges 3 sessions; `proj-one-session` merges 1 — all
/// with identical (zero) token totals, so only `sessions` can drive order.
///
/// ## Validation Strategy
/// Group by `Project`; sort by `Sessions` desc; assert the 3-session project
/// sorts first.
#[ test ]
fn sort_by_sessions_uses_sessions_metric()
{
  let entries = vec!
  [
    input( "sess-a", "proj-one-session", None, 0, 0, 0, 0, 0, None, None ),
    input( "sess-b", "proj-many-sessions", None, 0, 0, 0, 0, 0, None, None ),
    input( "sess-c", "proj-many-sessions", None, 0, 0, 0, 0, 0, None, None ),
    input( "sess-d", "proj-many-sessions", None, 0, 0, 0, 0, 0, None, None ),
  ];
  let mut params = default_params();
  params.group_by = GroupKey::Project;
  params.sort_by = SortKey::Sessions;
  let rows = build_rollup( &entries, &params );

  assert_eq!( rows[ 0 ].group, "proj-many-sessions", "must sort by sessions (3 vs 1); got: {rows:?}" );
}

/// Test `SortKey::Group` orders lexicographically by the group label, ignoring
/// every token metric.
///
/// ## Purpose
/// Guards `sort_rows`'s `Group` match arm against a copy-paste mis-wiring.
///
/// ## Coverage
/// `"alpha"` has a far larger total than `"zulu"`; lexicographic order must
/// still place `"alpha"` first under `Asc`, proving token values are ignored.
///
/// ## Validation Strategy
/// Two rows named `"zulu"` (high total) and `"alpha"` (low total); sort by
/// `Group` asc; assert alphabetical order wins over total.
#[ test ]
fn sort_by_group_is_lexicographic()
{
  let entries = vec!
  [
    input( "zulu", "proj-x", None, 9000, 0, 0, 0, 0, None, None ),
    input( "alpha", "proj-x", None, 1, 0, 0, 0, 0, None, None ),
  ];
  let mut params = default_params();
  params.sort_by = SortKey::Group;
  params.order = SortOrder::Asc;
  let rows = build_rollup( &entries, &params );

  let order : Vec< &str > = rows.iter().map( | r | r.group.as_str() ).collect();
  assert_eq!( order, vec![ "alpha", "zulu" ], "must sort lexicographically, ignoring token totals; got: {order:?}" );
}
