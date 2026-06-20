//! `clr tools` Subcommand Integration Tests
//!
//! Covers the `tools` command from `docs/cli/command/08_tools.md`.
//! Verifies that `clr tools` lists all 26 Claude Code built-in tools with correct
//! column headers and exit 0.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::run_cli;

// ── IT-1: `clr tools` exits 0 ─────────────────────────────────────────────────

/// IT-1: `clr tools` exits 0.
#[ test ]
fn it1_tools_exits_zero()
{
  let out = run_cli( &[ "tools" ] );
  assert!(
    out.status.success(),
    "`clr tools` must exit 0. Got: {:?}",
    out.status.code()
  );
}

// ── IT-2: Output contains expected tool names ─────────────────────────────────

/// IT-2: Stdout lists at least "Read", "Write", "Edit", "Bash" and "Agent".
#[ test ]
fn it2_tools_lists_core_tools()
{
  let out = run_cli( &[ "tools" ] );
  assert!( out.status.success(), "`clr tools` must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  for tool in &[ "Read", "Write", "Edit", "Bash", "Agent" ]
  {
    assert!(
      stdout.contains( tool ),
      "`clr tools` stdout must contain '{tool}'. Got:\n{stdout}"
    );
  }
}

// ── IT-3: Output contains expected categories ─────────────────────────────────

/// IT-3: Stdout contains the category headers present in the TOOLS table.
#[ test ]
fn it3_tools_lists_categories()
{
  let out = run_cli( &[ "tools" ] );
  assert!( out.status.success(), "`clr tools` must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  for cat in &[ "File Operations", "Shell", "Search", "Web" ]
  {
    assert!(
      stdout.contains( cat ),
      "`clr tools` stdout must contain category '{cat}'. Got:\n{stdout}"
    );
  }
}

// ── IT-4: Output contains table caption with count ────────────────────────────

/// IT-4: Stdout contains the caption "Claude Code Tools" and "26 built-in".
#[ test ]
fn it4_tools_caption_present()
{
  let out = run_cli( &[ "tools" ] );
  assert!( out.status.success(), "`clr tools` must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "Claude Code Tools" ),
    "`clr tools` stdout must contain table caption 'Claude Code Tools'. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "26" ),
    "`clr tools` stdout must reference 26 tools. Got:\n{stdout}"
  );
}

// ── IT-5: `clr tools --help` exits 0 ─────────────────────────────────────────

/// IT-5: `clr tools --help` exits 0 and prints usage.
#[ test ]
fn it5_tools_help_exits_zero()
{
  let out = run_cli( &[ "tools", "--help" ] );
  assert!(
    out.status.success(),
    "`clr tools --help` must exit 0. Got: {:?}",
    out.status.code()
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "tools" ),
    "`clr tools --help` stdout must contain 'tools'. Got:\n{stdout}"
  );
}

// ── IT-6: `clr tools` output contains Cron and Mode tools ────────────────────

/// IT-6: Stdout lists Cron and Mode tool names from the Scheduling/Mode categories.
#[ test ]
fn it6_tools_lists_scheduling_and_mode_tools()
{
  let out = run_cli( &[ "tools" ] );
  assert!( out.status.success(), "`clr tools` must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  for tool in &[ "CronCreate", "CronDelete", "CronList", "EnterPlanMode", "ExitPlanMode" ]
  {
    assert!(
      stdout.contains( tool ),
      "`clr tools` stdout must contain '{tool}'. Got:\n{stdout}"
    );
  }
}

// ── IT-7: `clr help` mentions `tools` command ────────────────────────────────

/// IT-7: `clr --help` lists `tools` in the COMMANDS section.
#[ test ]
fn it7_tools_listed_in_main_help()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "`clr --help` must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "tools" ),
    "`clr --help` stdout must mention 'tools' command. Got:\n{stdout}"
  );
}

// ── IT-8: `clr tools -h` short help flag ─────────────────────────────────────

/// IT-8: `clr tools -h` exits 0 and prints usage containing "tools".
///
/// Spec: `tests/docs/cli/command/08_tools.md` IT-8
#[ test ]
fn it8_tools_short_help_flag()
{
  let out = run_cli( &[ "tools", "-h" ] );
  assert!(
    out.status.success(),
    "`clr tools -h` must exit 0. Got: {:?}",
    out.status.code()
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "tools" ),
    "`clr tools -h` stdout must contain 'tools'. Got:\n{stdout}"
  );
}

// ── IT-9: `clr tools <unexpected-arg>` rejects arguments ─────────────────────

/// IT-9: `clr tools some-arg` exits 1 with error message — no flags or positional
/// args are accepted (other than --help/-h).
///
/// Spec: `tests/docs/cli/command/08_tools.md` IT-9
///
/// # Root Cause (bug reproducer)
/// `dispatch_tools()` had no unknown-arg guard: any input after the help check
/// fell through to table rendering and exited 0 silently.
///
/// # Fix Applied
/// Unknown-arg guard inserted before table build in `src/cli/tools.rs`.
#[ test ]
fn it9_tools_rejects_unknown_arg()
{
  let out = run_cli( &[ "tools", "some-unknown-arg" ] );
  assert!(
    !out.status.success(),
    "`clr tools some-unknown-arg` must exit non-zero. Got exit 0"
  );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "`clr tools some-unknown-arg` must exit 1. Got: {:?}",
    out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "does not accept arguments" ),
    "`clr tools some-unknown-arg` stderr must contain 'does not accept arguments'. Got:\n{stderr}"
  );
}
