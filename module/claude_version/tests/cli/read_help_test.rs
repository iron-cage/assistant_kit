//! Integration tests for `.help` — E1, and per-command help — IT-10..IT-12.
//!
//! | TC | Description | P/N | Exit |
//! |----|-------------|-----|------|
//! | TC-01 | `clv .` → help output, exit 0 | P | 0 |
//! | TC-02 | `clv` (empty argv) → help output, exit 0 | P | 0 |
//! | IT-10 | `clv .version.list.help` → exit 0, non-empty stdout, contains command name | P | 0 |
//! | IT-11 | `clv .version.list.help` → stdout does NOT contain global section headers | P | 0 |
//! | IT-12 | `clv .version.list.help` → stdout contains arg name and `(default:` | P | 0 |

use crate::subprocess_helpers::{ assert_exit, run_clv, stdout };

// ─── E1: help ────────────────────────────────────────────────────────────────

// TC-01
#[ test ]
fn tc01_dot_alias_shows_help()
{
  let out = run_clv( &[ "." ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( ".status" ), "expected help listing, got: {text}" );
}

// TC-02
#[ test ]
fn tc02_empty_argv_shows_help()
{
  let out = run_clv( &[] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( ".status" ), "expected help listing, got: {text}" );
}

// ─── per-command help ────────────────────────────────────────────────────────

// IT-10: `clv .version.list.help` exits 0, stdout non-empty, contains the command name.
#[ test ]
fn it10_command_help_exits_0_contains_command_name()
{
  let out = run_clv( &[ ".version.list.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.is_empty(), ".version.list.help must produce non-empty stdout" );
  assert!( text.contains( ".version.list" ), "stdout must contain the command name; got: {text}" );
}

// IT-11: `clv .version.list.help` stdout does NOT contain global section headers.
#[ test ]
fn it11_command_help_omits_global_headers()
{
  let out = run_clv( &[ ".version.list.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for header in &[ "Version Management", "Settings & Config", "Process Lifecycle" ]
  {
    assert!(
      !text.contains( header ),
      ".version.list.help must not show global section '{header}'; got: {text}"
    );
  }
}

// IT-12: `clv .version.list.help` stdout contains an arg name and `(default:`.
#[ test ]
fn it12_command_help_shows_arg_with_default()
{
  let out = run_clv( &[ ".version.list.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(default:" ),
    "stdout must contain '(default:' for at least one arg; got: {text}"
  );
}
