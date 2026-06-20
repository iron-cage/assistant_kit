//! Edge case tests for `--columns` parameter of `clr ps`.
//!
//! Test spec: [`tests/docs/cli/param/059_columns.md`](docs/cli/param/059_columns.md).
//!
//! # Test Case Index
//!
//! | ID   | Name                                                                          | Category      |
//! |------|-------------------------------------------------------------------------------|---------------|
//! | EC-1 | `--columns pid,path,task` shows exactly those 3 column headers                | Behavioral    |
//! | EC-2 | `--columns bogus` exits 1 with error listing valid keys                       | Validation    |
//! | EC-3 | `CLR_PS_COLUMNS=pid,elapsed` env var shows PID and Elapsed only               | Env Var       |
//! | EC-4 | CLI `--columns pid,path` wins over `CLR_PS_COLUMNS=pid,elapsed`               | CLI-wins      |
//! | EC-5 | `--columns pid,task` with `--wide` → `--columns` wins                        | Precedence    |
//! | EC-6 | `--columns idx,pid,mode,cmd,binary` shows optional columns                    | Behavioral    |
//! | EC-7 | Default (no `--columns`) shows the 9 default columns (including Mode)         | Default       |
//! | EC-8  | `clr ps --help` output contains `--columns`                                   | Documentation |
//! | EC-9  | `idx` counter is 1-based after `--mode` filtering                             | Interaction   |
//! | EC-10 | `clr ps --help` lists `idx`/`cmd`, not `num`/`command` (BUG-303 regression)   | Documentation |

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, stderr_str, stdout_str };

#[ cfg( unix ) ]
use cli_binary_test_helpers::{ fake_claude_binary_dir, spawn_fake_claude, spawn_print_claude };

// ── EC-1: Custom column subset ────────────────────────────────────────────────

/// EC-1: `clr ps --columns pid,path,task` shows PID, Absolute Path, Task;
/// hides CPU%, RAM, State, Elapsed.
#[ cfg( unix ) ]
#[ test ]
fn ec1_custom_columns_correct_headers()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--columns", "pid,path,task" ] )
    .env( "PATH", &path_val )
    .output()
    .expect( "run clr ps --columns pid,path,task" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-1: exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "PID" ),           "EC-1: PID header must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "Absolute Path" ), "EC-1: Absolute Path header must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "Task" ),          "EC-1: Task header must appear. Got:\n{stdout}" );
  assert!( !stdout.contains( "CPU%" ),     "EC-1: CPU% must NOT appear. Got:\n{stdout}" );
  assert!( !stdout.contains( "RAM" ),      "EC-1: RAM must NOT appear. Got:\n{stdout}" );
  assert!( !stdout.contains( "State" ),    "EC-1: State must NOT appear. Got:\n{stdout}" );
  assert!( !stdout.contains( "Elapsed" ),  "EC-1: Elapsed must NOT appear. Got:\n{stdout}" );
}

// ── EC-2: Unknown column key → exit 1 ────────────────────────────────────────

/// EC-2: `clr ps --columns bogus` exits 1 with stderr listing valid keys.
#[ test ]
fn ec2_unknown_column_exits_1()
{
  let out    = run_cli( &[ "ps", "--columns", "bogus" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "EC-2: exit 1 expected, got {:?}", out.status.code() );
  assert!(
    stderr.contains( "bogus" ),
    "EC-2: stderr must mention the unknown key. Got: {stderr}"
  );
  assert!(
    stderr.contains( "pid" ) && stderr.contains( "elapsed" ) && stderr.contains( "task" ),
    "EC-2: stderr must list valid column keys. Got: {stderr}"
  );
}

// ── EC-3: `CLR_PS_COLUMNS` env var fallback ──────────────────────────────────

/// EC-3: `CLR_PS_COLUMNS=pid,elapsed` env var shows PID and Elapsed; hides CPU%, RAM, Task.
#[ cfg( unix ) ]
#[ test ]
fn ec3_env_var_columns_fallback()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", &path_val )
    .env( "CLR_PS_COLUMNS", "pid,elapsed" )
    .output()
    .expect( "run clr ps with CLR_PS_COLUMNS=pid,elapsed" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-3: exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "PID" ),     "EC-3: PID must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "Elapsed" ), "EC-3: Elapsed must appear. Got:\n{stdout}" );
  assert!( !stdout.contains( "CPU%" ),   "EC-3: CPU% must NOT appear. Got:\n{stdout}" );
  assert!( !stdout.contains( "RAM" ),    "EC-3: RAM must NOT appear. Got:\n{stdout}" );
  assert!( !stdout.contains( "Task" ),   "EC-3: Task must NOT appear. Got:\n{stdout}" );
}

// ── EC-4: CLI `--columns` wins over `CLR_PS_COLUMNS` ────────────────────────

/// EC-4: CLI `--columns pid,path` wins over `CLR_PS_COLUMNS=pid,elapsed` env var.
#[ cfg( unix ) ]
#[ test ]
fn ec4_cli_columns_wins_over_env_var()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--columns", "pid,path" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PS_COLUMNS", "pid,elapsed" )
    .output()
    .expect( "run clr ps --columns pid,path with CLR_PS_COLUMNS=pid,elapsed" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-4: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "Absolute Path" ),
    "EC-4: Absolute Path must appear (CLI --columns wins). Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "Elapsed" ),
    "EC-4: Elapsed must NOT appear (CLI wins over env var). Got:\n{stdout}"
  );
}

// ── EC-5: `--columns` overrides `--wide` ─────────────────────────────────────

/// EC-5: `--columns pid,task --wide` → only PID and Task shown; Mode/Command/Binary absent.
#[ cfg( unix ) ]
#[ test ]
fn ec5_columns_wins_over_wide()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--columns", "pid,task", "--wide" ] )
    .env( "PATH", &path_val )
    .output()
    .expect( "run clr ps --columns pid,task --wide" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-5: exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "PID" ),  "EC-5: PID must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "Task" ), "EC-5: Task must appear. Got:\n{stdout}" );
  assert!( !stdout.contains( "Mode" ),    "EC-5: Mode must NOT appear (--columns wins). Got:\n{stdout}" );
  assert!( !stdout.contains( "Command" ), "EC-5: Command must NOT appear. Got:\n{stdout}" );
  assert!( !stdout.contains( "Binary" ),  "EC-5: Binary must NOT appear. Got:\n{stdout}" );
}

// ── EC-6: Optional columns displayed when requested ──────────────────────────

/// EC-6: `--columns idx,pid,mode,cmd,binary` shows `#`, PID, Mode, Command, Binary.
#[ cfg( unix ) ]
#[ test ]
fn ec6_optional_columns_displayed()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--columns", "idx,pid,mode,cmd,binary" ] )
    .env( "PATH", &path_val )
    .output()
    .expect( "run clr ps --columns idx,pid,mode,cmd,binary" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-6: exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( '#' ),       "EC-6: # (idx) header must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "PID" ),     "EC-6: PID header must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "Mode" ),    "EC-6: Mode header must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "Command" ), "EC-6: Command header must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "Binary" ),  "EC-6: Binary header must appear. Got:\n{stdout}" );
}

// ── EC-7: Default columns shown without `--columns` ──────────────────────────

/// EC-7: Default `clr ps` shows 9 default columns (including Mode); hides Command, Binary.
#[ cfg( unix ) ]
#[ test ]
fn ec7_default_columns_shown()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", &path_val )
    .env_remove( "CLR_PS_COLUMNS" )
    .output()
    .expect( "run clr ps (default columns)" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-7: exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "PID" ),           "EC-7: PID must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "Elapsed" ),       "EC-7: Elapsed must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "CPU%" ),          "EC-7: CPU% must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "RAM" ),           "EC-7: RAM must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "State" ),         "EC-7: State must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "Mode" ),          "EC-7: Mode must appear in default (9th default col). Got:\n{stdout}" );
  assert!( stdout.contains( "Absolute Path" ), "EC-7: Absolute Path must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "Task" ),          "EC-7: Task must appear. Got:\n{stdout}" );
  assert!( !stdout.contains( "Command" ), "EC-7: Command must NOT appear in default. Got:\n{stdout}" );
  assert!( !stdout.contains( "Binary" ),  "EC-7: Binary must NOT appear in default. Got:\n{stdout}" );
}

// ── EC-8: Help output contains `--columns` ───────────────────────────────────

/// EC-8: `clr ps --help` stdout contains `--columns`.
#[ test ]
fn ec8_help_contains_columns()
{
  let out    = run_cli( &[ "ps", "--help" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-8: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "--columns" ),
    "EC-8: --help must document --columns. Got: {stdout}"
  );
}

// ── EC-9: `idx` counter reflects visible rows after filtering ─────────────────

/// EC-9: `--mode print --columns idx,pid,task` shows only print-mode row;
/// `#` header is visible (idx column included).
#[ cfg( unix ) ]
#[ test ]
fn ec9_idx_counter_reflects_filtered_rows()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();

  let mut bg_interactive = spawn_fake_claude( &path_val );
  let mut bg_print       = spawn_print_claude( &path_val );
  let     pid_print      = bg_print.id();

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--mode", "print", "--columns", "idx,pid,task" ] )
    .env( "PATH", &path_val )
    .output()
    .expect( "run clr ps --mode print --columns idx,pid,task" );

  let _ = bg_interactive.kill();
  let _ = bg_interactive.wait();
  let _ = bg_print.kill();
  let _ = bg_print.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-9: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( '#' ),
    "EC-9: idx (#) header must appear. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( &pid_print.to_string() ),
    "EC-9: print PID {pid_print} must appear. Got:\n{stdout}"
  );
}

// ── EC-10: Help lists correct column key names (BUG-303 regression) ───────────

/// EC-10: `clr ps --help` lists `idx` and `cmd` as column keys, NOT `num` or `command`.
///
/// # Root Cause
/// `print_ps_help()` was authored with `num`/`command` key names that diverged from
/// the `COLUMN_KEYS` constant (`idx`/`cmd`). A user reading help and typing
/// `--columns num` would receive "unknown column key" despite following the docs.
///
/// # Why Not Caught
/// EC-8 only checked that `--columns` appeared in help; no test verified that the
/// listed key names matched what `validate_columns()` actually accepts.
///
/// # Fix Applied
/// Changed 3 lines in `src/cli/help.rs`: `num` → `idx`, `command` → `cmd`,
/// and the DEFAULT COLUMNS summary line. Fix comment added to `help.rs` (Fix(BUG-303)).
///
/// # Prevention
/// This test asserts the correct key names appear and the wrong ones do not.
/// It will fail immediately if `COLUMN_KEYS` is renamed without updating `print_ps_help()`.
///
/// # Pitfall
/// `COLUMN_KEYS` and `print_ps_help()` are separate sources — there is no compile-time
/// link between them. Renaming a column key MUST update both the constant and the help
/// text; they do not auto-synchronize.
#[ test ]
fn ec10_help_column_keys_match_column_keys_constant()
{
  let out    = run_cli( &[ "ps", "--help" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-10: exit 0 expected, got {:?}", out.status.code() );

  // Correct key names must appear.
  assert!(
    stdout.contains( "  idx " ),
    "EC-10: 'idx' key must appear in help COLUMN KEYS. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "  cmd " ),
    "EC-10: 'cmd' key must appear in help COLUMN KEYS. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "DEFAULT COLUMNS: idx" ),
    "EC-10: DEFAULT COLUMNS must start with 'idx'. Got:\n{stdout}"
  );

  // Wrong key names from BUG-303 must not appear (regression guard).
  assert!(
    !stdout.contains( "  num " ),
    "EC-10: 'num' must NOT appear as a column key in help (BUG-303 regression). Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "  command " ),
    "EC-10: 'command' must NOT appear as a column key in help (BUG-303 regression). Got:\n{stdout}"
  );
}
