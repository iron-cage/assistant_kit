//! User-story-level integration tests for `clr ps` (Session Listing).
//!
//! Test spec: [`tests/docs/cli/user_story/26_session_listing.md`](docs/cli/user_story/26_session_listing.md).
//!
//! # Test Case Index
//!
//! | ID   | Name                                              | AC        |
//! |------|---------------------------------------------------|-----------|
//! | US-1 | No sessions: exit 0, no-sessions message          | AC-002    |
//! | US-2 | Help lists `ps` subcommand                        | AC-003    |
//! | US-3 | Typo `clr p` triggers guard                       | AC-004    |
//! | US-4 | Sessions present: plain-style table with headers   | AC-001,005|
//! | US-5 | `$PRO` prefix shortened in Absolute Path column   | AC-007    |
//! | US-6 | Queued CLR shown: PID, CWD, Waiting headers       | AC-008    |

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env, stderr_str, stdout_str };

#[ cfg( unix ) ]
use cli_binary_test_helpers::{ fake_claude_binary_dir, run_clr_ps, spawn_fake_claude };

// ── US-1: No sessions ─────────────────────────────────────────────────────────

/// US-1 (AC-002): No `claude` processes → exit 0, "No active Claude Code sessions.".
#[ test ]
fn us_01_no_sessions()
{
  let out = run_cli( &[ "ps" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success() );
  assert!( stdout.contains( "No active Claude Code sessions." ) );
  // Must NOT contain unicode box border when no sessions.
  assert!( !stdout.contains( '\u{250C}' ), "must not contain ┌ when 0 sessions" );
}

// ── US-2: Help lists ps ───────────────────────────────────────────────────────

/// US-2 (AC-003): `clr --help` includes the `ps` subcommand.
#[ test ]
fn us_02_help_lists_ps()
{
  let out = run_cli( &[ "--help" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success() );
  assert!( stdout.contains( "ps" ) );
}

// ── US-3: Typo guard ──────────────────────────────────────────────────────────

/// US-3 (AC-004): `clr p` → exit 1, stderr: "Did you mean".
#[ test ]
fn us_03_typo_guard()
{
  let out = run_cli( &[ "p" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success() );
  assert!( stderr.contains( "Did you mean" ) );
}

// ── US-4: Sessions present ────────────────────────────────────────────────────

/// US-4 (AC-001, AC-005): ≥1 fake `claude` process → exit 0, stdout uses plain
/// style (no `┌` border) and contains `PID`, `Elapsed`, `Absolute Path`, `Task`
/// column headers.
#[ cfg( unix ) ]
#[ test ]
fn us_04_sessions_plain_style_with_headers()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let out = run_clr_ps( &path_val );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!( !stdout.contains( '\u{250C}' ), "must not contain ┌ border (plain style): {stdout}" );
  assert!( stdout.contains( "PID" ), "missing PID header: {stdout}" );
  assert!( stdout.contains( "Elapsed" ), "missing Elapsed header: {stdout}" );
  assert!( stdout.contains( "Absolute Path" ), "missing Absolute Path header: {stdout}" );
  assert!( stdout.contains( "Task" ), "missing Task header: {stdout}" );
}

// ── US-5: $PRO prefix shortened ───────────────────────────────────────────────

/// US-5 (AC-007): when `PRO` env var is set, sessions whose CWD starts with
/// that prefix show `$PRO/…` in the Absolute Path column rather than the full path.
#[ cfg( unix ) ]
#[ test ]
fn us_05_pro_prefix_shortened_in_path()
{
  let pro_dir = tempfile::TempDir::new().expect( "create tmp PRO dir" );
  let sub_dir = pro_dir.path().join( "workspace" );
  std::fs::create_dir_all( &sub_dir ).expect( "create workspace subdir" );
  let pro_str = pro_dir.path().to_str().expect( "PRO path is UTF-8" );

  let ( _bin_dir, path_val ) = fake_claude_binary_dir();
  let mut bg = std::process::Command::new( "claude" )
    .env( "PATH", &path_val )
    .arg( "30" )
    .current_dir( &sub_dir )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude in workspace" );
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", &path_val )
    .env( "PRO", pro_str )
    .output()
    .expect( "run clr ps with PRO set" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success() );
  assert!(
    stdout.contains( "$PRO" ),
    "US-5: Absolute Path column must show $PRO/ prefix. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( pro_str ),
    "US-5: full PRO path must not appear in table. Got:\n{stdout}"
  );
}

// ── US-6: Queued CLR processes ────────────────────────────────────────────────

/// US-6 (AC-008): when a gate file exists in `CLR_GATE_DIR`, `clr ps` exits 0
/// and stdout contains the queued CLR processes table with PID, CWD, Waiting.
///
/// No live `clr` waiter needed — the test writes a gate state file directly.
#[ test ]
fn us_06_queued_clr_shows_queued_headers()
{
  let gate_dir      = tempfile::TempDir::new().expect( "create gate temp dir" );
  let gate_dir_path = gate_dir.path().to_str().expect( "gate dir UTF-8" );
  let gate_file     = gate_dir.path().join( "88888.json" );
  std::fs::write(
    &gate_file,
    r#"{"cwd":"/tmp/us6-project","since":1720000000,"attempt":1,"message":"waiting for session slot"}"#,
  ).expect( "write gate file" );

  let out    = run_cli_with_env( &[ "ps" ], &[ ( "CLR_GATE_DIR", gate_dir_path ) ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected" );
  assert!( stdout.contains( "PID" ), "queued table must show PID: {stdout}" );
  assert!( stdout.contains( "CWD" ), "queued table must show CWD: {stdout}" );
  assert!( stdout.contains( "Waiting" ), "queued table must show Waiting: {stdout}" );
}
