//! Edge case tests for `--pid` parameter of `clr ps`.
//!
//! Test spec: [`tests/docs/cli/param/068_pid.md`](docs/cli/param/068_pid.md).
//!
//! # Test Case Index
//!
//! | ID   | Name                                                                             | Category     |
//! |------|----------------------------------------------------------------------------------|--------------|
//! | EC-1 | `--pid <A>` shows only session A; session B absent                               | Behavioral   |
//! | EC-2 | `--pid <A>,<B>` shows sessions A and B; others absent                            | Behavioral   |
//! | EC-3 | `--pid` with non-existent PID → empty state, exit 0                              | Behavioral   |
//! | EC-4 | `--pid abc` (non-numeric) → exit 1                                               | Validation   |
//! | EC-5 | `--pid <A> --mode interactive` AND semantics: only matching session shown         | Interaction  |
//! | EC-6 | `--pid <A> --inspect` → inspect block for session A                              | Interaction  |
//! | EC-7 | `clr ps --help` output contains `--pid`                                          | Documentation|
//! | EC-8 | `CLR_PS_PID=<A>` env var filters active table to session A                       | Env Var      |

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, stderr_str, stdout_str };

#[ cfg( unix ) ]
use cli_binary_test_helpers::{ fake_claude_binary_dir, make_proc_dir, spawn_fake_claude };

// ── EC-1: Single PID filter ────────────────────────────────────────────────

/// EC-1: `clr ps --pid <A>` shows session A; session B absent.
#[ cfg( unix ) ]
#[ test ]
fn ec1_single_pid_shows_only_matching()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg_a = spawn_fake_claude( &path_val );
  let pid_a    = bg_a.id();
  let mut bg_b = spawn_fake_claude( &path_val );
  let pid_b    = bg_b.id();
  let proc     = make_proc_dir( &[ pid_a, pid_b ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--pid", &pid_a.to_string() ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps --pid <A>" );

  let _ = bg_a.kill(); let _ = bg_a.wait();
  let _ = bg_b.kill(); let _ = bg_b.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-1: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_a.to_string() ),
    "EC-1: PID A {pid_a} must appear. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &pid_b.to_string() ),
    "EC-1: PID B {pid_b} must NOT appear. Got:\n{stdout}"
  );
}

// ── EC-2: Multi-PID filter ─────────────────────────────────────────────────

/// EC-2: `clr ps --pid <A>,<B>` shows both A and B; other sessions absent.
#[ cfg( unix ) ]
#[ test ]
fn ec2_multi_pid_shows_both_matching()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg_a = spawn_fake_claude( &path_val );
  let pid_a    = bg_a.id();
  let mut bg_b = spawn_fake_claude( &path_val );
  let pid_b    = bg_b.id();
  let mut bg_c = spawn_fake_claude( &path_val );
  let pid_c    = bg_c.id();
  let proc     = make_proc_dir( &[ pid_a, pid_b, pid_c ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let pids_arg = format!( "{pid_a},{pid_b}" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--pid", &pids_arg ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps --pid <A>,<B>" );

  let _ = bg_a.kill(); let _ = bg_a.wait();
  let _ = bg_b.kill(); let _ = bg_b.wait();
  let _ = bg_c.kill(); let _ = bg_c.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-2: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_a.to_string() ),
    "EC-2: PID A {pid_a} must appear. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( &pid_b.to_string() ),
    "EC-2: PID B {pid_b} must appear. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &pid_c.to_string() ),
    "EC-2: PID C {pid_c} must NOT appear. Got:\n{stdout}"
  );
}

// ── EC-3: Non-existent PID → empty state ──────────────────────────────────

/// EC-3: `--pid` with a PID that no process holds → empty state, exit 0.
///
/// PID 99999999 is chosen as a value highly unlikely to be in use.
#[ test ]
fn ec3_unknown_pid_empty_state()
{
  let out    = run_cli( &[ "ps", "--pid", "99999999" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-3: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "No active Claude Code sessions" ),
    "EC-3: empty-state message expected. Got:\n{stdout}"
  );
}

// ── EC-4: Non-numeric --pid → exit 1 ──────────────────────────────────────

/// EC-4: `clr ps --pid abc` exits 1 with an error message.
#[ test ]
fn ec4_non_numeric_pid_exits_1()
{
  let out    = run_cli( &[ "ps", "--pid", "abc" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "EC-4: exit 1 expected, got {:?}", out.status.code() );
  assert!(
    stderr.contains( "abc" ),
    "EC-4: stderr must mention the invalid value 'abc'. Got: {stderr}"
  );
}

// ── EC-5: --pid + --mode AND semantics ────────────────────────────────────

/// EC-5: `--pid <A> --mode interactive` shows session A only if it is interactive.
///
/// Session A is spawned as interactive; the combined filter (PID=A AND mode=interactive)
/// returns it.  Session B is a different PID — must not appear regardless.
#[ cfg( unix ) ]
#[ test ]
fn ec5_pid_and_mode_filter_and_semantics()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg_a = spawn_fake_claude( &path_val );
  let pid_a    = bg_a.id();
  let mut bg_b = spawn_fake_claude( &path_val );
  let pid_b    = bg_b.id();
  let proc     = make_proc_dir( &[ pid_a, pid_b ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--pid", &pid_a.to_string(), "--mode", "interactive" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps --pid <A> --mode interactive" );

  let _ = bg_a.kill(); let _ = bg_a.wait();
  let _ = bg_b.kill(); let _ = bg_b.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-5: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_a.to_string() ),
    "EC-5: PID A {pid_a} must appear (interactive + matching PID). Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &pid_b.to_string() ),
    "EC-5: PID B {pid_b} must NOT appear (different PID). Got:\n{stdout}"
  );
}

// ── EC-6: --pid + --inspect ────────────────────────────────────────────────

/// EC-6: `--pid <A> --inspect` produces a key:value inspect block for session A.
#[ cfg( unix ) ]
#[ test ]
fn ec6_pid_with_inspect_shows_inspect_block()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg_a = spawn_fake_claude( &path_val );
  let pid_a    = bg_a.id();
  let mut bg_b = spawn_fake_claude( &path_val );
  let pid_b    = bg_b.id();
  let proc     = make_proc_dir( &[ pid_a, pid_b ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--pid", &pid_a.to_string(), "--inspect" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps --pid <A> --inspect" );

  let _ = bg_a.kill(); let _ = bg_a.wait();
  let _ = bg_b.kill(); let _ = bg_b.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-6: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_a.to_string() ),
    "EC-6: PID A {pid_a} must appear in inspect block. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "pid:" ),
    "EC-6: inspect key 'pid:' must appear. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &pid_b.to_string() ),
    "EC-6: PID B {pid_b} must NOT appear. Got:\n{stdout}"
  );
}

// ── EC-7: Help contains --pid ──────────────────────────────────────────────

/// EC-7: `clr ps --help` stdout contains `--pid`.
#[ test ]
fn ec7_help_contains_pid()
{
  let out    = run_cli( &[ "ps", "--help" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-7: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "--pid" ),
    "EC-7: --help must document --pid. Got: {stdout}"
  );
}

// ── EC-8: CLR_PS_PID env var ───────────────────────────────────────────────

/// EC-8: `CLR_PS_PID=<A>` env var filters active table to session A only.
#[ cfg( unix ) ]
#[ test ]
fn ec8_clr_ps_pid_env_var_filters()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg_a = spawn_fake_claude( &path_val );
  let pid_a    = bg_a.id();
  let mut bg_b = spawn_fake_claude( &path_val );
  let pid_b    = bg_b.id();
  let proc     = make_proc_dir( &[ pid_a, pid_b ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", &path_val )
    .env( "CLR_PS_PID", pid_a.to_string() )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps with CLR_PS_PID=<A>" );

  let _ = bg_a.kill(); let _ = bg_a.wait();
  let _ = bg_b.kill(); let _ = bg_b.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-8: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_a.to_string() ),
    "EC-8: PID A {pid_a} must appear (CLR_PS_PID=A). Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &pid_b.to_string() ),
    "EC-8: PID B {pid_b} must NOT appear. Got:\n{stdout}"
  );
}
