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
//! | T01 | 6 print-mode processes active, print invocation, default → gate triggers at 6 | T01 |
//! | T02 | 5 print-mode processes active, print invocation, default → gate does not trigger | T02 |
//! | T03 | 15 print-mode + 1 interactive active, interactive invocation → gate skipped, zero wait | T03 |
//! | T04 | 5 print-mode + 10 interactive active, print invocation, `--max-sessions 5` → print-mode-only count | T04 |
//! | T06 | `--max-sessions 0`, any process count → gate disabled, unchanged behavior | T06 |
//! | T07 | gate state file `cwd` field remains valid JSON when cwd contains a literal `"` (BUG-384) | — |
//! | T08 | N concurrent live `clr` invocations racing a shared, dynamically-mutating occupier set → peak admitted count never exceeds `--max-sessions` (BUG-387) | — |
//! | T13 | gate state file `cwd` field remains valid JSON when cwd contains raw control characters (BEL, tab), not just `"` (BUG-384 residual) | — |
//! | T09 | `CLR_GATE_POLL_SECS=1 CLR_GATE_MAX_ATTEMPTS=2` + `--retry-override 0`, 1 permanent occupier → both overrides change real timing; exhausts in ~2s with the exact `[Runner]` message | — |
//! | T10 | `CLR_GATE_POLL_SECS=notanumber` (+ valid `CLR_GATE_MAX_ATTEMPTS=2`, `--retry-override 0`) → invalid value silently falls back to the 30s default | — |
//! | T11 | `CLR_GATE_MAX_ATTEMPTS=notanumber` (+ valid `CLR_GATE_POLL_SECS=1`) → invalid value silently falls back to the 1000-attempt default | — |
//! | T14 | N concurrent live `clr` invocations racing a single pre-seeded dead-owner slot → peak concurrently-admitted children never exceeds 1 (BUG-392) | — |
//! | T15 | 2 racers, `--max-sessions 1`, 0 pre-existing occupiers → loser's wait message names "slot held by another session", not "at capacity" or a reclaim race (BUG-393/BUG-396) | — |
//! | T16 | 2 racers, `--max-sessions 1`, pre-seeded confirmed-dead owner → loser's wait message names "lost reservation race", not "at capacity" or a live hold (BUG-396) | — |
//! | T17 | pre-seeded dead-owner slot + its own orphaned reclaim ticket (ticket's recorded claimant also dead), single caller, no live contender → still admitted promptly, not permanently blocked (BUG-402) | — |
//! | T18 | `--max-sessions 2`, count-derived index (1) pre-seeded genuinely `HeldByLive`, other index (0) left completely free → still admitted promptly via fallback scan, not gate-wait exhaustion (BUG-404) | — |
//!
//! T05 (`clr --help` shows `default: 6`) is covered by
//! `param_edge_cases_test.rs::ec9_max_sessions_help_shows_default_six`.
//!
//! T12 (regression: pre-existing T01/T02/T04/T08 still pass using the renamed
//! `CLR_GATE_POLL_SECS` var) is covered by those same tests post-rename — no
//! separate function.

// BUG-387 — T01-T07 above all pre-seed a static synthetic /proc snapshot and
// invoke exactly one clr binary; none launch N concurrent clr invocations
// racing each other against a shared, mutating occupier set, so none could
// exercise the check-then-spawn TOCTOU race. T08 below closes that gap: it
// launches N concurrent live `clr` invocations and asserts peak
// simultaneously-admitted count never exceeds --max-sessions.

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

// ── T13: gate state file stays valid JSON when cwd contains control chars (BUG-384) ──

/// T13 (BUG-384 residual): the gate-state file's `cwd` field must be JSON-escaped for
/// raw control characters, not just `"` and `\`. Forces the gate to actually block
/// (`--max-sessions 1` against a single active print-mode occupier) from a
/// `current_dir` containing a literal BEL (`\u{07}`, no named JSON escape — must fall
/// back to `\u00XX`) and a literal tab (`\t`, a named JSON escape), then reads the
/// resulting `$CLR_GATE_DIR/{pid}.json` file directly and asserts it parses as valid
/// JSON. Prior to this fix, the gate only escaped `"` and `\` via chained `.replace()`
/// calls, so an embedded raw control byte (legal in a Unix path) produced invalid JSON
/// (RFC 8259 §7 forbids literal control bytes in a JSON string).
#[ test ]
fn t13_gate_state_file_valid_json_for_control_char_cwd()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();

  let mut occupier = spawn_print_claude( &occupier_path );
  let proc = make_proc_dir( &[ occupier.id() ] );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );

  let control_cwd_parent = tempfile::TempDir::new().expect( "control-char cwd parent" );
  let control_cwd = control_cwd_parent.path().join( "needs\u{07}control\tchar" );
  std::fs::create_dir_all( &control_cwd ).expect( "create control-char cwd" );

  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut child = Command::new( bin )
    .args( [ "-p", "--max-sessions", "1", "--journal", "off", "x" ] )
    .current_dir( &control_cwd )
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

  assert_eq!( entries.len(), 1, "T13: expected exactly one gate state file to be written" );
  let content = content.expect( "T13: gate state file content" );
  assert!(
    serde_json::from_str::< serde_json::Value >( &content ).is_ok(),
    "T13 (BUG-384): gate state file must be valid JSON when cwd contains raw control characters. Got:\n{content}"
  );
  assert!(
    content.contains( "needs\\u0007control\\tchar" ),
    "T13 (BUG-384): escaped BEL (\\u0007) and tab (\\t) must appear in the JSON cwd field. Got:\n{content}"
  );
}

// ── T01: gate triggers at exactly 6 print-mode processes (default limit) ───────

/// T01: 6 print-mode processes active (5 long-lived + 1 short-lived), new print-mode
/// invocation, `--max-sessions` unset (default 6) → gate triggers and emits the
/// "6/6 sessions active; waiting" message, then releases once the short-lived
/// process self-expires and the count drops below 6.
#[ test ]
fn t01_gate_triggers_at_six_print_mode_processes()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();

  let mut long_lived : Vec< std::process::Child > =
    ( 0..5 ).map( |_| spawn_print_claude( &occupier_path ) ).collect();
  let mut short_lived = spawn_print_claude_for( &occupier_path, 5 );

  let mut pids : Vec< u32 > = long_lived.iter().map( std::process::Child::id ).collect();
  pids.push( short_lived.id() );
  let proc = make_proc_dir( &pids );

  // Dispatched command's own fake claude — fast, exits immediately once the gate releases.
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
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
    // Anchored on "Info: " so a wrong larger count (e.g. "Info: 16/6") can never
    // false-positive match via the "6/6" tail — AF1.
    stderr.contains( "Info: 6/6 sessions active; waiting" ),
    "T01: gate must report 6/6 print-mode sessions active. Got:\n{stderr}"
  );
}

// ── T02: gate does not trigger below the limit ──────────────────────────────────

/// T02: 5 print-mode processes active, new print-mode invocation, `--max-sessions`
/// unset (default 6) → gate does not trigger; the dispatched command proceeds
/// immediately with no wait message on stderr.
#[ test ]
fn t02_gate_does_not_trigger_below_six_print_mode_processes()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();

  let mut occupiers : Vec< std::process::Child > =
    ( 0..5 ).map( |_| spawn_print_claude( &occupier_path ) ).collect();
  let pids : Vec< u32 > = occupiers.iter().map( std::process::Child::id ).collect();
  let proc = make_proc_dir( &pids );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
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
/// `CLR_GATE_POLL_SECS` is deliberately left at its 30-second production default:
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
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let start = std::time::Instant::now();
  let out = Command::new( bin )
    .args( [ "--interactive", "--max-sessions", "1", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
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
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--max-sessions", "5", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
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
      .env( "CLR_GATE_POLL_SECS", "1" )
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

  // Fix(BUG-387 test): reap every racer via non-blocking, order-independent
  // `try_wait()` polling for the test's ENTIRE lifetime — both during sampling
  // and while draining stragglers afterward — never a sequential `.wait()`.
  //
  // Root cause (two compounding defects, both in this harness, not in gate.rs):
  // 1. A `clr` process that has exited but not yet been `wait()`-ed on is a
  //    zombie, and a zombie still has a `/proc/{pid}` entry — so `pid_alive()`
  //    (which `gate.rs::acquire_slot()` uses to decide whether a slot is
  //    reclaimable) sees an unreaped zombie as "still alive" indefinitely.
  // 2. A sequential `for child in &mut children { child.wait(); }` reaps in
  //    launch order. If an EARLY-indexed racer is itself still legitimately
  //    waiting for a slot (never admitted yet), `.wait()` on it blocks forever
  //    — so LATER-indexed racers that already exited are never reached by the
  //    loop and sit as permanent zombies, permanently blocking their own held
  //    slots (defect 1) from ever being reclaimed by the still-waiting racers.
  //    This head-of-line-blocking deadlock is only ever broken once the stuck
  //    racer exhausts `apply_runner_retry`'s default 2 retries (100 attempts ×
  //    1s + 30s backoff, twice) and calls `std::process::exit(1)` — explaining
  //    the exact, repeatable ~360s runtime observed before this fix.
  //
  // Fix: poll every child with `try_wait()` on the same 20ms cadence for as
  // long as ANY child remains unfinished, with no ordering dependency between
  // them, so a slot's owner is reaped within milliseconds of actually exiting
  // — matching how promptly a real shell reaps a foreground child — and a
  // bounded drain deadline + force-`kill()` safety net so a genuine regression
  // fails loudly (leftover process / assertion) instead of hanging the suite.
  //
  // Pitfall: any harness holding `Child` handles across a polling window must
  // reap them all on that same cadence and without sequential ordering, or it
  // silently reintroduces an artificial zombie-accumulation window with a
  // head-of-line-blocking deadlock that no real caller would ever hit.
  let mut peak = 0usize;
  let mut finished = vec![ false; children.len() ];
  let reap = | children : &mut [ std::process::Child ], finished : &mut [ bool ] |
  {
    for ( child, done ) in children.iter_mut().zip( finished.iter_mut() )
    {
      if !*done && matches!( child.try_wait(), Ok( Some( _ ) ) ) { *done = true; }
    }
  };

  let sample_deadline = std::time::Instant::now() + core::time::Duration::from_secs( 8 );
  while std::time::Instant::now() < sample_deadline
  {
    reap( &mut children, &mut finished );
    peak = peak.max( count_live_held_slots( gate_dir.path() ) );
    std::thread::sleep( core::time::Duration::from_millis( 20 ) );
  }

  let drain_deadline = std::time::Instant::now() + core::time::Duration::from_secs( 30 );
  while finished.iter().any( | done | !done ) && std::time::Instant::now() < drain_deadline
  {
    reap( &mut children, &mut finished );
    std::thread::sleep( core::time::Duration::from_millis( 20 ) );
  }
  for ( child, done ) in children.iter_mut().zip( finished.iter_mut() )
  {
    if !*done { let _ = child.kill(); let _ = child.wait(); }
  }
  let _ = sync_handle.join();

  // Final sample after every racer has finished — catches a peak that only
  // occurred right at the tail end of the sampling window.
  peak = peak.max( count_live_held_slots( gate_dir.path() ) );

  assert!(
    peak <= MAX as usize,
    "T08 (BUG-387): peak concurrently-held slots ({peak}) must never exceed --max-sessions ({MAX})"
  );
}

// ── T09-T11: `CLR_GATE_POLL_SECS`/`CLR_GATE_MAX_ATTEMPTS` env var overrides ────
// task/claude_runner/389_gate_poll_secs_max_attempts_env_vars.md

/// Poll `child` with `try_wait()` until it exits or `deadline` passes, sleeping
/// 50ms between checks. Never blocks past `deadline` — unlike `.output()`
/// (blocks until natural exit), this lets a test fail fast when the gate is
/// still reading pre-rename hardcoded defaults instead of hanging for however
/// long the real 30s/1000-attempt production values would otherwise take.
/// Mirrors T08's existing `try_wait()` reap-loop pattern in this same file.
fn wait_bounded( child : &mut std::process::Child, deadline : std::time::Instant ) -> Option< std::process::ExitStatus >
{
  while std::time::Instant::now() < deadline
  {
    if let Ok( Some( status ) ) = child.try_wait() { return Some( status ); }
    std::thread::sleep( core::time::Duration::from_millis( 50 ) );
  }
  None
}

/// T09: `CLR_GATE_POLL_SECS=1` and `CLR_GATE_MAX_ATTEMPTS=2` together must change
/// the gate's actual runtime behavior (not just documented intent). With one
/// print-mode occupier permanently holding the only `--max-sessions 1` slot and
/// `--retry-override 0` disabling the outer Runner-retry wrapper, the gate must
/// exhaust after exactly 2 polls at 1-second intervals (~2s total) — not the
/// production default of 1000 attempts × 30s (~8.3h) — and report the exact
/// exhaustion message on stderr. Bounded to a 10s deadline: if gate.rs still
/// reads the pre-Phase-1 hardcoded defaults, neither override takes effect and
/// this deadline elapses long before natural exit, failing fast.
///
/// Source: `task/claude_runner/389_gate_poll_secs_max_attempts_env_vars.md` T09, AC-009/AC-010.
#[ test ]
fn t09_gate_env_var_overrides_change_real_poll_timing()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  // T09's 10s deadline is well under spawn_print_claude()'s own 30s self-expiry
  // (spawn_print_claude_for(_, 30)) — but pin the lifetime explicitly rather
  // than rely on that margin, so this test never races the occupier's own exit.
  let mut occupier = spawn_print_claude_for( &occupier_path, 60 );
  let proc = make_proc_dir( &[ occupier.id() ] );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut child = Command::new( bin )
    .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .expect( "spawn clr" );

  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 10 );
  let exited = wait_bounded( &mut child, deadline );
  if exited.is_none() { let _ = child.kill(); }
  let out = child.wait_with_output().expect( "reap clr" );

  let _ = occupier.kill();
  let _ = occupier.wait();

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    exited.is_some(),
    "T09: gate must exhaust within 10s when both overrides are active (2 attempts x 1s poll) \
     — still running means the overrides are not taking effect. stderr:\n{stderr}"
  );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 1 ),
    "T09: exit must be 1 once the gate exhausts. stderr: {stderr}"
  );
  assert!(
    stderr.contains(
      "Error: [Runner] session gate timed out — 1 active sessions, max-sessions=1 — retries exhausted (exit 1)"
    ),
    "T09: exact exhaustion message required. Got:\n{stderr}"
  );
}

/// T10: `CLR_GATE_POLL_SECS=notanumber` must not panic or surface any error about
/// the env var itself — it silently falls back to the 30-second production
/// default. Paired with a valid, small `CLR_GATE_MAX_ATTEMPTS=2` and
/// `--retry-override 0` so the gate reaches exhaustion after exactly one 30s
/// poll instead of the full 1000-attempt production ceiling — bounding the wait
/// to ~30-33s (confirmed via the 40s deadline) rather than the ~8.3 real hours
/// a literal 1000-attempt run at the true 30s interval would otherwise take,
/// while still genuinely measuring the fallback poll interval via both the
/// gate's own stderr message and wall-clock elapsed time.
///
/// Source: `task/claude_runner/389_gate_poll_secs_max_attempts_env_vars.md` T10, AC-009.
#[ test ]
fn t10_invalid_poll_secs_env_var_falls_back_to_default()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  // Fix(test bug found during Phase 1 validation): spawn_print_claude() is a thin
  // wrapper over spawn_print_claude_for(_, 30) — it self-expires at 30s, which
  // collides with this test's ~30-33s expected exhaustion time (one real 30s
  // poll sleep). A permanent-looking occupier that dies right as attempt 2's
  // check runs intermittently frees the slot, making the gate admit (exit 0)
  // instead of exhaust (exit 1). Pin the lifetime past the 40s deadline instead.
  let mut occupier = spawn_print_claude_for( &occupier_path, 60 );
  let proc = make_proc_dir( &[ occupier.id() ] );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut child = Command::new( bin )
    .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "notanumber" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .expect( "spawn clr" );

  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 40 );
  let exited = wait_bounded( &mut child, deadline );
  if exited.is_none() { let _ = child.kill(); }
  let out = child.wait_with_output().expect( "reap clr" );

  let _ = occupier.kill();
  let _ = occupier.wait();

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    exited.is_some(),
    "T10: gate must exhaust within 40s when CLR_GATE_MAX_ATTEMPTS=2 is active \
     — still running means the override is not taking effect. stderr:\n{stderr}"
  );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 1 ),
    "T10: exit must be 1 once the gate exhausts. stderr: {stderr}"
  );
  assert!(
    stderr.contains( "waiting 30s for a slot" ),
    "T10: invalid CLR_GATE_POLL_SECS must fall back to the 30s default. Got:\n{stderr}"
  );
  assert!(
    !stderr.to_lowercase().contains( "panic" ),
    "T10: invalid value must fail silently — no panic. Got:\n{stderr}"
  );
}

/// T11: `CLR_GATE_MAX_ATTEMPTS=notanumber` must not panic or surface any error
/// about the env var itself — it silently falls back to the 1000-attempt
/// production default. Paired with a valid `CLR_GATE_POLL_SECS=1` and a
/// short-lived occupier (releases after ~3s): once genuinely active, the 1s
/// poll interval admits within ~10s of the occupier releasing. Bounded to a
/// 10s deadline — if gate.rs still reads the pre-Phase-1 hardcoded 30s poll
/// interval, `CLR_GATE_POLL_SECS=1` has no effect and the first re-check after
/// the occupier releases doesn't happen until a real 30s sleep elapses, well
/// past this deadline, failing fast instead of hanging.
///
/// Source: `task/claude_runner/389_gate_poll_secs_max_attempts_env_vars.md` T11, AC-010.
#[ test ]
fn t11_invalid_max_attempts_env_var_falls_back_to_default()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  let mut occupier = spawn_print_claude_for( &occupier_path, 3 );
  let proc = make_proc_dir( &[ occupier.id() ] );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut child = Command::new( bin )
    .args( [ "-p", "--max-sessions", "1", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "notanumber" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .expect( "spawn clr" );

  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 10 );
  let exited = wait_bounded( &mut child, deadline );
  if exited.is_none() { let _ = child.kill(); }
  let out = child.wait_with_output().expect( "reap clr" );

  let _ = occupier.kill();
  let _ = occupier.wait();

  assert!(
    exited.is_some(),
    "T11: gate must admit within 10s once the occupier releases — CLR_GATE_POLL_SECS=1 \
     must take effect regardless of the invalid CLR_GATE_MAX_ATTEMPTS value. stderr:\n{}",
    String::from_utf8_lossy( &out.stderr )
  );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 0 ),
    "T11: exit must be 0 once the gate admits. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.to_lowercase().contains( "panic" ),
    "T11: invalid CLR_GATE_MAX_ATTEMPTS must fail silently, no panic. Got:\n{stderr}"
  );
}

// ── T14: reclaim-branch race admits at most one caller for a dead owner's
//         slot (BUG-392, residual of BUG-387) ──────────────────────────────
// BUG-392 — acquire_slot()'s dead-owner reclaim branch was non-atomic (TOCTOU)

/// T14 (BUG-392): pre-seeds `gate_dir` with a slot file owned by a PID
/// confirmed dead (a real process, spawned then reaped, so `/proc/{pid}` is
/// genuinely absent — not a made-up number), keeps `CLR_PROC_DIR` permanently
/// empty so every racer's live print-mode count reads 0 for the entire run —
/// forcing all racers toward the SAME index-0 reclaim rather than T08's
/// fresh-claim path — then launches 8 concurrent `clr` racers with
/// `--max-sessions 1` against it. Tracks each racer's own dispatched child (a
/// slow argv-tolerant sleeper) directly via
/// `/proc/{clr_pid}/task/{clr_pid}/children`, independent of `CLR_PROC_DIR`,
/// and samples how many are alive at once, asserting the peak never exceeds
/// 1 — the exact invariant the pre-fix `remove_file()` + `claim_slot_file()`
/// reclaim sequence in `acquire_slot()` could violate.
///
/// Root Cause: `acquire_slot()`'s reclaim branch treated "the previous owner
/// is dead" as a fact stable across two subsequent, independently-fallible
/// I/O calls (`remove_file()` then `claim_slot_file()`), with no
/// synchronization between racers who observed the identical dead-owner
/// record. `remove_file()` unconditionally unlinks whatever currently
/// occupies the path, so a second racer's `remove_file()` could delete a
/// first racer's freshly-reclaimed file out from under it — both then
/// returned `true` for the same index.
///
/// Why Not Caught: T08 (the existing concurrency regression test, added by
/// BUG-387's own fix) exercises the gate exclusively via live, healthy
/// occupier processes — it never constructs a slot file whose recorded owner
/// has actually died before a second caller races the reclaim. The
/// crash-recovery reclaim branch this test targets was entirely unexercised
/// by the existing suite.
///
/// Fix Applied: `acquire_slot()`'s reclaim branch now gates the actual
/// remove/recreate behind its own atomic arbitration — a ticket file keyed
/// by (index, dead owner pid, dead owner since), claimed via the same
/// `create_new` atomicity already used for the fresh-claim path — so exactly
/// one racer wins the right to reclaim. Only the winner writes to the
/// original slot path, via `rename()` from a per-caller-unique temp file
/// (atomic replace, no observably-absent gap). See `Fix(BUG-392)` on
/// `acquire_slot()` in `src/cli/gate.rs` for the full explanation.
///
/// Prevention: this test — asserts peak concurrently-alive dispatched
/// children sharing one contested, dead-owned slot never exceeds 1, under
/// genuine concurrent OS scheduling with 8 real racing `clr` processes.
///
/// Pitfall: a test asserting this property must never reuse
/// `count_live_held_slots()` (defined above for T08) — it treats ANY file
/// whose stem starts with `slot_` as a held slot regardless of extension.
/// The fix's ticket and temp files are deliberately named with a `reclaim_`
/// prefix instead (never `slot_`) for exactly this reason: an earlier
/// revision of this fix used a `slot_`-prefixed name for both, and while
/// that stayed invisible to `ps.rs::build_queued_table()` (which filters on
/// the `.json` extension first), `count_live_held_slots()` has no such
/// extension check — it counted the ticket and temp files as extra held
/// slots for the brief window they existed, intermittently failing T08 with
/// an inflated peak even though only one session was genuinely admitted.
/// This test sidesteps the whole class of helper-miscount risk by tracking
/// each racer's own real OS child process directly instead.
#[ test ]
fn t14_reclaim_race_admits_at_most_one_caller_for_a_dead_owners_slot()
{
  const N : usize = 8;

  let ( _bin_dir, bin_path ) = build_argv_tolerant_sleeper( 3 );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir = tempfile::TempDir::new().expect( "proc dir" ); // deliberately static/empty

  // Pre-seed a slot file owned by a definitely-dead PID: spawn a real,
  // immediately-exiting process and reap it, so /proc/{dead_pid} is confirmed
  // absent from this point on — a real crash-recovery precondition rather
  // than a made-up PID number.
  let mut dead = Command::new( "true" ).spawn().expect( "spawn short-lived process" );
  let dead_pid = dead.id();
  let _ = dead.wait();
  std::fs::write(
    gate_dir.path().join( "slot_0.json" ),
    format!( r#"{{"pid":{dead_pid},"since":0}}"# ),
  ).expect( "pre-seed dead-owner slot file" );

  let mut children : Vec< std::process::Child > = ( 0..N ).map( | i |
  {
    Command::new( env!( "CARGO_BIN_EXE_clr" ) )
      .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", &format!( "race-{i}" ) ] )
      .env( "PATH", &bin_path )
      .env( "CLR_PROC_DIR", proc_dir.path() )
      .env( "CLR_GATE_DIR", gate_dir.path() )
      .env( "CLR_GATE_POLL_SECS", "1" )
      .env( "CLR_GATE_MAX_ATTEMPTS", "5" )
      // Widen the reclaim race window deterministically (see reclaim_test_delay()
      // in gate.rs) so this test forces genuine contention on every run instead
      // of depending on incidental OS scheduling jitter between racers.
      .env( "CLR_GATE_RECLAIM_TEST_DELAY_MS", "50" )
      .stdout( std::process::Stdio::null() )
      .stderr( std::process::Stdio::null() )
      .spawn()
      .expect( "spawn racing clr" )
  } ).collect();

  let clr_pids : Vec< u32 > = children.iter().map( std::process::Child::id ).collect();

  // Independent of CLR_PROC_DIR (which stays empty throughout — see doc
  // comment above): track each racer's own dispatched child directly, so an
  // over-admission shows up as 2+ concurrently-alive children regardless of
  // what the gate's own (deliberately blinded) live-count read believes.
  let mut known_children : std::collections::HashSet< u32 > = std::collections::HashSet::new();
  let mut peak = 0usize;
  let mut finished = vec![ false; children.len() ];
  let reap = | children : &mut [ std::process::Child ], finished : &mut [ bool ] |
  {
    for ( child, done ) in children.iter_mut().zip( finished.iter_mut() )
    {
      if !*done && matches!( child.try_wait(), Ok( Some( _ ) ) ) { *done = true; }
    }
  };

  let sample_deadline = std::time::Instant::now() + core::time::Duration::from_secs( 10 );
  while std::time::Instant::now() < sample_deadline
  {
    reap( &mut children, &mut finished );
    for &parent in &clr_pids
    {
      if let Ok( raw ) = std::fs::read_to_string( format!( "/proc/{parent}/task/{parent}/children" ) )
      {
        for child_pid in raw.split_whitespace().filter_map( |t| t.parse::< u32 >().ok() )
        {
          known_children.insert( child_pid );
        }
      }
    }
    let live_now = known_children.iter()
      .filter( |&&pid| std::path::Path::new( &format!( "/proc/{pid}" ) ).exists() )
      .count();
    peak = peak.max( live_now );
    std::thread::sleep( core::time::Duration::from_millis( 20 ) );
  }

  let drain_deadline = std::time::Instant::now() + core::time::Duration::from_secs( 30 );
  while finished.iter().any( | done | !done ) && std::time::Instant::now() < drain_deadline
  {
    reap( &mut children, &mut finished );
    std::thread::sleep( core::time::Duration::from_millis( 20 ) );
  }
  for ( child, done ) in children.iter_mut().zip( finished.iter_mut() )
  {
    if !*done { let _ = child.kill(); let _ = child.wait(); }
  }

  assert!(
    peak <= 1,
    "T14 (BUG-392): peak concurrently-alive dispatched children sharing one \
     contested dead-owner slot ({peak}) must never exceed 1 — acquire_slot()'s \
     reclaim branch admitted more than one caller for the same slot"
  );
}

// ── T15: slot-wait message names a live hold, not a race (BUG-393/BUG-396) ───

/// T15 (BUG-393, corrected by BUG-396): races exactly 2 concurrent `clr`
/// invocations against `--max-sessions 1` with zero pre-existing occupiers,
/// so both racers read `count_u32 = 0 < max = 1` on their very first attempt
/// and contend for the same reservation index. Captures both racers' stderr
/// directly (not `Stdio::null()`, unlike T08/T14) and asserts the losing
/// racer's message names the cause as the slot being held by another
/// session, and that neither racer's message claims capacity exhaustion or a
/// reclaim race.
///
/// ## Root Cause
/// `wait_for_session_slot()`'s admission check at `gate.rs:334` is a compound
/// condition, `has_capacity && acquire_slot(...)`. Originally (BUG-393) this
/// test's docs assumed the losing racer's non-admission was itself a "race" —
/// but the loser's `pid_alive(owner)` check observes the WINNING racer's own
/// PID, and that racer remains present in `/proc` (at minimum as a zombie,
/// since this test's harness does not reap either racer until the 2s
/// deadline below) for the entire observation window. The loser therefore
/// always takes `acquire_slot()`'s `HeldByLive` branch — it never contends
/// for anything, because it never even attempts a reclaim; the winner's slot
/// is simply, unambiguously "held by a live session" from the loser's very
/// first check onward. See BUG-396 for the genuine reclaim-race scenario
/// (T16), which requires a pre-seeded, CONFIRMED-dead owner instead.
///
/// ## Why Not Caught
/// BUG-393's own fix shipped with this test asserting `"lost reservation
/// race"` for the loser, and it passed — because at the time, `acquire_slot()`
/// collapsed `HeldByLive` and `LostReclaimRace` into the same bare `false`,
/// so both this test's scenario AND a genuine reclaim race produced
/// identical output. The mislabeling was only exposed by production
/// evidence (job #40: `has_capacity` true, message claimed a "race", but the
/// recorded slot owner was directly confirmed alive via `/proc` — no reclaim
/// was ever attempted).
///
/// ## Fix Applied
/// `acquire_slot()` now returns `Result<(), SlotDenialCause>` with
/// `HeldByLive` and `LostReclaimRace` as distinct variants (`gate.rs`);
/// `wait_for_session_slot()` matches on the variant to choose the cause
/// suffix, rather than collapsing every non-admission under `has_capacity`
/// into one label.
///
/// ## Prevention
/// A test asserting on `acquire_slot()`'s denial-cause message must confirm
/// which branch it actually exercises by reasoning about `pid_alive()`'s
/// observable inputs (is the checked PID a fresh racer that may still be a
/// zombie, or a genuinely pre-reaped dead process?), not by assuming
/// "not admitted" implies any particular cause.
///
/// ## Pitfall
/// Do not reuse T08/T14's 8-racer, compiled-sleeper-binary infrastructure for
/// this — it is sized for peak-concurrency sampling, not message-content
/// capture, and switching its uniform `Stdio::null()` to per-child `piped()`
/// would need individual incremental reads threaded through its careful
/// non-blocking reap loop. A minimal 2-racer, `--max-sessions 1` fixture
/// isolates the same branches with far less moving parts.
// test_kind: bug_reproducer(BUG-393)
#[ test ]
fn t15_slot_wait_message_names_live_hold_when_owner_alive()
{
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir = tempfile::TempDir::new().expect( "proc dir" ); // empty: 0 pre-existing occupiers

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let spawn_racer = | label : &str |
  {
    Command::new( bin )
      .args( [ "-p", "--max-sessions", "1", "--journal", "off", label ] )
      .env( "PATH", &script_path )
      .env( "CLR_PROC_DIR", proc_dir.path() )
      .env( "CLR_GATE_DIR", gate_dir.path() )
      .env( "CLR_GATE_POLL_SECS", "1" )
      .env( "CLR_GATE_MAX_ATTEMPTS", "5" )
      .stdout( std::process::Stdio::null() )
      .stderr( std::process::Stdio::piped() )
      .spawn()
      .expect( "spawn racing clr" )
  };

  let mut racer_a = spawn_racer( "race-a" );
  let mut racer_b = spawn_racer( "race-b" );

  // Both racers read count_u32=0 < max=1 on attempt 1 and contend for the same
  // reservation index; the loser's message prints immediately (no delay before
  // the first poll's eprintln). 2s is a generous margin before either racer
  // could reach CLR_GATE_MAX_ATTEMPTS=5's own timeout/retry path, and is also
  // why the winner is still present in `/proc` (unreaped by this harness) for
  // the loser's entire observation window — see `## Root Cause` above.
  std::thread::sleep( core::time::Duration::from_secs( 2 ) );
  let _ = racer_a.kill();
  let _ = racer_b.kill();
  let out_a = racer_a.wait_with_output().expect( "reap racer a" );
  let out_b = racer_b.wait_with_output().expect( "reap racer b" );

  let stderr_a = String::from_utf8_lossy( &out_a.stderr );
  let stderr_b = String::from_utf8_lossy( &out_b.stderr );

  let a_held = stderr_a.contains( "slot held by another session" );
  let b_held = stderr_b.contains( "slot held by another session" );
  assert!(
    a_held != b_held,
    "T15 (BUG-393/396): exactly one racer must report the slot held by \
     another (live) session. stderr_a:\n{stderr_a}\nstderr_b:\n{stderr_b}"
  );
  assert!(
    !stderr_a.contains( "at capacity" ) && !stderr_b.contains( "at capacity" ),
    "T15 (BUG-393/396): neither racer should report capacity exhaustion when both \
     read count_u32=0 < max=1 on the racing attempt. stderr_a:\n{stderr_a}\n\
     stderr_b:\n{stderr_b}"
  );
  assert!(
    !stderr_a.contains( "lost reservation race" ) && !stderr_b.contains( "lost reservation race" ),
    "T15 (BUG-393/396): neither racer should claim a reclaim race — the loser \
     never attempts a reclaim because the observed owner (the winning racer) \
     is alive. stderr_a:\n{stderr_a}\nstderr_b:\n{stderr_b}"
  );
}

// ── T16: slot-wait message names a genuine reclaim race (BUG-396) ──────────

/// T16 (BUG-396): races exactly 2 concurrent `clr` invocations against a
/// pre-seeded, CONFIRMED-dead slot owner — mirroring T14's dead-owner
/// technique: a real short-lived process is spawned and reaped so
/// `/proc/{dead_pid}` is genuinely absent, not a lingering zombie — with
/// `--max-sessions 1` so `has_capacity` is true for both racers on every
/// attempt (`CLR_PROC_DIR` stays empty and static throughout). Both racers
/// observe the identical dead owner and both attempt the reclaim-ticket
/// path; exactly one wins the ticket (admitted via reclaim, no wait message
/// at all), the other loses the ticket race. Captures the loser's stderr
/// directly and asserts it names the cause as a lost reservation race — the
/// one `acquire_slot()` code path where that label is actually accurate —
/// and that it does NOT claim capacity exhaustion or a live-held slot,
/// distinguishing it from T15's scenario.
///
/// ## Root Cause
/// See `Fix(BUG-396)` on `acquire_slot()`/`SlotDenialCause` in `gate.rs`:
/// prior to that fix, `HeldByLive` and `LostReclaimRace` were both a bare
/// `false`, so the message site could not tell them apart.
///
/// ## Why Not Caught
/// T15 was believed to already cover "lost reservation race", but its
/// 2-fresh-racer fixture never exercises the reclaim-ticket path at all (see
/// T15's `## Root Cause`, above) — it always takes the `HeldByLive` branch.
/// T14 exercises the true dead-owner reclaim path but discards every
/// racer's stderr via `Stdio::null()`, so no test asserted on this specific
/// message content before now.
///
/// ## Fix Applied
/// `acquire_slot()` now returns `Result<(), SlotDenialCause>`;
/// `wait_for_session_slot()` matches `SlotDenialCause::LostReclaimRace`
/// specifically for this path, separately from `HeldByLive`.
///
/// ## Prevention
/// Any test claiming to cover "lost reservation race" must pre-seed a
/// confirmed-dead owner (spawn + reap a real process, as T14 and this test
/// do) rather than relying on two fresh racers racing an empty path.
///
/// ## Pitfall
/// Do not assume `acquire_slot()` returning an `Err` means a race occurred —
/// verify which `SlotDenialCause` variant fired. Only `LostReclaimRace`
/// involves any actual contention; `HeldByLive` is simply "someone else has
/// this index," which may be a session that started microseconds or hours
/// ago — the code cannot tell, and the message must not claim it can.
// test_kind: bug_reproducer(BUG-396)
#[ test ]
fn t16_slot_wait_message_names_genuine_reclaim_race_for_dead_owner()
{
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir = tempfile::TempDir::new().expect( "proc dir" ); // empty: 0 counted occupiers throughout

  // Pre-seed a slot file owned by a definitely-dead PID, mirroring T14: spawn
  // a real, immediately-exiting process and reap it so `/proc/{dead_pid}` is
  // confirmed absent (not a lingering zombie) from this point on.
  let mut dead = Command::new( "true" ).spawn().expect( "spawn short-lived process" );
  let dead_pid = dead.id();
  let _ = dead.wait();
  std::fs::write(
    gate_dir.path().join( "slot_0.json" ),
    format!( r#"{{"pid":{dead_pid},"since":0}}"# ),
  ).expect( "pre-seed dead-owner slot file" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let spawn_racer = | label : &str |
  {
    Command::new( bin )
      // count_u32 stays 0 throughout (proc_dir is empty and static), so both
      // racers read has_capacity=true and target index 0 — the pre-seeded
      // dead-owner slot — on every attempt.
      .args( [ "-p", "--max-sessions", "1", "--journal", "off", label ] )
      .env( "PATH", &script_path )
      .env( "CLR_PROC_DIR", proc_dir.path() )
      .env( "CLR_GATE_DIR", gate_dir.path() )
      .env( "CLR_GATE_POLL_SECS", "1" )
      .env( "CLR_GATE_MAX_ATTEMPTS", "5" )
      // Widen the reclaim race window deterministically (see
      // reclaim_test_delay() in gate.rs), matching T14, so this test forces
      // genuine ticket contention on every run instead of depending on
      // incidental OS scheduling jitter between the two racers.
      .env( "CLR_GATE_RECLAIM_TEST_DELAY_MS", "50" )
      .stdout( std::process::Stdio::null() )
      .stderr( std::process::Stdio::piped() )
      .spawn()
      .expect( "spawn racing clr" )
  };

  let mut racer_a = spawn_racer( "race-a" );
  let mut racer_b = spawn_racer( "race-b" );

  std::thread::sleep( core::time::Duration::from_secs( 2 ) );
  let _ = racer_a.kill();
  let _ = racer_b.kill();
  let out_a = racer_a.wait_with_output().expect( "reap racer a" );
  let out_b = racer_b.wait_with_output().expect( "reap racer b" );

  let stderr_a = String::from_utf8_lossy( &out_a.stderr );
  let stderr_b = String::from_utf8_lossy( &out_b.stderr );

  // The winner is admitted on attempt 1 and returns immediately — it never
  // reaches the `!quiet` message block on any attempt, so its stderr is
  // empty. The loser, however, polls more than once within the 2s window
  // (CLR_GATE_POLL_SECS=1): its FIRST attempt is the genuine reclaim-ticket
  // race this test targets, but by its SECOND attempt the winner's own slot
  // record is in place and alive, so the loser legitimately (and correctly)
  // shifts to reporting "slot held by another session" from then on. Only
  // the loser's first message is this test's subject.
  let loser_stderr = if stderr_a.trim().is_empty() { stderr_b.as_ref() } else { stderr_a.as_ref() };
  let first_line   = loser_stderr.lines().next().unwrap_or_default();
  assert!(
    first_line.contains( "lost reservation race" ),
    "T16 (BUG-396): the losing racer's FIRST poll attempt must report losing the \
     reclaim-ticket race for the pre-seeded dead owner — later attempts may \
     legitimately shift to \"slot held by another session\" once the winner's own \
     slot record is alive and observed on a subsequent poll. stderr_a:\n{stderr_a}\n\
     stderr_b:\n{stderr_b}"
  );
  assert!(
    !stderr_a.contains( "at capacity" ) && !stderr_b.contains( "at capacity" ),
    "T16 (BUG-396): neither racer should report capacity exhaustion — \
     has_capacity is true for both throughout. stderr_a:\n{stderr_a}\n\
     stderr_b:\n{stderr_b}"
  );
}

// ── T17: an orphaned reclaim ticket must not permanently block the slot (BUG-402) ──

/// T17 (BUG-402): pre-seeds `gate_dir` with a dead-owner slot file AND its exact
/// reclaim ticket already on disk, keyed and content-shaped exactly as
/// `acquire_slot()` would have left it had a PREVIOUS ticket-winner crashed after
/// winning the ticket's `create_new()` but before completing `rename()` onto the
/// slot path — the ticket's own recorded claimant (`dead_claimant_pid`) is a
/// second, independently confirmed-dead PID, distinct from the slot's recorded
/// owner (`dead_owner_pid`). `CLR_PROC_DIR` stays empty for the whole run (0
/// counted occupiers), so the single `clr` invocation below always targets index
/// 0. Bounds the wait via a small `CLR_GATE_MAX_ATTEMPTS`/`CLR_GATE_POLL_SECS`
/// pair plus `--retry-override 0` (mirroring T09), so this test fails fast
/// rather than hanging if the bug reproduces. Asserts the invocation is
/// admitted (exit 0 — the fake `claude` script's own exit code) instead of
/// exhausting the gate-wait budget and exiting 1 with "session gate timed
/// out" — the permanent-block symptom BUG-402 describes.
///
/// Root Cause: `acquire_slot()`'s reclaim branch treated ANY pre-existing
/// ticket file as unconditional proof that a live reclaimer was already
/// contending for the slot — `claim_slot_file(&ticket, ..)` failing (because
/// the ticket already exists) went straight to `LostReclaimRace` with no
/// check of the ticket's OWN recorded claimant. A claimant that won the
/// ticket and then crashed before `rename()` leaves that ticket on disk
/// forever (tickets are deliberately never deleted — see the `Fix(BUG-392)`
/// Pitfall on `acquire_slot()`), so every subsequent caller hit the same
/// false denial, with nothing on disk ever going to change.
///
/// Why Not Caught: T14 (BUG-392's own regression test) and T08 both exercise
/// the reclaim path only through its single-generation happy path — a racer
/// either wins the one ticket outright or loses to a still-live winner that
/// finishes its rename shortly after. Neither constructs a ticket whose own
/// claimant has *also* already died, so the permanently-orphaned-ticket case
/// — a second, independent crash on top of the first — was entirely
/// unexercised.
///
/// Fix Applied: `acquire_slot()`'s reclaim branch now walks the reclaim-
/// ticket chain instead of stopping at the first existing ticket — when a
/// ticket's own recorded claimant is dead AND the slot record hasn't moved
/// on from the original dead owner, it advances to a new ticket keyed by
/// that dead claimant's own (pid, since) and retries the same atomic
/// `create_new()` arbitration, repeating until it either wins an unclaimed
/// generation or hits a live claimant / already-reclaimed slot. See
/// `Fix(BUG-402)` on `acquire_slot()` in `src/cli/gate.rs` for the full
/// explanation.
///
/// Prevention: this test — a fresh caller must still acquire the slot
/// promptly, well inside the bounded gate-wait budget, when the only
/// obstruction is an orphaned reclaim ticket, instead of exhausting its
/// retries and exiting with "session gate timed out".
///
/// Pitfall: any future change to this branch must preserve the exact
/// two-variant `SlotDenialCause` diagnostic contract (`HeldByLive` → "slot
/// held by another session", `LostReclaimRace` → "lost reservation race") —
/// T15 and T16 in this same file assert these exact message suffixes
/// verbatim (`config_file_test.rs` asserts only the older, generic
/// "N/N sessions active; waiting" wait-message shape, not these cause
/// suffixes). A ticket-chain fix must also re-check the
/// slot's CURRENT owner before advancing generations, not just each
/// generation's ticket-claimant liveness — otherwise a concurrent caller
/// that completes its own rename mid-walk would be silently missed, and
/// this call would report a stale verdict instead of reflecting the slot's
/// true, just-changed state.
///
/// Bug file: `task/claude_runner/402_orphaned_reclaim_ticket_permanent_slot_block.md`.
// test_kind: bug_reproducer(BUG-402)
#[ test ]
fn t17_orphaned_reclaim_ticket_does_not_permanently_block_slot()
{
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir = tempfile::TempDir::new().expect( "proc dir" ); // empty: 0 counted occupiers throughout

  // Two distinct, confirmed-dead PIDs — one for the slot's recorded owner, one
  // for the reclaim ticket's recorded claimant — mirroring T14/T16's spawn+reap
  // pattern so /proc/{pid} is genuinely absent for both, not a made-up number.
  let mut dead_owner = Command::new( "true" ).spawn().expect( "spawn short-lived process" );
  let dead_owner_pid = dead_owner.id();
  let _ = dead_owner.wait();

  let mut dead_claimant = Command::new( "true" ).spawn().expect( "spawn short-lived process" );
  let dead_claimant_pid = dead_claimant.id();
  let _ = dead_claimant.wait();

  std::fs::write(
    gate_dir.path().join( "slot_0.json" ),
    format!( r#"{{"pid":{dead_owner_pid},"since":0}}"# ),
  ).expect( "pre-seed dead-owner slot file" );

  // The orphaned reclaim ticket: exactly the file acquire_slot() would have left
  // behind had dead_claimant_pid won the ticket's create_new() and then crashed
  // before rename() — keyed by (index=0, dead_owner_pid, owner_since=0), matching
  // acquire_slot()'s own `reclaim_{index}_{owner}_{owner_since}.lock` naming.
  std::fs::write(
    gate_dir.path().join( format!( "reclaim_0_{dead_owner_pid}_0.lock" ) ),
    format!( r#"{{"pid":{dead_claimant_pid},"since":0}}"# ),
  ).expect( "pre-seed orphaned reclaim ticket" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut child = Command::new( bin )
    .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc_dir.path() )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .expect( "spawn clr" );

  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 10 );
  let exited = wait_bounded( &mut child, deadline );
  if exited.is_none() { let _ = child.kill(); }
  let out = child.wait_with_output().expect( "reap clr" );
  let stderr = String::from_utf8_lossy( &out.stderr );

  assert!(
    exited.is_some(),
    "T17 (BUG-402): clr must exit within 10s — still running past the 2-attempt, \
     1s-poll gate budget means the process is stuck outside the gate-wait loop \
     entirely. stderr:\n{stderr}"
  );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 0 ),
    "T17 (BUG-402): a fresh caller must still acquire slot 0 promptly when the only \
     obstruction is an ORPHANED reclaim ticket (its own recorded claimant is also \
     dead, not a live contender) — acquire_slot() must not treat a pre-existing \
     ticket as \"lost the race to a live reclaimer\" forever. Got exit {:?}, stderr:\n{stderr}",
    exited.and_then( |s| s.code() )
  );
  assert!(
    !stderr.contains( "session gate timed out" ),
    "T17 (BUG-402): must not exhaust the gate-wait budget — stderr:\n{stderr}"
  );
}

// ── T18: gate must scan other free indices, not just the count-derived one (BUG-404) ──

/// T18 (BUG-404): a fresh caller must not starve when its single, count-derived
/// candidate index (`count_u32`) collides with a live, genuinely-active owner
/// while ANOTHER index within `0..max` sits completely free. `--max-sessions 2`
/// creates two indices (0, 1); one real print-mode occupier is spawned and
/// registered via `make_proc_dir` so `count_u32` is always `1` — the pre-fix
/// algorithm's single candidate is always index 1. `slot_1.json` is pre-seeded
/// with that same occupier's own (genuinely alive) PID as owner, so index 1 is
/// legitimately `HeldByLive` on every attempt. `slot_0.json` is left completely
/// unclaimed.
///
/// Prior to the fix, `wait_for_session_slot()` (`src/cli/gate.rs`) computed and
/// tried only the single index `count_u32`; it never scanned `0..max` for a
/// different available index. Asserts the invocation is admitted (exit 0 — the
/// fake `claude` script's own exit code) instead of exhausting the gate-wait
/// budget and exiting non-zero with "session gate timed out" — the starvation
/// symptom BUG-404 describes.
///
/// Bug file: `task/claude_runner/bug/unverified/404_gate_single_candidate_index_no_scan.md`.
// test_kind: bug_reproducer(BUG-404)
#[ test ]
fn t18_gate_tries_other_free_index_when_count_derived_index_is_live_held()
{
  let ( _script_dir, script_path )     = fake_claude_dir( "exit 0" );
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();

  let mut occupier   = spawn_print_claude_for( &occupier_path, 10 );
  let occupier_pid   = occupier.id();
  let proc           = make_proc_dir( &[ occupier_pid ] );

  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  // Index 1 is exactly the index count_u32=1 will always compute to (one live
  // print-mode occupier registered above) — seed it as HeldByLive by the SAME
  // real, genuinely-alive occupier PID. Index 0 is left completely unclaimed.
  std::fs::write(
    gate_dir.path().join( "slot_1.json" ),
    format!( r#"{{"pid":{occupier_pid},"since":0}}"# ),
  ).expect( "pre-seed live-owner slot file at index 1" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut child = Command::new( bin )
    .args( [ "-p", "--max-sessions", "2", "--retry-override", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path() )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .expect( "spawn clr" );

  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 10 );
  let exited = wait_bounded( &mut child, deadline );
  if exited.is_none() { let _ = child.kill(); }
  let out    = child.wait_with_output().expect( "reap clr" );
  let stderr = String::from_utf8_lossy( &out.stderr );

  let _ = occupier.kill();
  let _ = occupier.wait();

  assert!(
    exited.is_some(),
    "T18 (BUG-404): clr must exit within 10s — still running past the 2-attempt, \
     1s-poll gate budget means the process is stuck outside the gate-wait loop \
     entirely. stderr:\n{stderr}"
  );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 0 ),
    "T18 (BUG-404): a fresh caller must acquire the FREE index 0 promptly even \
     though its count-derived candidate index (1) is genuinely held by a live \
     session — the gate must scan for any available index, not try only the \
     single count-derived one. Got exit {:?}, stderr:\n{stderr}",
    exited.and_then( |s| s.code() )
  );
  assert!(
    !stderr.contains( "session gate timed out" ),
    "T18 (BUG-404): must not exhaust the gate-wait budget while index 0 sits \
     completely free — stderr:\n{stderr}"
  );
}

// ── T19: a caller that wins its own reclaim ticket but fails to complete
// admission must not permanently self-deny on retry (BUG-405) ──

/// T19 (BUG-405): pre-seeds `gate_dir` with a dead-owner slot file (no
/// pre-existing ticket — this caller will be the FIRST to reach the ticket
/// for this generation, unlike T17 which pre-seeds a ticket already
/// orphaned by a DIFFERENT dead process). Sets
/// `CLR_GATE_FORCE_TMP_CLAIM_FAIL_ONCE` so the single `clr` invocation's
/// FIRST attempt at winning the ticket deterministically fails its own
/// tmp-claim step, simulating a transient fs fault — exactly the scenario
/// where the pre-fix code left the ticket behind, keyed by this same
/// process's own (pid, since), causing every subsequent retry within the
/// SAME invocation to read back its own still-alive pid and self-deny
/// forever. `CLR_GATE_MAX_ATTEMPTS=3` gives the invocation two more
/// attempts after the forced failure to prove it recovers. `CLR_PROC_DIR`
/// stays empty (0 counted occupiers), so the invocation always targets
/// index 0.
///
/// Root Cause: the ticket-win branch in `acquire_slot()` returned
/// `LostReclaimRace` on tmp-claim or rename failure without removing the
/// ticket it had just won. Because `pid`/`since` are fixed for this
/// caller's entire `wait_for_session_slot()` call, every later retry
/// recomputes the identical ticket path, finds it already claimed by
/// ITSELF, reads back its own `(pid, since)` as `next_claimant`, and
/// `pid_alive()` reports `true` — a caller can never lose a fair race to
/// its own still-running self, so the false denial repeats on every
/// subsequent attempt for that specific slot index, indefinitely.
///
/// Why Not Caught: T17 (BUG-402's own regression test) pre-seeds a ticket
/// already orphaned by a DIFFERENT, already-dead process — it never
/// exercises the case where the CURRENT invocation is itself the one that
/// wins a ticket and then fails to complete it. T14 races several live
/// concurrent callers to completion and never induces a tmp-claim or
/// rename failure on any of them. No existing test forced a caller to
/// collide with its own abandoned ticket.
///
/// Fix Applied: the ticket-win branch now removes the ticket it just won on
/// both non-admission paths (tmp-claim failure, rename failure) before
/// returning `LostReclaimRace`, so the next retry re-contends the same
/// generation fresh instead of reading back its own abandoned claim. See
/// `Fix(BUG-405)` on `acquire_slot()` in `src/cli/gate.rs`.
///
/// Prevention: this test — a caller whose own tmp-claim transiently fails
/// once must still acquire the slot within its bounded gate-wait budget on
/// a later attempt, instead of self-denying for the rest of its own
/// invocation.
///
/// Pitfall: `CLR_GATE_FORCE_TMP_CLAIM_FAIL_ONCE` is a one-shot, in-process
/// injection (an `AtomicBool` consumed on first check) — it fires exactly
/// once regardless of how many `acquire_slot()` calls precede it, matching
/// a real transient fault's lifecycle (occurs once, then clears). A test
/// relying on this env var must not assume it fires on every attempt.
///
/// Bug file: `task/claude_runner/bug/closed/405_reclaim_ticket_winner_self_collision_denial.md`.
// test_kind: bug_reproducer(BUG-405)
#[ test ]
fn t19_ticket_winner_that_fails_own_admission_does_not_self_deny_forever()
{
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir = tempfile::TempDir::new().expect( "proc dir" ); // empty: 0 counted occupiers throughout

  let mut dead_owner = Command::new( "true" ).spawn().expect( "spawn short-lived process" );
  let dead_owner_pid = dead_owner.id();
  let _ = dead_owner.wait();

  std::fs::write(
    gate_dir.path().join( "slot_0.json" ),
    format!( r#"{{"pid":{dead_owner_pid},"since":0}}"# ),
  ).expect( "pre-seed dead-owner slot file" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut child = Command::new( bin )
    .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc_dir.path() )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "3" )
    .env( "CLR_GATE_FORCE_TMP_CLAIM_FAIL_ONCE", "1" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .expect( "spawn clr" );

  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 10 );
  let exited = wait_bounded( &mut child, deadline );
  if exited.is_none() { let _ = child.kill(); }
  let out = child.wait_with_output().expect( "reap clr" );
  let stderr = String::from_utf8_lossy( &out.stderr );

  assert!(
    exited.is_some(),
    "T19 (BUG-405): clr must exit within 10s — still running past the 3-attempt, \
     1s-poll gate budget means the process is stuck outside the gate-wait loop \
     entirely. stderr:\n{stderr}"
  );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 0 ),
    "T19 (BUG-405): a caller whose own tmp-claim fails once (forced) must still \
     acquire slot 0 on a later attempt — acquire_slot() must not leave its own \
     abandoned ticket behind to self-deny every subsequent retry. Got exit {:?}, \
     stderr:\n{stderr}",
    exited.and_then( |s| s.code() )
  );
  assert!(
    !stderr.contains( "session gate timed out" ),
    "T19 (BUG-405): must not exhaust the gate-wait budget — stderr:\n{stderr}"
  );
}

// ── T20: acquire_slot() walks an arbitrarily deep reclaim-ticket chain, not
// just a single orphaned ticket (BUG-402 chain-walk capability) ──

/// T20 (BUG-402): extends T17's single-orphaned-ticket scenario to a THREE-
/// generation chain — `slot_0.json` records a dead owner
/// (`dead_owner_pid`); its reclaim ticket
/// (`reclaim_0_{dead_owner_pid}_0.lock`) is pre-seeded as already claimed by
/// a second, independently-confirmed-dead PID (`dead_claimant_1`); THAT
/// claimant's own ticket (`reclaim_0_{dead_claimant_1}_0.lock`) is in turn
/// pre-seeded as claimed by a THIRD independently-confirmed-dead PID
/// (`dead_claimant_2`) — two full orphaned generations stacked before any
/// live contender ever runs. Only the third generation's ticket
/// (`reclaim_0_{dead_claimant_2}_0.lock`) is left genuinely unclaimed, for
/// the real `clr` invocation to win fresh. Proves `acquire_slot()`'s loop
/// walks past MULTIPLE dead generations in one call, not merely the single
/// extra hop T17 exercises — the capability the chain-walk design in
/// `Fix(BUG-402)` is specifically built to provide, per its own rationale
/// comment on `acquire_slot()` in `gate.rs`. `CLR_PROC_DIR` stays empty
/// throughout (0 counted occupiers), so the invocation always targets
/// index 0.
///
/// Prevention: this test — a fresh caller must still acquire the slot
/// promptly when TWO full orphaned-ticket generations precede it, not just
/// one, proving the chain-walk loop's depth is not silently bounded to a
/// single hop.
///
/// Pitfall: each generation's ticket key is derived from the PRIOR
/// generation's own (pid, since) — `reclaim_{index}_{claimant}_{since}.lock`
/// — so the three ticket paths in this test must be constructed in the
/// exact same chained order `acquire_slot()` computes them, not assembled
/// independently, or the test would silently exercise a different, shorter
/// chain than intended.
///
/// Bug file: `task/claude_runner/bug/402_orphaned_reclaim_ticket_permanent_slot_block.md`.
// test_kind: bug_reproducer(BUG-402)
#[ test ]
fn t20_acquire_slot_walks_multi_generation_reclaim_ticket_chain()
{
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir = tempfile::TempDir::new().expect( "proc dir" ); // empty: 0 counted occupiers throughout

  // Three distinct, confirmed-dead PIDs, chained exactly as acquire_slot()'s
  // loop would advance through them — mirrors T17/T14's spawn+reap pattern.
  let mut dead_owner = Command::new( "true" ).spawn().expect( "spawn short-lived process" );
  let dead_owner_pid = dead_owner.id();
  let _ = dead_owner.wait();

  let mut dead_claimant_1 = Command::new( "true" ).spawn().expect( "spawn short-lived process" );
  let dead_claimant_1_pid = dead_claimant_1.id();
  let _ = dead_claimant_1.wait();

  let mut dead_claimant_2 = Command::new( "true" ).spawn().expect( "spawn short-lived process" );
  let dead_claimant_2_pid = dead_claimant_2.id();
  let _ = dead_claimant_2.wait();

  std::fs::write(
    gate_dir.path().join( "slot_0.json" ),
    format!( r#"{{"pid":{dead_owner_pid},"since":0}}"# ),
  ).expect( "pre-seed dead-owner slot file" );

  // Generation 1's ticket: claimed by dead_claimant_1 (also dead).
  std::fs::write(
    gate_dir.path().join( format!( "reclaim_0_{dead_owner_pid}_0.lock" ) ),
    format!( r#"{{"pid":{dead_claimant_1_pid},"since":0}}"# ),
  ).expect( "pre-seed generation-1 ticket" );

  // Generation 2's ticket: claimed by dead_claimant_2 (also dead) — keyed by
  // generation 1's own claimant, exactly as acquire_slot() would advance.
  std::fs::write(
    gate_dir.path().join( format!( "reclaim_0_{dead_claimant_1_pid}_0.lock" ) ),
    format!( r#"{{"pid":{dead_claimant_2_pid},"since":0}}"# ),
  ).expect( "pre-seed generation-2 ticket" );

  // Generation 3's ticket is deliberately left unclaimed — the real `clr`
  // invocation below must walk past both dead generations and win it fresh.

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut child = Command::new( bin )
    .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc_dir.path() )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .expect( "spawn clr" );

  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 10 );
  let exited = wait_bounded( &mut child, deadline );
  if exited.is_none() { let _ = child.kill(); }
  let out = child.wait_with_output().expect( "reap clr" );
  let stderr = String::from_utf8_lossy( &out.stderr );

  assert!(
    exited.is_some(),
    "T20 (BUG-402): clr must exit within 10s — still running past the 2-attempt, \
     1s-poll gate budget means the process is stuck outside the gate-wait loop \
     entirely. stderr:\n{stderr}"
  );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 0 ),
    "T20 (BUG-402): a fresh caller must still acquire slot 0 promptly when TWO \
     full orphaned reclaim-ticket generations precede it — acquire_slot() must \
     walk the entire chain, not just a single hop. Got exit {:?}, stderr:\n{stderr}",
    exited.and_then( |s| s.code() )
  );
  assert!(
    !stderr.contains( "session gate timed out" ),
    "T20 (BUG-402): must not exhaust the gate-wait budget — stderr:\n{stderr}"
  );
}
