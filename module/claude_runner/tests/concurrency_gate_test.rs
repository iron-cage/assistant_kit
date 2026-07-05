//! Integration tests for the `--max-sessions` concurrency gate (Unix-only: uses
//! synthetic `/proc` isolation and real ELF/shell-script fake `claude` processes).
#![ cfg( unix ) ]
//!
//! Test spec: `docs/cli/user_story/025_concurrency_gate.md`, `docs/cli/param/033_max_sessions.md`.
//!
//! # Test Case Index
//!
//! | ID  | Name                                                                      | TSK-368 Row |
//! |-----|----------------------------------------------------------------------------|-------------|
//! | T01 | 10 print-mode processes active, print invocation, default → gate triggers at 10 | T01 |
//! | T02 | 9 print-mode processes active, print invocation, default → gate does not trigger | T02 |
//! | T03 | 15 print-mode + 1 interactive active, interactive invocation → gate skipped, zero wait | T03 |
//! | T04 | 5 print-mode + 10 interactive active, print invocation, `--max-sessions 5` → print-mode-only count | T04 |
//! | T06 | `--max-sessions 0`, any process count → gate disabled, unchanged behavior | T06 |
//!
//! T05 (`clr --help` shows `default: 10`) is covered by
//! `param_edge_cases_test.rs::ec9_max_sessions_help_shows_default_ten`.

// BUG-387 task/bug/387_print_mode_concurrency_gate_toctou_race.md — every test above pre-seeds
// a static synthetic /proc snapshot and invokes exactly one clr binary; none launch N concurrent
// clr invocations racing each other against a shared, mutating occupier set, so none can exercise
// the check-then-spawn TOCTOU race. Missing: a T07 launching N concurrent invocations and
// asserting peak simultaneously-alive count never exceeds --max-sessions.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::
{
  fake_claude_binary_dir, fake_claude_dir, make_proc_dir,
  spawn_fake_claude, spawn_print_claude, spawn_print_claude_for,
};
use std::process::Command;

// ── T01: gate triggers at exactly 10 print-mode processes (default limit) ──────

/// T01: 10 print-mode processes active (9 long-lived + 1 short-lived), new print-mode
/// invocation, `--max-sessions` unset (default 10) → gate triggers and emits the
/// "10/10 sessions active; waiting" message, then releases once the short-lived
/// process self-expires and the count drops below 10.
#[ test ]
fn t01_gate_triggers_at_ten_print_mode_processes()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();

  let mut long_lived : Vec< std::process::Child > =
    ( 0..9 ).map( |_| spawn_print_claude( &occupier_path ) ).collect();
  let mut short_lived = spawn_print_claude_for( &occupier_path, 5 );

  let mut pids : Vec< u32 > = long_lived.iter().map( std::process::Child::id ).collect();
  pids.push( short_lived.id() );
  let proc = make_proc_dir( &pids );

  // Dispatched command's own fake claude — fast, exits immediately once the gate releases.
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "_CLR_GATE_POLL_SECS", "1" )
    .output()
    .expect( "invoke clr" );

  for child in &mut long_lived { let _ = child.kill(); let _ = child.wait(); }
  let _ = short_lived.kill();
  let _ = short_lived.wait();

  assert!(
    out.status.success(),
    "T01: exit must be 0 after gate releases. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    // Anchored on "Info: " so a wrong larger count (e.g. "Info: 110/10") can never
    // false-positive match via the "10/10" tail — AF1.
    stderr.contains( "Info: 10/10 sessions active; waiting" ),
    "T01: gate must report 10/10 print-mode sessions active. Got:\n{stderr}"
  );
}

// ── T02: gate does not trigger below the limit ──────────────────────────────────

/// T02: 9 print-mode processes active, new print-mode invocation, `--max-sessions`
/// unset (default 10) → gate does not trigger; the dispatched command proceeds
/// immediately with no wait message on stderr.
#[ test ]
fn t02_gate_does_not_trigger_below_ten_print_mode_processes()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();

  let mut occupiers : Vec< std::process::Child > =
    ( 0..9 ).map( |_| spawn_print_claude( &occupier_path ) ).collect();
  let pids : Vec< u32 > = occupiers.iter().map( std::process::Child::id ).collect();
  let proc = make_proc_dir( &pids );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "_CLR_GATE_POLL_SECS", "1" )
    .output()
    .expect( "invoke clr" );

  for child in &mut occupiers { let _ = child.kill(); let _ = child.wait(); }

  assert!(
    out.status.success(),
    "T02: exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "sessions active; waiting" ),
    "T02: gate must not trigger below the limit. Got:\n{stderr}"
  );
}

// ── T03: interactive invocations bypass the gate entirely ──────────────────────

/// T03 (AF1): 15 print-mode processes + 1 interactive process active — well above
/// any reasonable limit — plus an explicit `--max-sessions 1` (guaranteeing the gate
/// would trigger if entered at all). A new **interactive** invocation must skip the
/// gate entirely: no process scan, no wait. Proven by measuring wall-clock elapsed
/// time around the dispatched invocation only (excluding background-process setup)
/// and asserting it completes near-instantly rather than blocking for a poll cycle.
///
/// `_CLR_GATE_POLL_SECS` is deliberately left at its 30-second production default:
/// if the gate were mistakenly entered, the test would take at least 30 real seconds
/// (the first poll sleep) rather than failing fast — a stronger, unambiguous signal
/// than a short override would give.
#[ test ]
fn t03_interactive_invocation_bypasses_gate_with_zero_wait()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();

  let mut print_occupiers : Vec< std::process::Child > =
    ( 0..15 ).map( |_| spawn_print_claude( &occupier_path ) ).collect();
  let mut interactive_occupier = spawn_fake_claude( &occupier_path );

  let mut pids : Vec< u32 > = print_occupiers.iter().map( std::process::Child::id ).collect();
  pids.push( interactive_occupier.id() );
  let proc = make_proc_dir( &pids );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let start = std::time::Instant::now();
  let out = Command::new( bin )
    .args( [ "--interactive", "--max-sessions", "1", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "invoke clr" );
  let elapsed = start.elapsed();

  for child in &mut print_occupiers { let _ = child.kill(); let _ = child.wait(); }
  let _ = interactive_occupier.kill();
  let _ = interactive_occupier.wait();

  assert!(
    out.status.success(),
    "T03: exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  assert!(
    elapsed < core::time::Duration::from_secs( 5 ),
    "T03 (AF1): interactive dispatch must complete near-instantly (no gate poll). Elapsed: {elapsed:?}"
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "sessions active; waiting" ),
    "T03 (AF1): gate must never be entered for an interactive invocation. Got:\n{stderr}"
  );
}

// ── T04: print-mode-only counting excludes interactive processes ───────────────

/// T04: 5 print-mode processes (4 long-lived + 1 short-lived) + 10 long-lived
/// interactive processes active, new print-mode invocation, `--max-sessions 5` →
/// the gate must count print-mode processes only. It triggers at "5/5" (not
/// "15/5"), proving the 10 interactive processes are excluded, then releases once
/// the short-lived print-mode process self-expires.
#[ test ]
fn t04_gate_counts_print_mode_only_excludes_interactive()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();

  let mut long_lived_print : Vec< std::process::Child > =
    ( 0..4 ).map( |_| spawn_print_claude( &occupier_path ) ).collect();
  let mut short_lived_print = spawn_print_claude_for( &occupier_path, 5 );
  let mut interactive : Vec< std::process::Child > =
    ( 0..10 ).map( |_| spawn_fake_claude( &occupier_path ) ).collect();

  let mut pids : Vec< u32 > = long_lived_print.iter().map( std::process::Child::id ).collect();
  pids.push( short_lived_print.id() );
  pids.extend( interactive.iter().map( std::process::Child::id ) );
  let proc = make_proc_dir( &pids );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--max-sessions", "5", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "_CLR_GATE_POLL_SECS", "1" )
    .output()
    .expect( "invoke clr" );

  for child in &mut long_lived_print { let _ = child.kill(); let _ = child.wait(); }
  let _ = short_lived_print.kill();
  let _ = short_lived_print.wait();
  for child in &mut interactive { let _ = child.kill(); let _ = child.wait(); }

  assert!(
    out.status.success(),
    "T04: exit must be 0 after gate releases. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    // Anchored on "Info: " — "15/5" (unfiltered count) contains "5/5" as a bare
    // substring, which would false-positive an unanchored check. AF1.
    stderr.contains( "Info: 5/5 sessions active; waiting" ),
    "T04: gate must count print-mode processes only (5/5, not 15/5). Got:\n{stderr}"
  );
}

// ── T06: `--max-sessions 0` disables the gate regardless of process count ──────

/// T06: `--max-sessions 0` disables the gate entirely, regardless of active
/// process count (unchanged existing behavior — regression guard).
#[ test ]
fn t06_max_sessions_zero_disables_gate_regardless_of_count()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();

  let mut occupiers : Vec< std::process::Child > =
    ( 0..3 ).map( |_| spawn_print_claude( &occupier_path ) ).collect();
  let pids : Vec< u32 > = occupiers.iter().map( std::process::Child::id ).collect();
  let proc = make_proc_dir( &pids );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--max-sessions", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "invoke clr" );

  for child in &mut occupiers { let _ = child.kill(); let _ = child.wait(); }

  assert!(
    out.status.success(),
    "T06: exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "sessions active; waiting" ),
    "T06: --max-sessions 0 must disable the gate. Got:\n{stderr}"
  );
}
