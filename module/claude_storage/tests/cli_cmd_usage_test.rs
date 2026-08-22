//! Integration tests for the `.usage` command.
//!
//! ## Source
//!
//! - Command spec: `tests/docs/cli/command/13_usage.md`
//! - Param spec: `tests/docs/cli/param/26_depth.md` — carries the exhaustive
//!   `depth::` coverage that `.rollup`'s own smoke tests defer to
//!
//! ## Coverage
//!
//! INT-1 through INT-21 per `tests/docs/cli/command/13_usage.md` — scope
//! resolution (local/relevant/under/around/global), `path::` anchoring,
//! `depth::` boundaries, agent-session exclusion, `limit::` capping and
//! mtime ordering, column formatting (8-char short id, 35-char command
//! truncation, k/M token suffixes, s/m/h durations), the worked-example
//! byte-exact table, and exit codes.
//!
//! ## Test Case Index
//!
//! | ID | Test Name | Category |
//! |----|-----------|----------|
//! | INT-1 | `int_1_default_local_single_session` | Scope Behavior |
//! | INT-2 | `int_2_relevant_includes_ancestors` | Scope Behavior |
//! | INT-3 | `int_3_under_includes_descendants` | Scope Behavior |
//! | INT-4 | `int_4_around_includes_both_directions` | Scope Behavior |
//! | INT-5 | `int_5_global_ignores_path_and_depth` | Scope Behavior |
//! | INT-6 | `int_6_path_overrides_cwd_anchor` | Path Anchoring |
//! | INT-7 | `int_7_depth_caps_component_distance` | Depth Boundary |
//! | INT-8 | `int_8_depth_zero_is_unbounded` | Depth Boundary |
//! | INT-9 | `int_9_agent_sessions_excluded` | Agent Exclusion |
//! | INT-10 | `int_10_limit_caps_flat_result_set` | Limit & Ordering |
//! | INT-11 | `int_11_most_recent_mtime_first` | Limit & Ordering |
//! | INT-12 | `int_12_session_column_short_id` | Output Formatting |
//! | INT-13 | `int_13_command_truncated_at_35_chars` | Output Formatting |
//! | INT-14 | `int_14_token_columns_k_m_suffixes` | Output Formatting |
//! | INT-15 | `int_15_duration_band_boundaries` | Output Formatting |
//! | INT-16 | `int_16_worked_example_byte_exact` | Column Values |
//! | INT-17 | `int_17_empty_non_local_scope_exits_0` | Exit Codes |
//! | INT-18 | `int_18_local_without_project_exits_2` | Exit Codes |
//! | INT-19 | `int_19_invalid_scope_rejected` | Input Validation |
//! | INT-20 | `int_20_negative_depth_rejected` | Input Validation |
//! | INT-21 | `int_21_negative_limit_rejected` | Input Validation |

mod common;

use tempfile::TempDir;




/// Fully controlled session fixture: every value the `.usage` table renders.
struct UsageSession< 'a >
{
  cwd : &'a str,
  first_msg : &'a str,
  turns : usize,
  input_tokens : u64,
  output_tokens : u64,
  cache_tokens : u64,
  first_ts : &'a str,
  last_ts : &'a str,
}

impl< 'a > UsageSession< 'a >
{
  /// One-turn session with small token counts and a 45-second span.
  fn simple( cwd : &'a str ) -> Self
  {
    Self
    {
      cwd,
      first_msg : "hello work",
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
/// carrying `first_msg`/`first_ts`/`cwd`, then `turns` assistant entries with
/// all tokens on the first one and `last_ts` on the final one.
///
/// `first_msg` must be JSON-safe (no `"`, `\`, or control characters).
///
/// Returns the encoded project ID.
fn write_usage_session(
  storage_root : &std::path::Path,
  project_path : &std::path::Path,
  session_id   : &str,
  fx           : &UsageSession< '_ >,
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
    r#"{{"type":"user","uuid":"u-000","parentUuid":null,"timestamp":"{first_ts}","cwd":"{cwd}","sessionId":"{session_id}","version":"2.0.0","gitBranch":"master","userType":"human","isSidechain":false,"message":{{"role":"user","content":"{first_msg}"}}}}"#,
    first_ts = fx.first_ts,
    cwd = fx.cwd,
    first_msg = fx.first_msg,
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
      r#"{{"type":"assistant","uuid":"a-{i}","parentUuid":"u-000","timestamp":"{ts}","cwd":"{cwd}","sessionId":"{session_id}","version":"2.0.0","gitBranch":"master","userType":"external","isSidechain":false,"requestId":"req_{i}","message":{{"role":"assistant","model":"claude-test","id":"msg_{i}","content":[{{"type":"text","text":"ok"}}],"stop_reason":"end_turn","stop_sequence":null,"usage":{{"input_tokens":{input},"output_tokens":{output},"cache_read_input_tokens":{cache},"cache_creation_input_tokens":0}}}}}}"#,
      cwd = fx.cwd,
    )
    .expect( "write assistant entry" );
  }

  encoded
}

/// Set a session file's modification time to `now - secs` seconds.
fn set_mtime_secs_ago( path : &std::path::Path, secs : u64 )
{
  let t = std::time::SystemTime::now() - core::time::Duration::from_secs( secs );
  let f = std::fs::OpenOptions::new().write( true ).open( path )
    .expect( "open session file for mtime update" );
  f.set_times( std::fs::FileTimes::new().set_modified( t ) )
    .expect( "set session file mtime" );
}

/// Count of data rows in a `.usage` table (total lines minus the header).
fn data_rows( s : &str ) -> usize
{
  s.lines().count().saturating_sub( 1 )
}

/// INT-1: No args defaults to `scope::local`, single session in cwd's project.
///
/// ## Purpose
/// Validates the default scope: bare `.usage` shows only the cwd project's
/// sessions.
///
/// ## Coverage
/// Header row plus exactly one data row; an unrelated project's session never
/// appears.
///
/// ## Validation Strategy
/// One session in the cwd project, one in an unrelated project; run bare
/// `.usage` from the project directory; assert row set.
///
/// ## Related Requirements
/// `tests/docs/cli/command/13_usage.md` — INT-1
#[ test ]
fn int_1_default_local_single_session()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "work" );
  let elsewhere = root.path().join( "elsewhere" );
  std::fs::create_dir_all( &project ).unwrap();

  write_usage_session(
    &storage_root, &project, "localaa1",
    &UsageSession::simple( project.to_str().unwrap() ),
  );
  write_usage_session(
    &storage_root, &elsewhere, "otheraa2",
    &UsageSession::simple( elsewhere.to_str().unwrap() ),
  );

  let out = common::clg_cmd()
    .current_dir( &project )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".usage" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert!( s.lines().next().is_some_and( | l | l.starts_with( "Session" ) ),
    "INT-1: header row must lead the table; got:\n{s}" );
  assert!( s.contains( "localaa1" ), "INT-1: cwd project's session must appear; got:\n{s}" );
  assert!( !s.contains( "otheraa2" ), "INT-1: unrelated project must be excluded; got:\n{s}" );
  assert_eq!( data_rows( &s ), 1, "INT-1: exactly one data row expected; got:\n{s}" );
}

/// INT-2: `scope::relevant` includes ancestor project sessions.
///
/// ## Purpose
/// Validates the ancestor-chain walk: `relevant` gathers the anchor project
/// plus every ancestor project.
///
/// ## Coverage
/// All three of anchor, parent, and grandparent projects listed.
///
/// ## Validation Strategy
/// Projects at `a`, `a/b`, `a/b/c`; run from `a/b/c` with `scope::relevant`;
/// assert all three session IDs appear.
///
/// ## Related Requirements
/// `tests/docs/cli/command/13_usage.md` — INT-2
#[ test ]
fn int_2_relevant_includes_ancestors()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let deep = root.path().join( "a/b/c" );
  std::fs::create_dir_all( &deep ).unwrap();

  for ( rel, id ) in [ ( "a", "relaaaa1" ), ( "a/b", "relbbbb2" ), ( "a/b/c", "relcccc3" ) ]
  {
    let p = root.path().join( rel );
    write_usage_session( &storage_root, &p, id, &UsageSession::simple( p.to_str().unwrap() ) );
  }

  let out = common::clg_cmd()
    .current_dir( &deep )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".usage" )
    .arg( "scope::relevant" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  for id in [ "relaaaa1", "relbbbb2", "relcccc3" ]
  {
    assert!( s.contains( id ), "INT-2: ancestor-chain session {id} must appear; got:\n{s}" );
  }
}

/// INT-3: `scope::under` includes descendant project sessions.
///
/// ## Purpose
/// Validates the subtree walk: `under` gathers the anchor project plus every
/// project nested beneath it, and nothing unrelated.
///
/// ## Coverage
/// Anchor, child, and grandchild listed; an unrelated sibling tree excluded.
///
/// ## Validation Strategy
/// Projects at `a/b`, `a/b/c`, `a/b/c/d`, and `z`; run from `a/b` with
/// `scope::under`; assert inclusion and exclusion.
///
/// ## Related Requirements
/// `tests/docs/cli/command/13_usage.md` — INT-3
#[ test ]
fn int_3_under_includes_descendants()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let anchor = root.path().join( "a/b" );
  std::fs::create_dir_all( root.path().join( "a/b/c/d" ) ).unwrap();
  std::fs::create_dir_all( root.path().join( "z" ) ).unwrap();

  for ( rel, id ) in
  [
    ( "a/b", "undaaaa1" ), ( "a/b/c", "undbbbb2" ),
    ( "a/b/c/d", "undcccc3" ), ( "z", "zzzeeee4" ),
  ]
  {
    let p = root.path().join( rel );
    write_usage_session( &storage_root, &p, id, &UsageSession::simple( p.to_str().unwrap() ) );
  }

  let out = common::clg_cmd()
    .current_dir( &anchor )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".usage" )
    .arg( "scope::under" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  for id in [ "undaaaa1", "undbbbb2", "undcccc3" ]
  {
    assert!( s.contains( id ), "INT-3: subtree session {id} must appear; got:\n{s}" );
  }
  assert!( !s.contains( "zzzeeee4" ), "INT-3: unrelated tree must be excluded; got:\n{s}" );
}

/// INT-4: `scope::around` includes both ancestor and descendant sessions.
///
/// ## Purpose
/// Validates the union scope: `around` = `relevant` ∪ `under`, deduplicated.
///
/// ## Coverage
/// Ancestor, current, and descendant projects each contribute exactly one row.
///
/// ## Validation Strategy
/// Projects at `a`, `a/b`, `a/b/c`; run from `a/b` with `scope::around`;
/// assert all three appear and the row count shows no duplicate.
///
/// ## Related Requirements
/// `tests/docs/cli/command/13_usage.md` — INT-4
#[ test ]
fn int_4_around_includes_both_directions()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let anchor = root.path().join( "a/b" );
  std::fs::create_dir_all( root.path().join( "a/b/c" ) ).unwrap();

  for ( rel, id ) in [ ( "a", "aroaaaa1" ), ( "a/b", "arobbbb2" ), ( "a/b/c", "arocccc3" ) ]
  {
    let p = root.path().join( rel );
    write_usage_session( &storage_root, &p, id, &UsageSession::simple( p.to_str().unwrap() ) );
  }

  let out = common::clg_cmd()
    .current_dir( &anchor )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".usage" )
    .arg( "scope::around" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  for id in [ "aroaaaa1", "arobbbb2", "arocccc3" ]
  {
    assert!( s.contains( id ), "INT-4: neighborhood session {id} must appear; got:\n{s}" );
  }
  assert_eq!( data_rows( &s ), 3, "INT-4: union must deduplicate — 3 rows exactly; got:\n{s}" );
}

/// INT-5: `scope::global` returns all sessions regardless of `path::`/`depth::`.
///
/// ## Purpose
/// Validates that `global` ignores the anchor and depth cap entirely.
///
/// ## Coverage
/// Three unrelated projects all listed despite a narrow `path::`/`depth::1`.
///
/// ## Validation Strategy
/// Projects at `a/b`, `c/d`, `e/f`; run `scope::global path::<a/b> depth::1`;
/// assert all three appear.
///
/// ## Related Requirements
/// `tests/docs/cli/command/13_usage.md` — INT-5
#[ test ]
fn int_5_global_ignores_path_and_depth()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  for ( rel, id ) in [ ( "a/b", "glbaaaa1" ), ( "c/d", "glbbbbb2" ), ( "e/f", "glbcccc3" ) ]
  {
    let p = root.path().join( rel );
    write_usage_session( &storage_root, &p, id, &UsageSession::simple( p.to_str().unwrap() ) );
  }

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".usage" )
    .arg( "scope::global" )
    .arg( format!( "path::{}", root.path().join( "a/b" ).display() ) )
    .arg( "depth::1" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  for id in [ "glbaaaa1", "glbbbbb2", "glbcccc3" ]
  {
    assert!( s.contains( id ), "INT-5: global must list {id} despite path::/depth::; got:\n{s}" );
  }
}

/// INT-6: `path::` overrides cwd as scope anchor.
///
/// ## Purpose
/// Validates that `path::` replaces the current directory as the anchor —
/// cwd itself has no effect once `path::` is given.
///
/// ## Coverage
/// Only the `path::`-anchored project's session listed, from an unrelated cwd.
///
/// ## Validation Strategy
/// Projects at `a`, `a/b`, `a/b/c`; run from the storage root (no project)
/// with `scope::local path::<a/b/c>`; assert only `a/b/c` appears.
///
/// ## Related Requirements
/// `tests/docs/cli/command/13_usage.md` — INT-6
#[ test ]
fn int_6_path_overrides_cwd_anchor()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  std::fs::create_dir_all( root.path().join( "a/b/c" ) ).unwrap();

  for ( rel, id ) in [ ( "a", "anchaaa1" ), ( "a/b", "anchbbb2" ), ( "a/b/c", "anchccc3" ) ]
  {
    let p = root.path().join( rel );
    write_usage_session( &storage_root, &p, id, &UsageSession::simple( p.to_str().unwrap() ) );
  }

  let out = common::clg_cmd()
    .current_dir( root.path() )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".usage" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", root.path().join( "a/b/c" ).display() ) )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert!( s.contains( "anchccc3" ), "INT-6: path::-anchored project must appear; got:\n{s}" );
  assert!( !s.contains( "anchaaa1" ) && !s.contains( "anchbbb2" ),
    "INT-6: non-anchored projects must be excluded; got:\n{s}" );
  assert_eq!( data_rows( &s ), 1, "INT-6: exactly one data row expected; got:\n{s}" );
}

/// INT-7: `depth::` caps candidates beyond the component distance.
///
/// ## Purpose
/// Validates the depth boundary: candidates more than `depth::` path
/// components from the anchor are dropped.
///
/// ## Coverage
/// Distance 0 and 1 kept; distance 2 dropped under `depth::1`.
///
/// ## Validation Strategy
/// Projects at `a`, `a/b`, `a/b/c` with session cwds matching; run
/// `scope::under path::<a> depth::1`; assert the cut.
///
/// ## Related Requirements
/// `tests/docs/cli/command/13_usage.md` — INT-7
#[ test ]
fn int_7_depth_caps_component_distance()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  std::fs::create_dir_all( root.path().join( "a/b/c" ) ).unwrap();

  for ( rel, id ) in [ ( "a", "depaaaa1" ), ( "a/b", "depbbbb2" ), ( "a/b/c", "depcccc3" ) ]
  {
    let p = root.path().join( rel );
    write_usage_session( &storage_root, &p, id, &UsageSession::simple( p.to_str().unwrap() ) );
  }

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".usage" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", root.path().join( "a" ).display() ) )
    .arg( "depth::1" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert!( s.contains( "depaaaa1" ), "INT-7: distance-0 project must survive depth::1; got:\n{s}" );
  assert!( s.contains( "depbbbb2" ), "INT-7: distance-1 project must survive depth::1; got:\n{s}" );
  assert!( !s.contains( "depcccc3" ), "INT-7: distance-2 project must be dropped by depth::1; got:\n{s}" );
}

/// INT-8: `depth::0` is unbounded.
///
/// ## Purpose
/// Validates the zero sentinel: `depth::0` removes the component-distance cap
/// entirely.
///
/// ## Coverage
/// The distance-2 project dropped in INT-7 survives under `depth::0`.
///
/// ## Validation Strategy
/// Same fixture as INT-7; run with `depth::0`; assert all three appear.
///
/// ## Related Requirements
/// `tests/docs/cli/command/13_usage.md` — INT-8
#[ test ]
fn int_8_depth_zero_is_unbounded()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  std::fs::create_dir_all( root.path().join( "a/b/c" ) ).unwrap();

  for ( rel, id ) in [ ( "a", "unbaaaa1" ), ( "a/b", "unbbbbb2" ), ( "a/b/c", "unbcccc3" ) ]
  {
    let p = root.path().join( rel );
    write_usage_session( &storage_root, &p, id, &UsageSession::simple( p.to_str().unwrap() ) );
  }

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".usage" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", root.path().join( "a" ).display() ) )
    .arg( "depth::0" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  for id in [ "unbaaaa1", "unbbbbb2", "unbcccc3" ]
  {
    assert!( s.contains( id ), "INT-8: depth::0 must keep {id} (unbounded); got:\n{s}" );
  }
}

/// INT-9: Agent sessions excluded from every scope.
///
/// ## Purpose
/// Validates the main/agent distinction: `agent-*`-named sidecar sessions
/// never appear as their own rows.
///
/// ## Coverage
/// One main session row; zero rows for the agent sidecar.
///
/// ## Validation Strategy
/// One UUID-named main session plus one `agent-*` session in the same
/// project; run `scope::global`; assert exactly one data row.
///
/// ## Related Requirements
/// `tests/docs/cli/command/13_usage.md` — INT-9
#[ test ]
fn int_9_agent_sessions_excluded()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "agentproj" );

  write_usage_session(
    &storage_root, &project, "beefcafe-1234-4abc-9def-0123456789ab",
    &UsageSession::simple( project.to_str().unwrap() ),
  );
  write_usage_session(
    &storage_root, &project, "agent-deadbeef",
    &UsageSession::simple( project.to_str().unwrap() ),
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".usage" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert!( s.contains( "beefcafe" ), "INT-9: main session must appear; got:\n{s}" );
  assert!( !s.contains( "agent-deadbeef" ), "INT-9: agent session must never be a row; got:\n{s}" );
  assert_eq!( data_rows( &s ), 1, "INT-9: exactly one data row expected; got:\n{s}" );
}

/// INT-10: `limit::N` caps the flat result set.
///
/// ## Purpose
/// Validates that `limit::` is a flat cap across the whole result set —
/// most-recent-first — not a per-project cap.
///
/// ## Coverage
/// Exactly 2 rows from a 3-session/3-project scope; the oldest is the one
/// dropped.
///
/// ## Validation Strategy
/// Three projects with distinct session mtimes; run `scope::global limit::2`;
/// assert the two newest survive and the oldest is cut.
///
/// ## Related Requirements
/// `tests/docs/cli/command/13_usage.md` — INT-10
#[ test ]
fn int_10_limit_caps_flat_result_set()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  for ( rel, id, age ) in
  [
    ( "p1", "limaaa01", 100_u64 ),
    ( "p2", "limbbb02", 200 ),
    ( "p3", "limccc03", 300 ),
  ]
  {
    let p = root.path().join( rel );
    let enc = write_usage_session( &storage_root, &p, id, &UsageSession::simple( p.to_str().unwrap() ) );
    set_mtime_secs_ago(
      &storage_root.join( "projects" ).join( &enc ).join( format!( "{id}.jsonl" ) ),
      age,
    );
  }

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".usage" )
    .arg( "scope::global" )
    .arg( "limit::2" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert!( s.contains( "limaaa01" ), "INT-10: newest session must survive the cap; got:\n{s}" );
  assert!( s.contains( "limbbb02" ), "INT-10: second-newest must survive the cap; got:\n{s}" );
  assert!( !s.contains( "limccc03" ), "INT-10: oldest must be cut by limit::2; got:\n{s}" );
  assert_eq!( data_rows( &s ), 2, "INT-10: exactly 2 data rows expected; got:\n{s}" );
}

/// INT-11: Sessions ordered most-recent-first by mtime.
///
/// ## Purpose
/// Validates result ordering: rows sort by session-file mtime, newest first.
///
/// ## Coverage
/// The newer session's row precedes the older session's row in stdout.
///
/// ## Validation Strategy
/// Two sessions with explicitly set mtimes (10s vs 500s ago); compare byte
/// offsets of their IDs in stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/13_usage.md` — INT-11
#[ test ]
fn int_11_most_recent_mtime_first()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  for ( rel, id, age ) in [ ( "pn", "newer111", 10_u64 ), ( "po", "older222", 500 ) ]
  {
    let p = root.path().join( rel );
    let enc = write_usage_session( &storage_root, &p, id, &UsageSession::simple( p.to_str().unwrap() ) );
    set_mtime_secs_ago(
      &storage_root.join( "projects" ).join( &enc ).join( format!( "{id}.jsonl" ) ),
      age,
    );
  }

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".usage" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  let newer = s.find( "newer111" ).expect( "newer session row must exist" );
  let older = s.find( "older222" ).expect( "older session row must exist" );
  assert!( newer < older, "INT-11: newer mtime must sort first; got:\n{s}" );
}

/// INT-12: Session column shows 8-character short id.
///
/// ## Purpose
/// Validates ID display: full-UUID session IDs render as their first 8
/// characters, never the full UUID.
///
/// ## Coverage
/// Short form present; the longer UUID prefix absent.
///
/// ## Validation Strategy
/// A session with a known 36-char UUID; assert `bf61b676` appears and
/// `bf61b676-1234` does not.
///
/// ## Related Requirements
/// `tests/docs/cli/command/13_usage.md` — INT-12
#[ test ]
fn int_12_session_column_short_id()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "shortid" );

  write_usage_session(
    &storage_root, &project, "bf61b676-1234-4abc-9def-0123456789ab",
    &UsageSession::simple( project.to_str().unwrap() ),
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".usage" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert!( s.contains( "bf61b676" ), "INT-12: 8-char short id must appear; got:\n{s}" );
  assert!( !s.contains( "bf61b676-1234" ), "INT-12: full UUID must never appear; got:\n{s}" );
}

/// INT-13: Command column truncates at 35 chars with trailing ….
///
/// ## Purpose
/// Validates command truncation: text beyond 35 characters is cut and marked
/// with a trailing `…`.
///
/// ## Coverage
/// The 35-char prefix plus `…` present; the full 50-char text absent.
///
/// ## Validation Strategy
/// A session whose first user entry is a known 50-char string; assert the
/// truncated form and the absence of the full form.
///
/// ## Related Requirements
/// `tests/docs/cli/command/13_usage.md` — INT-13
#[ test ]
fn int_13_command_truncated_at_35_chars()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "trunc" );

  let fifty = "01234567890123456789012345678901234567890123456789";
  let mut fx = UsageSession::simple( project.to_str().unwrap() );
  fx.first_msg = fifty;
  write_usage_session( &storage_root, &project, "truncaa1", &fx );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".usage" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  let expected : String = fifty.chars().take( 35 ).collect();
  assert!(
    s.contains( &format!( "{expected}…" ) ),
    "INT-13: 35-char prefix plus … must appear; got:\n{s}"
  );
  assert!( !s.contains( fifty ), "INT-13: full 50-char text must not appear; got:\n{s}" );
}

/// INT-14: In/Out/Cache columns use k/M-suffix formatting.
///
/// ## Purpose
/// Validates all three numeric bands: bare integer below 1000, `N.Nk` in the
/// thousands, `N.NM` from a million up.
///
/// ## Coverage
/// In=500 → `500`, Out=44800 → `44.8k`, Cache=4800000 → `4.8M`.
///
/// ## Validation Strategy
/// One session with those exact totals; assert each rendered form and the
/// absence of raw 6-7 digit integers.
///
/// ## Related Requirements
/// `tests/docs/cli/command/13_usage.md` — INT-14
#[ test ]
fn int_14_token_columns_k_m_suffixes()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "bands" );

  let mut fx = UsageSession::simple( project.to_str().unwrap() );
  fx.input_tokens = 500;
  fx.output_tokens = 44_800;
  fx.cache_tokens = 4_800_000;
  write_usage_session( &storage_root, &project, "bandsaa1", &fx );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".usage" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert!( s.contains( "500" ), "INT-14: sub-1000 count must render bare; got:\n{s}" );
  assert!( s.contains( "44.8k" ), "INT-14: mid-range count must render as N.Nk; got:\n{s}" );
  assert!( s.contains( "4.8M" ), "INT-14: million-range count must render as N.NM; got:\n{s}" );
  assert!( !s.contains( "44800" ), "INT-14: raw mid-range integer must not appear; got:\n{s}" );
  assert!( !s.contains( "4800000" ), "INT-14: raw 7-digit integer must not appear; got:\n{s}" );
}

/// INT-15: Dur column formats seconds/minutes/hours boundaries.
///
/// ## Purpose
/// Validates all three duration bands: `Ns` below a minute, `NmNNs` below an
/// hour, `NhNNm` from an hour up.
///
/// ## Coverage
/// Spans 45s → `45s`, 324s → `5m24s`, 3661s → `1h01m`.
///
/// ## Validation Strategy
/// Three sessions with those exact first/last timestamp spans; assert each
/// rendered form.
///
/// ## Related Requirements
/// `tests/docs/cli/command/13_usage.md` — INT-15
#[ test ]
fn int_15_duration_band_boundaries()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "durs" );

  for ( id, last_ts ) in
  [
    ( "duraaaa1", "2025-06-01T10:00:45Z" ),
    ( "durbbbb2", "2025-06-01T10:05:24Z" ),
    ( "durcccc3", "2025-06-01T11:01:01Z" ),
  ]
  {
    let mut fx = UsageSession::simple( project.to_str().unwrap() );
    fx.last_ts = last_ts;
    write_usage_session( &storage_root, &project, id, &fx );
  }

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".usage" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert!( s.contains( "45s" ), "INT-15: sub-minute span must render as Ns; got:\n{s}" );
  assert!( s.contains( "5m24s" ), "INT-15: sub-hour span must render as NmNNs; got:\n{s}" );
  assert!( s.contains( "1h01m" ), "INT-15: hour-plus span must render as NhNNm; got:\n{s}" );
}

/// INT-16: Column values match `Session::stats()` aggregation exactly.
///
/// ## Purpose
/// Validates the whole pipeline against the doc's worked example: every
/// column of both example rows, byte-for-byte, including the header.
///
/// ## Coverage
/// Full-table equality — short id, `<command-name>` unwrapping to `/role`,
/// turns, k/M tokens, m/s duration, per-session `Dir` from entry cwd, and
/// mtime ordering — in one assertion.
///
/// ## Validation Strategy
/// Two sessions rebuilt to the doc's exact numbers (31/44.8k/105.8k/4.8M/
/// 5m24s and 35/55.2k/109.2k/5.0M/4m22s); assert stdout equals the doc's
/// 3-line table verbatim.
///
/// ## Related Requirements
/// `tests/docs/cli/command/13_usage.md` — INT-16
#[ test ]
fn int_16_worked_example_byte_exact()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "worked" );
  std::fs::create_dir_all( &project ).unwrap();

  let role_msg = "<command-name>/role</command-name><command-message>role</command-message>";

  let s1 = UsageSession
  {
    cwd : "/data/repos/yrd_review/2026_troy_venue_pipeline_dev/pr_101",
    first_msg : role_msg,
    turns : 31,
    input_tokens : 44_800,
    output_tokens : 105_800,
    cache_tokens : 4_800_000,
    first_ts : "2025-06-01T10:00:00Z",
    last_ts : "2025-06-01T10:05:24Z",
  };
  let s2 = UsageSession
  {
    cwd : "/data/repos/yrd_review/2026_troy_venue_pipeline_dev/pr_144",
    first_msg : role_msg,
    turns : 35,
    input_tokens : 55_200,
    output_tokens : 109_200,
    cache_tokens : 5_000_000,
    first_ts : "2025-06-01T10:00:00Z",
    last_ts : "2025-06-01T10:04:22Z",
  };

  let enc1 = write_usage_session( &storage_root, &project, "bf61b676-1234-4abc-9def-0123456789ab", &s1 );
  let enc2 = write_usage_session( &storage_root, &project, "a2201ceb-1234-4abc-9def-0123456789ab", &s2 );
  assert_eq!( enc1, enc2, "both worked-example sessions share one project" );

  let sessions_dir = storage_root.join( "projects" ).join( &enc1 );
  set_mtime_secs_ago( &sessions_dir.join( "bf61b676-1234-4abc-9def-0123456789ab.jsonl" ), 60 );
  set_mtime_secs_ago( &sessions_dir.join( "a2201ceb-1234-4abc-9def-0123456789ab.jsonl" ), 3600 );

  let out = common::clg_cmd()
    .current_dir( &project )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".usage" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let expected = "\
Session   Command                            Turns      In     Out   Cache      Dur  Dir\n\
bf61b676  /role                                 31   44.8k  105.8k   4.8M    5m24s  /data/repos/yrd_review/2026_troy_venue_pipeline_dev/pr_101\n\
a2201ceb  /role                                 35   55.2k  109.2k   5.0M    4m22s  /data/repos/yrd_review/2026_troy_venue_pipeline_dev/pr_144\n";
  assert_eq!(
    common::stdout( &out ),
    expected,
    "INT-16: full table must match docs/cli/command/13_usage.md's worked example byte-for-byte"
  );
}

/// INT-17: No matching sessions in non-local scope exits 0 with empty table.
///
/// ## Purpose
/// Validates the empty-result contract: a zero-row result for a non-`local`
/// scope is a success, not an error.
///
/// ## Coverage
/// Exit 0; stdout is exactly the header row; stderr empty.
///
/// ## Validation Strategy
/// Empty storage, `scope::global`; assert exit 0 and header-only stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/13_usage.md` — INT-17
#[ test ]
fn int_17_empty_non_local_scope_exits_0()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  std::fs::create_dir_all( storage_root.join( "projects" ) ).unwrap();

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".usage" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert_eq!(
    s,
    "Session   Command                            Turns      In     Out   Cache      Dur  Dir\n",
    "INT-17: zero-row result must print exactly the header row"
  );
  assert!( common::stderr( &out ).is_empty(), "INT-17: no error output expected; got: {}", common::stderr( &out ) );
}

/// INT-18: `scope::local` with no project at cwd exits 2.
///
/// ## Purpose
/// Validates the local-scope storage error: no project for the current
/// directory is a usage error (exit 2), matching `.tail`/`.status`.
///
/// ## Coverage
/// Exit 2; stderr names the missing current-directory project.
///
/// ## Validation Strategy
/// Valid empty storage, cwd with no project, bare `.usage`; assert exit 2
/// and the stderr message.
///
/// ## Related Requirements
/// `tests/docs/cli/command/13_usage.md` — INT-18
#[ test ]
fn int_18_local_without_project_exits_2()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  std::fs::create_dir_all( storage_root.join( "projects" ) ).unwrap();

  let out = common::clg_cmd()
    .current_dir( root.path() )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".usage" )
    .output()
    .unwrap();

  common::assert_exit( &out, 2 );
  assert!(
    common::stderr( &out ).contains( "No project found for current directory" ),
    "INT-18: stderr must name the missing cwd project; got: {}",
    common::stderr( &out )
  );
}

/// INT-19: Invalid `scope::` value rejected.
///
/// ## Purpose
/// Validates scope validation: an unrecognized `scope::` value is an argument
/// error naming the bad value.
///
/// ## Coverage
/// Exit 1; stderr names `badvalue`; no table on stdout.
///
/// ## Validation Strategy
/// Run `.usage scope::badvalue`; assert exit, stderr content, empty stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/13_usage.md` — INT-19
#[ test ]
fn int_19_invalid_scope_rejected()
{
  let out = common::clg_cmd()
    .arg( ".usage" )
    .arg( "scope::badvalue" )
    .output()
    .unwrap();

  common::assert_exit( &out, 1 );
  assert!(
    common::stderr( &out ).contains( "badvalue" ),
    "INT-19: stderr must name the invalid value; got: {}",
    common::stderr( &out )
  );
  assert!( common::stdout( &out ).is_empty(), "INT-19: no table output expected; got:\n{}", common::stdout( &out ) );
}

/// INT-20: Negative `depth::` is rejected.
///
/// ## Purpose
/// Validates depth validation: a negative `depth::` is an argument error with
/// the exact documented message.
///
/// ## Coverage
/// Exit 1; stderr is exactly `depth must be non-negative`; no stdout table.
///
/// ## Validation Strategy
/// Run `.usage depth::-1`; assert exit, exact stderr, empty stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/13_usage.md` — INT-20
#[ test ]
fn int_20_negative_depth_rejected()
{
  let out = common::clg_cmd()
    .arg( ".usage" )
    .arg( "depth::-1" )
    .output()
    .unwrap();

  common::assert_exit( &out, 1 );
  assert_eq!(
    common::stderr( &out ).trim(),
    "depth must be non-negative",
    "INT-20: stderr must be exactly the documented message"
  );
  assert!( common::stdout( &out ).is_empty(), "INT-20: no table output expected; got:\n{}", common::stdout( &out ) );
}

/// INT-21: Negative `limit::` is rejected.
///
/// ## Purpose
/// Validates limit validation: a negative `limit::` is an argument error with
/// the exact documented message.
///
/// ## Coverage
/// Exit 1; stderr is exactly `limit must be non-negative`; no stdout table.
///
/// ## Validation Strategy
/// Run `.usage limit::-1`; assert exit, exact stderr, empty stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/13_usage.md` — INT-21
#[ test ]
fn int_21_negative_limit_rejected()
{
  let out = common::clg_cmd()
    .arg( ".usage" )
    .arg( "limit::-1" )
    .output()
    .unwrap();

  common::assert_exit( &out, 1 );
  assert_eq!(
    common::stderr( &out ).trim(),
    "limit must be non-negative",
    "INT-21: stderr must be exactly the documented message"
  );
  assert!( common::stdout( &out ).is_empty(), "INT-21: no table output expected; got:\n{}", common::stdout( &out ) );
}
