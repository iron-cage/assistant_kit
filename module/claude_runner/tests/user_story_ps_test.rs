//! User-story-level integration tests for `clr ps` (Session Listing).
//!
//! Test spec: [`tests/docs/cli/user_story/026_session_listing.md`](docs/cli/user_story/026_session_listing.md).
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
//! | US-7 | Active table caption: `Active Sessions` + interactive/print breakdown | AC-010  |
//! | US-8 | `clr ps --help` → exit 0, stdout contains help text  | AC-011  |
//! | US-9 | Active sessions ordered oldest-first (row #1 = longest elapsed) | AC-012 |
//! | US-10 | `--mode print` filters to print-mode sessions only | AC-013 |
//! | US-11 | `--mode bogus` exits 1 with error message | AC-014 |
//! | US-12 | `--columns pid,path,task` shows PID, Absolute Path, Task | AC-015 |
//! | US-13 | `--columns bogus` exits 1 with error listing valid keys | AC-016 |
//! | US-14 | `--wide` shows all 11 columns including Mode, Command, Binary | AC-017 |
//! | US-15 | `--wide --columns pid,task` → only PID and Task visible | AC-018 |
//! | US-16 | `CLR_PS_MODE=print` env var shows only print-mode sessions | AC-019 |
//! | US-17 | `CLR_PS_COLUMNS=pid,elapsed` env var shows PID and Elapsed only | AC-020 |

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env, stderr_str, stdout_str };

#[ cfg( unix ) ]
use cli_binary_test_helpers::{
  fake_claude_binary_dir, make_proc_dir, run_clr_ps_proc, spawn_fake_claude, spawn_print_claude,
};

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
/// style (no `┌` border in caption/header) and contains `PID`, `Elapsed`,
/// `Absolute Path`, `Task` column headers.
///
/// Only the first non-blank line is checked for `┌`: task-column data from real
/// host sessions may contain unicode box characters; the structural caption line
/// will never have `┌` in plain style.
///
/// `CLR_PROC_DIR` is set to a fake proc dir so `find_claude_processes()` sees only
/// the test's background process — ambient sessions do not reach
/// `RowBuilder::validate_row_length` and cannot cause a panic (exit 101).
#[ cfg( unix ) ]
#[ test ]
fn us_04_sessions_plain_style_with_headers()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );
  let proc   = make_proc_dir( &[ bg.id() ] );

  let out = run_clr_ps_proc( &path_val, proc.path().to_str().expect( "proc dir UTF-8" ) );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  let first_line = stdout.lines().find( |l| !l.trim().is_empty() ).unwrap_or( "" );
  assert!(
    !first_line.contains( '\u{250C}' ),
    "table caption must use plain style — no ┌ border, got first line: {first_line}"
  );
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
  let proc = make_proc_dir( &[ bg.id() ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", &path_val )
    .env( "PRO", pro_str )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
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
///
/// Linux-only: the liveness filter probes `/proc/{pid}`.
#[ cfg( target_os = "linux" ) ]
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
  let proc          = make_proc_dir( &[] );
  let proc_dir_path = proc.path().to_str().expect( "proc dir UTF-8" );

  let out    = run_cli_with_env( &[ "ps" ], &[ ( "CLR_GATE_DIR", gate_dir_path ), ( "CLR_PROC_DIR", proc_dir_path ) ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected" );
  assert!( stdout.contains( "PID" ), "queued table must show PID: {stdout}" );
  assert!( stdout.contains( "CWD" ), "queued table must show CWD: {stdout}" );
  assert!( stdout.contains( "Waiting" ), "queued table must show Waiting: {stdout}" );
}

// ── US-7: Active table caption ────────────────────────────────────────────────

/// US-7 (AC-010): when ≥1 session is active, `clr ps` prefixes the table with
/// a titled caption rule containing "Active Sessions · N running (I interactive,
/// P print, Q query)" above the column headers, under the default `--mode all`.
///
/// The heading is emitted by `Heading::new("Active Sessions").with_field(...)`.
/// This test confirms the caption text — including the interactive/print/query
/// breakdown — is present in the rendered output so that AC-010 is
/// machine-verifiable.
///
/// `CLR_PROC_DIR` isolates proc scanning to this test's process only.
#[ cfg( unix ) ]
#[ test ]
fn us_07_active_table_caption()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );
  let proc   = make_proc_dir( &[ bg.id() ] );

  let out = run_clr_ps_proc( &path_val, proc.path().to_str().expect( "proc dir UTF-8" ) );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  let caption = stdout.lines().find( | l | l.contains( "Active Sessions" ) )
    .unwrap_or_else( || panic!( "US-7: no 'Active Sessions' caption line found in:\n{stdout}" ) );

  // Anchored on the full whitespace-delimited numeric token, not a raw substring
  // match — otherwise e.g. N=21 would false-pass an assertion checking "1 running"
  // (since "21 running" contains "1 running" as a trailing substring).
  let running_pos = caption.find( " running" )
    .unwrap_or_else( || panic!( "US-7: caption missing ' running' suffix:\n{caption}" ) );
  let n : usize = caption[ ..running_pos ].rsplit( char::is_whitespace ).next()
    .unwrap_or_else( || panic!( "US-7: caption missing N before 'running':\n{caption}" ) )
    .parse().unwrap_or_else( |e| panic!( "US-7: N not numeric ({e}):\n{caption}" ) );
  assert_eq!( n, 1, "US-7: expected exactly 1 running. Got:\n{caption}" );
  assert!(
    caption.contains( "(1 interactive, 0 print, 0 query)" ),
    "US-7: active table caption must contain the interactive/print/query breakdown, got: {caption}"
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
  let proc = make_proc_dir( &[ pid_a, pid_b ] );

  let out = run_clr_ps_proc( &path_val, proc.path().to_str().expect( "proc dir UTF-8" ) );

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

// ── US-10: `--mode print` shows only print-mode sessions ─────────────────────

/// US-10 (AC-013): `clr ps --mode print` shows only print-mode sessions.
#[ cfg( unix ) ]
#[ test ]
fn us_10_mode_print_filters_sessions()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();

  let mut bg_interactive = spawn_fake_claude( &path_val );
  let pid_interactive     = bg_interactive.id();

  let mut bg_print = spawn_print_claude( &path_val );
  let pid_print    = bg_print.id();
  let proc         = make_proc_dir( &[ pid_interactive, pid_print ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--mode", "print" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps --mode print" );

  let _ = bg_interactive.kill();
  let _ = bg_interactive.wait();
  let _ = bg_print.kill();
  let _ = bg_print.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "US-10 (AC-013): exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_print.to_string() ),
    "US-10 (AC-013): print PID {pid_print} must appear. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &pid_interactive.to_string() ),
    "US-10 (AC-013): interactive PID {pid_interactive} must NOT appear. Got:\n{stdout}"
  );
}

// ── US-11: `--mode bogus` exits 1 ────────────────────────────────────────────

/// US-11 (AC-014): `clr ps --mode bogus` exits 1 with error message.
#[ test ]
fn us_11_mode_bogus_exits_1()
{
  let out    = run_cli( &[ "ps", "--mode", "bogus" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "US-11 (AC-014): exit 1 expected, got {:?}", out.status.code() );
  assert!(
    stderr.contains( "interactive" ) && stderr.contains( "print" ),
    "US-11 (AC-014): stderr must list valid mode values. Got: {stderr}"
  );
}

// ── US-12: `--columns pid,path,task` shows custom subset ─────────────────────

/// US-12 (AC-015): `clr ps --columns pid,path,task` shows PID, Absolute Path, Task.
#[ cfg( unix ) ]
#[ test ]
fn us_12_columns_custom_subset()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );
  let proc   = make_proc_dir( &[ bg.id() ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--columns", "pid,path,task" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps --columns" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "US-12 (AC-015): exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "PID" ),           "US-12: PID must be present: {stdout}" );
  assert!( stdout.contains( "Absolute Path" ), "US-12: Absolute Path must be present: {stdout}" );
  assert!( stdout.contains( "Task" ),          "US-12: Task must be present: {stdout}" );
  // Header-only check — legend "🐘 High RAM" would false-positive whole-stdout search.
  let header = stdout.lines().find( | l | l.contains( "PID" ) ).unwrap_or( "" );
  assert!( !header.contains( "CPU%" ), "US-12: CPU% must be absent from headers: {stdout}" );
  assert!( !header.contains( "RAM" ),  "US-12: RAM must be absent from headers: {stdout}" );
}

// ── US-13: `--columns bogus` exits 1 ─────────────────────────────────────────

/// US-13 (AC-016): `clr ps --columns bogus` exits 1 with error listing valid keys.
#[ test ]
fn us_13_columns_bogus_exits_1()
{
  let out    = run_cli( &[ "ps", "--columns", "bogus" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "US-13 (AC-016): exit 1 expected, got {:?}", out.status.code() );
  assert!(
    stderr.contains( "bogus" ) && ( stderr.contains( "pid" ) || stderr.contains( "idx" ) ),
    "US-13 (AC-016): stderr must list valid column keys. Got: {stderr}"
  );
}

// ── US-14: `--wide` shows all 11 columns ─────────────────────────────────────

/// US-14 (AC-017): `clr ps --wide` shows all 11 columns including Mode, Command, Binary.
#[ cfg( unix ) ]
#[ test ]
fn us_14_wide_shows_all_columns()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );
  let proc   = make_proc_dir( &[ bg.id() ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--wide" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps --wide" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "US-14 (AC-017): exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "Mode" ),    "US-14: Mode must be present: {stdout}" );
  assert!( stdout.contains( "Command" ), "US-14: Command must be present: {stdout}" );
  assert!( stdout.contains( "Binary" ),  "US-14: Binary must be present: {stdout}" );
}

// ── US-15: `--columns` overrides `--wide` ────────────────────────────────────

/// US-15 (AC-018): `clr ps --wide --columns pid,task` → only PID and Task visible.
#[ cfg( unix ) ]
#[ test ]
fn us_15_columns_overrides_wide()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );
  let proc   = make_proc_dir( &[ bg.id() ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--wide", "--columns", "pid,task" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps --wide --columns" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "US-15 (AC-018): exit 0 expected, got {:?}", out.status.code() );
  let header = stdout.lines().find( | l | l.contains( "PID" ) ).unwrap_or( "" );
  assert!( header.contains( "PID" ),  "US-15: PID must be present in header: {stdout}" );
  assert!( header.contains( "Task" ), "US-15: Task must be present in header: {stdout}" );
  assert!( !header.contains( "Mode" ),    "US-15: Mode must be absent from header: {stdout}" );
  assert!( !header.contains( "Command" ), "US-15: Command must be absent from header: {stdout}" );
  assert!( !header.contains( "Binary" ),  "US-15: Binary must be absent from header: {stdout}" );
}

// ── US-16: `CLR_PS_MODE` env var filters sessions ────────────────────────────

/// US-16 (AC-019): `CLR_PS_MODE=print` env var shows only print-mode sessions.
#[ cfg( unix ) ]
#[ test ]
fn us_16_clr_ps_mode_env_var()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();

  let mut bg_interactive = spawn_fake_claude( &path_val );
  let pid_interactive     = bg_interactive.id();

  let mut bg_print = spawn_print_claude( &path_val );
  let pid_print    = bg_print.id();
  let proc         = make_proc_dir( &[ pid_interactive, pid_print ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", &path_val )
    .env( "CLR_PS_MODE", "print" )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps with CLR_PS_MODE=print" );

  let _ = bg_interactive.kill();
  let _ = bg_interactive.wait();
  let _ = bg_print.kill();
  let _ = bg_print.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "US-16 (AC-019): exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_print.to_string() ),
    "US-16 (AC-019): print PID {pid_print} must appear. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &pid_interactive.to_string() ),
    "US-16 (AC-019): interactive PID {pid_interactive} must NOT appear. Got:\n{stdout}"
  );
}

// ── US-17: `CLR_PS_COLUMNS` env var selects columns ─────────────────────────

/// US-17 (AC-020): `CLR_PS_COLUMNS=pid,elapsed` shows PID and Elapsed only.
#[ cfg( unix ) ]
#[ test ]
fn us_17_clr_ps_columns_env_var()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );
  let proc   = make_proc_dir( &[ bg.id() ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", &path_val )
    .env( "CLR_PS_COLUMNS", "pid,elapsed" )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps with CLR_PS_COLUMNS=pid,elapsed" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "US-17 (AC-020): exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "PID" ),     "US-17: PID must be present: {stdout}" );
  assert!( stdout.contains( "Elapsed" ), "US-17: Elapsed must be present: {stdout}" );
  // Header-only check — legend "🐘 High RAM" would false-positive whole-stdout search.
  let header = stdout.lines().find( | l | l.contains( "PID" ) ).unwrap_or( "" );
  assert!( !header.contains( "CPU%" ),          "US-17: CPU% must be absent from headers: {stdout}" );
  assert!( !header.contains( "RAM" ),           "US-17: RAM must be absent from headers: {stdout}" );
  assert!( !header.contains( "Task" ),          "US-17: Task must be absent from headers: {stdout}" );
  assert!( !header.contains( "Absolute Path" ), "US-17: Absolute Path must be absent from headers: {stdout}" );
}
