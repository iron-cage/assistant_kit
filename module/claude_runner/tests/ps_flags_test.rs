//! Session-flags tests for `clr ps`.
//!
//! Test spec: [`tests/docs/cli/command/06_ps.md`](docs/cli/command/06_ps.md) IT-30–IT-38
//! and [`tests/docs/cli/user_story/26_session_listing.md`](docs/cli/user_story/26_session_listing.md)
//! US-18–US-26, plus E41–E42 env var tests.
//!
//! # Test Case Index
//!
//! | ID     | Name                                                                | Category     |
//! |--------|---------------------------------------------------------------------|--------------|
//! | IT-30  | `Flags` column absent when no session has any flag                  | Behavioral   |
//! | IT-31  | 🐳 flag for session cwd outside `$HOME`                             | Behavioral   |
//! | IT-32  | 🕰 flag when elapsed exceeds `CLR_PS_ANCIENT_SECS` threshold        | Behavioral   |
//! | IT-33  | 🐘 flag when RAM exceeds `CLR_PS_HIGH_RAM_MB` threshold             | Behavioral   |
//! | IT-34  | ⚠ flag for TOCTOU-dead session (no `/proc/{pid}/stat`)             | Behavioral   |
//! | IT-35  | 🖨 flag for print-mode session                                       | Behavioral   |
//! | IT-36  | Legend printed below active table when ≥1 flag present              | Behavioral   |
//! | IT-37  | Legend absent when no flags present                                 | Behavioral   |
//! | IT-38  | `CLR_PS_ANCIENT_SECS`/`CLR_PS_HIGH_RAM_MB` override thresholds     | Behavioral   |
//! | US-18  | `Flags` column absent when no flags apply                           | User Story   |
//! | US-19  | 🐳 Container flag for session cwd outside `$HOME`                   | User Story   |
//! | US-20  | 🕰 Ancient flag with `CLR_PS_ANCIENT_SECS=0` threshold              | User Story   |
//! | US-21  | 🐘 High-RAM flag with `CLR_PS_HIGH_RAM_MB=0` threshold              | User Story   |
//! | US-22  | ⚠ Dead-metrics flag for session with unreadable proc stats          | User Story   |
//! | US-23  | ⚡ Running flag for session in kernel state R                        | User Story   |
//! | US-24  | 🖨 Print-mode flag for print-mode session                            | User Story   |
//! | US-25  | Legend appears below active table when flags present                | User Story   |
//! | US-26  | Legend absent when no flags present                                 | User Story   |
//! | E41    | `CLR_PS_ANCIENT_SECS` env var: valid triggers 🕰; invalid silently ignored | Env Var |
//! | E42    | `CLR_PS_HIGH_RAM_MB` env var: valid triggers 🐘; invalid silently ignored  | Env Var |

mod cli_binary_test_helpers;
use cli_binary_test_helpers::stdout_str;

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
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
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

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
    .env( "HOME", temp_home.path() )
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

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
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

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
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

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
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

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
    .env( "HOME", temp_home.path() )
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
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
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

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
    .env( "HOME", temp_home.path() )
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
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
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
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn us19_container_flag_for_cwd_outside_home()
{
  use cli_binary_test_helpers::fake_claude_binary_dir;

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

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
    .env( "HOME", temp_home.path() )
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

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
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

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
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

// ── US-23: ⚡ Running flag for session in kernel state R ───────────────────

/// US-23: Developer running `clr ps` sees ⚡ Running for a CPU-intensive session
/// whose `/proc/{pid}/stat` state field is `R`.
///
/// Spawns a tight shell busy-loop via `/bin/sh --arg0 claude -c 'while :; do :; done'`.
/// The loop has no blocking syscalls so the kernel state is `R` essentially all the time.
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn us23_running_flag_for_cpu_intensive_session()
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

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
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
    "US-23: ⚡ flag must appear for session in kernel state R. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "Running" ),
    "US-23: legend must contain 'Running'. Got:\n{stdout}"
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

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
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

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
    .env( "HOME", temp_home.path() )
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
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps" ] )
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

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out_valid = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
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

  // Sub-case (b): invalid value → silently ignored; default 28800 used → 🕰 absent.
  let ( _bin_dir2, path_val2 ) = fake_claude_binary_dir();
  let mut bg2 = std::process::Command::new( "claude" )
    .env( "PATH", &path_val2 )
    .arg( "30" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude 2" );
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );

  let out_invalid = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val2 )
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
  assert!(
    !stdout_invalid.contains( "🕰" ),
    "E41b: 🕰 must NOT fire when CLR_PS_ANCIENT_SECS is invalid (default 28800 used). Got:\n{stdout_invalid}"
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

  let out_valid = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val )
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
  let ( _bin_dir2, path_val2 ) = fake_claude_binary_dir();
  let mut bg2 = std::process::Command::new( "claude" )
    .env( "PATH", &path_val2 )
    .arg( "30" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude 2" );
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );

  let out_invalid = std::process::Command::new( bin )
    .args( [ "ps" ] )
    .env( "PATH", &path_val2 )
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
  // sleep uses far less than 400 MB RSS.
  assert!(
    !stdout_invalid.contains( "🐘" ),
    "E42b: 🐘 must NOT fire when CLR_PS_HIGH_RAM_MB is invalid (default 400 used). Got:\n{stdout_invalid}"
  );
}
