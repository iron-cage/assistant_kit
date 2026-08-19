//! Integration tests for the `--max-sessions` concurrency gate — staleness
//! reclaim, hardening fixes, and diagnostic wording (extended).
#![ cfg( unix ) ]
//!
//! Extension of `concurrency_gate_test.rs` (T01–T14) covering the
//! opt-in staleness threshold (T21, staleness variant) and its immediate
//! post-claim content check (T22, content-validity variant — a distinct
//! function from `concurrency_gate_ext_test.rs`'s own T21/T22, a pre-existing
//! T-ID reuse this split does not introduce), the three hardening fixes
//! (T23–T24 fresh-claim arbitration and residual corrupted-content handling;
//! T25–T26 loud failure on an unavailable `/proc` scan; T27–T28 `isolated`
//! gate parity; T29–T31 CLI-flag and env-var gate-timing tiers; T32
//! `isolated`'s JSON-config `max-sessions` tier), the gate's progress-message
//! wording, and the T33/T34 diagnostic-wording checks (INV-013 IN-4/IN-5).
//!
//! See `concurrency_gate_test.rs`'s own header for the full Test Case Index
//! across all 5 split files.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::
{
  build_argv_tolerant_sleeper, fake_claude_binary_dir, fake_claude_dir, make_creds_file, make_proc_dir,
  slot_owner_pid, spawn_print_claude, spawn_print_claude_for, wait_bounded,
};
use std::io::Write as _;
use std::process::Command;
use tempfile::NamedTempFile;

// ── T21: opt-in staleness threshold makes a live-but-stale owner's slot
// reclaimable, without changing default (no-threshold) behavior (BUG-400) ──

/// T21 (BUG-400): a slot owner can be genuinely alive yet make no progress
/// indefinitely (e.g. a hung session) — pre-fix, `acquire_slot()` treats any
/// live owner as an unconditional hold with no time bound, so such a slot can
/// never be reclaimed. `CLR_GATE_STALE_SECS` is an opt-in override: once set,
/// a live owner whose recorded `since` is older than the threshold becomes
/// reclaim-eligible via the SAME ticket-arbitration path dead owners already
/// use (see `Fix(BUG-392)`/`Fix(BUG-402)` on `acquire_slot()`).
///
/// Sub-case a (`CLR_GATE_STALE_SECS` unset): pre-existing behavior is fully
/// preserved — a live owner denies unconditionally regardless of how old its
/// `since` is.
///
/// Sub-case b (`CLR_GATE_STALE_SECS` set below the slot's elapsed age): the
/// SAME kind of live owner becomes reclaimable — the caller acquires the slot
/// promptly instead of exhausting its gate-wait budget.
///
/// Each sub-case uses its own `gate_dir` and its own genuinely-alive occupier
/// process (a real, long-lived `/bin/sleep` child — NOT a `claude`/`clr`
/// process; `pid_alive()` only checks `/proc/{pid}` existence directly, so any
/// real child qualifies) so the two sub-cases cannot interfere with each
/// other. `since: 0` gives an elapsed age of decades — any positive-integer
/// threshold is "below elapsed duration".
///
/// Bug file: `task/claude_runner/bug/400_gate_reclaim_no_staleness_check.md`.
// BUG-479 task/claude_runner/bug/479_zombie_blind_pid_liveness.md — T21 sub-case a pins
// CLR_GATE_STALE_SECS default-off, the permanence enabler for zombie-held slots
// (contract unchanged by the fix).
// test_kind: bug_reproducer(BUG-400)
#[ test ]
fn t21_stale_alive_owner_becomes_reclaimable_when_threshold_set()
{
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );

  // ── sub-case a: no threshold set → current behavior unchanged (denied) ──
  {
    let mut owner = Command::new( "/bin/sleep" ).arg( "30" ).spawn().expect( "spawn alive owner" );
    let owner_pid = owner.id();
    let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
    let proc_dir = tempfile::TempDir::new().expect( "proc dir" ); // empty: 0 counted occupiers

    std::fs::write(
      gate_dir.path().join( "slot_0.json" ),
      format!( r#"{{"pid":{owner_pid},"since":0}}"# ),
    ).expect( "pre-seed live-owner slot file" );

    let bin = env!( "CARGO_BIN_EXE_clr" );
    let mut child = Command::new( bin )
      .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
      .env( "PATH", &script_path )
      .env( "CLR_PROC_DIR", proc_dir.path() )
      .env( "CLR_GATE_DIR", gate_dir.path() )
      .env( "CLR_GATE_POLL_SECS", "1" )
      .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
      .env_remove( "CLR_GATE_STALE_SECS" )
      .stdout( std::process::Stdio::null() )
      .stderr( std::process::Stdio::piped() )
      .spawn()
      .expect( "spawn clr" );

    let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 10 );
    let exited = wait_bounded( &mut child, deadline );
    if exited.is_none() { let _ = child.kill(); }
    let out = child.wait_with_output().expect( "reap clr" );
    let stderr = String::from_utf8_lossy( &out.stderr );

    let _ = owner.kill();
    let _ = owner.wait();

    assert!(
      exited.is_some(),
      "T21a (BUG-400): clr must exit within 10s — stderr:\n{stderr}"
    );
    assert!(
      stderr.contains( "session gate timed out" ),
      "T21a (BUG-400): with CLR_GATE_STALE_SECS unset, a live owner must still \
       deny unconditionally (pre-existing behavior) — expected a gate timeout, \
       got exit {:?}, stderr:\n{stderr}",
      exited.and_then( |s| s.code() )
    );
  }

  // ── sub-case b: threshold set below elapsed age → reclaim succeeds ──
  {
    let mut owner = Command::new( "/bin/sleep" ).arg( "30" ).spawn().expect( "spawn alive owner" );
    let owner_pid = owner.id();
    let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
    let proc_dir = tempfile::TempDir::new().expect( "proc dir" ); // empty: 0 counted occupiers

    std::fs::write(
      gate_dir.path().join( "slot_0.json" ),
      format!( r#"{{"pid":{owner_pid},"since":0}}"# ),
    ).expect( "pre-seed live-owner slot file" );

    let bin = env!( "CARGO_BIN_EXE_clr" );
    let mut child = Command::new( bin )
      .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
      .env( "PATH", &script_path )
      .env( "CLR_PROC_DIR", proc_dir.path() )
      .env( "CLR_GATE_DIR", gate_dir.path() )
      .env( "CLR_GATE_POLL_SECS", "1" )
      .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
      .env( "CLR_GATE_STALE_SECS", "10" )
      .stdout( std::process::Stdio::null() )
      .stderr( std::process::Stdio::piped() )
      .spawn()
      .expect( "spawn clr" );

    let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 10 );
    let exited = wait_bounded( &mut child, deadline );
    if exited.is_none() { let _ = child.kill(); }
    let out = child.wait_with_output().expect( "reap clr" );
    let stderr = String::from_utf8_lossy( &out.stderr );

    let _ = owner.kill();
    let _ = owner.wait();

    assert!(
      exited.is_some(),
      "T21b (BUG-400): clr must exit within 10s — stderr:\n{stderr}"
    );
    assert_eq!(
      exited.and_then( |s| s.code() ), Some( 0 ),
      "T21b (BUG-400): with CLR_GATE_STALE_SECS=10 and a since:0 (decades-old) \
       live owner, the caller must reclaim the slot and be admitted promptly \
       instead of exhausting its gate-wait budget. Got exit {:?}, stderr:\n{stderr}",
      exited.and_then( |s| s.code() )
    );
    assert!(
      !stderr.contains( "session gate timed out" ),
      "T21b (BUG-400): must not exhaust the gate-wait budget once the staleness \
       threshold makes the live owner reclaim-eligible — stderr:\n{stderr}"
    );
  }
}

// ── T22: claim_slot_file()'s content is valid immediately after a successful claim (BUG-407) ──

/// T22 (BUG-407): direct-correctness check of the rewritten `claim_slot_file()`
/// — a single, uncontested fresh claim followed by an immediate read of the
/// on-disk slot file must find fully-valid, complete JSON content (this `clr`
/// invocation's own pid), never a partially-written or empty artifact.
/// Proves there is no create-then-populate window to observe by
/// construction — the on-disk path only ever becomes visible via
/// `hard_link()` from an already-fully-written temp file.
#[ test ]
fn t22_claim_slot_file_content_valid_immediately_after_call()
{
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir = tempfile::TempDir::new().expect( "proc dir" ); // empty: 0 pre-existing occupiers

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut child = Command::new( bin )
    .args( [ "-p", "--max-sessions", "6", "--retry-override", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc_dir.path() )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .expect( "spawn clr" );
  let clr_pid = child.id();

  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 10 );
  let exited = wait_bounded( &mut child, deadline );
  if exited.is_none() { let _ = child.kill(); }
  let out = child.wait_with_output().expect( "reap clr" );
  let stderr = String::from_utf8_lossy( &out.stderr );

  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 0 ),
    "T22 (BUG-407): a single, uncontested fresh claim must be admitted \
     promptly. Got exit {:?}, stderr:\n{stderr}",
    exited.and_then( |s| s.code() )
  );

  let slot_path = gate_dir.path().join( "slot_0.json" );
  let content = std::fs::read_to_string( &slot_path ).unwrap_or_else(
    | e | panic!( "T22 (BUG-407): slot_0.json must exist and be readable after a successful claim: {e}" )
  );
  assert!(
    content.contains( r#""pid":"# ) && content.contains( r#""since":"# ),
    "T22 (BUG-407): slot file content must be fully-formed JSON with both \
     `pid` and `since` fields immediately after a successful claim — got: {content:?}"
  );
  assert_eq!(
    slot_owner_pid( &content ), Some( clr_pid ),
    "T22 (BUG-407): slot file's recorded pid must match this clr invocation's \
     own pid, fully written (not truncated) — got: {content:?}"
  );
}

// ── T23: claim_slot_file()'s fresh-claim arbitration still admits exactly one racer (BUG-407) ──

/// T23 (BUG-407): arbitration-preserved regression guard for the rewritten
/// `claim_slot_file()` — N racers contending for the SAME never-before-seen
/// slot path (a completely empty gate dir, `--max-sessions 1` forcing every
/// racer onto index 0) must still yield at most one admitted (concurrently
/// alive, dispatched-child-holding) winner. Confirms the write-to-temp-then-
/// `hard_link()` rewrite did not weaken the exactly-one-claimant guarantee
/// every call site depends on — `hard_link()` fails with `AlreadyExists`
/// exactly like `create_new()` did.
#[ test ]
fn t23_concurrent_racers_still_yield_exactly_one_winner()
{
  const N : usize = 8;

  let ( _bin_dir, bin_path ) = build_argv_tolerant_sleeper( 3 );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" ); // deliberately empty: no pre-seeded slot
  let proc_dir = tempfile::TempDir::new().expect( "proc dir" ); // deliberately static/empty

  let mut children : Vec< std::process::Child > = ( 0..N ).map( | i |
  {
    Command::new( env!( "CARGO_BIN_EXE_clr" ) )
      .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", &format!( "race-{i}" ) ] )
      .env( "PATH", &bin_path )
      .env( "CLR_PROC_DIR", proc_dir.path() )
      .env( "CLR_GATE_DIR", gate_dir.path() )
      .env( "CLR_GATE_POLL_SECS", "1" )
      .env( "CLR_GATE_MAX_ATTEMPTS", "5" )
      .stdout( std::process::Stdio::null() )
      .stderr( std::process::Stdio::null() )
      .spawn()
      .expect( "spawn racing clr" )
  } ).collect();

  let clr_pids : Vec< u32 > = children.iter().map( std::process::Child::id ).collect();

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
    "T23 (BUG-407): peak concurrently-alive dispatched children racing for one \
     never-before-seen slot ({peak}) must never exceed 1 — the rewritten \
     claim_slot_file() must still admit at most one racer"
  );
  assert!(
    peak >= 1,
    "T23 (BUG-407): at least one racer must have been admitted — a peak of 0 \
     would mean the rewritten claim_slot_file() admits NO ONE, a different \
     regression (over-strict arbitration) from the one this test targets"
  );
}

// ── T24: pre-existing corrupted slot content remains a documented residual (BUG-407) ──

/// T24 (BUG-407): documents the explicit scope boundary of the atomic-publish
/// fix. A slot file that was ALREADY on disk with empty/unparseable content
/// BEFORE any call to the rewritten `claim_slot_file()` ever touched it (e.g.
/// a stray `touch`, or a leftover artifact from a crash under a pre-upgrade
/// binary) is NOT repaired by this fix — `hard_link()`, like `create_new()`,
/// cannot claim a path that already exists, so the fresh-claim attempt still
/// returns `false`, and `acquire_slot()`'s unconditional `None` ->
/// `HeldByLive` branch (Fix(BUG-396), unchanged by this fix) still denies
/// forever.
///
/// This is an intentional, explicitly-accepted residual, not a silent gap:
/// the atomic rewrite's actual guarantee is that `claim_slot_file()` itself
/// can never again CREATE a new empty/incomplete file (proven directly by
/// T22) — it does not add a repair path for corruption that predates any
/// call to it, matching this fix's own Fix Location scope ("a `None` result
/// ... can only mean genuine on-disk corruption unrelated to this race (out
/// of scope)"). Verified via Tier 4 Paired Verification (independent primary
/// and adversarial code trace, both confirming that `hard_link` and
/// `create_new` share identical `AlreadyExists` semantics against an
/// existing destination) before this test was written, after the original
/// bug filing's Prevention sketch (recovery expected) was found to
/// contradict its own, more precise Fix Location section.
///
/// Asserts the residual is STABLE — denied identically pre-fix and post-fix
/// — so a future change cannot silently narrow or widen this boundary
/// without this test flagging it.
#[ test ]
fn t24_preexisting_empty_slot_file_remains_a_documented_residual()
{
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir = tempfile::TempDir::new().expect( "proc dir" ); // empty: 0 counted occupiers

  // Pre-seed slot_0.json as a 0-byte file — simulates content that was
  // ALREADY corrupted/incomplete before this test's clr invocation ever
  // runs (out of scope for this fix; see doc comment above).
  std::fs::write( gate_dir.path().join( "slot_0.json" ), b"" )
    .expect( "pre-seed empty slot file" );

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
    "T24 (BUG-407): clr must exit within 10s — stderr:\n{stderr}"
  );
  assert!(
    stderr.contains( "session gate timed out" ),
    "T24 (BUG-407): a pre-existing empty slot file is a documented residual \
     — it must still deny (gate-wait exhausted), identically pre-fix and \
     post-fix. If this now PASSES (no timeout), the residual boundary has \
     shifted and this test's doc comment / BUG-407's closure notes must be \
     updated to reflect the new behavior. Got exit {:?}, stderr:\n{stderr}",
    exited.and_then( |s| s.code() )
  );
}

// ── T25/T26: proc scanner unavailable → loud failure, not a silent no-op (hardening fix 1) ──

/// T25 (hardening fix 1): `CLR_PROC_DIR` points at a path that does not exist on
/// disk (simulating a non-Linux host, or a broken/unreadable `/proc`), with
/// `--max-sessions` left at its nonzero default. Before this fix,
/// `find_claude_processes()` silently returned an empty list whenever its proc
/// root was unreadable, so the gate always saw "0 active sessions" and admitted
/// immediately — the concurrency guarantee silently evaporated with no signal
/// to the operator. After the fix, `wait_for_session_slot()` checks
/// `claude_core::process::proc_scan_available()` before entering the poll loop
/// and exits loudly instead.
#[ test ]
fn t25_proc_scan_unavailable_fails_loudly_instead_of_silent_admit()
{
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", "/nonexistent/clr-t25-proc-dir" )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .output()
    .expect( "invoke clr" );

  assert_eq!(
    out.status.code(), Some( 1 ),
    "T25: exit must be 1 when the process scanner cannot read the process list. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains(
      "Error: [Runner] session gate unavailable — process scanner cannot read the process list \
       (--max-sessions requires working /proc; pass --max-sessions 0 to disable the gate) (exit 1)"
    ),
    "T25: exact GateUnavailable message required. Got:\n{stderr}"
  );
}

/// T26 (hardening fix 1 regression guard): the pre-existing `--max-sessions 0`
/// escape hatch (T06) must survive the loud-failure change — `wait_for_session_slot()`
/// returns before ever checking `proc_scan_available()` when `max == 0`, so a broken
/// `CLR_PROC_DIR` must never surface the `GateUnavailable` error when the gate itself
/// is explicitly disabled.
#[ test ]
fn t26_max_sessions_zero_bypasses_gate_even_when_proc_scan_unavailable()
{
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--max-sessions", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", "/nonexistent/clr-t26-proc-dir" )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "T26: --max-sessions 0 must bypass the gate entirely, even with /proc unavailable. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "session gate unavailable" ),
    "T26: GateUnavailable must never fire when --max-sessions 0 disables the gate. Got:\n{stderr}"
  );
}

// ── T27/T28: `clr isolated` participates in the same concurrency gate as run/ask (hardening fix 2) ──

/// T27 (hardening fix 2): unlike `refresh` (which always runs a fixed, throwaway
/// prompt and discards the response), `isolated` can run arbitrarily long real
/// user tasks, so it must contend for a gate slot exactly like `run`/`ask`.
/// Mirrors T01's structure with `isolated` prepended and a real (if minimal)
/// `--creds` file: 2 long-lived + 1 short-lived (self-expiring) print-mode
/// occupiers, `--max-sessions 3` → gate triggers and reports "3/3 sessions
/// active", then releases once the short-lived occupier self-expires.
#[ test ]
fn t27_isolated_gate_triggers_at_capacity()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  let mut long_lived : Vec< std::process::Child > =
    ( 0..2 ).map( |_| spawn_print_claude( &occupier_path ) ).collect();
  let mut short_lived = spawn_print_claude_for( &occupier_path, 5 );

  let mut pids : Vec< u32 > = long_lived.iter().map( std::process::Child::id ).collect();
  pids.push( short_lived.id() );
  let proc = make_proc_dir( &pids );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir   = tempfile::TempDir::new().expect( "gate dir" );
  let creds      = make_creds_file( "{}" );
  let creds_path = creds.path().to_str().expect( "creds path UTF-8" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "isolated", "--creds", creds_path, "--max-sessions", "3", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env_remove( "CLAUDECODE" )
    .output()
    .expect( "invoke clr isolated" );

  for child in &mut long_lived { let _ = child.kill(); let _ = child.wait(); }
  let _ = short_lived.kill();
  let _ = short_lived.wait();

  assert!(
    out.status.success(),
    "T27: exit must be 0 after gate releases. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "gate-wait  active=3/3" ),
    "T27: isolated must participate in the gate and report 3/3 active. Got:\n{stderr}"
  );
}

/// T28 (hardening fix 2, parity with T02): 2 print-mode processes active,
/// `clr isolated --max-sessions 3` → gate does not trigger; isolated proceeds
/// immediately with no wait message on stderr.
#[ test ]
fn t28_isolated_gate_does_not_trigger_below_capacity()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  let mut occupiers : Vec< std::process::Child > =
    ( 0..2 ).map( |_| spawn_print_claude( &occupier_path ) ).collect();
  let pids : Vec< u32 > = occupiers.iter().map( std::process::Child::id ).collect();
  let proc = make_proc_dir( &pids );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir   = tempfile::TempDir::new().expect( "gate dir" );
  let creds      = make_creds_file( "{}" );
  let creds_path = creds.path().to_str().expect( "creds path UTF-8" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "isolated", "--creds", creds_path, "--max-sessions", "3", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env_remove( "CLAUDECODE" )
    .output()
    .expect( "invoke clr isolated" );

  for child in &mut occupiers { let _ = child.kill(); let _ = child.wait(); }

  assert!(
    out.status.success(),
    "T28: exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "gate-wait" ),
    "T28: gate must not trigger below the limit for isolated. Got:\n{stderr}"
  );
}

// ── trace_gate_wait_exposure() on the `isolated` gate call site (BUG-445 Fix
// Location #3) — not T-numbered, same idiom as the `t_gate_*` override-tier
// matrix below; see `concurrency_gate_deadline_test.rs`'s "086" section for the
// run/ask-path coverage of the same diagnostic. ──────────────────────────────

/// `clr isolated --trace` with NO expressed timeout and unset
/// `CLR_REMAINING_TIMEOUT_SECS` must warn on stderr, same as the run/ask
/// path — `gate_isolated_session()` calls `trace_gate_wait_exposure()` with
/// `cli.timeout_expressed` (BUG-445 Fix Location #2 added the expression bit,
/// so isolated's baked-in 30s default no longer masquerades as a caller
/// choice; an expressed `--timeout` now defaults the gate budget instead of
/// warning).
// test_kind: edge_case
#[ test ]
fn t_gate_isolated_trace_exposure_warns_when_remaining_timeout_unset()
{
  let proc = make_proc_dir( &[] );
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir   = tempfile::TempDir::new().expect( "gate dir" );
  let creds      = make_creds_file( "{}" );
  let creds_path = creds.path().to_str().expect( "creds path UTF-8" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [
      "isolated", "--creds", creds_path, "--max-sessions", "3",
      "--trace", "--journal", "off", "x",
    ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env_remove( "CLR_REMAINING_TIMEOUT_SECS" )
    .env_remove( "CLR_TIMEOUT" )
    .env_remove( "CLAUDECODE" )
    .output()
    .expect( "invoke clr isolated" );

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "Trace: gate-wait is unbounded — no --timeout given and CLR_REMAINING_TIMEOUT_SECS is unset" ),
    "isolated must also warn about unbounded gate-wait exposure (BUG-445). Got:\n{stderr}"
  );
}

/// `clr isolated --trace --timeout 0` is an explicit unlimited opt-out — must
/// not warn, mirroring the run/ask path's `--timeout 0` guard.
// test_kind: edge_case
#[ test ]
fn t_gate_isolated_trace_exposure_silent_when_timeout_zero()
{
  let proc = make_proc_dir( &[] );
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir   = tempfile::TempDir::new().expect( "gate dir" );
  let creds      = make_creds_file( "{}" );
  let creds_path = creds.path().to_str().expect( "creds path UTF-8" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [
      "isolated", "--creds", creds_path, "--max-sessions", "3", "--timeout", "0",
      "--trace", "--journal", "off", "x",
    ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env_remove( "CLR_REMAINING_TIMEOUT_SECS" )
    .env_remove( "CLAUDECODE" )
    .output()
    .expect( "invoke clr isolated" );

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "Trace: gate-wait is unbounded" ),
    "isolated --timeout 0 is an explicit unlimited opt-out — must not warn. Got:\n{stderr}"
  );
}

/// `clr isolated --timeout 5` (expressed) with `CLR_REMAINING_TIMEOUT_SECS`
/// unset must default the gate-wait budget from the flag (BUG-445 Fix
/// Location #2), pinning the isolated call site's
/// `if cli.timeout_expressed { cli.timeout_secs } else { 0 }` threading —
/// the run/ask-path reproducer lives in `concurrency_gate_deadline_test.rs`
/// (086/FIX-2). Denial fixture per T39: the sole slot pre-seeded with this
/// test process's own live pid, empty census.
// test_kind: edge_case
#[ test ]
fn t_gate_isolated_expressed_timeout_defaults_gate_wait_budget()
{
  let proc = make_proc_dir( &[] );
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir   = tempfile::TempDir::new().expect( "gate dir" );
  let creds      = make_creds_file( "{}" );
  let creds_path = creds.path().to_str().expect( "creds path UTF-8" );
  let owner_pid  = std::process::id();
  std::fs::write(
    gate_dir.path().join( "slot_0.json" ),
    format!( r#"{{"pid":{owner_pid},"since":0}}"# ),
  ).expect( "pre-seed live-owner slot file" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [
      "isolated", "--creds", creds_path, "--max-sessions", "1", "--timeout", "5",
      "--journal", "off", "x",
    ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS",    "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "8" )
    .env_remove( "CLR_REMAINING_TIMEOUT_SECS" )
    .env_remove( "CLR_GATE_STALE_SECS" )
    .env_remove( "CLAUDECODE" )
    .output()
    .expect( "invoke clr isolated" );

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert_eq!(
    out.status.code(), Some( 1 ),
    "isolated's gate exhaustion closure exits 1. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "engaged (5s from --timeout clamps to 5 of 8 attempts)" ),
    "isolated's expressed --timeout 5 must default the gate budget and name \
     its source (BUG-445). Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "gate-wait budget exhausted" ),
    "exhaustion under the --timeout-sourced budget must report the budget \
     diagnostic. Got:\n{stderr}"
  );
}

// ── T29/T30/T31: CLI-flag + isolated env-var-only parity for gate tuning (hardening fix 3) ──

/// T29 (hardening fix 3, CLI-flag tier): `--gate-poll-secs 1 --gate-max-attempts 2`
/// as CLI flags (deliberately no env vars set) must change the gate's actual
/// runtime behavior for `run`/`ask`, exactly as `CLR_GATE_POLL_SECS`/
/// `CLR_GATE_MAX_ATTEMPTS` already do via the env var tier (T09). With one
/// print-mode occupier permanently holding the only `--max-sessions 1` slot and
/// `--retry-override 0` disabling the outer Runner-retry wrapper, the gate must
/// exhaust after exactly 2 polls at 1-second intervals (~2s), not the
/// production default of 1000 attempts × 30s. Bounded to a 10s deadline.
#[ test ]
fn t29_gate_cli_flags_change_real_poll_timing()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  let mut occupier = spawn_print_claude_for( &occupier_path, 60 );
  let proc = make_proc_dir( &[ occupier.id() ] );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut child = Command::new( bin )
    .args( [
      "-p", "--max-sessions", "1", "--retry-override", "0",
      "--gate-poll-secs", "1", "--gate-max-attempts", "2",
      "--journal", "off", "x",
    ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
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
    "T29: gate must exhaust within 10s when both CLI flags are active (2 attempts x 1s poll) \
     — still running means the flags are not taking effect. stderr:\n{stderr}"
  );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 1 ),
    "T29: exit must be 1 once the gate exhausts. stderr: {stderr}"
  );
  // BUG-480 task/claude_runner/bug/480_gate_diagnostic_hides_slot_occupancy.md — fixed with NO change
  // here: this full-line guard (and T31's) pins an AT-CAPACITY exhaustion (census
  // 1/1, sweep never ran), and the fix emits `slots=` only for measured sweeps —
  // these lines stay byte-identical and now double as the exemption's pin.
  // T38 in concurrency_gate_test.rs covers the slot-side `slots=H/M held` variant.
  assert!(
    stderr.contains(
      "Error: [Runner] session gate timed out — 1 print sessions, max-sessions=1 — retries exhausted (exit 1)"
    ),
    "T29: exact exhaustion message required. Got:\n{stderr}"
  );
}

/// T30 (hardening fix 3, CLI-flags-hard-error contract): `--gate-max-attempts abc`
/// must be a hard parse error at argument-parsing time (exit 1, before any
/// subprocess spawn or gate wait) — the deliberate asymmetry with the env var
/// equivalent, which silently falls back to the default instead (T11). PATH is
/// deliberately `/nonexistent`: parsing fails before any claude binary lookup
/// is ever attempted, so no fake claude script is needed.
#[ test ]
fn t30_invalid_gate_max_attempts_cli_flag_is_a_hard_parse_error()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--gate-max-attempts", "abc", "--journal", "off", "x" ] )
    .env( "PATH", "/nonexistent" )
    .output()
    .expect( "invoke clr" );

  assert_eq!(
    out.status.code(), Some( 1 ),
    "T30: an invalid --gate-max-attempts value must be a hard parse error (exit 1), \
     not a silent fallback like the env var equivalent (T11). stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "invalid --gate-max-attempts value: abc" ),
    "T30: parse error must name the flag and the bad value. Got:\n{stderr}"
  );
}

/// T31 (hardening fix 3, isolated's env-var-only tier): `clr isolated` has no
/// CLI-flag or config-file tier for the 3 gate-tuning knobs (consistent with
/// its other fields — see `gate_isolated_session`'s doc comment) but must still
/// honor `CLR_GATE_POLL_SECS`/`CLR_GATE_MAX_ATTEMPTS` via its one-shot
/// `gate_poll_secs_from`/`gate_max_attempts_from` resolution. Mirrors T09's
/// exhaustion scenario for `run`, but dispatched via `isolated` — and asserts
/// isolated's un-suffixed exhaustion message (its `on_exhausted` closure exits
/// directly, with no `apply_runner_retry` wrapper, so no "— retries exhausted"
/// suffix appears).
#[ test ]
fn t31_isolated_gate_env_vars_change_real_poll_timing()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  let mut occupier = spawn_print_claude_for( &occupier_path, 60 );
  let proc = make_proc_dir( &[ occupier.id() ] );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir   = tempfile::TempDir::new().expect( "gate dir" );
  let creds      = make_creds_file( "{}" );
  let creds_path = creds.path().to_str().expect( "creds path UTF-8" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut child = Command::new( bin )
    .args( [ "isolated", "--creds", creds_path, "--max-sessions", "1", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .env_remove( "CLAUDECODE" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .expect( "spawn clr isolated" );

  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 10 );
  let exited = wait_bounded( &mut child, deadline );
  if exited.is_none() { let _ = child.kill(); }
  let out = child.wait_with_output().expect( "reap clr isolated" );

  let _ = occupier.kill();
  let _ = occupier.wait();

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    exited.is_some(),
    "T31: isolated's gate must exhaust within 10s when both env var overrides are active \
     (2 attempts x 1s poll) — still running means isolated is not honoring them. stderr:\n{stderr}"
  );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 1 ),
    "T31: exit must be 1 once the gate exhausts. stderr: {stderr}"
  );
  assert!(
    stderr.contains(
      "Error: [Runner] session gate timed out — 1 print sessions, max-sessions=1 (exit 1)"
    ),
    "T31: exact exhaustion message required (isolated's on_exhausted closure exits directly, \
     no retry-exhausted suffix). Got:\n{stderr}"
  );
}

// ── T32: isolated's `--max-sessions` JSON-key tier (`apply_json_config_isolated()`) ──

/// T32: `clr isolated --args-file <json with "max-sessions": 3>`, 3 print-mode
/// processes active, no `--max-sessions` CLI flag → gate must trigger and report
/// 3/3 active. `isolated`'s default `--max-sessions` is 8, so 3 active occupiers
/// would never trigger the gate under the default (see T28's below-capacity
/// shape) — seeing the 3/3 wait message here is only possible if
/// `apply_json_config_isolated()`'s `"max-sessions" =>` arm actually applied the
/// JSON-supplied value of 3, not merely parsed-and-discarded it.
#[ test ]
fn t32_isolated_max_sessions_json_config_changes_real_gate_limit()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  let mut long_lived : Vec< std::process::Child > =
    ( 0..2 ).map( |_| spawn_print_claude( &occupier_path ) ).collect();
  let mut short_lived = spawn_print_claude_for( &occupier_path, 5 );

  let mut pids : Vec< u32 > = long_lived.iter().map( std::process::Child::id ).collect();
  pids.push( short_lived.id() );
  let proc = make_proc_dir( &pids );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir   = tempfile::TempDir::new().expect( "gate dir" );
  let creds      = make_creds_file( "{}" );
  let creds_path = creds.path().to_str().expect( "creds path UTF-8" );

  let mut cfg = NamedTempFile::new().expect( "args-file" );
  write!( cfg, r#"{{"max-sessions": 3}}"# ).expect( "write args-file JSON" );
  let cfg_path = cfg.path().to_str().expect( "args-file path UTF-8" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "isolated", "--creds", creds_path, "--args-file", cfg_path, "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env_remove( "CLAUDECODE" )
    .output()
    .expect( "invoke clr isolated" );

  for child in &mut long_lived { let _ = child.kill(); let _ = child.wait(); }
  let _ = short_lived.kill();
  let _ = short_lived.wait();

  assert!(
    out.status.success(),
    "T32: exit must be 0 after gate releases. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "gate-wait  active=3/3" ),
    "T32: --args-file \"max-sessions\": 3 must set the real gate limit to 3, proving the JSON \
     key tier is functional for isolated (default is 8, which would never trigger at 3 active). \
     Got:\n{stderr}"
  );
}

// ── t_gate_progress_message_names_print_sessions: BUG-431 regression guard ──

/// BUG-431 regression guard: gate progress message must include the "print" mode
/// qualifier when counting print-mode-only occupiers.
///
/// Exercises the `eprintln!` at `gate.rs:703`. Covers IN-6 (INV-013).
///
/// # Root Cause
///
/// `count` in `gate.rs` is filtered to print-mode processes only, but the progress
/// message said `"sessions active"` — indistinguishable from a total session count.
/// A reader diagnosing capacity issues could not tell whether the number referred
/// to all running claude sessions or only print-mode ones.
///
/// # Why Not Caught
///
/// The existing test assertions checked `!stderr.contains("sessions active; waiting")`
/// (negative — ensuring the gate didn't trigger without occupiers) but no positive
/// assertion required the `"print"` qualifier to be present in the message text.
///
/// # Fix Applied
///
/// `gate.rs:703` message changed from `"sessions active"` to `"print sessions active"`,
/// making the mode qualifier explicit in every gate progress line.
///
/// # Prevention
///
/// This test now asserts `stderr.contains("gate-wait  active=")` (TSK-452 updated the
/// format to a structured timestamp-prefixed line; the print-mode scope is preserved
/// via the same `display_count` which counts only print-mode processes). Any future
/// edit to the progress message format must preserve the `"gate-wait  active="` label.
///
/// # Pitfall
///
/// The four negative assertions now check `!stderr.contains("gate-wait")` — the old
/// `"sessions active; waiting"` substring no longer appears in any gate progress line
/// after TSK-452's format change.
#[ test ]
fn t_gate_progress_message_names_print_sessions()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  let mut occupier = spawn_print_claude( &occupier_path );
  let proc = make_proc_dir( &[ occupier.id() ] );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .output()
    .expect( "invoke clr" );

  let _ = occupier.kill();
  let _ = occupier.wait();

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "gate-wait  active=" ),
    "BUG-431 regression (TSK-452 format): gate progress message must include 'gate-wait  active='. Got:\n{stderr}"
  );
}

// ── T33: genuine exhaustion names "[at capacity]" (INV-013 IN-4) ─────────────

/// T33 (INV-013 IN-4): one long-running occupier holds the sole slot
/// (`--max-sessions 1`), so a second invocation always observes
/// `count_u32 >= max` (`has_capacity=false`) without ever attempting
/// to acquire any slot index. Captures stderr and asserts it names the
/// cause as `[at capacity]`, NOT `[slot held by another session]` (which
/// requires `has_capacity=true` and a live slot-file holder) or
/// `[lost reservation race]` (which requires `has_capacity=true` and a
/// dead owner with a contested reclaim ticket).
///
/// ## Root Cause (INV-013 motivation)
///
/// Prior to this test the `has_capacity=false` branch's cause label had
/// zero focused positive-assertion coverage. T15/T16 exercise only the
/// `has_capacity=true` branches; T09 captures the final exhaustion error
/// line rather than the per-attempt diagnostic. The `[at capacity]` label
/// was therefore unverified reachable from any test.
///
/// ## Why Not Caught
///
/// T01/T04 positively assert `"gate-wait  active="` (TSK-452) but run with
/// `count_u32 < max`, so the `[at capacity]` suffix is never emitted
/// in their fixture and neither can catch a future regression that
/// routes the exhaustion path through the wrong cause label.
///
/// ## Fix Applied
///
/// No production code change — coverage-only addition confirming
/// `wait_for_session_slot()`'s `[at capacity]` suffix is reachable
/// and correct for the `has_capacity=false` path.
///
/// ## Prevention
///
/// Assert `stderr.contains("at capacity")` plus the two negative guards
/// so any future refactor that routes exhaustion through the wrong label
/// fails here explicitly rather than silently.
///
/// ## Pitfall
///
/// `count_u32 >= max` requires a LIVE counted occupier in `CLR_PROC_DIR`,
/// not just a pre-seeded slot file: `count_u32` is derived from the proc
/// scan, not slot files. A dead occupier in a slot file with an empty
/// `CLR_PROC_DIR` yields `count_u32 = 0 < max = 1`, routing through the
/// `has_capacity=true` reclaim path instead — T16's scenario, not this one.
// test_kind: invariant_guard(INV-013)
#[ test ]
fn t33_slot_wait_message_names_at_capacity_for_exhaustion()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  // One live occupier whose PID appears in CLR_PROC_DIR → count_u32 = 1.
  // 60s lifetime is well above this test's ~3s window (2 attempts × 1s).
  let mut occupier = spawn_print_claude_for( &occupier_path, 60 );
  let proc = make_proc_dir( &[ occupier.id() ] );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .output()
    .expect( "invoke clr" );

  let _ = occupier.kill();
  let _ = occupier.wait();

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "at capacity" ),
    "T33 (INV-013 IN-4): exhaustion branch must name cause as \"at capacity\" when \
     count_u32 >= max. Got:\n{stderr}"
  );
  assert!(
    !stderr.contains( "slot held by another session" ),
    "T33 (INV-013 IN-4): exhaustion branch must NOT name \"slot held by another session\" — \
     that label requires has_capacity=true. Got:\n{stderr}"
  );
  assert!(
    !stderr.contains( "lost reservation race" ),
    "T33 (INV-013 IN-4): exhaustion branch must NOT name \"lost reservation race\" — \
     that label requires has_capacity=true and a dead-owner reclaim ticket. Got:\n{stderr}"
  );
}

// ── T34: non-admission message preserves "gate-wait  active=" prefix (INV-013 IN-5)

/// T34 (INV-013 IN-5): any non-admission diagnostic — here the exhaustion case
/// (same fixture as T33: `count_u32 >= max`) — must preserve the TSK-452
/// structured prefix `"gate-wait  active="`. The differentiating cause suffix
/// `[at capacity]` / `[slot held by another session]` / `[lost reservation
/// race]` appears in the `(reason: ...)` trailer at the end of the same line,
/// so all assertions that pattern-match the prefix survive any future
/// trailing-format change.
///
/// ## Root Cause (INV-013 motivation)
///
/// BUG-393 introduced the cause suffix appended AFTER the gate-wait line body.
/// TSK-452 replaced the old `"active; waiting"` body with a structured
/// `"gate-wait  active=X/Y ..."` prefix. A hypothetical refactor dropping or
/// renaming the prefix would silently break assertions with no dedicated guard.
///
/// ## Why Not Caught
///
/// T01/T04 assert `"gate-wait  active="` positively but only in fixtures where
/// `count_u32 < max` — the cause-suffix-appended path is never exercised
/// in those tests, so they cannot detect a regression in the cause-labeled
/// branch's format string. No prior test positively asserted the prefix
/// survives the suffix addition in the non-admission path.
///
/// ## Fix Applied (TSK-452)
///
/// No production code change — updated to assert the TSK-452 format prefix
/// `"gate-wait  active="` in the cause-labeled format string in `gate.rs`.
///
/// ## Prevention
///
/// Assert `stderr.contains("gate-wait  active=")` with a fixture that DOES emit
/// a cause suffix, not just a no-cause wait message.
///
/// ## Pitfall
///
/// Using T01/T04's fixture (fewer occupiers than max) would trivially pass even
/// if the prefix were removed from the cause-labeled branch, since those
/// fixtures never reach the cause-labeled `eprintln!` path. Only a fixture
/// that forces cause-labeled output validates IN-5's exact requirement.
// test_kind: invariant_guard(INV-013)
#[ test ]
fn t34_non_admission_message_preserves_active_waiting_substring()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  let mut occupier = spawn_print_claude_for( &occupier_path, 60 );
  let proc = make_proc_dir( &[ occupier.id() ] );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .output()
    .expect( "invoke clr" );

  let _ = occupier.kill();
  let _ = occupier.wait();

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "gate-wait  active=" ),
    "T34 (INV-013 IN-5): cause-labeled diagnostic must preserve \"gate-wait  active=\" \
     prefix (TSK-452 format) — the count ratio follows immediately after. Got:\n{stderr}"
  );
}

