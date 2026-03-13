//! Framework smoke tests: verify help and dispatch work end-to-end.
//!
//! ## Test Matrix
//!
//! | TC | Description | Exit |
//! |----|-------------|------|
//! | TC-079 | `.help` exits 0 | 0 |
//! | TC-080 | `.help` lists 12 commands under Available commands | 0 |
//! | TC-082 | `.help` output includes Available commands section | 0 |
//! | TC-091 | Unknown command `.nonexistent` exits 1 | 1 |
//! | TC-092 | Unknown two-word command `.zzz.nope` exits 1 | 1 |
//! | TC-093 | Empty args → help, exits 0 | 0 |
//! | TC-094 | `.help` exits 0 and shows commands | 0 |
//! | TC-095 | All 11 non-help commands appear in help output | 0 |

use crate::helpers::{ run_clm, stdout, assert_exit };

const VISIBLE_COMMANDS : &[ &str ] = &[
  ".status",
  ".version.show",
  ".version.install",
  ".version.guard",
  ".version.list",
  ".version.history",
  ".processes",
  ".processes.kill",
  ".settings.show",
  ".settings.get",
  ".settings.set",
];

// TC-079: .help exits 0
#[ test ]
fn tc079_help_command_exits_0()
{
  let out = run_clm( &[ ".help" ] );
  assert_exit( &out, 0 );
}

// TC-080: .help lists 12 commands (11 operational + help itself)
//
// Unilang outputs "Available commands:" as the section header (not "COMMANDS:").
// Command lines are indented with leading whitespace followed by ".".
#[ test ]
fn tc080_help_lists_12_commands()
{
  let out = run_clm( &[ ".help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  // Count lines that are command entries: start with whitespace then ".".
  let cmd_lines = text
    .lines()
    .filter( | l | l.starts_with( ' ' ) && l.trim_start().starts_with( '.' ) )
    .count();

  assert_eq!(
    cmd_lines, 12,
    "help must list 12 commands (11 + help), found {cmd_lines}\nFull output:\n{text}"
  );
}

// TC-082: .help output includes "Available commands:" section
//
// Unilang does not have a USAGE section in global help — it shows
// "Available commands:" followed by an indented command listing.
#[ test ]
fn tc082_help_includes_available_commands_section()
{
  let out = run_clm( &[ ".help" ] );
  let text = stdout( &out );
  assert!( text.contains( "Available commands:" ), "help must include Available commands section: {text}" );
}

// TC-091: unknown command exits 1
#[ test ]
fn tc091_unknown_command_exits_1()
{
  let out = run_clm( &[ ".nonexistent" ] );
  assert_exit( &out, 1 );
}

// TC-092: unknown two-word command exits 1
#[ test ]
fn tc092_another_unknown_command_exits_1()
{
  let out = run_clm( &[ ".zzz.nope" ] );
  assert_exit( &out, 1 );
}

// TC-093: empty args → help, exit 0
#[ test ]
fn tc093_empty_args_exits_0()
{
  let out = run_clm( &[] );
  assert_exit( &out, 0 );
  assert!( stdout( &out ).contains( "Available commands:" ), "empty args must show help" );
}

// TC-094: .help exits 0 and output contains expected command names
#[ test ]
fn tc094_help_exits_0_and_shows_commands()
{
  let out = run_clm( &[ ".help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for cmd in &[ ".status", ".processes", ".settings.get" ]
  {
    assert!( text.contains( cmd ), ".help output must mention {cmd}" );
  }
}

// TC-095: all 11 non-help commands appear in help output
#[ test ]
fn tc095_all_visible_commands_in_help()
{
  let out = run_clm( &[ ".help" ] );
  let text = stdout( &out );
  for cmd in VISIBLE_COMMANDS
  {
    assert!( text.contains( cmd ), "help output must contain '{cmd}'\nFull output:\n{text}" );
  }
}
