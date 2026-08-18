//! Integration tests for `clr ps` — the session listing command (extended).
//!
//! Extension of `ps_command_test.rs` (IT-1–IT-19) covering mode/columns filters,
//! caption breakdown text, live slot reservation files, and escaped-quote truncation.
//!
//! Test spec: [`tests/docs/cli/command/06_ps.md`](docs/cli/command/06_ps.md).
//!
//! # Test Case Index
//!
//! | ID   | Name                                        | Category         |
//! |------|---------------------------------------------|------------------|
//! | IT-20 | Active sessions ordered oldest-first (row #1 has longest elapsed) | BUG-301 repro   |
//! | IT-21 | `--mode print` shows only print-mode sessions                  | Mode filter      |
//! | IT-22 | `--mode interactive` shows only interactive sessions           | Mode filter      |
//! | IT-23 | `--mode bogus` → exit 1                                        | Mode validation  |
//! | IT-24 | `--columns pid,path,task` shows custom column subset           | Column select    |
//! | IT-25 | `--columns bogus` → exit 1                                     | Column validation|
//! | IT-26 | `--wide` shows all 11 columns                                  | Wide output      |
//! | IT-27 | `--wide --columns pid,task` → `--columns` wins                 | Precedence       |
//! | IT-28 | `CLR_PS_MODE=print` env var fallback filters print sessions    | Env var          |
//! | IT-29 | `CLR_PS_COLUMNS=pid,elapsed` env var fallback selects columns  | Env var          |
//! | IT-30 | `--mode all`, 1 interactive + 1 print → breakdown "1 interactive, 1 print" | Caption breakdown |
//! | IT-31 | `--mode all`, 3 interactive → breakdown "3 interactive, 0 print"          | Caption breakdown |
//! | IT-32 | `--mode all`, 3 print → breakdown "0 interactive, 3 print"                | Caption breakdown |
//! | IT-33 | `--mode interactive`, mixed set → plain caption, no breakdown             | Caption plain     |
//! | IT-34 | `--mode print`, mixed set → plain caption, no breakdown                   | Caption plain     |
//! | IT-35 | Live `slot_{n}.json` reservation file survives a `clr ps` scan             | BUG-387 follow-up |
//! | IT-36 | Task column: escaped `"` in Form A content is not truncated at the escaped quote | BUG-394 site 1 |
//! | IT-37 | Queued table CWD column: escaped `"` in gate-state `cwd` is not truncated at the escaped quote | BUG-394 site 2 |
//! | IT-38 | `--mode` combined with query-session filter → caption reflects filtered set   | Mode+query       |

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env, stderr_str, stdout_str };

#[ cfg( unix ) ]
use cli_binary_test_helpers::{
  fake_claude_binary_dir, make_proc_dir, run_clr_ps_proc,
  spawn_fake_claude, spawn_print_claude, spawn_query_claude,
};

// ── IT-20: active sessions sorted oldest-first (BUG-301) ────────────────────

/// IT-20 (BUG-301): `build_active_table()` sorts rows by `started_at` so the
/// oldest session appears at row `#1` with the longest elapsed time.
///
/// ## Root Cause
/// `build_active_table()` iterated `procs.iter().enumerate()` in `/proc` scan
/// order (PID-ascending) with no sort — PID order only approximates age order
/// and breaks on PID rollover.
///
/// ## Why Not Caught
/// IT-01–IT-19 checked row presence and content but never verified ordering.
///
/// ## Fix Applied
/// `sort_by_key()` using `read_process_metrics(p.pid).map(|m| m.started_at)`
/// inserted after the `procs.is_empty()` guard in `build_active_table()`.
///
/// ## Prevention
/// Always add an ordering assertion when implementing a "sorted by X" requirement.
///
/// ## Pitfall
/// PID-ascending order approximates age order on most Linux systems (monotonic
/// PID allocation), masking the bug until PID rollover.  Use a 1-second sleep
/// between spawns to guarantee distinct `started_at` values.
// test_kind: bug_reproducer(BUG-301)
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn it_20_active_sessions_sorted_by_age()
{
  let ( _bin_dir, path_val ) = fake_claude_binary_dir();

  // Spawn process A (oldest session).
  let mut bg_a = std::process::Command::new( "claude" )
    .arg( "30" )
    .env( "PATH", &path_val )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude A" );
  let pid_a = bg_a.id();

  // 1-second gap guarantees distinct started_at values in /proc/{pid}/stat.
  std::thread::sleep( core::time::Duration::from_secs( 1 ) );

  // Spawn process B (newer session).
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
    "IT-20: exit 0 expected, got {:?}", out.status.code()
  );

  // Oldest session (A) must appear before newest (B) in the table output.
  let older_pid = pid_a.to_string();
  let newer_pid = pid_b.to_string();
  let row_a = stdout.lines().position( |l| l.contains( &older_pid ) );
  let row_b = stdout.lines().position( |l| l.contains( &newer_pid ) );
  assert!(
    row_a.is_some() && row_b.is_some(),
    "IT-20 (BUG-301): both PIDs must appear in output. A={pid_a}, B={pid_b}\n{stdout}"
  );
  assert!(
    row_a.unwrap() < row_b.unwrap(),
    "IT-20 (BUG-301): oldest session (PID {pid_a}) must appear before newest (PID {pid_b}).\n{stdout}"
  );
}

// ── IT-21: `--mode print` shows only print-mode sessions ─────────────────────

/// IT-21: `clr ps --mode print` shows only sessions whose cmdline args include `--print`.
#[ cfg( unix ) ]
#[ test ]
fn it_21_mode_print_shows_only_print_sessions()
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
  assert!( out.status.success(), "IT-21: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_print.to_string() ),
    "IT-21: print-mode PID {pid_print} must appear with --mode print. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &pid_interactive.to_string() ),
    "IT-21: interactive PID {pid_interactive} must NOT appear with --mode print. Got:\n{stdout}"
  );
}

// ── IT-22: `--mode interactive` shows only interactive sessions ───────────────

/// IT-22: `clr ps --mode interactive` shows only sessions without `--print` in cmdline.
#[ cfg( unix ) ]
#[ test ]
fn it_22_mode_interactive_shows_only_interactive_sessions()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();

  let mut bg_interactive = spawn_fake_claude( &path_val );
  let pid_interactive     = bg_interactive.id();

  let mut bg_print = spawn_print_claude( &path_val );
  let pid_print    = bg_print.id();
  let proc         = make_proc_dir( &[ pid_interactive, pid_print ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--mode", "interactive" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps --mode interactive" );

  let _ = bg_interactive.kill();
  let _ = bg_interactive.wait();
  let _ = bg_print.kill();
  let _ = bg_print.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-22: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_interactive.to_string() ),
    "IT-22: interactive PID {pid_interactive} must appear with --mode interactive. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &pid_print.to_string() ),
    "IT-22: print-mode PID {pid_print} must NOT appear with --mode interactive. Got:\n{stdout}"
  );
}

// ── IT-23: `--mode bogus` → exit 1 ───────────────────────────────────────────

/// IT-23: `clr ps --mode bogus` exits 1 with stderr listing valid mode values.
#[ test ]
fn it_23_mode_bogus_exits_1()
{
  let out    = run_cli( &[ "ps", "--mode", "bogus" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "IT-23: exit 1 expected, got {:?}", out.status.code() );
  assert!(
    stderr.contains( "interactive" ) && stderr.contains( "print" ),
    "IT-23: stderr must list valid mode values (interactive, print). Got: {stderr}"
  );
}

// ── IT-24: `--columns pid,path,task` shows custom column subset ───────────────

/// IT-24: `clr ps --columns pid,path,task` shows PID, Absolute Path, Task
/// and does NOT show CPU%, RAM, State, Elapsed.
#[ cfg( unix ) ]
#[ test ]
fn it_24_columns_custom_subset()
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
  assert!( out.status.success(), "IT-24: exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "PID" ),           "IT-24: PID must be present: {stdout}" );
  assert!( stdout.contains( "Absolute Path" ), "IT-24: Absolute Path must be present: {stdout}" );
  assert!( stdout.contains( "Task" ),          "IT-24: Task must be present: {stdout}" );
  // Header-only check — legend "🐘 High RAM" would false-positive whole-stdout search.
  let header = stdout.lines().find( | l | l.contains( "PID" ) ).unwrap_or( "" );
  assert!( !header.contains( "CPU%" ),    "IT-24: CPU% must be absent from headers: {stdout}" );
  assert!( !header.contains( "RAM" ),     "IT-24: RAM must be absent from headers: {stdout}" );
  assert!( !header.contains( "Elapsed" ), "IT-24: Elapsed must be absent from headers: {stdout}" );
  assert!( !header.contains( "State" ),   "IT-24: State must be absent from headers: {stdout}" );
}

// ── IT-25: `--columns bogus` → exit 1 ────────────────────────────────────────

/// IT-25: `clr ps --columns bogus` exits 1 with stderr listing valid column keys.
#[ test ]
fn it_25_columns_bogus_exits_1()
{
  let out    = run_cli( &[ "ps", "--columns", "bogus" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "IT-25: exit 1 expected, got {:?}", out.status.code() );
  assert!(
    stderr.contains( "bogus" ) && ( stderr.contains( "pid" ) || stderr.contains( "idx" ) ),
    "IT-25: stderr must contain the unknown key and list valid keys. Got: {stderr}"
  );
}

// ── IT-26: `--wide` shows all 11 columns ─────────────────────────────────────

/// IT-26: `clr ps --wide` shows all 11 columns including Mode, Command, Binary.
#[ cfg( unix ) ]
#[ test ]
fn it_26_wide_shows_all_columns()
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
  assert!( out.status.success(), "IT-26: exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "Mode" ),    "IT-26: Mode header must be present: {stdout}" );
  assert!( stdout.contains( "Command" ), "IT-26: Command header must be present: {stdout}" );
  assert!( stdout.contains( "Binary" ),  "IT-26: Binary header must be present: {stdout}" );
}

// ── IT-27: `--wide --columns pid,task` → `--columns` wins ────────────────────

/// IT-27: When both `--wide` and `--columns` are given, `--columns` wins.
#[ cfg( unix ) ]
#[ test ]
fn it_27_columns_wins_over_wide()
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
  assert!( out.status.success(), "IT-27: exit 0 expected, got {:?}", out.status.code() );
  let header = stdout.lines().find( | l | l.contains( "PID" ) ).unwrap_or( "" );
  assert!( header.contains( "PID" ),  "IT-27: PID must be present in header: {stdout}" );
  assert!( header.contains( "Task" ), "IT-27: Task must be present in header: {stdout}" );
  assert!( !header.contains( "Mode" ),    "IT-27: Mode must be absent from header when --columns wins: {stdout}" );
  assert!( !header.contains( "Command" ), "IT-27: Command must be absent from header when --columns wins: {stdout}" );
  assert!( !header.contains( "Binary" ),  "IT-27: Binary must be absent from header when --columns wins: {stdout}" );
}

// ── IT-28: `CLR_PS_MODE=print` env var fallback ──────────────────────────────

/// IT-28: `CLR_PS_MODE=print` env var applies the print mode filter.
#[ cfg( unix ) ]
#[ test ]
fn it_28_clr_ps_mode_env_var()
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
  assert!( out.status.success(), "IT-28: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_print.to_string() ),
    "IT-28: print PID {pid_print} must appear with CLR_PS_MODE=print. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &pid_interactive.to_string() ),
    "IT-28: interactive PID {pid_interactive} must NOT appear with CLR_PS_MODE=print. Got:\n{stdout}"
  );
}

// ── IT-29: `CLR_PS_COLUMNS=pid,elapsed` env var fallback ─────────────────────

/// IT-29: `CLR_PS_COLUMNS=pid,elapsed` env var selects PID and Elapsed columns only.
///
/// `CLR_PROC_DIR` is set to a fake proc dir containing only the background process
/// so `find_claude_processes()` returns exactly one entry regardless of ambient sessions.
/// Pitfall: without `CLR_PROC_DIR`, ambient claude processes on the host cause
/// `clr ps` to find unexpected process counts, producing row/header mismatches that
/// panic in `RowBuilder::validate_row_length`.
#[ cfg( unix ) ]
#[ test ]
fn it_29_clr_ps_columns_env_var()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let fake_proc     = tempfile::TempDir::new().expect( "fake_proc" );
  let fake_proc_str = fake_proc.path().to_str().expect( "fake_proc UTF-8" );
  std::os::unix::fs::symlink(
    format!( "/proc/{}", bg.id() ),
    fake_proc.path().join( bg.id().to_string() ),
  ).expect( "pid symlink" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", &path_val )
    .env( "CLR_PS_COLUMNS", "pid,elapsed" )
    .env( "CLR_PROC_DIR", fake_proc_str )
    .output()
    .expect( "run clr ps with CLR_PS_COLUMNS=pid,elapsed" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-29: exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "PID" ),     "IT-29: PID must be present: {stdout}" );
  assert!( stdout.contains( "Elapsed" ), "IT-29: Elapsed must be present: {stdout}" );
  // Header-only check — legend "🐘 High RAM" would false-positive whole-stdout search.
  let header = stdout.lines().find( | l | l.contains( "PID" ) ).unwrap_or( "" );
  assert!( !header.contains( "CPU%" ),          "IT-29: CPU% must be absent from headers: {stdout}" );
  assert!( !header.contains( "RAM" ),           "IT-29: RAM must be absent from headers: {stdout}" );
  assert!( !header.contains( "Task" ),          "IT-29: Task must be absent from headers: {stdout}" );
  assert!( !header.contains( "Absolute Path" ), "IT-29: Absolute Path must be absent from headers: {stdout}" );
}

// ── Caption breakdown helper (AF1 anti-faking check) ─────────────────────────

/// Locate the `"Active Sessions"` caption line in `stdout`. Panics with the
/// full `stdout` on failure so assertion failures are self-diagnosing.
#[ cfg( unix ) ]
fn caption_line( stdout : &str ) -> &str
{
  stdout.lines().find( | l | l.contains( "Active Sessions" ) )
    .unwrap_or_else( || panic!( "no 'Active Sessions' caption line found in:\n{stdout}" ) )
}

/// Parse the leading `N` from `"N running..."` in the `Active Sessions` caption,
/// anchored on the full whitespace-delimited numeric token — never a raw
/// substring match, which would let e.g. `"11 running"` false-positive against
/// an assertion checking for `"1 running"`.
#[ cfg( unix ) ]
fn parse_running_count( stdout : &str ) -> usize
{
  let line = caption_line( stdout );
  let running_pos = line.find( " running" )
    .unwrap_or_else( || panic!( "caption missing ' running' suffix:\n{line}" ) );
  line[ ..running_pos ].rsplit( char::is_whitespace ).next()
    .unwrap_or_else( || panic!( "caption missing N before 'running':\n{line}" ) )
    .parse().unwrap_or_else( |e| panic!( "N not numeric ({e}):\n{line}" ) )
}

/// Parse `"Active Sessions · N running (I interactive, P print, Q query)"` from
/// `stdout`, returning `(N, I, P)` — the trailing `Q query` segment is present
/// in the rendered caption but intentionally not parsed here (callers only
/// assert the interactive/print split). Panics with the full `stdout` on any
/// parse failure so assertion failures are self-diagnosing.
#[ cfg( unix ) ]
fn parse_breakdown_counts( stdout : &str ) -> ( usize, usize, usize )
{
  let n = parse_running_count( stdout );
  let line = caption_line( stdout );
  let open  = line.find( '(' ).unwrap_or_else( || panic!( "caption missing '(' breakdown:\n{line}" ) );
  let close = line.find( ')' ).unwrap_or_else( || panic!( "caption missing ')' breakdown:\n{line}" ) );
  let mut parts = line[ open + 1 .. close ].split( ", " );
  let i : usize = parts.next().unwrap_or_else( || panic!( "missing interactive part:\n{line}" ) )
    .split_whitespace().next().unwrap_or_else( || panic!( "missing I number:\n{line}" ) )
    .parse().unwrap_or_else( |e| panic!( "I not numeric ({e}):\n{line}" ) );
  let p : usize = parts.next().unwrap_or_else( || panic!( "missing print part:\n{line}" ) )
    .split_whitespace().next().unwrap_or_else( || panic!( "missing P number:\n{line}" ) )
    .parse().unwrap_or_else( |e| panic!( "P not numeric ({e}):\n{line}" ) );
  ( n, i, p )
}

// ── IT-30 (T01): `--mode all`, 1 interactive + 1 print → breakdown ─────────────

/// IT-30 (T01): 2 active sessions (1 interactive + 1 print), `--mode all` (default)
/// → caption reads "2 running (1 interactive, 1 print, 0 query)". AF1: `I + P == N`
/// is checked by parsing the rendered caption, not just substring matching.
#[ cfg( unix ) ]
#[ test ]
fn it_30_mode_all_breakdown_mixed_interactive_and_print()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg_interactive = spawn_fake_claude( &path_val );
  let mut bg_print       = spawn_print_claude( &path_val );
  let proc = make_proc_dir( &[ bg_interactive.id(), bg_print.id() ] );

  let out = run_clr_ps_proc( &path_val, proc.path().to_str().expect( "proc dir UTF-8" ) );

  let _ = bg_interactive.kill();
  let _ = bg_interactive.wait();
  let _ = bg_print.kill();
  let _ = bg_print.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-30: exit 0 expected, got {:?}", out.status.code() );
  let ( n, i, p ) = parse_breakdown_counts( &stdout );
  assert_eq!( ( n, i, p ), ( 2, 1, 1 ), "IT-30: expected 2 running (1 interactive, 1 print). Got:\n{stdout}" );
  assert_eq!( i + p, n, "IT-30 (AF1): I + P must equal N. Got:\n{stdout}" );
}

// ── IT-31 (T02): `--mode all`, all interactive → breakdown ─────────────────────

/// IT-31 (T02): 3 active sessions, all interactive, `--mode all` (default) →
/// caption reads "3 running (3 interactive, 0 print, 0 query)".
#[ cfg( unix ) ]
#[ test ]
fn it_31_mode_all_breakdown_all_interactive()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg : Vec< std::process::Child > = ( 0..3 ).map( |_| spawn_fake_claude( &path_val ) ).collect();
  let pids : Vec< u32 > = bg.iter().map( std::process::Child::id ).collect();
  let proc = make_proc_dir( &pids );

  let out = run_clr_ps_proc( &path_val, proc.path().to_str().expect( "proc dir UTF-8" ) );

  for child in &mut bg { let _ = child.kill(); let _ = child.wait(); }

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-31: exit 0 expected, got {:?}", out.status.code() );
  let ( n, i, p ) = parse_breakdown_counts( &stdout );
  assert_eq!( ( n, i, p ), ( 3, 3, 0 ), "IT-31: expected 3 running (3 interactive, 0 print). Got:\n{stdout}" );
  assert_eq!( i + p, n, "IT-31 (AF1): I + P must equal N. Got:\n{stdout}" );
}

// ── IT-32 (T03): `--mode all`, all print → breakdown ────────────────────────────

/// IT-32 (T03): 3 active sessions, all print, `--mode all` (default) → caption
/// reads "3 running (0 interactive, 3 print, 0 query)".
#[ cfg( unix ) ]
#[ test ]
fn it_32_mode_all_breakdown_all_print()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg : Vec< std::process::Child > = ( 0..3 ).map( |_| spawn_print_claude( &path_val ) ).collect();
  let pids : Vec< u32 > = bg.iter().map( std::process::Child::id ).collect();
  let proc = make_proc_dir( &pids );

  let out = run_clr_ps_proc( &path_val, proc.path().to_str().expect( "proc dir UTF-8" ) );

  for child in &mut bg { let _ = child.kill(); let _ = child.wait(); }

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-32: exit 0 expected, got {:?}", out.status.code() );
  let ( n, i, p ) = parse_breakdown_counts( &stdout );
  assert_eq!( ( n, i, p ), ( 3, 0, 3 ), "IT-32: expected 3 running (0 interactive, 3 print). Got:\n{stdout}" );
  assert_eq!( i + p, n, "IT-32 (AF1): I + P must equal N. Got:\n{stdout}" );
}

// ── IT-33 (T04): `--mode interactive`, mixed set → plain caption ───────────────

/// IT-33 (T04): 2 active sessions (1 interactive + 1 print), `--mode interactive`
/// → only the interactive row is shown; caption reads plain "1 running" with no
/// breakdown parentheses.
#[ cfg( unix ) ]
#[ test ]
fn it_33_mode_interactive_caption_has_no_breakdown()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg_interactive = spawn_fake_claude( &path_val );
  let mut bg_print       = spawn_print_claude( &path_val );
  let proc = make_proc_dir( &[ bg_interactive.id(), bg_print.id() ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--mode", "interactive" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps --mode interactive" );

  let _ = bg_interactive.kill();
  let _ = bg_interactive.wait();
  let _ = bg_print.kill();
  let _ = bg_print.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-33: exit 0 expected, got {:?}", out.status.code() );
  assert_eq!( parse_running_count( &stdout ), 1, "IT-33: expected exactly 1 running. Got:\n{stdout}" );
  let caption = caption_line( &stdout );
  assert!( !caption.contains( '(' ), "IT-33: caption must NOT contain a breakdown. Got: {caption}" );
}

// ── IT-34 (T05): `--mode print`, mixed set → plain caption ──────────────────────

/// IT-34 (T05): 2 active sessions (1 interactive + 1 print), `--mode print` →
/// only the print row is shown; caption reads plain "1 running" with no
/// breakdown parentheses.
#[ cfg( unix ) ]
#[ test ]
fn it_34_mode_print_caption_has_no_breakdown()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg_interactive = spawn_fake_claude( &path_val );
  let mut bg_print       = spawn_print_claude( &path_val );
  let proc = make_proc_dir( &[ bg_interactive.id(), bg_print.id() ] );

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
  assert!( out.status.success(), "IT-34: exit 0 expected, got {:?}", out.status.code() );
  assert_eq!( parse_running_count( &stdout ), 1, "IT-34: expected exactly 1 running. Got:\n{stdout}" );
  let caption = caption_line( &stdout );
  assert!( !caption.contains( '(' ), "IT-34: caption must NOT contain a breakdown. Got: {caption}" );
}

// ── IT-38: `--mode query`, mixed set → filters to query row, plain caption ─────

/// IT-38 (task 418): 2 active sessions (1 print + 1 query), `--mode query` →
/// only the query row is shown (proves `classify_mode()`'s query branch is
/// actually reachable through `clr ps`'s mode filter, not just through
/// `clr query`'s own dispatch, which TSK-418's QT-2 already covers); caption
/// reads plain "1 running" with no breakdown parentheses, matching the
/// interactive/print precedent (IT-33/IT-34).
#[ cfg( unix ) ]
#[ test ]
fn it_38_mode_query_filter_and_caption()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg_print = spawn_print_claude( &path_val );
  let mut bg_query = spawn_query_claude( &path_val );
  let proc = make_proc_dir( &[ bg_print.id(), bg_query.id() ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--mode", "query" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps --mode query" );

  let _ = bg_print.kill();
  let _ = bg_print.wait();
  let _ = bg_query.kill();
  let _ = bg_query.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-38: exit 0 expected, got {:?}", out.status.code() );
  assert_eq!( parse_running_count( &stdout ), 1, "IT-38: expected exactly 1 running. Got:\n{stdout}" );
  assert!( stdout.contains( "query" ), "IT-38: Mode column must show 'query'. Got:\n{stdout}" );
  let caption = caption_line( &stdout );
  assert!( !caption.contains( '(' ), "IT-38: caption must NOT contain a breakdown. Got: {caption}" );
}

// ── IT-35: slot reservation file survives a `clr ps` scan (BUG-387 follow-up) ──

/// IT-35: a live BUG-387 slot reservation file (`slot_{n}.json`, written by
/// `gate_slot.rs::acquire_slot()`) must NOT be deleted by `clr ps`'s queued-table
/// scan. Regression guard for a gap found during BUG-387's own MAAV
/// validation: `build_queued_table()`'s liveness filter parses the gate
/// file's *filename* as a PID (the `{pid}.json` convention used by queued-
/// waiter telemetry) — for `slot_{n}.json` this always fails to parse, so
/// the file was wrongly self-healed away by every `clr ps` call regardless
/// of whether its recorded owner is still alive, silently reopening the
/// exact check-then-reserve race BUG-387 closed.
///
/// Linux-only: the liveness filter probes `/proc/{pid}` which does not exist
/// on Windows or macOS.
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn it_35_slot_reservation_file_not_deleted_by_ps_scan()
{
  let gate_dir      = tempfile::TempDir::new().expect( "create gate temp dir" );
  let gate_dir_path = gate_dir.path().to_str().expect( "gate dir UTF-8" );
  let live_pid      = std::process::id();
  let slot_file     = gate_dir.path().join( "slot_0.json" );
  std::fs::write( &slot_file, format!( r#"{{"pid":{live_pid},"since":1720000000}}"# ) )
    .expect( "write slot file" );
  let proc          = make_proc_dir( &[] );
  let proc_dir_path = proc.path().to_str().expect( "proc dir UTF-8" );

  let out    = run_cli_with_env( &[ "ps" ], &[ ( "CLR_GATE_DIR", gate_dir_path ), ( "CLR_PROC_DIR", proc_dir_path ) ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-35: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    slot_file.exists(),
    "IT-35 (BUG-387 follow-up): live slot reservation file must survive a `clr ps` scan. \
     build_queued_table() must not misparse slot_*.json as an unparseable/dead gate file. Got stdout:\n{stdout}"
  );
}

// ── IT-36: Task column — escaped quote in Form A content not truncated (BUG-394) ──

/// IT-36 (BUG-394 site 1): `try_jsonl_task()`'s Task column preview is not truncated at
/// an escaped `"` inside the human's Form A message text.
///
/// ## Root Cause
/// `try_jsonl_task()` used a bare `rest.find('"')` to locate the content value's closing
/// quote, stopping at the first escaped `\"` inside the message text instead of the true
/// terminator.
///
/// ## Why Not Caught
/// IT-16/IT-17's fixtures (`"fix the auth module"`, `"the actual task"`) contain no quote
/// character at all, so the naive `.find('"')` always happened to land on the true
/// terminator by coincidence.
///
/// ## Fix Applied
/// `try_jsonl_task()` now uses `find_unescaped_quote()` (escape-aware scan) in place of
/// the bare `rest.find('"')`.
///
/// ## Prevention
/// See `docs/invariant/014_json_string_extraction_escape_handling.md` IN-1.
///
/// ## Pitfall
/// Never assume user-authored message text cannot contain a literal `"` — the extracted
/// text is not unescaped, so the correctly-bounded result still contains literal
/// backslashes exactly as they appear in the on-disk JSONL text.
// test_kind: bug_reproducer(BUG-394)
#[ cfg( unix ) ]
#[ test ]
fn it_36_task_column_escaped_quote_not_truncated()
{
  let ( _bin_dir, path_val ) = fake_claude_binary_dir();

  let proj_tmp = tempfile::TempDir::new().expect( "create project tmp" );
  let cwd      = proj_tmp.path().join( "proj" );
  std::fs::create_dir_all( &cwd ).expect( "create CWD" );
  let mut bg = std::process::Command::new( "claude" )
    .arg( "30" )
    .env( "PATH", &path_val )
    .current_dir( &cwd )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude" );
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );
  let proc = make_proc_dir( &[ bg.id() ] );

  let encoded      = claude_storage_core::encode_path( &cwd ).expect( "encode cwd" );
  let home_tmp     = tempfile::TempDir::new().expect( "create temp HOME" );
  let project_path = home_tmp.path()
    .join( ".claude" ).join( "projects" ).join( &encoded );
  std::fs::create_dir_all( &project_path ).expect( "create project path" );
  std::fs::write(
    project_path.join( "session.jsonl" ),
    r#"{"type":"user","message":{"role":"user","content":"He said \"hi\" and left"}}"#,
  ).expect( "write JSONL" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", &path_val )
    .env( "HOME", home_tmp.path() )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( r#"He said \"hi\" and left"# ),
    "IT-36 (BUG-394): Task column must show the full content bounded at the true closing \
     quote, not truncated at the escaped quote 9 bytes in (`He said \\`). Got:\n{stdout}"
  );
}

// ── IT-37: Queued table CWD — escaped quote in gate-state cwd not truncated (BUG-394) ──

/// IT-37 (BUG-394 site 2): `parse_json_str()`'s "Queued CLR Processes" CWD column is not
/// truncated at an escaped `"` inside the gate-state file's `cwd` field.
///
/// ## Root Cause
/// `parse_json_str()` used a bare `rest.find('"')` to locate the `cwd` value's closing
/// quote, stopping at the first escaped `\"` instead of the true terminator — the
/// unpaired read side of a round-trip whose write side (`gate.rs::json_escape_str()`,
/// BUG-384) already escapes `cwd` correctly.
///
/// ## Why Not Caught
/// T07/T13 (BUG-384, `tests/concurrency_gate_test.rs`) assert only that the gate-state
/// file's on-disk JSON is well-formed on write; neither invokes `clr ps` to verify the
/// read side correctly reverses that escaping.
///
/// ## Fix Applied
/// `parse_json_str()` now uses `find_unescaped_quote()` (escape-aware scan) in place of
/// the bare `rest.find('"')`.
///
/// ## Prevention
/// See `docs/invariant/014_json_string_extraction_escape_handling.md` IN-2.
///
/// ## Pitfall
/// Fixing a JSON round-trip's write side does not imply the read side correctly reverses
/// it — each direction must be independently verified.
// test_kind: bug_reproducer(BUG-394)
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn it_37_queued_table_cwd_escaped_quote_not_truncated()
{
  let gate_dir      = tempfile::TempDir::new().expect( "create gate temp dir" );
  let gate_dir_path = gate_dir.path().to_str().expect( "gate dir UTF-8" );
  let live_pid      = std::process::id();
  let gate_file     = gate_dir.path().join( format!( "{live_pid}.json" ) );
  std::fs::write(
    &gate_file,
    r#"{"cwd":"/tmp/proj-\"quoted\"-dir","since":1720000000,"attempt":2,"message":"waiting for session slot"}"#,
  ).expect( "write gate file" );
  let proc          = make_proc_dir( &[] );
  let proc_dir_path = proc.path().to_str().expect( "proc dir UTF-8" );

  let out    = run_cli_with_env( &[ "ps" ], &[ ( "CLR_GATE_DIR", gate_dir_path ), ( "CLR_PROC_DIR", proc_dir_path ) ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( r#"/tmp/proj-\"quoted\"-dir"# ),
    "IT-37 (BUG-394): CWD column must show the full path bounded at the true closing \
     quote, not truncated at the escaped quote. Got:\n{stdout}"
  );
}
