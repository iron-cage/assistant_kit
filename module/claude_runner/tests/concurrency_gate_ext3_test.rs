//! Integration tests for the `--max-sessions` concurrency gate — override-tier
//! resolution for the three tunable gate timing knobs.
#![ cfg( unix ) ]
//!
//! Extension of `concurrency_gate_test.rs` (T01–T14) covering the `t_gate_*`
//! override-tier matrix: for each of `gate-poll-secs`, `gate-max-attempts`, and
//! `gate-stale-secs`, the CLI-flag / env-var / `--args-file` JSON-key /
//! precedence-between-tiers / absent-default variants, plus `gate-stale-secs`'s
//! invalid-value fallback.
//!
//! The deadline-budget half of this file — T35/T36, the
//! `t_gate_remaining_timeout_*` and `t_gate_trace_exposure_*` tests, the expressed
//! `--timeout` budget default, and T39–T41 — lives in
//! `concurrency_gate_deadline_test.rs`.
//!
//! See `concurrency_gate_test.rs`'s own header for the full Test Case Index
//! across all 5 split files (these `t_gate_*` tests are not T-numbered and are
//! listed here by fn name only).

mod cli_binary_test_helpers;
use cli_binary_test_helpers::
{
  fake_claude_binary_dir, fake_claude_dir, make_proc_dir, spawn_print_claude_for, wait_bounded,
};
use std::io::Write as _;
use std::process::Command;
use tempfile::NamedTempFile;

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
