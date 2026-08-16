//! Integration tests for the `--max-sessions` concurrency gate (Unix-only: uses
//! synthetic `/proc` isolation and real ELF/shell-script fake `claude` processes).
#![ cfg( unix ) ]
//!
//! Test spec: `docs/cli/user_story/025_concurrency_gate.md`, `docs/cli/param/033_max_sessions.md`.
//!
//! T15–T22 (slot-wait messaging, reclaim-ticket chains) live in `concurrency_gate_ext_test.rs`.
//! T21 (staleness threshold) onward through T34 (progress-message wording) live in
//! `concurrency_gate_ext2_test.rs`. T35/T36 and all `t_gate_*` override-tier tests
//! live in `concurrency_gate_ext3_test.rs`.
//!
//! # Test Case Index
//!
//! | ID  | Name                                                                      | TSK-368 Row |
//! |-----|----------------------------------------------------------------------------|-------------|
//! | T01 | 8 print-mode processes active, print invocation, default → gate triggers at 8 | T01 |
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
//! | T19 | `CLR_GATE_CLAIM_TEST_DELAY_MS` widens `claim_slot_file()`'s internal claim window → slot file must never be observed existing-but-unparseable during that window, only fully absent or fully valid (BUG-407) | — |
//! | T20 | pre-seeded slot owned by a genuinely alive PID with `since=0` (maximally stale) → `CLR_GATE_STALE_SECS` unset denies reclaim (default/backward-compatible); set below elapsed duration, reclaim succeeds and the waiter is admitted immediately (BUG-400) | — |
//! | T21 | pre-seeded dead-owner slot, no pre-existing ticket, forced one-time tmp-claim failure via `CLR_GATE_FORCE_TMP_CLAIM_FAIL_ONCE` → ticket winner that fails own admission still acquires the slot on retry, no permanent self-denial (BUG-405) | — |
//! | T22 | three-generation orphaned reclaim-ticket chain (two dead claimants stacked before an unclaimed generation) → `acquire_slot()` walks past both, admitting a fresh caller (BUG-402 chain-walk depth) | — |
//! | T25 | `CLR_PROC_DIR` points at a nonexistent path, `--max-sessions` unset (nonzero default) → hard exit 1 with the exact `GateUnavailable` message, no silent no-op (hardening fix 1) | — |
//! | T26 | `CLR_PROC_DIR` points at a nonexistent path, `--max-sessions 0` → gate bypassed entirely, exit 0 (hardening fix 1 regression guard: the 0=disable escape hatch must survive the loud-failure change) | — |
//! | T27 | `clr isolated`, 3 print-mode processes active, `--max-sessions 3` → gate triggers and reports 3/3, then releases once occupiers exit (hardening fix 2: isolated now contends for a slot like run/ask) | — |
//! | T28 | `clr isolated`, 2 print-mode processes active, `--max-sessions 3` → gate does not trigger (hardening fix 2 parity with T02) | — |
//! | T29 | `--gate-poll-secs 1 --gate-max-attempts 2` CLI flags (no env vars), 1 permanent occupier, `--max-sessions 1` → gate exhausts in ~2s using the CLI-flag values, not the 30s/1000-attempt hardcoded defaults (hardening fix 3, CLI-flag tier) | — |
//! | T30 | `--gate-max-attempts abc` CLI flag → immediate parse-error exit before any subprocess spawn or gate wait, distinct from the env var's silent fallback (T10/T11) (hardening fix 3, CLI-flags-hard-error contract) | — |
//! | T31 | `clr isolated`, `CLR_GATE_POLL_SECS=1 CLR_GATE_MAX_ATTEMPTS=2` env vars, 1 permanent occupier, `--max-sessions 1` → isolated's one-shot env-var-only resolution changes its real gate timing (hardening fix 3, isolated's env-var-only tier) | — |
//! | T32 | `clr isolated`, 3 print-mode processes active, `--args-file` JSON `{"max-sessions": 3}` (no CLI flag) → gate triggers and reports 3/3, proving `apply_json_config_isolated()`'s `"max-sessions"` arm actually changes isolated's resolved gate limit, not just its unused-default no-op | — |
//! | T33 | 1 live occupier in `CLR_PROC_DIR`, `--max-sessions 1` → second invocation observes `count_u32 >= max` (`has_capacity=false`) → stderr names `[at capacity]`, NOT `[slot held by another session]` or `[lost reservation race]` (INV-013 IN-4) | — |
//! | T34 | same fixture as T33 → diagnostic line preserves `"gate-wait  active="` prefix (TSK-452 format, replaces pre-TSK-452 `"active; waiting"`) when cause suffix is appended (INV-013 IN-5) | — |
//! | T37 | pre-seeded slot owned by an exited-but-unreaped (zombie) PID, `CLR_GATE_STALE_SECS` unset → owner reads as dead, slot reclaimed, caller admitted promptly (BUG-479) | — |
//! | T38 | census 0 < max 1 but the sole slot held by a live foreign owner → poll line and timeout message both name the measured occupancy `slots=1/1`; at-capacity lines never carry the field (BUG-480) | — |
//! | T39 | `CLR_REMAINING_TIMEOUT_SECS` unset / unparseable / set-but-nonlimiting → each denied gate run announces its resolution state (`gate-deadline` line, incl. `stale-reclaim` state) and the three states are mutually distinguishable (BUG-481) | — |
//! | T40 | `CLR_REMAINING_TIMEOUT_SECS` empty / negative → announced off-unparseable with raw value; `"0"` → engaged with the `.max(1)` one-attempt floor, budget-exhaustion path (BUG-481 edge matrix) | — |
//! | T41 | `CLR_GATE_POLL_SECS=0` + numeric `CLR_REMAINING_TIMEOUT_SECS` → no divide-by-zero panic; quotient divisor floored to 1, announced nonlimiting (BUG-481) | — |
//! | T42 | pre-seeded slot owned by a dead PID whose number is a live non-leader thread TID (parked helper thread), `CLR_GATE_STALE_SECS` unset → owner fails the thread-group-leader clause, slot reclaimed, caller admitted promptly (BUG-488) | — |
//! | T43 | pre-seeded slot recording a live leader PID with a deliberately mismatched start time → occupant fails the same-incarnation clause, slot reclaimed, caller admitted promptly (BUG-488) | — |
//! | T44 | pre-seeded legacy-shape slot record (no start-time field) owned by a live leader → still denied `HeldByLive`; the same-incarnation clause is inert for legacy records (BUG-488 compatibility boundary) | — |
//!
//! T05 (`clr --help` shows `default: 8`) is covered by
//! `param_edge_cases_test.rs::ec9_max_sessions_help_shows_default_eight`.
//!
//! T12 (regression: pre-existing T01/T02/T04/T08 still pass using the renamed
//! `CLR_GATE_POLL_SECS` var) is covered by those same tests post-rename — no
//! separate function.
//!
//! Note: this Test Case Index enumerates every T-ID across all 4 split files
//! (`concurrency_gate_test.rs`, `_ext_test.rs`, `_ext2_test.rs`, `_ext3_test.rs`);
//! it is kept whole in this file only to preserve its pre-existing cross-references
//! rather than being torn across files. Some T-IDs below (e.g. T21, T22) label two
//! distinct functions in different files — a pre-existing quirk, not introduced by
//! this split. `t_gate_*` override-tier tests (T35/T36 plus the poll-secs/max-attempts/
//! stale-secs/remaining-timeout variants) are not T-numbered and are listed by fn name
//! only in `concurrency_gate_ext3_test.rs`'s own header.

// BUG-387 — T01-T07 above all pre-seed a static synthetic /proc snapshot and
// invoke exactly one clr binary; none launch N concurrent clr invocations
// racing each other against a shared, mutating occupier set, so none could
// exercise the check-then-spawn TOCTOU race. T08 below closes that gap: it
// launches N concurrent live `clr` invocations and asserts peak
// simultaneously-admitted count never exceeds --max-sessions.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::
{
  build_argv_tolerant_sleeper, fake_claude_binary_dir, fake_claude_dir, make_proc_dir,
  slot_owner_pid, spawn_fake_claude, spawn_parked_helper_thread, spawn_print_claude,
  spawn_print_claude_for, wait_bounded,
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

// ── T01: gate triggers at exactly 8 print-mode processes (default limit) ───────

/// T01: 8 print-mode processes active (7 long-lived + 1 short-lived), new print-mode
/// invocation, `--max-sessions` unset (default 8) → gate triggers and emits the
/// "8/8 sessions active; waiting" message, then releases once the short-lived
/// process self-expires and the count drops below 8.
#[ test ]
fn t01_gate_triggers_at_eight_print_mode_processes()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();

  let mut long_lived : Vec< std::process::Child > =
    ( 0..7 ).map( |_| spawn_print_claude( &occupier_path ) ).collect();
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
    // Anchored on "active=" so a wrong larger count (e.g. "active=18/8") can never
    // false-positive match via the "8/8" tail — AF1.
    stderr.contains( "gate-wait  active=8/8" ),
    "T01: gate must report 8/8 print-mode sessions active. Got:\n{stderr}"
  );
}

// ── T02: gate does not trigger below the limit ──────────────────────────────────

/// T02: 7 print-mode processes active, new print-mode invocation, `--max-sessions`
/// unset (default 8) → gate does not trigger; the dispatched command proceeds
/// immediately with no wait message on stderr.
#[ test ]
fn t02_gate_does_not_trigger_below_eight_print_mode_processes()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();

  let mut occupiers : Vec< std::process::Child > =
    ( 0..7 ).map( |_| spawn_print_claude( &occupier_path ) ).collect();
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
    !stderr.contains( "gate-wait" ),
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
    !stderr.contains( "gate-wait" ),
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
    // Anchored on "active=" — "active=15/5" (unfiltered count) would not match
    // "active=5/5" as a bare substring (the `=` anchor prevents false-positives). AF1.
    stderr.contains( "gate-wait  active=5/5" ),
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
    !stderr.contains( "gate-wait" ),
    "T06: --max-sessions 0 must disable the gate. Got:\n{stderr}"
  );
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
///
/// Local to this file — not shared with any `_ext_test.rs` split.
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

// BUG-480 task/claude_runner/bug/480_gate_diagnostic_hides_slot_occupancy.md — fixed: this helper
// computes the same occupancy quantity the production sweep now tallies as
// `denied_slots` and renders as `slots=H/M`; the display assertion lives in T38
// (census/occupancy-divergence fixture), while T15-family fixtures keep
// asserting cause labels only.
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

// ── T08: N concurrent live `clr` invocations racing a shared, mutating occupier
//         set never admit more than --max-sessions at once (BUG-387) ──────────

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

  // BUG-479 task/claude_runner/bug/479_zombie_blind_pid_liveness.md — fixed: the zombie-reads-alive
  // semantics this harness note documents manifested in production; the
  // zombie-owner reclaim regression test is T37 below.
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
      "Error: [Runner] session gate timed out — 1 print sessions, max-sessions=1 — retries exhausted (exit 1)"
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
    stderr.contains( "wait=30s" ),
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

// ── T37: zombie-owned slot reclaimed, caller admitted (BUG-479) ───────────────

/// T37 (BUG-479): a slot file owned by an exited-but-unreaped (zombie) PID must
/// be reclaimed like any other dead owner's, admitting the caller promptly with
/// zero `slot held by another session` denials — `CLR_GATE_STALE_SECS` unset.
///
/// ## Root Cause
/// `pid_alive()` was bare `/proc/{pid}` existence. A zombie keeps its `/proc`
/// entry for as long as its parent fails to `wait()`, so `acquire_slot()`
/// returned `Err( HeldByLive )` for zombie-owned slots forever — and with
/// `CLR_GATE_STALE_SECS` unset (the default at every tier), no time-based
/// fallback ever reclaimed them (observed live: 7/8 slots zombie-held for
/// 9h–67h, starving all print-mode admission).
///
/// ## Why Not Caught
/// T14/T16/T17 all seed dead owners by spawning AND reaping (`wait()`), so
/// `/proc/{pid}` is absent; no fixture covered the exited-but-unreaped middle
/// state, even though T08's own harness had already tripped over exactly that
/// semantics (see its Fix(BUG-387 test) note above).
///
/// ## Fix Applied
/// `pid_alive()` now reads `/proc/{pid}/stat` and requires the state field
/// (after the last `)`) to not be `Z` — bare existence is no longer liveness.
///
/// ## Prevention
/// Liveness = `/proc/{pid}/stat` readable AND state ∉ {Z}, held in exactly one
/// predicate shared by every consumer (gate reclaim + ps queued render); never
/// bare `/proc/{pid}` existence.
///
/// ## Pitfall
/// A `/proc/{pid}` directory proves a PID exists, not that a process runs —
/// zombies keep the directory after death, so existence-keyed reclaim
/// deadlocks the moment any supervisor stops reaping its children.
// test_kind: bug_reproducer(BUG-479)
#[ test ]
fn t37_zombie_owned_slot_reclaimed_and_caller_admitted()
{
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir = tempfile::TempDir::new().expect( "proc dir" ); // empty: census sees 0 sessions

  // Manufacture a genuine zombie: spawn a real, immediately-exiting process and
  // do NOT wait() on it — this test process is its parent, so until the reap at
  // the end it stays state Z with a live /proc/{pid} entry.
  let mut zombie = Command::new( "true" ).spawn().expect( "spawn zombie-to-be" );
  let zombie_pid = zombie.id();
  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 5 );
  loop
  {
    let stat = std::fs::read_to_string( format!( "/proc/{zombie_pid}/stat" ) ).unwrap_or_default();
    if stat.rsplit_once( ')' ).is_some_and( | ( _, rest ) | rest.trim_start().starts_with( 'Z' ) ) { break; }
    assert!( std::time::Instant::now() < deadline, "fixture: PID {zombie_pid} never became a zombie" );
    std::thread::sleep( core::time::Duration::from_millis( 20 ) );
  }

  std::fs::write(
    gate_dir.path().join( "slot_0.json" ),
    format!( r#"{{"pid":{zombie_pid},"since":0}}"# ),
  ).expect( "pre-seed zombie-owner slot file" );

  let out = Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc_dir.path() )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .env_remove( "CLR_GATE_STALE_SECS" )
    .output()
    .expect( "run clr" );

  let _ = zombie.wait(); // reap only after clr has run

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    out.status.success(),
    "T37 (BUG-479): caller must be admitted after zombie-owner reclaim; got exit {:?}, stderr:\n{stderr}",
    out.status.code()
  );
  assert!(
    !stderr.contains( "slot held by another session" ),
    "T37 (BUG-479): zombie owner must not read as a live slot holder. stderr:\n{stderr}"
  );
}

// ── T38: slot-side denial names the measured occupancy (BUG-480) ──────────────

/// T38 (BUG-480): when admission blocks on the slot conjunct (census says
/// capacity is free, `active=0/1`, but the sole slot file is held by a live
/// foreign owner), the poll line and the timeout message must both name the
/// blocking quantity: `slots=1/1`.
///
/// ## Root Cause
/// Admission is a conjunction (census AND slot-CAS), but every diagnostic
/// surface interpolated only the census conjunct's locals (`count`/`max`).
/// `acquire_slot()`'s sweep collapsed per-index outcomes into one fieldless
/// `Result< (), SlotDenialCause >`, so the occupancy that actually blocked
/// admission was erased at that return boundary and never reached the
/// message sites — 66 consecutive `active=1/8` polls while 8/8 slot files
/// were held, with the real blocker appearing on no surface.
///
/// ## Why Not Caught
/// All 9+ format guards pin the `gate-wait  active=` prefix and `active=N/N`
/// ratios (the census half); the T15-family checks cause labels only. No
/// assertion anywhere named an occupancy, holder, or denied-index token.
///
/// ## Fix Applied
/// The sweep tallies denied indices (`denied_slots`) beside its surviving
/// `Result`; the poll line and both exhaustion messages append
/// ` slots={denied_slots}/{max}` — only for measured sweeps (slot-side
/// denials); the at-capacity arm never ran the sweep, so it never carries
/// the field (keeping every pinned at-capacity line byte-identical).
///
/// ## Prevention
/// When an admission predicate is a conjunction, every denial diagnostic must
/// interpolate at least one measured value from the conjunct that actually
/// failed — INV-013 extended to mandate the occupancy field for slot-side
/// causes (IN-7).
///
/// ## Pitfall
/// A diagnostic that interpolates only one conjunct's variables misattributes
/// every denial caused by the other conjunct — operators read `active=1/8` as
/// "7 slots free" while all 8 are occupied.
// test_kind: bug_reproducer(BUG-480)
#[ test ]
fn t38_slot_side_denial_names_measured_occupancy()
{
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir = tempfile::TempDir::new().expect( "proc dir" ); // empty: census sees 0 sessions

  // The sole slot (max-sessions=1) is held by a live foreign owner — this test
  // process itself: guaranteed alive for the duration, no child to manage.
  // `since:0` is irrelevant here: with CLR_GATE_STALE_SECS unset (explicitly
  // removed below), no staleness comparison ever runs against a live owner.
  let owner_pid = std::process::id();
  std::fs::write(
    gate_dir.path().join( "slot_0.json" ),
    format!( r#"{{"pid":{owner_pid},"since":0}}"# ),
  ).expect( "pre-seed live-owner slot file" );

  let out = Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc_dir.path() )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .env_remove( "CLR_GATE_STALE_SECS" )
    .output()
    .expect( "run clr" );

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!( !out.status.success(), "T38 (BUG-480): fixture must deny admission. stderr:\n{stderr}" );

  let denial_line = stderr.lines()
    .find( |l| l.contains( "reason: slot held by another session" ) )
    .unwrap_or_else( || panic!( "T38 (BUG-480): expected a slot-held denial line. stderr:\n{stderr}" ) );
  assert!(
    denial_line.contains( "slots=1/1" ),
    "T38 (BUG-480): slot-side poll line must name the measured occupancy slots=1/1. Line:\n{denial_line}"
  );
  assert!(
    denial_line.contains( "gate-wait  active=0/1" ),
    "T38 (BUG-480): census half of the line must be preserved unchanged. Line:\n{denial_line}"
  );

  let timeout_line = stderr.lines()
    .find( |l| l.contains( "session gate timed out" ) )
    .unwrap_or_else( || panic!( "T38 (BUG-480): expected the gate-timeout message. stderr:\n{stderr}" ) );
  assert!(
    timeout_line.contains( "slots=1/1 held" ),
    "T38 (BUG-480): slot-side timeout message must mirror the measured occupancy. Line:\n{timeout_line}"
  );
}

// ── T42: thread-TID-masked dead slot owner reclaimed, caller admitted (BUG-488) ─

/// T42 (BUG-488): a slot file whose recorded owner PID is dead, but whose PID
/// number is currently occupied by a live NON-LEADER thread of an unrelated
/// process, must be reclaimed like any other dead owner's — admitting the
/// caller promptly with zero `slot held by another session` denials.
///
/// ## Root Cause
/// `pid_alive()` implemented only two clauses — `/proc/{pid}/stat` readable
/// AND state ∉ {`Z`} — but Linux resolves direct `/proc/<tid>` lookups for
/// non-leader thread IDs of unrelated processes (readdir-invisible, yet
/// stat-readable with a running state). A dead recorded owner whose number a
/// live thread later occupied therefore read as alive forever, and
/// `acquire_slot()` denied `HeldByLive` indefinitely — BUG-479's admission
/// starvation with no zombie involved (observed live: dockerd startup thread
/// TID 1744061 masking a dead gate waiter for 76+ hours).
///
/// ## Why Not Caught
/// T14/T16/T17 seed dead owners by spawning AND reaping (`/proc/{pid}` absent);
/// T37 covers the exited-but-unreaped zombie middle state. No fixture covered
/// a PID number occupied by a live non-leader thread — `/proc/<tid>`'s
/// direct-lookup resolution for readdir-invisible TIDs was unmodeled.
///
/// ## Fix Applied
/// `pid_alive()` clause (c): `/proc/{pid}/status` must report `Tgid == pid`
/// (thread-group leadership) — a bare thread occupying the number fails
/// liveness, so the dead owner is reclaimed via the normal ticket-arbitrated
/// handoff.
///
/// ## Prevention
/// Liveness = stat readable AND state ∉ {`Z`} AND `Tgid == pid` AND (when the
/// record carries one) matching start time — INV-012 contract clauses (a)–(d),
/// held in exactly one predicate shared by every consumer.
///
/// ## Pitfall
/// `ls /proc` never lists non-leader TIDs, but direct `/proc/<tid>` lookup
/// resolves them — any PID-number-keyed liveness probe that skips the `Tgid`
/// check reads dead processes as alive whenever a thread occupies the number.
// test_kind: bug_reproducer(BUG-488)
#[ test ]
fn t42_thread_tid_masked_dead_slot_owner_reclaimed_and_caller_admitted()
{
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir = tempfile::TempDir::new().expect( "proc dir" ); // empty: census sees 0 sessions

  let ( tid, park_send, park_handle ) = spawn_parked_helper_thread();
  // Fixture validity: the TID must present exactly the occupancy shape under
  // test — stat readable via direct lookup, not a zombie, NOT a group leader.
  let stat = std::fs::read_to_string( format!( "/proc/{tid}/stat" ) )
    .expect( "fixture: /proc/<tid>/stat must be readable via direct lookup" );
  assert!(
    stat.rsplit_once( ')' ).is_some_and( | ( _, rest ) | !rest.trim_start().starts_with( 'Z' ) ),
    "fixture: TID {tid} must not be a zombie"
  );
  let reported_tgid = std::fs::read_to_string( format!( "/proc/{tid}/status" ) )
    .expect( "fixture: /proc/<tid>/status must be readable" )
    .lines()
    .find_map( | l | l.strip_prefix( "Tgid:" ).and_then( | v | v.trim().parse::< u32 >().ok() ) );
  assert!(
    reported_tgid.is_some_and( | t | t != tid ),
    "fixture: TID {tid} must be a non-leader thread (Tgid {reported_tgid:?})"
  );

  // Legacy record shape (no start-time field): only the thread-group-leader
  // clause can reclaim this — isolating clause (c), and matching the live
  // incident (the masked record was written by a pre-fix binary).
  std::fs::write(
    gate_dir.path().join( "slot_0.json" ),
    format!( r#"{{"pid":{tid},"since":0}}"# ),
  ).expect( "pre-seed thread-masked slot file" );

  let out = Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc_dir.path() )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .env_remove( "CLR_GATE_STALE_SECS" )
    .output()
    .expect( "run clr" );

  drop( park_send ); // release the parked helper thread
  let _ = park_handle.join();

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    out.status.success(),
    "T42 (BUG-488): caller must be admitted after thread-masked dead-owner reclaim; got exit {:?}, stderr:\n{stderr}",
    out.status.code()
  );
  assert!(
    !stderr.contains( "slot held by another session" ),
    "T42 (BUG-488): a live non-leader thread occupying the number must not read as a live slot holder. stderr:\n{stderr}"
  );

  // Record plumbing: the admitted caller's fresh claim must carry its own
  // start time — the incarnation binding exists on disk, not just in code.
  let slot_content = std::fs::read_to_string( gate_dir.path().join( "slot_0.json" ) )
    .expect( "read reclaimed slot record" );
  assert!(
    slot_content.contains( r#""starttime":"# ),
    "T42 (BUG-488): a freshly-claimed slot record must carry the writer's start time. Got:\n{slot_content}"
  );
  assert_ne!(
    slot_owner_pid( &slot_content ), Some( tid ),
    "T42 (BUG-488): reclaim must have rewritten the slot record away from the masked TID. Got:\n{slot_content}"
  );
}

// ── T43: mismatched-start-time slot owner reclaimed, caller admitted (BUG-488) ─

/// T43 (BUG-488): a slot file recording a genuinely alive LEADER PID but a
/// start time that mismatches the live occupant's `/proc/{pid}/stat` field 22
/// must be reclaimed — the record's writer is provably not the current
/// occupant of that PID number.
///
/// ## Root Cause
/// The slot record stored only the bare PID number, so liveness conflated
/// number-identity with incarnation-identity: any live process occupying the
/// recorded number read as the recorded writer. Clause (c) (thread-group
/// leadership) closes the thread-occupancy case but cannot catch a recycled
/// LEADER PID — a full wrap of the kernel PID space lands the number on a new,
/// unrelated process that passes every per-occupant check.
///
/// ## Why Not Caught
/// No fixture could produce a same-number-different-process leader without an
/// actual host PID wrap; nothing in the record identified the writer beyond
/// its number, so there was nothing for a test to compare against.
///
/// ## Fix Applied
/// `claim_slot_file()` records the writer's own start time (`/proc/{pid}/stat`
/// field 22) in the slot record; `pid_alive()` clause (d) compares the
/// recorded value against the current occupant's and reads any mismatch as
/// dead. Fabricating the mismatch in the record is equivalent to the number
/// having been recycled to a different process — a deterministic stand-in for
/// a host PID wrap.
///
/// ## Prevention
/// Records bind to `(pid, starttime)` — the process incarnation — never to the
/// bare PID number; INV-012 contract clause (d).
///
/// ## Pitfall
/// Clause (c) alone cannot catch a recycled *leader* PID — only the start-time
/// binding distinguishes two incarnations of the same number. Conversely the
/// comparison must be exact equality of the recorded field-22 value: it is
/// stable for a process's entire life, so any mismatch proves a different
/// incarnation.
// test_kind: bug_reproducer(BUG-488)
#[ test ]
fn t43_mismatched_start_time_slot_owner_reclaimed_and_caller_admitted()
{
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir = tempfile::TempDir::new().expect( "proc dir" ); // empty: census sees 0 sessions

  // Live leader occupant: this test process itself (nextest runs each test in
  // its own process) — guaranteed alive for the duration, no child to manage.
  let owner_pid = std::process::id();
  let real_starttime : u64 = std::fs::read_to_string( format!( "/proc/{owner_pid}/stat" ) )
    .ok()
    .and_then( | stat | stat.rsplit_once( ')' ).and_then( | ( _, rest ) | rest.split_whitespace().nth( 19 ).and_then( | f | f.parse().ok() ) ) )
    .expect( "fixture: parse own starttime (stat field 22)" );

  // Deliberately wrong start time: the record's writer is provably not the
  // current occupant of this PID number.
  std::fs::write(
    gate_dir.path().join( "slot_0.json" ),
    format!( r#"{{"pid":{owner_pid},"since":0,"starttime":{}}}"#, real_starttime + 1 ),
  ).expect( "pre-seed mismatched-start-time slot file" );

  let out = Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc_dir.path() )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .env_remove( "CLR_GATE_STALE_SECS" )
    .output()
    .expect( "run clr" );

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    out.status.success(),
    "T43 (BUG-488): caller must be admitted after mismatched-incarnation reclaim; got exit {:?}, stderr:\n{stderr}",
    out.status.code()
  );
  assert!(
    !stderr.contains( "slot held by another session" ),
    "T43 (BUG-488): a mismatched-start-time occupant must not read as the recorded live holder. stderr:\n{stderr}"
  );
}

// ── T44: legacy record without start time, live leader owner → still held (BUG-488) ─

/// T44 (BUG-488 compatibility boundary): a legacy-shape slot record —
/// `{"pid":N,"since":M}`, no start-time field — recording a genuinely alive
/// leader PID must still deny `HeldByLive`. NOT a reproducer: the denial
/// behavior must hold identically pre-fix and post-fix; the test pins the
/// boundary so the incarnation binding can never silently widen into mass
/// reclaim. A final post-fix-only assert additionally pins the record
/// plumbing: the denied caller's own surviving waiter file carries the
/// start-time field.
///
/// ## Root Cause
/// (Boundary guard, not a defect repro.) The BUG-488 fix adds a start-time
/// comparison to `pid_alive()`; applied unconditionally it would read EVERY
/// record written by a pre-fix binary — which carries no start-time field —
/// as a mismatch, mass-reclaiming slots held by live sessions across a
/// mid-upgrade mixed fleet.
///
/// ## Why Not Caught
/// (Preventive.) Nothing exercised the pre-fix record shape against the
/// post-fix predicate before this test existed.
///
/// ## Fix Applied
/// `read_slot_owner_record()` parses the start-time field as optional;
/// `pid_alive()` applies clause (d) only when the record actually carries the
/// field — legacy records keep clauses (a)–(c) semantics until rewritten by a
/// post-fix binary.
///
/// ## Prevention
/// The incarnation binding is additive by contract (INV-012 clause (d),
/// "legacy records" sentence); this test pins that boundary permanently.
///
/// ## Pitfall
/// An absent field and a mismatched field are different facts — treating
/// absence as mismatch converts an upgrade into an outage (every live
/// session's slot reclaimed at once by the first post-fix caller).
#[ test ]
fn t44_legacy_record_without_start_time_live_leader_still_held()
{
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir = tempfile::TempDir::new().expect( "proc dir" ); // empty: census sees 0 sessions

  // Live leader owner: this test process itself, recorded in the exact
  // pre-BUG-488 shape (no start-time field).
  let owner_pid = std::process::id();
  std::fs::write(
    gate_dir.path().join( "slot_0.json" ),
    format!( r#"{{"pid":{owner_pid},"since":0}}"# ),
  ).expect( "pre-seed legacy-shape live-owner slot file" );

  let out = Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc_dir.path() )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .env_remove( "CLR_GATE_STALE_SECS" )
    .output()
    .expect( "run clr" );

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !out.status.success(),
    "T44 (BUG-488): a live leader owner recorded without a start-time field must still hold its slot. stderr:\n{stderr}"
  );
  assert!(
    stderr.contains( "slot held by another session" ),
    "T44 (BUG-488): denial must classify as HeldByLive, not a reclaim race. stderr:\n{stderr}"
  );
  assert!(
    stderr.contains( "session gate timed out" ),
    "T44 (BUG-488): the caller must exhaust its gate-wait budget against the held slot. stderr:\n{stderr}"
  );

  // Record plumbing: waiter gate files written post-fix carry the writer's
  // start time. The denied caller exits via the exhaustion path (exit 1),
  // which skips the drop guard — its {pid}.json survives for inspection.
  let waiter_path = std::fs::read_dir( gate_dir.path() )
    .expect( "list gate dir" )
    .flatten()
    .map( | e | e.path() )
    .find( | p | p.file_stem().and_then( std::ffi::OsStr::to_str ).is_some_and( | s | s.parse::< u32 >().is_ok() ) )
    .expect( "denied caller's waiter gate file must survive the exhaustion exit" );
  let waiter_content = std::fs::read_to_string( &waiter_path ).expect( "read waiter gate file" );
  assert!(
    waiter_content.contains( r#""starttime":"# ),
    "T44 (BUG-488): a waiter gate file written post-fix must carry the writer's start time. Got:\n{waiter_content}"
  );
}

