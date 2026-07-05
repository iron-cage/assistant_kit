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
//! | T07 | gate state file `cwd` field remains valid JSON when cwd contains a literal `"` (BUG-384) | — |
//! | T08 | N concurrent live `clr` invocations racing a shared, dynamically-mutating occupier set → peak admitted count never exceeds `--max-sessions` (BUG-387) | — |
//!
//! T05 (`clr --help` shows `default: 10`) is covered by
//! `param_edge_cases_test.rs::ec9_max_sessions_help_shows_default_ten`.

// BUG-387 task/bug/387_print_mode_concurrency_gate_toctou_race.md — T01-T07 above all pre-seed
// a static synthetic /proc snapshot and invoke exactly one clr binary; none launch N concurrent
// clr invocations racing each other against a shared, mutating occupier set, so none could exercise
// the check-then-spawn TOCTOU race. T08 below closes that gap: it launches N concurrent live `clr`
// invocations and asserts peak simultaneously-admitted count never exceeds --max-sessions.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::
{
  fake_claude_binary_dir, fake_claude_dir, make_proc_dir,
  spawn_fake_claude, spawn_print_claude, spawn_print_claude_for,
};
use std::process::Command;

// ── T07: gate state file stays valid JSON when cwd contains a quote (BUG-384) ──

/// T07 (BUG-384): the gate-state file's `cwd` field must be JSON-escaped. Forces the
/// gate to actually block (`--max-sessions 1` against a single active print-mode
/// occupier) from a `current_dir` containing a literal `"` character, then reads the
/// resulting `$CLR_GATE_DIR/{pid}.json` file directly and asserts it parses as valid
/// JSON. Prior to the fix, `wait_for_session_slot()` spliced `cwd` unescaped into a
/// hand-rolled `format!()` JSON literal, so the embedded `"` prematurely closed the
/// string value and produced invalid JSON.
#[ test ]
fn t07_gate_state_file_valid_json_for_quoted_cwd()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();

  let mut occupier = spawn_print_claude( &occupier_path );
  let proc = make_proc_dir( &[ occupier.id() ] );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );

  let quoted_cwd_parent = tempfile::TempDir::new().expect( "quoted cwd parent" );
  let quoted_cwd = quoted_cwd_parent.path().join( "needs\"quote" );
  std::fs::create_dir_all( &quoted_cwd ).expect( "create quoted cwd" );

  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut child = Command::new( bin )
    .args( [ "-p", "--max-sessions", "1", "--journal", "off", "x" ] )
    .current_dir( &quoted_cwd )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn clr" );

  std::thread::sleep( core::time::Duration::from_millis( 500 ) );

  let entries : Vec< _ > = std::fs::read_dir( gate_dir.path() )
    .expect( "read gate dir" )
    .filter_map( Result::ok )
    .collect();

  let content = entries.first().map( |e| std::fs::read_to_string( e.path() ).unwrap_or_default() );

  let _ = child.kill();
  let _ = child.wait();
  let _ = occupier.kill();
  let _ = occupier.wait();

  assert_eq!( entries.len(), 1, "T07: expected exactly one gate state file to be written" );
  let content = content.expect( "T07: gate state file content" );
  assert!(
    serde_json::from_str::< serde_json::Value >( &content ).is_ok(),
    "T07 (BUG-384): gate state file must be valid JSON when cwd contains a quote. Got:\n{content}"
  );
  assert!(
    content.contains( "needs\\\"quote" ),
    "T07 (BUG-384): escaped quote must appear in the JSON cwd field. Got:\n{content}"
  );
}

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

// ── T08: N concurrent live `clr` invocations racing a shared, mutating occupier
//         set never admit more than --max-sessions at once (BUG-387) ──────────

/// Compile a tiny real ELF binary named `claude` that ignores all argv and sleeps
/// for `sleep_secs` seconds before exiting.
///
/// Needed because neither existing fake-`claude` fixture fits this test: a
/// shebang shell script (`fake_claude_dir`) shows its *interpreter* as argv[0]
/// in `/proc/{pid}/cmdline`, making it invisible to `find_claude_processes()`'s
/// basename check; and `/bin/sleep` (`fake_claude_binary_dir`) errors out
/// immediately on the non-numeric flags `clr` itself forwards to the dispatched
/// `claude` process (e.g. `-p`). This binary is a real ELF (so the basename
/// check passes) that never inspects `std::env::args()` at all (so it tolerates
/// whatever `clr` forwards) and blocks for a fixed duration (so concurrently
/// racing invocations have an observable overlap window).
///
/// Returns `(TempDir, path_val)` — `path_val` prepends the dir to `$PATH`,
/// mirroring `fake_claude_binary_dir()`'s contract.
///
/// # Panics
/// Panics if `rustc` is unavailable on `$PATH` or compilation fails.
fn build_argv_tolerant_sleeper( sleep_secs : u64 ) -> ( tempfile::TempDir, String )
{
  let dir = tempfile::TempDir::new().expect( "tmpdir" );
  let src = dir.path().join( "sleeper.rs" );
  std::fs::write(
    &src,
    format!( "fn main() {{ std::thread::sleep(std::time::Duration::from_secs({sleep_secs})); }}" ),
  ).expect( "write sleeper source" );
  let bin = dir.path().join( "claude" );
  let status = Command::new( "rustc" )
    .arg( "-O" )
    .arg( "-o" ).arg( &bin )
    .arg( &src )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .status()
    .expect( "invoke rustc for T08 fixture" );
  assert!( status.success(), "T08 fixture: rustc failed to compile the argv-tolerant sleeper" );
  let path_val = format!( "{}:{}", dir.path().display(), std::env::var( "PATH" ).unwrap_or_default() );
  ( dir, path_val )
}

/// Mirror each PID in `clr_pids`'s direct children (per `/proc/{pid}/task/{pid}/children`)
/// into `proc_dir` as a `/proc/{child}` symlink, polling every 5ms for up to `duration`.
///
/// This is what makes the synthetic `CLR_PROC_DIR` "dynamically mutating" rather
/// than a static pre-launch snapshot (BUG-387's own Prevention note) — each
/// racing `clr` invocation's own spawned `claude` child becomes visible to
/// `find_claude_processes()` shortly after it actually spawns, exactly as it
/// would against the real `/proc` outside a test. Scoped to only `clr_pids`'
/// direct children (not a blind host-wide `claude`-basename scan) so it cannot
/// pick up an unrelated process from another test binary running concurrently
/// under nextest.
fn sync_children_into_proc_dir( clr_pids : &[ u32 ], proc_dir : &std::path::Path, duration : core::time::Duration )
{
  let deadline = std::time::Instant::now() + duration;
  let mut known : std::collections::HashSet< u32 > = std::collections::HashSet::new();
  while std::time::Instant::now() < deadline
  {
    for &parent in clr_pids
    {
      let Ok( raw ) = std::fs::read_to_string( format!( "/proc/{parent}/task/{parent}/children" ) )
      else { continue; };
      for child_pid in raw.split_whitespace().filter_map( |t| t.parse::< u32 >().ok() )
      {
        if known.insert( child_pid )
        {
          let _ = std::os::unix::fs::symlink(
            format!( "/proc/{child_pid}" ),
            proc_dir.join( child_pid.to_string() ),
          );
        }
      }
    }
    std::thread::sleep( core::time::Duration::from_millis( 5 ) );
  }
}

/// Extract the `pid` field from a slot-reservation file's JSON content
/// (`{"pid":N,"since":M}`), written by `claim_slot_file()` in `src/cli/gate.rs`.
fn slot_owner_pid( content : &str ) -> Option< u32 >
{
  let marker = "\"pid\":";
  let start  = content.find( marker )? + marker.len();
  let rest   = &content[ start.. ];
  let end    = rest.find( [ ',', '}' ] )?;
  rest[ ..end ].trim().parse().ok()
}

/// Count how many `slot_*.json` files in `gate_dir` are currently held by a
/// live process — mirrors the exact liveness convention `build_queued_table()`
/// already applies to `GateFile` orphans in `ps.rs`, so a slot left behind by
/// an already-exited racer is never miscounted as still held.
fn count_live_held_slots( gate_dir : &std::path::Path ) -> usize
{
  std::fs::read_dir( gate_dir )
    .map_or( 0, |it| it.flatten().filter( |e|
    {
      let is_slot = e.path().file_stem()
        .and_then( |s| s.to_str() )
        .is_some_and( |s| s.starts_with( "slot_" ) );
      if !is_slot { return false; }
      let content = std::fs::read_to_string( e.path() ).unwrap_or_default();
      slot_owner_pid( &content )
        .is_some_and( |pid| std::path::Path::new( &format!( "/proc/{pid}" ) ).exists() )
    } ).count() )
}

/// T08 (BUG-387): launches 8 real `clr` print-mode invocations concurrently,
/// sharing one `CLR_GATE_DIR` and one `CLR_PROC_DIR`, with `--max-sessions 3`.
/// A background thread mirrors each racer's real spawned `claude` child into
/// the shared proc dir as it appears (`sync_children_into_proc_dir`) so the
/// gate's live-process count actually varies during the burst, unlike T01-T07's
/// static snapshots. Samples the shared gate dir's live-held slot count at
/// short intervals throughout the burst and asserts the peak never exceeds the
/// configured limit — the property the check-then-act race
/// (`task/bug/387_print_mode_concurrency_gate_toctou_race.md`) could previously
/// violate silently.
#[ test ]
fn t08_concurrent_clr_invocations_never_exceed_max_sessions()
{
  const N   : usize = 8;
  const MAX : u32   = 3;

  let ( _bin_dir, bin_path ) = build_argv_tolerant_sleeper( 3 );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir = tempfile::TempDir::new().expect( "proc dir" );

  let mut children : Vec< std::process::Child > = ( 0..N ).map( | i |
  {
    Command::new( env!( "CARGO_BIN_EXE_clr" ) )
      .args( [ "-p", "--max-sessions", &MAX.to_string(), "--journal", "off", &format!( "race-{i}" ) ] )
      .env( "PATH", &bin_path )
      .env( "CLR_PROC_DIR", proc_dir.path() )
      .env( "CLR_GATE_DIR", gate_dir.path() )
      .env( "_CLR_GATE_POLL_SECS", "1" )
      .stdout( std::process::Stdio::null() )
      .stderr( std::process::Stdio::null() )
      .spawn()
      .expect( "spawn racing clr" )
  } ).collect();

  let clr_pids : Vec< u32 > = children.iter().map( std::process::Child::id ).collect();

  let sync_proc_dir = proc_dir.path().to_path_buf();
  let sync_pids     = clr_pids.clone();
  let sync_handle = std::thread::spawn( move ||
  {
    sync_children_into_proc_dir( &sync_pids, &sync_proc_dir, core::time::Duration::from_secs( 8 ) );
  } );

  let mut peak = 0usize;
  let sample_deadline = std::time::Instant::now() + core::time::Duration::from_secs( 8 );
  while std::time::Instant::now() < sample_deadline
  {
    peak = peak.max( count_live_held_slots( gate_dir.path() ) );
    std::thread::sleep( core::time::Duration::from_millis( 20 ) );
  }

  for child in &mut children { let _ = child.wait(); }
  let _ = sync_handle.join();

  // Final sample after every racer has finished — catches a peak that only
  // occurred right at the tail end of the sampling window.
  peak = peak.max( count_live_held_slots( gate_dir.path() ) );

  assert!(
    peak <= MAX as usize,
    "T08 (BUG-387): peak concurrently-held slots ({peak}) must never exceed --max-sessions ({MAX})"
  );
}
