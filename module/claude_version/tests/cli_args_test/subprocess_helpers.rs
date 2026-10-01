//! Local subprocess helpers for `cli_args_test` integration tests.
//!
//! Provides binary runners and output extractors used across all
//! `cli_args_test` sub-modules.  Both runners check `crate::spawn_guard` first.

use crate::spawn_guard::{ assert_container, assert_no_history_mode };

/// Run `claude_version` with the given arguments and return the full output.
///
/// The subprocess inherits this process's `HOME`.  Under runbox that's the
/// developer's real `~/.claude`, mounted read-write, so use this only for
/// commands that write nothing under `HOME` — everything else goes through
/// [`run_in_home`] (BUG-581).
///
/// # Panics
///
/// Panics if the binary cannot be executed, or if `args` contains `mode::history`.
#[ inline ]
#[ must_use ]
pub fn run( args : &[ &str ] ) -> std::process::Output
{
  assert_container();
  assert_no_history_mode( args, "run_in_home" );
  let bin = env!( "CARGO_BIN_EXE_claude_version" );
  std::process::Command::new( bin )
    .args( args )
    .output()
    .expect( "failed to run clv" )
}

/// Run `claude_version` with `HOME` set to `home` and return the full output.
///
/// For every command that writes under `HOME`: settings, installs, and the
/// history-mode release cache.
///
/// # Panics
///
/// Panics if the binary cannot be executed.
#[ inline ]
#[ must_use ]
pub fn run_in_home( args : &[ &str ], home : &std::path::Path ) -> std::process::Output
{
  assert_container();
  let bin = env!( "CARGO_BIN_EXE_claude_version" );
  std::process::Command::new( bin )
    .args( args )
    .env( "HOME", home )
    .output()
    .expect( "failed to run clv" )
}

/// Extract stdout from a process output as a `String`.
#[ inline ]
#[ must_use ]
pub fn out_stdout( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

/// Extract stderr from a process output as a `String`.
#[ inline ]
#[ must_use ]
pub fn out_stderr( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stderr ).into_owned()
}

/// Extract the exit code from a process output (`-1` if unavailable).
#[ inline ]
#[ must_use ]
pub fn code( out : &std::process::Output ) -> i32
{
  out.status.code().unwrap_or( -1 )
}
