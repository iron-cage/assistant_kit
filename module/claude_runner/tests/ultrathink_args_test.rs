//! Ultrathink Argument Tests — message suffix injection
//!
//! ## Purpose
//!
//! Verify that `claude_runner` appends `"\n\nultrathink"` to every message by default
//! and that `--no-ultrathink` correctly suppresses that injection. Uses `--dry-run` to
//! inspect command construction without requiring the Claude binary in PATH.
//!
//! ## Strategy
//!
//! All tests invoke the compiled binary via `env!("CARGO_BIN_EXE_clr")`.
//! `--dry-run` outputs the command line that would be executed, allowing
//! assertions against the translation of flags → builder calls.
//!
//! ## Corner Cases Covered
//!
//! - T50: message is suffixed with `"\n\nultrathink"` by default in dry-run output
//! - T51: `--no-ultrathink` suppresses the default `"\n\nultrathink"` suffix
//! - T52: idempotent guard — message already ending with `"ultrathink"` not double-suffixed
//! - T53: `--no-ultrathink` listed in `--help` output
//! - T54: empty string positional arg `""` is silently skipped (no message, no degenerate suffix)
//! - T55: `--help` wins over subsequent unknown flags in argv (pre-scan)
//! - T56: `--help` wins over preceding unknown flags in argv (pre-scan)
//! - T57: empty string positional arg after `--` separator is silently skipped
//! - T58: message is suffixed (not prefixed) with `"\n\nultrathink"` — suffix position guard

mod common;
use common::run_cli;

// T50: message is suffixed with "\n\nultrathink" by default
//
// Default-on behavior: every message passed to clr is appended with "\n\nultrathink"
// before being forwarded to claude. This activates extended thinking mode for all
// automation without requiring the user to write "ultrathink" in every prompt.
#[ test ]
fn t50_default_message_gets_ultrathink_suffix()
{
  let out = run_cli( &[ "--dry-run", "hello" ] );
  assert!(
    out.status.success(),
    "dry-run with message must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "\"hello\n\nultrathink\"" ),
    "message must be suffixed with \"\\n\\nultrathink\". Got:\n{stdout}"
  );
}

// T51: --no-ultrathink suppresses the default "\n\nultrathink" suffix
//
// Opt-out: when --no-ultrathink is given, the message is forwarded verbatim
// without appending "\n\nultrathink". Allows callers to manage their own prompts.
#[ test ]
fn t51_no_ultrathink_suppresses_suffix()
{
  let out = run_cli( &[ "--dry-run", "--no-ultrathink", "hello" ] );
  assert!(
    out.status.success(),
    "--no-ultrathink must be a known flag (exit 0). stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "\"hello\"" ),
    "message must appear verbatim with --no-ultrathink. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "ultrathink" ),
    "suffix must be suppressed with --no-ultrathink. Got:\n{stdout}"
  );
}

// T52: idempotent guard — message already ending with "ultrathink" is not double-suffixed
//
// If the user's message already ends with "ultrathink", the suffix injection is skipped.
// Guard uses trim_end().ends_with("ultrathink") to also catch trailing-whitespace variants.
// This prevents accumulation in scripts that call clr with pre-suffixed prompts.
#[ test ]
fn t52_idempotent_guard_no_double_suffix()
{
  let out = run_cli( &[ "--dry-run", "fix the bug ultrathink" ] );
  assert!(
    out.status.success(),
    "ultrathink-suffixed message must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "\"fix the bug ultrathink\"" ),
    "message must appear verbatim (guard fires, no re-suffix). Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "ultrathink\n\nultrathink" ),
    "double-suffix must not appear. Got:\n{stdout}"
  );
}

// T53: --no-ultrathink listed in --help output
//
// Documentation hygiene: every user-facing flag must be discoverable via --help.
// Regression guard: if the help line is accidentally removed, this test catches it.
#[ test ]
fn t53_help_lists_no_ultrathink()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "--help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--no-ultrathink" ),
    "--help must list --no-ultrathink flag. Got:\n{stdout}"
  );
}

// T54: empty positional arg `""` is ignored — treated as no message
//
// ## Root Cause (bug_reproducer(issue-empty-msg-ultrathink))
//
// An empty string passed as a positional arg (`clr ""`) was pushed to the positional
// list, joined into `message = Some("")`, then the ultrathink prefix produced
// `"ultrathink "` (trailing space). This also triggered print mode (--print added)
// for what was effectively "no message".
//
// ## Why Not Caught
//
// All tests used non-empty messages. Empty-string positional was never exercised.
//
// ## Fix Applied
//
// Skip empty tokens in the positional-arg collection path of `parse_args`.
// Empty positional args now have no effect — `clr ""` behaves identically to bare `clr`.
//
// ## Prevention
//
// Scripts that pass a variable as a positional arg may pass `""` when the variable is
// empty. The fix ensures this degeneracy is silently handled rather than forwarded to claude.
//
// ## Pitfall
//
// Do not use `positional.join(" ").trim().is_empty()` to filter after joining — this
// would also filter whitespace-only strings which are valid non-empty messages (e.g. " ").
// The correct fix skips only empty tokens at the individual-token level.
// test_kind: bug_reproducer(issue-empty-msg-ultrathink)
#[ test ]
fn t54_empty_positional_arg_ignored()
{
  let out = run_cli( &[ "--dry-run", "" ] );
  assert!(
    out.status.success(),
    "empty positional arg must not error. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  let last_line = stdout.trim_end().lines().last().unwrap_or_default();
  assert_eq!(
    last_line,
    "claude --dangerously-skip-permissions --chrome --effort max -c",
    "empty positional arg must produce bare command (no --print, no message). Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "\"ultrathink \"" ),
    "empty positional arg must NOT produce degenerate 'ultrathink ' message. Got:\n{stdout}"
  );
}

// T55: `--help` wins over subsequent unknown flags
//
// ## Root Cause (bug_reproducer(issue-help-loses-to-unknown))
//
// `parse_args` processes tokens left-to-right and returns Err immediately on the first
// unknown flag. When `--help` precedes an unknown flag, `parsed.help` is set to true, but
// the subsequent unknown flag triggers early return with Err. `main()` then exits 1 with
// an error message instead of calling `print_help()`.
//
// ## Why Not Caught
//
// T26 tests `--dir /tmp --help` (valid flags before --help) and T14 tests `--help` alone.
// No test exercised --help combined with an UNKNOWN flag.
//
// ## Fix Applied
//
// Pre-scan tokens for `--help`/`-h` at the start of `parse_args`. If found, return
// `CliArgs { help: true, .. }` immediately without attempting full parsing. This ensures
// --help always wins regardless of what other flags (valid or invalid) appear in argv.
//
// ## Prevention
//
// Test --help in combination with invalid flags (both before and after --help position).
//
// ## Pitfall
//
// Don't use `cli.help` to gate the pre-scan — the pre-scan IS what sets cli.help for
// the error-recovery path. Without the pre-scan, the error path in main() runs first.
// test_kind: bug_reproducer(issue-help-loses-to-unknown)
#[ test ]
fn t55_help_wins_over_subsequent_unknown_flag()
{
  let out = run_cli( &[ "--help", "--not-a-real-flag" ] );
  assert!(
    out.status.success(),
    "--help before unknown flag must exit 0 (help wins). stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "USAGE:" ),
    "--help before unknown flag must show USAGE. Got:\n{stdout}"
  );
}

// T56: `--help` wins over preceding unknown flags (part 2 of issue-help-loses-to-unknown)
//
// Companion to T55: when the unknown flag appears BEFORE --help, the early-return Err
// triggered by the unknown flag also prevents --help from ever being processed.
// The fix (pre-scan in parse_args) handles both orderings.
// test_kind: bug_reproducer(issue-help-loses-to-unknown)
#[ test ]
fn t56_help_wins_over_preceding_unknown_flag()
{
  let out = run_cli( &[ "--not-a-real-flag", "--help" ] );
  assert!(
    out.status.success(),
    "--help after unknown flag must exit 0 (help wins). stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "USAGE:" ),
    "--help after unknown flag must show USAGE. Got:\n{stdout}"
  );
}

// T57: empty string positional arg after `--` separator is silently skipped
//
// ## Root Cause (bug_reproducer(issue-empty-msg-double-dash))
//
// The `--` arm in `parse_args` uses `positional.extend(tokens[i+1..])` which copies
// all remaining tokens verbatim, including empty strings. The `_` arm (which handles
// bare positional tokens) filters empty tokens via `!tokens[i].is_empty()`, but that
// filter is bypassed entirely by the `--` code path.
//
// ## Why Not Caught
//
// T38 tests `-- ` (no args after `--`) and T54 tests bare `""` (without `--`).
// No test exercised the combination `-- ""`.
//
// ## Fix Applied
//
// Filter empty tokens in the `--` arm before extending positional, matching the
// filter already applied in the `_` arm.
//
// ## Prevention
//
// Test the `--` separator with an empty string argument in addition to testing
// bare empty strings.
//
// ## Pitfall
//
// The `--` arm must filter at the individual-token level, not on the joined string,
// for the same reason as the `_` arm: whitespace-only strings like `" "` are valid
// messages and must pass through.
// test_kind: bug_reproducer(issue-empty-msg-double-dash)
#[ test ]
fn t57_empty_positional_after_double_dash_ignored()
{
  let out = run_cli( &[ "--dry-run", "--", "" ] );
  assert!(
    out.status.success(),
    "empty arg after -- must not error. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  let last_line = stdout.trim_end().lines().last().unwrap_or_default();
  assert_eq!(
    last_line,
    "claude --dangerously-skip-permissions --chrome --effort max -c",
    "empty arg after -- must produce bare command (no --print, no message). Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "\"ultrathink \"" ),
    "empty arg after -- must NOT produce degenerate 'ultrathink ' message. Got:\n{stdout}"
  );
}

// T58: ultrathink is appended as suffix ("\n\nultrathink") not prepended as prefix
//
// ## Root Cause (bug_reproducer(issue-ultrathink-suffix))
//
// TSK-090 implemented ultrathink injection as `format!("ultrathink {msg}")` (prefix),
// but the correct behavior is `format!("{msg}\n\nultrathink")` (suffix after two
// newlines). Live feedback (`-feedback.md`) showed `"ultrathink hi"` when `"hi\n\nultrathink"`
// was expected.
//
// ## Why Not Caught
//
// Existing tests only asserted that "ultrathink" was present (containment check),
// never that it was at the END of the message. `String::contains("ultrathink")` is
// position-blind — it returns true for both prefix and suffix forms.
//
// ## Fix Applied
//
// Changed `format!("ultrathink {msg}")` → `format!("{msg}\n\nultrathink")` and
// the idempotent guard from `msg.starts_with("ultrathink")` → `msg.trim_end().ends_with("ultrathink")`.
//
// ## Prevention
//
// Assert the EXACT expected string including position (`contains("\"hello\n\nultrathink\"")`),
// not just containment (`contains("ultrathink")`). Injection-position bugs are invisible
// to containment-only assertions.
//
// ## Pitfall
//
// `String::contains("ultrathink")` passes for both `"ultrathink hello"` (prefix) and
// `"hello\n\nultrathink"` (suffix). Always test the exact injection form.
#[ test ]
fn t58_default_message_gets_ultrathink_suffix()
{
  let out = run_cli( &[ "--dry-run", "hello" ] );
  assert!(
    out.status.success(),
    "dry-run with message must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "\"hello\n\nultrathink\"" ),
    "message must be suffixed with \"\\n\\nultrathink\". Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "\"ultrathink hello\"" ),
    "prefix form must be absent after fix. Got:\n{stdout}"
  );
}
