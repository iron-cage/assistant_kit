//! Shared helpers for all `claude_version` integration tests.
//!
//! Provides subprocess execution helpers so every integration test can invoke
//! the compiled `claude_version` binary without boilerplate.
//!
//! # Binary Name Coupling
//!
//! The compile-time macro `env!("CARGO_BIN_EXE_claude_version")` inside
//! `run_clv_with_env` is tightly coupled to `[[bin]] name = "claude_version"`
//! in `Cargo.toml`.  If the binary is ever renamed, both must change atomically —
//! a partial rename compiles fine locally (cached artefact) but breaks on a clean build:
//!
//! 1. `Cargo.toml` — `[[bin]] name`
//! 2. `env!("CARGO_BIN_EXE_<name>")` in `run_clv_with_env`

fn assert_container()
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

/// Run `clv` with the given arguments and return the full output.
///
/// # Panics
///
/// Panics if the binary cannot be executed.
#[ inline ]
#[ must_use ]
pub fn run_clv( args : &[ &str ] ) -> std::process::Output
{
  run_clv_with_env( args, &[] )
}

/// Run `clv` with arguments and explicit environment overrides.
///
/// `env_overrides` is a list of `(key, value)` pairs appended to the
/// inherited environment.  Use `HOME` to isolate from the real `~/.claude/`.
///
/// # Panics
///
/// Panics if the binary cannot be executed.
#[ inline ]
#[ must_use ]
pub fn run_clv_with_env(
  args         : &[ &str ],
  env_overrides : &[ ( &str, &str ) ],
) -> std::process::Output
{
  assert_container();
  let bin = env!( "CARGO_BIN_EXE_claude_version" );
  let mut cmd = std::process::Command::new( bin );
  cmd.args( args );
  for ( key, val ) in env_overrides
  {
    cmd.env( key, val );
  }
  cmd.output().expect( "failed to execute claude_version binary" )
}

/// Create a minimal `~/.claude/settings.json` inside `home_dir`.
///
/// Writes `{ "key": "value", ... }` pairs passed as a slice.
/// Returns the path to the created file.
///
/// # Panics
///
/// Panics if the directory cannot be created or the file cannot be written.
#[ inline ]
pub fn write_settings(
  home_dir : &std::path::Path,
  pairs    : &[ ( &str, &str ) ],
)
{
  let dir = home_dir.join( ".claude" );
  std::fs::create_dir_all( &dir ).unwrap();
  let path = dir.join( "settings.json" );
  let entries : Vec< String > = pairs.iter().map( |( k, v ) |
    format!( "  \"{k}\": \"{v}\"" )
  ).collect();
  let json = if entries.is_empty()
  {
    "{}".to_string()
  }
  else
  {
    format!( "{{\n{}\n}}", entries.join( ",\n" ) )
  };
  std::fs::write( &path, json ).unwrap();
}

/// Path segments for the credential store directory, relative to `$HOME`.
///
/// Used by [`write_account`] to locate `_active` and individual credential files.
const CREDENTIAL_PATH : &[ &str ] = &[ ".persistent", "claude", "credential" ];

/// Write a credential file into `{home_dir}/.persistent/claude/credential/{name}.credentials.json`
/// and optionally write `_active` to mark it as active.
///
/// # Panics
///
/// Panics if the directory cannot be created or the files cannot be written.
#[ inline ]
pub fn write_account(
  home_dir    : &std::path::Path,
  name        : &str,
  make_active : bool,
)
{
  let credential_store = CREDENTIAL_PATH
    .iter()
    .fold( home_dir.to_path_buf(), | p, seg | p.join( seg ) );
  std::fs::create_dir_all( &credential_store ).unwrap();

  // Minimal credential stub — clv reads only `_active`; credential file content is not parsed.
  std::fs::write( credential_store.join( format!( "{name}.credentials.json" ) ), b"{}" ).unwrap();

  if make_active
  {
    std::fs::write( credential_store.join( "_active" ), name ).unwrap();
  }
}

/// Extract stdout from a process output as a `String`.
#[ inline ]
#[ must_use ]
pub fn stdout( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

/// Extract stderr from a process output as a `String`.
#[ inline ]
#[ must_use ]
pub fn stderr( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stderr ).into_owned()
}

/// Assert that the process exited with `expected` exit code.
///
/// On failure, prints both stdout and stderr to help diagnose the problem.
///
/// # Panics
///
/// Panics if the actual exit code does not match `expected`.
#[ inline ]
pub fn assert_exit( out : &std::process::Output, expected : i32 )
{
  let actual = out.status.code().unwrap_or( -1 );
  assert_eq!(
    actual, expected,
    "expected exit {expected}, got {actual}\nstdout: {}\nstderr: {}",
    stdout( out ),
    stderr( out ),
  );
}
