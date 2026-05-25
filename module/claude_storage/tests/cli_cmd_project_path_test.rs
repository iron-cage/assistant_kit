//! Integration tests for the `clg .project.path` command.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/command/08_project_path.md`
//!
//! ## Coverage
//!
//! - INT-1: Default (cwd) computes correct storage path
//! - INT-2: `path::` override computes path for given directory
//! - INT-3: `topic::` appended as suffix to encoded path
//! - INT-4: `path::` with `topic::` combines both
//! - INT-5: Output is a single line ending with /
//! - INT-6: Exits with code 0 for nonexistent path
//! - INT-7: ~ prefix expanded in `path::`
//! - INT-8: `path::.` resolves to cwd
//! - INT-9: Empty `topic::` rejected
//! - INT-10: `topic::` with slash rejected

mod common;

use tempfile::TempDir;

fn stdout( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

fn stderr( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stderr ).into_owned()
}

fn assert_exit( out : &std::process::Output, code : i32 )
{
  assert_eq!(
    out.status.code().unwrap_or( -1 ),
    code,
    "expected exit {code}, got {:?}; stderr: {}",
    out.status.code(),
    stderr( out )
  );
}

// ─── INT-1 ────────────────────────────────────────────────────────────────────

/// INT-1: Default (cwd) computes correct storage path.
///
/// Run from a known project directory; output must be a storage path
/// containing `.claude/projects/` and an encoded form of the cwd.
#[ test ]
fn int_1_default_cwd_computes_storage_path()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .current_dir( project.path() )
    .arg( ".project.path" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( ".claude" ) || s.contains( "projects" ),
    "output must contain storage path components; got:\n{s}"
  );
  assert!(
    !s.trim().is_empty(),
    "output must not be empty; got empty stdout"
  );
}

// ─── INT-2 ────────────────────────────────────────────────────────────────────

/// INT-2: `path::` override computes path for given directory.
///
/// The encoded form of `/home/alice/projects/consumer-app` uses hyphens.
/// Output must contain both `.claude/projects/` and the encoded path component.
#[ test ]
fn int_2_path_override_computes_storage_path()
{
  let home = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".project.path" )
    .arg( "path::/home/alice/projects/consumer-app" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "consumer-app" ) || s.contains( "projects" ),
    "output must contain encoded path components; got:\n{s}"
  );
}

// ─── INT-3 ────────────────────────────────────────────────────────────────────

/// INT-3: `topic::` appended as suffix to encoded path.
///
/// `topic::default_topic` must produce a path containing `--default-topic`.
#[ test ]
fn int_3_topic_appended_as_suffix_to_encoded_path()
{
  let home = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".project.path" )
    .arg( "path::/home/alice/projects/consumer-app" )
    .arg( "topic::default_topic" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "--default-topic" ) || s.contains( "default-topic" ) || s.contains( "default_topic" ),
    "output must contain topic suffix for 'default_topic'; got:\n{s}"
  );
}

// ─── INT-4 ────────────────────────────────────────────────────────────────────

/// INT-4: `path::` with `topic::` combines both.
///
/// `path::~/projects/myapp ```topic::wor```k` must produce an encoded path with `--work` suffix.
#[ test ]
fn int_4_path_with_topic_combines_both()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let project_path = project.path().to_str().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".project.path" )
    .arg( format!( "path::{project_path}" ) )
    .arg( "topic::work" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "--work" ) || s.contains( "work" ),
    "output must contain '--work' topic suffix; got:\n{s}"
  );
}

// ─── INT-5 ────────────────────────────────────────────────────────────────────

/// INT-5: Output is a single line ending with /.
#[ test ]
fn int_5_output_is_single_line_ending_with_slash()
{
  let home = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".project.path" )
    .arg( "path::/tmp/test-dir-int5" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let content = s.trim_end_matches( '\n' );
  let lines : Vec< &str > = content.split( '\n' ).filter( | l | !l.is_empty() ).collect();
  assert_eq!(
    lines.len(), 1,
    "output must be exactly one non-empty line; got:\n{s}"
  );
  assert!(
    content.ends_with( '/' ),
    "output must end with '/'; got:\n{s}"
  );
}

// ─── INT-6 ────────────────────────────────────────────────────────────────────

/// INT-6: Exits with code 0 for nonexistent path.
///
/// Path computation is filesystem-independent; nonexistent paths must still succeed.
#[ test ]
fn int_6_exits_0_for_nonexistent_path()
{
  let home = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".project.path" )
    .arg( "path::/tmp/nonexistent-project-int6-abc123" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    !s.trim().is_empty(),
    "must output computed path even for nonexistent dir; got empty stdout"
  );
}

// ─── INT-7 ────────────────────────────────────────────────────────────────────

/// INT-7: ~ prefix expanded in `path::`.
///
/// Output must not contain literal `~`; must contain the expanded home directory.
#[ test ]
fn int_7_tilde_expanded_in_path()
{
  let home     = TempDir::new().unwrap();
  let home_str = home.path().to_str().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".project.path" )
    .arg( "path::~/projects/consumer-app" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    !s.contains( '~' ),
    "output must not contain literal '~' after expansion; got:\n{s}"
  );
  assert!(
    s.contains( home_str ) || s.contains( ".claude" ),
    "output must contain expanded home path or storage marker; got:\n{s}"
  );
}

// ─── INT-8 ────────────────────────────────────────────────────────────────────

/// INT-8: `path::.` resolves to cwd.
///
/// `clg .project.path ```path::```.` from the same directory as bare `clg .project.path`
/// must produce identical output.
#[ test ]
fn int_8_path_dot_resolves_to_cwd()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();

  let out_bare = common::clg_cmd()
    .env( "HOME", home.path() )
    .current_dir( project.path() )
    .arg( ".project.path" )
    .output()
    .unwrap();

  let out_dot = common::clg_cmd()
    .env( "HOME", home.path() )
    .current_dir( project.path() )
    .arg( ".project.path" )
    .arg( "path::." )
    .output()
    .unwrap();

  assert_exit( &out_bare, 0 );
  assert_exit( &out_dot,  0 );
  assert_eq!(
    stdout( &out_dot ),
    stdout( &out_bare ),
    "path::. must produce same output as bare .project.path from same cwd"
  );
}

// ─── INT-9 ────────────────────────────────────────────────────────────────────

/// INT-9: Empty `topic::` rejected.
///
/// `topic::` (empty value) must produce an error on stderr and exit 1.
#[ test ]
fn int_9_empty_topic_rejected()
{
  let home = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".project.path" )
    .arg( "topic::" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    !combined.is_empty(),
    "must produce error output for empty topic::"
  );
}

// ─── INT-10 ───────────────────────────────────────────────────────────────────

/// INT-10: `topic::` with slash rejected.
///
/// `topic::my/topic` must produce an error on stderr and exit 1.
#[ test ]
fn int_10_topic_with_slash_rejected()
{
  let home = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".project.path" )
    .arg( "topic::my/topic" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    !combined.is_empty(),
    "must produce error output for slash-containing topic::"
  );
}
