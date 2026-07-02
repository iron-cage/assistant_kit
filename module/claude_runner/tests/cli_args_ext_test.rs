//! CLI Argument Parsing Tests — Extended (T36–T47, T49, EC01–EC06, S58–S69, S79, BUG-212, BUG-215, BUG-302)
//!
//! ## Purpose
//!
//! Extension of `cli_args_test.rs` (T01–T35) covering positional arg edge cases,
//! session/interaction/permissions flag combinations, new runner flags (--file,
//! --strip-fences, --keep-claudecode), and bug reproducers for the `run` subcommand.
//!
//! ## Strategy
//!
//! All tests invoke the compiled binary via `env!("CARGO_BIN_EXE_clr")`.
//! `--dry-run` outputs the command line that would be executed, allowing
//! assertions against the translation of flags → builder calls.
//!
//! ## Corner Cases Covered
//!
//! - T36: flags after positional args still parsed
//! - T37: multiple positional words joined as message
//! - T38: `--` with nothing after → no message
//! - T39: `--max-tokens ""` empty string rejected
//! - T40: all value-flags at end of argv require value
//! - T41: `--new-session --dry-run` output does NOT contain `-c`
//! - T42: message without `-p` → dry-run output contains `--print`
//! - T43: `--interactive` with message → dry-run output does NOT contain `--print`
//! - T44: `--interactive` alone (no message) → accepted, no error
//! - T45: `--interactive` listed in `--help` output
//! - T46: `--no-skip-permissions` removes `--dangerously-skip-permissions`
//! - T47: `--dangerously-skip-permissions` explicit → rejected as unknown option
//! - T49: all `--help` option lines have descriptions at the same column
//! - S58–S79: new flag parsing (--strip-fences, --keep-claudecode, --file)
//! - BUG-212: `clr run <message>` → leading `run` token stripped
//! - BUG-215: `clr run help` → output identical to `clr help`
//! - T48: `--no-skip-permissions --new-session` combo → no `-c`, no skip-permissions
//! - BUG-302: `clr is` (common word, `"isolated".starts_with("is")`) does NOT trigger guard

mod cli_binary_test_helpers;
use cli_binary_test_helpers::run_cli;

// T36: flags after positional are still parsed
#[ test ]
fn t36_flags_after_positional()
{
  let out = run_cli( &[ "--dry-run", "msg", "--verbose" ] );
  assert!( out.status.success(), "flags after positional must work" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--verbose" ),
    "--verbose after positional must be parsed as flag. Got:\n{stdout}"
  );
}

// T37: multiple positional words joined as message
#[ test ]
fn t37_multiple_positional_words_joined()
{
  let out = run_cli( &[ "--dry-run", "Fix", "the", "bug", "now" ] );
  assert!( out.status.success(), "multiple positional words must be accepted" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "\"Fix the bug now\n\nultrathink\"" ),
    "all positional words must join with space and be ultrathink-suffixed. Got:\n{stdout}"
  );
}

// T38: `--` as only arg (besides --dry-run) → no message
#[ test ]
fn t38_double_dash_only_no_message()
{
  // Empty session dir → no -c injection (session_exists returns `None` for empty dir).
  // Do NOT use make_session_dir() here: that writes a dummy .jsonl so session_exists()
  // returns `Some(SessionId)` and injects -c, which contradicts this test's "no -c" intent.
  let empty_dir = tempfile::TempDir::new().expect( "create empty session dir" );
  let session_path = empty_dir.path().to_str().expect( "session dir path valid utf-8" );
  let out = run_cli( &[ "--dry-run", "--session-dir", session_path, "--" ] );
  assert!( out.status.success(), "-- as only arg must not error" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  let last_line = stdout.trim_end().lines().last().unwrap_or_default();
  // Fix(BUG-246): describe() now starts with "env -u CLAUDECODE" (default unset_claudecode=true)
  assert_eq!(
    last_line,
    "env -u CLAUDECODE claude --dangerously-skip-permissions --chrome --effort max",
    "-- with nothing after must produce bare command (no -c in empty session dir). Got:\n{stdout}"
  );
}

// T39: --max-tokens empty string rejected
#[ test ]
fn t39_max_tokens_empty_string_rejected()
{
  let out = run_cli( &[ "--dry-run", "--max-tokens", "", "test" ] );
  assert!( !out.status.success(), "--max-tokens '' must be rejected" );
}

// T40: all value-flags at end of argv produce "requires a value" error
#[ test ]
fn t40_all_value_flags_require_value()
{
  for flag in &[
    "--max-tokens", "--session-dir", "--dir",
    "--system-prompt", "--append-system-prompt",
  ]
  {
    let out = run_cli( &[ "--dry-run", flag ] );
    assert!(
      !out.status.success(),
      "{flag} as last arg must exit non-zero"
    );
    let stderr = String::from_utf8_lossy( &out.stderr );
    assert!(
      stderr.contains( "requires a value" ),
      "{flag} must mention 'requires a value'. Got:\n{stderr}"
    );
  }
}

// T41: --new-session --dry-run output does NOT contain -c
#[ test ]
fn t41_new_session_suppresses_continue_flag()
{
  let out = run_cli( &[ "--dry-run", "--new-session", "test" ] );
  assert!( out.status.success(), "--new-session --dry-run must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( " -c" ),
    "--new-session must suppress -c in dry-run output. Got:\n{stdout}"
  );
}

// T42: message without -p → dry-run output contains --print (default print with message)
#[ test ]
fn t42_message_defaults_to_print_mode()
{
  let out = run_cli( &[ "--dry-run", "Fix the bug" ] );
  assert!( out.status.success(), "message without -p must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--print" ),
    "message without -p must default to print mode (--print in dry-run). Got:\n{stdout}"
  );
}

// T43: --interactive with message → dry-run output does NOT contain --print
#[ test ]
fn t43_interactive_flag_suppresses_print()
{
  let out = run_cli( &[ "--dry-run", "--interactive", "Fix the bug" ] );
  assert!( out.status.success(), "--interactive with message must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--print" ),
    "--interactive must suppress --print default. Got:\n{stdout}"
  );
}

// T44: --interactive alone (no message) → accepted, no error
#[ test ]
fn t44_interactive_flag_alone_accepted()
{
  // --interactive with no message must not crash; bare clr still opens interactive REPL.
  // Use --dry-run to avoid needing a real claude binary.
  let out = run_cli( &[ "--dry-run", "--interactive" ] );
  assert!(
    out.status.success(),
    "--interactive alone must be accepted (exit 0). stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--print" ),
    "--interactive with no message must not add --print. Got:\n{stdout}"
  );
}

// T45: --interactive listed in --help output
#[ test ]
fn t45_interactive_flag_in_help()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "--help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--interactive" ),
    "--interactive must appear in --help output. Got:\n{stdout}"
  );
}

// T46: --no-skip-permissions disables the default permission bypass
#[ test ]
fn t46_no_skip_permissions_disables_default()
{
  let out = run_cli( &[ "--dry-run", "--no-skip-permissions", "test" ] );
  assert!( out.status.success(), "exit={} stderr={}", out.status.code().unwrap_or( -1 ), String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--dangerously-skip-permissions" ),
    "--no-skip-permissions must suppress automatic bypass. Got:\n{stdout}"
  );
}

// T47: --dangerously-skip-permissions explicit → rejected as unknown option
//
// Regression guard: this flag was previously user-facing in clr. After task 058 it was
// hidden (always-on by default). Explicit use must be rejected so users know to use
// --no-skip-permissions as the opt-out instead of trying to pass the hidden flag.
#[ test ]
fn t47_explicit_dangerously_skip_permissions_rejected()
{
  let out = run_cli( &[ "--dry-run", "--dangerously-skip-permissions", "test" ] );
  assert!(
    !out.status.success(),
    "--dangerously-skip-permissions explicit must exit non-zero (now hidden; always-on by default)"
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "unknown option" ),
    "explicit --dangerously-skip-permissions must report 'unknown option'. Got:\n{stderr}"
  );
}

// T49: all option lines in --help have descriptions aligned at the same column
//
// Regression guard for help output formatting: when a flag name is longer than the
// standard padding width, it's easy to add one extra space and misalign the column.
// All option lines (starting with "  -") must start their description word at the
// same character position in the line.
#[ test ]
fn t49_help_options_column_aligned()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "--help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );

  // Collect (column, line) for every option line (starts with "  -").
  // Column = index of the first description character (first char after a 2+ space gap).
  let mut col_by_line : Vec< ( usize, String ) > = Vec::new();
  for line in stdout.lines()
  {
    if !line.starts_with( "  -" ) { continue; }
    let bytes = line.as_bytes();
    let mut i = 2; // skip leading "  "
    while i < bytes.len()
    {
      if bytes[ i ] == b' '
      {
        let gap_start = i;
        while i < bytes.len() && bytes[ i ] == b' ' { i += 1; }
        if i - gap_start >= 2
        {
          col_by_line.push( ( i, line.to_string() ) );
          break;
        }
      }
      else { i += 1; }
    }
  }

  assert!( !col_by_line.is_empty(), "--help must contain option lines" );
  let expected_col = col_by_line[ 0 ].0;
  for ( col, line ) in &col_by_line
  {
    assert_eq!(
      *col, expected_col,
      "all option descriptions must start at column {expected_col}. Misaligned line:\n  {line}"
    );
  }
}

// ── EC-01–EC-06: Help section split (TSK-232 / Plan 030) ──────────────────────

// EC-01: help contains RUNNER OPTIONS section header
#[ test ]
fn ec01_help_contains_runner_options_section()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "RUNNER OPTIONS:" ),
    "help must contain RUNNER OPTIONS section. Got:\n{stdout}"
  );
}

// EC-02: help contains CLAUDE CODE OPTIONS (forwarded) section header
#[ test ]
fn ec02_help_contains_claude_code_options_section()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "CLAUDE CODE OPTIONS (forwarded):" ),
    "help must contain CLAUDE CODE OPTIONS (forwarded) section. Got:\n{stdout}"
  );
}

// EC-03: help has nine usage forms (one per command, including scope)
#[ test ]
fn ec03_help_has_nine_usage_forms()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  let count = stdout.lines().filter( | l | l.starts_with( "  clr " ) ).count();
  assert_eq!(
    count, 9,
    "help must have 9 usage lines starting with '  clr '. Got {count}:\n{stdout}"
  );
}

// EC-04: help contains Commands section
#[ test ]
fn ec04_help_contains_commands_section()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "Commands:" ),
    "help must contain Commands section. Got:\n{stdout}"
  );
}

// EC-05: help contains key flags across both option groups
#[ test ]
fn ec05_help_contains_key_flags_across_groups()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "--model" ), "missing --model. Got:\n{stdout}" );
  assert!( stdout.contains( "--timeout" ), "missing --timeout. Got:\n{stdout}" );
  assert!( stdout.contains( "--max-sessions" ), "missing --max-sessions. Got:\n{stdout}" );
}

// EC-06: help does not contain standalone OPTIONS header (replaced by split groups)
#[ test ]
fn ec06_help_no_standalone_options_header()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "\nOPTIONS:\n" ),
    "help must NOT contain standalone OPTIONS header. Got:\n{stdout}"
  );
}

// ── S58–S69, S79: New flag parsing tests ────────────────────────────────────────

// S58: --strip-fences accepted in dry-run
#[ test ]
fn s58_strip_fences_flag_accepted()
{
  let out = run_cli( &[ "--dry-run", "--strip-fences", "t" ] );
  assert!( out.status.success(), "--strip-fences must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
}

// S59: --keep-claudecode accepted in dry-run
#[ test ]
fn s59_keep_claudecode_flag_accepted()
{
  let out = run_cli( &[ "--dry-run", "--keep-claudecode", "t" ] );
  assert!( out.status.success(), "--keep-claudecode must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
}

// S60: --file without value → error
#[ test ]
fn s60_file_requires_a_value()
{
  let out = run_cli( &[ "--dry-run", "--file" ] );
  assert!( !out.status.success(), "--file without value must fail" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!( stderr.contains( "requires a value" ), "stderr must mention 'requires a value'. Got: {stderr}" );
}

// S61: --file with path accepted
#[ test ]
fn s61_file_with_path_accepted()
{
  let out = run_cli( &[ "--dry-run", "--file", "/tmp/x.txt", "t" ] );
  assert!( out.status.success(), "--file with path must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
}

// S62: strip_fences absent by default
#[ test ]
fn s62_strip_fences_absent_by_default()
{
  let out = run_cli( &[ "--dry-run", "t" ] );
  assert!( out.status.success(), "default must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
}

// S63: keep_claudecode absent by default
#[ test ]
fn s63_keep_claudecode_absent_by_default()
{
  let out = run_cli( &[ "--dry-run", "t" ] );
  assert!( out.status.success(), "default must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
}

// S64: --file and --strip-fences together
#[ test ]
fn s64_file_and_strip_fences_together()
{
  let out = run_cli( &[ "--dry-run", "--file", "/tmp/x.txt", "--strip-fences", "t" ] );
  assert!( out.status.success(), "combo must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
}

// S65: --file and --keep-claudecode together
#[ test ]
fn s65_file_and_keep_claudecode_together()
{
  let out = run_cli( &[ "--dry-run", "--file", "/tmp/x.txt", "--keep-claudecode", "t" ] );
  assert!( out.status.success(), "combo must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
}

// S66: --strip-fences and --keep-claudecode together
#[ test ]
fn s66_strip_fences_and_keep_claudecode_together()
{
  let out = run_cli( &[ "--dry-run", "--strip-fences", "--keep-claudecode", "t" ] );
  assert!( out.status.success(), "combo must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
}

// S67: all three new flags together
#[ test ]
fn s67_all_three_new_flags_together()
{
  let out = run_cli( &[ "--dry-run", "--file", "/tmp/x.txt", "--strip-fences", "--keep-claudecode", "t" ] );
  assert!( out.status.success(), "all three must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
}

// S68: help includes --file
#[ test ]
fn s68_help_includes_file()
{
  let out = run_cli( &[ "--help" ] );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "--file" ), "help must mention --file. Got:\n{stdout}" );
}

// S69: help includes --strip-fences
#[ test ]
fn s69_help_includes_strip_fences()
{
  let out = run_cli( &[ "--help" ] );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "--strip-fences" ), "help must mention --strip-fences. Got:\n{stdout}" );
}

// S79: help includes --keep-claudecode
#[ test ]
fn s79_help_includes_keep_claudecode()
{
  let out = run_cli( &[ "--help" ] );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "--keep-claudecode" ), "help must mention --keep-claudecode. Got:\n{stdout}" );
}

// BUG-212: `clr run <message>` treated "run" as the first positional message word.
//
// ## Root Cause (bug_reproducer(BUG-212))
//
// `run_cli()` collected argv tokens and passed them directly to `parse_args()` without
// special-casing the `run` subcommand.  `clr run "Fix bug"` yielded tokens
// `["run", "Fix bug"]`; both were treated as positional words, producing
// message "run Fix bug" instead of "Fix bug".
//
// ## Why Not Caught
//
// All existing message tests invoked `clr <message>` without an explicit `run` prefix.
// The `run` subcommand appeared in `--help` USAGE but was never exercised in test.
//
// ## Fix Applied
//
// `lib.rs run_cli()` strips the leading "run" token before passing `tokens` to
// `parse_args()`, making `clr run <args>` and `clr <args>` parse identically.
//
// ## Prevention
//
// Pin the invariant: `clr run <args>` and `clr <args>` must produce identical dry-run
// output.  Any regression that reintroduces "run" in the message causes the equivalence
// assertion to fail.
//
// ## Pitfall
//
// Strip only `tokens[0] == "run"` before flag parsing — a message starting with "run"
// (e.g. `clr "run tests"`) must NOT be stripped.  The check is position-sensitive:
// only the very first token, only when it equals "run" exactly.
// test_kind: bug_reproducer(BUG-212)
#[ test ]
fn bug_reproducer_212_run_subcommand_strips_token()
{
  // Invoke with explicit `run` subcommand prefix.
  let with_run = run_cli( &[ "run", "--dry-run", "Fix bug" ] );
  assert!(
    with_run.status.success(),
    "clr run must exit 0. stderr: {}",
    String::from_utf8_lossy( &with_run.stderr )
  );

  // Invoke without `run` prefix — canonical form; both must be identical.
  let without_run = run_cli( &[ "--dry-run", "Fix bug" ] );
  assert!( without_run.status.success(), "clr without run must exit 0" );

  let out_with    = String::from_utf8_lossy( &with_run.stdout );
  let out_without = String::from_utf8_lossy( &without_run.stdout );

  // Message must be "Fix bug", not "run Fix bug".
  assert!(
    out_with.contains( "\"Fix bug\n\nultrathink\"" ),
    "message must be 'Fix bug' (not 'run Fix bug'). Got:\n{out_with}"
  );

  // `clr run <args>` and `clr <args>` must produce identical dry-run output.
  assert_eq!(
    out_with.trim(), out_without.trim(),
    "`clr run <args>` and `clr <args>` must produce identical dry-run output"
  );
}

// BUG-212 (extended coverage): `clr run` — message content, bare form, and flag passthrough.
//
// ## Root Cause (bug_reproducer(BUG-212))
//
// Same root as primary reproducer: `run_cli()` passed argv tokens verbatim to
// `parse_args()`.  Any token at position 0 named "run" was collected as the first
// positional word of the message.  Three separate observable symptoms: (a) message
// contamination, (b) bare form divergence from implicit default, (c) flags following
// "run" were still parsed correctly only by accident because they started with "-".
//
// ## Why Not Caught
//
// The three symptom variants were never tested in isolation.  Only the equivalence
// check (`clr run x` == `clr x`) was added in the primary reproducer; the bare
// no-message form and flag passthrough were not independently asserted.
//
// ## Fix Applied
//
// `lib.rs run_cli()` strips `tokens[0]` when it equals "run" before calling
// `parse_args()`.  All three observable symptoms are resolved by the single strip.
//
// ## Prevention
//
// Test the three distinct behavioural surface areas separately so that a partial
// regression (e.g. bare form works but message contamination returns) fails visibly
// rather than hiding behind a passing equivalence check.
//
// ## Pitfall
//
// Only strip position 0 when it equals "run" exactly.  A message that starts with
// the word "run" (e.g. `clr "run tests"`) must not be stripped — the guard is
// position-sensitive and only fires when tokens[0] == "run".
//
// test_kind: bug_reproducer(BUG-212)
#[ test ]
fn bug_reproducer_212_run_subcommand_args()
{
  // (a) `clr run hello --dry-run` — "hello" is the message; "run" must NOT appear in it.
  let out = run_cli( &[ "run", "hello", "--dry-run" ] );
  assert!(
    out.status.success(),
    "clr run hello --dry-run must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "\"hello\n\nultrathink\"" ),
    "message must be 'hello' with ultrathink suffix. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "run hello" ),
    "'run' must NOT appear in the message (bug: treated as first positional word). Got:\n{stdout}"
  );

  // (b) `clr run --dry-run` with no message — identical to `clr --dry-run`.
  let with_run    = run_cli( &[ "run", "--dry-run" ] );
  let without_run = run_cli( &[ "--dry-run" ] );
  assert!( with_run.status.success(), "clr run --dry-run must exit 0" );
  assert_eq!(
    String::from_utf8_lossy( &with_run.stdout ).trim(),
    String::from_utf8_lossy( &without_run.stdout ).trim(),
    "clr run --dry-run must produce same output as clr --dry-run (bare command form)"
  );

  // (c) `clr run --model sonnet --dry-run` — flag parsed after run token stripped.
  let out = run_cli( &[ "run", "--model", "sonnet", "--dry-run" ] );
  assert!(
    out.status.success(),
    "clr run --model sonnet --dry-run must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--model sonnet" ),
    "--model sonnet must appear in assembled command. Got:\n{stdout}"
  );
}

// T48: --no-skip-permissions --new-session combo disables BOTH automatic defaults
//
// When both opt-out flags are present: no --dangerously-skip-permissions AND no -c.
// The resulting command is bare `claude --print "msg"` (or `claude` without message).
#[ test ]
fn t48_no_skip_permissions_new_session_combination()
{
  let out = run_cli( &[ "--dry-run", "--no-skip-permissions", "--new-session", "--no-ultrathink", "hello" ] );
  assert!(
    out.status.success(),
    "--no-skip-permissions --new-session must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--dangerously-skip-permissions" ),
    "--no-skip-permissions must suppress automatic bypass. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( " -c" ),
    "--new-session must suppress automatic continuation. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "\"hello\"" ),
    "message must still appear. Got:\n{stdout}"
  );
}

// BUG-215: `clr run help` fell through to parse_args treating "help" as a positional
// message — invoking claude instead of printing help.
//
// ## Root Cause
//
// `run_cli()` in `lib.rs` dispatches `help` before the `run` token stripping. After
// stripping, the remaining `["help"]` was NOT re-checked against the `help` dispatch.
// `parse_args(["help"])` collected "help" as a positional argument (message = "help")
// and the binary proceeded to invoke claude with "help\n\nultrathink" as a prompt.
//
// ## Why Not Caught
//
// BUG-212 tests verified `clr run <message>` and `clr run --dry-run` but never tested
// `clr run help`. The subcommand routing for `help` was covered only for bare `clr help`,
// not for the `run`-prefixed form.
//
// ## Fix Applied
//
// Added a `help` re-dispatch check in `lib.rs` immediately after the `run` token strip:
// if the first remaining token is "help", print_help() is called and the function returns.
//
// ## Prevention
//
// Whenever a token is stripped before subcommand dispatch, all dispatches that were
// checked before the strip must be re-checked after it. The `help` dispatch lived above
// the strip and therefore required a matching post-strip guard.
//
// ## Pitfall
//
// Only the `run` prefix triggers the strip. Other subcommand prefixes do not; `help`
// appears in KNOWN and is passed to guard_unknown_subcommand — but "help" == "help"
// means the prefix-check fires for `first != sub` = false, so it passes through silently
// to parse_args instead of erroring. The re-dispatch is the only correct fix.
//
// test_kind: bug_reproducer(BUG-215)
#[ test ]
fn bug_reproducer_215_run_help_dispatches_help()
{
  // `clr run help` must print help and exit 0 — same as `clr help`.
  let with_run = run_cli( &[ "run", "help" ] );
  assert!(
    with_run.status.success(),
    "`clr run help` must exit 0 (BUG-215: was hanging invoking claude). stderr: {}",
    String::from_utf8_lossy( &with_run.stderr )
  );

  let out_run  = String::from_utf8_lossy( &with_run.stdout );
  assert!(
    out_run.contains( "RUNNER OPTIONS:" ),
    "`clr run help` must print RUNNER OPTIONS. Got:\n{out_run}"
  );

  // Output must be identical to bare `clr help`.
  let bare_help = run_cli( &[ "help" ] );
  assert!(
    bare_help.status.success(),
    "`clr help` must exit 0"
  );
  assert_eq!(
    out_run.trim(), String::from_utf8_lossy( &bare_help.stdout ).trim(),
    "`clr run help` output must be identical to `clr help`"
  );
}

// ── BUG-302: prefix guard false-positive on short common words ───────────────

/// BUG-302: `clr is` must NOT exit 1 via the unknown-subcommand guard.
///
/// ## Root Cause
/// `guard_unknown_subcommand()` fired `"isolated".starts_with("is")` = true with
/// no minimum-length gate on `first`, causing any two-letter word whose chars
/// happen to start a subcommand name to trigger "Did you mean 'isolated'?".
///
/// ## Why Not Caught
/// All guard tests exercised genuine typos (truncations, insertions).  No test
/// verified that common short words with subcommand prefixes pass through.
///
/// ## Fix Applied
/// Added `first.len() >= 4` to the `sub.starts_with(first)` branch; removed
/// `first.starts_with(sub)` (morphological extensions are not typos; `is_close_typo`
/// covers all 1-char edits including short truncations like "kil").
///
/// ## Prevention
/// Every new subcommand name must be accompanied by false-positive tests for
/// any common English word that shares a ≥2-char prefix with the new name.
///
/// ## Pitfall
/// Removing `first.starts_with(sub)` does NOT break "pss" / "assk" detection:
/// those are caught by `is_close_typo` (edit distance 1).  Short truncations like
/// "kil" (len 3 < 4) are also caught by `is_close_typo`, not by `starts_with`.
// test_kind: bug_reproducer(BUG-302)
#[ test ]
fn bug_reproducer_302_prefix_guard_false_positive_is()
{
  // "is" shares a 2-char prefix with "isolated" — must NOT trigger the guard.
  // Fix: --dry-run avoids a real claude invocation; the guard fires before
  // dry-run processing so the assertion is unaffected.
  let out    = run_cli( &[ "--dry-run", "is", "it", "so?" ] );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "unknown subcommand" ),
    "BUG-302: `clr is it so?` must not emit 'unknown subcommand'. stderr:\n{stderr}"
  );
  assert!(
    out.status.code() != Some( 1 ) || !stderr.contains( "Did you mean" ),
    "BUG-302: `clr is it so?` must not exit 1 via guard. stderr:\n{stderr}"
  );
}

/// False-positive guard: short common words with subcommand prefixes must pass through.
#[ test ]
fn bug_reproducer_302_false_positive_prevention()
{
  // "is": prefix of "isolated".
  for word in &[ "is", "asked", "running", "he", "a" ]
  {
    let out    = run_cli( &[ word, "--dry-run" ] );
    let stderr = String::from_utf8_lossy( &out.stderr );
    assert!(
      !stderr.contains( "unknown subcommand" ),
      "BUG-302: `clr {word}` must not be rejected by the guard. stderr:\n{stderr}"
    );
  }
}

/// True-positive guard: "kil" is a genuine truncation typo caught by `is_close_typo`.
///
/// After the BUG-302 fix, `"kill".starts_with("kil")` no longer fires because
/// `"kil".len()` = 3 < 4 (below the new minimum-length threshold).  The guard
/// still rejects "kil" via `is_close_typo("kil", "kill")` = true (deletion, `abs_diff=1`).
#[ test ]
fn bug_302_regression_kil_still_caught_by_close_typo()
{
  let out    = run_cli( &[ "kil" ] );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    out.status.code() == Some( 1 ),
    "regression: `clr kil` must still exit 1 via is_close_typo after BUG-302 fix. stderr:\n{stderr}"
  );
  assert!(
    stderr.contains( "Did you mean" ),
    "regression: `clr kil` must still emit 'Did you mean'. stderr:\n{stderr}"
  );
}

/// True-positive guard (IN-8): "isolat" is a genuine prefix truncation caught by
/// `starts_with` with minimum-length threshold (`"isolat".len()` = 6 ≥ 4).
///
/// Confirms the BUG-302 fix does NOT break true-positive detection for subcommand
/// truncations that satisfy the minimum-length requirement.
#[ test ]
fn bug_302_regression_isolat_still_caught_by_prefix()
{
  let out    = run_cli( &[ "isolat" ] );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    out.status.code() == Some( 1 ),
    "regression (IN-8): `clr isolat` must still exit 1 via prefix guard after BUG-302 fix. stderr:\n{stderr}"
  );
  assert!(
    stderr.contains( "Did you mean" ) && stderr.contains( "isolated" ),
    "regression (IN-8): `clr isolat` must emit 'Did you mean ... isolated'. stderr:\n{stderr}"
  );
}
