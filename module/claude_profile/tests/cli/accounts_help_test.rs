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

