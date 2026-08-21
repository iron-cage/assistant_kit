//! Session-gate admission: the poll loop, its waiter-telemetry state file, and the
//! diagnostics it emits.
//!
//! The knobs it polls with are resolved in `gate_limits.rs`, the on-disk reservation
//! protocol it arbitrates through lives in `gate_slot.rs`, and the `/proc` liveness
//! predicates both rely on live in `gate_liveness.rs`.

use claude_core::process::find_claude_processes;
use claude_runner_core::ps_table::classify_mode;
use core::fmt::Write as _;
use std::path::PathBuf;
use claude_journal::{ EventRecord, EventType, JournalWriter };
use super::gate_limits::effective_gate_attempts;
use super::gate_liveness::proc_starttime;
use super::gate_slot::{ acquire_slot, unix_now, SlotDenialCause };

// Return the gate state directory — $CLR_GATE_DIR or <sys-temp>/clr-gate.
//
// $CLR_GATE_DIR is the single test-injection point; tests override it to a temp
// dir so IT-10/IT-11 never touch the real default path on the host.
pub( super ) fn gate_dir() -> PathBuf
{
  std::env::var( "CLR_GATE_DIR" )
    .ok()
    .filter( |s| !s.is_empty() )
    .map_or_else( || std::env::temp_dir().join( "clr-gate" ), PathBuf::from )
}

// Fix(BUG-293): RAII guard for gate file cleanup.
// Root cause: wait_for_session_slot() had no Drop impl — abnormal exit
// (panic, Ctrl+C) left orphaned gate files on disk permanently.
// Pitfall: Drop does NOT run on SIGKILL (bypasses destructors) — the
// /proc/{pid} liveness filter in build_queued_table() handles those
// orphans via self-healing deletion.
struct GateFile( PathBuf );

impl Drop for GateFile
{
  fn drop( &mut self )
  {
    let _ = std::fs::remove_file( &self.0 );
  }
}

// Fix(BUG-384): escape a string for embedding as a JSON string value, per RFC 8259 §7.
//
// Root cause: the gate-state writer originally spliced `cwd` (an OS-controlled string)
// into a hand-rolled JSON literal with zero escaping. A first fix pass added
// `.replace('\\', ..).replace('"', ..)`, which closed the two most common cases but left
// raw control characters (bytes < 0x20 — Unix paths may legally contain a literal
// newline, tab, or other control byte) completely unescaped, still producing invalid
// JSON for such a `cwd`. This single-pass escaper closes that gap by handling every
// JSON-reserved character in one place instead of chaining ad hoc `.replace()` calls.
//
// Pitfall: never hand-roll JSON escaping via a growing chain of `.replace()` calls for
// individual characters — it's easy to cover the common cases (`"`, `\`) and forget the
// full control-character class the JSON grammar also requires escaping.
fn json_escape_str( s : &str ) -> String
{
  let mut out = String::with_capacity( s.len() );
  for c in s.chars()
  {
    match c
    {
      '"' => out.push_str( "\\\"" ),
      '\\' => out.push_str( "\\\\" ),
      '\u{08}' => out.push_str( "\\b" ),
      '\u{0C}' => out.push_str( "\\f" ),
      '\n' => out.push_str( "\\n" ),
      '\r' => out.push_str( "\\r" ),
      '\t' => out.push_str( "\\t" ),
      c if ( c as u32 ) < 0x20 => { let _ = write!( out, "\\u{:04x}", c as u32 ); },
      c => out.push( c ),
    }
  }
  out
}

fn emit_gate_wait_event(
  gate_emitted : bool,
  wait_start   : &std::time::Instant,
  journal      : Option< &JournalWriter >,
  max          : u32,
  attempt      : u32,
)
{
  if !gate_emitted { return; }
  let wait_ms = u64::try_from( wait_start.elapsed().as_millis() ).unwrap_or( u64::MAX );
  if let Some( w ) = journal
  {
    let mut ev              = EventRecord::new( EventType::GateWait );
    ev.fields.max_sessions  = Some( max );
    ev.fields.wait_ms       = Some( wait_ms );
    ev.fields.gate_attempts = Some( attempt.saturating_sub( 1 ) );
    ev.fields.gate_outcome  = Some( "acquired".to_string() );
    super::execution::stamp_attribution( &mut ev );
    let _ = w.append( &ev );
  }
}

/// Block until fewer than `max` `claude` sessions are running, or until `max_attempts`
/// is exhausted.  `max == 0` means unlimited — returns immediately without checking.
///
/// Exits the process immediately (loud failure) if the process scanner
/// ([`claude_core::process::proc_scan_available`]) cannot read the process list —
/// e.g. `/proc` missing on a non-Linux host, or a misconfigured `CLR_PROC_DIR` in
/// tests. Silently proceeding would make [`claude_core::process::find_claude_processes`]'s
/// deliberately-silent empty-`Vec` fallback look identical to "zero sessions running",
/// letting the gate wave through unlimited concurrent sessions while believing it
/// still enforces `max` — see `pid_alive()` in `gate_liveness.rs` for why the gate
/// targets Linux hosts only.
///
/// While waiting, writes a JSON state file to `$CLR_GATE_DIR/{pid}.json` so that
/// `clr ps` can display this process in its "Queued CLR Processes" table.  The file
/// is updated each polling iteration and removed automatically by the `GateFile` Drop
/// guard on both normal and panic exit paths.
///
/// When `max_attempts` is reached, calls `on_exhausted` with the timeout error instead
/// of retrying internally — callers with Runner-class retry (`run`/`ask`, via
/// `apply_runner_retry()`) retry the whole polling sequence from their closure; callers
/// without it (`isolated`) report and exit from theirs. Either way, returning from the
/// closure resumes the outer poll loop; a closure that exits the process never returns.
///
/// `caller_timeout_secs` carries the caller's EXPRESSED timeout (`--timeout` flag or
/// `CLR_TIMEOUT` env) for BUG-445's gate-budget defaulting: pass the expressed value in
/// seconds, or `0` when the caller expressed nothing (built-in defaults never qualify)
/// or explicitly opted out via `--timeout 0` — `0` disables the fallback, leaving
/// `CLR_REMAINING_TIMEOUT_SECS` as the only budget input, exactly the pre-fix behavior.
#[ allow( clippy::too_many_lines, clippy::too_many_arguments ) ] // gate admission orchestration — census read, slot sweep, one-time resolution announcement, exhaustion routing, and telemetry in one coherent poll loop (mirrors execution.rs retry orchestration); the 8th param is BUG-445's expressed-timeout budget input, resolved per-caller
pub( super ) fn wait_for_session_slot(
  max                 : u32,
  quiet               : bool,
  poll_secs           : u64,
  max_attempts        : u32,
  stale_secs          : Option< u64 >,
  caller_timeout_secs : u64,
  journal             : Option< &JournalWriter >,
  on_exhausted        : &mut dyn FnMut( &std::io::Error ),
)
{
  if max == 0 { return; }

  // Fix: fail loudly instead of silently no-op'ing when the process scanner is
  // unavailable — see doc comment above.
  if !claude_core::process::proc_scan_available()
  {
    eprintln!(
      "Error: [Runner] session gate unavailable — process scanner cannot read the process list (--max-sessions requires working /proc; pass --max-sessions 0 to disable the gate) (exit 1)"
    );
    std::process::exit( 1 );
  }

  let ( effective_max_attempts, budget_is_limiting, deadline_state ) = effective_gate_attempts( max_attempts, poll_secs, caller_timeout_secs );
  // Fix(BUG-481): resolved state of the staleness-reclaim protection, from the
  // already-tier-resolved parameter (run/ask: 5-tier chain; isolated: env-only)
  // rather than re-reading CLR_GATE_STALE_SECS — the announcement must report
  // what this gate entry actually runs with, whichever tier supplied it.
  // Root cause: gate_stale_secs_from's None (unset or invalid) silently
  // disabled staleness reclaim — the permanence enabler for BUG-479's
  // zombie-held slots — with no surface distinguishing off from on.
  // Pitfall: same silent-off pattern as the deadline clamp; both optional
  // protections announce in one line at the first denied attempt.
  let stale_state = match stale_secs
  {
    Some( s ) => format!( "on ({s}s)" ),
    None      => "off".to_string(),
  };

  let poll = core::time::Duration::from_secs( poll_secs );

  // Gate state file — best-effort; I/O failures must not abort the caller.
  let pid        = std::process::id();
  // Fix(BUG-488): capture this process's own start time once — recorded in
  // every slot/ticket/waiter artifact this call writes, binding each record
  // to this exact process incarnation. See pid_alive() for the full comment.
  let my_starttime = proc_starttime( pid );
  let dir        = gate_dir();
  let _          = std::fs::create_dir_all( &dir );
  let state_path = dir.join( format!( "{pid}.json" ) );
  let cwd        = std::env::current_dir()
    .map( |p| p.display().to_string() )
    .unwrap_or_default();
  // Fix(BUG-384): escape reserved JSON characters before interpolating cwd into the
  // hand-rolled JSON literal below — Unix paths may contain `"`, `\`, or raw control
  // characters, any of which would otherwise corrupt the gate-state file's JSON.
  // Root cause: format!() performs no JSON escaping; cwd was spliced in raw.
  // Pitfall: never hand-roll JSON from an OS-controlled string without escaping —
  // Unix paths permit any byte except `/` and NUL. See json_escape_str() above for
  // why a single-pass escaper replaced this fix's first, incomplete `.replace()` chain.
  let cwd_escaped = json_escape_str( &cwd );
  let since = unix_now();
  // Fix(BUG-488): waiter telemetry carries the same incarnation binding as
  // slot records, so ps.rs's display-liveness filter can verify it too.
  let starttime_field = my_starttime.map_or_else( String::new, | st | format!( r#","starttime":{st}"# ) );
  let _     = std::fs::write(
    &state_path,
    format!( r#"{{"cwd":"{cwd_escaped}","since":{since}{starttime_field},"attempt":0,"message":"waiting for session slot"}}"# ),
  );

  // Drop guard removes the gate file on return or unwinding panic ONLY —
  // std::process::exit() (e.g. the exhaustion path's exit(1)) and signals skip
  // destructors, leaving the file behind. Those orphans are cleaned up by the
  // zombie-aware liveness self-heal in ps.rs::build_queued_table() (BUG-479).
  let _guard         = GateFile( state_path.clone() );
  let wait_start     = std::time::Instant::now();
  let mut gate_emitted = false;
  let mut deadline_emitted = false;

  // Outer loop: each iteration is one full max_attempts-poll sequence.
  // on_exhausted() either returns (retries the sequence) or exits.
  loop
  {
    for attempt in 1..=effective_max_attempts
    {
      // Print-mode only: interactive sessions never contend for a print-mode slot.
      let count = find_claude_processes()
        .iter()
        .filter( | p | classify_mode( &p.args ) == "print" )
        .count();
      let count_u32 = u32::try_from( count ).unwrap_or( u32::MAX );
      // BUG-422: this count can transiently over-read by 1 due to a fork→exec
      // race — a `claude --print` child briefly inherits the parent's cmdline
      // before exec() replaces it; a concurrent /proc scan counts both parent
      // and child, yielding count+1.  Impact is bounded: at most one wasted
      // 30s poll cycle.  Real admission is protected by the slot-file CAS
      // below (acquire_slot()), so no over-admission is possible.  The display
      // count is clamped to max at the eprintln! site further below.
      // Fix(BUG-387): admission now additionally requires winning the atomic
      // reservation at index `count_u32` — see slot_path() for why the index
      // is derived from this same count read instead of a separate counter.
      // A losing race falls through to the existing wait-and-retry tail below,
      // exactly as the old `count >= max` case already did.
      let has_capacity = count_u32 < max;
      // Fix(BUG-404): a denial at the single count-derived index does NOT mean
      // no index anywhere is available — count_u32 is just the live process
      // count, unrelated to which of the 0..max slot indices are actually
      // free or dead-and-reclaimable at this instant. Try count_u32 first
      // (preserves BUG-387's shared-stale-count arbitration for the common
      // contested-same-index case — concurrent racers observing the same
      // stale count still contend for the same primary index first), then
      // fall back to every other index in 0..max before giving up on this
      // attempt.
      // Root cause: wait_for_session_slot() computed and tried exactly one
      // candidate index per attempt; a live (even perfectly healthy) owner at
      // that one index starved the waiter even while other indices sat
      // completely free or dead — confirmed empirically in production (4 of 6
      // real /tmp/clr-gate slots dead-and-untried during a live user report).
      // Pitfall: each acquire_slot() call is independently atomic
      // (create_new); trying several within one attempt introduces no new
      // race — it only widens which single index this attempt can land on.
      // Fix(BUG-480) task/claude_runner/bug/480_gate_diagnostic_hides_slot_occupancy.md: tally denied
      // indices beside the sweep's surviving Result, so the blocking quantity
      // (slot occupancy) survives to the poll line and the exhaustion messages.
      // Root cause: admission is a conjunction (census AND slot-CAS), but this
      // sweep collapsed per-index outcomes into one fieldless Result — every
      // diagnostic then interpolated only the census conjunct's locals, so 66
      // consecutive `active=1/8` polls showed "7 free" while 8/8 slot files
      // were held, the actual blocker appearing on no surface.
      // Pitfall: when an admission predicate is a conjunction, thread at least
      // one measured value from the conjunct that actually failed into every
      // denial diagnostic — a message interpolating only the other conjunct's
      // variables misattributes the denial.
      let mut denied_slots : u32 = 0;
      let claim = if has_capacity
      {
        let mut result = acquire_slot( &dir, count_u32, pid, since, my_starttime, stale_secs );
        if result.is_err()
        {
          denied_slots += 1;
          for candidate in 0..max
          {
            if candidate == count_u32 { continue; }
            result = acquire_slot( &dir, candidate, pid, since, my_starttime, stale_secs );
            if result.is_ok() { break; }
            denied_slots += 1;
          }
        }
        Some( result )
      }
      else { None };
      if let Some( Ok( () ) ) = claim
      {
        // Emit GateWait event if we actually waited at least one poll cycle.
        emit_gate_wait_event( gate_emitted, &wait_start, journal, max, attempt );
        return; // _guard.drop() removes only the {pid}.json telemetry file —
                // the slot reservation from acquire_slot() is deliberately
                // left in place for the rest of this session; see slot_path().
      }
      // Fix(BUG-481): announce the resolved state of both optional gate
      // protections (deadline clamp, staleness reclaim) exactly once per gate
      // entry, on the first DENIED attempt — admission without waiting stays
      // silent (user_story/025 AC-001 promises no gate messages on immediate
      // admission), and retry-restarted sequences do not re-announce.
      // Root cause: every non-engaged resolution of either protection was
      // silent, so a dead deadline clamp and a disabled staleness reclaim
      // were indistinguishable from healthy engaged ones on every surface.
      // Pitfall: emit before the exhaustion branch, not inside the poll-line
      // branch — a max_attempts=1 run exhausts without ever emitting a poll
      // line, and the resolution state matters most on exactly that run.
      if !quiet && !deadline_emitted
      {
        deadline_emitted = true;
        eprintln!(
          "{}gate-deadline  {deadline_state} · stale-reclaim {stale_state}",
          claude_core::trace_ts()
        );
      }
      if attempt == effective_max_attempts
      {
        // Fix(BUG-298): add [Runner] prefix + correct message text to match 14_error_class.md.
        // Root cause: gate-timeout message lacked [Runner] class prefix; display showed no class label.
        // Pitfall: every message-construction site must inject the [Runner] prefix, not only spawn paths.
        // Fix(BUG-299): wrap gate-timeout in retry handling instead of unconditional exit(1).
        // Root cause: gate-timeout path called exit(1) directly; runner retry system not invoked here.
        // Pitfall: every early-exit path (including gate timeouts) must route through `on_exhausted` —
        // the caller's closure decides retry-vs-exit (e.g. run/ask wraps apply_runner_retry(); isolated
        // exits directly), rather than gate.rs hardcoding one policy for every caller.
        // Fix(BUG-433): gate timeout error omitted the "print" qualifier.
        // Root cause: `count` counts only print-mode processes, but the error said
        // "active sessions" — suggesting it counted all sessions (total occupancy).
        // Pitfall: "print sessions" must not be changed back to "active sessions";
        // t09/t29/t31/t16 timeout assertions guard the exact substring.
        // Fix(BUG-480): mirror the measured slot occupancy into both exhaustion
        // messages — only when this final attempt's denial was slot-side (the
        // sweep actually ran and measured). The at-capacity arm (claim == None)
        // never ran the sweep, so denied_slots is unmeasured there and the
        // field is omitted — which also keeps the T29/T31 full-line guards
        // (at-capacity fixtures) byte-identical.
        // Root cause: both messages interpolated only the census conjunct
        // (count/max), hiding the slot-side blocker at exhaustion exactly as
        // the poll line did during the wait.
        // Pitfall: emit `slots=` only for measured sweeps — interpolating an
        // unmeasured 0 on the at-capacity arm would misreport "0 held" while
        // slots may well be occupied.
        let slots_field = match claim
        {
          Some( Err( _ ) ) => format!( ", slots={denied_slots}/{max} held" ),
          _                => String::new(),
        };
        let e = if budget_is_limiting {
          // CLR_REMAINING_TIMEOUT_SECS budget exhausted: emit a distinct diagnostic
          // so operators can identify gate-wait budget exhaustion in job stderr output
          // without counting attempt lines manually (see Fix(BUG-423) in gate_limits.rs).
          std::io::Error::other(
            format!( "gate-wait budget exhausted — {count} print sessions, max-sessions={max}, budget={effective_max_attempts} attempt(s){slots_field}" )
          )
        } else {
          std::io::Error::other(
            format!( "session gate timed out — {count} print sessions, max-sessions={max}{slots_field}" )
          )
        };
        on_exhausted( &e );
        break; // non-exhaustion path: restart outer poll loop
      }
      if !quiet
      {
        // Fix(BUG-393): distinguish global exhaustion (no slot numerically free)
        // from every other non-admission cause — both previously produced
        // byte-identical text since the message only interpolated the
        // count/max counters shared across every false-branch of the
        // compound admission condition above.
        // Fix(BUG-396): the has_capacity-true branch itself further splits
        // into "another live session already holds this index" (confirmed
        // via production evidence to be the overwhelmingly common case: job
        // #40 reported "lost reservation race" at 4/6 sessions while
        // slot_4.json's recorded owner was actually alive — no reclaim was
        // ever attempted, so no race occurred) versus "the recorded owner
        // was dead but I lost the reclaim-ticket race to another concurrent
        // reclaimer" (the only scenario that is genuinely a race; see T14's
        // dead-owner fixture). BUG-393's original fix distinguished capacity
        // from non-capacity only, and mislabeled every non-capacity denial a
        // "race" regardless of which of acquire_slot()'s two distinct
        // `Err` branches actually produced it.
        // Root cause: acquire_slot() returned a bare `bool`, discarding which
        // of its 2 internal denial branches fired; the message site then had
        // no way to tell "owner alive, no race occurred" apart from "owner
        // dead, ticket race lost" and defaulted to naming both a "race".
        // Pitfall: a diagnostic that names a specific mechanism ("race") must
        // be verified against every code path that reaches it — the
        // overwhelmingly common non-admission cause (an unrelated live
        // session already owns this index) never contends with anything at
        // all, and calling it a "race" actively misleads an operator into
        // expecting imminent, capacity-unrelated resolution that may not
        // arrive until that specific session ends. The cause is placed in
        // the `(reason: ...)` trailer of the TSK-452 structured line
        // `"gate-wait  active=X/Y attempt=A/MA wait=Ss (reason: {cause})"`.
        // T01/T04/T27/T32 assert "gate-wait  active="; T02/T03/T06/T28
        // assert absence of "gate-wait"; 5 positive sites in config_file_test.rs
        // assert "gate-wait  active=" anchored to the count ratio.
        let cause = match claim
        {
          Some( Err( SlotDenialCause::HeldByLive ) )        => "slot held by another session",
          Some( Err( SlotDenialCause::LostReclaimRace ) )   => "lost reservation race",
          // None: has_capacity was false. Some(Ok(())): unreachable — the
          // admitted branch already returned above — but required for
          // match exhaustiveness, so it shares the "at capacity" arm rather
          // than duplicating it (clippy::match_same_arms).
          None | Some( Ok( () ) )                           => "at capacity",
        };
        // Fix(BUG-422): clamp display count to max — fork→exec race can produce
        // count > max transiently (see comment at count computation site above).
        // Root cause: /proc scan counts a forked-but-not-yet-exec'd child that
        // briefly inherits the parent's claude --print cmdline; yields count+1.
        // Pitfall: clamp applies to display only — has_capacity and acquire_slot()
        // are intentionally left unchanged; CAS provides the real admission gate.
        let display_count = count_u32.min( max );
        // Fix(BUG-431): gate progress message omitted the "print" qualifier.
        // Root cause: `count` is filtered to print-mode processes only, but the message
        // said "sessions active" — indistinguishable from a total count.
        // TSK-452: format changed to structured timestamp-prefixed line; the mode scope
        // is preserved — `display_count` counts print-mode processes only (same count
        // as before); `t_gate_progress_message_names_print_sessions` now guards
        // the `"gate-wait  active="` label rather than the old `"print sessions active"` phrase.
        // Fix(BUG-480): additive ` slots={denied_slots}/{max}` field after the
        // pinned `active=` ratio — only when this attempt's denial was slot-side
        // (the sweep ran and measured). At-capacity lines never carry the field:
        // denied_slots is unmeasured there, and every pinned `active=N/N`
        // format guard is an at-capacity fixture, so those lines stay
        // byte-identical. See the sweep-site Fix(BUG-480) comment above for
        // root cause and pitfall.
        let slots_field = match claim
        {
          Some( Err( _ ) ) => format!( " slots={denied_slots}/{max}" ),
          _                => String::new(),
        };
        eprintln!(
          "{}gate-wait  active={display_count}/{max}{slots_field} attempt={attempt}/{effective_max_attempts} wait={poll_secs}s (reason: {cause})",
          claude_core::trace_ts()
        );
      }
      gate_emitted = true;
      let _ = std::fs::write(
        &state_path,
        format!( r#"{{"cwd":"{cwd_escaped}","since":{since}{starttime_field},"attempt":{attempt},"message":"waiting for session slot"}}"# ),
      );
      std::thread::sleep( poll );
    }
  }
}
