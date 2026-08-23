//! Session-flags tests for `clr ps`.
//!
//! Test spec: [`tests/docs/cli/command/06_ps.md`](docs/cli/command/06_ps.md) IT-30–IT-40,
//! IT-48–IT-49
//! and [`tests/docs/cli/user_story/26_session_listing.md`](docs/cli/user_story/26_session_listing.md)
//! US-18–US-20.
//!
//! US-21–US-26, E41–E42, IT-39–IT-41 live in `ps_flags_ext_test.rs`.
//!
//! # Test Case Index
//!
//! | ID     | Name                                                                | Category     |
//! |--------|---------------------------------------------------------------------|--------------|
//! | IT-30  | `Flags` column absent when no session has any flag                  | Behavioral   |
//! | IT-31  | 🐳 flag for session cwd outside `$HOME`                             | Behavioral   |
//! | IT-32  | 🕰 flag when elapsed exceeds `CLR_PS_ANCIENT_SECS` threshold        | Behavioral   |
//! | IT-33  | 🐘 flag when RAM exceeds `CLR_PS_HIGH_RAM_MB` threshold             | Behavioral   |
//! | IT-34  | ⚠ flag for TOCTOU-dead session (no `/proc/{pid}` entry at all)      | Behavioral   |
//! | IT-35  | 🖨 flag for print-mode session                                       | Behavioral   |
//! | IT-36  | Legend printed below active table when ≥1 flag present              | Behavioral   |
//! | IT-37  | Legend absent when no flags present                                 | Behavioral   |
//! | IT-38  | `CLR_PS_ANCIENT_SECS`/`CLR_PS_HIGH_RAM_MB` override thresholds     | Behavioral   |
//! | IT-48  | 🧟 flag for a `SIGSTOP`-suspended session (state `T`, not ⚠)        | Behavioral   |
//! | IT-49  | `ps --help` lists every session flag, symbol and name              | Documentation |
//! | US-18  | `Flags` column absent when no flags apply                           | User Story   |
//! | US-19  | 🐳 Container flag for session cwd outside `$HOME`                   | User Story   |
//! | US-20  | 🕰 Ancient flag with `CLR_PS_ANCIENT_SECS=0` threshold              | User Story   |

#![ cfg( unix ) ]

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ make_proc_dir, stdout_str };

// ── IT-30: Flags column absent when no session has any flag ────────────────

/// IT-30: When no flag conditions apply, the `Flags` column must not appear.
///
/// Setup: fake `claude` ELF spawned inside `$HOME`; impossibly high thresholds
/// prevent 🕰 and 🐘; interactive mode prevents 🖨; sleep state prevents ⚡;
/// not the parent of `clr ps` prevents 👈.
#[ cfg( unix ) ]
#[ test ]
fn it30_flags_column_absent_when_no_flags()
{
  use cli_binary_test_helpers::fake_claude_binary_dir;

  let ( _bin_dir, path_val ) = fake_claude_binary_dir();
  let temp_home = tempfile::TempDir::new().expect( "tmp home" );
  let work_dir  = temp_home.path().join( "work" );
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

  // Use --pid to isolate the fake process — other host sessions with flags would
  // make the Flags column appear, false-failing the absence assertion.
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
  assert!( out.status.success(), "IT-30: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    !stdout.contains( "Flags" ),
    "IT-30: 'Flags' column must NOT appear when no flags fire. Got:\n{stdout}"
  );
}

// ── IT-31: 🐳 flag for session cwd outside $HOME ───────────────────────────

// BUG-383: this test (and US-19/US-25 below) builds `home`/`cwd` from two independent
// TempDir::new() calls, which can never share a string prefix — so none of them exercise
// the sibling-prefix false-match case (home=/home/alice, cwd=/home/alice2/x). A dedicated
// sibling-prefix case is still needed; see bug file § Prevention.

/// IT-31: 🐳 flag fires when session cwd is outside `$HOME`.
///
/// Setup: fake `claude` ELF spawned in a temp dir outside the fake HOME;
/// `HOME` is set to a separate temp dir so the session cwd does not start with HOME.
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn it31_container_flag_for_session_outside_home()
{
  use cli_binary_test_helpers::fake_claude_binary_dir;

  let ( _bin_dir, path_val ) = fake_claude_binary_dir();
  let temp_home    = tempfile::TempDir::new().expect( "tmp home" );
  let outside_home = tempfile::TempDir::new().expect( "tmp outside home" );

  // Verify the two dirs don't accidentally overlap.
  let home_str    = temp_home.path().to_string_lossy().to_string();
  let outside_str = outside_home.path().to_string_lossy().to_string();
  assert!(
    !outside_str.starts_with( &home_str ),
    "IT-31: outside_home must not be inside temp_home for this test to be valid"
  );

  let mut bg = std::process::Command::new( "claude" )
    .env( "PATH", &path_val )
    .arg( "30" )
    .current_dir( outside_home.path() )
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
  assert!( out.status.success(), "IT-31: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "🐳" ),
    "IT-31: 🐳 flag must appear for session outside HOME. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "Flags" ),
    "IT-31: 'Flags' column header must appear when 🐳 fires. Got:\n{stdout}"
  );
}

// ── IT-32: 🕰 flag when elapsed exceeds CLR_PS_ANCIENT_SECS ───────────────

/// IT-32: 🕰 flag fires when `CLR_PS_ANCIENT_SECS=0` (every session is "ancient").
///
/// Wait 1 100 ms after spawn so that the /proc elapsed computation yields ≥ 1 s
/// (`started_at` is in whole seconds; `unix_now()` must exceed it by ≥ 1).
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn it32_ancient_flag_fires_with_zero_threshold()
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
  // Sleep 1 100 ms total: ensures unix_now() > started_at so elapsed > 0 with threshold 0.
  std::thread::sleep( core::time::Duration::from_millis( 1_100 ) );

  let proc = make_proc_dir( &[ bg.id() ] );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_PS_ANCIENT_SECS", "0" )
    .env( "CLR_PS_HIGH_RAM_MB", "999999" )
    .output()
    .expect( "run clr ps" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-32: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "🕰" ),
    "IT-32: 🕰 flag must appear when elapsed > 0 and CLR_PS_ANCIENT_SECS=0. Got:\n{stdout}"
  );
}

// ── IT-33: 🐘 flag when RAM exceeds CLR_PS_HIGH_RAM_MB ────────────────────

/// IT-33: 🐘 flag fires when `CLR_PS_HIGH_RAM_MB=0` (any non-zero RSS triggers it).
///
/// Every running process has some resident memory; `ram_kb` > 0 satisfies the condition.
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn it33_high_ram_flag_fires_with_zero_threshold()
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
  assert!( out.status.success(), "IT-33: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "🐘" ),
    "IT-33: 🐘 flag must appear when ram_kb > 0 and CLR_PS_HIGH_RAM_MB=0. Got:\n{stdout}"
  );
}

// ── IT-34: ⚠ flag for TOCTOU-dead session ─────────────────────────────────

/// IT-34: ⚠ flag fires when the process's `/proc/{pid}/stat` is absent.
///
/// Uses `CLR_PROC_DIR` to inject a fake proc entry for PID 99999998 with
/// only a `cmdline` file (no `stat`).  `find_claude_processes()` finds the
/// entry as "claude"; `read_process_metrics(99999998)` tries the real
/// `/proc/99999998/stat` which does not exist → returns `None` → ⚠ fires.
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn it34_dead_metrics_flag_for_missing_stat()
{
  let fake_proc = tempfile::TempDir::new().expect( "fake proc dir" );
  let pid_dir   = fake_proc.path().join( "99999998" );
  std::fs::create_dir_all( &pid_dir ).expect( "create fake pid dir" );
  // NUL-delimited cmdline: "claude\030" — basename "claude" makes find_claude_processes include it.
  std::fs::write( pid_dir.join( "cmdline" ), b"claude\x0030\x00" )
    .expect( "write fake cmdline" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "CLR_PROC_DIR", fake_proc.path() )
    .output()
    .expect( "run clr ps" );

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-34: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "⚠" ),
    "IT-34: ⚠ flag must appear when /proc/{{pid}}/stat is absent. Got:\n{stdout}"
  );
}

// ── IT-35: 🖨 flag for print-mode session ─────────────────────────────────

/// IT-35: 🖨 flag fires for sessions with `--print` in their cmdline args.
///
/// Uses `/bin/sh` with `arg0("claude")` so `/proc/{pid}/cmdline` shows
/// `"claude"` as argv[0] (visible to `find_claude_processes()`).
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn it35_print_mode_flag_for_print_session()
{
  use std::os::unix::process::CommandExt as _;
  use cli_binary_test_helpers::fake_claude_binary_dir;

  let ( _bin_dir, path_val ) = fake_claude_binary_dir();

  // Spawn print-mode process: argv = ["claude", "-c", "sleep 30; :", "--print"]
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
  assert!( out.status.success(), "IT-35: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "🖨" ),
    "IT-35: 🖨 flag must appear for print-mode session. Got:\n{stdout}"
  );
}

// ── IT-36: Legend printed below active table when ≥1 flag present ─────────

/// IT-36: Legend line appears below the active table when any flag fires.
///
/// Uses the 🐳 scenario (cwd outside HOME) to ensure at least one flag fires.
/// Asserts the legend contains the flag emoji and its human-readable name.
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn it36_legend_present_when_flags_fire()
{
  use cli_binary_test_helpers::fake_claude_binary_dir;

  let ( _bin_dir, path_val ) = fake_claude_binary_dir();
  let temp_home    = tempfile::TempDir::new().expect( "tmp home" );
  let outside_home = tempfile::TempDir::new().expect( "tmp outside home" );

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
    .env( "CLR_PS_ANCIENT_SECS", "999999" )
    .env( "CLR_PS_HIGH_RAM_MB", "999999" )
    .output()
    .expect( "run clr ps" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-36: exit 0 expected, got {:?}", out.status.code() );
  // Active table must be present.
  assert!(
    stdout.contains( "Active Sessions" ),
    "IT-36: active table must appear. Got:\n{stdout}"
  );
  // Legend must contain the 🐳 emoji and the name "Container".
  assert!(
    stdout.contains( "🐳" ) && stdout.contains( "Container" ),
    "IT-36: legend must contain '🐳 Container'. Got:\n{stdout}"
  );
}

// ── IT-37: Legend absent when no flags present ─────────────────────────────

/// IT-37: No legend line appears when no flags fire across all rows.
///
/// Setup is the same as IT-30 (thresholds maximised, cwd inside HOME).
#[ cfg( unix ) ]
#[ test ]
fn it37_legend_absent_when_no_flags()
{
  use cli_binary_test_helpers::fake_claude_binary_dir;

  let ( _bin_dir, path_val ) = fake_claude_binary_dir();
  let temp_home = tempfile::TempDir::new().expect( "tmp home" );
  let work_dir  = temp_home.path().join( "work" );
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
  assert!( out.status.success(), "IT-37: exit 0 expected, got {:?}", out.status.code() );
  // No flag emoji must appear in output.
  for emoji in [ "👈", "🖨", "⚡", "🕰", "🐘", "⚠", "🐳" ]
  {
    assert!(
      !stdout.contains( emoji ),
      "IT-37: flag emoji '{emoji}' must NOT appear when no flags fire. Got:\n{stdout}"
    );
  }
}

// ── IT-38: High thresholds suppress 🕰 and 🐘 ─────────────────────────────

/// IT-38: `CLR_PS_ANCIENT_SECS=999999` and `CLR_PS_HIGH_RAM_MB=999999` prevent
/// 🕰 and 🐘 from firing; no other flags fire in the standard sleep session.
#[ cfg( unix ) ]
#[ test ]
fn it38_high_thresholds_suppress_time_and_ram_flags()
{
  use cli_binary_test_helpers::fake_claude_binary_dir;

  let ( _bin_dir, path_val ) = fake_claude_binary_dir();
  let temp_home = tempfile::TempDir::new().expect( "tmp home" );
  let work_dir  = temp_home.path().join( "work" );
  std::fs::create_dir_all( &work_dir ).expect( "create work dir" );

  let mut bg = std::process::Command::new( "claude" )
    .env( "PATH", &path_val )
    .arg( "30" )
    .current_dir( &work_dir )
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
    .env( "CLR_PS_ANCIENT_SECS", "999999" )
    .env( "CLR_PS_HIGH_RAM_MB", "999999" )
    .output()
    .expect( "run clr ps" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-38: exit 0 expected, got {:?}", out.status.code() );
  assert!( !stdout.contains( "🕰" ), "IT-38: 🕰 must NOT fire with threshold 999999. Got:\n{stdout}" );
  assert!( !stdout.contains( "🐘" ), "IT-38: 🐘 must NOT fire with threshold 999999. Got:\n{stdout}" );
}

// ── US-18: Flags column absent when no flags apply ─────────────────────────

/// US-18: Developer running `clr ps` with a clean session (cwd inside HOME,
/// no ancient/high-RAM flags) sees no `Flags` column in the active table.
#[ cfg( unix ) ]
#[ test ]
fn us18_flags_column_absent_when_no_flags_apply()
{
  use cli_binary_test_helpers::fake_claude_binary_dir;

  let ( _bin_dir, path_val ) = fake_claude_binary_dir();
  let temp_home = tempfile::TempDir::new().expect( "tmp home" );
  let work_dir  = temp_home.path().join( "src" );
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
  assert!( out.status.success(), "US-18: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    !stdout.contains( "Flags" ),
    "US-18: 'Flags' column must NOT appear when no flags apply. Got:\n{stdout}"
  );
}

// ── US-19: 🐳 Container flag for session cwd outside $HOME ─────────────────

/// US-19: Developer sees 🐳 flag for a Claude session running inside a container
/// (cwd outside `$HOME`).
///
/// ## Why `CLR_PROC_DIR` isolation is required
///
/// Without `CLR_PROC_DIR`, `clr ps` scans real `/proc` and may pick up ambient
/// `claude` processes spawned by other tests running in parallel.  Those processes
/// have their own cwd which may or may not lie outside `$HOME`, causing the 🐳 flag
/// assertion to produce a false negative (missing 🐳) or false positive depending on
/// what processes happen to be alive at assertion time.  `make_proc_dir` confines
/// the scan to exactly the one fake-claude PID spawned by this test.
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn us19_container_flag_for_cwd_outside_home()
{
  use cli_binary_test_helpers::{ fake_claude_binary_dir, make_proc_dir };

  let ( _bin_dir, path_val ) = fake_claude_binary_dir();
  let temp_home    = tempfile::TempDir::new().expect( "tmp home" );
  let container    = tempfile::TempDir::new().expect( "container dir" );

  let home_str      = temp_home.path().to_string_lossy().to_string();
  let container_str = container.path().to_string_lossy().to_string();
  assert!( !container_str.starts_with( &home_str ), "dirs must not overlap" );

  let mut bg = std::process::Command::new( "claude" )
    .env( "PATH", &path_val )
    .arg( "30" )
    .current_dir( container.path() )
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
  assert!( out.status.success(), "US-19: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "🐳" ),
    "US-19: 🐳 flag must appear for cwd outside HOME. Got:\n{stdout}"
  );
  // Legend must name the flag.
  assert!(
    stdout.contains( "Container" ),
    "US-19: legend must contain 'Container'. Got:\n{stdout}"
  );
}

// ── US-20: 🕰 Ancient flag with CLR_PS_ANCIENT_SECS=0 ─────────────────────

/// US-20: Developer running `clr ps` with `CLR_PS_ANCIENT_SECS=0` sees every
/// running session marked as 🕰 Ancient (elapsed > 0 threshold).
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn us20_ancient_flag_with_zero_threshold()
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
  std::thread::sleep( core::time::Duration::from_millis( 1_100 ) );

  let proc = make_proc_dir( &[ bg.id() ] );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_PS_ANCIENT_SECS", "0" )
    .env( "CLR_PS_HIGH_RAM_MB", "999999" )
    .output()
    .expect( "run clr ps" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "US-20: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "🕰" ),
    "US-20: 🕰 flag must appear with CLR_PS_ANCIENT_SECS=0. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "Ancient" ),
    "US-20: legend must contain 'Ancient'. Got:\n{stdout}"
  );
}


// ── IT-48: 🧟 Odd state flag for a non-R/S kernel state ────────────────────

/// IT-48: 🧟 fires when `/proc/{pid}/stat` field 3 is neither `R` nor `S`.
///
/// Fixture is a session suspended with `SIGSTOP` — kernel state `T` — which is
/// what a user actually produces by suspending a session with Ctrl-Z.  Its `/proc` entries
/// stay fully readable, so `read_process_metrics()` returns `Some` and the ⚠
/// branch (metrics read failed outright) is never reached: 🧟 and ⚠ are disjoint.
///
/// A zombie (`Z`) deliberately is NOT the fixture here even though the flag
/// condition would match it: the kernel clears `cmdline` on exit, so
/// `find_claude_processes()` — which requires argv[0]'s basename to be `claude` —
/// drops a zombie before flags are ever computed.  `Z` is unreachable in practice;
/// `D`, `T`, and `t` are the states this flag actually surfaces.
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn it48_odd_state_flag_for_stopped_session()
{
  use cli_binary_test_helpers::{ fake_claude_binary_dir, spawn_fake_claude };

  let ( _bin_dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );
  let pid    = bg.id();
  let proc   = make_proc_dir( &[ pid ] );

  let stop = std::process::Command::new( "sh" )
    .args( [ "-c", &format!( "kill -STOP {pid}" ) ] )
    .status()
    .expect( "send SIGSTOP" );
  assert!( stop.success(), "IT-48: SIGSTOP delivery failed for pid {pid}" );
  std::thread::sleep( core::time::Duration::from_millis( 300 ) );

  // Precondition: confirm the kernel really reports a stopped state before
  // asserting on clr's rendering, so a failure points at the flag logic rather
  // than at the fixture.
  let stat  = std::fs::read_to_string( format!( "/proc/{pid}/stat" ) ).expect( "read stopped stat" );
  let state = stat
    .rfind( ')' )
    .and_then( | i | stat[ i + 1 .. ].split_whitespace().next() )
    .unwrap_or( "" );
  assert!(
    matches!( state, "T" | "t" ),
    "IT-48: precondition — pid {pid} must be stopped (T/t). Got stat: {stat}"
  );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_PS_ANCIENT_SECS", "999999" )
    .env( "CLR_PS_HIGH_RAM_MB", "999999" )
    .output()
    .expect( "run clr ps" );

  // SIGCONT first — a SIGSTOPped process ignores SIGKILL delivery scheduling
  // until it is resumed, so reaping without it can hang the test.
  let _ = std::process::Command::new( "sh" )
    .args( [ "-c", &format!( "kill -CONT {pid}" ) ] )
    .status();
  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-48: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "🧟" ),
    "IT-48: 🧟 must fire for a stopped session. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "Odd state" ),
    "IT-48: legend must contain 'Odd state'. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "⚠" ),
    "IT-48: ⚠ must NOT fire — a stopped process's /proc entries still parse. Got:\n{stdout}"
  );
}

// ── IT-49: `ps --help` lists every session flag ────────────────────────────

/// IT-49: `clr ps --help` documents all 9 session flags, symbol and name.
///
/// Guards the drift that let 🔌 Query mode ship in `FLAG_LEGEND` while `--help`
/// listed only 7 flags: the legend a user sees under the table and the legend
/// `--help` promises must name the same set.
#[ test ]
fn it49_help_lists_every_session_flag()
{
  use cli_binary_test_helpers::run_cli;

  let out    = run_cli( &[ "ps", "--help" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-49: exit 0 expected, got {:?}", out.status.code() );

  // Canonical display order, matching `FLAG_LEGEND` in `claude_runner_core::ps_table`.
  let expected : &[ ( &str, &str ) ] = &[
    ( "👈", "This session" ),
    ( "🖨",  "Print mode"   ),
    ( "🔌", "Query mode"   ),
    ( "⚡", "Active"       ),
    ( "🕰",  "Ancient"      ),
    ( "🐘", "High RAM"     ),
    ( "🧟", "Odd state"    ),
    ( "⚠",  "Dead metrics" ),
    ( "🐳", "Container"    ),
  ];

  for ( symbol, name ) in expected
  {
    assert!(
      stdout.contains( symbol ),
      "IT-49: --help must list the {symbol} symbol ({name}). Got:\n{stdout}"
    );
    assert!(
      stdout.contains( name ),
      "IT-49: --help must name the {symbol} flag as '{name}'. Got:\n{stdout}"
    );
  }
}
