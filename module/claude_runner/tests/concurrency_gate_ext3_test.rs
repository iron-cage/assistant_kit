//! Integration tests for the `--max-sessions` concurrency gate — override-tier
//! resolution for gate timing knobs (extended).
#![ cfg( unix ) ]
//!
//! Extension of `concurrency_gate_test.rs` (T01–T14) covering T35/T36 (the
//! `CLR_REMAINING_TIMEOUT_SECS` budget clamp on `effective_max_attempts`, and
//! its behavior when the remaining budget is below one poll interval), T39–T41
//! (the BUG-481 resolution-announcement contract: non-engaged deadline states
//! distinguishable and announced, boundary inputs, and the `poll_secs=0`
//! divide-by-zero guard) plus the `t_gate_*` override-tier matrix: for each of `gate-poll-secs`,
//! `gate-max-attempts`, and `gate-stale-secs`, the CLI-flag / env-var /
//! `--args-file` JSON-key / precedence-between-tiers / absent-default variants,
//! plus `gate-stale-secs`'s invalid-value fallback and the remaining-timeout
//! knob's absent-default and non-numeric-fallback variants.
//!
//! See `concurrency_gate_test.rs`'s own header for the full Test Case Index
//! across all 4 split files (these `t_gate_*` tests are not T-numbered and are
//! listed here by fn name only).

mod cli_binary_test_helpers;
use cli_binary_test_helpers::
{
  fake_claude_binary_dir, fake_claude_dir, make_proc_dir, spawn_print_claude_for, wait_bounded,
};
use std::io::Write as _;
use std::process::Command;
use tempfile::NamedTempFile;

// ── T35 / T36: CLR_REMAINING_TIMEOUT_SECS budget clamp (BUG-423 regression) ─

/// T35 (BUG-423): `CLR_REMAINING_TIMEOUT_SECS` clamps `effective_max_attempts` to
/// `floor(remaining / poll_secs).max(1)`.  With remaining=2, poll=1, max=1000
/// the gate must exhaust after exactly 2 attempts, not 1000, and must emit a
/// diagnostic containing "budget" in the error line so operators can distinguish
/// budget-exhaustion from ordinary gate timeout in job stderr.
///
/// ## Root Cause (BUG-423)
///
/// `wait_for_session_slot()` polled up to `CLR_GATE_MAX_ATTEMPTS` (default 1000)
/// with no awareness of any external job-runner deadline, causing gate-wait
/// alone to consume the entire `wplan_executor` budget (observed: 258 × 30s =
/// 7740s exceeded a 7200s wplan timeout).
///
/// ## Why Not Caught
///
/// `CLR_REMAINING_TIMEOUT_SECS` did not exist before this fix; no test could
/// exercise the clamping path.
///
/// ## Fix Applied
///
/// gate.rs reads `CLR_REMAINING_TIMEOUT_SECS` and computes `effective_max_attempts`
/// = `(remaining_secs / poll_secs).max(1)`; the for loop runs to `effective_max_attempts`
/// instead of `max_attempts`; budget exhaustion emits a distinct "gate-wait budget
/// exhausted" diagnostic routed through `on_exhausted` exactly like the normal path.
///
/// ## Prevention
///
/// Assert that stderr contains "budget" and that the gate exits without sleeping
/// 1000 poll intervals (practical time bound: test completes in < 5s).
///
/// ## Pitfall
///
/// The test uses `CLR_REMAINING_TIMEOUT_SECS=2` and `CLR_GATE_POLL_SECS=1` (not the
/// production 60/30 values) to keep the test to 1 inter-attempt sleep of 1 second.
/// The invariant under test — clamping to floor(remaining/poll) — is identical.
// test_kind: bug_reproducer(BUG-423)
#[ test ]
fn t35_remaining_timeout_budget_clamps_gate_attempts()
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
    .env( "CLR_GATE_POLL_SECS",        "1"    )  // 1s poll keeps test to ~1s elapsed
    .env( "CLR_GATE_MAX_ATTEMPTS",     "1000" )  // without budget clamp: 1000 attempts
    .env( "CLR_REMAINING_TIMEOUT_SECS", "2"   )  // floor(2/1)=2 → clamp to 2 attempts
    .output()
    .expect( "invoke clr" );

  let _ = occupier.kill();
  let _ = occupier.wait();

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert_ne!(
    out.status.code(),
    Some( 0 ),
    "T35 (BUG-423): gate must exit non-zero when budget exhausts. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "budget" ),
    "T35 (BUG-423): budget-exhaustion diagnostic must contain \"budget\" so operators \
     can distinguish it from ordinary gate timeout. Got:\n{stderr}"
  );
  assert!(
    !stderr.contains( "session gate timed out" ),
    "T35 (BUG-423): budget-exhaustion must NOT produce the normal \"session gate timed \
     out\" message — the two exhaustion paths must be distinguishable. Got:\n{stderr}"
  );
}

/// T36 (BUG-423): when `CLR_REMAINING_TIMEOUT_SECS` is less than one poll interval,
/// `.max(1)` ensures at least one admission attempt is made before declaring budget
/// exhausted — the gate must not silently skip the admission check entirely.
///
/// ## Root Cause / Fix Applied
///
/// See T35 above. This case exercises the `.max(1)` floor: `floor(1/30) = 0`, but
/// `.max(1)` yields 1, so attempt 1 fires and the exhaustion check immediately
/// follows (no sleep, since sleep happens AFTER the exhaustion check).
///
/// ## Pitfall
///
/// With `effective_max_attempts=1`, the `if attempt == effective_max_attempts` branch
/// fires on the very first attempt, before any `std::thread::sleep(poll)` call.
/// The test therefore completes in < 1s even with `CLR_GATE_POLL_SECS=30`.
// test_kind: bug_reproducer(BUG-423)
#[ test ]
fn t36_remaining_timeout_below_poll_interval_still_makes_one_attempt()
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
    .env( "CLR_GATE_POLL_SECS",        "30"  )  // 30s poll — but .max(1) gives 1 attempt
    .env( "CLR_GATE_MAX_ATTEMPTS",     "1000" ) // without budget clamp: 1000 attempts
    .env( "CLR_REMAINING_TIMEOUT_SECS", "1"   ) // floor(1/30)=0 → .max(1)=1 attempt
    .output()
    .expect( "invoke clr" );

  let _ = occupier.kill();
  let _ = occupier.wait();

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert_ne!(
    out.status.code(),
    Some( 0 ),
    "T36 (BUG-423): gate must exit non-zero even when budget < 1 poll interval. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "budget" ),
    "T36 (BUG-423): budget exhaustion diagnostic must contain \"budget\" even on the \
     single-attempt floor path. Got:\n{stderr}"
  );
}

// ──────────────────────────────────────────────────────────────────────────────
// 082 — --gate-poll-secs edge cases
// ──────────────────────────────────────────────────────────────────────────────

/// 082/EC-1: `--gate-poll-secs 5` CLI flag reduces the wait between gate attempts.
/// Gate-wait diagnostic contains `wait=5s`; exhaustion completes within a 12s
/// deadline (would take ~30s if the 30s default were applied instead).
// test_kind: edge_case
#[ test ]
fn t_gate_poll_secs_cli_flag_reduces_wait_interval()
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
      "--gate-poll-secs", "5", "--journal", "off", "x",
    ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .expect( "spawn clr" );

  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 12 );
  let exited = wait_bounded( &mut child, deadline );
  if exited.is_none() { let _ = child.kill(); }
  let out = child.wait_with_output().expect( "reap clr" );

  let _ = occupier.kill();
  let _ = occupier.wait();

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    exited.is_some(),
    "082/EC-1: gate must exhaust within 12s when --gate-poll-secs 5 is set \
     (would take ~30s with the 30s default). stderr:\n{stderr}"
  );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 1 ),
    "082/EC-1: exit must be 1 once the gate exhausts. stderr: {stderr}"
  );
  assert!(
    stderr.contains( "wait=5s" ),
    "082/EC-1: gate-wait diagnostic must contain `wait=5s` confirming the CLI flag \
     was applied. Got:\n{stderr}"
  );
}

/// 082/EC-2: `CLR_GATE_POLL_SECS=5` env var produces identical behavior to `--gate-poll-secs 5`.
/// Confirms the env-var fallback tier is active for `run`/`ask`.
// test_kind: edge_case
#[ test ]
fn t_gate_poll_secs_env_var_equivalent_to_cli_flag()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
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
    .env( "CLR_GATE_POLL_SECS",    "5" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .expect( "spawn clr" );

  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 12 );
  let exited = wait_bounded( &mut child, deadline );
  if exited.is_none() { let _ = child.kill(); }
  let out = child.wait_with_output().expect( "reap clr" );

  let _ = occupier.kill();
  let _ = occupier.wait();

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    exited.is_some(),
    "082/EC-2: gate must exhaust within 12s when CLR_GATE_POLL_SECS=5. stderr:\n{stderr}"
  );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 1 ),
    "082/EC-2: exit must be 1. stderr: {stderr}"
  );
  assert!(
    stderr.contains( "wait=5s" ),
    "082/EC-2: gate-wait diagnostic must contain `wait=5s` confirming env var applied. Got:\n{stderr}"
  );
}

/// 082/EC-3: When `--gate-poll-secs` and `CLR_GATE_POLL_SECS` are both absent, the
/// 30s default is used. Verified via dry-run: parameter accepted without error,
/// exit 0 (gate never triggers in dry-run mode).
// test_kind: edge_case
#[ test ]
fn t_gate_poll_secs_absent_uses_30s_default()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--dry-run", "--journal", "off", "x" ] )
    .env( "HOME", "/tmp/clr-isolated-home" )
    .env_remove( "CLR_GATE_POLL_SECS" )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "082/EC-3: dry-run with absent --gate-poll-secs must exit 0 (30s default, gate never triggers). \
     stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

/// 082/EC-4: `"gate-poll-secs"` JSON key in `--args-file` is accepted and applied.
/// Same 5s timing behavior as EC-1/EC-2.
// test_kind: edge_case
#[ test ]
fn t_gate_poll_secs_json_key_accepted_via_args_file()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  let mut occupier = spawn_print_claude_for( &occupier_path, 60 );
  let proc = make_proc_dir( &[ occupier.id() ] );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  let mut cfg = NamedTempFile::new().expect( "args-file" );
  write!( cfg, r#"{{"gate-poll-secs": 5}}"# ).expect( "write args-file JSON" );
  let cfg_path = cfg.path().to_str().expect( "args-file path UTF-8" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut child = Command::new( bin )
    .args( [
      "-p", "--max-sessions", "1", "--retry-override", "0",
      "--args-file", cfg_path, "--journal", "off", "x",
    ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .expect( "spawn clr" );

  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 12 );
  let exited = wait_bounded( &mut child, deadline );
  if exited.is_none() { let _ = child.kill(); }
  let out = child.wait_with_output().expect( "reap clr" );

  let _ = occupier.kill();
  let _ = occupier.wait();

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    exited.is_some(),
    "082/EC-4: gate must exhaust within 12s when {{\"gate-poll-secs\":5}} in args-file. stderr:\n{stderr}"
  );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 1 ),
    "082/EC-4: exit must be 1. stderr: {stderr}"
  );
  assert!(
    stderr.contains( "wait=5s" ),
    "082/EC-4: gate-wait diagnostic must contain `wait=5s` confirming JSON key applied. Got:\n{stderr}"
  );
}

/// 082/EC-5: CLI flag takes precedence over env var.
/// `--gate-poll-secs 5` wins over `CLR_GATE_POLL_SECS=60`: gate exhausts in <12s
/// (would take ~60s if the env var won).
// test_kind: edge_case
#[ test ]
fn t_gate_poll_secs_cli_flag_takes_precedence_over_env_var()
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
      "--gate-poll-secs", "5", "--journal", "off", "x",
    ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS",    "60" )  // env: 60s; CLI wins with 5s
    .env( "CLR_GATE_MAX_ATTEMPTS", "2"  )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .expect( "spawn clr" );

  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 12 );
  let exited = wait_bounded( &mut child, deadline );
  if exited.is_none() { let _ = child.kill(); }
  let out = child.wait_with_output().expect( "reap clr" );

  let _ = occupier.kill();
  let _ = occupier.wait();

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    exited.is_some(),
    "082/EC-5: gate must exhaust within 12s when --gate-poll-secs 5 overrides CLR_GATE_POLL_SECS=60 \
     (would take ~60s if env var won). stderr:\n{stderr}"
  );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 1 ),
    "082/EC-5: exit must be 1. stderr: {stderr}"
  );
  assert!(
    stderr.contains( "wait=5s" ),
    "082/EC-5: diagnostic must contain `wait=5s` (CLI value), not `wait=60s` (env var). Got:\n{stderr}"
  );
}

// ──────────────────────────────────────────────────────────────────────────────
// 083 — --gate-max-attempts edge cases
// ──────────────────────────────────────────────────────────────────────────────

/// 083/EC-1: `--gate-max-attempts 2` → gate exhausts after exactly 2 attempts.
/// Only 1 wait-diagnostic line is emitted (attempt=1/2); attempt 2 triggers the
/// exhaustion check before the eprintln runs.
// test_kind: edge_case
#[ test ]
fn t_gate_max_attempts_cli_flag_exhausts_after_n_attempts()
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
      "--gate-max-attempts", "2", "--journal", "off", "x",
    ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
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
    "083/EC-1: gate must exhaust within 10s when --gate-max-attempts 2 with 1s poll. stderr:\n{stderr}"
  );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 1 ),
    "083/EC-1: exit must be 1. stderr: {stderr}"
  );
  assert!(
    stderr.contains( "session gate timed out" ),
    "083/EC-1: exhaustion message must contain \"session gate timed out\". Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "attempt=1/2" ),
    "083/EC-1: gate must show ceiling 2 (not 1000 default); diagnostic `attempt=1/2` expected. Got:\n{stderr}"
  );
}

/// 083/EC-2: `CLR_GATE_MAX_ATTEMPTS=2` env var produces identical behavior to `--gate-max-attempts 2`.
// test_kind: edge_case
#[ test ]
fn t_gate_max_attempts_env_var_equivalent_to_cli_flag()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
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
    .env( "CLR_GATE_POLL_SECS",    "1" )
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
    "083/EC-2: gate must exhaust within 10s when CLR_GATE_MAX_ATTEMPTS=2. stderr:\n{stderr}"
  );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 1 ),
    "083/EC-2: exit must be 1. stderr: {stderr}"
  );
  assert!(
    stderr.contains( "session gate timed out" ),
    "083/EC-2: exhaustion message required. Got:\n{stderr}"
  );
}

/// 083/EC-3: When `--gate-max-attempts` and `CLR_GATE_MAX_ATTEMPTS` are both absent,
/// the 1000-attempt default is used. Verified via dry-run: parameter accepted without
/// error, exit 0 (gate never triggers in dry-run mode).
// test_kind: edge_case
#[ test ]
fn t_gate_max_attempts_absent_uses_1000_default()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--dry-run", "--journal", "off", "x" ] )
    .env( "HOME", "/tmp/clr-isolated-home" )
    .env_remove( "CLR_GATE_MAX_ATTEMPTS" )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "083/EC-3: dry-run with absent --gate-max-attempts must exit 0 (1000-attempt default, gate never triggers). \
     stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

/// 083/EC-4: `"gate-max-attempts"` JSON key in `--args-file` is accepted and applied.
/// Gate exhausts after 2 attempts (same timing as EC-1/EC-2).
// test_kind: edge_case
#[ test ]
fn t_gate_max_attempts_json_key_accepted_via_args_file()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  let mut occupier = spawn_print_claude_for( &occupier_path, 60 );
  let proc = make_proc_dir( &[ occupier.id() ] );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  let mut cfg = NamedTempFile::new().expect( "args-file" );
  write!( cfg, r#"{{"gate-max-attempts": 2}}"# ).expect( "write args-file JSON" );
  let cfg_path = cfg.path().to_str().expect( "args-file path UTF-8" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut child = Command::new( bin )
    .args( [
      "-p", "--max-sessions", "1", "--retry-override", "0",
      "--args-file", cfg_path, "--journal", "off", "x",
    ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
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
    "083/EC-4: gate must exhaust within 10s when {{\"gate-max-attempts\":2}} in args-file. stderr:\n{stderr}"
  );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 1 ),
    "083/EC-4: exit must be 1. stderr: {stderr}"
  );
  assert!(
    stderr.contains( "session gate timed out" ),
    "083/EC-4: exhaustion message required. Got:\n{stderr}"
  );
}

/// 083/EC-5: CLI flag takes precedence over env var.
/// `--gate-max-attempts 2` wins over `CLR_GATE_MAX_ATTEMPTS=100`: gate exhausts
/// within 10s and diagnostic shows ceiling 2 (not 100).
// test_kind: edge_case
#[ test ]
fn t_gate_max_attempts_cli_flag_takes_precedence_over_env_var()
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
      "--gate-max-attempts", "2", "--journal", "off", "x",
    ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS",    "1"   )
    .env( "CLR_GATE_MAX_ATTEMPTS", "100" )  // env: 100 attempts; CLI wins with 2
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
    "083/EC-5: gate must exhaust within 10s when --gate-max-attempts 2 overrides \
     CLR_GATE_MAX_ATTEMPTS=100. stderr:\n{stderr}"
  );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 1 ),
    "083/EC-5: exit must be 1. stderr: {stderr}"
  );
  assert!(
    stderr.contains( "attempt=1/2" ),
    "083/EC-5: diagnostic must show ceiling 2 (CLI value), not 100 (env var). Got:\n{stderr}"
  );
}

// ──────────────────────────────────────────────────────────────────────────────
// 084 — --gate-stale-secs edge cases
// ──────────────────────────────────────────────────────────────────────────────
//
// Fixture: pre-seed `slot_0.json` with a live occupier's PID; use an EMPTY
// proc_dir so count=0 < max=1 (has_capacity=true). acquire_slot() then reads
// the pre-seeded slot and decides reclaim eligibility based on `stale_secs`.
// Mirrors T20's two-phase shape.

/// 084/EC-1: Default (absent) → `CLR_GATE_STALE_SECS`/`--gate-stale-secs` absent.
/// Live owner (`since=0`, maximally stale) is never reclaimed; gate exhausts.
// test_kind: edge_case
#[ test ]
fn t_gate_stale_secs_absent_live_owner_never_reclaimed()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  let mut occupier = spawn_print_claude_for( &occupier_path, 30 );
  let occupier_pid = occupier.id();

  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir  = tempfile::TempDir::new().expect( "proc dir" );

  std::fs::write(
    gate_dir.path().join( "slot_0.json" ),
    format!( r#"{{"pid":{occupier_pid},"since":0}}"# ),
  ).expect( "pre-seed stale live slot" );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut waiter = Command::new( bin )
    .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc_dir.path() )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS",    "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .env_remove( "CLR_GATE_STALE_SECS" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .expect( "spawn clr" );

  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 10 );
  let exited = wait_bounded( &mut waiter, deadline );
  if exited.is_none() { let _ = waiter.kill(); }
  let out = waiter.wait_with_output().expect( "reap clr" );

  let _ = occupier.kill();
  let _ = occupier.wait();

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 1 ),
    "084/EC-1: gate must exhaust (exit 1) when CLR_GATE_STALE_SECS absent — \
     live owner never reclaimed. stderr:\n{stderr}"
  );
  assert!(
    stderr.contains( "slot held by another session" ),
    "084/EC-1: unset stale threshold must not reclaim a live owner; \
     \"slot held by another session\" expected. Got:\n{stderr}"
  );
}

/// 084/EC-2: `--gate-stale-secs 1` CLI flag reclaims a slot whose `since` is 0
/// (elapsed ≈ decades >> 1s threshold). Waiter is admitted immediately → exit 0.
// test_kind: edge_case
#[ test ]
fn t_gate_stale_secs_cli_flag_reclaims_stale_slot()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  let mut occupier = spawn_print_claude_for( &occupier_path, 30 );
  let occupier_pid = occupier.id();

  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir  = tempfile::TempDir::new().expect( "proc dir" );

  std::fs::write(
    gate_dir.path().join( "slot_0.json" ),
    format!( r#"{{"pid":{occupier_pid},"since":0}}"# ),
  ).expect( "pre-seed stale live slot" );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [
      "-p", "--max-sessions", "1", "--retry-override", "0",
      "--gate-stale-secs", "1", "--journal", "off", "x",
    ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc_dir.path() )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS",    "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "5" )
    .output()
    .expect( "invoke clr" );

  let _ = occupier.kill();
  let _ = occupier.wait();

  assert!(
    out.status.success(),
    "084/EC-2: --gate-stale-secs 1 must reclaim the stale slot (since=0) and admit \
     the waiter (exit 0). stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

/// 084/EC-3: `CLR_GATE_STALE_SECS=10` env var reclaims a stale slot — behavior
/// identical to EC-2 but via the env-var tier.
// test_kind: edge_case
#[ test ]
fn t_gate_stale_secs_env_var_reclaims_stale_slot()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  let mut occupier = spawn_print_claude_for( &occupier_path, 30 );
  let occupier_pid = occupier.id();

  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir  = tempfile::TempDir::new().expect( "proc dir" );

  std::fs::write(
    gate_dir.path().join( "slot_0.json" ),
    format!( r#"{{"pid":{occupier_pid},"since":0}}"# ),
  ).expect( "pre-seed stale live slot" );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc_dir.path() )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS",    "1"  )
    .env( "CLR_GATE_MAX_ATTEMPTS", "5"  )
    .env( "CLR_GATE_STALE_SECS",   "10" )
    .output()
    .expect( "invoke clr" );

  let _ = occupier.kill();
  let _ = occupier.wait();

  assert!(
    out.status.success(),
    "084/EC-3: CLR_GATE_STALE_SECS=10 must reclaim the stale slot (since=0) and admit \
     the waiter (exit 0). stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

/// 084/EC-4: `"gate-stale-secs"` JSON key in `--args-file` is accepted and applied.
/// Stale slot reclaimed, waiter admitted → exit 0.
// test_kind: edge_case
#[ test ]
fn t_gate_stale_secs_json_key_accepted_via_args_file()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  let mut occupier = spawn_print_claude_for( &occupier_path, 30 );
  let occupier_pid = occupier.id();

  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir  = tempfile::TempDir::new().expect( "proc dir" );

  std::fs::write(
    gate_dir.path().join( "slot_0.json" ),
    format!( r#"{{"pid":{occupier_pid},"since":0}}"# ),
  ).expect( "pre-seed stale live slot" );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );

  let mut cfg = NamedTempFile::new().expect( "args-file" );
  write!( cfg, r#"{{"gate-stale-secs": 1}}"# ).expect( "write args-file JSON" );
  let cfg_path = cfg.path().to_str().expect( "args-file path UTF-8" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [
      "-p", "--max-sessions", "1", "--retry-override", "0",
      "--args-file", cfg_path, "--journal", "off", "x",
    ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc_dir.path() )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS",    "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "5" )
    .output()
    .expect( "invoke clr" );

  let _ = occupier.kill();
  let _ = occupier.wait();

  assert!(
    out.status.success(),
    "084/EC-4: {{\"gate-stale-secs\":1}} JSON key must reclaim stale slot and admit \
     the waiter (exit 0). stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

/// 084/EC-5: CLI flag takes precedence over env var.
///
/// `CLR_GATE_STALE_SECS=0` (env, reclaims any slot — `elapsed >= 0` is always true)
/// is overridden by `--gate-stale-secs 9999999` (CLI, ~115 days — too high for a
/// freshly-created slot). Pre-seed with a FRESH slot (`since` ≈ now, elapsed ≈ 0s).
/// With CLI winning, the fresh slot is NOT reclaimed → gate exhausts → exit 1.
// test_kind: edge_case
#[ test ]
fn t_gate_stale_secs_cli_flag_takes_precedence_over_env_var()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  let mut occupier = spawn_print_claude_for( &occupier_path, 30 );
  let occupier_pid = occupier.id();

  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir  = tempfile::TempDir::new().expect( "proc dir" );

  // Fresh slot: since ≈ now → elapsed ≈ 0s.
  // CLR_GATE_STALE_SECS=0 (env) would reclaim it (0s >= 0s → true).
  // --gate-stale-secs 9999999 (CLI) would NOT reclaim it (0s < 9999999s → false).
  let since_now = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .map_or( 0, |d| d.as_secs() );
  std::fs::write(
    gate_dir.path().join( "slot_0.json" ),
    format!( r#"{{"pid":{occupier_pid},"since":{since_now}}}"# ),
  ).expect( "pre-seed fresh live slot" );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut waiter = Command::new( bin )
    .args( [
      "-p", "--max-sessions", "1", "--retry-override", "0",
      "--gate-stale-secs", "9999999", "--journal", "off", "x",
    ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc_dir.path() )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS",    "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .env( "CLR_GATE_STALE_SECS",   "0" )  // env: reclaim everything; CLI wins with 9999999s
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .expect( "spawn clr" );

  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 10 );
  let exited = wait_bounded( &mut waiter, deadline );
  if exited.is_none() { let _ = waiter.kill(); }
  let out = waiter.wait_with_output().expect( "reap clr" );

  let _ = occupier.kill();
  let _ = occupier.wait();

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 1 ),
    "084/EC-5: CLI --gate-stale-secs 9999999 must override CLR_GATE_STALE_SECS=0 — \
     fresh slot must NOT be reclaimed, gate must exhaust (exit 1). stderr:\n{stderr}"
  );
}

/// 084/EC-6: `CLR_GATE_STALE_SECS=notanumber` → invalid value resolves to `None`
/// (feature off); live owner never reclaimed; gate exhausts; no crash.
// test_kind: edge_case
#[ test ]
fn t_gate_stale_secs_invalid_value_resolves_to_none()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  let mut occupier = spawn_print_claude_for( &occupier_path, 30 );
  let occupier_pid = occupier.id();

  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir  = tempfile::TempDir::new().expect( "proc dir" );

  std::fs::write(
    gate_dir.path().join( "slot_0.json" ),
    format!( r#"{{"pid":{occupier_pid},"since":0}}"# ),
  ).expect( "pre-seed stale live slot" );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut waiter = Command::new( bin )
    .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc_dir.path() )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS",    "1"          )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2"          )
    .env( "CLR_GATE_STALE_SECS",   "notanumber" )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .expect( "spawn clr" );

  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 10 );
  let exited = wait_bounded( &mut waiter, deadline );
  if exited.is_none() { let _ = waiter.kill(); }
  let out = waiter.wait_with_output().expect( "reap clr" );

  let _ = occupier.kill();
  let _ = occupier.wait();

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 1 ),
    "084/EC-6: invalid CLR_GATE_STALE_SECS must resolve to None (feature off) — \
     live owner not reclaimed, gate exhausts (exit 1). stderr:\n{stderr}"
  );
  assert!(
    !stderr.to_lowercase().contains( "panic" ),
    "084/EC-6: invalid value must fail silently — no panic. Got:\n{stderr}"
  );
}

// ──────────────────────────────────────────────────────────────────────────────
// 085 — CLR_REMAINING_TIMEOUT_SECS edge cases (EC-3 / EC-4)
// EC-1 and EC-2 are implemented as T35 / T36 above.
// ──────────────────────────────────────────────────────────────────────────────

/// 085/EC-3: When `CLR_REMAINING_TIMEOUT_SECS` is absent, no budget clamp is applied.
/// The gate uses the normal `CLR_GATE_MAX_ATTEMPTS` ceiling and emits
/// `"session gate timed out"` (not `"budget"`).
// test_kind: edge_case
#[ test ]
fn t_gate_remaining_timeout_absent_uses_normal_max_attempts()
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
    .env( "CLR_GATE_POLL_SECS",    "1" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .env_remove( "CLR_REMAINING_TIMEOUT_SECS" )
    .output()
    .expect( "invoke clr" );

  let _ = occupier.kill();
  let _ = occupier.wait();

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert_ne!(
    out.status.code(), Some( 0 ),
    "085/EC-3: gate must exhaust (non-zero exit) with absent CLR_REMAINING_TIMEOUT_SECS. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "session gate timed out" ),
    "085/EC-3: absent CLR_REMAINING_TIMEOUT_SECS must use the normal timeout path — \
     \"session gate timed out\" expected (not \"budget\"). Got:\n{stderr}"
  );
  assert!(
    !stderr.contains( "budget" ),
    "085/EC-3: absent CLR_REMAINING_TIMEOUT_SECS must NOT produce budget-exhaustion \
     diagnostic. Got:\n{stderr}"
  );
  // Fix(BUG-481): the off-state is no longer silent — it announces itself once.
  assert!(
    stderr.contains( "off (CLR_REMAINING_TIMEOUT_SECS unset)" ),
    "085/EC-3: the absent var must be announced as off-unset (BUG-481). Got:\n{stderr}"
  );
}

// BUG-481 task/claude_runner/bug/481_silent_off_env_protection_boundary.md — fixed: the off-state
// now announces itself (gate-deadline line, "budget"-free wording preserves the
// assertions below); the added ECs live in T39 (states distinguishable), T40
// (empty/"0"/negative), and T41 (poll_secs=0 divide-by-zero) at end of file.
/// 085/EC-4: Non-numeric `CLR_REMAINING_TIMEOUT_SECS` resolves to `None` (feature off).
/// Gate polling behaves as if the var were absent — normal `CLR_GATE_MAX_ATTEMPTS` ceiling,
/// `"session gate timed out"`, no crash — but the resolution announcement differs:
/// the off-state names the raw unparseable value (BUG-481), so misconfiguration
/// is distinguishable from non-configuration.
// test_kind: edge_case
#[ test ]
fn t_gate_remaining_timeout_non_numeric_resolves_to_none()
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
    .env( "CLR_GATE_POLL_SECS",        "1"          )
    .env( "CLR_GATE_MAX_ATTEMPTS",      "2"          )
    .env( "CLR_REMAINING_TIMEOUT_SECS", "notanumber" )
    .output()
    .expect( "invoke clr" );

  let _ = occupier.kill();
  let _ = occupier.wait();

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert_ne!(
    out.status.code(), Some( 0 ),
    "085/EC-4: gate must exhaust (non-zero exit) with non-numeric CLR_REMAINING_TIMEOUT_SECS. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "session gate timed out" ),
    "085/EC-4: invalid CLR_REMAINING_TIMEOUT_SECS must resolve to None (feature off) — \
     \"session gate timed out\" expected. Got:\n{stderr}"
  );
  assert!(
    !stderr.contains( "budget" ),
    "085/EC-4: invalid CLR_REMAINING_TIMEOUT_SECS must NOT produce budget-exhaustion diagnostic. Got:\n{stderr}"
  );
  assert!(
    !stderr.to_lowercase().contains( "panic" ),
    "085/EC-4: invalid value must not crash — no panic. Got:\n{stderr}"
  );
  // Fix(BUG-481): misconfiguration is announced with the raw value, so it is
  // distinguishable from non-configuration (EC-3's off-unset announcement).
  assert!(
    stderr.contains( r#"off (CLR_REMAINING_TIMEOUT_SECS="notanumber" unparseable)"# ),
    "085/EC-4: the unparseable value must be announced with its raw text (BUG-481). Got:\n{stderr}"
  );
}

// ──────────────────────────────────────────────────────────────────────────────
// 086 — trace_gate_wait_exposure(): --trace-gated gate-wait exposure diagnostic
// (BUG-445 Fix Location #3)
// ──────────────────────────────────────────────────────────────────────────────

/// 086: With `--trace` and a finite `--timeout` but `CLR_REMAINING_TIMEOUT_SECS`
/// unset, `clr` must emit a stderr note before entering gate-wait, warning that
/// `--timeout` will not bound the wait for a `--max-sessions` slot.
///
/// ## Root Cause (BUG-445)
///
/// `--timeout` only ever bounded the post-gate subprocess-execution phase.
/// `CLR_REMAINING_TIMEOUT_SECS` (BUG-423) is the only mechanism that couples
/// gate-wait to an external deadline, and it is opt-in with no default — a
/// caller who sets only `--timeout` gets zero gate-wait protection with no
/// signal that this is happening (confirmed in production: `watchdog.sh`
/// stalls of 9697s/272s/903s against a 60s `--timeout` budget).
///
/// ## Why Not Caught
///
/// No test exercised the combination of `--trace` + finite `--timeout` +
/// unset `CLR_REMAINING_TIMEOUT_SECS` for its own diagnostic value — existing
/// gate tests (T35/T36, 085/EC-3/EC-4) cover the budget-clamp mechanism
/// itself, not whether an unprotected caller is ever told they are unprotected.
///
/// ## Fix Applied
///
/// `trace_gate_wait_exposure()` (gate.rs) is called from both gate call sites
/// (`run_built_command`, `gate_isolated_session`) immediately before
/// `wait_for_session_slot()`. When the gate is active (`max != 0`), `--trace`
/// is set, and the caller's timeout is finite but `CLR_REMAINING_TIMEOUT_SECS`
/// does not resolve to `Some`, it prints a stderr note naming the exposure.
///
/// ## Prevention
///
/// Assert the exact warning substring appears on stderr for the unprotected
/// case, and is absent for each guard condition (see the sibling edge-case
/// tests below).
///
/// ## Pitfall
///
/// The diagnostic fires unconditionally on flag/env state alone — it does
/// NOT require actual gate contention (no occupier process is needed in this
/// test). Fix Location #3 warns about *exposure* (an unbounded ceiling that
/// would apply *if* this invocation had to queue), not actual queuing.
// test_kind: bug_reproducer(BUG-445)
#[ test ]
fn t_gate_trace_exposure_warns_when_remaining_timeout_unset()
{
  let proc = make_proc_dir( &[] );
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [
      "-p", "--max-sessions", "1", "--timeout", "5", "--trace",
      "--retry-override", "0", "--journal", "off", "x",
    ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env_remove( "CLR_REMAINING_TIMEOUT_SECS" )
    .output()
    .expect( "invoke clr" );

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains(
      "Trace: --timeout is set but CLR_REMAINING_TIMEOUT_SECS is unset"
    ),
    "086: --trace + finite --timeout + unset CLR_REMAINING_TIMEOUT_SECS must \
     warn about unbounded gate-wait exposure (BUG-445). Got:\n{stderr}"
  );
}

/// 086/EC-1: Non-numeric `CLR_REMAINING_TIMEOUT_SECS` must warn with the raw
/// value quoted, distinguishing misconfiguration from non-configuration —
/// mirrors 085/EC-4's distinction for the budget-clamp diagnostic.
// test_kind: edge_case
#[ test ]
fn t_gate_trace_exposure_warns_when_remaining_timeout_non_numeric()
{
  let proc = make_proc_dir( &[] );
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [
      "-p", "--max-sessions", "1", "--timeout", "5", "--trace",
      "--retry-override", "0", "--journal", "off", "x",
    ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_REMAINING_TIMEOUT_SECS", "notanumber" )
    .output()
    .expect( "invoke clr" );

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains(
      "Trace: --timeout is set but CLR_REMAINING_TIMEOUT_SECS is set but unparseable (\"notanumber\")"
    ),
    "086/EC-1: non-numeric CLR_REMAINING_TIMEOUT_SECS must warn with the raw \
     value quoted. Got:\n{stderr}"
  );
}

/// 086/EC-2: A valid, parseable `CLR_REMAINING_TIMEOUT_SECS` means gate-wait
/// IS bounded — no warning.
// test_kind: edge_case
#[ test ]
fn t_gate_trace_exposure_silent_when_remaining_timeout_set()
{
  let proc = make_proc_dir( &[] );
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [
      "-p", "--max-sessions", "1", "--timeout", "5", "--trace",
      "--retry-override", "0", "--journal", "off", "x",
    ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_REMAINING_TIMEOUT_SECS", "60" )
    .output()
    .expect( "invoke clr" );

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "Trace: --timeout is set but" ),
    "086/EC-2: a set, parseable CLR_REMAINING_TIMEOUT_SECS must suppress the \
     exposure warning — gate-wait is actually bounded. Got:\n{stderr}"
  );
}

/// 086/EC-3: `--timeout 0` is an explicit unlimited opt-out (matching
/// `036_timeout.md`/`020_timeout.md` semantics) — warning about unbounded
/// gate-wait would be noise when the caller already declined any timeout
/// bound at all.
// test_kind: edge_case
#[ test ]
fn t_gate_trace_exposure_silent_when_timeout_zero()
{
  let proc = make_proc_dir( &[] );
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [
      "-p", "--max-sessions", "1", "--timeout", "0", "--trace",
      "--retry-override", "0", "--journal", "off", "x",
    ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env_remove( "CLR_REMAINING_TIMEOUT_SECS" )
    .output()
    .expect( "invoke clr" );

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "Trace: --timeout is set but" ),
    "086/EC-3: --timeout 0 is an explicit unlimited opt-out — must not warn. \
     Got:\n{stderr}"
  );
}

/// 086/EC-4: Without `--trace`, no diagnostics are printed at all — the
/// feature is opt-in via `--trace`, matching every other `--trace`-gated
/// diagnostic in this CLI.
// test_kind: edge_case
#[ test ]
fn t_gate_trace_exposure_silent_without_trace_flag()
{
  let proc = make_proc_dir( &[] );
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [
      "-p", "--max-sessions", "1", "--timeout", "5",
      "--retry-override", "0", "--journal", "off", "x",
    ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env_remove( "CLR_REMAINING_TIMEOUT_SECS" )
    .output()
    .expect( "invoke clr" );

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "Trace:" ),
    "086/EC-4: without --trace, no trace diagnostics (including the gate-wait \
     exposure note) may print. Got:\n{stderr}"
  );
}

// ── T39–T41: silent-off resolution boundary of the optional gate protections
// (BUG-481) ───────────────────────────────────────────────────────────────────

/// T39 (BUG-481): the three non-engaged resolution states of
/// `CLR_REMAINING_TIMEOUT_SECS` — unset, unparseable, and set-but-nonlimiting —
/// must be mutually distinguishable on stderr, and each must announce itself.
/// Pre-fix all three produced byte-identical gate output (modulo timestamps):
/// the deadline clamp could be dead while every surface looked healthy.
///
/// ## Root Cause (BUG-481)
///
/// `effective_gate_attempts()` resolved `CLR_REMAINING_TIMEOUT_SECS` through
/// env read → parse → strict-`<` selection with zero diagnostic on every
/// non-engaged path, so misconfiguration, non-configuration, and
/// correct-but-nonlimiting configuration converged to one indistinguishable
/// output surface — while the same file recovers invalid input to safe
/// defaults for its two always-on knobs (`gate_max_attempts_from`,
/// `gate_poll_secs_from`).
///
/// ## Why Not Caught
///
/// EC-4 pinned silent-off as intended (`!stderr.contains("budget")`, "must
/// fail silently"); no test diffed gate output across the non-engaged env
/// states, and no diagnostic existed for any test to assert on.
///
/// ## Fix Applied
///
/// `effective_gate_attempts()` now also returns a resolution-state string
/// (off-unset / off-unparseable / nonlimiting / engaged, naming the raw value
/// where present); `wait_for_session_slot()` emits it once per gate entry, on
/// the first denied attempt, joined with the staleness-reclaim state:
/// `"{ts}gate-deadline  {state} · stale-reclaim {state}"`. Admission without
/// waiting stays silent (AC-001), and the text avoids the `"budget"` substring
/// so EC-3/EC-4's feature-off assertions still hold.
///
/// ## Prevention
///
/// Every optional-protection resolver must emit its resolution state (raw
/// input, parse outcome, engaged-or-off) exactly once at resolution time — an
/// off-state must be distinguishable from an on-state on a surface the
/// operator reads.
///
/// ## Pitfall
///
/// A protection that fails to engage must say so — when misconfiguration,
/// non-configuration, and correct-but-nonlimiting configuration all produce
/// identical output, the absence of a diagnostic reads as health, and the
/// feature's death is discovered only by the incident it existed to prevent.
// test_kind: bug_reproducer(BUG-481)
#[ test ]
fn t39_deadline_resolution_states_announced_and_distinguishable()
{
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );

  // One denied gate run per env state; fresh dirs per leg so legs cannot
  // interfere. The sole slot is held by this test process itself (alive for
  // the whole run, no child to manage) with an empty census — the denial
  // cause is irrelevant to deadline resolution, this fixture is just the
  // cheapest deterministic denial.
  let run_leg = | remaining : Option< &str > | -> String
  {
    let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
    let proc_dir = tempfile::TempDir::new().expect( "proc dir" );
    let owner_pid = std::process::id();
    std::fs::write(
      gate_dir.path().join( "slot_0.json" ),
      format!( r#"{{"pid":{owner_pid},"since":0}}"# ),
    ).expect( "pre-seed live-owner slot file" );

    let bin = env!( "CARGO_BIN_EXE_clr" );
    let mut cmd = Command::new( bin );
    cmd
      .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
      .env( "PATH", &script_path )
      .env( "CLR_PROC_DIR", proc_dir.path() )
      .env( "CLR_GATE_DIR", gate_dir.path() )
      .env( "CLR_GATE_POLL_SECS", "1" )
      .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
      .env_remove( "CLR_GATE_STALE_SECS" );
    match remaining
    {
      Some( v ) => { cmd.env( "CLR_REMAINING_TIMEOUT_SECS", v ); },
      None      => { cmd.env_remove( "CLR_REMAINING_TIMEOUT_SECS" ); },
    }
    let out = cmd.output().expect( "invoke clr" );
    assert!(
      !out.status.success(),
      "T39 (BUG-481): every leg's fixture must deny admission ({remaining:?})"
    );
    String::from_utf8_lossy( &out.stderr ).into_owned()
  };

  let unset       = run_leg( None );
  let unparseable = run_leg( Some( "notanumber" ) );
  let nonlimiting = run_leg( Some( "30000" ) ); // floor(30000/1)=30000 >= 2 → never clamps

  let deadline_line = | stderr : &str, leg : &str | -> String
  {
    stderr.lines()
      .find( | l | l.contains( "gate-deadline" ) )
      .unwrap_or_else( || panic!( "T39 (BUG-481): {leg} leg must announce its resolution. stderr:\n{stderr}" ) )
      .to_string()
  };
  let line_unset       = deadline_line( &unset, "unset" );
  let line_unparseable = deadline_line( &unparseable, "unparseable" );
  let line_nonlimiting = deadline_line( &nonlimiting, "nonlimiting" );

  assert!(
    line_unset.contains( "off (CLR_REMAINING_TIMEOUT_SECS unset)" ),
    "T39 (BUG-481): unset leg must name the off-unset state. Line:\n{line_unset}"
  );
  assert!(
    line_unset.contains( "stale-reclaim off" ),
    "T39 (BUG-481): the same announcement must carry the staleness-reclaim \
     off-state (CLR_GATE_STALE_SECS removed in this fixture). Line:\n{line_unset}"
  );
  assert!(
    line_unparseable.contains( r#"off (CLR_REMAINING_TIMEOUT_SECS="notanumber" unparseable)"# ),
    "T39 (BUG-481): unparseable leg must name the raw value and the parse \
     outcome. Line:\n{line_unparseable}"
  );
  assert!(
    line_nonlimiting.contains( "nonlimiting (30000s covers all 2 attempts)" ),
    "T39 (BUG-481): set-but-nonlimiting leg must name the strict-< silence \
     explicitly. Line:\n{line_nonlimiting}"
  );

  // The MRE's core assertion: the three states are mutually distinguishable.
  assert_ne!(
    line_unset, line_unparseable,
    "T39 (BUG-481): unset and unparseable must not be output-indistinguishable"
  );
  assert_ne!(
    line_unset, line_nonlimiting,
    "T39 (BUG-481): unset and nonlimiting must not be output-indistinguishable"
  );
  assert_ne!(
    line_unparseable, line_nonlimiting,
    "T39 (BUG-481): unparseable and nonlimiting must not be output-indistinguishable"
  );

  // Resolution semantics stay feature-off: all three legs exhaust on the
  // normal (non-budget) path.
  for ( stderr, leg ) in [ ( &unset, "unset" ), ( &unparseable, "unparseable" ), ( &nonlimiting, "nonlimiting" ) ]
  {
    assert!(
      stderr.contains( "session gate timed out" ),
      "T39 (BUG-481): {leg} leg must still exhaust on the normal timeout path. stderr:\n{stderr}"
    );
  }
}

/// T40 (BUG-481 edge matrix): boundary inputs of `CLR_REMAINING_TIMEOUT_SECS`
/// resolve and announce deterministically — empty string and negative values
/// are unparseable (feature off, announced with the raw value); `"0"` parses
/// and engages with the documented `.max(1)` one-attempt floor (BUG-423
/// pitfall), taking the budget-exhaustion path.
// test_kind: edge_case
#[ test ]
fn t40_deadline_boundary_inputs_resolve_and_announce()
{
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );

  let run_leg = | remaining : &str | -> String
  {
    let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
    let proc_dir = tempfile::TempDir::new().expect( "proc dir" );
    let owner_pid = std::process::id();
    std::fs::write(
      gate_dir.path().join( "slot_0.json" ),
      format!( r#"{{"pid":{owner_pid},"since":0}}"# ),
    ).expect( "pre-seed live-owner slot file" );

    let bin = env!( "CARGO_BIN_EXE_clr" );
    let out = Command::new( bin )
      .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
      .env( "PATH", &script_path )
      .env( "CLR_PROC_DIR", proc_dir.path() )
      .env( "CLR_GATE_DIR", gate_dir.path() )
      .env( "CLR_GATE_POLL_SECS", "1" )
      .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
      .env_remove( "CLR_GATE_STALE_SECS" )
      .env( "CLR_REMAINING_TIMEOUT_SECS", remaining )
      .output()
      .expect( "invoke clr" );
    assert!(
      !out.status.success(),
      "T40 (BUG-481): every leg's fixture must deny admission ({remaining:?})"
    );
    String::from_utf8_lossy( &out.stderr ).into_owned()
  };

  let empty = run_leg( "" );
  assert!(
    empty.contains( r#"off (CLR_REMAINING_TIMEOUT_SECS="" unparseable)"# ),
    "T40 (BUG-481): empty string must resolve off and announce it. stderr:\n{empty}"
  );
  assert!(
    empty.contains( "session gate timed out" ),
    "T40 (BUG-481): empty string stays feature-off (normal timeout path). stderr:\n{empty}"
  );

  let negative = run_leg( "-5" );
  assert!(
    negative.contains( r#"off (CLR_REMAINING_TIMEOUT_SECS="-5" unparseable)"# ),
    "T40 (BUG-481): negative value must resolve off (u64 parse) and announce it. stderr:\n{negative}"
  );
  assert!(
    negative.contains( "session gate timed out" ),
    "T40 (BUG-481): negative value stays feature-off (normal timeout path). stderr:\n{negative}"
  );

  let zero = run_leg( "0" );
  assert!(
    zero.contains( "engaged (0s clamps to 1 of 2 attempts)" ),
    "T40 (BUG-481): \"0\" parses, engages, and floors to 1 attempt (.max(1), \
     BUG-423 pitfall) — the announcement must say so. stderr:\n{zero}"
  );
  assert!(
    zero.contains( "gate-wait budget exhausted" ),
    "T40 (BUG-481): an engaged clamp exhausts on the budget path. stderr:\n{zero}"
  );
}

/// T41 (BUG-481): `CLR_GATE_POLL_SECS=0` combined with a numeric
/// `CLR_REMAINING_TIMEOUT_SECS` must not crash. Pre-fix,
/// `effective_gate_attempts()` computed `remaining / poll_secs` with an
/// unguarded divisor: `poll_secs=0` is accepted by the parser, so a numeric
/// env value reached an integer divide-by-zero panic — an env-dependent
/// boundary never evaluated when the var is unset.
///
/// ## Root Cause (BUG-481)
///
/// The clamp divisor used raw `poll_secs`; `gate_poll_secs_from` accepts `"0"`
/// (any parseable u64), and the division only runs when
/// `CLR_REMAINING_TIMEOUT_SECS` parses — the panic path needed both knobs set
/// and no test combined them.
///
/// ## Why Not Caught
///
/// t35/t36/EC-3/EC-4 cover limiting/floor/absent/non-numeric with poll >= 1;
/// no test drove the divisor to zero while the budget path was live.
///
/// ## Fix Applied
///
/// `remaining / poll_secs.max( 1 )` — the divisor is floored to 1 second for
/// the quotient only; the gate's actual sleep cadence is unchanged.
///
/// ## Prevention
///
/// Every env-derived divisor must be range-guarded at the division site, not
/// only at parse time — parse acceptance is not arithmetic safety.
///
/// ## Pitfall
///
/// A divide-by-zero that requires TWO independently-valid env values is
/// invisible to per-knob edge-case tests — boundary coverage must include the
/// cross-product of env knobs that meet in one expression.
// test_kind: bug_reproducer(BUG-481)
#[ test ]
fn t41_poll_secs_zero_with_numeric_budget_does_not_panic()
{
  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let proc_dir = tempfile::TempDir::new().expect( "proc dir" );
  let owner_pid = std::process::id();
  std::fs::write(
    gate_dir.path().join( "slot_0.json" ),
    format!( r#"{{"pid":{owner_pid},"since":0}}"# ),
  ).expect( "pre-seed live-owner slot file" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc_dir.path() )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "0" )
    .env( "CLR_GATE_MAX_ATTEMPTS", "2" )
    .env_remove( "CLR_GATE_STALE_SECS" )
    .env( "CLR_REMAINING_TIMEOUT_SECS", "10" )
    .output()
    .expect( "invoke clr" );

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.to_lowercase().contains( "panic" ),
    "T41 (BUG-481): poll_secs=0 with a numeric budget must not divide by zero. stderr:\n{stderr}"
  );
  assert!(
    !out.status.success(),
    "T41 (BUG-481): fixture must deny admission and exhaust normally. stderr:\n{stderr}"
  );
  assert!(
    stderr.contains( "nonlimiting (10s covers all 2 attempts)" ),
    "T41 (BUG-481): floor(10/max(0,1))=10 >= 2 — announced as nonlimiting. stderr:\n{stderr}"
  );
  assert!(
    stderr.contains( "session gate timed out" ),
    "T41 (BUG-481): nonlimiting resolution exhausts on the normal timeout path. stderr:\n{stderr}"
  );
}
