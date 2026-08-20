//! Integration tests for the `.rollup` command.
//!
//! ## Coverage
//!
//! INT-1 through INT-21 per `tests/docs/cli/command/14_rollup.md` — grouping
//! (session/project/model/day), sort/order wiring, column projection
//! (`columns::`), the `model::` filter's effect on per-row percentages,
//! `limit::`'s post-aggregation cap semantics, the worked-example byte-exact
//! table, and exit/validation codes.
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

mod common;

use tempfile::TempDir;

fn stdout( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

fn stderr( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stderr ).into_owned()
}

fn assert_exit( out : &std::process::Output, code : i32 )
{
  assert_eq!(
    out.status.code().unwrap_or( -1 ),
    code,
    "expected exit {code}, got {:?}; stderr: {}",
    out.status.code(),
    stderr( out )
  );
}

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
  cache_tokens : u64,
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
      cache_tokens : 0,
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
    let ( input, output, cache ) = if i == 0
    {
      ( fx.input_tokens, fx.output_tokens, fx.cache_tokens )
    }
    else
    {
      ( 0, 0, 0 )
    };
    let ts = if i + 1 == fx.turns { fx.last_ts } else { fx.first_ts };
    writeln!(
      file,
      r#"{{"type":"assistant","uuid":"a-{i}","parentUuid":"u-000","timestamp":"{ts}","cwd":"{cwd}","sessionId":"{session_id}","version":"2.0.0","gitBranch":"master","userType":"external","isSidechain":false,"requestId":"req_{i}","message":{{"role":"assistant","model":"{model}","id":"msg_{session_id}_{i}","content":[{{"type":"text","text":"ok"}}],"stop_reason":"end_turn","stop_sequence":null,"usage":{{"input_tokens":{input},"output_tokens":{output},"cache_read_input_tokens":{cache},"cache_creation_input_tokens":0}}}}}}"#,
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

  assert_exit( &out, 0 );
  let s = stdout( &out );
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

  assert_exit( &out, 0 );
  let s = stdout( &out );
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

  assert_exit( &out, 0 );
  let s = stdout( &out );
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

  assert_exit( &out, 0 );
  let s = stdout( &out );
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

  assert_exit( &out, 0 );
  let s = stdout( &out );
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

  assert_exit( &out, 0 );
  let s = stdout( &out );
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

  assert_exit( &out, 0 );
  let header = stdout( &out ).lines().next().unwrap_or_default().to_string();
  assert!( header.contains( "Group" ), "INT-7: Group column must appear; got:\n{header}" );
  assert!( header.contains( "Total" ), "INT-7: Total column must appear; got:\n{header}" );
  for absent in [ "Sessions", "Calls", "Input", "Output", "Cache", "MaxCtx", "Pct", "First", "Last" ]
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

  assert_exit( &out, 0 );
  let header = stdout( &out ).lines().next().unwrap_or_default().to_string();
  for present in [ "Group", "Sessions", "Calls", "Input", "Output", "Cache", "MaxCtx", "Total", "Pct" ]
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

  assert_exit( &out, 0 );
  let s = stdout( &out );
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

  assert_exit( &out, 0 );
  let s = stdout( &out );
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

  assert_exit( &out, 0 );
  let s = stdout( &out );
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

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "deptaaa1" ), "INT-12: distance-0 project must survive depth::1; got:\n{s}" );
  assert!( s.contains( "deptbbb2" ), "INT-12: distance-1 project must survive depth::1; got:\n{s}" );
  assert!( !s.contains( "deptccc3" ), "INT-12: distance-2 project must be dropped by depth::1; got:\n{s}" );
}

/// INT-13: Full table render matches the worked example byte-for-byte.
///
/// ## Purpose
/// Locks in the exact column widths, alignment, and formatting of the
/// default-column render.
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
    cache_tokens : 200,
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
    cache_tokens : 50,
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
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let expected = "\
Group                     Sessions   Calls     Input    Output     Cache    MaxCtx     Total     Pct\n\
aaaaaaaa                         1       4       500       300       200       700      1.0k   83.3%\n\
bbbbbbbb                         1       2       100        50        50       150       200   16.7%\n";
  assert_eq!(
    stdout( &out ),
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

  assert_exit( &out, 0 );
  assert_eq!(
    stdout( &out ),
    "Group                     Sessions   Calls     Input    Output     Cache    MaxCtx     Total     Pct\n",
    "INT-14: zero-row result must print exactly the header row"
  );
  assert!( stderr( &out ).is_empty(), "INT-14: no error output expected; got: {}", stderr( &out ) );
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

  assert_exit( &out, 2 );
  assert!(
    stderr( &out ).contains( "No project found for current directory" ),
    "INT-15: stderr must name the missing cwd project; got: {}",
    stderr( &out )
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

  assert_exit( &out, 1 );
  assert!( stderr( &out ).contains( "bogus" ), "INT-16: stderr must name the invalid value; got: {}", stderr( &out ) );
  assert!( stdout( &out ).is_empty(), "INT-16: no table output expected; got:\n{}", stdout( &out ) );
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

  assert_exit( &out, 1 );
  assert!( stderr( &out ).contains( "bogus" ), "INT-17: stderr must name the invalid value; got: {}", stderr( &out ) );
  assert!( stdout( &out ).is_empty(), "INT-17: no table output expected; got:\n{}", stdout( &out ) );
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

  assert_exit( &out, 1 );
  assert!( stderr( &out ).contains( "bogus" ), "INT-18: stderr must name the invalid value; got: {}", stderr( &out ) );
  assert!( stdout( &out ).is_empty(), "INT-18: no table output expected; got:\n{}", stdout( &out ) );
}

/// INT-19: Invalid `columns::` entry rejected.
///
/// ## Purpose
/// Validates `columns::` validation: an unrecognized column name is an
/// argument error naming the bad value, even alongside valid entries.
///
/// ## Coverage
/// Exit 1; stderr names `bogus`; no table on stdout.
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

  assert_exit( &out, 1 );
  assert!( stderr( &out ).contains( "bogus" ), "INT-19: stderr must name the invalid value; got: {}", stderr( &out ) );
  assert!( stdout( &out ).is_empty(), "INT-19: no table output expected; got:\n{}", stdout( &out ) );
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

  assert_exit( &out, 1 );
  assert_eq!( stderr( &out ).trim(), "depth must be non-negative", "INT-20: stderr must be exactly the documented message" );
  assert!( stdout( &out ).is_empty(), "INT-20: no table output expected; got:\n{}", stdout( &out ) );
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

  assert_exit( &out, 1 );
  assert_eq!( stderr( &out ).trim(), "limit must be non-negative", "INT-21: stderr must be exactly the documented message" );
  assert!( stdout( &out ).is_empty(), "INT-21: no table output expected; got:\n{}", stdout( &out ) );
}
