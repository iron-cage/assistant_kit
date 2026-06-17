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
//! | US-7 | Active table caption: `Active Sessions` + `running` | AC-010  |
//! | US-8 | `clr ps --help` → exit 0, stdout contains help text  | AC-011  |
//! | US-9 | Active sessions ordered oldest-first (row #1 = longest elapsed) | AC-012 |

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env, stderr_str, stdout_str };

#[ cfg( unix ) ]
use cli_binary_test_helpers::{ fake_claude_binary_dir, run_clr_ps, spawn_fake_claude };

// ── US-1: No sessions ─────────────────────────────────────────────────────────

/// US-1 (AC-002): No `claude` processes → exit 0, "No active Claude Code sessions.".
///
/// Passes an empty temp dir as `CLR_PROC_DIR` so `find_claude_processes()` sees
/// no entries regardless of how many real Claude sessions are running on the host.
#[ test ]
fn us_01_no_sessions()
{
  let empty_proc = tempfile::TempDir::new().expect( "create empty proc dir" );
  let proc_dir   = empty_proc.path().to_str().expect( "proc dir UTF-8" );
  let out    = run_cli_with_env( &[ "ps" ], &[ ( "CLR_PROC_DIR", proc_dir ) ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
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
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "ps" ) );
}

// ── US-3: Typo guard ──────────────────────────────────────────────────────────

/// US-3 (AC-004): `clr p` → exit 1, stderr: "Did you mean".
#[ test ]
fn us_03_typo_guard()
{
  let out = run_cli( &[ "p" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "expected non-zero exit for typo input, got 0" );
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
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
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
/// Uses the test process's own PID so the `/proc/{pid}` liveness filter
/// passes — gate files with dead PIDs are filtered out (BUG-293).
#[ test ]
fn us_06_queued_clr_shows_queued_headers()
{
  let gate_dir      = tempfile::TempDir::new().expect( "create gate temp dir" );
  let gate_dir_path = gate_dir.path().to_str().expect( "gate dir UTF-8" );
  let live_pid      = std::process::id();
  let gate_file     = gate_dir.path().join( format!( "{live_pid}.json" ) );
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

// ── US-7: Active table caption ────────────────────────────────────────────────

/// US-7 (AC-010): when ≥1 session is active, `clr ps` prefixes the table with
/// a titled caption rule containing "Active Sessions · N running" above the
/// column headers.
///
/// The caption is emitted by `TableCaption::new("Active Sessions").field(format!("{} running", n))`.
/// This test confirms the caption text is present in the rendered output so that
/// AC-010 is machine-verifiable.
#[ cfg( unix ) ]
#[ test ]
fn us_07_active_table_caption()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let out = run_clr_ps( &path_val );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "Active Sessions" ),
    "US-7: active table caption must contain 'Active Sessions', got: {stdout}"
  );
  assert!(
    stdout.contains( "running" ),
    "US-7: active table caption must contain 'running' count suffix, got: {stdout}"
  );
}

// ── US-8: `clr ps --help` ─────────────────────────────────────────────────────

/// US-8 (AC-011): `clr ps --help` must exit 0 and print ps help text.
///
/// Before fix (BUG-294): `dispatch_ps()` had no help intercept — `--help` was
/// treated as an unknown argument and rejected with exit 1.
/// After fix: `print_ps_help()` is called, process exits 0 with help on stdout.
// test_kind: bug_reproducer(BUG-294)
#[ test ]
fn us_08_ps_help()
{
  let out    = run_cli( &[ "ps", "--help" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "US-8 (AC-011): exit 0 expected, got {:?}", out.status.code() );
  assert!(
    !stdout.is_empty(),
    "US-8 (AC-011): stdout must contain ps help text, got empty output"
  );
}

// ── US-9: Active sessions ordered oldest-first ──────────────────────────────

/// US-9 (AC-012): active session rows are ordered by session start time; the
/// row with the longest elapsed time appears at row `#1`.
///
/// Spawns two fake `claude` processes 1 second apart, then verifies the older
/// process (longer elapsed) appears in an earlier row than the newer one.
// Verifies: AC-012
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn us_09_active_sessions_ordered_oldest_first()
{
  let ( _bin_dir, path_val ) = fake_claude_binary_dir();

  let mut bg_a = std::process::Command::new( "claude" )
    .arg( "30" )
    .env( "PATH", &path_val )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude A" );
  let pid_a = bg_a.id();

  std::thread::sleep( core::time::Duration::from_secs( 1 ) );

  let mut bg_b = std::process::Command::new( "claude" )
    .arg( "30" )
    .env( "PATH", &path_val )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude B" );
  let pid_b = bg_b.id();

  std::thread::sleep( core::time::Duration::from_millis( 200 ) );

  let out = run_clr_ps( &path_val );

  let _ = bg_a.kill();
  let _ = bg_a.wait();
  let _ = bg_b.kill();
  let _ = bg_b.wait();

  let stdout = stdout_str( &out );
  assert!(
    out.status.success(),
    "US-9 (AC-012): exit 0 expected, got {:?}", out.status.code()
  );

  let older_pid = pid_a.to_string();
  let newer_pid = pid_b.to_string();
  let row_a = stdout.lines().position( |l| l.contains( &older_pid ) );
  let row_b = stdout.lines().position( |l| l.contains( &newer_pid ) );
  assert!(
    row_a.is_some() && row_b.is_some(),
    "US-9 (AC-012): both PIDs must appear in output. A={pid_a}, B={pid_b}\n{stdout}"
  );
  assert!(
    row_a.unwrap() < row_b.unwrap(),
    "US-9 (AC-012): oldest session (PID {pid_a}) must appear at earlier row than newest (PID {pid_b}).\n{stdout}"
  );
}
