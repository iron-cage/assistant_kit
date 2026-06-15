//! `--expect` / `--expect-strategy` Integration Tests
//!
//! ## Purpose
//!
//! Verify T01–T18 covering the expect-group parameters:
//! `30_expect.md` and `31_expect_strategy.md`.
//!
//! ## Key Design Note
//!
//! Retry behavior for expect mismatches is now controlled by `--retry-on-validation`
//! (param 048) via the 3-tier retry system.  `--expect-strategy retry` must still
//! be set to enter the retry branch; `--retry-on-validation` supplies the class-specific
//! count.  Full retry edge-case coverage is in `retry_validation_test.rs`.
//!
//! ## Strategy
//!
//! Tests T01–T04, T07–T09 use a fake `claude` shell script injected via PATH
//! manipulation to produce deterministic output without requiring the real binary.
//! Tests T05, T06, T10, T12 use dry-run or parser validation — no subprocess needed.
//!
//! ## Test Layout
//!
//! - T01: Output matches → exit 0
//! - T02: Mismatch + default strategy (fail) → exit 3
//! - T03: Case-insensitive match → exit 0
//! - T04: Leading/trailing whitespace trimmed → exit 0
//! - T05: `--dry-run` with `--expect` → exit 0 (validation skipped)
//! - T06: `clr --help` lists `--expect`, `--expect-strategy`, `--retry-on-validation`
//! - T07: retry strategy — matches on 2nd attempt → exit 0
//! - T08: retry strategy — all retries exhausted → exit 3
//! - T09: `default:<VAL>` strategy → emits fallback, exit 0
//! - T10: invalid `--expect-strategy` value → exit 1 at parse time
//! - T12: `--expect-strategy` without `--expect` → silently ignored
//! - T13: `--retry-on-validation 0 --expect-strategy retry` → single attempt
//! - T15: `--retry-on-validation` with fail strategy → silently ignored
//! - T16: `--retry-on-validation 3` → exactly 4 invocations
//! - T18: `default:` with empty VALUE → accepted at parse time (dry-run exits 0)
#![ cfg( unix ) ]

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ fake_claude, run_cli, run_with_path };
use std::os::unix::fs::PermissionsExt;

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

/// T06: `clr --help` lists `--expect`, `--expect-strategy`, and `--retry-on-validation`;
/// old flag `--expect-retries` is absent.
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
    stdout.contains( "--retry-on-validation" ),
    "--help must list --retry-on-validation. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "--expect-retries" ),
    "--help must NOT list --expect-retries. Got:\n{stdout}"
  );
}

// ── T07: retry — matches on 2nd attempt → exit 0 ─────────────────────────────

/// T07: With `--expect-strategy retry --retry-on-validation 1`, a mismatch on the 1st call
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
    &[ "-p", "--expect", "yes|no", "--expect-strategy", "retry", "--retry-on-validation", "1", "--validation-delay", "0", "answer" ],
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
/// With `--retry-on-validation 2` and a fake that always returns `"maybe"`, 3 total
/// attempts are made (1 initial + 2 retries) and all fail → exit 3.
#[ test ]
fn t08_retry_exhausted_exits_3()
{
  let ( _tmp, path ) = fake_claude( "#!/bin/sh\necho 'maybe'" );
  let out = run_with_path(
    &[ "-p", "--expect", "yes|no", "--expect-strategy", "retry", "--retry-on-validation", "2", "--validation-delay", "0", "answer" ],
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

// ── T13: --retry-on-validation 0 → single attempt ────────────────────────────

/// T13: `--retry-on-validation 0` with `--expect-strategy retry` means exactly 1 invocation.
///
/// `resolve_count(None, 0, None) = 0` retries — subprocess called once, then exit 3.
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
    &[ "-p", "--expect", "yes|no", "--expect-strategy", "retry", "--retry-on-validation", "0", "answer" ],
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

// ── T15: --retry-on-validation without retry strategy → silently ignored ──────

/// T15: `--retry-on-validation` with `--expect-strategy fail` is silently ignored.
///
/// Retry count has no effect when strategy is `fail` — the fail branch
/// exits 3 immediately without consulting the retry count.
#[ test ]
fn t15_retries_without_retry_strategy_ignored()
{
  let out = run_cli( &[
    "--dry-run",
    "--expect",              "yes|no",
    "--expect-strategy",     "fail",
    "--retry-on-validation", "5",
    "task",
  ] );
  assert!(
    out.status.success(),
    "--retry-on-validation with fail strategy in dry-run must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── T16: --retry-on-validation 3 → exactly 4 invocations ─────────────────────

/// T16: `--retry-on-validation 3` with an always-failing fake makes exactly 4 invocations.
///
/// Exit 3 after 1 initial + 3 retries = 4 total invocations.
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
    &[ "-p", "--expect", "yes|no", "--expect-strategy", "retry", "--retry-on-validation", "3", "--validation-delay", "0", "answer" ],
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
