//! Session-gate knob resolution: poll interval, attempt ceiling, staleness threshold,
//! and the external deadline budget that clamps the attempt ceiling.
//!
//! Split out of `gate.rs` (which was over the line-count guideline) — every resolver here
//! is pure (raw env-var string in, value out), alongside the two paths that announce which
//! state those knobs actually resolved to (BUG-481's clamp state, BUG-445's exposure note).

/// Resolve the attempt-limit override from a raw env var string. Pure — no I/O —
/// so the parse-or-default fallback can be unit-tested directly. This crate's
/// tests never call `std::env::set_var` (see `tests/env_var_test.rs`); taking the raw
/// value as a parameter instead of reading the environment internally is what makes
/// that possible here, and means `remove_var` is never needed either.
#[ inline ]
#[ must_use ]
pub fn gate_max_attempts_from( raw : Option< &str > ) -> u32
{
  raw.and_then( | s | s.parse().ok() ).unwrap_or( 1000 )
}

/// Resolve the gate poll interval (seconds) from a raw env var string. Pure — no I/O.
/// Default: 30 seconds. Sibling of [`gate_max_attempts_from`] — see its doc for why
/// this takes the raw value as a parameter instead of reading the environment directly.
///
/// This is the one-shot resolver: `isolated` (which has no CLI flag or config-file tier
/// for this knob) calls it directly against the raw env var. `run`/`ask` instead resolve
/// `CliArgs.gate_poll_secs` through the full CLI > `--args-file` JSON > env var > config.toml
/// tier chain in `apply_env_vars()`/`apply_config_defaults()` (src/cli/env.rs, src/cli/config.rs),
/// falling back to this same 30s default only once every tier has been checked.
#[ inline ]
#[ must_use ]
pub fn gate_poll_secs_from( raw : Option< &str > ) -> u64
{
  raw.and_then( | s | s.parse().ok() ).unwrap_or( 30 )
}

/// Resolve the staleness threshold (seconds) from a raw env var string. Pure — no I/O.
/// `None` (unset or invalid) disables staleness-based reclaim entirely — there is no
/// safe numeric default (see the `Fix(BUG-400)` note in `gate_slot.rs` for why). Sibling of
/// [`gate_max_attempts_from`]; same one-shot-vs-tiered split as [`gate_poll_secs_from`]
/// applies — `isolated` calls this directly, `run`/`ask` resolve through the tier chain.
#[ inline ]
#[ must_use ]
pub fn gate_stale_secs_from( raw : Option< &str > ) -> Option< u64 >
{
  raw.and_then( | s | s.parse().ok() )
}

/// Resolve the remaining external timeout budget (seconds) from a raw env var string.
/// Pure — no I/O. `None` (unset or non-numeric) means no external budget is imposed —
/// polling is limited only by `CLR_GATE_MAX_ATTEMPTS`. Unlike the other gate knobs,
/// this value is env-var-only (no CLI flag, no config-file tier): it is set by a
/// wrapping job runner (e.g. `wplan_executor`) before spawning `clr`, not by the
/// operator configuring `clr` itself. See `wait_for_session_slot()`'s `Fix(BUG-423)`
/// note in `gate.rs` for the full semantics and clamping behaviour.
#[ inline ]
#[ must_use ]
pub fn gate_remaining_timeout_secs_from( raw : Option< &str > ) -> Option< u64 >
{
  raw.and_then( | s | s.parse().ok() )
}

// Fix(BUG-481) task/claude_runner/bug/481_silent_off_env_protection_boundary.md: resolve the
// deadline clamp WITH a resolution-state string, so the caller can announce
// which state the protection landed in (off-unset / off-unparseable /
// nonlimiting / engaged) instead of resolving silently.
// Root cause: env read → parse → strict-< selection carried zero diagnostic on
// every non-engaged path, so misconfiguration ("notanumber"), non-configuration
// (unset), and correct-but-nonlimiting configuration (quotient >= max_attempts)
// were output-indistinguishable — the sole pre-admission deadline mechanism
// could be dead while every surface looked healthy (Job #424: 66 polls
// advertising /1000 inside a 7200s external kill window). The same file
// recovers invalid input to safe defaults for its always-on knobs
// (gate_max_attempts_from/gate_poll_secs_from); only the optional protections
// switched off silently.
// Pitfall: an optional safety protection must announce its resolution state
// (raw input, parse outcome, engaged-or-off) exactly once, on a surface the
// operator reads — a silent off-state reads as health until the incident the
// feature existed to prevent. And: the state text must avoid the "budget"
// substring, which EC-3/EC-4 pin as engaged-path-only.
pub( super ) fn effective_gate_attempts( max_attempts : u32, poll_secs : u64, caller_timeout_secs : u64 ) -> ( u32, bool, String )
{
  // Fix(BUG-423): clamp effective_max_attempts to the remaining external timeout budget
  // so the gate does not outlive a wrapping job-runner deadline (e.g. wplan_executor).
  // Root cause: wait_for_session_slot() had no knowledge of any external timeout budget —
  // it polled to CLR_GATE_MAX_ATTEMPTS regardless of how much wall-clock time the
  // surrounding job was permitted to consume, causing gate-wait alone to exhaust
  // multi-hour job timeouts (observed: 258 attempts × 30s = 7740s > 7200s budget).
  // Pitfall: apply .max(1) so at least one admission attempt is always made before
  // declaring budget exhausted — a remaining budget below one poll interval must not
  // silently bypass the gate check entirely.
  // Fix(BUG-481): .max( 1 ) on the divisor — gate_poll_secs_from accepts "0"
  // (any parseable u64), and this division only runs when the env var parses
  // numeric, so poll_secs=0 reached an integer divide-by-zero here.
  // Root cause: the divisor was range-guarded at neither parse nor use site;
  // the panic needed two independently-valid env values meeting in one
  // expression, invisible to per-knob edge-case tests (T41 pins it).
  // Pitfall: parse acceptance is not arithmetic safety — guard env-derived
  // divisors at the division site. The floor affects the quotient only; the
  // gate's actual sleep cadence is untouched.
  // Fix(BUG-445): when CLR_REMAINING_TIMEOUT_SECS does not parse, an EXPRESSED
  // caller timeout (--timeout flag or CLR_TIMEOUT env; callers pass 0 for
  // unexpressed or an explicit `--timeout 0` opt-out) defaults the budget, so
  // `--timeout N` alone bounds total exposure instead of only the post-gate
  // execution phase. A parseable env var still wins — it is the deliberate
  // per-dispatch coupling signal (BUG-423); the flag is its fallback.
  // Root cause: the gate's sole timing input was the opt-in env var; a caller
  // expressing `--timeout N` as a total budget got zero gate-wait protection
  // (watchdog.sh health probes stalled 9697s/272s/903s against 60s).
  // Pitfall: only EXPRESSED timeouts may default the budget — the built-in
  // defaults (isolated 30s; print-mode's is 0 since TSK-503, but the
  // _CLR_DEFAULT_TIMEOUT test hook can still arm one) must never reach this
  // fallback, or a default invocation flips from queue-patiently (~8.3h
  // ceiling) to fail-fast. The state string names the source so env-vs-flag budgets stay
  // distinguishable; an unparseable env value masked by the fallback is still
  // reported (BUG-481's misconfiguration-visibility invariant).
  let raw      = std::env::var( "CLR_REMAINING_TIMEOUT_SECS" ).ok();
  let parsed   = gate_remaining_timeout_secs_from( raw.as_deref() );
  let from_env = parsed.is_some();
  let fallback = if caller_timeout_secs > 0 { Some( caller_timeout_secs ) } else { None };
  let budget_secs = parsed.or( fallback );
  let budget = budget_secs.map( | remaining | {
    u32::try_from( remaining / poll_secs.max( 1 ) ).unwrap_or( u32::MAX ).max( 1 )
  } );
  let effective = match budget {
    Some( b ) if b < max_attempts => b,
    _                             => max_attempts,
  };
  let limiting = effective < max_attempts;
  let state = match ( raw.as_deref(), budget_secs )
  {
    ( None, None )        => "off (CLR_REMAINING_TIMEOUT_SECS unset)".to_string(),
    ( Some( r ), None )   => format!( "off (CLR_REMAINING_TIMEOUT_SECS={r:?} unparseable)" ),
    ( raw_opt, Some( s ) ) =>
    {
      let source = if from_env { "" } else { " from --timeout" };
      let masked = match ( from_env, raw_opt )
      {
        ( false, Some( r ) ) => format!( "; CLR_REMAINING_TIMEOUT_SECS={r:?} unparseable" ),
        _                    => String::new(),
      };
      if limiting
      { format!( "engaged ({s}s{source} clamps to {effective} of {max_attempts} attempts{masked})" ) }
      else
      { format!( "nonlimiting ({s}s{source} covers all {max_attempts} attempts{masked})" ) }
    },
  };
  ( effective, limiting, state )
}

// Fix(BUG-445): --trace-gated stderr note when a caller expressed NO timeout
// (no --timeout flag, no CLR_TIMEOUT) and CLR_REMAINING_TIMEOUT_SECS is
// unusable, so gate-wait (if entered) has no deadline bound at all.
// Root cause: --timeout only bounded the post-gate execution phase; originally
// this warned callers who SET --timeout that gate-wait ignored it. Fix
// Location #2 made an expressed --timeout default the gate budget, so those
// callers are now protected and the old warning would be false for them —
// unbounded exposure remains only for callers who expressed nothing.
// Pitfall: only fires when the gate is actually active (max != 0) and the
// caller expressed no timeout — an explicit `--timeout 0` is a deliberate
// unlimited opt-out (timeout_expressed = true), and warning a caller who
// already declined any bound would be noise, not signal.
pub( super ) fn trace_gate_wait_exposure( max : u32, trace : bool, timeout_expressed : bool )
{
  if max == 0 || !trace || timeout_expressed { return; }
  let raw = std::env::var( "CLR_REMAINING_TIMEOUT_SECS" ).ok();
  if gate_remaining_timeout_secs_from( raw.as_deref() ).is_some() { return; }
  let why = match raw
  {
    None      => "unset".to_string(),
    Some( r ) => format!( "set but unparseable ({r:?})" ),
  };
  eprintln!(
    "Trace: gate-wait is unbounded — no --timeout given and CLR_REMAINING_TIMEOUT_SECS is {why}; \
     if this invocation queues for a --max-sessions slot, total exposure could reach the full \
     gate-wait ceiling (CLR_GATE_POLL_SECS x CLR_GATE_MAX_ATTEMPTS, ~8.3h by default). Pass \
     --timeout N (it also bounds gate-wait) or set CLR_REMAINING_TIMEOUT_SECS. See \
     docs/cli/param/033_max_sessions.md."
  );
}
