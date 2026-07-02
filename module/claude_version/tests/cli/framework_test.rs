//! Framework smoke tests: verify help and dispatch work end-to-end.
//!
//! ## Test Matrix
//!
//! | TC | Description | Exit |
//! |----|-------------|------|
//! | TC-079 | `.help` exits 0 | 0 |
//! | TC-080 | `.help` lists all 12 commands by name | 0 |
//! | TC-082 | `.help` output includes cli_fmt section headers | 0 |
//! | TC-091 | Unknown command `.nonexistent` exits 1 | 1 |
//! | TC-092 | Unknown two-word command `.zzz.nope` exits 1 | 1 |
//! | TC-093 | Empty args → help, exits 0 | 0 |
//! | TC-094 | `.help` exits 0 and shows commands | 0 |
//! | TC-095 | All 12 visible commands appear in help output | 0 |
//! | IT-9 | `.help` contains all 4 section headers | 0 |

use crate::subprocess_helpers::{ run_clm, stdout, assert_exit };

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
  ".config",
];

// TC-079: .help exits 0
#[ test ]
fn tc079_help_command_exits_0()
{
  let out = run_clm( &[ ".help" ] );
  assert_exit( &out, 0 );
}

// TC-080: .help lists all 12 operational commands by name
#[ test ]
fn tc080_help_lists_12_commands()
{
  let out = run_clm( &[ ".help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  for cmd in VISIBLE_COMMANDS
  {
    assert!( text.contains( cmd ), "help must list command '{cmd}'\nFull output:\n{text}" );
  }
}

// TC-082: .help output includes cli_fmt section headers
#[ test ]
fn tc082_help_includes_available_commands_section()
{
  let out = run_clm( &[ ".help" ] );
  let text = stdout( &out );
  assert!( text.contains( "Version Management" ), "help must include section headers: {text}" );
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
  assert!( stdout( &out ).contains( "Version Management" ), "empty args must show help" );
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

// TC-095: all 12 visible commands appear in help output
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

// IT-9: help output contains all 4 grouped section headers
#[ test ]
fn it9_help_contains_grouped_section_headers()
{
  let out = run_clm( &[ ".help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for header in &[ "Version Management", "Settings & Config", "Process Lifecycle", "Status" ]
  {
    assert!( text.contains( header ), "help must contain section header '{header}'\nFull output:\n{text}" );
  }
}
