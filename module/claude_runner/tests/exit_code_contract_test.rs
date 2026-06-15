#![ allow( clippy::doc_markdown ) ] // test doc comments use code identifiers in prose
//! CLR-Layer Exit Code Contract Integration Tests
//!
//! ## Purpose
//!
//! Verify EC-1 through EC-3 from `tests/docs/invariant/06_exit_codes.md`.
//! Tests validate that CLR-layer exit codes are correct at the binary level.
//!
//! ## Test Layout
//!
//! - EC-1: timeout watchdog produces exit 4 (not exit 2 — disambiguation from RateLimit)
//! - EC-2: `--expect` mismatch produces exit 3
//! - EC-3: gate bypass (`--max-sessions 0`) exits 0 on subprocess success
//!
//! ## Corner Cases Covered
//!
//! - EC-1: fake subprocess sleeping 10s; `--timeout 1` → exit 4; stderr has "Error: timeout after 1s"
//! - EC-2: fake subprocess printing "foo"; `--expect "bar"` → exit 3
//! - EC-3: fake subprocess exiting 0; `--max-sessions 0` → exit 0
//!
//! ## Architectural Constraint
//!
//! EC-1 requires a real sleeping process killed by the CLR watchdog — the subprocess must NOT
//! be mocked. EC-2 requires a fake claude that prints output not matching the `--expect` pattern.
//! EC-3 requires a fake claude that exits 0 with `--max-sessions 0` to confirm gate bypass does
//! not interfere with the exit code. All tests use `--max-sessions 0` to bypass the session gate.

#![ cfg( unix ) ]

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ fake_claude_dir, run_with_path };

// ── EC-1: Timeout → exit 4 ────────────────────────────────────────────────────

/// EC-1: fake subprocess sleeping 10s; `--timeout 1 --max-sessions 0 -p "x"` → exit 4.
///
/// Root Cause: poll_timeout() used exit(2) — colliding with RateLimit exit code
/// Why Not Caught: no dedicated exit-code-contract test existed; classification tests only verified subprocess exit codes
/// Fix Applied: poll_timeout() in execution.rs changed to exit(4) (TSK-202)
/// Prevention: this integration test ensures the binary-level exit code is correct
/// Pitfall: polling at 50ms intervals — kill fires up to 50ms after deadline; allow 5s wall time
#[ test ]
fn ec_01_timeout_exits_4()
{
  let ( _dir, path_val ) = fake_claude_dir( "sleep 10" );

  let out = run_with_path(
    &[ "-p", "--timeout", "1", "--max-sessions", "0", "--retry-override", "0", "x" ],
    &path_val,
  );

  assert_eq!(
    out.status.code(),
    Some( 4 ),
    "CLR timeout watchdog must exit 4 (not 2). Got: {:?}. stderr: {}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "timeout after 1s" ),
    "stderr must contain 'timeout after 1s'. Got:\n{stderr}"
  );
}

// ── EC-2: Expect mismatch → exit 3 ───────────────────────────────────────────

/// EC-2: fake claude prints "foo"; `--expect "bar" --max-sessions 0 -p "x"` → exit 3.
///
/// Verifies that expect-validation mismatch produces exit 3 exclusively (invariant 006 Rule 4).
#[ test ]
fn ec_02_expect_mismatch_exits_3()
{
  let ( _dir, path_val ) = fake_claude_dir( "printf 'foo'" );

  let out = run_with_path(
    &[ "-p", "--expect", "bar", "--max-sessions", "0", "x" ],
    &path_val,
  );

  assert_eq!(
    out.status.code(),
    Some( 3 ),
    "expect-validation mismatch must exit 3. Got: {:?}. stderr: {}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-3: Gate bypass → exit 0 ───────────────────────────────────────────────

/// EC-3: fake claude exits 0; `--max-sessions 0 -p "x"` → exit 0.
///
/// Verifies gate bypass (`--max-sessions 0`) does not interfere with successful subprocess exit.
#[ test ]
fn ec_03_gate_bypass_exits_0()
{
  let ( _dir, path_val ) = fake_claude_dir( "exit 0" );

  let out = run_with_path(
    &[ "-p", "--max-sessions", "0", "x" ],
    &path_val,
  );

  assert!(
    out.status.success(),
    "gate bypass with subprocess exit 0 must relay exit 0. Got: {:?}. stderr: {}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
}
