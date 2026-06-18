//! Edge case tests for `--wide` / `-w` parameter of `clr ps`.
//!
//! Test spec: [`tests/docs/cli/param/060_wide.md`](docs/cli/param/060_wide.md).
//!
//! # Test Case Index
//!
//! | ID   | Name                                                              | Category      |
//! |------|-------------------------------------------------------------------|---------------|
//! | EC-1 | `clr ps --wide` shows all 11 columns including Mode, Command, Binary | Behavioral |
//! | EC-2 | `clr ps -w` short form shows Mode, Command, Binary                | Behavioral    |
//! | EC-3 | `clr ps --wide --columns pid,task` → `--columns` wins             | Precedence    |
//! | EC-4 | `clr ps` without `--wide` hides Mode, Command, Binary             | Default       |
//! | EC-5 | `clr ps --help` output contains `--wide` and `-w`                 | Documentation |

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, stdout_str };

#[ cfg( unix ) ]
use cli_binary_test_helpers::{ fake_claude_binary_dir, spawn_fake_claude };

// ── EC-1: `--wide` shows all 11 columns ──────────────────────────────────────

/// EC-1: `clr ps --wide` shows Mode, Command, Binary in addition to default columns.
#[ cfg( unix ) ]
#[ test ]
fn ec1_wide_shows_all_columns()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--wide" ] )
    .env( "PATH", &path_val )
    .output()
    .expect( "run clr ps --wide" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-1: exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "PID" ),           "EC-1: PID must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "Elapsed" ),       "EC-1: Elapsed must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "CPU%" ),          "EC-1: CPU% must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "RAM" ),           "EC-1: RAM must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "State" ),         "EC-1: State must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "Absolute Path" ), "EC-1: Absolute Path must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "Task" ),          "EC-1: Task must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "Mode" ),    "EC-1: Mode must appear with --wide. Got:\n{stdout}" );
  assert!( stdout.contains( "Command" ), "EC-1: Command must appear with --wide. Got:\n{stdout}" );
  assert!( stdout.contains( "Binary" ),  "EC-1: Binary must appear with --wide. Got:\n{stdout}" );
}

// ── EC-2: `-w` short form ────────────────────────────────────────────────────

/// EC-2: `clr ps -w` short form shows Mode, Command, Binary.
#[ cfg( unix ) ]
#[ test ]
fn ec2_short_form_w_shows_wide_columns()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "-w" ] )
    .env( "PATH", &path_val )
    .output()
    .expect( "run clr ps -w" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-2: exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "Mode" ),    "EC-2: Mode must appear with -w. Got:\n{stdout}" );
  assert!( stdout.contains( "Command" ), "EC-2: Command must appear with -w. Got:\n{stdout}" );
  assert!( stdout.contains( "Binary" ),  "EC-2: Binary must appear with -w. Got:\n{stdout}" );
}

// ── EC-3: `--columns` overrides `--wide` ─────────────────────────────────────

/// EC-3: `--wide --columns pid,task` → only PID and Task; Mode/Command/Binary absent.
#[ cfg( unix ) ]
#[ test ]
fn ec3_columns_overrides_wide()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--wide", "--columns", "pid,task" ] )
    .env( "PATH", &path_val )
    .output()
    .expect( "run clr ps --wide --columns pid,task" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-3: exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "PID" ),  "EC-3: PID must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "Task" ), "EC-3: Task must appear. Got:\n{stdout}" );
  assert!( !stdout.contains( "Mode" ),    "EC-3: Mode must NOT appear (--columns wins). Got:\n{stdout}" );
  assert!( !stdout.contains( "Command" ), "EC-3: Command must NOT appear. Got:\n{stdout}" );
  assert!( !stdout.contains( "Binary" ),  "EC-3: Binary must NOT appear. Got:\n{stdout}" );
}

// ── EC-4: Default hides optional columns ────────────────────────────────────

/// EC-4: `clr ps` without `--wide` does not show Mode, Command, Binary.
#[ cfg( unix ) ]
#[ test ]
fn ec4_default_hides_wide_columns()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", &path_val )
    .env_remove( "CLR_PS_COLUMNS" )
    .output()
    .expect( "run clr ps (no --wide)" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-4: exit 0 expected, got {:?}", out.status.code() );
  assert!( !stdout.contains( "Mode" ),    "EC-4: Mode must NOT appear without --wide. Got:\n{stdout}" );
  assert!( !stdout.contains( "Command" ), "EC-4: Command must NOT appear without --wide. Got:\n{stdout}" );
  assert!( !stdout.contains( "Binary" ),  "EC-4: Binary must NOT appear without --wide. Got:\n{stdout}" );
}

// ── EC-5: Help output contains `--wide` and `-w` ─────────────────────────────

/// EC-5: `clr ps --help` stdout contains `--wide` and `-w`.
#[ test ]
fn ec5_help_contains_wide()
{
  let out    = run_cli( &[ "ps", "--help" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-5: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "--wide" ),
    "EC-5: --help must document --wide. Got: {stdout}"
  );
  assert!(
    stdout.contains( "-w" ),
    "EC-5: --help must document -w short form. Got: {stdout}"
  );
}
