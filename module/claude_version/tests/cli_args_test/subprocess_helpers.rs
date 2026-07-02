//! Local subprocess helpers for `cli_args_test` integration tests.
//!
//! Provides container guard, binary runner, and output extractors used across
//! all `cli_args_test` sub-modules.

/// Assert that the current process is inside a container or has bypassed the guard.
///
/// # Panics
///
/// Panics if neither a container environment nor the `VERB_LAYER=l0` bypass is detected.
#[ inline ]
pub fn assert_container()
{
  let in_container = std::path::Path::new( "/.dockerenv" ).exists()
    || std::path::Path::new( "/run/.containerenv" ).exists()
    || std::env::var( "RUNBOX_CONTAINER" ).as_deref() == Ok( "1" );
  let escaped = std::env::var( "VERB_LAYER" ).as_deref() == Ok( "l0" );
  assert!(
    in_container || escaped,
    "\n\nTests must run inside a container.\n\
     Standard invocation: cd module/claude_version && ./verb/test\n\
     Host bypass:         VERB_LAYER=l0 cargo nextest run --all-features\n"
  );
}

/// Run `claude_version` with the given arguments and return the full output.
///
/// # Panics
///
/// Panics if the binary cannot be executed.
#[ inline ]
#[ must_use ]
pub fn run( args : &[ &str ] ) -> std::process::Output
{
  assert_container();
  let bin = env!( "CARGO_BIN_EXE_claude_version" );
  std::process::Command::new( bin )
    .args( args )
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
