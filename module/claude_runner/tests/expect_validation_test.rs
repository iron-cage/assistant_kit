//! `--expect` / `--expect-strategy` / `--expect-retries` Integration Tests
//!
//! ## Purpose
//!
//! Verify T01–T11 covering the three expect-group parameters:
//! `30_expect.md`, `31_expect_strategy.md`, `32_expect_retries.md`.
//!
//! ## Key Design Note
//!
//! `--expect-retries` default is **0** (zero retries = 1 total attempt). This was corrected
//! from an original design of 2; the implementation uses `unwrap_or(0)` throughout. Tests that
//! exercise the implicit default (no `--expect-retries` flag) expect 1 total attempt.
//!
//! ## Strategy
//!
//! Tests T01–T04, T07–T09 use a fake `claude` shell script injected via PATH
//! manipulation to produce deterministic output without requiring the real binary.
//! Tests T05, T06, T10, T11 use dry-run or parser validation — no subprocess needed.
//!
//! ## Test Layout
//!
//! - T01: Output matches → exit 0
//! - T02: Mismatch + default strategy (fail) → exit 3
//! - T03: Case-insensitive match → exit 0
//! - T04: Leading/trailing whitespace trimmed → exit 0
//! - T05: `--dry-run` with `--expect` → exit 0 (validation skipped)
//! - T06: `clr --help` lists `--expect`, `--expect-strategy`, `--expect-retries`
//! - T07: retry strategy — matches on 2nd attempt → exit 0
//! - T08: retry strategy — all retries exhausted → exit 3
//! - T09: `default:<VAL>` strategy → emits fallback, exit 0
//! - T10: invalid `--expect-strategy` value → exit 1 at parse time
//! - T11: `--expect-retries 256` → exit 1 at parse time (out of range)
//! - T18: `default:` with empty VALUE → accepted at parse time (dry-run exits 0)
#![ cfg( unix ) ]

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ fake_claude, run_cli, run_with_path };
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

// ── T01: Output matches → exit 0 ─────────────────────────────────────────────

/// T01: When the captured output matches an expected value, exit 0.
///
/// Also implicitly verifies that a trailing newline from `echo` is trimmed before
/// comparison — `echo 'yes'` emits `"yes\n"` which trims to `"yes"`.
#[ test ]
fn t01_expect_match_exits_0()
{
  let ( _tmp, path ) = fake_claude( "#!/bin/sh\necho 'yes'" );
  let out = run_with_path( &[ "-p", "--expect", "yes|no", "answer" ], &path );
  assert_eq!(
    out.status.code(),
    Some( 0 ),
    "match must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── T02: Mismatch + default strategy (fail) → exit 3 ─────────────────────────

/// T02: When the output does not match and no strategy is given (default = fail), exit 3.
///
/// Exit code 3 is exclusive to `--expect` mismatch — no other code path uses it.
#[ test ]
fn t02_expect_mismatch_default_fail_exits_3()
{
  let ( _tmp, path ) = fake_claude( "#!/bin/sh\necho 'maybe'" );
  let out = run_with_path( &[ "-p", "--expect", "yes|no", "answer" ], &path );
  assert_eq!(
    out.status.code(),
    Some( 3 ),
    "mismatch with default strategy must exit 3. stdout: {}",
    String::from_utf8_lossy( &out.stdout )
  );
}

// ── T03: Case-insensitive match → exit 0 ─────────────────────────────────────

/// T03: Matching is case-insensitive — `"YES"` matches the expected value `"yes"`.
#[ test ]
fn t03_expect_case_insensitive_match()
{
  let ( _tmp, path ) = fake_claude( "#!/bin/sh\necho 'YES'" );
  let out = run_with_path( &[ "-p", "--expect", "yes|no", "answer" ], &path );
  assert_eq!(
    out.status.code(),
    Some( 0 ),
    "case-insensitive match must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── T04: Whitespace trimmed → exit 0 ─────────────────────────────────────────

/// T04: Leading and trailing whitespace is trimmed before comparison.
///
/// `printf '  yes  '` emits `"  yes  "` which trims to `"yes"` — matching `"yes|no"`.
#[ test ]
fn t04_expect_whitespace_trimmed()
{
  let ( _tmp, path ) = fake_claude( "#!/bin/sh\nprintf '  yes  '" );
  let out = run_with_path( &[ "-p", "--expect", "yes|no", "answer" ], &path );
  assert_eq!(
    out.status.code(),
    Some( 0 ),
    "whitespace trim must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── T05: dry-run with --expect → exit 0 ──────────────────────────────────────

/// T05: `--expect` is silently accepted in dry-run — no subprocess, no validation.
///
/// Verifies the flag is parsed successfully without causing an error.
#[ test ]
fn t05_expect_dry_run_exits_0()
{
  let out = run_cli( &[ "--dry-run", "--expect", "yes|no", "answer" ] );
  assert!(
    out.status.success(),
    "dry-run with --expect must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── T06: --help lists all three expect params ─────────────────────────────────

/// T06: `clr --help` lists `--expect`, `--expect-strategy`, and `--expect-retries`.
#[ test ]
fn t06_help_lists_all_expect_params()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--expect" ),
    "--help must list --expect. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "--expect-strategy" ),
    "--help must list --expect-strategy. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "--expect-retries" ),
    "--help must list --expect-retries. Got:\n{stdout}"
  );
}

// ── T07: retry — matches on 2nd attempt → exit 0 ─────────────────────────────

/// T07: With `--expect-strategy retry --expect-retries 1`, a mismatch on the 1st call
/// followed by a match on the 2nd call exits 0.
///
/// Uses a counter file inside the temp dir: first invocation returns `"maybe"`,
/// second returns `"yes"`.
#[ test ]
fn t07_retry_matches_on_second_attempt()
{
  let tmp = tempfile::tempdir().expect( "create temp dir" );
  let count_path = tmp.path().join( "count.txt" );
  let fake = tmp.path().join( "claude" );
  let script = format!(
    "#!/bin/sh\nCF={}\nN=0\n[ -f \"$CF\" ] && N=$(cat \"$CF\")\nN=$((N+1))\necho $N > \"$CF\"\n[ \"$N\" -eq 1 ] && echo 'maybe' || echo 'yes'\n",
    count_path.display()
  );
  std::fs::write( &fake, &script ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );
  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );

  let out = run_with_path(
    &[ "-p", "--expect", "yes|no", "--expect-strategy", "retry", "--expect-retries", "1", "answer" ],
    &new_path,
  );
  assert_eq!(
    out.status.code(),
    Some( 0 ),
    "retry must succeed on 2nd attempt. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── T08: retry — all retries exhausted → exit 3 ──────────────────────────────

/// T08: When all retry attempts fail to match, exit 3.
///
/// With `--expect-retries 2` and a fake that always returns `"maybe"`, 3 total
/// attempts are made (1 initial + 2 retries) and all fail → exit 3.
#[ test ]
fn t08_retry_exhausted_exits_3()
{
  let ( _tmp, path ) = fake_claude( "#!/bin/sh\necho 'maybe'" );
  let out = run_with_path(
    &[ "-p", "--expect", "yes|no", "--expect-strategy", "retry", "--expect-retries", "2", "answer" ],
    &path,
  );
  assert_eq!(
    out.status.code(),
    Some( 3 ),
    "exhausted retries must exit 3. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── T09: default:<VAL> strategy → emits fallback, exit 0 ─────────────────────

/// T09: With `--expect-strategy default:no`, a mismatch emits the fallback `"no"` and exits 0.
#[ test ]
fn t09_default_strategy_outputs_fallback_exits_0()
{
  let ( _tmp, path ) = fake_claude( "#!/bin/sh\necho 'maybe'" );
  let out = run_with_path(
    &[ "-p", "--expect", "yes|no", "--expect-strategy", "default:no", "answer" ],
    &path,
  );
  assert_eq!(
    out.status.code(),
    Some( 0 ),
    "default strategy must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert_eq!(
    stdout.trim(),
    "no",
    "stdout must be the fallback 'no'. Got:\n{stdout}"
  );
}

// ── T10: invalid --expect-strategy → exit 1 ──────────────────────────────────

/// T10: An unrecognised `--expect-strategy` value is rejected at parse time with exit 1.
#[ test ]
fn t10_invalid_strategy_exits_1()
{
  let out = run_cli( &[ "--expect", "yes|no", "--expect-strategy", "bogus", "answer" ] );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "invalid strategy must exit 1. Got: {:?}",
    out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "strategy" ) || stderr.contains( "Error" ),
    "stderr must contain error message. Got:\n{stderr}"
  );
}

// ── T11: --expect-retries 256 → exit 1 ───────────────────────────────────────

/// T11: `--expect-retries` values outside 0–255 are rejected at parse time with exit 1.
///
/// Covers 32-EC-3.
#[ test ]
fn t11_out_of_range_retries_exits_1()
{
  let out = run_cli( &[
    "--expect",          "yes|no",
    "--expect-strategy", "retry",
    "--expect-retries",  "256",
    "answer",
  ] );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "retries > 255 must exit 1. Got: {:?}",
    out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!( !stderr.is_empty(), "stderr must contain error message. Got empty stderr" );
}

// ── T12: --expect-strategy without --expect → silently ignored (31-EC-6) ─────

/// T12: `--expect-strategy fail` set without `--expect` is silently ignored.
///
/// Covers 31-EC-6: strategy has no effect when no expect value is set.
#[ test ]
fn t12_strategy_without_expect_silently_ignored()
{
  let out = run_cli( &[ "--dry-run", "--expect-strategy", "fail", "task" ] );
  assert!(
    out.status.success(),
    "--expect-strategy without --expect must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── T13: --expect-retries 0 → single attempt (32-EC-2) ───────────────────────

/// T13: `--expect-retries 0` with `--expect-strategy retry` means exactly 1 invocation.
///
/// Covers 32-EC-2: retries=0 means no retries — subprocess called once, then exit 3.
#[ test ]
fn t13_retries_0_means_single_attempt()
{
  let tmp = tempfile::tempdir().expect( "create temp dir" );
  let count_path = tmp.path().join( "count.txt" );
  let fake = tmp.path().join( "claude" );
  let script = format!(
    "#!/bin/sh\nCF={}\nN=0\n[ -f \"$CF\" ] && N=$(cat \"$CF\")\nN=$((N+1))\necho $N > \"$CF\"\necho 'maybe'\n",
    count_path.display()
  );
  std::fs::write( &fake, &script ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );
  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );

  let out = run_with_path(
    &[ "-p", "--expect", "yes|no", "--expect-strategy", "retry", "--expect-retries", "0", "answer" ],
    &new_path,
  );
  assert_eq!(
    out.status.code(),
    Some( 3 ),
    "retries=0 mismatch must exit 3. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let count = std::fs::read_to_string( &count_path )
    .expect( "count file must exist — subprocess must have run" )
    .trim()
    .parse::<u32>()
    .expect( "count is a number" );
  assert_eq!( count, 1, "must invoke exactly 1 time (0 retries). Got: {count}" );
}

// ── T14: CLR_EXPECT_RETRIES env var applied (32-EC-4) ─────────────────────────

/// T14: `CLR_EXPECT_RETRIES=3` applies when the CLI flag is absent.
///
/// Covers 32-EC-4: env var equivalent to `--expect-retries 3`; exit 3 after 4 attempts.
#[ test ]
fn t14_clr_expect_retries_env_var_applied()
{
  let tmp = tempfile::tempdir().expect( "create temp dir" );
  let count_path = tmp.path().join( "count.txt" );
  let fake = tmp.path().join( "claude" );
  let script = format!(
    "#!/bin/sh\nCF={}\nN=0\n[ -f \"$CF\" ] && N=$(cat \"$CF\")\nN=$((N+1))\necho $N > \"$CF\"\necho 'maybe'\n",
    count_path.display()
  );
  std::fs::write( &fake, &script ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );
  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--expect", "yes|no", "--expect-strategy", "retry", "answer" ] )
    .env( "PATH", &new_path )
    .env( "CLR_EXPECT_RETRIES", "3" )
    .output()
    .expect( "invoke clr" );

  assert_eq!(
    out.status.code(),
    Some( 3 ),
    "CLR_EXPECT_RETRIES=3 all-fail must exit 3. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let count = std::fs::read_to_string( &count_path )
    .expect( "count file must exist" )
    .trim()
    .parse::<u32>()
    .expect( "count is a number" );
  assert_eq!( count, 4, "must invoke exactly 4 times (1 initial + 3 env-var retries). Got: {count}" );
}

// ── T15: --expect-retries without retry strategy → silently ignored (32-EC-5) ─

/// T15: `--expect-retries` with `--expect-strategy fail` is silently ignored.
///
/// Covers 32-EC-5: retry count has no effect when strategy is `fail`.
#[ test ]
fn t15_retries_without_retry_strategy_ignored()
{
  let out = run_cli( &[
    "--dry-run",
    "--expect",          "yes|no",
    "--expect-strategy", "fail",
    "--expect-retries",  "5",
    "task",
  ] );
  assert!(
    out.status.success(),
    "--expect-retries with fail strategy in dry-run must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── T17: no --expect-retries flag → default 0 retries → 1 attempt (32-EC-6) ──

/// T17: When `--expect-retries` is absent, the default of 0 retries is used.
///
/// With retry strategy but no explicit retries flag, exactly 1 invocation is made.
/// Covers 32-EC-6: implicit default is 0, not "unlimited".
#[ test ]
fn t17_no_retries_flag_default_zero_means_single_attempt()
{
  let tmp = tempfile::tempdir().expect( "create temp dir" );
  let count_path = tmp.path().join( "count.txt" );
  let fake = tmp.path().join( "claude" );
  let script = format!(
    "#!/bin/sh\nCF={}\nN=0\n[ -f \"$CF\" ] && N=$(cat \"$CF\")\nN=$((N+1))\necho $N > \"$CF\"\necho 'maybe'\n",
    count_path.display()
  );
  std::fs::write( &fake, &script ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );
  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );

  // No --expect-retries: default is 0 (unwrap_or(0) in run_print_mode)
  let out = run_with_path(
    &[ "-p", "--expect", "yes|no", "--expect-strategy", "retry", "answer" ],
    &new_path,
  );
  assert_eq!(
    out.status.code(),
    Some( 3 ),
    "default 0 retries must exit 3 on mismatch. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let count = std::fs::read_to_string( &count_path )
    .expect( "count file must exist — subprocess must have run" )
    .trim()
    .parse::< u32 >()
    .expect( "count is a number" );
  assert_eq!( count, 1, "must invoke exactly 1 time (default 0 retries). Got: {count}" );
}

// ── T16: --expect-retries 3 → exactly 4 invocations (32-EC-1) ─────────────────

/// T16: `--expect-retries 3` with an always-failing fake makes exactly 4 subprocess invocations.
///
/// Covers 32-EC-1: exit 3 after 1 initial + 3 retries = 4 total invocations.
#[ test ]
fn t16_retries_3_makes_4_total_attempts()
{
  let tmp = tempfile::tempdir().expect( "create temp dir" );
  let count_path = tmp.path().join( "count.txt" );
  let fake = tmp.path().join( "claude" );
  let script = format!(
    "#!/bin/sh\nCF={}\nN=0\n[ -f \"$CF\" ] && N=$(cat \"$CF\")\nN=$((N+1))\necho $N > \"$CF\"\necho 'maybe'\n",
    count_path.display()
  );
  std::fs::write( &fake, &script ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );
  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );

  let out = run_with_path(
    &[ "-p", "--expect", "yes|no", "--expect-strategy", "retry", "--expect-retries", "3", "answer" ],
    &new_path,
  );
  assert_eq!(
    out.status.code(),
    Some( 3 ),
    "exhausted 3 retries must exit 3. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let count = std::fs::read_to_string( &count_path )
    .expect( "count file must exist" )
    .trim()
    .parse::<u32>()
    .expect( "count is a number" );
  assert_eq!( count, 4, "must invoke exactly 4 times (1 initial + 3 retries). Got: {count}" );
}

// ── T18: default: with empty VALUE → accepted at parse time (31-EC-7) ─────────

/// T18: `--expect-strategy "default:"` (empty VALUE after colon) is a valid parse.
///
/// The spec states the fallback is emitted "as-is" — an empty string is a legal
/// fallback value (signals "no output on mismatch" while still exiting 0).
/// Covers 31-EC-7.
#[ test ]
fn t18_default_strategy_empty_value_accepted()
{
  let out = run_cli( &[
    "--dry-run",
    "--expect",          "yes",
    "--expect-strategy", "default:",
    "test",
  ] );
  assert!(
    out.status.success(),
    "default: with empty VALUE must exit 0 at parse time. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}
