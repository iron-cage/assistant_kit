//! Unix-only integration tests.
#![ cfg( unix ) ]
//! Credential default fallback tests.
//!
//! Verifies the 3-tier `--creds` resolution for `isolated` and `refresh`:
//!
//! - Tier 1: `--creds` CLI flag (always wins)
//! - Tier 2: `CLR_CREDS` env var (wins when `--creds` absent)
//! - Tier 3: `$HOME/.claude/.credentials.json` default (this plan's scope)
//!
//! ## Test Matrix
//!
//! | # | Scenario | Command | Expected |
//! |---|----------|---------|----------|
//! | T1 | No --creds, CLR_CREDS unset, HOME set, file present | `isolated --trace` | trace shows HOME default path |
//! | T2 | No --creds, CLR_CREDS unset, HOME set, file present | `refresh --trace` | trace shows HOME default path |
//! | T3 | CLR_CREDS set, HOME default also present | `isolated --trace` | trace shows CLR_CREDS (tier 2 wins) |
//! | T4 | HOME unset, no CLR_CREDS, no --creds | `isolated` | exit 1; stderr references HOME |
//! | T5 | HOME set, file absent, no CLR_CREDS, no --creds | `isolated --trace` | exit 1; stderr contains expected path |

use std::process::Command;

/// Create `{home}/.claude/.credentials.json` with empty JSON placeholder.
fn write_placeholder_creds( home : &std::path::Path )
{
  let dir = home.join( ".claude" );
  std::fs::create_dir_all( &dir ).expect( "create .claude dir" );
  std::fs::write( dir.join( ".credentials.json" ), "{}" )
    .expect( "write placeholder creds" );
}

// T1: isolated — no --creds, CLR_CREDS unset, HOME set, default file present.
// Trace must confirm $HOME/.claude/.credentials.json as the resolved credentials path.
// Fails before 3rd tier: guard fires with "missing required argument: --creds" (no trace output).
#[ test ]
fn t1_isolated_no_creds_uses_home_default()
{
  let tmp = tempfile::tempdir().expect( "create tmp home" );
  write_placeholder_creds( tmp.path() );
  let expected = tmp.path().join( ".claude" ).join( ".credentials.json" );

  let out = Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "isolated", "--trace", "test" ] )
    .env( "HOME", tmp.path() )
    .env_remove( "CLR_CREDS" )
    .env( "PATH", "/nonexistent" )
    .output()
    .expect( "invoke clr isolated" );

  let stderr = String::from_utf8_lossy( &out.stderr );
  let expected_str = expected.to_str().unwrap();
  assert!(
    stderr.contains( "# creds:" ),
    "trace must emit '# creds:' line; got stderr:\n{stderr}"
  );
  assert!(
    stderr.contains( expected_str ),
    "trace must contain default path '{expected_str}'; got stderr:\n{stderr}"
  );
}

// T2: refresh — no --creds, CLR_CREDS unset, HOME set, default file present.
// Trace must confirm $HOME/.claude/.credentials.json as the resolved credentials path.
// Fails before 3rd tier: guard fires with "missing required argument: --creds" (no trace output).
#[ test ]
fn t2_refresh_no_creds_uses_home_default()
{
  let tmp = tempfile::tempdir().expect( "create tmp home" );
  write_placeholder_creds( tmp.path() );
  let expected = tmp.path().join( ".claude" ).join( ".credentials.json" );

  let out = Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "refresh", "--trace" ] )
    .env( "HOME", tmp.path() )
    .env_remove( "CLR_CREDS" )
    .env( "PATH", "/nonexistent" )
    .output()
    .expect( "invoke clr refresh" );

  let stderr = String::from_utf8_lossy( &out.stderr );
  let expected_str = expected.to_str().unwrap();
  assert!(
    stderr.contains( "# creds:" ),
    "trace must emit '# creds:' line; got stderr:\n{stderr}"
  );
  assert!(
    stderr.contains( expected_str ),
    "trace must contain default path '{expected_str}'; got stderr:\n{stderr}"
  );
}

// T3: CLR_CREDS wins over HOME default (regression guard: tier 2 > tier 3).
// Passes even before 3rd tier — CLR_CREDS is already the 2nd tier.
// Ensures the 3rd tier never overrides CLR_CREDS after implementation.
#[ test ]
fn t3_clr_creds_wins_over_home_default()
{
  let tmp_home = tempfile::tempdir().expect( "create tmp home" );
  write_placeholder_creds( tmp_home.path() );
  let home_default = tmp_home.path().join( ".claude" ).join( ".credentials.json" );

  let tmp_creds = tempfile::NamedTempFile::new().expect( "create tmp creds" );
  std::fs::write( tmp_creds.path(), "{}" ).expect( "write creds" );
  let clr_path = tmp_creds.path().to_str().unwrap().to_string();

  let out = Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "isolated", "--trace", "test" ] )
    .env( "HOME", tmp_home.path() )
    .env( "CLR_CREDS", &clr_path )
    .env( "PATH", "/nonexistent" )
    .output()
    .expect( "invoke clr isolated" );

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( &clr_path ),
    "CLR_CREDS path must appear in trace; got stderr:\n{stderr}"
  );
  assert!(
    !stderr.contains( home_default.to_str().unwrap() ),
    "HOME default must NOT appear when CLR_CREDS is set; got stderr:\n{stderr}"
  );
}

// T4: HOME unset, no CLR_CREDS, no --creds → exit 1, error references HOME.
// Fails before 3rd tier: exits with "missing required argument: --creds" (no HOME mention).
// After 3rd tier: ClaudePaths::new() returns None → guard fires with HOME-referencing error.
#[ test ]
fn t4_home_unset_exits_with_error()
{
  let out = Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "isolated", "test" ] )
    .env_remove( "HOME" )
    .env_remove( "CLR_CREDS" )
    .env( "PATH", "/nonexistent" )
    .output()
    .expect( "invoke clr isolated" );

  assert!( !out.status.success(), "must exit non-zero when HOME unset and no --creds" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "HOME" ) || stderr.contains( "cannot resolve" ),
    "error must reference HOME or resolution failure; got stderr:\n{stderr}"
  );
}

// T5: HOME set but $HOME/.claude/.credentials.json absent, no CLR_CREDS → exit 1, path in error.
// Fails before 3rd tier: exits "missing required argument: --creds" (path not mentioned).
// After 3rd tier: trace fires showing resolved path; read_to_string fails naming the same path.
#[ test ]
fn t5_default_file_missing_exits_with_error()
{
  let tmp = tempfile::tempdir().expect( "create tmp home" );
  // Deliberately NOT creating .claude/.credentials.json
  let expected = tmp.path().join( ".claude" ).join( ".credentials.json" );

  let out = Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "isolated", "--trace", "test" ] )
    .env( "HOME", tmp.path() )
    .env_remove( "CLR_CREDS" )
    .env( "PATH", "/nonexistent" )
    .output()
    .expect( "invoke clr isolated" );

  assert!( !out.status.success(), "must exit non-zero when default creds file missing" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  let expected_str = expected.to_str().unwrap();
  assert!(
    stderr.contains( expected_str ),
    "stderr must contain the resolved default path '{expected_str}'; got:\n{stderr}"
  );
}
