//! `.help` command and anywhere-in-argv help routing tests.
//!
//! ## TC-N: anywhere-in-argv help
//! | TC | Description | Kind |
//! |----|-------------|------|
//! | TC-001 | Empty argv → help output, exit 0 | P |
//! | TC-002 | `.help` → help output, exit 0 | P |
//! | TC-026 | `.help` subcommand explicitly → help output, exit 0 | P |
//! | TC-038 | `.help` in second position → exit 0, help output | N→P |
//! | TC-039 | `.help` after multi-part command → exit 0, help output | N→P |
//! | TC-040 | `.help` after params → exit 0, help output | N→P |
//! | TC-489 | bare `help` after command → routes to `.help`, exit 0 | N→P |
//! | TC-490 | bare `help` after params → routes to `.help`, exit 0 | N→P |
//!
//! ## EC-N: `10_help.md` spec edge cases
//! | Function | Spec | Description | Kind |
//! |----------|------|-------------|------|
//! | `ec3_help_mutation_no_side_effects` | `10_help` | `.help` + mutation → no side effects | P |
//! | `ec4_help_position_first_arg` | `10_help` | `.help` position independence | P |
//! | `ec5_absent_help_not_triggered` | `10_help` | absent `.help` → command executes normally | P |
//! | `ec6_help_output_contains_commands` | `10_help` | `.help` output has command names | P |
//! | `ec7_help_accepted_by_all_commands` | `10_help` | `.help` universally accepted | P |
//! | `ec8_help_wins_over_params` | `10_help` | `.help` wins over all params | P |

use crate::subprocess_helpers::{ assert_container, run, out_stdout, code };

// TC-001: empty argv → help output, exit 0
#[ test ]
fn tc001_empty_argv_shows_help()
{
  let out = run( &[] );
  assert_eq!( code( &out ), 0, "empty argv must exit 0" );
  assert!( out_stdout( &out ).contains( "Version Management" ), "must show help" );
}

// TC-002: .help → help, exit 0
#[ test ]
fn tc002_dot_help()
{
  let out = run( &[ ".help" ] );
  assert_eq!( code( &out ), 0 );
  assert!( out_stdout( &out ).contains( "Version Management" ), "must show help" );
}

// TC-026: .help subcommand explicitly
#[ test ]
fn tc026_help_subcommand_explicitly()
{
  let out = run( &[ ".help" ] );
  assert_eq!( code( &out ), 0 );
  assert!( out_stdout( &out ).contains( "Version Management" ), "must show help" );
}

// TC-038: .help in second position → exit 0, help output
//
// Root Cause
//
// `argv_to_unilang_tokens` only checked for `.help` as `argv[0]`.  Tokens in
// positions 1+ that lacked `::` were rejected as malformed params, so
// `.status .help` raised a parse error instead of showing help (FR-02 violation).
//
// Why Not Caught
//
// Only TC-002 tested `.help`, and only as the sole argument.  No test supplied
// `.help` after a command name.
//
// Fix Applied
//
// Added a pre-scan pass over all argv for the exact token `".help"`.  If found
// anywhere, routes to `".help"` immediately (satisfies FR-02).
//
// Prevention
//
// TC-038..TC-040 lock down `".help"` anywhere-in-argv behaviour.
//
// Pitfall
//
// Without the pre-scan, `.status .help` raises a parse error instead of
// showing help, breaking the discoverable help pattern that FR-02 requires.
#[ test ]
fn tc038_help_in_second_position()
{
  let out = run( &[ ".status", ".help" ] );
  assert_eq!( code( &out ), 0, "`.status .help` must exit 0" );
  let stdout = out_stdout( &out );
  assert!( stdout.contains( "Version Management" ), "must show help listing: {stdout}" );
}

// TC-039: .help after multi-part command → exit 0, help output
#[ test ]
fn tc039_help_after_multi_part_command()
{
  let out = run( &[ ".version.install", ".help" ] );
  assert_eq!( code( &out ), 0, "`.version.install .help` must exit 0" );
  let stdout = out_stdout( &out );
  assert!( stdout.contains( "Version Management" ), "must show help listing: {stdout}" );
}

// TC-040: .help after params → exit 0, help output
#[ test ]
fn tc040_help_after_params()
{
  let out = run( &[ ".version.guard", "dry::1", ".help" ] );
  assert_eq!( code( &out ), 0, "`.version.guard dry::1 .help` must exit 0" );
  let stdout = out_stdout( &out );
  assert!( stdout.contains( "Version Management" ), "must show help listing: {stdout}" );
}

// TC-489: bare `help` after command → routes to `.help`, exit 0
//
// Root Cause
//
// The adapter recognised `.help` anywhere in argv (Step 1b) but not bare `help`
// (without the leading dot). The unilang help footer instructs users
// "Use '<command> help' to get detailed help for a specific command." — so
// `clm .version.show help` is the documented invocation. Without the bare-`help`
// check, the adapter rejected `help` with "expected param::value syntax, got: 'help'"
// because `help` lacks the `::` separator required for key::value tokens.
//
// Why Not Caught
//
// All existing tests used `.help` (with dot). The help footer's example was
// never tested against the actual adapter behaviour; the mismatch went unnoticed.
//
// Fix Applied
//
// Step 1b of `argv_to_unilang_tokens` now checks for both `".help"` and `"help"`,
// routing either form to the global `.help` command.
//
// Prevention
//
// TC-489 and TC-490 lock the bare-`help` path. Any future regression in
// Step 1b will be caught immediately.
//
// Pitfall
//
// The help footer is generated by the `unilang` crate and cannot be patched here.
// The adapter must accept both spellings so the documented syntax actually works.
#[ test ]
fn tc489_bare_help_after_command_routes_to_help()
{
  let out = run( &[ ".version.show", "help" ] );
  assert_eq!( code( &out ), 0, "`.version.show help` must exit 0" );
  let stdout = out_stdout( &out );
  assert!( stdout.contains( "Version Management" ), "must show help listing: {stdout}" );
}

// TC-490: bare `help` after params → routes to `.help`, exit 0
#[ test ]
fn tc490_bare_help_after_params_routes_to_help()
{
  let out = run( &[ ".version.history", "count::3", "help" ] );
  assert_eq!( code( &out ), 0, "`.version.history count::3 help` must exit 0" );
  let stdout = out_stdout( &out );
  assert!( stdout.contains( "Version Management" ), "must show help listing: {stdout}" );
}

// ─── 10_help.md EC-3..EC-8 ───────────────────────────────────────────────────

// EC-3: `.help` combined with mutation command → no side effects (settings.json not created)
#[ test ]
fn ec3_help_mutation_no_side_effects()
{
  assert_container();
  let dir = tempfile::TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let bin = env!( "CARGO_BIN_EXE_claude_version" );
  let out = std::process::Command::new( bin )
    .args( [ ".settings.set", "key::theme", "value::dark", ".help" ] )
    .env( "HOME", home )
    .output()
    .expect( "failed to run" );
  assert_eq!( code( &out ), 0, ".help must exit 0 even with mutation command: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = out_stdout( &out );
  assert!( stdout.contains( "Version Management" ), "must show help: {stdout}" );
  // settings.json must NOT be created — mutation suppressed by .help
  assert!(
    !dir.path().join( ".claude/settings.json" ).exists(),
    "settings.json must not be created when .help is present"
  );
}

// EC-4: `.help` position independence — works as first arg
#[ test ]
fn ec4_help_position_first_arg()
{
  let out = run( &[ ".help", ".version.list" ] );
  assert_eq!( code( &out ), 0, "`.help .version.list` must exit 0" );
  let stdout = out_stdout( &out );
  assert!( stdout.contains( "Version Management" ), "must show help: {stdout}" );
}

// EC-5: absent `.help` → command executes normally, NOT help output
#[ test ]
fn ec5_absent_help_not_triggered()
{
  let out = run( &[ ".version.list" ] );
  assert_eq!( code( &out ), 0, ".version.list must exit 0 without .help" );
  let stdout = out_stdout( &out );
  assert!( stdout.contains( "stable" ), ".version.list must show aliases not help: {stdout}" );
  // help text must NOT appear (that would mean .help was incorrectly triggered)
  assert!(
    !stdout.contains( "Version Management" ),
    "help must NOT appear when .help is absent: {stdout}"
  );
}

// EC-6: `.help` output contains recognized command names or usage text
#[ test ]
fn ec6_help_output_contains_commands()
{
  let out = run( &[ ".help" ] );
  assert_eq!( code( &out ), 0 );
  let stdout = out_stdout( &out );
  // Must contain at least one recognized command name or usage keyword
  assert!(
    stdout.contains( ".status" ) || stdout.contains( ".version" ) || stdout.contains( "usage" ) || stdout.contains( "commands" ),
    "help output must contain command names or usage text: {stdout}"
  );
}

// EC-7: `.help` universally accepted by `.settings.show`, `.processes`, `.config`
#[ test ]
fn ec7_help_accepted_by_all_commands()
{
  for args in &[
    vec![ ".settings.show", ".help" ],
    vec![ ".processes", ".help" ],
    vec![ ".config", ".help" ],
  ]
  {
    let out = run( args );
    assert_eq!( code( &out ), 0, ".help must exit 0 for {args:?}" );
    let stdout = out_stdout( &out );
    assert!(
      stdout.contains( "Version Management" ),
      ".help must show help for {args:?}: {stdout}"
    );
  }
}

// EC-8: `.help` with other params — help wins, output is not a JSON array
#[ test ]
fn ec8_help_wins_over_params()
{
  let out = run( &[ ".version.list", "format::json", "v::0", ".help" ] );
  assert_eq!( code( &out ), 0, "`.version.list format::json v::0 .help` must exit 0" );
  let stdout = out_stdout( &out );
  // .help must override format::json — output must NOT be a JSON array
  assert!(
    !stdout.trim_start().starts_with( '[' ),
    "output must not be a JSON array when .help is present: {stdout}"
  );
  assert!( stdout.contains( "Version Management" ), "must show help: {stdout}" );
}
