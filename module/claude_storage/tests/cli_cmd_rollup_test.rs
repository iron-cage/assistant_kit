//! Integration tests for the `.rollup` command.
//!
//! ## Source
//!
//! - Command spec: `tests/docs/cli/command/14_rollup.md`
//! - Param specs: `tests/docs/cli/param/34_group.md`, `35_sort.md`, `36_order.md`,
//!   `37_model.md`, `38_columns.md`, `26_depth.md` — each maps its EC cases onto
//!   the INT tests below by function name
//!
//! ## Coverage
//!
//! INT-1 through INT-28 per `tests/docs/cli/command/14_rollup.md` — grouping
//! (session/project/model/day), sort/order wiring, column projection
//! (`columns::`, including the `rank`/`cache_write`/`cache_read` columns
//! added by `Fix(BUG-530)`), the `model::` filter's effect on per-row
//! percentages, `limit::`'s post-aggregation cap semantics (and its
//! interaction with `rank`), the worked-example byte-exact table, and
//! exit/validation codes.
//!
//! `scope::`/`depth::` themselves are NOT re-verified exhaustively here —
//! `.rollup` reuses `validate_scope`/`resolve_scoped_projects`/
//! `resolve_base_path`/`beyond_depth`/`component_distance` byte-for-byte from
//! `.usage`, whose own `cli_cmd_usage_test.rs` (INT-1 through INT-11) already
//! covers every scope value and the depth boundary exhaustively. INT-11/INT-12
//! below are single representative smoke tests confirming the wiring reaches
//! `.rollup`, not a re-derivation of `.usage`'s own coverage.
//!
//! ## Test Case Index
//!
//! | ID | Test Name | Category |
//! |----|-----------|----------|
//! | INT-1 | `int_1_default_group_session_one_row_per_session` | Grouping |
//! | INT-2 | `int_2_group_project_sums_sessions_into_one_row` | Grouping |
//! | INT-3 | `int_3_group_model_separates_rows_by_model` | Grouping |
//! | INT-4 | `int_4_group_day_separates_rows_by_calendar_day` | Grouping |
//! | INT-5 | `int_5_sort_calls_desc_orders_by_call_count` | Sorting & Order |
//! | INT-6 | `int_6_order_asc_reverses_sort_calls_result` | Sorting & Order |
//! | INT-7 | `int_7_columns_custom_subset_projects_only_those` | Column Projection |
//! | INT-8 | `int_8_columns_default_excludes_first_last` | Column Projection |
//! | INT-9 | `int_9_model_filter_recomputes_percent_against_filtered_total` | Filtering |
//! | INT-10 | `int_10_limit_caps_grouped_rows_not_raw_sessions` | Limit Semantics |
//! | INT-11 | `int_11_scope_global_smoke` | Reused Scope Machinery |
//! | INT-12 | `int_12_depth_caps_component_distance_smoke` | Reused Scope Machinery |
//! | INT-13 | `int_13_worked_example_byte_exact` | Worked Example |
//! | INT-14 | `int_14_empty_non_local_scope_exits_0_header_only` | Exit Codes |
//! | INT-15 | `int_15_local_without_project_exits_2` | Exit Codes |
//! | INT-16 | `int_16_invalid_group_rejected` | Input Validation |
//! | INT-17 | `int_17_invalid_sort_rejected` | Input Validation |
//! | INT-18 | `int_18_invalid_order_rejected` | Input Validation |
//! | INT-19 | `int_19_invalid_columns_rejected` | Input Validation |
//! | INT-20 | `int_20_negative_depth_rejected` | Input Validation |
//! | INT-21 | `int_21_negative_limit_rejected` | Input Validation |
//! | INT-22 | `int_22_multiple_parameters_compose_correctly_together` | Composition |
//! | INT-23 | `int_23_model_filter_matching_zero_sessions_exits_0_header_only` | Filtering |
//! | INT-24 | `int_24_columns_first_last_render_timestamps` | Column Projection |
//! | INT-25 | `int_25_columns_rank_numbers_rows_by_sorted_position` | Column Projection |
//! | INT-26 | `int_26_rank_reflects_post_limit_position` | Column Projection |
//! | INT-27 | `int_27_columns_cache_write_cache_read_split_sums_to_cache` | Column Projection |
//! | INT-28 | `int_28_columns_default_excludes_rank_and_cache_split` | Column Projection |
//! | B528 | `bug_528_cross_project_session_id_duplication_inflates_totals` | Bug Reproducer |
//! | B544 | `bug_544_group_header_tracks_dimension_and_sessions_name_project` | Bug Reproducer |

mod common;

use tempfile::TempDir;




/// Fully controlled session fixture: every value `.rollup` aggregates.
///
/// Deliberately a local, parallel copy of `cli_cmd_usage_test.rs`'s own
/// `UsageSession` — this file adds a per-session `model` field that
/// `.usage`'s fixture has no use for (`.usage` never groups or filters by
/// model), so the two builders diverge rather than sharing one
/// over-parameterized helper (same precedent `usage.rs`'s own
/// `session_mtime`/`short_id` local-duplication comments establish).
struct RollupSession< 'a >
{
  cwd : &'a str,
  model : &'a str,
  turns : usize,
  input_tokens : u64,
  output_tokens : u64,
  /// Tokens read from prompt cache (`cache_read_input_tokens`) — the `cache_read`/
  /// `CacheR` column. Split from a single `cache_tokens` field (`Fix(BUG-530)`)
  /// once `columns::` gained separate `cache_write`/`cache_read` projections.
  cache_read_tokens : u64,
  /// Tokens written to prompt cache (`cache_creation_input_tokens`) — the
  /// `cache_write`/`CacheW` column (`Fix(BUG-530)`).
  cache_write_tokens : u64,
  first_ts : &'a str,
  last_ts : &'a str,
}

impl< 'a > RollupSession< 'a >
{
  /// One-turn session, `claude-opus-5`, small token counts.
  fn simple( cwd : &'a str ) -> Self
  {
    Self
    {
      cwd,
      model : "claude-opus-5",
      turns : 1,
      input_tokens : 10,
      output_tokens : 7,
      cache_read_tokens : 0,
      cache_write_tokens : 0,
      first_ts : "2025-06-01T10:00:00Z",
      last_ts : "2025-06-01T10:00:45Z",
    }
  }
}

/// Write a session whose stats are fully controlled: a leading user entry
/// carrying `first_ts`/`cwd`, then `turns` assistant entries (`model` on
/// every line, all tokens on the first, `last_ts` on the final one).
///
/// Returns the encoded project ID.
fn write_rollup_session(
  storage_root : &std::path::Path,
  project_path : &std::path::Path,
  session_id   : &str,
  fx           : &RollupSession< '_ >,
) -> String
{
  use std::io::Write as _;

  let encoded = claude_storage_core::encode_path( project_path )
    .expect( "encode project path" );
  let dir = storage_root.join( "projects" ).join( &encoded );
  std::fs::create_dir_all( &dir ).expect( "create project dir" );
  let path = dir.join( format!( "{session_id}.jsonl" ) );
  let mut file = std::fs::File::create( &path ).expect( "create session file" );

  writeln!(
    file,
    r#"{{"type":"user","uuid":"u-000","parentUuid":null,"timestamp":"{first_ts}","cwd":"{cwd}","sessionId":"{session_id}","version":"2.0.0","gitBranch":"master","userType":"human","isSidechain":false,"message":{{"role":"user","content":"hello work"}}}}"#,
    first_ts = fx.first_ts,
    cwd = fx.cwd,
  )
  .expect( "write user entry" );

  for i in 0..fx.turns
  {
    let ( input, output, cache_read, cache_write ) = if i == 0
    {
      ( fx.input_tokens, fx.output_tokens, fx.cache_read_tokens, fx.cache_write_tokens )
    }
    else
    {
      ( 0, 0, 0, 0 )
    };
    let ts = if i + 1 == fx.turns { fx.last_ts } else { fx.first_ts };
    writeln!(
      file,
      r#"{{"type":"assistant","uuid":"a-{i}","parentUuid":"u-000","timestamp":"{ts}","cwd":"{cwd}","sessionId":"{session_id}","version":"2.0.0","gitBranch":"master","userType":"external","isSidechain":false,"requestId":"req_{i}","message":{{"role":"assistant","model":"{model}","id":"msg_{session_id}_{i}","content":[{{"type":"text","text":"ok"}}],"stop_reason":"end_turn","stop_sequence":null,"usage":{{"input_tokens":{input},"output_tokens":{output},"cache_read_input_tokens":{cache_read},"cache_creation_input_tokens":{cache_write}}}}}}}"#,
      cwd = fx.cwd,
      model = fx.model,
    )
    .expect( "write assistant entry" );
  }

  encoded
}

/// Count of data rows in a `.rollup` table (total lines minus the header).
fn data_rows( s : &str ) -> usize
{
  s.lines().count().saturating_sub( 1 )
}

/// INT-1: Default `group::session` shows one row per session.
///
/// ## Purpose
/// Validates the default grouping dimension: bare `.rollup` behaves like
/// `.usage`'s own granularity, one row per session.
///
/// ## Coverage
/// Two sessions in the same project both appear as separate rows.
///
/// ## Validation Strategy
/// Two sessions, same project; run bare `.rollup`; assert 2 data rows, both
/// short ids present.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-1
#[ test ]
fn int_1_default_group_session_one_row_per_session()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "work" );
  std::fs::create_dir_all( &project ).unwrap();

  write_rollup_session(
    &storage_root, &project, "sessaaa1-1111-4abc-9def-000000000001",
    &RollupSession::simple( project.to_str().unwrap() ),
  );
  write_rollup_session(
    &storage_root, &project, "sessbbb2-2222-4abc-9def-000000000002",
    &RollupSession::simple( project.to_str().unwrap() ),
  );

  let out = common::clg_cmd()
    .current_dir( &project )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert!( s.contains( "sessaaa1" ), "INT-1: first session must appear; got:\n{s}" );
  assert!( s.contains( "sessbbb2" ), "INT-1: second session must appear; got:\n{s}" );
  assert_eq!( data_rows( &s ), 2, "INT-1: one row per session expected; got:\n{s}" );
}

/// INT-2: `group::project` sums multiple sessions into one row.
///
/// ## Purpose
/// Validates project-level aggregation: token totals across every session in
/// a project are summed into a single row, not shown per-session.
///
/// ## Coverage
/// Two sessions (totals 600 and 400) collapse into exactly 1 row whose
/// `Total` is `1.0k` — a value neither session shows alone.
///
/// ## Validation Strategy
/// Two sessions, same project, distinct non-summed-looking totals; run
/// `group::project`; assert exactly 1 row and the summed value.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-2
#[ test ]
fn int_2_group_project_sums_sessions_into_one_row()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "summed" );
  std::fs::create_dir_all( &project ).unwrap();

  let mut fx1 = RollupSession::simple( project.to_str().unwrap() );
  fx1.input_tokens = 600;
  fx1.output_tokens = 0;
  let mut fx2 = RollupSession::simple( project.to_str().unwrap() );
  fx2.input_tokens = 400;
  fx2.output_tokens = 0;

  write_rollup_session( &storage_root, &project, "sumaaaa1-1111-4abc-9def-000000000001", &fx1 );
  write_rollup_session( &storage_root, &project, "sumbbbb2-2222-4abc-9def-000000000002", &fx2 );

  let out = common::clg_cmd()
    .current_dir( &project )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .arg( "group::project" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert_eq!( data_rows( &s ), 1, "INT-2: two sessions in one project must collapse to 1 row; got:\n{s}" );
  assert!( s.contains( "1.0k" ), "INT-2: summed total (600+400=1000) must appear as 1.0k; got:\n{s}" );
}

/// INT-3: `group::model` separates rows by model name.
///
/// ## Purpose
/// Validates model-level aggregation: sessions with different models produce
/// distinct rows labeled by model name.
///
/// ## Coverage
/// Two sessions with distinct models produce 2 rows, each labeled by its
/// model.
///
/// ## Validation Strategy
/// Two sessions, same project, distinct `model`; run `group::model`; assert
/// both model names appear as row labels.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-3
#[ test ]
fn int_3_group_model_separates_rows_by_model()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "models" );
  std::fs::create_dir_all( &project ).unwrap();

  let mut fx1 = RollupSession::simple( project.to_str().unwrap() );
  fx1.model = "claude-opus-5";
  let mut fx2 = RollupSession::simple( project.to_str().unwrap() );
  fx2.model = "claude-haiku-5";

  write_rollup_session( &storage_root, &project, "modaaaa1-1111-4abc-9def-000000000001", &fx1 );
  write_rollup_session( &storage_root, &project, "modbbbb2-2222-4abc-9def-000000000002", &fx2 );

  let out = common::clg_cmd()
    .current_dir( &project )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .arg( "group::model" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert!( s.contains( "claude-opus-5" ), "INT-3: opus row must appear; got:\n{s}" );
  assert!( s.contains( "claude-haiku-5" ), "INT-3: haiku row must appear; got:\n{s}" );
  assert_eq!( data_rows( &s ), 2, "INT-3: distinct models must not merge; got:\n{s}" );
}

/// INT-4: `group::day` separates rows by calendar day.
///
/// ## Purpose
/// Validates day-level aggregation: sessions whose `first_timestamp` falls on
/// different calendar days produce distinct rows labeled `YYYY-MM-DD`.
///
/// ## Coverage
/// Two sessions on different days produce 2 rows, each labeled by date.
///
/// ## Validation Strategy
/// Two sessions, same project, distinct `first_ts` dates; run `group::day`;
/// assert both date labels appear.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-4
#[ test ]
fn int_4_group_day_separates_rows_by_calendar_day()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "days" );
  std::fs::create_dir_all( &project ).unwrap();

  let mut fx1 = RollupSession::simple( project.to_str().unwrap() );
  fx1.first_ts = "2025-06-01T10:00:00Z";
  let mut fx2 = RollupSession::simple( project.to_str().unwrap() );
  fx2.first_ts = "2025-06-05T10:00:00Z";

  write_rollup_session( &storage_root, &project, "dayaaaa1-1111-4abc-9def-000000000001", &fx1 );
  write_rollup_session( &storage_root, &project, "daybbbb2-2222-4abc-9def-000000000002", &fx2 );

  let out = common::clg_cmd()
    .current_dir( &project )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .arg( "group::day" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert!( s.contains( "2025-06-01" ), "INT-4: first day must appear; got:\n{s}" );
  assert!( s.contains( "2025-06-05" ), "INT-4: second day must appear; got:\n{s}" );
  assert_eq!( data_rows( &s ), 2, "INT-4: distinct days must not merge; got:\n{s}" );
}

/// INT-5: `sort::calls order::desc` orders rows by call count, not by total.
///
/// ## Purpose
/// Validates `sort::` wiring: choosing `calls` reorders rows away from the
/// default `total`-based order.
///
/// ## Coverage
/// Three sessions with calls/total deliberately inversely correlated (fewest
/// calls has the highest total) — `sort::calls order::desc` must show the
/// most-calls session first, the fewest-calls session last.
///
/// ## Validation Strategy
/// S1(1 call,total 300), S2(3 calls,total 200), S3(5 calls,total 100); run
/// `sort::calls order::desc`; assert byte-offset order S3 < S2 < S1.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-5
#[ test ]
fn int_5_sort_calls_desc_orders_by_call_count()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "sortproj" );
  std::fs::create_dir_all( &project ).unwrap();

  for ( id, turns, input ) in
  [
    ( "sortaaa1-1111-4abc-9def-000000000001", 1_usize, 300_u64 ),
    ( "sortbbb2-2222-4abc-9def-000000000002", 3, 200 ),
    ( "sortccc3-3333-4abc-9def-000000000003", 5, 100 ),
  ]
  {
    let mut fx = RollupSession::simple( project.to_str().unwrap() );
    fx.turns = turns;
    fx.input_tokens = input;
    fx.output_tokens = 0;
    write_rollup_session( &storage_root, &project, id, &fx );
  }

  let out = common::clg_cmd()
    .current_dir( &project )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .arg( "sort::calls" )
    .arg( "order::desc" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  let a = s.find( "sortaaa1" ).expect( "sortaaa1 row must exist" );
  let b = s.find( "sortbbb2" ).expect( "sortbbb2 row must exist" );
  let c = s.find( "sortccc3" ).expect( "sortccc3 row must exist" );
  assert!( c < b && b < a, "INT-5: sort::calls order::desc must order 5>3>1 calls; got:\n{s}" );
}

/// INT-6: `order::asc` reverses the `sort::calls` result from INT-5.
///
/// ## Purpose
/// Validates `order::` wiring independent of `sort::`: the same sort key
/// under `asc` produces the exact reverse row order of `desc`.
///
/// ## Coverage
/// Same 3-session fixture as INT-5; `sort::calls order::asc` must show the
/// fewest-calls session first, most-calls last — reversed from INT-5.
///
/// ## Validation Strategy
/// Identical fixture to INT-5; run `sort::calls order::asc`; assert
/// byte-offset order S1 < S2 < S3.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-6
#[ test ]
fn int_6_order_asc_reverses_sort_calls_result()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "sortproj" );
  std::fs::create_dir_all( &project ).unwrap();

  for ( id, turns, input ) in
  [
    ( "sortaaa1-1111-4abc-9def-000000000001", 1_usize, 300_u64 ),
    ( "sortbbb2-2222-4abc-9def-000000000002", 3, 200 ),
    ( "sortccc3-3333-4abc-9def-000000000003", 5, 100 ),
  ]
  {
    let mut fx = RollupSession::simple( project.to_str().unwrap() );
    fx.turns = turns;
    fx.input_tokens = input;
    fx.output_tokens = 0;
    write_rollup_session( &storage_root, &project, id, &fx );
  }

  let out = common::clg_cmd()
    .current_dir( &project )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .arg( "sort::calls" )
    .arg( "order::asc" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  let a = s.find( "sortaaa1" ).expect( "sortaaa1 row must exist" );
  let b = s.find( "sortbbb2" ).expect( "sortbbb2 row must exist" );
  let c = s.find( "sortccc3" ).expect( "sortccc3 row must exist" );
  assert!( a < b && b < c, "INT-6: order::asc must reverse INT-5's order to 1<3<5 calls; got:\n{s}" );
}

/// INT-7: `columns::` custom subset projects only the chosen columns.
///
/// ## Purpose
/// Validates column projection: an explicit `columns::` list shows exactly
/// those columns, nothing else.
///
/// ## Coverage
/// `columns::group,total` header shows `Group`/`Total` only — every other
/// column label absent.
///
/// ## Validation Strategy
/// One session; run `columns::group,total`; assert header line contains only
/// the two chosen labels.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-7
#[ test ]
fn int_7_columns_custom_subset_projects_only_those()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "cols" );
  std::fs::create_dir_all( &project ).unwrap();

  write_rollup_session(
    &storage_root, &project, "colsaaa1-1111-4abc-9def-000000000001",
    &RollupSession::simple( project.to_str().unwrap() ),
  );

  let out = common::clg_cmd()
    .current_dir( &project )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .arg( "columns::group,total" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let header = common::stdout( &out ).lines().next().unwrap_or_default().to_string();
  // `Fix(BUG-544)`: the group column's header is the active grouping dimension,
  // so the default `group::session` labels it `Session`, not a constant `Group`.
  assert!( header.starts_with( "Session " ), "INT-7: group column must be labelled Session; got:\n{header}" );
  assert!( header.contains( "Total" ), "INT-7: Total column must appear; got:\n{header}" );
  for absent in [ "Project", "Sessions", "Calls", "Input", "Output", "Cache", "MaxCtx", "Pct", "First", "Last" ]
  {
    assert!( !header.contains( absent ), "INT-7: {absent} must be excluded; got:\n{header}" );
  }
}

/// INT-8: Default `columns::` excludes `First`/`Last`.
///
/// ## Purpose
/// Validates the default column set boundary: every count/token metric shows
/// by default, but the verbose timestamp columns do not.
///
/// ## Coverage
/// Header contains all 9 default labels; `First`/`Last` are absent.
///
/// ## Validation Strategy
/// One session; run bare `.rollup`; assert the full default label set and the
/// two omitted labels.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-8
#[ test ]
fn int_8_columns_default_excludes_first_last()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "defcols" );
  std::fs::create_dir_all( &project ).unwrap();

  write_rollup_session(
    &storage_root, &project, "defcaaa1-1111-4abc-9def-000000000001",
    &RollupSession::simple( project.to_str().unwrap() ),
  );

  let out = common::clg_cmd()
    .current_dir( &project )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let header = common::stdout( &out ).lines().next().unwrap_or_default().to_string();
  // `Fix(BUG-544)`: under the default `group::session` the group column is
  // labelled `Session` and a `Project` column follows it, so a bare session id
  // is always attributable to a directory.
  assert!( header.starts_with( "Session " ), "INT-8: group column must be labelled Session; got:\n{header}" );
  for present in [ "Project", "Sessions", "Calls", "Input", "Output", "Cache", "MaxCtx", "Total", "Pct" ]
  {
    assert!( header.contains( present ), "INT-8: default column {present} must appear; got:\n{header}" );
  }
  assert!( !header.contains( "First" ), "INT-8: First must be excluded by default; got:\n{header}" );
  assert!( !header.contains( "Last" ), "INT-8: Last must be excluded by default; got:\n{header}" );
}

/// INT-9: `model::` filters sessions before grouping; `Pct` recomputes
/// against the filtered total only.
///
/// ## Purpose
/// Validates the filter-then-aggregate order documented on
/// `RollupParams::model_filter` (core engine): a non-matching session is
/// dropped entirely — including from the percent denominator, not merely
/// hidden from view.
///
/// ## Coverage
/// Two `opus` sessions (100 tokens each) and one `haiku` session (800
/// tokens); with `model::opus`, each surviving row shows `50.0%`, not the
/// `10.0%` it would show against the unfiltered 1000-token grand total.
///
/// ## Validation Strategy
/// 3-session fixture; run `model::opus`; assert 2 rows, haiku session
/// absent, and `50.0%` present (not `10.0%`).
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-9
#[ test ]
fn int_9_model_filter_recomputes_percent_against_filtered_total()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "filtered" );
  std::fs::create_dir_all( &project ).unwrap();

  for ( id, model, input ) in
  [
    ( "opusaaa1-1111-4abc-9def-000000000001", "claude-opus-5", 100_u64 ),
    ( "opusbbb2-2222-4abc-9def-000000000002", "claude-opus-5", 100 ),
    ( "haikccc3-3333-4abc-9def-000000000003", "claude-haiku-5", 800 ),
  ]
  {
    let mut fx = RollupSession::simple( project.to_str().unwrap() );
    fx.model = model;
    fx.input_tokens = input;
    fx.output_tokens = 0;
    write_rollup_session( &storage_root, &project, id, &fx );
  }

  let out = common::clg_cmd()
    .current_dir( &project )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .arg( "model::opus" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert!( s.contains( "opusaaa1" ), "INT-9: first opus session must survive; got:\n{s}" );
  assert!( s.contains( "opusbbb2" ), "INT-9: second opus session must survive; got:\n{s}" );
  assert!( !s.contains( "haikccc3" ), "INT-9: haiku session must be filtered out; got:\n{s}" );
  assert_eq!( data_rows( &s ), 2, "INT-9: exactly 2 surviving rows expected; got:\n{s}" );
  assert!( s.contains( "50.0%" ), "INT-9: percent must be against the FILTERED total (50.0%), not unfiltered (10.0%); got:\n{s}" );
  assert!( !s.contains( "10.0%" ), "INT-9: unfiltered-total percent must never appear; got:\n{s}" );
}

/// INT-10: `limit::` caps the grouped row count, not the raw session count.
///
/// ## Purpose
/// Validates `limit::`'s post-aggregation semantics for `.rollup` — distinct
/// from `.usage`'s flat per-session cap (`.rollup` caps AFTER `group::`
/// collapses sessions into rows).
///
/// ## Coverage
/// Three distinct projects (totals 900/600/300); `group::project limit::2`
/// keeps the two highest-total rows, drops the lowest.
///
/// ## Validation Strategy
/// Three single-session projects with distinct totals; run `group::project
/// limit::2`; assert exactly 2 rows and the specific totals kept/dropped.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-10
#[ test ]
fn int_10_limit_caps_grouped_rows_not_raw_sessions()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  for ( rel, id, input ) in
  [
    ( "p1", "limpaaa1-1111-4abc-9def-000000000001", 900_u64 ),
    ( "p2", "limpbbb2-2222-4abc-9def-000000000002", 600 ),
    ( "p3", "limpccc3-3333-4abc-9def-000000000003", 300 ),
  ]
  {
    let p = root.path().join( rel );
    let mut fx = RollupSession::simple( p.to_str().unwrap() );
    fx.input_tokens = input;
    fx.output_tokens = 0;
    write_rollup_session( &storage_root, &p, id, &fx );
  }

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .arg( "scope::global" )
    .arg( "group::project" )
    .arg( "limit::2" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert_eq!( data_rows( &s ), 2, "INT-10: limit::2 must cap grouped rows to 2; got:\n{s}" );
  assert!( s.contains( "900" ), "INT-10: highest-total row must survive; got:\n{s}" );
  assert!( s.contains( "600" ), "INT-10: second-highest row must survive; got:\n{s}" );
  assert!( !s.contains( "300" ), "INT-10: lowest-total row must be cut entirely; got:\n{s}" );
}

/// INT-11: `scope::global` reaches `.rollup` (representative smoke test).
///
/// ## Purpose
/// Confirms `.rollup` genuinely wires `scope::` into `resolve_scoped_projects`
/// — not an exhaustive re-derivation of `.usage`'s own 5-scope-value
/// coverage (`cli_cmd_usage_test.rs` INT-1 through INT-5).
///
/// ## Coverage
/// Two unrelated projects both appear under `scope::global`.
///
/// ## Validation Strategy
/// Two sessions in unrelated projects; run `scope::global`; assert both
/// appear.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-11
#[ test ]
fn int_11_scope_global_smoke()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  for ( rel, id ) in [ ( "ga", "glbtaaa1-1111-4abc-9def-000000000001" ), ( "gb", "glbtbbb2-2222-4abc-9def-000000000002" ) ]
  {
    let p = root.path().join( rel );
    write_rollup_session( &storage_root, &p, id, &RollupSession::simple( p.to_str().unwrap() ) );
  }

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert!( s.contains( "glbtaaa1" ), "INT-11: first unrelated project must appear; got:\n{s}" );
  assert!( s.contains( "glbtbbb2" ), "INT-11: second unrelated project must appear; got:\n{s}" );
}

/// INT-12: `depth::` caps candidates beyond the component distance
/// (representative smoke test).
///
/// ## Purpose
/// Confirms `.rollup` genuinely wires `depth::` into the same
/// `beyond_depth`/`component_distance` boundary check `.usage` already
/// exhaustively tests (`cli_cmd_usage_test.rs` INT-7/INT-8) — not a
/// re-derivation.
///
/// ## Coverage
/// Distance 0 and 1 kept; distance 2 dropped under `depth::1`.
///
/// ## Validation Strategy
/// Projects at `a`, `a/b`, `a/b/c`; run `scope::under path::<a> depth::1`;
/// assert the cut.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-12
#[ test ]
fn int_12_depth_caps_component_distance_smoke()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  std::fs::create_dir_all( root.path().join( "a/b/c" ) ).unwrap();

  for ( rel, id ) in
  [
    ( "a", "deptaaa1-1111-4abc-9def-000000000001" ),
    ( "a/b", "deptbbb2-2222-4abc-9def-000000000002" ),
    ( "a/b/c", "deptccc3-3333-4abc-9def-000000000003" ),
  ]
  {
    let p = root.path().join( rel );
    write_rollup_session( &storage_root, &p, id, &RollupSession::simple( p.to_str().unwrap() ) );
  }

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", root.path().join( "a" ).display() ) )
    .arg( "depth::1" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert!( s.contains( "deptaaa1" ), "INT-12: distance-0 project must survive depth::1; got:\n{s}" );
  assert!( s.contains( "deptbbb2" ), "INT-12: distance-1 project must survive depth::1; got:\n{s}" );
  assert!( !s.contains( "deptccc3" ), "INT-12: distance-2 project must be dropped by depth::1; got:\n{s}" );
}

/// INT-13: Full table render matches the worked example byte-for-byte.
///
/// ## Purpose
/// Locks in the exact column widths, alignment, and formatting of the
/// metric-column render (`Fix(BUG-544)` moved default-set column *order* into
/// INT-14, whose header-only output stays deterministic).
///
/// ## Coverage
/// Full-table equality — header plus 2 data rows — against real captured
/// binary output.
///
/// ## Validation Strategy
/// Two sessions with controlled counts (`500/300/200` and `100/50/50`
/// input/output/cache); assert stdout equals the exact captured table.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-13
#[ test ]
fn int_13_worked_example_byte_exact()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "worked" );
  std::fs::create_dir_all( &project ).unwrap();

  let s1 = RollupSession
  {
    cwd : project.to_str().unwrap(),
    model : "claude-opus-5",
    turns : 4,
    input_tokens : 500,
    output_tokens : 300,
    cache_read_tokens : 200,
    cache_write_tokens : 0,
    first_ts : "2025-06-01T10:00:00Z",
    last_ts : "2025-06-01T10:00:45Z",
  };
  let s2 = RollupSession
  {
    cwd : project.to_str().unwrap(),
    model : "claude-opus-5",
    turns : 2,
    input_tokens : 100,
    output_tokens : 50,
    cache_read_tokens : 50,
    cache_write_tokens : 0,
    first_ts : "2025-06-01T09:00:00Z",
    last_ts : "2025-06-01T09:00:10Z",
  };

  write_rollup_session( &storage_root, &project, "aaaaaaaa-1111-4abc-9def-000000000001", &s1 );
  write_rollup_session( &storage_root, &project, "bbbbbbbb-2222-4abc-9def-000000000002", &s2 );

  let out = common::clg_cmd()
    .current_dir( &project )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    // `Fix(BUG-544)`: the bare default now also renders `Project`, whose value
    // is the session's recorded cwd — a per-run `TempDir` path, so it cannot
    // appear in a byte-exact literal. Pinning the metric projection explicitly
    // keeps this test's actual subject (widths, alignment, number formatting)
    // deterministic; the default set's own column order stays byte-locked by
    // INT-14, which renders the header with no data rows.
    .arg( "columns::group,sessions,calls,input,output,cache,max_context,total,percent" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let expected = "\
Session                   Sessions   Calls     Input    Output     Cache    MaxCtx     Total     Pct\n\
aaaaaaaa                         1       4       500       300       200       700      1.0k   83.3%\n\
bbbbbbbb                         1       2       100        50        50       150       200   16.7%\n";
  assert_eq!(
    common::stdout( &out ),
    expected,
    "INT-13: full table must match the captured worked example byte-for-byte"
  );
}

/// INT-14: No matching sessions in non-local scope exits 0 with header-only
/// output.
///
/// ## Purpose
/// Validates the empty-result contract, matching `.usage`'s own INT-17.
///
/// ## Coverage
/// Exit 0; stdout is exactly the header row; stderr empty.
///
/// ## Validation Strategy
/// Empty storage, `scope::global`; assert exit 0 and header-only stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-14
#[ test ]
fn int_14_empty_non_local_scope_exits_0_header_only()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  std::fs::create_dir_all( storage_root.join( "projects" ) ).unwrap();

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  assert_eq!(
    common::stdout( &out ),
    // `Fix(BUG-544)`: default `group::session` labels the group column `Session`
    // and inserts `Project` after it. With zero data rows this line is fully
    // deterministic, so it — not INT-13 — is the byte-exact lock on default
    // column order.
    "Session                   Project                   Sessions   Calls     Input    Output     Cache    MaxCtx     Total     Pct\n",
    "INT-14: zero-row result must print exactly the header row"
  );
  assert!( common::stderr( &out ).is_empty(), "INT-14: no error output expected; got: {}", common::stderr( &out ) );
}

/// INT-15: `scope::local` with no project at cwd exits 2.
///
/// ## Purpose
/// Validates the local-scope storage error, matching `.usage`'s own INT-18.
///
/// ## Coverage
/// Exit 2; stderr names the missing current-directory project.
///
/// ## Validation Strategy
/// Valid empty storage, cwd with no project, bare `.rollup`; assert exit 2
/// and the stderr message.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-15
#[ test ]
fn int_15_local_without_project_exits_2()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  std::fs::create_dir_all( storage_root.join( "projects" ) ).unwrap();

  let out = common::clg_cmd()
    .current_dir( root.path() )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .output()
    .unwrap();

  common::assert_exit( &out, 2 );
  assert!(
    common::stderr( &out ).contains( "No project found for current directory" ),
    "INT-15: stderr must name the missing cwd project; got: {}",
    common::stderr( &out )
  );
}

/// INT-16: Invalid `group::` value rejected.
///
/// ## Purpose
/// Validates `group::` validation: an unrecognized value is an argument
/// error naming the bad value.
///
/// ## Coverage
/// Exit 1; stderr names `bogus`; no table on stdout.
///
/// ## Validation Strategy
/// Run `.rollup group::bogus`; assert exit, stderr content, empty stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-16
#[ test ]
fn int_16_invalid_group_rejected()
{
  let out = common::clg_cmd().arg( ".rollup" ).arg( "group::bogus" ).output().unwrap();

  common::assert_exit( &out, 1 );
  assert!( common::stderr( &out ).contains( "bogus" ), "INT-16: stderr must name the invalid value; got: {}", common::stderr( &out ) );
  assert!( common::stdout( &out ).is_empty(), "INT-16: no table output expected; got:\n{}", common::stdout( &out ) );
}

/// INT-17: Invalid `sort::` value rejected.
///
/// ## Purpose
/// Validates `sort::` validation: an unrecognized value is an argument error
/// naming the bad value.
///
/// ## Coverage
/// Exit 1; stderr names `bogus`; no table on stdout.
///
/// ## Validation Strategy
/// Run `.rollup sort::bogus`; assert exit, stderr content, empty stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-17
#[ test ]
fn int_17_invalid_sort_rejected()
{
  let out = common::clg_cmd().arg( ".rollup" ).arg( "sort::bogus" ).output().unwrap();

  common::assert_exit( &out, 1 );
  assert!( common::stderr( &out ).contains( "bogus" ), "INT-17: stderr must name the invalid value; got: {}", common::stderr( &out ) );
  assert!( common::stdout( &out ).is_empty(), "INT-17: no table output expected; got:\n{}", common::stdout( &out ) );
}

/// INT-18: Invalid `order::` value rejected.
///
/// ## Purpose
/// Validates `order::` validation: an unrecognized value is an argument error
/// naming the bad value.
///
/// ## Coverage
/// Exit 1; stderr names `bogus`; no table on stdout.
///
/// ## Validation Strategy
/// Run `.rollup order::bogus`; assert exit, stderr content, empty stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-18
#[ test ]
fn int_18_invalid_order_rejected()
{
  let out = common::clg_cmd().arg( ".rollup" ).arg( "order::bogus" ).output().unwrap();

  common::assert_exit( &out, 1 );
  assert!( common::stderr( &out ).contains( "bogus" ), "INT-18: stderr must name the invalid value; got: {}", common::stderr( &out ) );
  assert!( common::stdout( &out ).is_empty(), "INT-18: no table output expected; got:\n{}", common::stdout( &out ) );
}

/// INT-19: Invalid `columns::` entry rejected.
///
/// ## Purpose
/// Validates `columns::` validation: an unrecognized column name is an
/// argument error naming the bad value, even alongside valid entries.
///
/// ## Coverage
/// Exit 1; stderr names `bogus` and lists every one of the 14 valid keys,
/// including the 3 opt-in-only ones (`rank`/`cache_write`/`cache_read`); no
/// table on stdout.
///
/// ## Validation Strategy
/// Run `.rollup columns::group,bogus`; assert exit, stderr content, empty
/// stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-19
#[ test ]
fn int_19_invalid_columns_rejected()
{
  let out = common::clg_cmd().arg( ".rollup" ).arg( "columns::group,bogus" ).output().unwrap();

  common::assert_exit( &out, 1 );
  let err = common::stderr( &out );
  assert!( err.contains( "bogus" ), "INT-19: stderr must name the invalid value; got: {err}" );
  for key in [ "rank", "cache_write", "cache_read" ]
  {
    assert!( err.contains( key ), "INT-19: valid-keys list must include the opt-in column {key} (Fix(BUG-530)); got: {err}" );
  }
  assert!( common::stdout( &out ).is_empty(), "INT-19: no table output expected; got:\n{}", common::stdout( &out ) );
}

/// INT-20: Negative `depth::` is rejected.
///
/// ## Purpose
/// Validates depth validation is reused unchanged from `.usage`, matching
/// its own INT-20.
///
/// ## Coverage
/// Exit 1; stderr is exactly `depth must be non-negative`; no stdout table.
///
/// ## Validation Strategy
/// Run `.rollup depth::-1`; assert exit, exact stderr, empty stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-20
#[ test ]
fn int_20_negative_depth_rejected()
{
  let out = common::clg_cmd().arg( ".rollup" ).arg( "depth::-1" ).output().unwrap();

  common::assert_exit( &out, 1 );
  assert_eq!( common::stderr( &out ).trim(), "depth must be non-negative", "INT-20: stderr must be exactly the documented message" );
  assert!( common::stdout( &out ).is_empty(), "INT-20: no table output expected; got:\n{}", common::stdout( &out ) );
}

/// INT-21: Negative `limit::` is rejected.
///
/// ## Purpose
/// Validates limit validation is reused unchanged from `.usage`, matching
/// its own INT-21.
///
/// ## Coverage
/// Exit 1; stderr is exactly `limit must be non-negative`; no stdout table.
///
/// ## Validation Strategy
/// Run `.rollup limit::-1`; assert exit, exact stderr, empty stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-21
#[ test ]
fn int_21_negative_limit_rejected()
{
  let out = common::clg_cmd().arg( ".rollup" ).arg( "limit::-1" ).output().unwrap();

  common::assert_exit( &out, 1 );
  assert_eq!( common::stderr( &out ).trim(), "limit must be non-negative", "INT-21: stderr must be exactly the documented message" );
  assert!( common::stdout( &out ).is_empty(), "INT-21: no table output expected; got:\n{}", common::stdout( &out ) );
}

/// INT-22: Multiple parameters compose correctly in a single invocation.
///
/// ## Purpose
/// Every INT-1 through INT-21 test varies exactly one parameter at a time.
/// None of them prove `group::`/`sort::`/`order::`/`columns::`/`limit::`
/// still behave correctly when all five are set together in one call — this
/// closes that composition gap.
///
/// ## Coverage
/// `group::model sort::sessions order::asc columns::group,sessions limit::1`
/// together: grouping picks the model dimension, sorting picks the
/// least-common metric ascending (opposite of every other test's `desc`),
/// `limit::1` keeps only the ascending-sorted winner, and `columns::` hides
/// every field except the two requested.
///
/// ## Validation Strategy
/// Three models with distinct session counts (`opus`:1, `haiku`:2,
/// `sonnet`:3); run all five parameters together; assert exactly 1 data row,
/// it is the 1-session `opus` group, and only the `Group`/`Sessions` headers
/// are present — no other column label leaks through.
#[ test ]
fn int_22_multiple_parameters_compose_correctly_together()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "combo" );
  std::fs::create_dir_all( &project ).unwrap();

  for ( id, model ) in
  [
    ( "int22a01-1111-4abc-9def-000000000001", "claude-opus-5" ),
    ( "int22b02-2222-4abc-9def-000000000002", "claude-haiku-5" ),
    ( "int22b03-3333-4abc-9def-000000000003", "claude-haiku-5" ),
    ( "int22c04-4444-4abc-9def-000000000004", "claude-sonnet-5" ),
    ( "int22c05-5555-4abc-9def-000000000005", "claude-sonnet-5" ),
    ( "int22c06-6666-4abc-9def-000000000006", "claude-sonnet-5" ),
  ]
  {
    let mut fx = RollupSession::simple( project.to_str().unwrap() );
    fx.model = model;
    write_rollup_session( &storage_root, &project, id, &fx );
  }

  let out = common::clg_cmd()
    .current_dir( &project )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .arg( "group::model" )
    .arg( "sort::sessions" )
    .arg( "order::asc" )
    .arg( "columns::group,sessions" )
    .arg( "limit::1" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert_eq!( data_rows( &s ), 1, "INT-22: limit::1 must cap to exactly 1 row; got:\n{s}" );
  assert!( s.contains( "opus" ), "INT-22: ascending sort::sessions must keep the 1-session opus group first; got:\n{s}" );
  assert!( !s.contains( "haiku" ) && !s.contains( "sonnet" ), "INT-22: only the winning row may appear; got:\n{s}" );
  let header = s.lines().next().unwrap_or_default();
  // `Fix(BUG-544)`: with `group::model` the group column is labelled `Model`.
  assert!( header.starts_with( "Model " ) && header.contains( "Sessions" ), "INT-22: requested columns must be present; got header:\n{header}" );
  for absent in [ "Calls", "Input", "Output", "Cache", "MaxCtx", "Total", "Pct" ]
  {
    assert!( !header.contains( absent ), "INT-22: columns::group,sessions must hide {absent}; got header:\n{header}" );
  }
}

/// INT-23: `model::` matching zero sessions exits 0 with header-only output.
///
/// ## Purpose
/// INT-14 covers empty *storage* (no projects at all). This covers the
/// distinct case where storage has real sessions but `model::` filters every
/// one of them out — the zero-grand-total percent branch
/// (`percent_is_zero_not_nan_when_grand_total_is_zero` at the unit level)
/// exercised end-to-end through the CLI, confirming it never panics or
/// prints `NaN`/`inf` when there is simply nothing left to divide.
///
/// ## Coverage
/// One real session that exists but does not match `model::`.
///
/// ## Validation Strategy
/// One `claude-opus-5` session; run `.rollup model::nonexistent-model-xyz`;
/// assert exit 0, zero data rows, the header is still printed, and no
/// `NaN`/`inf` leaks into stdout.
#[ test ]
fn int_23_model_filter_matching_zero_sessions_exits_0_header_only()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "nomatch" );
  std::fs::create_dir_all( &project ).unwrap();

  let fx = RollupSession::simple( project.to_str().unwrap() );
  write_rollup_session( &storage_root, &project, "int23xa1-1111-4abc-9def-000000000001", &fx );

  let out = common::clg_cmd()
    .current_dir( &project )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .arg( "model::nonexistent-model-xyz" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert_eq!( data_rows( &s ), 0, "INT-23: every session must be filtered out; got:\n{s}" );
  // `Fix(BUG-544)`: default `group::session` labels the group column `Session`.
  assert!(
    s.lines().next().unwrap_or_default().starts_with( "Session " ),
    "INT-23: the header row must still print; got:\n{s}"
  );
  assert!( !s.to_lowercase().contains( "nan" ) && !s.to_lowercase().contains( "inf" ), "INT-23: zero-total percent must never render NaN/inf; got:\n{s}" );
}

/// INT-24: `columns::` including `first`/`last` renders raw ISO-8601 timestamps.
///
/// ## Purpose
/// INT-7/INT-8 only ever project *default-set* columns. Neither exercises
/// the two columns excluded from the default (`first`/`last`) actually
/// being requested and rendered — this closes that gap.
///
/// ## Coverage
/// One session with known, distinct `first_ts`/`last_ts` values.
///
/// ## Validation Strategy
/// Run `.rollup columns::group,first,last`; assert the header shows exactly
/// those three labels and the data row contains both raw timestamp strings
/// verbatim (no reformatting, no truncation).
#[ test ]
fn int_24_columns_first_last_render_timestamps()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "tsproj" );
  std::fs::create_dir_all( &project ).unwrap();

  let mut fx = RollupSession::simple( project.to_str().unwrap() );
  fx.first_ts = "2025-06-01T10:00:00Z";
  fx.last_ts = "2025-06-01T10:00:45Z";
  write_rollup_session( &storage_root, &project, "int24fla-1111-4abc-9def-000000000001", &fx );

  let out = common::clg_cmd()
    .current_dir( &project )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .arg( "columns::group,first,last" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  let header = s.lines().next().unwrap_or_default();
  assert!( header.contains( "First" ) && header.contains( "Last" ), "INT-24: requested First/Last headers must be present; got:\n{header}" );
  assert!( !header.contains( "Sessions" ) && !header.contains( "Calls" ), "INT-24: unrequested columns must be absent; got:\n{header}" );
  assert!( s.contains( "2025-06-01T10:00:00Z" ), "INT-24: raw first_ts must render verbatim; got:\n{s}" );
  assert!( s.contains( "2025-06-01T10:00:45Z" ), "INT-24: raw last_ts must render verbatim; got:\n{s}" );
}

/// INT-25: `columns::rank` renders each row's 1-indexed sorted position.
///
/// ## Purpose
/// Validates the new `rank` column (`Fix(BUG-530)`): the CLI-synthesized
/// display position tracks `sort::`'s actual output order, not input order
/// or group-label order.
///
/// ## Coverage
/// Three projects with distinct totals (900/600/300); default `sort::total
/// order::desc` must number them `1`, `2`, `3` top to bottom.
///
/// ## Validation Strategy
/// Three single-session projects with distinct totals; run `group::project
/// columns::rank,group,total`; split each data line on whitespace and assert
/// the `Rank` field is `1`/`2`/`3` in row order, paired with the matching
/// total.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-25
#[ test ]
fn int_25_columns_rank_numbers_rows_by_sorted_position()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  for ( rel, id, input ) in
  [
    ( "r1", "rankaaa1-1111-4abc-9def-000000000001", 900_u64 ),
    ( "r2", "rankbbb2-2222-4abc-9def-000000000002", 600 ),
    ( "r3", "rankccc3-3333-4abc-9def-000000000003", 300 ),
  ]
  {
    let p = root.path().join( rel );
    let mut fx = RollupSession::simple( p.to_str().unwrap() );
    fx.input_tokens = input;
    fx.output_tokens = 0;
    write_rollup_session( &storage_root, &p, id, &fx );
  }

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .arg( "scope::global" )
    .arg( "group::project" )
    .arg( "columns::rank,group,total" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  let lines : Vec< &str > = s.lines().skip( 1 ).collect();
  assert_eq!( lines.len(), 3, "INT-25: 3 projects must yield 3 rows; got:\n{s}" );
  for ( expected_rank, expected_total ) in [ ( "1", "900" ), ( "2", "600" ), ( "3", "300" ) ]
  {
    let line = lines.iter().find( | l | l.contains( expected_total ) )
      .unwrap_or_else( || panic!( "INT-25: row with total {expected_total} must exist; got:\n{s}" ) );
    let fields : Vec< &str > = line.split_whitespace().collect();
    assert_eq!( fields[ 0 ], expected_rank, "INT-25: row with total {expected_total} must carry rank {expected_rank}; got row:\n{line}" );
  }
}

/// INT-26: `rank` reflects final rendered position after `limit::` truncates,
/// not the row's position in the full unlimited set.
///
/// ## Purpose
/// `limit::` truncation happens inside `build_rollup()` (core engine, per
/// Algorithm step 8) BEFORE the CLI's render loop ever sees the rows — this
/// locks in that `rank` is computed from the already-limited slice, never
/// leaking a pre-limit index. Regression guard: a future refactor that moved
/// rank computation earlier (e.g. before `limit::` applied) would silently
/// break this without a dedicated assertion.
///
/// ## Coverage
/// Four projects (totals 900/700/500/300); `limit::2` must keep only the top
/// 2 by total, numbered `1`/`2` — never `3`/`4`, and the 2 dropped totals
/// must be entirely absent.
///
/// ## Validation Strategy
/// Four single-session projects with distinct totals; run `group::project
/// limit::2 columns::rank,total`; assert exactly 2 rows, ranks `1`/`2` paired
/// with totals `900`/`700`, and totals `500`/`300` absent from stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-26
#[ test ]
fn int_26_rank_reflects_post_limit_position()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  for ( rel, id, input ) in
  [
    ( "l1", "ranklim1-1111-4abc-9def-000000000001", 900_u64 ),
    ( "l2", "ranklim2-2222-4abc-9def-000000000002", 700 ),
    ( "l3", "ranklim3-3333-4abc-9def-000000000003", 500 ),
    ( "l4", "ranklim4-4444-4abc-9def-000000000004", 300 ),
  ]
  {
    let p = root.path().join( rel );
    let mut fx = RollupSession::simple( p.to_str().unwrap() );
    fx.input_tokens = input;
    fx.output_tokens = 0;
    write_rollup_session( &storage_root, &p, id, &fx );
  }

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .arg( "scope::global" )
    .arg( "group::project" )
    .arg( "limit::2" )
    .arg( "columns::rank,total" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert_eq!( data_rows( &s ), 2, "INT-26: limit::2 must cap to exactly 2 rows; got:\n{s}" );
  assert!( !s.contains( "500" ), "INT-26: dropped row's total must be entirely absent; got:\n{s}" );
  assert!( !s.contains( "300" ), "INT-26: dropped row's total must be entirely absent; got:\n{s}" );

  let lines : Vec< &str > = s.lines().skip( 1 ).collect();
  for ( expected_rank, expected_total ) in [ ( "1", "900" ), ( "2", "700" ) ]
  {
    let line = lines.iter().find( | l | l.contains( expected_total ) )
      .unwrap_or_else( || panic!( "INT-26: surviving row with total {expected_total} must exist; got:\n{s}" ) );
    let fields : Vec< &str > = line.split_whitespace().collect();
    assert_eq!( fields[ 0 ], expected_rank, "INT-26: surviving row with total {expected_total} must carry rank {expected_rank}, never a pre-limit index; got row:\n{line}" );
  }
}

/// INT-27: `columns::cache_write,cache_read` renders the two components
/// `cache` already sums, and their sum always equals `cache`.
///
/// ## Purpose
/// Validates the new `cache_write`/`cache_read` columns (`Fix(BUG-530)`):
/// they must read off `RollupRow.cache_creation`/`RollupRow.cache_read`
/// respectively (never swapped), and — since `RollupRow.cache()` is defined
/// as their sum — `cache_write + cache_read` must always equal `cache` for
/// the same row.
///
/// ## Coverage
/// One session with deliberately distinct, unambiguous
/// `cache_write`/`cache_read` values (`50`/`200`, summing to `250`).
///
/// ## Validation Strategy
/// One session; run `columns::cache,cache_write,cache_read`; split the single
/// data row on whitespace and assert each field exactly (`250`/`50`/`200`).
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-27
#[ test ]
fn int_27_columns_cache_write_cache_read_split_sums_to_cache()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "cachesplit" );
  std::fs::create_dir_all( &project ).unwrap();

  let mut fx = RollupSession::simple( project.to_str().unwrap() );
  fx.cache_write_tokens = 50;
  fx.cache_read_tokens = 200;
  write_rollup_session( &storage_root, &project, "cachspl1-1111-4abc-9def-000000000001", &fx );

  let out = common::clg_cmd()
    .current_dir( &project )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .arg( "columns::cache,cache_write,cache_read" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  let header = s.lines().next().unwrap_or_default();
  assert!( header.contains( "CacheW" ) && header.contains( "CacheR" ), "INT-27: CacheW/CacheR headers must be present; got:\n{header}" );

  let data_line = s.lines().nth( 1 ).unwrap_or_else( || panic!( "INT-27: data row must exist; got:\n{s}" ) );
  let fields : Vec< &str > = data_line.split_whitespace().collect();
  assert_eq!( fields.len(), 3, "INT-27: expected exactly 3 projected columns; got: {fields:?}" );
  assert_eq!( fields[ 0 ], "250", "INT-27: Cache must be cache_write+cache_read=50+200=250; got row:\n{data_line}" );
  assert_eq!( fields[ 1 ], "50", "INT-27: CacheW must be the cache_write value, never swapped with CacheR; got row:\n{data_line}" );
  assert_eq!( fields[ 2 ], "200", "INT-27: CacheR must be the cache_read value, never swapped with CacheW; got row:\n{data_line}" );
}

/// INT-28: Default `columns::` excludes `Rank`/`CacheW`/`CacheR`.
///
/// ## Purpose
/// Mirrors INT-8 for the 3 columns added by `Fix(BUG-530)`: they are opt-in
/// only and must never appear unless explicitly requested via `columns::`.
///
/// ## Coverage
/// Header contains all 9 default labels (same set INT-8 already checks);
/// `Rank`/`CacheW`/`CacheR` are absent.
///
/// ## Validation Strategy
/// One session; run bare `.rollup`; assert the 9 default labels and the 3
/// newly-added opt-in labels are absent.
///
/// ## Related Requirements
/// `tests/docs/cli/command/14_rollup.md` — INT-28
#[ test ]
fn int_28_columns_default_excludes_rank_and_cache_split()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "defcols2" );
  std::fs::create_dir_all( &project ).unwrap();

  write_rollup_session(
    &storage_root, &project, "defc2aa1-1111-4abc-9def-000000000001",
    &RollupSession::simple( project.to_str().unwrap() ),
  );

  let out = common::clg_cmd()
    .current_dir( &project )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let header = common::stdout( &out ).lines().next().unwrap_or_default().to_string();
  // `Fix(BUG-544)`: default group label is `Session`, followed by `Project`.
  assert!( header.starts_with( "Session " ), "INT-28: group column must be labelled Session; got:\n{header}" );
  for present in [ "Project", "Sessions", "Calls", "Input", "Output", "Cache", "MaxCtx", "Total", "Pct" ]
  {
    assert!( header.contains( present ), "INT-28: default column {present} must appear; got:\n{header}" );
  }
  assert!( !header.contains( "Rank" ), "INT-28: Rank must be excluded by default (Fix(BUG-530)); got:\n{header}" );
  assert!( !header.contains( "CacheW" ), "INT-28: CacheW must be excluded by default (Fix(BUG-530)); got:\n{header}" );
  assert!( !header.contains( "CacheR" ), "INT-28: CacheR must be excluded by default (Fix(BUG-530)); got:\n{header}" );
}

/// BUG-528: cross-project `session_id` duplication inflates every grouping
/// dimension.
///
/// ## Root Cause
/// `collect_inputs()` (`src/cli/rollup.rs`) walks every scope-resolved
/// `Project` independently, pushing one `RollupInput` per physical session
/// file with no `session_id`-level deduplication across projects. When the
/// same `session_id` exists as a top-level file in more than one project
/// directory (the git-worktree-style pattern: a session's history forked
/// into a sibling working-tree directory and continued diverging from
/// there — confirmed occurring in real `~/.claude` data), each physical copy
/// becomes its own `RollupInput`, and `accumulate()` sums every one of them
/// into the same `GroupKey` bucket, inflating `sessions`/`calls`/every token
/// field by the duplicate-file count.
///
/// ## Why Not Caught
/// Every prior `.rollup` fixture used `session_ids` unique across the whole
/// test — none constructed the specific shape this bug requires (identical
/// `session_id` under two different `project_path`s). `scope::local` (the
/// CLI's own default) is structurally immune since one project directory
/// can never contain the same `session_id` twice, so casual single-project
/// testing never exercises this path at all.
///
/// ## Fix Applied
/// `collect_inputs()` now deduplicates by `session_id` across the entire
/// `projects` walk, keeping — for any `session_id` seen more than once —
/// the copy with the greatest `stats.total_entries` (the richer/more-
/// complete transcript) and discarding the rest, before any `RollupInput`
/// reaches `build_rollup()`.
///
/// ## Prevention
/// A wide-scope aggregation command needs at least one fixture with a
/// deliberately duplicated identity across its scope units — this test is
/// that fixture, pinned permanently as a regression guard.
///
/// ## Pitfall
/// An aggregation engine's own "distinct N" invariant (`RollupRow.sessions`'
/// doc comment) is a claim about its *caller's* input discipline, not
/// something the aggregation function itself can enforce — the fix belongs
/// at the boundary where physical duplication is actually knowable
/// (`collect_inputs()`), not inside `accumulate()`, which has no way to
/// detect it.
///
/// ## Coverage
/// Two projects, `proj-a`/`proj-b`, each holding a session file with the
/// **identical** `session_id` but different content (`proj-a`: 1 turn,
/// 100/50 input/output; `proj-b`: 3 turns, 300/150 input/output on the first
/// turn — the richer copy). Default `group::session scope::global` must
/// show exactly 1 row, `Sessions: 1` (not 2), and the kept totals must be
/// `proj-b`'s own numbers (300/150/450) — never the summed 400/200/600.
///
/// ## Validation Strategy
/// Write the same `session_id` under two distinct `project_path`s with
/// deliberately different `total_entries`; run bare `.rollup scope::global
/// columns::group,sessions,input,output,total`; split the single data row
/// on whitespace and assert each field exactly.
// test_kind: bug_reproducer(BUG-528)
#[ test ]
fn bug_528_cross_project_session_id_duplication_inflates_totals()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let proj_a = root.path().join( "proj-a" );
  let proj_b = root.path().join( "proj-b" );
  std::fs::create_dir_all( &proj_a ).unwrap();
  std::fs::create_dir_all( &proj_b ).unwrap();

  let dup_id = "dupeaaa1-1111-4abc-9def-000000000001";

  let mut fx_a = RollupSession::simple( proj_a.to_str().unwrap() );
  fx_a.turns = 1;
  fx_a.input_tokens = 100;
  fx_a.output_tokens = 50;

  let mut fx_b = RollupSession::simple( proj_b.to_str().unwrap() );
  fx_b.turns = 3;
  fx_b.input_tokens = 300;
  fx_b.output_tokens = 150;

  // Same session_id, two distinct project directories — the git-worktree-
  // style forked-history shape confirmed in real production data.
  write_rollup_session( &storage_root, &proj_a, dup_id, &fx_a );
  write_rollup_session( &storage_root, &proj_b, dup_id, &fx_b );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .arg( "scope::global" )
    .arg( "columns::group,sessions,input,output,total" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert_eq!( data_rows( &s ), 1, "BUG-528: one physical session duplicated across 2 projects must still be exactly 1 row; got:\n{s}" );

  let data_line = s.lines().find( | l | l.contains( "dupeaaa1" ) )
    .unwrap_or_else( || panic!( "BUG-528: data row must exist; got:\n{s}" ) );
  let fields : Vec< &str > = data_line.split_whitespace().collect();
  assert_eq!( fields.len(), 5, "BUG-528: expected exactly 5 projected columns; got: {fields:?}" );
  assert_eq!( fields[ 1 ], "1", "BUG-528: Sessions must be 1 (distinct session), not 2 (physical-file count); got row:\n{data_line}" );
  assert_eq!( fields[ 2 ], "300", "BUG-528: Input must be proj-b's own 300, never the summed 400; got row:\n{data_line}" );
  assert_eq!( fields[ 3 ], "150", "BUG-528: Output must be proj-b's own 150, never the summed 200; got row:\n{data_line}" );
  assert_eq!( fields[ 4 ], "450", "BUG-528: Total must be 300+150=450, never the summed 400+200=600; got row:\n{data_line}" );
}

/// BUG-544: the group column's header ignores `group::`, and session rows
/// carry no project attribution.
///
/// ## Root Cause
/// Two independent defects in `src/cli/rollup.rs`, both rooted in the same
/// omission — the renderer never consulted `group_by`:
///
/// 1. `column_header()` matched on `ColumnKey` alone and returned a constant
///    `"Group"` for `ColumnKey::Group`. The same column holds session ids,
///    project paths, model names or dates depending on `group::`, so every
///    non-default grouping printed an unlabelled dimension.
/// 2. `DEFAULT_COLUMNS` was a flat `const` with no project column at all.
///    Under the default `group::session` the group label is a bare 8-char
///    session id, which names no directory — so the default table could not
///    answer "which project is this row from?" even in `scope::global`, where
///    that is the first question a reader has.
///
/// ## Why Not Caught
/// Every prior header assertion (INT-7, INT-8, INT-22, INT-23, INT-28) tested
/// `header.contains( "Group" )` — a literal that was correct *because* the
/// header was hardcoded, so the tests pinned the defect in place rather than
/// exposing it. INT-22 even ran `group::model` and still asserted `"Group"`,
/// which is precisely the mislabelled case, and passed. No test compared the
/// header against the requested `group::` value, and none asserted project
/// attribution on a session row.
///
/// ## Fix Applied
/// `column_header()` and `render_header()` take `group_by`, mapping
/// `ColumnKey::Group` to `Session`/`Project`/`Model`/`Day`. `DEFAULT_COLUMNS`
/// became `default_columns( group_by )`, inserting the new
/// `ColumnKey::Project` after the group label only under `group::session`.
/// `Project` resolves through a `session_id -> project_label` map built in
/// `rollup_routine()` from the `RollupInput`s, captured before `build_rollup()`
/// aggregates them away.
///
/// ## Prevention
/// Assert a rendered header against the *parameter that determines it*, never
/// against a literal copied from current output — a hardcoded label and a
/// correctly-derived one are indistinguishable to `contains()` at the default
/// setting, and only diverge on the non-default paths tests rarely cover.
///
/// ## Pitfall
/// A column whose value is not a `RollupRow` field must be resolved before the
/// aggregation step that discards it. `RollupRow` carries only `group`, so by
/// render time the owning project of a session is unrecoverable — the map has
/// to be built in `rollup_routine()` while the `RollupInput`s are still in
/// hand. `ColumnKey::Rank` set the same precedent for CLI-synthesized columns.
///
/// ## Coverage
/// Two projects, `bug544-x`/`bug544-y`, one distinct session each. Part 1:
/// bare `.rollup scope::global` — each session row names its own project and
/// not its sibling's. Part 2: all four `group::` values — each labels the
/// group column with its own dimension, and none prints the literal `Group`.
///
/// ## Validation Strategy
/// One fixture, two runs. Part 1 locates each data row by its `short_id`
/// prefix and asserts the owning project path appears in it (and the sibling's
/// does not). Part 2 loops the four `group::` values, asserting the header
/// starts with the expected label and never contains `Group`.
// test_kind: bug_reproducer(BUG-544)
#[ test ]
fn bug_544_group_header_tracks_dimension_and_sessions_name_project()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let proj_x = root.path().join( "bug544-x" );
  let proj_y = root.path().join( "bug544-y" );
  std::fs::create_dir_all( &proj_x ).unwrap();
  std::fs::create_dir_all( &proj_y ).unwrap();

  write_rollup_session(
    &storage_root, &proj_x, "bug544xa-1111-4abc-9def-000000000001",
    &RollupSession::simple( proj_x.to_str().unwrap() ),
  );
  write_rollup_session(
    &storage_root, &proj_y, "bug544yb-2222-4abc-9def-000000000002",
    &RollupSession::simple( proj_y.to_str().unwrap() ),
  );

  // ── Part 1: session rows are attributable to a project ──────────────────
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".rollup" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert_eq!( data_rows( &s ), 2, "BUG-544: both sessions must render; got:\n{s}" );

  for ( id_prefix, own, sibling ) in
  [
    ( "bug544xa", "bug544-x", "bug544-y" ),
    ( "bug544yb", "bug544-y", "bug544-x" ),
  ]
  {
    let row = s.lines().find( | l | l.starts_with( id_prefix ) )
      .unwrap_or_else( || panic!( "BUG-544: row for {id_prefix} must exist; got:\n{s}" ) );
    assert!(
      row.contains( own ),
      "BUG-544: session row must name its owning project {own}; got row:\n{row}"
    );
    assert!(
      !row.contains( sibling ),
      "BUG-544: session row must not name the sibling project {sibling}; got row:\n{row}"
    );
  }

  // ── Part 2: the group column's header tracks `group::` ──────────────────
  for ( group, label ) in
  [
    ( "session", "Session" ), ( "project", "Project" ),
    ( "model", "Model" ),     ( "day", "Day" ),
  ]
  {
    let out = common::clg_cmd()
      .env( "HOME", root.path().to_str().unwrap() )
      .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
      .arg( ".rollup" )
      .arg( "scope::global" )
      .arg( format!( "group::{group}" ) )
      .output()
      .unwrap();

    common::assert_exit( &out, 0 );
    let header = common::stdout( &out ).lines().next().unwrap_or_default().to_string();
    assert!(
      header.starts_with( &format!( "{label} " ) ),
      "BUG-544: group::{group} must label the group column {label}; got header:\n{header}"
    );
    assert!(
      !header.contains( "Group" ),
      "BUG-544: the constant 'Group' header must never appear; got header:\n{header}"
    );
  }
}
