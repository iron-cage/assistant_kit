//! Integration tests for CLR error classification (BUG-037).
//!
//! Verifies that `run_print_mode` emits labeled per-type diagnostics on stderr
//! when `classify_error()` identifies a specific failure mode. Uses fake-claude
//! shell scripts to control subprocess exit code and output — no real claude binary.
//!
//! # Test Matrix
//!
//! | Test | Scenario | Expected stderr |
//! |------|----------|-----------------|
//! | T09 | fake-claude exits 2, empty output | `"Error: [Transient] rate limit (exit 2)"` |
//! | T10 | fake-claude writes auth pattern to stdout, exits 1 | `"Error: [Auth]"` prefix |
//! | T11 | fake-claude writes quota pattern to stderr, exits 1 | `"Error: [Account]"` prefix |
//! | T11a | fake-claude writes the real captured session-limit form to stderr, exits 1 | `"Error: [Account]"` prefix, no `[Unknown]` |
//! | T11b | fake-claude writes the real captured weekly-limit form to stdout, exits 1 | `"Error: [Account]"` prefix, no `[Unknown]` |
//! | T12 | fake-claude exits 0 with a zero-turn envelope whose `result` embeds a quota pattern (BUG-590) | `"Error: [Account]"` prefix, exit 1 |
//! | T13 | fake-claude exits 0 with a zero-turn envelope whose `result` embeds an auth pattern (BUG-590) | `"Error: [Auth]"` prefix, exit 1 |
//! | T14 | fake-claude exits 0 with a zero-turn envelope whose `result` has no failure pattern (BUG-590 guard) | exit 0, no `[Class]` label |
//! | T15 | fake-claude exits 0 with a 3-turn envelope whose `result` embeds a failure-looking pattern (BUG-590 guard) | exit 0, no `[Class]` label |
//!
//! # Root Cause (BUG-037)
//!
//! `run_print_mode` emitted `"Claude exited without output (possible rate limit or quota
//! exhaustion)"` for ALL silent non-zero exits. Callers and monitoring tools could not
//! distinguish rate-limit from auth failure from API error.
//!
//! # Why Not Caught
//!
//! No integration test asserted the stderr message format for specific exit codes or
//! output patterns. The generic message was accepted as "good enough" at review time.
//!
//! # Fix Applied
//!
//! BUG-037 block replaced with a match on `output.classify_error()`. Each `ErrorKind`
//! variant emits `"Error: {label} (exit {code})"`, providing distinct signals per type.
//!
//! # Prevention
//!
//! For each `ErrorKind` variant, add an integration test that drives the CLR binary with
//! a fake-claude script and asserts the expected labeled string on stderr.
//!
//! # Pitfall
//!
//! `classify_error()` scans both stderr AND stdout. When claude writes the failure reason
//! to stdout (e.g. auth errors via `--print` JSON output), the stderr scan alone would
//! miss it. Always drive a test that puts the pattern in stdout, not only stderr.

#![ cfg( unix ) ]

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ exit_code, fake_claude_dir, run_cli_with_env, stderr_str, stdout_str };

// ── T09 ───────────────────────────────────────────────────────────────────────

/// T09 (BUG-037): fake-claude exits 2 with no output → clr stderr contains
/// `"Error: [Transient] rate limit (exit 2)"`.
///
/// Before fix: stderr contained the generic phrase "possible rate limit or quota exhaustion".
/// After fix (3-tier redesign): stderr contains the `[Class]`-prefixed label.
/// `--retry-override 0` disables all retries so the label fires immediately rather than
/// after sleeping 30s and emitting "retries exhausted".
#[ test ]
#[ doc = "bug_reproducer(BUG-037)" ]
fn rate_limit_exit2_emits_labeled_message()
{
  let ( _dir, path_val ) = fake_claude_dir( "exit 2" );
  // --retry-override 0 disables all error-class retries (overrides the built-in default=2).
  let out = run_cli_with_env(
    &[ "--print", "--retry-override", "0", "--max-sessions", "0", "test" ],
    &[ ( "PATH", &path_val ) ],
  );
  let err = stderr_str( &out );
  assert!(
    err.contains( "Error: [Transient] rate limit (exit 2)" ),
    "T09 (BUG-037): stderr must contain 'Error: [Transient] rate limit (exit 2)'; got:\n{err}"
  );
  assert!(
    !err.contains( "possible rate limit or quota exhaustion" ),
    "T09 (BUG-037): generic phrase must be absent; got:\n{err}"
  );
}

// ── T10 ───────────────────────────────────────────────────────────────────────

/// T10 (BUG-037): fake-claude writes auth pattern to stdout, exits 1 → clr stderr
/// contains `"Error: [Auth]"` prefix with the original message.
///
/// Validates that `classify_error()` scans stdout as well as stderr — auth failure
/// text from `claude --print` arrives via stdout, not stderr.
/// `--retry-override 0` disables Auth-class retry so the label fires immediately.
#[ test ]
#[ doc = "bug_reproducer(BUG-037)" ]
fn auth_error_pattern_in_stdout_emits_labeled_message()
{
  let ( _dir, path_val ) = fake_claude_dir(
    "echo 'Your organization does not have access to Claude'; exit 1",
  );
  let out = run_cli_with_env(
    &[ "--print", "--retry-override", "0", "--max-sessions", "0", "test" ],
    &[ ( "PATH", &path_val ) ],
  );
  let err = stderr_str( &out );
  assert!(
    err.contains( "Error: [Auth]" ),
    "T10 (BUG-037): stderr must contain 'Error: [Auth]' prefix; got:\n{err}"
  );
  assert!(
    err.contains( "Your organization does not have access to Claude" ),
    "T10 (BUG-037): stderr must contain the original auth message; got:\n{err}"
  );
  assert!(
    !err.contains( "possible rate limit or quota exhaustion" ),
    "T10 (BUG-037): generic phrase must be absent; got:\n{err}"
  );
}

// ── T11 ───────────────────────────────────────────────────────────────────────

/// T11 (TSK-253): fake-claude writes quota exhaustion pattern to stderr, exits 1 →
/// clr stderr contains `"Error: [Account]"` prefix with the original message.
///
/// Verifies that `QuotaExhausted` is distinct from `RateLimit` at the CLR output layer —
/// quota exhaustion maps to `[Account]` class, NOT `[Transient]`.
/// `--retry-override 0` disables Account-class retry so the label fires immediately.
#[ test ]
fn quota_exhausted_pattern_emits_labeled_message()
{
  let ( _dir, path_val ) = fake_claude_dir(
    "echo \"You've hit your limit\" >&2; exit 1",
  );
  let out = run_cli_with_env(
    &[ "--print", "--retry-override", "0", "--max-sessions", "0", "test" ],
    &[ ( "PATH", &path_val ) ],
  );
  let err = stderr_str( &out );
  assert!(
    err.contains( "Error: [Account]" ),
    "T11 (TSK-253): stderr must contain 'Error: [Account]' prefix; got:\n{err}"
  );
  assert!(
    err.contains( "You've hit your limit" ),
    "T11 (TSK-253): stderr must contain the original quota message; got:\n{err}"
  );
  assert!(
    !err.contains( "[Transient]" ),
    "T11 (TSK-253): [Transient] must be absent for quota exhaustion; got:\n{err}"
  );
}

// ── T11a ───────────────────────────────────────────────────────────────────────

/// T11a (BUG-495): fake-claude writes the REAL captured session-limit message form
/// to stderr, exits 1 → clr stderr contains `"Error: [Account]"` and never `[Unknown]`.
///
/// Extends T11 (plain synthetic form) and the core-layer reproducers B495a/B495b in
/// `claude_runner_core/tests/classify_error_test.rs` (real forms → `ErrorKind::QuotaExhausted`)
/// with the end-to-end invariant neither pins: the byte-exact real captured CLI text
/// routing through the spawned `clr` binary to the `[Account]` class label —
/// recommended by `yrd_aes` BUG-1841 after observing the invariant break in production.
///
/// ## Root Cause
/// `ERROR_PATTERNS`' sole `QuotaExhausted` pattern was the plain form `"You've hit your
/// limit"`, which is not a substring of the qualified forms the real CLI emits — 100%
/// of real quota messages classified `[Unknown]`, silently bypassing the Account-class
/// retry configuration (`--retry-on-account`/`--account-delay`) wired by consumers.
///
/// ## Why Not Caught
/// Every CLI-layer test (EC-7/8/9, T11) drove the plain synthetic string the CLI never
/// emits; the core-layer fix added real-form tests only for `classify_error()` in
/// isolation. No test asserted real captured text → `[Account]` through the binary.
///
/// ## Fix Applied
/// BUG-495 inserted the two qualified-form patterns into `ERROR_PATTERNS`
/// (`claude_runner_core/src/types.rs`); T11a/T11b pin that fix at the CLI layer.
///
/// ## Prevention
/// Classifier regression tests must use verbatim captured production strings, not
/// remembered/assumed forms — at every layer that consumes the classification.
///
/// ## Pitfall
/// A pattern fix proven at the pure-function layer can still be unreachable end-to-end
/// (`yrd_aes` BUG-1841: a stale deployed binary misclassified the byte-identical
/// message a fresh binary classified correctly, in the same pipeline run) — the
/// end-to-end pin catches regressions in the wiring, not only the pattern array.
// test_kind: bug_reproducer(BUG-495)
#[ test ]
fn t11a_bug495_session_limit_real_form_routes_to_account()
{
  let ( _dir, path_val ) = fake_claude_dir(
    "echo \"You've hit your session limit · resets 4:20am (Europe/Kyiv)\" >&2; exit 1",
  );
  let out = run_cli_with_env(
    &[ "--print", "--retry-override", "0", "--max-sessions", "0", "test" ],
    &[ ( "PATH", &path_val ) ],
  );
  let err = stderr_str( &out );
  assert!(
    err.contains( "Error: [Account]" ),
    "T11a (BUG-495): stderr must contain 'Error: [Account]' for the real session-limit form; got:\n{err}"
  );
  assert!(
    err.contains( "You've hit your session limit · resets 4:20am (Europe/Kyiv)" ),
    "T11a (BUG-495): stderr must echo the verbatim captured message; got:\n{err}"
  );
  assert!(
    !err.contains( "[Unknown]" ),
    "T11a (BUG-495): [Unknown] must be absent — that is the exact pre-fix misclassification; got:\n{err}"
  );
}

// ── T11b ───────────────────────────────────────────────────────────────────────

/// T11b (BUG-495): fake-claude writes the REAL captured weekly-limit message form
/// to stdout, exits 1 → clr stderr contains `"Error: [Account]"` and never `[Unknown]`.
///
/// Divergence from T11a: weekly form (the exact message `yrd_aes` BUG-1841's run-3641
/// evidence captured being tagged `[Unknown]` by a stale binary and `[Account]` by a
/// fresh one), delivered via stdout — the scan path T10's pitfall documents as the one
/// a stderr-only test would miss.
// test_kind: bug_reproducer(BUG-495)
#[ test ]
fn t11b_bug495_weekly_limit_real_form_routes_to_account()
{
  let ( _dir, path_val ) = fake_claude_dir(
    "echo \"You've hit your weekly limit · resets Aug 11, 11pm (Europe/Kyiv)\"; exit 1",
  );
  let out = run_cli_with_env(
    &[ "--print", "--retry-override", "0", "--max-sessions", "0", "test" ],
    &[ ( "PATH", &path_val ) ],
  );
  let err = stderr_str( &out );
  assert!(
    err.contains( "Error: [Account]" ),
    "T11b (BUG-495): stderr must contain 'Error: [Account]' for the real weekly-limit form; got:\n{err}"
  );
  assert!(
    err.contains( "You've hit your weekly limit · resets Aug 11, 11pm (Europe/Kyiv)" ),
    "T11b (BUG-495): stderr must echo the verbatim captured message; got:\n{err}"
  );
  assert!(
    !err.contains( "[Unknown]" ),
    "T11b (BUG-495): [Unknown] must be absent — that is the exact pre-fix misclassification; got:\n{err}"
  );
}

// ── TC-12 ──────────────────────────────────────────────────────────────────────

/// TC-12 (BUG-298): when `claude` binary exists but is `chmod 000` (no execute
/// permission), `clr --print` must exit 1 with `"[Runner]"` on stderr.
///
/// ## Root Cause
/// `spawn_error_msg()` did not prepend `[Runner]` to either branch; the no-timeout
/// spawn arm in `execute_print_attempt()` bypassed `spawn_error_msg()` entirely and
/// emitted bare `{e}` with no class tag.
///
/// ## Why Not Caught
/// Existing T09/T10/T11 tests drove fake-claude shell scripts (executable); none tested
/// a binary whose permissions deny execution. The EACCES path was never exercised.
///
/// ## Fix Applied
/// `spawn_error_msg()` now prepends `"[Runner]"` to both branches. The no-timeout
/// arm now calls the helper (or prepends `[Runner]` directly via `eprintln!("Error: [Runner] {e}")`).
///
/// ## Prevention
/// For each error class, add an integration test that exercises the CLR binary with
/// a trigger for that class and asserts the `[Class]` prefix on stderr.
///
/// ## Pitfall
/// Do NOT use `fake_claude_binary_dir()` — it sets `chmod 0o755` (executable).
/// TC-12 needs `chmod 000` to trigger EACCES. Copy the binary, then call
/// `fs::set_permissions()` to deny execution.
// test_kind: bug_reproducer(BUG-298)
#[ cfg( unix ) ]
#[ test ]
fn tc_12_runner_spawn_failed_prefix()
{
  use std::os::unix::fs::PermissionsExt;

  let dir      = tempfile::TempDir::new().expect( "create temp dir for chmod 000 test" );
  let claude   = dir.path().join( "claude" );
  std::fs::copy( "/bin/sleep", &claude ).expect( "copy sleep as claude" );
  std::fs::set_permissions( &claude, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "set 755 on claude copy" );
  // Now deny all execution to trigger EACCES on spawn.
  std::fs::set_permissions( &claude, std::fs::Permissions::from_mode( 0o000 ) )
    .expect( "set 000 on claude (deny execute)" );

  let path_val = dir.path().to_str().expect( "dir UTF-8" ).to_string();
  let out      = run_cli_with_env(
    &[ "--print", "--max-sessions", "0", "--retry-override", "0", "msg" ],
    &[ ( "PATH", &path_val ) ],
  );
  let err = stderr_str( &out );

  // Restore permissions so TempDir cleanup can delete the file.
  let _ = std::fs::set_permissions( &claude, std::fs::Permissions::from_mode( 0o644 ) );

  assert!(
    !out.status.success(),
    "TC-12 (BUG-298): expected non-zero exit for chmod 000 binary; got 0"
  );
  assert!(
    err.contains( "[Runner]" ),
    "TC-12 (BUG-298): stderr must contain '[Runner]' prefix; got:\n{err}"
  );
}

// ── T12 ───────────────────────────────────────────────────────────────────────

/// T12 (BUG-590): fake-claude exits 0 with a zero-turn JSON envelope whose `result`
/// text embeds the plain quota-limit pattern → clr must re-classify and exit 1 with
/// `"Error: [Account]"`, never silently report the zero-turn envelope as success.
///
/// ## Root Cause
/// `claude` can report a session-limit/auth/API failure as a zero-turn "success" —
/// `exit 0`, `"is_error":false`, `"num_turns":0` — and `classify_error()` returns `None`
/// unconditionally whenever `exit_code == 0`, so the failure is structurally invisible
/// to it regardless of what the envelope's `result` text says.
///
/// ## Why Not Caught
/// Every existing classifier test (T09–T11b) drove a genuinely non-zero exit code;
/// none exercised an `exit 0` subprocess carrying a failure pattern in its own output.
///
/// ## Fix Applied
/// `run_print_mode()` now probes a copy of the output with `exit_code` forced to `1`
/// whenever the real exit code is `0` and `summary::num_turns()` reads `0` from the
/// envelope. If that probe's `classify_error()` names anything but `Unknown`, the
/// attempt's real exit code is promoted to `1` so the existing retry/exit path runs.
///
/// ## Prevention
/// Any zero-turn `exit 0` envelope carrying a recognizable failure pattern must now
/// route through the same `[Class]` labeling as a genuine non-zero exit.
///
/// ## Pitfall
/// The probe must never fire on `num_turns != 0` or on `exit_code != 0` already — see
/// T15 and the existing T09–T11b coverage respectively; only the exit-0/zero-turn
/// combination is structurally blind to `classify_error()`.
// test_kind: bug_reproducer(BUG-590)
#[ test ]
fn t12_bug590_zero_turn_exit0_quota_pattern_reclassified_to_account()
{
  let ( _dir, path_val ) = fake_claude_dir(
    r#"echo '{"type":"result","subtype":"success","is_error":false,"num_turns":0,"result":"You'\''ve hit your limit"}'; exit 0"#,
  );
  let out = run_cli_with_env(
    &[ "--print", "--retry-override", "0", "--max-sessions", "0", "test" ],
    &[ ( "PATH", &path_val ) ],
  );
  let err = stderr_str( &out );
  assert_eq!(
    exit_code( &out ), 1,
    "T12 (BUG-590): clr must exit 1 for a zero-turn exit-0 quota-limit envelope; stderr:\n{err}"
  );
  assert!(
    err.contains( "Error: [Account]" ),
    "T12 (BUG-590): stderr must contain 'Error: [Account]' prefix; got:\n{err}"
  );
  assert!(
    err.contains( "You've hit your limit" ),
    "T12 (BUG-590): stderr must contain the envelope's own result text; got:\n{err}"
  );
  assert!(
    !err.contains( "[Unknown]" ),
    "T12 (BUG-590): [Unknown] must be absent — that is the exact pre-fix misclassification; got:\n{err}"
  );
}

// ── T13 ───────────────────────────────────────────────────────────────────────

/// T13 (BUG-590): fake-claude exits 0 with a zero-turn JSON envelope whose `result`
/// text embeds the auth-failure pattern → clr must re-classify and exit 1 with
/// `"Error: [Auth]"`. Divergence from T12: `AuthError` class, confirming the probe
/// in `run_print_mode()` is not hardcoded to a single `ErrorKind` variant.
// test_kind: bug_reproducer(BUG-590)
#[ test ]
fn t13_bug590_zero_turn_exit0_auth_pattern_reclassified_to_auth()
{
  let ( _dir, path_val ) = fake_claude_dir(
    r#"echo '{"type":"result","subtype":"success","is_error":false,"num_turns":0,"result":"Your organization does not have access to Claude"}'; exit 0"#,
  );
  let out = run_cli_with_env(
    &[ "--print", "--retry-override", "0", "--max-sessions", "0", "test" ],
    &[ ( "PATH", &path_val ) ],
  );
  let err = stderr_str( &out );
  assert_eq!(
    exit_code( &out ), 1,
    "T13 (BUG-590): clr must exit 1 for a zero-turn exit-0 auth-failure envelope; stderr:\n{err}"
  );
  assert!(
    err.contains( "Error: [Auth]" ),
    "T13 (BUG-590): stderr must contain 'Error: [Auth]' prefix; got:\n{err}"
  );
  assert!(
    err.contains( "Your organization does not have access to Claude" ),
    "T13 (BUG-590): stderr must contain the envelope's own result text; got:\n{err}"
  );
  assert!(
    !err.contains( "[Unknown]" ),
    "T13 (BUG-590): [Unknown] must be absent — that is the exact pre-fix misclassification; got:\n{err}"
  );
}

// ── T14 ───────────────────────────────────────────────────────────────────────

/// T14 (BUG-590 guard): fake-claude exits 0 with a zero-turn envelope whose `result`
/// carries no recognizable failure pattern → clr must stay exit 0 and print the
/// result normally, never treat "zero turns" alone as a failure signal.
///
/// Guards against the blanket "zero-turn-exit-0-as-failure" approach BUG-590's own
/// report explicitly rejected: a genuinely empty-but-successful zero-turn run (e.g.
/// an instantly-satisfied trivial prompt) must not be misreported as a failure.
#[ test ]
fn t14_bug590_zero_turn_exit0_no_pattern_stays_success()
{
  let ( _dir, path_val ) = fake_claude_dir(
    r#"echo '{"type":"result","subtype":"success","is_error":false,"num_turns":0,"result":"done"}'; exit 0"#,
  );
  let out = run_cli_with_env(
    &[ "--print", "--retry-override", "0", "--max-sessions", "0", "test" ],
    &[ ( "PATH", &path_val ) ],
  );
  let err = stderr_str( &out );
  assert_eq!(
    exit_code( &out ), 0,
    "T14 (BUG-590 guard): a pattern-free zero-turn exit-0 envelope must stay exit 0; stderr:\n{err}"
  );
  assert!(
    !err.contains( "Error: [" ),
    "T14 (BUG-590 guard): no [Class] label must fire for a pattern-free zero-turn envelope; got:\n{err}"
  );
  assert!(
    stdout_str( &out ).contains( "done" ),
    "T14 (BUG-590 guard): the result text must still be printed on the success path"
  );
}

// ── T15 ───────────────────────────────────────────────────────────────────────

/// T15 (BUG-590 guard): fake-claude exits 0 with a 3-turn envelope whose `result`
/// embeds a failure-looking pattern → clr must stay exit 0, because the probe in
/// `run_print_mode()` only fires when `num_turns` reads exactly `0`.
///
/// Guards the other half of the same rule: a nonzero-turn run that merely *mentions*
/// failure-pattern text (e.g. quoting it, discussing it) must never be reclassified —
/// the zero-turn signal is what distinguishes a real silent failure from ordinary text.
#[ test ]
fn t15_bug590_nonzero_turn_exit0_error_text_stays_success()
{
  let ( _dir, path_val ) = fake_claude_dir(
    r#"echo '{"type":"result","subtype":"success","is_error":false,"num_turns":3,"result":"mentioned: Your organization does not have access to Claude, but handled it"}'; exit 0"#,
  );
  let out = run_cli_with_env(
    &[ "--print", "--retry-override", "0", "--max-sessions", "0", "test" ],
    &[ ( "PATH", &path_val ) ],
  );
  let err = stderr_str( &out );
  assert_eq!(
    exit_code( &out ), 0,
    "T15 (BUG-590 guard): a 3-turn envelope must stay exit 0 regardless of result text; stderr:\n{err}"
  );
  assert!(
    !err.contains( "Error: [" ),
    "T15 (BUG-590 guard): no [Class] label must fire when num_turns != 0; got:\n{err}"
  );
  assert!(
    stdout_str( &out ).contains( "Your organization does not have access to Claude" ),
    "T15 (BUG-590 guard): the result text must still be printed verbatim on the success path"
  );
}
