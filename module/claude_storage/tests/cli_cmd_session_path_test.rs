//! Integration tests for the `clg .session.path` command.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/command/16_session_path.md`
//! - Param spec: `tests/docs/cli/param/41_latest.md`
//!
//! ## Coverage
//!
//! - SP-1: default selector = latest — resolves the storage's most recent session file
//! - SP-2: `latest::1` explicit — identical to the default selector
//! - SP-3: latest with empty storage → exit 2, "no sessions" on stderr
//! - SP-4: latest picks the newer of two sessions (mtime ordering)
//! - SP-5: `session::UUID` is pure computation — no existence check on file or storage
//! - SP-6: `topic::NAME` resolves via the fork-mode `UUIDv5` rule (NOT the `-{topic}` dir sense)
//! - SP-7: `session::` / `latest::` / `topic::` are mutually exclusive (any pair → exit 1)
//! - SP-8: selector validation — empty or slash-containing `topic::` / `session::` rejected
//! - SP-9: golden vector — `path::/tmp/x topic::a` ends in the published `UUIDv5` filename
//!
//! ## Topic Sense Collision (deliberate)
//!
//! Every other `claude_storage` command's `topic::` means the legacy dir-suffix
//! `{base}/-{topic}`. THIS command's `topic::` means the fork-mode `UUIDv5`
//! session file inside the BASE dir's storage. SP-6 pins the fork sense.

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

/// Command with isolated HOME and scrubbed `CLAUDE_HOME` (storage resolution
/// honors `CLAUDE_HOME` first — ambient leakage would silently retarget tests).
fn cmd_with_home( home : &std::path::Path ) -> std::process::Command
{
  let mut cmd = common::clg_cmd();
  cmd.env( "HOME", home );
  cmd.env_remove( "CLAUDE_HOME" );
  cmd
}

/// Expected session file path assembled from parts.
///
/// Deliberately NOT computed via `topic_session_file` / `to_storage_path_for`
/// in the test process: those read the TEST process env, while the command
/// under test reads the SUBPROCESS env (overridden HOME). Only the env-free
/// pieces (`encode_path`) come from the core crate.
fn expected_session_file(
  home : &std::path::Path,
  canonical_base : &std::path::Path,
  file_stem : &str,
) -> String
{
  let encoded = claude_storage_core::encode_path( canonical_base ).expect( "encode base" );
  format!(
    "{}/.claude/projects/{encoded}/{file_stem}.jsonl",
    home.display()
  )
}

// ─── SP-1 ────────────────────────────────────────────────────────────────────

/// SP-1: default selector = latest — resolves the most recent session file.
///
/// Passes the RAW tempdir path as `path::` while the fixture writes under the
/// CANONICALIZED path — proving the command canonicalizes before encoding.
#[ test ]
fn sp_1_default_selector_resolves_latest()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let canon   = project.path().canonicalize().unwrap();
  let root    = home.path().join( ".claude" );

  common::write_path_project_session( &root, &canon, "11111111-1111-1111-1111-111111111111", 2 );

  let out = cmd_with_home( home.path() )
    .arg( ".session.path" )
    .arg( format!( "path::{}", project.path().display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let expected = format!(
    "{}\n",
    expected_session_file( home.path(), &canon, "11111111-1111-1111-1111-111111111111" )
  );
  assert_eq!( stdout( &out ), expected, "default selector must print the latest session file" );
  assert!( stdout( &out ).starts_with( '/' ), "output must be absolute" );
}

// ─── SP-2 ────────────────────────────────────────────────────────────────────

/// SP-2: `latest::1` explicit — byte-identical to the default selector.
#[ test ]
fn sp_2_latest_explicit_matches_default()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let canon   = project.path().canonicalize().unwrap();
  let root    = home.path().join( ".claude" );

  common::write_path_project_session( &root, &canon, "22222222-2222-2222-2222-222222222222", 2 );

  let out_default = cmd_with_home( home.path() )
    .arg( ".session.path" )
    .arg( format!( "path::{}", canon.display() ) )
    .output()
    .unwrap();

  let out_latest = cmd_with_home( home.path() )
    .arg( ".session.path" )
    .arg( format!( "path::{}", canon.display() ) )
    .arg( "latest::1" )
    .output()
    .unwrap();

  assert_exit( &out_default, 0 );
  assert_exit( &out_latest, 0 );
  assert_eq!(
    stdout( &out_default ),
    stdout( &out_latest ),
    "latest::1 must be byte-identical to the default selector"
  );
}

// ─── SP-3 ────────────────────────────────────────────────────────────────────

/// SP-3: latest with empty storage → exit 2 with "no sessions" on stderr.
///
/// Exit 2 (not 1) distinguishes "nothing to resolve" from usage errors,
/// mirroring the `.status` exit-2 precedent.
#[ test ]
fn sp_3_empty_storage_exits_2()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();

  let out = cmd_with_home( home.path() )
    .arg( ".session.path" )
    .arg( format!( "path::{}", project.path().display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 2 );
  assert!(
    stderr( &out ).contains( "no sessions" ),
    "stderr must mention 'no sessions'; got: {}",
    stderr( &out )
  );
  assert!( stdout( &out ).trim().is_empty(), "stdout must be empty on exit 2" );
}

// ─── SP-4 ────────────────────────────────────────────────────────────────────

/// SP-4: latest picks the newer of two sessions (mtime ordering).
#[ test ]
fn sp_4_latest_picks_newer_session()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let canon   = project.path().canonicalize().unwrap();
  let root    = home.path().join( ".claude" );

  common::write_path_project_session( &root, &canon, "33333333-3333-3333-3333-333333333333", 2 );
  // mtime granularity guard: ensure the second write is strictly newer.
  std::thread::sleep( core::time::Duration::from_millis( 1100 ) );
  common::write_path_project_session( &root, &canon, "44444444-4444-4444-4444-444444444444", 2 );

  let out = cmd_with_home( home.path() )
    .arg( ".session.path" )
    .arg( format!( "path::{}", canon.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "44444444-4444-4444-4444-444444444444" ),
    "latest must pick the newer session; got: {}",
    stdout( &out )
  );
}

// ─── SP-5 ────────────────────────────────────────────────────────────────────

/// SP-5: `session::UUID` is pure computation — succeeds with no storage on disk.
#[ test ]
fn sp_5_session_selector_is_pure()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let canon   = project.path().canonicalize().unwrap();

  let out = cmd_with_home( home.path() )
    .arg( ".session.path" )
    .arg( format!( "path::{}", canon.display() ) )
    .arg( "session::55555555-5555-5555-5555-555555555555" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let expected = format!(
    "{}\n",
    expected_session_file( home.path(), &canon, "55555555-5555-5555-5555-555555555555" )
  );
  assert_eq!(
    stdout( &out ),
    expected,
    "session:: must join storage/<id>.jsonl without any existence check"
  );
}

// ─── SP-6 ────────────────────────────────────────────────────────────────────

/// SP-6: `topic::NAME` resolves via the fork-mode `UUIDv5` rule.
///
/// Expected file = base storage / UUIDv5(canonical base NUL topic).jsonl —
/// NOT the legacy `{base}/-{topic}` dir sense used by other commands.
/// This is also the automated half of the cross-binary parity check:
/// `clr topics --file NAME` pins its output to the same core-computed value.
#[ test ]
fn sp_6_topic_selector_uses_fork_rule()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let canon   = project.path().canonicalize().unwrap();

  let out = cmd_with_home( home.path() )
    .arg( ".session.path" )
    .arg( format!( "path::{}", canon.display() ) )
    .arg( "topic::alpha" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let uuid = claude_storage_core::topic_session_id( &canon, "alpha" ).unwrap();
  let expected = format!(
    "{}\n",
    expected_session_file( home.path(), &canon, uuid.as_str() )
  );
  assert_eq!(
    stdout( &out ),
    expected,
    "topic:: must resolve via UUIDv5 in the BASE storage (fork sense, not -{{topic}} dir sense)"
  );
  assert!(
    !stdout( &out ).contains( "/-alpha" ),
    "topic:: must NOT use the legacy -{{topic}} dir sense; got: {}",
    stdout( &out )
  );
}

// ─── SP-7 ────────────────────────────────────────────────────────────────────

/// SP-7: `session::` / `latest::` / `topic::` are mutually exclusive.
#[ test ]
fn sp_7_selectors_mutually_exclusive()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let base    = project.path().to_str().unwrap().to_owned();

  let pairs : &[ &[ &str ] ] = &
  [
    &[ "session::66666666-6666-6666-6666-666666666666", "latest::1" ],
    &[ "topic::alpha", "latest::1" ],
    &[ "session::66666666-6666-6666-6666-666666666666", "topic::alpha" ],
  ];

  for pair in pairs
  {
    let mut cmd = cmd_with_home( home.path() );
    cmd.arg( ".session.path" ).arg( format!( "path::{base}" ) );
    for arg in *pair
    {
      cmd.arg( arg );
    }
    let out = cmd.output().unwrap();
    assert_exit( &out, 1 );
    let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
    assert!(
      combined.contains( "mutually exclusive" ),
      "error for {pair:?} must mention mutual exclusion; got: {combined}"
    );
  }
}

// ─── SP-8 ────────────────────────────────────────────────────────────────────

/// SP-8: empty or slash-containing `topic::` / `session::` rejected with exit 1.
#[ test ]
fn sp_8_selector_validation()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let base    = project.path().to_str().unwrap().to_owned();

  for bad in [ "topic::", "topic::sub/dir", "session::", "session::a/b" ]
  {
    let out = cmd_with_home( home.path() )
      .arg( ".session.path" )
      .arg( format!( "path::{base}" ) )
      .arg( bad )
      .output()
      .unwrap();

    assert_exit( &out, 1 );
    let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
    assert!(
      !combined.is_empty(),
      "must produce error output for {bad}"
    );
  }
}

// ─── SP-9 ────────────────────────────────────────────────────────────────────

/// SP-9: golden vector — `path::/tmp/x topic::a` resolves to the published
/// `UUIDv5` filename `41299c24-a8f5-589f-9fce-8474fc855532.jsonl`.
///
/// Pins the cross-implementation contract (namespace + NUL name layout)
/// end-to-end through the CLI, matching the core golden-vector unit tests.
#[ test ]
fn sp_9_golden_vector_tmp_x_topic_a()
{
  let home = TempDir::new().unwrap();
  // The base must exist for physical canonicalization; /tmp is sanctioned
  // for test scratch. Creation is idempotent and the dir is left in place.
  std::fs::create_dir_all( "/tmp/x" ).unwrap();

  let out = cmd_with_home( home.path() )
    .arg( ".session.path" )
    .arg( "path::/tmp/x" )
    .arg( "topic::a" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.trim_end().ends_with( "/-tmp-x/41299c24-a8f5-589f-9fce-8474fc855532.jsonl" ),
    "golden vector mismatch; got: {s}"
  );
}
