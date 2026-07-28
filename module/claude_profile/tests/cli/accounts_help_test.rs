//! Integration tests: H — Help commands.
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | h01 | `h01_dot_shows_help` | `.` → shows .accounts | P |
//! | h02 | `h02_help_lists_all_registered_commands` | `.help` → .accounts listed | P |
//! | h03 | `h03_help_hides_dot` | `.help` → bare `.` not listed | P |
//! | h04 | `h04_help_exits_0` | `.help` → exit 0 | P |
//! | h05 | `h05_no_args_shows_help` | no args → help shows .accounts | P |
//! | h06 | `h06_double_dash_help` | `--help` → exit 1 (POSIX flags not supported) | N |
//! | h07 | `h07_unknown_command_exits_1` | `.nonexistent` → exit 1 + stderr | N |
//! | h08 | `h08_grouped_help_shows_all_group_headers_in_order` | `.accounts.help` → 6 group headers, documented order | P |
//! | h09 | `h09_grouped_help_all_params_under_correct_group` | `.accounts.help` → all 30 params under their documented group | P |
//! | h10 | `h10_grouped_help_colons_align_globally` | `.accounts.help` → `::` at same offset across all 30 rows | P |
//! | h11 | `h11_grouped_help_booleans_are_bare` | `.accounts.help` → boolean params render `name::0`, never `0\|1` | P |
//! | h12 | `h12_grouped_help_enums_show_uppercase_placeholder` | `.accounts.help` → enum params show uppercase placeholder | P |
//! | h13 | `h13_grouped_help_no_version_banner` | `.accounts.help` → no version/build banner | N |
//! | h14 | `h14_grouped_help_no_removed_param_mentions` | `.accounts.help` → no assign::/for::/unclaim::/active:: | N |
//! | h15 | `h15_grouped_help_plain_text_group_header_fallback` | `.accounts.help` (piped) → plain `Core:`, no brackets/ANSI | P |
//! | h16 | `h16_grouped_help_removed_toggle_runtime_unaffected` | `.account.rotate`, `.accounts active::0` → still exit 1 | N |

use crate::cli_runner::{ run_cs, stdout, stderr, assert_exit };

// ── H: Help commands ──────────────────────────────────────────────────────────

#[ test ]
fn h01_dot_shows_help()
{
  let out  = run_cs( &[ "." ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( ".accounts" ), "help must list .accounts, got:\n{text}" );
}

#[ test ]
fn h02_help_lists_all_registered_commands()
{
  let out  = run_cs( &[ ".help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for cmd in &[
    ".accounts",
    ".account.save",
    ".account.use",
    ".account.delete",
    ".token.status",
    ".paths",
    ".usage",
    ".credentials.status",
    ".account.limits",
  ]
  {
    assert!( text.contains( cmd ), "help must list {cmd}, got:\n{text}" );
  }
  assert!( !text.contains( ".account.list" ),   "help must not list .account.list, got:\n{text}" );
  assert!( !text.contains( ".account.status" ), "help must not list .account.status, got:\n{text}" );
}

#[ test ]
fn h03_help_hides_dot()
{
  let out   = run_cs( &[ ".help" ] );
  let text  = stdout( &out );
  // `.` is registered with `hidden_from_list: true` — must not appear as a listed command.
  // `.help` IS visible (auto-registered by unilang) — that's expected.
  let lines : Vec< &str > = text.lines()
    .filter( | l | l.trim().starts_with( '.' ) )
    .collect();
  for line in &lines
  {
    let cmd = line.split_whitespace().next().unwrap_or( "" );
    assert!( cmd != ".", "listing should not include bare '.', got line: {line}" );
  }
}

#[ test ]
fn h04_help_exits_0()
{
  let out = run_cs( &[ ".help" ] );
  assert_exit( &out, 0 );
}

#[ test ]
fn h05_no_args_shows_help()
{
  let out  = run_cs( &[] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( ".accounts" ), "no-args help must list .accounts, got:\n{text}" );
}

#[ test ]
fn h06_double_dash_help()
{
  // POSIX flags (--help, -h) are not supported — use `.help` command instead.
  let out = run_cs( &[ "--help" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "unexpected flag" ), "--help must produce unexpected flag error, got:\n{err}" );
}

#[ test ]
fn h07_unknown_command_exits_1()
{
  let out = run_cs( &[ ".nonexistent" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( !err.is_empty(), "unknown command must produce stderr" );
}

// ── Grouped `.accounts.help` rendering (Task 413) ───────────────────────────────

/// The 6 documented presentation groups and their parameter membership, per
/// `docs/cli/command/001_account.md § Help Rendering Scheme`.
const GROUPS : &[ ( &str, &[ &str ] ) ] = &
[
  ( "Core",                         &[ "name", "format", "dry" ] ),
  ( "Account Ownership",            &[ "owner", "assignee", "force" ] ),
  ( "Sort Control",                 &[ "sort", "desc", "prefer" ] ),
  ( "Row Filtering & Pagination",   &[ "cols", "count", "offset", "only_active", "only_next", "only_valid", "exclude_exhausted", "min_5h", "min_7d" ] ),
  ( "Display Rendering",            &[ "abs", "no_color", "get" ] ),
  ( "Refresh & Subprocess Control", &[ "trace", "refresh", "touch", "imodel", "effort", "set_model", "live", "interval", "jitter" ] ),
];

/// Every boolean `.accounts` parameter — must render bare (`name::0`, never `name::0|1`).
const BOOLEAN_PARAMS : &[ &str ] = &
[
  "dry", "trace", "force", "refresh", "touch", "desc",
  "only_active", "only_next", "only_valid", "exclude_exhausted", "abs", "no_color", "live",
];

/// Every enum `.accounts` parameter and its documented uppercase placeholder.
const ENUM_PARAMS : &[ ( &str, &str ) ] = &
[
  ( "imodel", "MODEL" ), ( "effort", "EFFORT" ), ( "set_model", "MODEL" ),
  ( "format", "FORMAT" ), ( "sort", "SORT" ), ( "prefer", "PREFER" ),
];

/// Find a parameter's rendered line by its exact leading token — avoids substring
/// collisions such as `active::` appearing inside `only_active::`.
fn find_param_line< 'a >( text : &'a str, name : &str ) -> &'a str
{
  text.lines().find( | l | l.trim_start().starts_with( name ) )
  .unwrap_or_else( || panic!( "missing line for param `{name}`, got:\n{text}" ) )
}

#[ test ]
fn h08_grouped_help_shows_all_group_headers_in_order()
{
  let out  = run_cs( &[ ".accounts.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let mut positions : Vec< usize > = Vec::new();
  for entry in GROUPS
  {
    positions.push( text.find( entry.0 ).unwrap_or_else( || panic!( "missing group header `{}`, got:\n{text}", entry.0 ) ) );
  }

  let mut sorted = positions.clone();
  sorted.sort_unstable();
  assert_eq!( positions, sorted, "group headers must appear in documented order, got:\n{text}" );
}

#[ test ]
fn h09_grouped_help_all_params_under_correct_group()
{
  let out  = run_cs( &[ ".accounts.help" ] );
  let text = stdout( &out );

  let mut positions : Vec< ( &str, usize ) > = Vec::new();
  for entry in GROUPS
  {
    let pos = text.find( entry.0 ).unwrap_or_else( || panic!( "missing group header `{}`, got:\n{text}", entry.0 ) );
    positions.push( ( entry.0, pos ) );
  }
  positions.sort_by_key( | p | p.1 );

  for i in 0..positions.len()
  {
    let header = positions[ i ].0;
    let start  = positions[ i ].1;
    let end    = if i + 1 < positions.len() { positions[ i + 1 ].1 } else { text.len() };
    let slice  = &text[ start..end ];

    let mut params : &[ &str ] = &[];
    for entry in GROUPS
    {
      if entry.0 == header { params = entry.1; }
    }
    for name in params.iter().copied()
    {
      assert!(
        slice.lines().any( | l | l.trim_start().starts_with( name ) ),
        "`{name}` must appear under group `{header}`, got slice:\n{slice}"
      );
    }
  }
}

#[ test ]
fn h10_grouped_help_colons_align_globally()
{
  let out  = run_cs( &[ ".accounts.help" ] );
  let text = stdout( &out );

  let mut offsets : Vec< usize > = Vec::new();
  for entry in GROUPS
  {
    for name in entry.1.iter().copied()
    {
      let line = find_param_line( &text, name );
      offsets.push( line.find( "::" ).unwrap_or_else( || panic!( "line for `{name}` has no `::`, got:\n{line}" ) ) );
    }
  }

  assert_eq!( offsets.len(), 30, "must have checked exactly 30 parameter rows" );
  let first = offsets[ 0 ];
  assert!(
    offsets.iter().all( | &o | o == first ),
    "`::` must align at the same offset across all 30 rows, got offsets: {offsets:?}\nfull output:\n{text}"
  );
}

#[ test ]
fn h11_grouped_help_booleans_are_bare()
{
  let out  = run_cs( &[ ".accounts.help" ] );
  let text = stdout( &out );
  assert!( !text.contains( '|' ), "boolean params must not show 0|1 alternation, got:\n{text}" );
  for name in BOOLEAN_PARAMS.iter().copied()
  {
    let line = find_param_line( &text, name );
    assert!( line.contains( "::0" ), "`{name}` must render as bare `{name}::0`, got line:\n{line}" );
  }
}

#[ test ]
fn h12_grouped_help_enums_show_uppercase_placeholder()
{
  let out  = run_cs( &[ ".accounts.help" ] );
  let text = stdout( &out );
  for entry in ENUM_PARAMS
  {
    let line   = find_param_line( &text, entry.0 );
    let needle = format!( "::{}", entry.1 );
    assert!(
      line.contains( needle.as_str() ),
      "`{}` must show placeholder `{}`, got line:\n{line}", entry.0, entry.1
    );
  }
}

#[ test ]
fn h13_grouped_help_no_version_banner()
{
  let out  = run_cs( &[ ".accounts.help" ] );
  let text = stdout( &out );
  assert!( !text.to_lowercase().contains( "version" ), "must not show a version banner, got:\n{text}" );
}

#[ test ]
fn h14_grouped_help_no_removed_param_mentions()
{
  let out  = run_cs( &[ ".accounts.help" ] );
  let text = stdout( &out );
  for token in [ "assign::", "for::", "unclaim::", "active::" ]
  {
    assert!( !text.contains( token ), "must not mention REMOVED param `{token}`, got:\n{text}" );
  }
}

#[ test ]
fn h15_grouped_help_plain_text_group_header_fallback()
{
  // run_cs() captures stdout via a pipe — already non-TTY, matching `| cat` semantics.
  let out  = run_cs( &[ ".accounts.help" ] );
  let text = stdout( &out );
  assert!( text.contains( "Core:" ), "plain-text group header must show trailing colon, got:\n{text}" );
  assert!( !text.contains( '[' ) && !text.contains( ']' ), "group headers must not use bracket punctuation, got:\n{text}" );
  assert!( !text.contains( '\u{1b}' ), "non-TTY output must not contain ANSI escape codes, got:\n{text:?}" );
}

#[ test ]
fn h16_grouped_help_removed_toggle_runtime_unaffected()
{
  let out1 = run_cs( &[ ".account.rotate" ] );
  assert_exit( &out1, 1 );
  let out2 = run_cs( &[ ".accounts", "active::0" ] );
  assert_exit( &out2, 1 );
}

