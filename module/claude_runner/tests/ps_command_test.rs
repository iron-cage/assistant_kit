//! Integration tests for `clr ps` — the session listing command.
//!
//! Test spec: [`tests/docs/cli/command/06_ps.md`](docs/cli/command/06_ps.md).
//!
//! # Test Case Index
//!
//! | ID   | Name                                        | Category         |
//! |------|---------------------------------------------|------------------|
//! | IT-1 | 0 sessions → no-sessions message            | No-sessions      |
//! | IT-2  | ≥1 session → plain table (no `┌` border)      | Sessions present |
//! | IT-3  | `clr --help` lists `ps`                       | Help listing     |
//! | IT-4  | `clr p` (typo) → exit 1, Did you mean         | Typo guard       |
//! | IT-5  | table contains PID, Elapsed, Absolute Path, Task | Column presence |
//! | IT-6  | `clr pss` (typo) → exit 1, Did you mean       | Typo guard       |
//! | IT-7  | own PID not in `clr ps` output                | Self-exclusion   |
//! | IT-8  | `clr ps --unknown` → exit 1                   | Error handling   |
//! | IT-9  | `$PRO` prefix replaced by `"$PRO"` in path    | Path shortening  |
//! | IT-10 | Gate file present → queued table with headers  | Queued present   |
//! | IT-11 | No gate files → no queued table in output      | Queued absent    |

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env, stderr_str, stdout_str };

#[ cfg( unix ) ]
use cli_binary_test_helpers::{ fake_claude_binary_dir, run_clr_ps, spawn_fake_claude };

// ── IT-1: 0 sessions ──────────────────────────────────────────────────────────

/// IT-1: `clr ps` with 0 sessions → exit 0, no-sessions message.
///
/// In the test container no live `claude` processes are running; the scanner
/// returns an empty vec and the output must be the sentinel string.
#[ test ]
fn it_01_no_sessions_shows_message()
{
  let out = run_cli( &[ "ps" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit code must be 0, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "No active Claude Code sessions." ),
    "stdout must contain the no-sessions message, got: {stdout}"
  );
}

// ── IT-2: ≥1 session → plain-style table ──────────────────────────────────────

/// IT-2: with a fake `claude` process running, `clr ps` exits 0 and the
/// output uses plain style — no unicode box-drawing border character (`┌`).
#[ cfg( unix ) ]
#[ test ]
fn it_02_sessions_present_plain_style()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let out = run_clr_ps( &path_val );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit code must be 0, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "PID" ),
    "stdout must contain PID header, got: {stdout}"
  );
  assert!(
    !stdout.contains( '\u{250C}' ), // must NOT have ┌
    "stdout must use plain style — no ┌ border, got: {stdout}"
  );
}

// ── IT-3: help lists ps ───────────────────────────────────────────────────────

/// IT-3: `clr --help` lists the `ps` subcommand.
#[ test ]
fn it_03_help_lists_ps()
{
  let out = run_cli( &[ "--help" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success() );
  assert!(
    stdout.contains( "ps" ),
    "help output must mention ps, got: {stdout}"
  );
}

// ── IT-4: typo guard `clr p` ─────────────────────────────────────────────────

/// IT-4: `clr p` (truncation typo) → exit 1, stderr: "Did you mean 'ps'?"
#[ test ]
fn it_04_typo_clr_p()
{
  let out = run_cli( &[ "p" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "expected non-zero exit" );
  assert!(
    stderr.contains( "Did you mean" ),
    "stderr must contain 'Did you mean', got: {stderr}"
  );
}

// ── IT-5: table headers present ───────────────────────────────────────────────

/// IT-5: with a session running, `clr ps` output contains the expected
/// column headers: PID, Elapsed, Absolute Path, Task.
#[ cfg( unix ) ]
#[ test ]
fn it_05_table_headers_present()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let out = run_clr_ps( &path_val );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success() );
  assert!( stdout.contains( "PID" ), "missing PID header: {stdout}" );
  assert!( stdout.contains( "Elapsed" ), "missing Elapsed header: {stdout}" );
  assert!( stdout.contains( "Absolute Path" ), "missing Absolute Path header: {stdout}" );
  assert!( stdout.contains( "Task" ), "missing Task header: {stdout}" );
}

// ── IT-6: typo guard `clr pss` ───────────────────────────────────────────────

/// IT-6: `clr pss` (extension typo) → exit 1, stderr: "Did you mean 'ps'?"
#[ test ]
fn it_06_typo_clr_pss()
{
  let out = run_cli( &[ "pss" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "expected non-zero exit" );
  assert!(
    stderr.contains( "Did you mean" ),
    "stderr must contain 'Did you mean', got: {stderr}"
  );
}

// ── IT-7: self-exclusion ──────────────────────────────────────────────────────

/// IT-7: the `clr ps` process's own PID is not listed in the output.
///
/// `find_claude_processes()` excludes `std::process::id()` (the caller).
/// We verify end-to-end by spawning a fake `claude` (so the table is non-empty)
/// and then checking that the test-runner PID (which is NOT a `claude` binary)
/// does not appear in the table.
#[ cfg( unix ) ]
#[ test ]
fn it_07_self_exclusion()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let out = run_clr_ps( &path_val );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success() );
  // The test-runner PID is not a `claude` binary, so it must not appear in the table.
  let self_pid = std::process::id().to_string();
  assert!(
    !stdout.contains( &self_pid ),
    "test-runner PID {self_pid} must not appear in ps output: {stdout}"
  );
}

// ── IT-8: unknown flag ────────────────────────────────────────────────────────

/// IT-8: `clr ps --unknown` → exit 1 with an error message on stderr.
#[ test ]
fn it_08_unknown_flag()
{
  let out = run_cli( &[ "ps", "--unknown" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "expected non-zero exit" );
  assert!(
    stderr.contains( "unexpected argument" ),
    "stderr must mention unexpected argument, got: {stderr}"
  );
}

// ── IT-9: $PRO prefix shortened ───────────────────────────────────────────────

/// IT-9: when `PRO` env var is set and a session CWD starts with that prefix,
/// the Absolute Path column shows `$PRO/…` rather than the full path.
///
/// `shorten_path()` replaces the `$PRO` prefix with the literal `"$PRO"` string;
/// the user already knows what `$PRO` expands to, keeping rows compact.
#[ cfg( unix ) ]
#[ test ]
fn it_09_pro_prefix_shortened_in_path_column()
{
  let pro_dir = tempfile::TempDir::new().expect( "create tmp PRO dir" );
  let sub_dir = pro_dir.path().join( "my" ).join( "project" );
  std::fs::create_dir_all( &sub_dir ).expect( "create project subdir" );
  let pro_str = pro_dir.path().to_str().expect( "PRO path is UTF-8" );

  let ( _bin_dir, path_val ) = fake_claude_binary_dir();
  let mut bg = std::process::Command::new( "claude" )
    .env( "PATH", &path_val )
    .arg( "30" )
    .current_dir( &sub_dir )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude in sub_dir" );
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
    "IT-9: path must be shortened to $PRO/… when PRO env var is set. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( pro_str ),
    "IT-9: full PRO prefix must not appear in the table. Got:\n{stdout}"
  );
}

// ── IT-10: gate file present → queued table ───────────────────────────────────

/// IT-10: when a gate JSON file exists in `CLR_GATE_DIR`, `clr ps` exits 0
/// and stdout contains the queued table headers (PID, CWD, Waiting).
///
/// No live `clr` waiter is needed — the test writes the state file directly.
#[ test ]
fn it_10_gate_file_present_shows_queued_table()
{
  let gate_dir      = tempfile::TempDir::new().expect( "create gate temp dir" );
  let gate_dir_path = gate_dir.path().to_str().expect( "gate dir UTF-8" );
  let gate_file     = gate_dir.path().join( "99999.json" );
  std::fs::write(
    &gate_file,
    r#"{"cwd":"/tmp/test-project","since":1720000000,"attempt":3,"message":"waiting for session slot"}"#,
  ).expect( "write gate file" );

  let out    = run_cli_with_env( &[ "ps" ], &[ ( "CLR_GATE_DIR", gate_dir_path ) ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "PID" ), "missing PID header in queued table: {stdout}" );
  assert!( stdout.contains( "CWD" ), "missing CWD header in queued table: {stdout}" );
  assert!( stdout.contains( "Waiting" ), "missing Waiting header in queued table: {stdout}" );
}

// ── IT-11: no gate files → no queued table ────────────────────────────────────

/// IT-11: when `CLR_GATE_DIR` points to an empty temp dir, `clr ps` exits 0
/// and stdout does NOT contain queued table headers.
#[ test ]
fn it_11_no_gate_files_no_queued_table()
{
  let gate_dir      = tempfile::TempDir::new().expect( "create gate temp dir" );
  let gate_dir_path = gate_dir.path().to_str().expect( "gate dir UTF-8" );
  // Dir is intentionally empty — no .json files.

  let out    = run_cli_with_env( &[ "ps" ], &[ ( "CLR_GATE_DIR", gate_dir_path ) ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "No active Claude Code sessions." ),
    "must show no-sessions message with empty gate dir: {stdout}"
  );
  assert!(
    !stdout.contains( "Waiting" ),
    "must not contain Waiting header when no gate files: {stdout}"
  );
  assert!(
    !stdout.contains( "Attempt" ),
    "must not contain Attempt header when no gate files: {stdout}"
  );
}
