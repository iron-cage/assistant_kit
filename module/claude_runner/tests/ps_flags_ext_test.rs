//! Session-flags tests for `clr ps` (extended).
//!
//! Extension of `ps_flags_test.rs` (IT-30–IT-38, US-18–US-20) covering the
//! remaining flag/legend user stories, env var overrides, and CPU-activity flags.
//!
//! Test spec: [`tests/docs/cli/command/06_ps.md`](docs/cli/command/06_ps.md)
//! and [`tests/docs/cli/user_story/26_session_listing.md`](docs/cli/user_story/26_session_listing.md).
//!
//! # Test Case Index
//!
//! | ID     | Name                                                                | Category     |
//! |--------|---------------------------------------------------------------------|--------------|
//! | US-21  | 🐘 High-RAM flag with `CLR_PS_HIGH_RAM_MB=0` threshold              | User Story   |
//! | US-22  | ⚠ Dead-metrics flag for session with unreadable proc stats          | User Story   |
//! | IT-39  | Sleeping process → no ⚡ flag (CPU delta = 0)                       | Behavioral   |
//! | IT-40  | Busy-loop process → ⚡ flag present (CPU delta ≫ 3)                 | Behavioral   |
//! | US-23  | ⚡ Active flag for session with CPU delta >= 3 ticks                | User Story   |
//! | US-24  | 🖨 Print-mode flag for print-mode session                            | User Story   |
//! | US-25  | Legend appears below active table when flags present                | User Story   |
//! | US-26  | Legend absent when no flags present                                 | User Story   |
//! | E41    | `CLR_PS_ANCIENT_SECS` env var: valid triggers 🕰; invalid silently ignored | Env Var |
//! | E42    | `CLR_PS_HIGH_RAM_MB` env var: valid triggers 🐘; invalid silently ignored  | Env Var |
//! | IT-41  | 🐳 flag for sibling-prefix cwd, e.g. `/home/alice2` vs `$HOME=/home/alice` (BUG-383) | Behavioral |

#![ cfg( unix ) ]

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ make_proc_dir, stdout_str };

// ── US-21: 🐘 High-RAM flag with CLR_PS_HIGH_RAM_MB=0 ─────────────────────

/// US-21: Developer running `clr ps` with `CLR_PS_HIGH_RAM_MB=0` sees every
/// running session marked as 🐘 High RAM (any non-zero RSS exceeds 0 MB threshold).
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn us21_high_ram_flag_with_zero_threshold()
{
  use cli_binary_test_helpers::fake_claude_binary_dir;

  let ( _bin_dir, path_val ) = fake_claude_binary_dir();
  let mut bg = std::process::Command::new( "claude" )
    .env( "PATH", &path_val )
    .arg( "30" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude" );
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );

  let proc = make_proc_dir( &[ bg.id() ] );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_PS_ANCIENT_SECS", "999999" )
    .env( "CLR_PS_HIGH_RAM_MB", "0" )
    .output()
    .expect( "run clr ps" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "US-21: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "🐘" ),
    "US-21: 🐘 flag must appear with CLR_PS_HIGH_RAM_MB=0. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "High RAM" ),
    "US-21: legend must contain 'High RAM'. Got:\n{stdout}"
  );
}

// ── US-22: ⚠ Dead-metrics flag ────────────────────────────────────────────

/// US-22: Developer running `clr ps` sees ⚠ Dead metrics for a process whose
/// `/proc/{pid}/stat` is absent (TOCTOU-dead session).
///
/// Uses `CLR_PROC_DIR` with a fake proc entry (PID 99999997) that has a `cmdline`
/// file but no `stat` file.  `read_process_metrics(99999997)` returns `None`.
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn us22_dead_metrics_flag_for_missing_stat()
{
  let fake_proc = tempfile::TempDir::new().expect( "fake proc dir" );
  let pid_dir   = fake_proc.path().join( "99999997" );
  std::fs::create_dir_all( &pid_dir ).expect( "create fake pid dir" );
  std::fs::write( pid_dir.join( "cmdline" ), b"claude\x0030\x00" )
    .expect( "write fake cmdline" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "CLR_PROC_DIR", fake_proc.path() )
    .output()
    .expect( "run clr ps" );

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "US-22: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "⚠" ),
    "US-22: ⚠ flag must appear for TOCTOU-dead session. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "Dead metrics" ),
    "US-22: legend must contain 'Dead metrics'. Got:\n{stdout}"
  );
}

// ── US-23: ⚡ Active flag for session with CPU delta >= 3 ticks ─────────────

/// US-23: Developer running `clr ps` sees ⚡ Active for a CPU-intensive session
/// whose CPU delta >= 3 ticks in the 1-second sample window.
///
/// Spawns a tight shell busy-loop via `/bin/sh --arg0 claude -c 'while :; do :; done'`.
/// The loop consumes ~100 ticks/s — well above the threshold of 3.
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn us23_active_flag_for_cpu_intensive_session()
{
  use std::os::unix::process::CommandExt as _;
  use cli_binary_test_helpers::fake_claude_binary_dir;

  let ( _bin_dir, path_val ) = fake_claude_binary_dir();

  // Busy-loop process: argv = ["claude", "-c", "while :; do :; done"]
  // arg0("claude") sets argv[0] → basename "claude" → visible to find_claude_processes().
  let mut bg = std::process::Command::new( "/bin/sh" )
    .arg0( "claude" )
    .arg( "-c" )
    .arg( "while :; do :; done" )
    .env( "PATH", &path_val )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn busy-loop claude" );
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );

  let proc = make_proc_dir( &[ bg.id() ] );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_PS_ANCIENT_SECS", "999999" )
    .env( "CLR_PS_HIGH_RAM_MB", "999999" )
    .output()
    .expect( "run clr ps" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "US-23: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "⚡" ),
    "US-23: ⚡ flag must appear for busy-loop session (CPU delta >> 3). Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "Active" ),
    "US-23: legend must contain 'Active'. Got:\n{stdout}"
  );
}

// ── US-24: 🖨 Print-mode flag for print-mode session ──────────────────────

/// US-24: Developer running `clr ps` sees 🖨 Print mode for sessions started
/// with `--print`.
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn us24_print_mode_flag_for_print_session()
{
  use std::os::unix::process::CommandExt as _;
  use cli_binary_test_helpers::fake_claude_binary_dir;

  let ( _bin_dir, path_val ) = fake_claude_binary_dir();

  let mut bg = std::process::Command::new( "/bin/sh" )
    .arg0( "claude" )
    .arg( "-c" )
    .arg( "sleep 30; :" )
    .arg( "--print" )
    .env( "PATH", &path_val )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn print-mode claude" );
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );

  let proc = make_proc_dir( &[ bg.id() ] );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_PS_ANCIENT_SECS", "999999" )
    .env( "CLR_PS_HIGH_RAM_MB", "999999" )
    .output()
    .expect( "run clr ps" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "US-24: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "🖨" ),
    "US-24: 🖨 flag must appear for print-mode session. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "Print mode" ),
    "US-24: legend must contain 'Print mode'. Got:\n{stdout}"
  );
}

// ── US-25: Legend appears when flags present ───────────────────────────────

/// US-25: Developer running `clr ps` with ≥1 flag-carrying session sees a legend
/// line after the active sessions table.
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn us25_legend_appears_when_flags_present()
{
  use cli_binary_test_helpers::fake_claude_binary_dir;

  let ( _bin_dir, path_val ) = fake_claude_binary_dir();
  let temp_home    = tempfile::TempDir::new().expect( "tmp home" );
  let outside_home = tempfile::TempDir::new().expect( "outside home" );

  let home_str    = temp_home.path().to_string_lossy().to_string();
  let outside_str = outside_home.path().to_string_lossy().to_string();
  assert!( !outside_str.starts_with( &home_str ), "dirs must not overlap" );

  let mut bg = std::process::Command::new( "claude" )
    .env( "PATH", &path_val )
    .arg( "30" )
    .current_dir( outside_home.path() )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude" );
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );

  let proc = make_proc_dir( &[ bg.id() ] );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
    .env( "HOME", temp_home.path() )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "US-25: exit 0 expected, got {:?}", out.status.code() );
  // Both the active table and the legend must appear.
  assert!(
    stdout.contains( "Active Sessions" ),
    "US-25: active table must appear. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "🐳" ) && stdout.contains( "Container" ),
    "US-25: legend must contain '🐳 Container'. Got:\n{stdout}"
  );
}

// ── US-26: Legend absent when no flags present ─────────────────────────────

/// US-26: Developer running `clr ps` with a clean session (no flags) sees
/// no flag emoji or legend in the output.
#[ cfg( unix ) ]
#[ test ]
fn us26_legend_absent_when_no_flags_present()
{
  use cli_binary_test_helpers::fake_claude_binary_dir;

  let ( _bin_dir, path_val ) = fake_claude_binary_dir();
  let temp_home = tempfile::TempDir::new().expect( "tmp home" );
  let work_dir  = temp_home.path().join( "project" );
  std::fs::create_dir_all( &work_dir ).expect( "create work dir" );

  let mut bg = std::process::Command::new( "claude" )
    .env( "PATH", &path_val )
    .arg( "30" )
    .current_dir( &work_dir )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude" );
  let pid = bg.id().to_string();
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );

  // Use --pid to isolate the fake process — live sessions with flags would pollute stdout.
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--pid", &pid ] )
    .env( "PATH", &path_val )
    .env( "HOME", temp_home.path() )
    .env( "CLR_PS_ANCIENT_SECS", "999999" )
    .env( "CLR_PS_HIGH_RAM_MB", "999999" )
    .output()
    .expect( "run clr ps" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "US-26: exit 0 expected, got {:?}", out.status.code() );
  for emoji in [ "👈", "🖨", "⚡", "🕰", "🐘", "⚠", "🐳" ]
  {
    assert!(
      !stdout.contains( emoji ),
      "US-26: flag emoji '{emoji}' must NOT appear when no flags fire. Got:\n{stdout}"
    );
  }
}

// ── E41: CLR_PS_ANCIENT_SECS env var ──────────────────────────────────────

/// E41: `CLR_PS_ANCIENT_SECS=0` triggers 🕰 for any running session;
/// an invalid value is silently ignored (default 28800 used — no 🕰 fires).
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn e41_ancient_secs_env_var()
{
  use cli_binary_test_helpers::fake_claude_binary_dir;

  let ( _bin_dir, path_val ) = fake_claude_binary_dir();

  // Sub-case (a): valid value 0 → 🕰 fires after 1 second.
  let mut bg = std::process::Command::new( "claude" )
    .env( "PATH", &path_val )
    .arg( "30" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude" );
  std::thread::sleep( core::time::Duration::from_millis( 1_100 ) );
  let proc_a = make_proc_dir( &[ bg.id() ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out_valid = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc_a.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_PS_ANCIENT_SECS", "0" )
    .env( "CLR_PS_HIGH_RAM_MB", "999999" )
    .output()
    .expect( "run clr ps (valid)" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout_valid = stdout_str( &out_valid );
  assert!(
    out_valid.status.success(),
    "E41a: exit 0 expected, got {:?}", out_valid.status.code()
  );
  assert!(
    stdout_valid.contains( "🕰" ),
    "E41a: 🕰 must fire with CLR_PS_ANCIENT_SECS=0. Got:\n{stdout_valid}"
  );

  // Sub-case (b): invalid value → silently ignored; default 28800 used → 🕰 absent
  // for our freshly-spawned process.  Check only the fake PID's row — host sessions
  // running >8 h will have 🕰 in their rows, which would false-positive whole-stdout search.
  let ( _bin_dir2, path_val2 ) = fake_claude_binary_dir();
  let mut bg2 = std::process::Command::new( "claude" )
    .env( "PATH", &path_val2 )
    .arg( "30" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude 2" );
  let pid2 = bg2.id().to_string();
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );
  let proc_b = make_proc_dir( &[ bg2.id() ] );

  let out_invalid = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val2 )
    .env( "CLR_PROC_DIR", proc_b.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_PS_ANCIENT_SECS", "not_a_number" )
    .env( "CLR_PS_HIGH_RAM_MB", "999999" )
    .output()
    .expect( "run clr ps (invalid)" );

  let _ = bg2.kill();
  let _ = bg2.wait();

  let stdout_invalid = stdout_str( &out_invalid );
  assert!(
    out_invalid.status.success(),
    "E41b: exit 0 expected with invalid CLR_PS_ANCIENT_SECS, got {:?}",
    out_invalid.status.code()
  );
  let pid_row = stdout_invalid.lines()
    .find( | l | l.contains( &pid2 ) )
    .unwrap_or( "" );
  assert!(
    !pid_row.contains( "🕰" ),
    "E41b: 🕰 must NOT fire for PID {pid2} when CLR_PS_ANCIENT_SECS is invalid (default 28800). Got:\n{stdout_invalid}"
  );
}

// ── E42: CLR_PS_HIGH_RAM_MB env var ───────────────────────────────────────

/// E42: `CLR_PS_HIGH_RAM_MB=0` triggers 🐘 for any running session;
/// an invalid value is silently ignored (default 400 MB used — 🐘 absent for sleep).
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn e42_high_ram_mb_env_var()
{
  use cli_binary_test_helpers::fake_claude_binary_dir;

  let bin = env!( "CARGO_BIN_EXE_clr" );

  // Sub-case (a): valid value 0 → 🐘 fires.
  let ( _bin_dir, path_val ) = fake_claude_binary_dir();
  let mut bg = std::process::Command::new( "claude" )
    .env( "PATH", &path_val )
    .arg( "30" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude" );
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );

  let proc_dir_zero     = tempfile::TempDir::new().expect( "proc_dir_zero" );
  let zero_ram_proc_dir = proc_dir_zero.path().to_str().expect( "proc_dir_zero UTF-8" );
  std::os::unix::fs::symlink(
    format!( "/proc/{}", bg.id() ),
    proc_dir_zero.path().join( bg.id().to_string() ),
  ).expect( "pid symlink zero-ram" );

  let out_valid = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", zero_ram_proc_dir )
    .env( "CLR_PS_ANCIENT_SECS", "999999" )
    .env( "CLR_PS_HIGH_RAM_MB", "0" )
    .output()
    .expect( "run clr ps (valid)" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout_valid = stdout_str( &out_valid );
  assert!(
    out_valid.status.success(),
    "E42a: exit 0 expected, got {:?}", out_valid.status.code()
  );
  assert!(
    stdout_valid.contains( "🐘" ),
    "E42a: 🐘 must fire with CLR_PS_HIGH_RAM_MB=0. Got:\n{stdout_valid}"
  );

  // Sub-case (b): invalid value → silently ignored; default 400 MB used → 🐘 absent for sleep.
  // Check only the fake PID's row — host sessions using >400 MB will have 🐘, false-positiving
  // a whole-stdout search.
  let ( _bin_dir2, path_val2 ) = fake_claude_binary_dir();
  let mut bg2 = std::process::Command::new( "claude" )
    .env( "PATH", &path_val2 )
    .arg( "30" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude 2" );
  let pid2 = bg2.id().to_string();
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );

  let proc_dir_bogus     = tempfile::TempDir::new().expect( "proc_dir_bogus" );
  let bogus_ram_proc_dir = proc_dir_bogus.path().to_str().expect( "proc_dir_bogus UTF-8" );
  std::os::unix::fs::symlink(
    format!( "/proc/{}", bg2.id() ),
    proc_dir_bogus.path().join( bg2.id().to_string() ),
  ).expect( "pid symlink bogus-ram" );

  let out_invalid = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val2 )
    .env( "CLR_PROC_DIR", bogus_ram_proc_dir )
    .env( "CLR_PS_ANCIENT_SECS", "999999" )
    .env( "CLR_PS_HIGH_RAM_MB", "bogus" )
    .output()
    .expect( "run clr ps (invalid)" );

  let _ = bg2.kill();
  let _ = bg2.wait();

  let stdout_invalid = stdout_str( &out_invalid );
  assert!(
    out_invalid.status.success(),
    "E42b: exit 0 expected with invalid CLR_PS_HIGH_RAM_MB, got {:?}",
    out_invalid.status.code()
  );
  let pid_row = stdout_invalid.lines()
    .find( | l | l.contains( &pid2 ) )
    .unwrap_or( "" );
  assert!(
    !pid_row.contains( "🐘" ),
    "E42b: 🐘 must NOT fire for PID {pid2} when CLR_PS_HIGH_RAM_MB is invalid (default 400). Got:\n{stdout_invalid}"
  );
}

// ── IT-39: Sleeping process → no ⚡ flag (CPU delta = 0) ────────────────────

/// IT-39: A sleeping `claude` process accumulates 0 CPU ticks in the 1-second
/// sample window, so the ⚡ flag must NOT fire.
///
/// Validates the negative path: delta = 0 < 3 → no ⚡.
///
/// # Host caveat
///
/// On the host with any CPU-active live sessions, those sessions appear in `clr ps`
/// output with ⚡, making `!stdout.contains("⚡")` fail falsely even though the
/// sleeping process is correctly unflagged. Reliable only in container (0 live sessions).
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn it39_sleeping_process_no_active_flag()
{
  use std::os::unix::process::CommandExt as _;
  use cli_binary_test_helpers::fake_claude_binary_dir;

  let ( _bin_dir, path_val ) = fake_claude_binary_dir();

  // Sleeping process: argv = ["claude", "-c", "sleep 300"]
  let mut bg = std::process::Command::new( "/bin/sh" )
    .arg0( "claude" )
    .arg( "-c" )
    .arg( "sleep 300" )
    .env( "PATH", &path_val )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn sleeping claude" );
  let pid = bg.id().to_string();
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--pid", &pid ] )
    .env( "PATH", &path_val )
    .env( "CLR_PS_ANCIENT_SECS", "999999" )
    .env( "CLR_PS_HIGH_RAM_MB", "999999" )
    .output()
    .expect( "run clr ps" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-39: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    !stdout.contains( "⚡" ),
    "IT-39: ⚡ must NOT appear for sleeping process (CPU delta = 0). Got:\n{stdout}"
  );
}

// ── IT-40: Busy-loop process → ⚡ flag present (CPU delta ≫ 3) ──────────────

/// IT-40: A busy-loop `claude` process consumes ~100 ticks/s, so the ⚡ flag
/// must fire (delta ≈ 100 >> 3). Also validates the legend reads "Active".
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn it40_busy_loop_process_active_flag()
{
  use std::os::unix::process::CommandExt as _;
  use cli_binary_test_helpers::fake_claude_binary_dir;

  let ( _bin_dir, path_val ) = fake_claude_binary_dir();

  // Busy-loop process: argv = ["claude", "-c", "while :; do :; done"]
  let mut bg = std::process::Command::new( "/bin/sh" )
    .arg0( "claude" )
    .arg( "-c" )
    .arg( "while :; do :; done" )
    .env( "PATH", &path_val )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn busy-loop claude" );
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );

  let proc = make_proc_dir( &[ bg.id() ] );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_PS_ANCIENT_SECS", "999999" )
    .env( "CLR_PS_HIGH_RAM_MB", "999999" )
    .output()
    .expect( "run clr ps" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-40: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "⚡" ),
    "IT-40: ⚡ must appear for busy-loop process (CPU delta >> 3). Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "Active" ),
    "IT-40: legend must contain 'Active'. Got:\n{stdout}"
  );
}

// ── IT-41: 🐳 flag for sibling-prefix cwd (BUG-383) ─────────────────────────

/// IT-41 (BUG-383): 🐳 flag fires when session cwd shares `$HOME`'s string
/// prefix without being a true path descendant of it (e.g. `home=/tmp/.tmpABC`,
/// `cwd=/tmp/.tmpABC2/project`). A raw `starts_with` comparison would wrongly
/// treat this sibling directory as "inside home" and suppress the flag; IT-31
/// and US-19 cannot exercise this case because two independent `TempDir`s
/// never share a string prefix by construction.
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn it41_container_flag_for_sibling_prefix_cwd()
{
  use cli_binary_test_helpers::fake_claude_binary_dir;

  let ( _bin_dir, path_val ) = fake_claude_binary_dir();
  let temp_home = tempfile::TempDir::new().expect( "tmp home" );
  let home_str  = temp_home.path().to_string_lossy().to_string();

  // Sibling cwd: shares home's full string prefix but is NOT a path descendant
  // (no `/` boundary immediately after the shared prefix).
  let sibling_cwd = format!( "{home_str}2/project" );
  std::fs::create_dir_all( &sibling_cwd ).expect( "create sibling cwd" );
  assert!(
    sibling_cwd.starts_with( &home_str ),
    "IT-41: precondition — sibling_cwd must share home's string prefix"
  );
  assert!(
    !sibling_cwd.starts_with( &format!( "{home_str}/" ) ),
    "IT-41: precondition — sibling_cwd must not be a true path descendant of home"
  );

  let mut bg = std::process::Command::new( "claude" )
    .env( "PATH", &path_val )
    .arg( "30" )
    .current_dir( &sibling_cwd )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude" );
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );

  let fake_proc     = tempfile::TempDir::new().expect( "fake_proc" );
  let fake_proc_str = fake_proc.path().to_str().expect( "fake_proc UTF-8" );
  std::os::unix::fs::symlink(
    format!( "/proc/{}", bg.id() ),
    fake_proc.path().join( bg.id().to_string() ),
  ).expect( "pid symlink" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
    .env( "HOME", temp_home.path() )
    .env( "CLR_PROC_DIR", fake_proc_str )
    .env( "CLR_PS_ANCIENT_SECS", "999999" )
    .env( "CLR_PS_HIGH_RAM_MB", "999999" )
    .output()
    .expect( "run clr ps" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-41: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "🐳" ),
    "IT-41 (BUG-383): 🐳 flag must fire for sibling-prefix cwd outside home. Got:\n{stdout}"
  );
}
