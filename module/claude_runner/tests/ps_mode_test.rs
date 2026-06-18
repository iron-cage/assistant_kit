//! Edge case tests for `--mode` parameter of `clr ps`.
//!
//! Test spec: [`tests/docs/cli/param/058_mode.md`](docs/cli/param/058_mode.md).
//!
//! # Test Case Index
//!
//! | ID   | Name                                                           | Category     |
//! |------|----------------------------------------------------------------|--------------|
//! | EC-1 | `--mode interactive` shows only interactive sessions           | Behavioral   |
//! | EC-2 | `--mode print` shows only print-mode sessions                  | Behavioral   |
//! | EC-3 | `--mode all` shows both session types                          | Behavioral   |
//! | EC-4 | `--mode bogus` exits 1 with valid-values list                  | Validation   |
//! | EC-5 | `CLR_PS_MODE=print` env var filters to print sessions          | Env Var      |
//! | EC-6 | CLI `--mode interactive` wins over `CLR_PS_MODE=print`         | CLI-wins     |
//! | EC-7 | Default (no --mode flag) shows all sessions                    | Default      |
//! | EC-8 | `clr ps --help` output contains `--mode`                       | Documentation|

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, stderr_str, stdout_str };

#[ cfg( unix ) ]
use cli_binary_test_helpers::{ fake_claude_binary_dir, spawn_fake_claude, spawn_print_claude };

// ── EC-1: `--mode interactive` shows only interactive sessions ────────────────

/// EC-1: `clr ps --mode interactive` shows only sessions without `--print` in cmdline.
#[ cfg( unix ) ]
#[ test ]
fn ec1_mode_interactive_shows_only_interactive()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();

  let mut bg_interactive = spawn_fake_claude( &path_val );
  let pid_interactive     = bg_interactive.id();

  let mut bg_print = spawn_print_claude( &path_val );
  let pid_print    = bg_print.id();

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--mode", "interactive" ] )
    .env( "PATH", &path_val )
    .output()
    .expect( "run clr ps --mode interactive" );

  let _ = bg_interactive.kill();
  let _ = bg_interactive.wait();
  let _ = bg_print.kill();
  let _ = bg_print.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-1: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_interactive.to_string() ),
    "EC-1: interactive PID {pid_interactive} must appear. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &pid_print.to_string() ),
    "EC-1: print-mode PID {pid_print} must NOT appear. Got:\n{stdout}"
  );
}

// ── EC-2: `--mode print` shows only print-mode sessions ──────────────────────

/// EC-2: `clr ps --mode print` shows only sessions whose cmdline contains `--print`.
#[ cfg( unix ) ]
#[ test ]
fn ec2_mode_print_shows_only_print()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();

  let mut bg_interactive = spawn_fake_claude( &path_val );
  let pid_interactive     = bg_interactive.id();

  let mut bg_print = spawn_print_claude( &path_val );
  let pid_print    = bg_print.id();

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--mode", "print" ] )
    .env( "PATH", &path_val )
    .output()
    .expect( "run clr ps --mode print" );

  let _ = bg_interactive.kill();
  let _ = bg_interactive.wait();
  let _ = bg_print.kill();
  let _ = bg_print.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-2: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_print.to_string() ),
    "EC-2: print PID {pid_print} must appear. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &pid_interactive.to_string() ),
    "EC-2: interactive PID {pid_interactive} must NOT appear. Got:\n{stdout}"
  );
}

// ── EC-3: `--mode all` shows both session types ───────────────────────────────

/// EC-3: `clr ps --mode all` shows both print-mode and interactive sessions.
#[ cfg( unix ) ]
#[ test ]
fn ec3_mode_all_shows_both()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();

  let mut bg_interactive = spawn_fake_claude( &path_val );
  let pid_interactive     = bg_interactive.id();

  let mut bg_print = spawn_print_claude( &path_val );
  let pid_print    = bg_print.id();

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--mode", "all" ] )
    .env( "PATH", &path_val )
    .output()
    .expect( "run clr ps --mode all" );

  let _ = bg_interactive.kill();
  let _ = bg_interactive.wait();
  let _ = bg_print.kill();
  let _ = bg_print.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-3: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_interactive.to_string() ),
    "EC-3: interactive PID {pid_interactive} must appear with --mode all. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( &pid_print.to_string() ),
    "EC-3: print PID {pid_print} must appear with --mode all. Got:\n{stdout}"
  );
}

// ── EC-4: `--mode bogus` → exit 1 ────────────────────────────────────────────

/// EC-4: `clr ps --mode bogus` exits 1 with stderr listing valid values.
#[ test ]
fn ec4_mode_bogus_exits_1()
{
  let out    = run_cli( &[ "ps", "--mode", "bogus" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "EC-4: exit 1 expected, got {:?}", out.status.code() );
  assert!(
    stderr.contains( "all" )
      && stderr.contains( "interactive" )
      && stderr.contains( "print" ),
    "EC-4: stderr must list all valid mode values (all, interactive, print). Got: {stderr}"
  );
}

// ── EC-5: `CLR_PS_MODE=print` env var fallback ───────────────────────────────

/// EC-5: `CLR_PS_MODE=print` env var applies print-mode filter.
#[ cfg( unix ) ]
#[ test ]
fn ec5_clr_ps_mode_env_var_filters()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();

  let mut bg_interactive = spawn_fake_claude( &path_val );
  let pid_interactive     = bg_interactive.id();

  let mut bg_print = spawn_print_claude( &path_val );
  let pid_print    = bg_print.id();

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", &path_val )
    .env( "CLR_PS_MODE", "print" )
    .output()
    .expect( "run clr ps with CLR_PS_MODE=print" );

  let _ = bg_interactive.kill();
  let _ = bg_interactive.wait();
  let _ = bg_print.kill();
  let _ = bg_print.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-5: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_print.to_string() ),
    "EC-5: print PID {pid_print} must appear with CLR_PS_MODE=print. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &pid_interactive.to_string() ),
    "EC-5: interactive PID {pid_interactive} must NOT appear. Got:\n{stdout}"
  );
}

// ── EC-6: CLI `--mode` wins over `CLR_PS_MODE` ───────────────────────────────

/// EC-6: CLI `--mode interactive` overrides env var `CLR_PS_MODE=print`.
#[ cfg( unix ) ]
#[ test ]
fn ec6_cli_mode_wins_over_env_var()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();

  let mut bg_interactive = spawn_fake_claude( &path_val );
  let pid_interactive     = bg_interactive.id();

  let mut bg_print = spawn_print_claude( &path_val );
  let pid_print    = bg_print.id();

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--mode", "interactive" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PS_MODE", "print" ) // CLI wins over this env var
    .output()
    .expect( "run clr ps --mode interactive with CLR_PS_MODE=print" );

  let _ = bg_interactive.kill();
  let _ = bg_interactive.wait();
  let _ = bg_print.kill();
  let _ = bg_print.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-6: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_interactive.to_string() ),
    "EC-6: interactive PID {pid_interactive} must appear (CLI --mode interactive wins). Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &pid_print.to_string() ),
    "EC-6: print PID {pid_print} must NOT appear (CLI wins). Got:\n{stdout}"
  );
}

// ── EC-7: Default shows all sessions ─────────────────────────────────────────

/// EC-7: With no `--mode` flag and no `CLR_PS_MODE` env var, all sessions appear.
#[ cfg( unix ) ]
#[ test ]
fn ec7_default_mode_shows_all()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();

  let mut bg_interactive = spawn_fake_claude( &path_val );
  let pid_interactive     = bg_interactive.id();

  let mut bg_print = spawn_print_claude( &path_val );
  let pid_print    = bg_print.id();

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", &path_val )
    .env_remove( "CLR_PS_MODE" ) // ensure env var is absent
    .output()
    .expect( "run clr ps (no mode filter)" );

  let _ = bg_interactive.kill();
  let _ = bg_interactive.wait();
  let _ = bg_print.kill();
  let _ = bg_print.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-7: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_interactive.to_string() ),
    "EC-7: interactive PID {pid_interactive} must appear with default mode. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( &pid_print.to_string() ),
    "EC-7: print PID {pid_print} must appear with default mode. Got:\n{stdout}"
  );
}

// ── EC-8: Help output contains `--mode` ──────────────────────────────────────

/// EC-8: `clr ps --help` stdout contains `--mode`.
#[ test ]
fn ec8_help_contains_mode()
{
  let out    = run_cli( &[ "ps", "--help" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-8: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "--mode" ),
    "EC-8: --help must document --mode. Got: {stdout}"
  );
}
