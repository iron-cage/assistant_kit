//! Argument parsing tests via the `claude_manager` binary.
//!
//! ## Test Matrix
//!
//! Tests verify dot-prefixed command parsing, `key::value` parameter parsing,
//! value validation, and all rejection paths through the binary.
//!
//! | TC | Description | Kind |
//! |----|-------------|------|
//! | TC-001 | Empty argv → help output, exit 0 | P |
//! | TC-002 | `.help` → help output, exit 0 | P |
//! | TC-004 | Unknown `bogus::1` param → exit 1 | N |
//! | TC-005 | `v::` empty value → exit 1 | N |
//! | TC-006 | `v::3` out of range → exit 1 | N |
//! | TC-007 | `v::abc` non-integer → exit 1 | N |
//! | TC-008 | `v::0` accepted via `.status` → exit 0 | P |
//! | TC-010 | Last `v::` wins when duplicated | P |
//! | TC-011 | Single-word subcommand `.status` parsed | P |
//! | TC-012 | Two-word subcommand `.version.list` parsed | P |
//! | TC-014 | Unknown command `.nonexistent` → exit 1 | N |
//! | TC-015 | `format::` empty value → exit 1 | N |
//! | TC-016 | `version::` empty value → exit 1 | N |
//! | TC-020 | `dry::1` is accepted | P |
//! | TC-021 | `force::1` is accepted | P |
//! | TC-022 | `v::0` produces consistent output | P |
//! | TC-024 | Bare token without `::` after command → exit 1 | N |
//! | TC-025 | Bare token without `.` prefix and without `::` → exit 1 | N |
//! | TC-026 | `.help` subcommand explicitly → help output, exit 0 | P |
//! | TC-027 | `--` double-dash token → exit 1 (not param::value) | N |
//! | TC-028 | `.version.install version::1.2.3.4` → exit 1 | N |
//! | TC-029 | `.version.install version::01.02.03` → exit 1 | P |
//! | TC-030 | `format::TEXT` (wrong case) → exit 1 | N |
//! | TC-031 | Command without dot prefix → exit 1, mentions '.' | N |
//! | TC-032 | Unknown param key `nope::x` → exit 1, mentions "unknown parameter" | N |
//! | TC-033 | `dry::true` (non-0/1 boolean) → exit 1 | N |
//! | TC-034 | `dry::yes` (non-0/1 boolean) → exit 1 | N |
//! | TC-035 | `force::true` (non-0/1 boolean) → exit 1 | N |
//! | TC-036 | `dry::0` explicitly accepted | P |
//! | TC-037 | `force::0` explicitly accepted | P |
//! | TC-038 | `.help` in second position → exit 0, help output | N→P |
//! | TC-039 | `.help` after multi-part command → exit 0, help output | N→P |
//! | TC-040 | `.help` after params → exit 0, help output | N→P |
//! | TC-484 | `verbosity::3` (canonical) rejected same as `v::3` | N |
//! | TC-485 | `verbosity::-1` (canonical negative) rejected | N |
//! | TC-486 | `verbosity::0` (canonical) accepted, exits 0 | P |
//! | TC-487 | `count::18446744073709551615` (u64 max) → clear error, exit 1 | N |
//! | TC-488 | `count::9223372036854775807` (i64 max) → accepted | P |
//! | TC-489 | bare `help` after command → routes to `.help`, exit 0 | N→P |
//! | TC-490 | bare `help` after params → routes to `.help`, exit 0 | N→P |
//! | TC-491 | `interval::18446744073709551615` (u64 max) → clear error, exit 1 | N |
//! | TC-493 | `dry::0 dry::1` last-wins → dry::1 wins, shows `[dry-run]` | P |
//! | TC-494 | `dry::1 dry::0` last-wins → dry::0 wins, file actually written | P |
//! | TC-495 | `format::text format::json` last-wins → json output | P |

fn run( args : &[ &str ] ) -> std::process::Output
{
  let bin = env!( "CARGO_BIN_EXE_claude_manager" );
  std::process::Command::new( bin )
    .args( args )
    .output()
    .expect( "failed to run cm" )
}

fn out_stdout( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

fn out_stderr( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stderr ).into_owned()
}

fn code( out : &std::process::Output ) -> i32
{
  out.status.code().unwrap_or( -1 )
}

// TC-001: empty argv → help output, exit 0
#[ test ]
fn tc001_empty_argv_shows_help()
{
  let out = run( &[] );
  assert_eq!( code( &out ), 0, "empty argv must exit 0" );
  assert!( out_stdout( &out ).contains( "Available commands:" ), "must show help" );
}

// TC-002: .help → help, exit 0
#[ test ]
fn tc002_dot_help()
{
  let out = run( &[ ".help" ] );
  assert_eq!( code( &out ), 0 );
  assert!( out_stdout( &out ).contains( "Available commands:" ), "must show help" );
}

// TC-004: unknown bogus::1 param → exit 1
#[ test ]
fn tc004_unknown_param_exits_1()
{
  let out = run( &[ ".status", "bogus::1" ] );
  assert_eq!( code( &out ), 1 );
  let err = out_stderr( &out );
  assert!( err.to_lowercase().contains( "unknown parameter" ), "must mention unknown parameter: {err}" );
}

// TC-005: v:: empty value → exit 1
#[ test ]
fn tc005_verbosity_empty_value()
{
  let out = run( &[ ".status", "v::" ] );
  assert_eq!( code( &out ), 1 );
  let err = out_stderr( &out );
  assert!( err.contains( "v::" ), "must mention v::: {err}" );
}

// TC-006: v::3 → exit 1 (out of range)
#[ test ]
fn tc006_verbosity_out_of_range()
{
  let out = run( &[ ".status", "v::3" ] );
  assert_eq!( code( &out ), 1 );
  let err = out_stderr( &out );
  assert!(
    err.contains( "out of range" ) || err.contains( "0, 1, or 2" ),
    "must mention range: {err}"
  );
}

// TC-007: v::abc → exit 1
#[ test ]
fn tc007_verbosity_non_integer()
{
  let out = run( &[ ".status", "v::abc" ] );
  assert_eq!( code( &out ), 1 );
  let err = out_stderr( &out );
  assert!( err.contains( "v::" ), "must mention v::: {err}" );
}

// TC-008: v::0 accepted
#[ test ]
fn tc008_verbosity_0_accepted()
{
  let out = run( &[ ".status", "v::0" ] );
  assert_eq!( code( &out ), 0 );
}

// TC-010: last v:: wins
#[ test ]
fn tc010_last_verbosity_wins()
{
  let out = run( &[ ".status", "v::2", "v::0" ] );
  assert_eq!( code( &out ), 0 );
  let text = out_stdout( &out );
  // v::0 = bare output (no labels)
  assert!( !text.contains( "Version:" ), "last v::0 must win: {text}" );
}

// TC-011: single-word subcommand
#[ test ]
fn tc011_single_word_subcommand()
{
  let out = run( &[ ".status" ] );
  assert_eq!( code( &out ), 0 );
}

// TC-012: two-word subcommand
#[ test ]
fn tc012_two_word_subcommand()
{
  let out = run( &[ ".version.list" ] );
  assert_eq!( code( &out ), 0 );
}

// TC-014: unknown command → exit 1
#[ test ]
fn tc014_unknown_command()
{
  let out = run( &[ ".nonexistent" ] );
  assert_eq!( code( &out ), 1 );
  let err = out_stderr( &out );
  assert!( err.contains( "not found" ), "must mention not found: {err}" );
}

// TC-015: format:: empty value → exit 1
#[ test ]
fn tc015_format_empty_value()
{
  let out = run( &[ ".status", "format::" ] );
  assert_eq!( code( &out ), 1 );
}

// TC-016: version:: empty value → exit 1
#[ test ]
fn tc016_version_param_empty_value()
{
  let out = run( &[ ".version.install", "version::" ] );
  assert_eq!( code( &out ), 1 );
}

// TC-020: dry::1 accepted
#[ test ]
fn tc020_dry_run_param()
{
  let out = run( &[ ".version.install", "dry::1" ] );
  assert_eq!( code( &out ), 0 );
  let text = out_stdout( &out );
  assert!( text.contains( "[dry-run]" ), "must show dry-run: {text}" );
}

// TC-021: force::1 accepted
#[ test ]
fn tc021_force_param()
{
  let out = run( &[ ".version.install", "dry::1", "force::1" ] );
  assert_eq!( code( &out ), 0 );
}

// TC-022: v::0 produces consistent output
#[ test ]
fn tc022_v_param_consistent()
{
  let out_a = run( &[ ".version.list", "v::0" ] );
  let out_b = run( &[ ".version.list", "v::0" ] );
  assert_eq!( code( &out_a ), 0 );
  assert_eq!( code( &out_b ), 0 );
  assert_eq!(
    out_stdout( &out_a ), out_stdout( &out_b ),
    "v::0 must produce identical output"
  );
}

// TC-024: bare token without :: after command → rejected
#[ test ]
fn tc024_bare_token_after_command_rejected()
{
  let out = run( &[ ".version.show", "extra" ] );
  assert_eq!( code( &out ), 1, "bare token after command must exit 1" );
  let err = out_stderr( &out );
  assert!( err.contains( "param::value" ), "must mention param::value syntax: {err}" );
}

// TC-025: bare token without dot prefix and without :: → rejected
#[ test ]
fn tc025_bare_token_without_dot_prefix()
{
  let out = run( &[ "status" ] );
  assert_eq!( code( &out ), 1 );
  let err = out_stderr( &out );
  assert!( err.contains( "'.'" ), "must mention dot prefix requirement: {err}" );
}

// TC-026: .help subcommand explicitly
#[ test ]
fn tc026_help_subcommand_explicitly()
{
  let out = run( &[ ".help" ] );
  assert_eq!( code( &out ), 0 );
  assert!( out_stdout( &out ).contains( "Available commands:" ), "must show help" );
}

// TC-027: -- double-dash token → rejected as non-param::value
#[ test ]
fn tc027_double_dash_rejected()
{
  let out = run( &[ ".status", "--" ] );
  assert_eq!( code( &out ), 1 );
  let err = out_stderr( &out );
  assert!( err.contains( "param::value" ), "-- must be rejected as non-param::value: {err}" );
}

// TC-028: 4-part semver rejected
#[ test ]
fn tc028_four_part_semver_rejected()
{
  let out = run( &[ ".version.install", "version::1.2.3.4" ] );
  assert_eq!( code( &out ), 1 );
}

// TC-029: leading-zero semver rejected
//
// Fix(issue-leading-zeros): leading zeros are not valid semver.
// Root cause: original digits-only check did not reject leading zeros.
// Pitfall: the installer silently accepts leading-zero versions but they
// 404, leaving the user without an installed binary after hot-swap.
#[ test ]
fn tc029_leading_zero_semver_rejected()
{
  let out = run( &[ ".version.install", "version::01.02.03", "dry::1" ] );
  assert_eq!( code( &out ), 1, "leading-zero semver must be rejected" );
}

// TC-030: format::TEXT (wrong case) → exit 1
#[ test ]
fn tc030_format_text_wrong_case_rejected()
{
  let out = run( &[ ".status", "format::TEXT" ] );
  assert_eq!( code( &out ), 1 );
}

// TC-031: command without dot prefix → exit 1, mentions '.'
#[ test ]
fn tc031_command_without_dot_prefix()
{
  let out = run( &[ "version" ] );
  assert_eq!( code( &out ), 1 );
  let err = out_stderr( &out );
  assert!( err.contains( "'.'" ), "must mention dot prefix: {err}" );
}

// TC-032: unknown param key → exit 1, mentions "unknown parameter"
#[ test ]
fn tc032_unknown_param_key()
{
  let out = run( &[ ".status", "nope::x" ] );
  assert_eq!( code( &out ), 1 );
  let err = out_stderr( &out );
  assert!( err.to_lowercase().contains( "unknown parameter" ), "must mention unknown parameter: {err}" );
}

// TC-033: dry::true (non-0/1 boolean) → exit 1
//
// ## Root Cause
//
// Parser accepted any string for `dry::` — only "1" set the flag,
// everything else silently treated as false.  `dry::true` appeared
// to enable dry-run but actually executed real operations.
//
// ## Why Not Caught
//
// Previous tests only used `dry::1`.  No test supplied a non-0/1 value.
//
// ## Fix Applied
//
// Parser rejects any `dry::` value that is not "0" or "1".
//
// ## Prevention
//
// These tests lock down the accepted value set for boolean params.
//
// ## Pitfall
//
// Silent boolean coercion is dangerous: users who type `dry::true`
// expect preview mode but get real execution.
#[ test ]
fn tc033_dry_true_rejected()
{
  let out = run( &[ ".version.install", "dry::true" ] );
  assert_eq!( code( &out ), 1, "dry::true must be rejected" );
  let err = out_stderr( &out );
  assert!( err.contains( "dry::" ), "error must mention dry::: {err}" );
}

// TC-034: dry::yes (non-0/1 boolean) → exit 1
#[ test ]
fn tc034_dry_yes_rejected()
{
  let out = run( &[ ".version.install", "dry::yes" ] );
  assert_eq!( code( &out ), 1, "dry::yes must be rejected" );
}

// TC-035: force::true (non-0/1 boolean) → exit 1
#[ test ]
fn tc035_force_true_rejected()
{
  let out = run( &[ ".version.install", "dry::1", "force::true" ] );
  assert_eq!( code( &out ), 1, "force::true must be rejected" );
  let err = out_stderr( &out );
  assert!( err.contains( "force::" ), "error must mention force::: {err}" );
}

// TC-036: dry::0 explicitly accepted
#[ test ]
fn tc036_dry_0_accepted()
{
  let out = run( &[ ".version.install", "dry::0" ] );
  // dry::0 means no dry-run — but command still runs (may exit 0 or 2)
  assert_ne!( code( &out ), 1, "dry::0 is valid, must not exit 1" );
}

// TC-037: force::0 explicitly accepted
#[ test ]
fn tc037_force_0_accepted()
{
  let out = run( &[ ".version.install", "dry::1", "force::0" ] );
  assert_eq!( code( &out ), 0, "force::0 is valid" );
}

// TC-038: .help in second position → exit 0, help output
//
// ## Root Cause
//
// `argv_to_unilang_tokens` only checked for `.help` as `argv[0]`.  Tokens in
// positions 1+ that lacked `::` were rejected as malformed params, so
// `.status .help` raised a parse error instead of showing help (FR-02 violation).
//
// ## Why Not Caught
//
// Only TC-002 tested `.help`, and only as the sole argument.  No test supplied
// `.help` after a command name.
//
// ## Fix Applied
//
// Added a pre-scan pass over all argv for the exact token `".help"`.  If found
// anywhere, routes to `".help"` immediately (satisfies FR-02).
//
// ## Prevention
//
// TC-038..TC-040 lock down `".help"` anywhere-in-argv behaviour.
//
// ## Pitfall
//
// Without the pre-scan, `.status .help` raises a parse error instead of
// showing help, breaking the discoverable help pattern that FR-02 requires.
#[ test ]
fn tc038_help_in_second_position()
{
  let out = run( &[ ".status", ".help" ] );
  assert_eq!( code( &out ), 0, "`.status .help` must exit 0" );
  let stdout = out_stdout( &out );
  assert!( stdout.contains( "Available commands:" ), "must show help listing: {stdout}" );
}

// TC-039: .help after multi-part command → exit 0, help output
#[ test ]
fn tc039_help_after_multi_part_command()
{
  let out = run( &[ ".version.install", ".help" ] );
  assert_eq!( code( &out ), 0, "`.version.install .help` must exit 0" );
  let stdout = out_stdout( &out );
  assert!( stdout.contains( "Available commands:" ), "must show help listing: {stdout}" );
}

// TC-040: .help after params → exit 0, help output
#[ test ]
fn tc040_help_after_params()
{
  let out = run( &[ ".version.guard", "dry::1", ".help" ] );
  assert_eq!( code( &out ), 0, "`.version.guard dry::1 .help` must exit 0" );
  let stdout = out_stdout( &out );
  assert!( stdout.contains( "Available commands:" ), "must show help listing: {stdout}" );
}

// TC-484: verbosity::3 (canonical key) rejected like v::3
//
// ## Root Cause
//
// The adapter validated `v::` (alias) but not `verbosity::` (canonical key).
// `verbosity::3` bypassed range checks: u8::try_from(3) succeeds, and the
// handler silently treated it as level 2 (v >= 2 branch).
//
// ## Why Not Caught
//
// All existing tests used `v::N` (alias form). No test supplied `verbosity::N`
// (canonical form) with an out-of-range value.
//
// ## Fix Applied
//
// Adapter now validates both `v::` and `verbosity::` in the same branch,
// rejecting any value outside 0–2 with a clear error message using the key
// name the user supplied.
//
// ## Prevention
//
// TC-484/TC-485 lock the canonical-key path; TC-006 already guards the alias.
//
// ## Pitfall
//
// Skipping canonical-key validation creates an asymmetry: `v::3` fails but
// `verbosity::3` silently succeeds, misleading users about accepted values.
#[ test ]
fn tc484_verbosity_canonical_out_of_range_rejected()
{
  let out = run( &[ ".status", "verbosity::3" ] );
  assert_eq!( code( &out ), 1, "verbosity::3 must be rejected (exit 1)" );
  let err = out_stderr( &out );
  assert!(
    err.contains( "out of range" ) || err.contains( "0, 1, or 2" ) || err.contains( "verbosity::" ),
    "error must mention range or verbosity: {err}"
  );
}

// TC-485: verbosity::-1 (canonical negative) rejected
//
// ## Root Cause
//
// Same bypass as TC-484: `verbosity::` skipped adapter range validation.
// parse::<u8>() on "-1" fails with `InvalidDigit`, so the error is actually
// "must be 0, 1, or 2" — but before the fix this branch was never reached,
// and unilang parsed -1 as i64 then u8::try_from(-1).unwrap_or(1) silently
// produced verbosity=1.
//
// ## Why Not Caught
//
// Only `v::-1` was tried. `verbosity::-1` was not tested.
//
// ## Fix Applied
//
// Same fix as TC-484: canonical key now goes through the same validation path.
//
// ## Prevention
//
// TC-485 covers the negative-value path for the canonical key.
//
// ## Pitfall
//
// Without the fix, `verbosity::-1` silently defaulted to verbosity=1 instead
// of exiting with a clear error, masking a user typo.
#[ test ]
fn tc485_verbosity_canonical_negative_rejected()
{
  let out = run( &[ ".status", "verbosity::-1" ] );
  assert_eq!( code( &out ), 1, "verbosity::-1 must be rejected (exit 1)" );
  let err = out_stderr( &out );
  assert!(
    err.contains( "verbosity::" ),
    "error must mention verbosity: {err}"
  );
}

// TC-486: verbosity::0 (canonical) accepted, exits 0
#[ test ]
fn tc486_verbosity_canonical_zero_accepted()
{
  let out = run( &[ ".status", "verbosity::0" ] );
  assert_eq!( code( &out ), 0, "verbosity::0 is valid, must exit 0" );
  // v::0 produces bare output (no "Version:" label)
  assert!( !out_stdout( &out ).contains( "Version:" ), "verbosity::0 must not show labels" );
}

// TC-487: count::18446744073709551615 (u64 max, exceeds i64 max) → clear error, exit 1
//
// ## Root Cause
//
// The adapter parsed count:: with u64 (accepting values > i64::MAX), then
// passed the raw string to unilang, which uses i64 internally. The unilang
// type parser then emitted a cryptic "number too large to fit in target type"
// error instead of the adapter's user-friendly "must be a non-negative integer"
// message.
//
// ## Why Not Caught
//
// Tests only used small values (0, 1, 10, 66). The u64/i64 boundary was not
// exercised.
//
// ## Fix Applied
//
// Adapter now rejects count:: / interval:: values > i64::MAX with a clear
// "value too large" message before the token reaches unilang.
//
// ## Prevention
//
// TC-487 reproduces the overflow scenario; TC-488 ensures the valid boundary
// (i64::MAX) is still accepted.
//
// ## Pitfall
//
// Documenting count:: as "non-negative integer" implies the full u64 range is
// valid. Without the upper bound check, values just above i64::MAX sneak through
// the adapter only to be rejected with an unhelpful internal error later.
#[ test ]
fn tc487_count_u64_max_rejected_with_clear_error()
{
  let out = run( &[ ".version.history", "count::18446744073709551615" ] );
  assert_eq!( code( &out ), 1, "count::u64_max must be rejected (exit 1)" );
  let err = out_stderr( &out );
  assert!(
    err.contains( "count::" ),
    "error must mention count: {err}"
  );
  // Must NOT produce the cryptic unilang "number too large to fit in target type" message.
  assert!(
    !err.contains( "fit in target type" ),
    "must not expose internal type error: {err}"
  );
}

// TC-488: count::9223372036854775807 (i64::MAX) accepted
#[ test ]
fn tc488_count_i64_max_accepted()
{
  // count::i64::MAX passes through the adapter without error.
  // .version.list doesn't use count:: so it rejects it as unknown param.
  // Use a command that ignores unknown count values (version.guard doesn't have count).
  // Actually we test at the adapter level: the adapter must NOT reject i64::MAX for count.
  // Use .version.history; it may fail at the network level (exit 2) but must NOT exit 1
  // due to count:: validation error.
  let out = run( &[ ".version.history", "count::9223372036854775807" ] );
  // Must not exit 1 (which would indicate a count:: validation failure)
  assert_ne!( code( &out ), 1, "count::i64_max must not be rejected by adapter (exit must not be 1)" );
}

// TC-489: bare `help` after command → routes to `.help`, exit 0
//
// ## Root Cause
//
// The adapter recognised `.help` anywhere in argv (Step 1b) but not bare `help`
// (without the leading dot). The unilang help footer instructs users
// "Use '<command> help' to get detailed help for a specific command." — so
// `clm .version.show help` is the documented invocation. Without the bare-`help`
// check, the adapter rejected `help` with "expected param::value syntax, got: 'help'"
// because `help` lacks the `::` separator required for key::value tokens.
//
// ## Why Not Caught
//
// All existing tests used `.help` (with dot). The help footer's example was
// never tested against the actual adapter behaviour; the mismatch went unnoticed.
//
// ## Fix Applied
//
// Step 1b of `argv_to_unilang_tokens` now checks for both `".help"` and `"help"`,
// routing either form to the global `.help` command.
//
// ## Prevention
//
// TC-489 and TC-490 lock the bare-`help` path. Any future regression in
// Step 1b will be caught immediately.
//
// ## Pitfall
//
// The help footer is generated by the `unilang` crate and cannot be patched here.
// The adapter must accept both spellings so the documented syntax actually works.
#[ test ]
fn tc489_bare_help_after_command_routes_to_help()
{
  let out = run( &[ ".version.show", "help" ] );
  assert_eq!( code( &out ), 0, "`.version.show help` must exit 0" );
  let stdout = out_stdout( &out );
  assert!( stdout.contains( "Available commands:" ), "must show help listing: {stdout}" );
}

// TC-490: bare `help` after params → routes to `.help`, exit 0
#[ test ]
fn tc490_bare_help_after_params_routes_to_help()
{
  let out = run( &[ ".version.history", "count::3", "help" ] );
  assert_eq!( code( &out ), 0, "`.version.history count::3 help` must exit 0" );
  let stdout = out_stdout( &out );
  assert!( stdout.contains( "Available commands:" ), "must show help listing: {stdout}" );
}

// TC-491: interval::u64max (exceeds i64::MAX) → clear error, exit 1
//
// ## Root Cause
//
// Same overflow boundary as count:: (TC-487): the adapter parses interval::
// as u64, then rejects values above i64::MAX before they reach unilang's i64
// parser. Without this guard, u64_max would produce a cryptic type-error.
//
// ## Why Not Caught
//
// TC-487/TC-488 document the count:: boundary but no parallel tests existed
// for interval::, leaving the overflow guard path untested for that param.
//
// ## Fix Applied
//
// Both count:: and interval:: share the same `validate_non_neg_int` path —
// the fix was already present; this test locks it down.
//
// ## Prevention
//
// Any non-negative integer param added in future must have a corresponding
// u64_max rejection test alongside its i64_max acceptance note.
//
// ## Pitfall
//
// Documenting a param as "non-negative integer" implies the full u64 range.
// Without an explicit upper-bound check (i64::MAX), values just above the
// boundary reach unilang and produce an opaque internal error.
#[ test ]
fn tc491_interval_u64_max_rejected_with_clear_error()
{
  let out = run( &[ ".version.guard", "interval::18446744073709551615" ] );
  assert_eq!( code( &out ), 1, "interval::u64_max must be rejected (exit 1)" );
  let err = out_stderr( &out );
  assert!(
    err.contains( "interval::" ),
    "error must mention interval: {err}"
  );
  assert!(
    !err.contains( "fit in target type" ),
    "must not expose internal type error: {err}"
  );
}

// TC-493: dry::0 dry::1 — last occurrence wins → dry::1 active (dry-run)
//
// ## Root Cause
//
// The adapter implements last-occurrence-wins for all repeated params via
// `pairs.iter_mut().find(...)`. Without a test, a regression could silently
// reverse the semantics so the FIRST occurrence wins, causing dry::0 dry::1
// to run real operations while appearing to accept the dry::1 override.
//
// ## Why Not Caught
//
// TC-010 tests last-wins for v::, but no test covered dry:: or force::.
//
// ## Fix Applied
//
// Behaviour was already correct; this test locks the contract.
//
// ## Prevention
//
// TC-493/TC-494 together verify both directions of dry:: last-wins.
//
// ## Pitfall
//
// If first-wins semantics were accidentally introduced, `dry::0 dry::1` would
// silently execute destructive operations despite the user's dry::1 intention.
#[ test ]
fn tc493_dry_0_then_1_last_wins_dry_active()
{
  let out = run( &[ ".version.install", "dry::0", "dry::1" ] );
  assert_eq!( code( &out ), 0, "dry::0 dry::1 must exit 0" );
  let text = out_stdout( &out );
  assert!(
    text.contains( "[dry-run]" ),
    "dry::1 (last) must win: output must contain [dry-run]: {text}"
  );
}

// TC-494: dry::1 dry::0 — last occurrence wins → dry::0 active (no dry-run)
//
// ## Root Cause
//
// Same as TC-493: verifies the other direction. With dry::0 winning, the
// command attempts real execution. Uses .settings.set in an isolated tmp dir
// to avoid destructive side effects.
//
// ## Why Not Caught
//
// Only TC-010 tested last-wins; dry:: was not covered.
//
// ## Fix Applied
//
// Behaviour was already correct.
//
// ## Prevention
//
// Isolation via temp HOME ensures no real settings are modified.
//
// ## Pitfall
//
// Without this test, a regression where first-wins takes hold would mean
// dry::1 dry::0 silently enables dry-run mode, suppressing real writes.
#[ test ]
fn tc494_dry_1_then_0_last_wins_dry_inactive()
{
  let dir = tempfile::TempDir::new().expect( "failed to create tmpdir" );
  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_claude_manager" ) )
  .args( [ ".settings.set", "key::probe", "value::check", "dry::1", "dry::0" ] )
  .env( "HOME", dir.path() )
  .output()
  .expect( "failed to run cm" );

  // dry::0 wins → real write, so settings file must exist
  let settings_file = dir.path().join( ".claude/settings.json" );
  assert!(
    settings_file.exists(),
    "dry::0 (last) must win: settings file must be written"
  );
  // Must NOT show [dry-run] prefix
  let text = String::from_utf8_lossy( &out.stdout ).into_owned();
  assert!(
    !text.contains( "[dry-run]" ),
    "dry::0 (last) must win: output must NOT contain [dry-run]: {text}"
  );
}

// TC-495: format::text format::json — last occurrence wins → json output
//
// ## Root Cause
//
// Last-wins is already verified for v:: (TC-010) but not for format::.
// A regression where first-wins takes hold would silently emit text instead
// of json when both params are supplied, breaking pipe-based tooling.
//
// ## Why Not Caught
//
// TC-010 only tested v::. No test verified format:: last-wins.
//
// ## Fix Applied
//
// Behaviour was already correct; this test locks it.
//
// ## Prevention
//
// Test both orderings to catch either direction of regression.
//
// ## Pitfall
//
// format:: errors are silent: wrong format produces valid but differently-
// structured output that downstream consumers may silently misparse.
#[ test ]
fn tc495_format_text_then_json_last_wins_json()
{
  let out = run( &[ ".version.list", "format::text", "format::json" ] );
  assert_eq!( code( &out ), 0, "format::text format::json must exit 0" );
  let text = out_stdout( &out );
  assert!(
    text.trim_start().starts_with( '[' ),
    "format::json (last) must win, output must start with '[': {text}"
  );
}
