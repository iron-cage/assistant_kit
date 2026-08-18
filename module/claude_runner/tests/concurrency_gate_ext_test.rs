//! Integration tests for the `--max-sessions` concurrency gate — slot-wait
//! messaging and reclaim-ticket chains (extended).
#![ cfg( unix ) ]
//!
//! Extension of `concurrency_gate_test.rs` (T01–T14) covering T15–T23: the
//! wait-message wording that distinguishes a live hold from a lost reclaim
//! race (T15/T16), an orphaned reclaim ticket that must not permanently block
//! a slot (T17), the fallback free-index scan (T18), atomic slot-file
//! publication under a widened claim window (T19), the opt-in staleness
//! threshold for reclaiming a live-but-stalled owner (T20), a ticket winner
//! that fails its own admission and must not self-deny forever (T21), a
//! multi-generation orphaned reclaim-ticket chain walk (T22), and the
//! deterministic (race-free) assertion of the lost-reclaim-race wording (T23).
//!
//! Fix(BUG-530): T16 and T23 split one previously-flaky assertion in two. T16
//! keeps the genuine two-process race but asserts only what a race can
//! guarantee, because which denial cause its loser observes depends on
//! inter-process spawn skew no test can enforce; T23 asserts the
//! `LostReclaimRace` wording itself from a pre-seeded live-claimant ticket,
//! where no race exists to be lost and no timing margin is involved.
//!
//! See `concurrency_gate_test.rs`'s own header for the full Test Case Index
//! across all 4 split files.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::
{
  fake_claude_binary_dir, fake_claude_dir, make_proc_dir,
  slot_owner_pid, spawn_print_claude_for, wait_bounded, wait_for_marker_in_files,
};
use std::process::Command;

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
  let stderr_dir = tempfile::TempDir::new().expect( "stderr dir" );
  let stderr_a_path = stderr_dir.path().join( "race-a.stderr" );
  let stderr_b_path = stderr_dir.path().join( "race-b.stderr" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let spawn_racer = | label : &str, stderr_path : &std::path::Path |
  {
    let stderr_file = std::fs::File::create( stderr_path ).expect( "create racer stderr file" );
    Command::new( bin )
      .args( [ "-p", "--max-sessions", "1", "--journal", "off", label ] )
      .env( "PATH", &script_path )
      .env( "CLR_PROC_DIR", proc_dir.path() )
      .env( "CLR_GATE_DIR", gate_dir.path() )
      .env( "CLR_GATE_POLL_SECS", "1" )
      .env( "CLR_GATE_MAX_ATTEMPTS", "5" )
      .stdout( std::process::Stdio::null() )
      .stderr( stderr_file )
      .spawn()
      .expect( "spawn racing clr" )
  };

  let mut racer_a = spawn_racer( "race-a", &stderr_a_path );
  let mut racer_b = spawn_racer( "race-b", &stderr_b_path );

  // Both racers read count_u32=0 < max=1 on attempt 1 and contend for the same
  // reservation index; the loser's message prints immediately (no delay before
  // the first poll's eprintln) — see `## Root Cause` above. Fix(BUG-508): poll
  // the racers' file-redirected stderr for the loser's message instead of a
  // fixed sleep — a fixed duration has no adaptive margin and can under-wait
  // under genuine host CPU contention, producing a false-red failure (both
  // racers' stderr empty) instead of the test simply taking a little longer.
  // 15s is a generous ceiling, well under either racer's own
  // CLR_GATE_MAX_ATTEMPTS=5 exhaustion; the winner is still present in
  // `/proc` (unreaped by this harness) throughout, since neither racer is
  // killed until the marker is observed or the ceiling is hit.
  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 15 );
  assert!(
    wait_for_marker_in_files( &[ stderr_a_path.as_path(), stderr_b_path.as_path() ], "gate-wait", deadline ),
    "T15 (BUG-393/396/508): timed out after 15s waiting for either racer's stderr \
     to report a gate-wait message"
  );
  let _ = racer_a.kill();
  let _ = racer_b.kill();
  let _ = racer_a.wait();
  let _ = racer_b.wait();

  let stderr_a = std::fs::read_to_string( &stderr_a_path ).unwrap_or_default();
  let stderr_b = std::fs::read_to_string( &stderr_b_path ).unwrap_or_default();

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
///
/// ## Fix(BUG-509): Root Cause
/// `CLR_GATE_RECLAIM_TEST_DELAY_MS` (in `acquire_slot()`, `gate.rs`) widens
/// the reclaim-ticket race only from the point a racer's process has already
/// been scheduled far enough to pass the dead-owner check onward — it gives
/// zero protection against OS scheduling delay in getting the racer's
/// process itself dispatched and run up to that point. At the original 50ms
/// (matching T14), under genuine host CPU contention one racer's process
/// could be scheduling-delayed long enough after spawn that the other racer
/// completed its entire reclaim first; the delayed racer then observed the
/// now-alive winner as current owner and returned `HeldByLive` immediately,
/// skipping the ticket-race branch this test's first-attempt assertion
/// requires.
///
/// ## Fix(BUG-509): Why Not Caught
/// The 50ms value was a deliberate, reasoned choice ("matching T14") and
/// correct under the scheduling conditions this test was authored and
/// normally run under — never exercised against severe, unrelated host-wide
/// contention until repeated back-to-back isolated re-runs (made practical
/// by BUG-508's own fix, which made this test resolve in ~0.1s instead of a
/// fixed ~2s) happened to coincide with such contention.
///
/// ## Fix(BUG-509): Fix Applied
/// Widened `CLR_GATE_RECLAIM_TEST_DELAY_MS` from `"50"` to `"500"` — a 10x
/// larger scheduling-slack budget for both racers to reach the dead-owner
/// check before either can complete its full reclaim. T14
/// (`concurrency_gate_test.rs`) is unchanged: its peak-admission invariant is
/// timing-independent and does not need this margin.
///
/// ## Fix(BUG-509): Prevention
/// A delay injected to widen a race window between two independently
/// scheduled OS processes must be sized against realistic *contended-host*
/// process-spawn-to-checkpoint scheduling latency, not just the in-process
/// work nominally being widened — and a value borrowed from a sibling test
/// must be re-validated against this test's own assertion strictness, not
/// assumed safe by association (T14 tolerates arbitrary scheduling skew;
/// this test's first-attempt-content assertion does not).
///
/// ## Fix(BUG-509): Pitfall
/// "Matching" an existing test's widening-delay value propagates whatever
/// margin that other test chose without re-validating adequacy for a
/// stricter assertion — the same numeric value can be safe for one test and
/// unsafe for another, depending on what each actually asserts once the
/// window closes.
///
/// ## Fix(BUG-530): Root Cause
/// The BUG-509 widening treated an insufficient margin as the root cause when
/// the actual defect was a missing synchronization. `acquire_slot()` returns
/// `HeldByLive` (`gate.rs:525`) BEFORE `reclaim_test_delay()` (`gate.rs:527`),
/// so the injected delay widens only the window in which the DEAD owner
/// record stays visible — it never synchronizes the racers' arrival. For the
/// loser to reach the ticket branch, both racers must execute their slot-read
/// within that window: a bound on relative process-spawn skew that this test
/// neither controls nor enforces. Under parallel-suite load the skew exceeds
/// 500ms often enough to fail regularly.
///
/// ## Fix(BUG-530): Why Not Caught
/// The dependency is invisible on an idle host, where back-to-back spawn skew
/// is far below any of the chosen margins. Both prior fixes were validated
/// that way, and neither made the required interleaving mandatory — so a
/// passing run could never falsify either one. Reproduction required inducing
/// CPU contention deliberately (`-0003_mre_530.sh`: 5/10 runs failed).
///
/// ## Fix(BUG-530): Fix Applied
/// Removed the timing dependency instead of widening it a third time. This
/// test's first-attempt assertion was relaxed to what a race can actually
/// guarantee (one of the two legitimate causes, never a spurious third), and
/// the deterministic `LostReclaimRace` classification assertion moved to T23,
/// which pre-seeds the slot's reclaim ticket with a LIVE claimant so a single
/// process walks dead-owner -> ticket-exists -> claimant-alive with no race.
/// `CLR_GATE_RECLAIM_TEST_DELAY_MS` stays at 500 — it still makes the race
/// genuine when skew permits; nothing now ASSERTS that it did.
///
/// ## Fix(BUG-530): Prevention
/// A test whose assertion depends on a specific interleaving must make that
/// interleaving mandatory — via a rendezvous, a pre-seeded end state, or a
/// single-process construction — never via a sleep sized against observed
/// skew. Assert the classification where it can be forced; assert only
/// non-determinism-tolerant properties where it cannot.
///
/// ## Fix(BUG-530): Pitfall
/// Widening a timing constant in response to an intermittent failure is
/// evidence that the synchronization is missing, not a fix for it. Each
/// widening lowers the rate enough to look resolved while leaving the
/// dependency fully intact, so the defect returns under any new load profile
/// — and each recurrence costs a fresh investigation from scratch (this was
/// the third on this one test).
// test_kind: bug_reproducer(BUG-396); bug_reproducer(BUG-509); bug_reproducer(BUG-530)
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

  let stderr_dir = tempfile::TempDir::new().expect( "stderr dir" );
  let stderr_a_path = stderr_dir.path().join( "race-a.stderr" );
  let stderr_b_path = stderr_dir.path().join( "race-b.stderr" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let spawn_racer = | label : &str, stderr_path : &std::path::Path |
  {
    let stderr_file = std::fs::File::create( stderr_path ).expect( "create racer stderr file" );
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
      // reclaim_test_delay() in gate.rs) so this test forces genuine ticket
      // contention on every run instead of depending on incidental OS
      // scheduling jitter between the two racers. Fix(BUG-509): 500ms, not
      // T14's 50ms — reclaim_test_delay() only widens the window AFTER a
      // racer's process has already been scheduled far enough to reach the
      // dead-owner check; under host contention a scheduling-delayed racer
      // can otherwise never reach that point at all before the other racer
      // completes its full reclaim. T14 tolerates that (timing-independent
      // mutual-exclusion invariant); this test's first-attempt-content
      // assertion does not, so it needs a materially larger margin.
      .env( "CLR_GATE_RECLAIM_TEST_DELAY_MS", "500" )
      .stdout( std::process::Stdio::null() )
      .stderr( stderr_file )
      .spawn()
      .expect( "spawn racing clr" )
  };

  let mut racer_a = spawn_racer( "race-a", &stderr_a_path );
  let mut racer_b = spawn_racer( "race-b", &stderr_b_path );

  // Fix(BUG-508): poll for the loser's first gate-wait line instead of a
  // fixed sleep — a fixed duration has no adaptive margin and can under-wait
  // under genuine host CPU contention, producing a false-red failure (both
  // racers' stderr empty). 15s is a generous ceiling, well under either
  // racer's own CLR_GATE_MAX_ATTEMPTS=5 exhaustion.
  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 15 );
  assert!(
    wait_for_marker_in_files( &[ stderr_a_path.as_path(), stderr_b_path.as_path() ], "gate-wait", deadline ),
    "T16 (BUG-392/508): timed out after 15s waiting for either racer's stderr \
     to report a gate-wait message"
  );
  let _ = racer_a.kill();
  let _ = racer_b.kill();
  let _ = racer_a.wait();
  let _ = racer_b.wait();

  let stderr_a = std::fs::read_to_string( &stderr_a_path ).unwrap_or_default();
  let stderr_b = std::fs::read_to_string( &stderr_b_path ).unwrap_or_default();

  // The winner is admitted on attempt 1 and returns immediately — it never
  // reaches the `!quiet` message block on any attempt, so its stderr is
  // empty. The loser polls until killed above (CLR_GATE_POLL_SECS=1).
  //
  // Fix(BUG-530): WHICH cause the loser observes is not something this test
  // can determine. Reaching the reclaim-ticket branch at all requires both
  // racers to execute their slot-read within the
  // `CLR_GATE_RECLAIM_TEST_DELAY_MS` window, because `acquire_slot()` returns
  // `HeldByLive` (gate.rs:525) BEFORE `reclaim_test_delay()` (gate.rs:527)
  // whenever the recorded owner is already alive. That is a constraint on
  // relative process-spawn skew between two independently spawned processes,
  // which this test neither controls nor enforces — under parallel-suite CPU
  // contention the winner routinely completes its `rename()` first, and the
  // loser then CORRECTLY reports "slot held by another session".
  //
  // So this test now asserts only what a genuine race can guarantee: the
  // loser's first gate-wait names one of the two legitimate
  // dead-owner-contention causes and never a spurious third. The
  // deterministic assertion — that `LostReclaimRace` specifically produces
  // "lost reservation race" — moved to T23, which pre-seeds a live-claimant
  // ticket and needs no race, no delay, and no margin at all.
  //
  // The loser's first gate-WAIT message is selected by its `gate-wait` token,
  // not as stderr's literal first line, because the gate's first denied
  // attempt is preceded by the one-time `gate-deadline` resolution
  // announcement (BUG-481; param/033), which is not a wait message.
  let loser_stderr = if stderr_a.trim().is_empty() { stderr_b.as_str() } else { stderr_a.as_str() };
  let first_line   = loser_stderr.lines().find( | l | l.contains( "gate-wait" ) ).unwrap_or_default();
  assert!(
    first_line.contains( "lost reservation race" ) || first_line.contains( "slot held by another session" ),
    "T16 (BUG-396/530): the losing racer's first gate-wait must name one of the two \
     legitimate dead-owner-contention causes — \"lost reservation race\" (it reached the \
     ticket branch) or \"slot held by another session\" (the winner's rename() landed \
     first). Any other reason is a message-differentiation defect. stderr_a:\n{stderr_a}\n\
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

// ── T19: claim_slot_file() must publish its content atomically with its claim (BUG-407) ──

/// T19 (BUG-407): a slot file must never be observable on disk in an
/// existing-but-unparseable state — the state a crash between
/// `claim_slot_file()`'s `create_new()` and its content `write!()` leaves
/// behind, which `acquire_slot()` then classifies as a permanent denial
/// (`HeldByLive` at the primary slot, `LostReclaimRace` at a reclaim ticket)
/// with no liveness recheck and no reclaim path, because no owner PID was
/// ever parsed out of the empty content.
///
/// `CLR_GATE_CLAIM_TEST_DELAY_MS` widens the window `claim_slot_file()`
/// leaves open around its content becoming durable — pre-fix, between
/// `create_new()` succeeding and `write!()` completing; post-fix, between
/// its temp file being fully written and `hard_link()` publishing it. While
/// a single `clr` invocation is inside that widened window, this test polls
/// the slot file directly from the test harness (the crash-mid-write state
/// is a pure filesystem artifact — no second racer is needed to observe it)
/// and asserts it is always either fully absent or fully parseable, never
/// present-but-unparseable.
///
/// Root Cause: `claim_slot_file()`'s `create_new(true).open(path)` and its
/// subsequent `write!()` are two independent, non-atomic steps
/// (`src/cli/gate.rs:115-124`) — `path` becomes visible to concurrent
/// readers the instant `create_new()` succeeds, before its content is
/// written.
///
/// Why Not Caught: no existing test pre-seeds or observes an
/// existing-but-empty slot/ticket file — T15/T17/T18 all exercise a
/// *readable* pre-existing record (dead owner, orphaned ticket, live
/// owner); none constructs, or waits to observe, a file that exists but
/// fails to parse.
///
/// Fix Applied: see `Fix(BUG-407)` on `claim_slot_file()` in `src/cli/gate.rs`.
///
/// Prevention: this test — with the window widened to 1s, a single fresh
/// claim on an empty gate dir must never leave the slot file observable in
/// an unparseable state at any point during its own claim.
///
/// Pitfall: this test polls tightly (5ms) throughout a generous 6s window
/// specifically to catch a transient bad state a slower or shorter poll
/// could miss — a passing result without ever observing a fully-valid
/// record would not actually prove the widened window was exercised.
///
/// Bug file: `task/claude_runner/bug/completed/407_claim_slot_file_non_atomic_create_then_write.md`.
// test_kind: bug_reproducer(BUG-407)
#[ test ]
fn t19_claim_slot_file_publish_is_atomic_under_widened_window()
{
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir = tempfile::TempDir::new().expect( "proc dir" ); // empty: 0 counted occupiers

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut child = Command::new( bin )
    .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc_dir.path() )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_CLAIM_TEST_DELAY_MS", "1000" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .expect( "spawn clr" );

  let slot_path      = gate_dir.path().join( "slot_0.json" );
  let poll_deadline  = std::time::Instant::now() + core::time::Duration::from_secs( 6 );
  let mut saw_unparseable = false;
  let mut saw_valid       = false;
  while std::time::Instant::now() < poll_deadline
  {
    match std::fs::read_to_string( &slot_path )
    {
      Ok( content ) if slot_owner_pid( &content ).is_some() =>
      {
        saw_valid = true;
        break;
      }
      Ok( _empty_or_malformed ) =>
      {
        saw_unparseable = true;
        break;
      }
      Err( _not_yet_created ) => {}
    }
    std::thread::sleep( core::time::Duration::from_millis( 5 ) );
  }

  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 10 );
  let exited = wait_bounded( &mut child, deadline );
  if exited.is_none() { let _ = child.kill(); }
  let out    = child.wait_with_output().expect( "reap clr" );
  let stderr = String::from_utf8_lossy( &out.stderr );

  assert!(
    !saw_unparseable,
    "T19 (BUG-407): observed slot_0.json existing but NOT yet parseable during \
     claim_slot_file()'s widened claim window — this is exactly the on-disk state \
     a crash between create_new() and write!() leaves behind, which acquire_slot() \
     then classifies as a permanent denial. stderr:\n{stderr}"
  );
  assert!(
    saw_valid,
    "T19 (BUG-407): never observed slot_0.json in a fully-valid state within the \
     poll window — the widened claim window may not have been exercised at all. \
     stderr:\n{stderr}"
  );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 0 ),
    "T19 (BUG-407): the claim must still succeed once its widened window elapses. \
     Got exit {:?}, stderr:\n{stderr}",
    exited.and_then( |s| s.code() )
  );
}

// ── T20: reclaim gate has no staleness check for a live-but-stalled owner (BUG-400) ──

/// T20 (BUG-400): pre-seeds `slot_0.json` with a genuinely alive occupier PID and
/// `since=0` — the earliest representable timestamp, i.e. maximally stale by any
/// real-world threshold — then races a single waiter against it twice against the
/// SAME pre-seeded file. `since=0` (rather than a delay-based near-past timestamp)
/// makes the elapsed-duration comparison deterministic: `unix_now() - 0` is always
/// far larger than any small test threshold, with no dependency on real wall-clock
/// timing precision or scheduling jitter.
///
/// Phase A: `CLR_GATE_STALE_SECS` unset — `acquire_slot()`'s only reclaim-eligibility
/// test is `pid_alive(owner)`, so a live owner blocks the waiter indefinitely
/// regardless of how stale `since` is; asserts the pre-fix/default behavior is
/// unchanged (denied, gate exhausts) — this must remain true after the fix too,
/// since an unset threshold means `is_stale` is always `false` (backward compatible).
/// Also confirms Phase A does not mutate the pre-seeded slot file, since Phase B
/// reuses it.
///
/// Phase B: `CLR_GATE_STALE_SECS=10`, well below the effectively-infinite elapsed
/// duration since `since=0` — the owner is live but now ALSO stale, so the fixed
/// `acquire_slot()` must fall through into the existing dead-owner reclaim-ticket
/// path (the same machinery BUG-392/396/402 already established) and admit the
/// waiter on its very first attempt.
///
/// ## Root Cause
/// `acquire_slot()`'s reclaim-eligibility branch (`src/cli/gate.rs`, `if
/// pid_alive( owner )`) is a single binary condition with no elapsed-time
/// comparison against the recorded `owner_since` anywhere — a live-but-stalled
/// (hung/deadlocked/SIGSTOPped) slot holder blocks a waiter forever even when
/// aggregate capacity exists elsewhere, because the waiter's candidate index is
/// deterministically re-derived from the same live count every poll, making the
/// collision sticky rather than a one-off.
///
/// ## Why Not Caught
/// T15/T18 both pre-seed a live owner, but neither varies `since` or exercises any
/// staleness comparison — no prior test asserts on elapsed-time-based reclaim
/// eligibility at all; the feature does not exist yet.
///
/// ## Fix Applied
/// See `Fix(BUG-400)` on `acquire_slot()`/`gate_stale_secs()` in `src/cli/gate.rs`.
///
/// ## Prevention
/// Any future reclaim-eligibility change must be re-verified against both an
/// unset threshold (Phase A: denied) and a set, exceeded threshold (Phase B:
/// admitted) — collapsing either direction silently reopens either the
/// starvation bug (BUG-400) or a backward-compatibility regression.
///
/// ## Pitfall
/// Do not replace `since=0` with a delay-based near-past timestamp (e.g.
/// `unix_now() - 2` paired with a 2s `std::thread::sleep`) — that reintroduces
/// exactly the wall-clock-precision flakiness this test's `since=0` choice avoids.
///
/// Bug file: `task/claude_runner/bug/completed/400_gate_reclaim_no_staleness_check.md`.
// test_kind: bug_reproducer(BUG-400)
#[ test ]
fn t20_gate_reclaims_stale_live_owner_when_threshold_set()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  // 30s: comfortably longer than both phases' bounded runs combined, so the
  // occupier's own self-expiry never races either waiter's observation window.
  let mut occupier = spawn_print_claude_for( &occupier_path, 30 );
  let occupier_pid = occupier.id();

  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  // Empty: the occupier is deliberately NOT registered here — its liveness is
  // checked directly against real /proc by pid_alive(), independent of the
  // synthetic process count this dir backs (mirrors T15/T16's convention).
  let proc_dir = tempfile::TempDir::new().expect( "proc dir" );

  std::fs::write(
    gate_dir.path().join( "slot_0.json" ),
    format!( r#"{{"pid":{occupier_pid},"since":0}}"# ),
  ).expect( "pre-seed live-but-stale owner slot file at index 0" );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  // ── Phase A: unset CLR_GATE_STALE_SECS -> denied, gate exhausts ──
  let mut waiter_a = Command::new( bin )
    .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc_dir.path() )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .expect( "spawn waiter phase A" );

  let deadline_a = std::time::Instant::now() + core::time::Duration::from_secs( 10 );
  let exited_a = wait_bounded( &mut waiter_a, deadline_a );
  if exited_a.is_none() { let _ = waiter_a.kill(); }
  let out_a    = waiter_a.wait_with_output().expect( "reap waiter phase A" );
  let stderr_a = String::from_utf8_lossy( &out_a.stderr );

  assert!(
    stderr_a.contains( "slot held by another session" ),
    "T20 (BUG-400) phase A: default/unset CLR_GATE_STALE_SECS must preserve \
     current behavior — a live owner (even one recorded since=0, i.e. maximally \
     stale) is never reclaimed. stderr:\n{stderr_a}"
  );
  assert_eq!(
    exited_a.and_then( |s| s.code() ), Some( 1 ),
    "T20 (BUG-400) phase A: waiter must exhaust the gate (exit 1), never be \
     admitted, while CLR_GATE_STALE_SECS is unset. Got exit {:?}, stderr:\n{stderr_a}",
    exited_a.and_then( |s| s.code() )
  );

  let still_owned_by_occupier = std::fs::read_to_string( gate_dir.path().join( "slot_0.json" ) )
    .ok()
    .is_some_and( |c| c.contains( &occupier_pid.to_string() ) );
  assert!(
    still_owned_by_occupier,
    "T20 (BUG-400): phase A (a pure denial, no reclaim attempted) must not mutate \
     the pre-seeded slot file — phase B below reuses it."
  );

  // ── Phase B: CLR_GATE_STALE_SECS below the elapsed duration -> reclaim succeeds ──
  let mut waiter_b = Command::new( bin )
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
    .expect( "spawn waiter phase B" );

  let deadline_b = std::time::Instant::now() + core::time::Duration::from_secs( 10 );
  let exited_b = wait_bounded( &mut waiter_b, deadline_b );
  if exited_b.is_none() { let _ = waiter_b.kill(); }
  let out_b    = waiter_b.wait_with_output().expect( "reap waiter phase B" );
  let stderr_b = String::from_utf8_lossy( &out_b.stderr );

  let _ = occupier.kill();
  let _ = occupier.wait();

  assert_eq!(
    exited_b.and_then( |s| s.code() ), Some( 0 ),
    "T20 (BUG-400) phase B: once CLR_GATE_STALE_SECS is set below the elapsed \
     duration, a live-but-stale owner must be reclaimed on the very first \
     attempt, admitting the waiter immediately. Got exit {:?}, stderr:\n{stderr_b}",
    exited_b.and_then( |s| s.code() )
  );
  assert!(
    !stderr_b.contains( "slot held by another session" ) && !stderr_b.contains( "session gate timed out" ),
    "T20 (BUG-400) phase B: waiter must be admitted immediately via the reclaim \
     path, not fall back to the live-hold denial or exhaust the gate. stderr:\n{stderr_b}"
  );
}

// ── T21: a caller that wins its own reclaim ticket but fails to complete
// admission must not permanently self-deny on retry (BUG-405) ──

/// T21 (BUG-405): pre-seeds `gate_dir` with a dead-owner slot file (no
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
/// Bug file: `task/claude_runner/bug/completed/405_reclaim_ticket_winner_self_collision_denial.md`.
// test_kind: bug_reproducer(BUG-405)
#[ test ]
fn t21_ticket_winner_that_fails_own_admission_does_not_self_deny_forever()
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
    "T21 (BUG-405): clr must exit within 10s — still running past the 3-attempt, \
     1s-poll gate budget means the process is stuck outside the gate-wait loop \
     entirely. stderr:\n{stderr}"
  );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 0 ),
    "T21 (BUG-405): a caller whose own tmp-claim fails once (forced) must still \
     acquire slot 0 on a later attempt — acquire_slot() must not leave its own \
     abandoned ticket behind to self-deny every subsequent retry. Got exit {:?}, \
     stderr:\n{stderr}",
    exited.and_then( |s| s.code() )
  );
  assert!(
    !stderr.contains( "session gate timed out" ),
    "T21 (BUG-405): must not exhaust the gate-wait budget — stderr:\n{stderr}"
  );
}

// ── T22: acquire_slot() walks an arbitrarily deep reclaim-ticket chain, not
// just a single orphaned ticket (BUG-402 chain-walk capability) ──

/// T22 (BUG-402): extends T17's single-orphaned-ticket scenario to a THREE-
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
fn t22_acquire_slot_walks_multi_generation_reclaim_ticket_chain()
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
    "T22 (BUG-402): clr must exit within 10s — still running past the 2-attempt, \
     1s-poll gate budget means the process is stuck outside the gate-wait loop \
     entirely. stderr:\n{stderr}"
  );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 0 ),
    "T22 (BUG-402): a fresh caller must still acquire slot 0 promptly when TWO \
     full orphaned reclaim-ticket generations precede it — acquire_slot() must \
     walk the entire chain, not just a single hop. Got exit {:?}, stderr:\n{stderr}",
    exited.and_then( |s| s.code() )
  );
  assert!(
    !stderr.contains( "session gate timed out" ),
    "T22 (BUG-402): must not exhaust the gate-wait budget — stderr:\n{stderr}"
  );
}


// ── T23: LostReclaimRace wording asserted with no race at all (BUG-396; BUG-530) ──

/// T23 (BUG-396/530): asserts the `LostReclaimRace` → "lost reservation race"
/// mapping deterministically — no second process, no injected delay, no timing
/// margin, and therefore nothing for host load to perturb.
///
/// Pre-seeds `gate_dir` with a dead-owner `slot_0.json` AND that owner's exact
/// reclaim ticket, keyed as `acquire_slot()` names it
/// (`reclaim_{index}_{owner}_{owner_since}.lock`), whose recorded claimant is a
/// **live** PID — this test's own process, which is by construction alive, its
/// own thread-group leader, and not a zombie, so `pid_alive()` returns true via
/// its `None => true` legacy-record arm. A single `clr` invocation then walks
/// `acquire_slot()` deterministically:
///
///   1. `claim_slot_file(slot_0.json)` fails — the path already exists.
///   2. Owner record reads back the dead PID, so the `HeldByLive` guard at
///      `gate.rs:525` does NOT fire.
///   3. `claim_slot_file(ticket)` fails — the ticket already exists.
///   4. The ticket's claimant is alive → `Err(LostReclaimRace)`.
///
/// Every step is a property of on-disk state this test wrote itself, so the
/// outcome cannot depend on scheduling.
///
/// Contrast T17, which pre-seeds the same ticket shape with a **dead** claimant
/// to prove the orphaned-ticket chain walk recovers; here a **live** claimant
/// proves the opposite branch reports genuine contention. Contrast T16, which
/// keeps the real two-process race but — per BUG-530 — can no longer assert
/// WHICH cause the loser observes, because that depends on inter-process spawn
/// skew no test can enforce.
///
/// ## Pitfall
/// The ticket filename must be keyed by the SLOT OWNER's pid/since, not the
/// claimant's — `acquire_slot()` derives it from the owner record it just read,
/// so a ticket named after the claimant is simply never consulted and the test
/// would silently exercise the ticket-win path instead of the contention path.
// test_kind: bug_reproducer(BUG-396); bug_reproducer(BUG-530)
#[ test ]
fn t23_slot_wait_message_names_lost_reclaim_race_without_a_race()
{
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir = tempfile::TempDir::new().expect( "proc dir" ); // empty: 0 counted occupiers

  // Confirmed-dead slot owner — spawned and reaped, so /proc/{pid} is genuinely
  // absent rather than a made-up number (same pattern as T14/T16/T17).
  let mut dead_owner = Command::new( "true" ).spawn().expect( "spawn short-lived process" );
  let dead_owner_pid = dead_owner.id();
  let _ = dead_owner.wait();

  // The live ticket claimant: this very test process.
  let live_claimant_pid = std::process::id();

  std::fs::write(
    gate_dir.path().join( "slot_0.json" ),
    format!( r#"{{"pid":{dead_owner_pid},"since":0}}"# ),
  ).expect( "pre-seed dead-owner slot file" );

  std::fs::write(
    gate_dir.path().join( format!( "reclaim_0_{dead_owner_pid}_0.lock" ) ),
    format!( r#"{{"pid":{live_claimant_pid},"since":0}}"# ),
  ).expect( "pre-seed live-claimant reclaim ticket" );

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
    "T23 (BUG-396/530): clr must exit within 10s — still running past the 2-attempt, \
     1s-poll gate budget means the process is stuck outside the gate-wait loop \
     entirely. stderr:\n{stderr}"
  );
  assert!(
    stderr.contains( "lost reservation race" ),
    "T23 (BUG-396/530): the slot's owner is dead and its reclaim ticket is held by a \
     LIVE claimant, so acquire_slot() must classify this as LostReclaimRace and say \
     \"lost reservation race\" — deterministically, on every attempt, with no race to \
     lose. stderr:\n{stderr}"
  );
  assert!(
    !stderr.contains( "slot held by another session" ),
    "T23 (BUG-396/530): must NOT report a live hold — the slot's recorded owner is \
     confirmed dead; only the ticket's claimant is alive, which is genuine reclaim \
     contention, not occupancy. Reporting occupancy here is exactly the \
     message-differentiation defect BUG-396 fixed. stderr:\n{stderr}"
  );
  assert!(
    !stderr.contains( "at capacity" ),
    "T23 (BUG-396/530): must NOT report capacity exhaustion — has_capacity is true \
     throughout (CLR_PROC_DIR is empty, so 0 counted occupiers). stderr:\n{stderr}"
  );
}
