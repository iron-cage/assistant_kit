//! Binary alias smoke tests: `clp` (short) and `claude_profile` (long).
//!
//! Verifies that both binary aliases build, run, and self-identify with their
//! own name in `--help` / `.` output. Required by the Binary Alias Smoke Tests
//! rule in `test_organization.rulebook.md`.
//!
//! ## Test Matrix
//!
//! | ID   | Test Function                          | Alias            | Condition                                  | P/N |
//! |------|----------------------------------------|------------------|--------------------------------------------|-----|
//! | a01  | `a01_clp_exits_0`                      | `clp`            | bare `.` invocation exits 0                | P   |
//! | a02  | `a02_clp_self_identifies`              | `clp`            | help stdout contains `"Usage: clp"`        | P   |
//! | a03  | `a03_claude_profile_exits_0`           | `claude_profile` | bare `.` invocation exits 0                | P   |
//! | a04  | `a04_claude_profile_self_identifies`   | `claude_profile` | help stdout contains `"Usage: claude_profile"` | P |

const CLP           : &str = env!( "CARGO_BIN_EXE_clp" );
const CLAUDE_PROFILE : &str = env!( "CARGO_BIN_EXE_claude_profile" );

fn assert_container()
{
  let in_container = std::path::Path::new( "/.dockerenv" ).exists()
    || std::path::Path::new( "/run/.containerenv" ).exists()
    || std::env::var( "RUNBOX_CONTAINER" ).as_deref() == Ok( "1" );
  let escaped = std::env::var( "VERB_LAYER" ).as_deref() == Ok( "l0" );
  assert!(
    in_container || escaped,
    "\n\nTests must run inside a container.\n\
     Standard invocation: cd module/claude_profile && ./verb/test\n\
     Host bypass:         VERB_LAYER=l0 cargo nextest run --all-features\n"
  );
}

fn run( bin : &str, args : &[ &str ] ) -> std::process::Output
{
  assert_container();
  std::process::Command::new( bin )
    .args( args )
    .output()
    .unwrap_or_else( |e| panic!( "failed to execute {bin}: {e}" ) )
}

// ── clp alias ─────────────────────────────────────────────────────────────────

#[ test ]
fn a01_clp_exits_0()
{
  let out = run( CLP, &[ "." ] );
  assert_eq!(
    out.status.code(),
    Some( 0 ),
    "clp . must exit 0; stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
}

#[ test ]
fn a02_clp_self_identifies()
{
  let out    = run( CLP, &[ "." ] );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "Usage: clp" ),
    "clp must show 'Usage: clp' in help; got: {stdout}",
  );
}

// ── claude_profile alias ──────────────────────────────────────────────────────

#[ test ]
fn a03_claude_profile_exits_0()
{
  let out = run( CLAUDE_PROFILE, &[ "." ] );
  assert_eq!(
    out.status.code(),
    Some( 0 ),
    "claude_profile . must exit 0; stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
}

#[ test ]
fn a04_claude_profile_self_identifies()
{
  let out    = run( CLAUDE_PROFILE, &[ "." ] );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "Usage: claude_profile" ),
    "claude_profile must show 'Usage: claude_profile' in help; got: {stdout}",
  );
}
