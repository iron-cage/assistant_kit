//! Invariant tests for session source isolation.
//!
//! Covers IN-1–IN-5 from `tests/docs/invariant/011_session_source_isolation.md`.
//!
//! These tests verify that when `--session-from` is used, the session UUID is
//! read from the source directory's storage (not the target's), the subprocess
//! working directory is the target (not the source), and the source session
//! files are never written to during the cross-loaded run.
//!
//! All tests use `--dry-run` so no real Claude binary is needed.
//!
//! | Test | Property |
//! |------|----------|
//! | IN-1 | Session UUID is read from source dir, not target |
//! | IN-2 | Subprocess working directory is target dir, not source |
//! | IN-3 | Source session file mtime and size unchanged after cross-loaded run |
//! | IN-4 | `--session-dir` raw path wins over `--session-from` computed path |
//! | IN-5 | `--session-from` + `--to`: session UUID from source, cwd is target |

// IN-3 is the most critical: if source files are written to during a cross-loaded
// run, the isolation contract is broken and source history would be polluted.
// Since --dry-run is used, no subprocess fires — but clr itself must not touch
// the source files during command building.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::run_cli;

// ── Shared helpers ─────────────────────────────────────────────────────────────

/// Container guard.
fn container_check()
{
  let in_container = std::path::Path::new( "/.dockerenv" ).exists()
    || std::path::Path::new( "/run/.containerenv" ).exists()
    || std::env::var( "RUNBOX_CONTAINER" ).as_deref() == Ok( "1" );
  let escaped = std::env::var( "VERB_LAYER" ).as_deref() == Ok( "l0" );
  assert!(
    in_container || escaped,
    "\n\nTests must run inside a container.\n\
     Host bypass: VERB_LAYER=l0 cargo nextest run --all-features\n"
  );
}

/// `Df()` path encoder (see `algorithm/001_path_encoding.md`).
fn df( path : &str ) -> String
{
  let stripped    = path.trim_start_matches( '/' );
  let components : Vec<String> = stripped.split( '/' )
    .map( | c | c.replace( '_', "-" ) )
    .collect();
  let mut result = String::from( "-" );
  if let Some( ( first, rest ) ) = components.split_first()
  {
    result.push_str( first );
    for comp in rest
    {
      if let Some( body ) = comp.strip_prefix( '-' )
      {
        result.push_str( "--" );
        result.push_str( body );
      }
      else
      {
        result.push( '-' );
        result.push_str( comp );
      }
    }
  }
  result
}

/// Create `<claude_home>/projects/<df(src)>/<uuid>.jsonl`; return the `.jsonl` path.
fn make_session_for( claude_home : &std::path::Path, src : &str, uuid : &str ) -> std::path::PathBuf
{
  let session_dir = claude_home.join( "projects" ).join( df( src ) );
  std::fs::create_dir_all( &session_dir ).expect( "create session dir" );
  let file = session_dir.join( format!( "{uuid}.jsonl" ) );
  std::fs::write( &file, b"{}" ).expect( "write session jsonl" );
  file
}

/// Run `clr --dry-run <args>` with extra env; return stdout on exit 0.
fn run_dry_env( args : &[ &str ], env : &[ ( &str, &str ) ] ) -> String
{
  container_check();
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut full = vec![ "--dry-run" ];
  full.extend_from_slice( args );
  let out = std::process::Command::new( bin )
    .args( &full )
    .envs( env.iter().copied() )
    .env_remove( "CLR_DIR" )
    .env_remove( "CLR_SESSION_DIR" )
    .env_remove( "CLR_SESSION_FROM" )
    .output()
    .expect( "failed to invoke clr binary" );
  assert!(
    out.status.success(),
    "dry-run failed (exit {})\nstdout: {}\nstderr: {}",
    out.status.code().unwrap_or( -1 ),
    String::from_utf8_lossy( &out.stdout ),
    String::from_utf8_lossy( &out.stderr ),
  );
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

// ── IN-1: Session UUID from source dir, not target ────────────────────────────

/// IN-1: UUID is read from source dir's `CLAUDE_SESSION_DIR`, not target's.
///
/// Target dir has no `.jsonl` files; source dir has `lll-001.jsonl`.
/// The dry-run must inject `-c lll-001` (from source), not any UUID from target.
#[ test ]
fn in1_uuid_read_from_source_not_target()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/011it1-src";
  make_session_for( ch.path(), src, "lll-001" );
  // Target dir exists but has NO session files in Claude storage
  let tgt = tempfile::TempDir::new().expect( "tgt tmpdir" );
  let tgt_str = tgt.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--session-from", src, "--to", tgt_str, "task" ],
    &[ ( "CLAUDE_HOME", ch.path().to_str().expect( "utf-8" ) ) ],
  );
  assert!(
    stdout.contains( "lll-001" ),
    "UUID must come from source. Got:\n{stdout}"
  );
}

// ── IN-2: Subprocess working directory is target, not source ──────────────────

/// IN-2: Subprocess working directory is target dir, not source dir.
///
/// `--to <tgt>` sets the working dir; `--session-from <src>` does not.
#[ test ]
fn in2_subprocess_dir_is_target_not_source()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/011it2-src";
  make_session_for( ch.path(), src, "mmm-002" );
  let tgt = tempfile::TempDir::new().expect( "tgt tmpdir" );
  let tgt_str = tgt.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--session-from", src, "--to", tgt_str, "task" ],
    &[ ( "CLAUDE_HOME", ch.path().to_str().expect( "utf-8" ) ) ],
  );
  // Target must appear as the working dir (`cd <tgt>` prefix)
  assert!(
    stdout.contains( &format!( "cd {tgt_str}" ) ),
    "subprocess dir must be target `{tgt_str}`. Got:\n{stdout}"
  );
  // Source dir must NOT appear as the working dir
  assert!(
    !stdout.contains( &format!( "cd {src}" ) ),
    "subprocess dir must NOT be source `{src}`. Got:\n{stdout}"
  );
}

// ── IN-3: Source session file mtime and size unchanged ────────────────────────

/// IN-3: Source session `.jsonl` mtime and size are unchanged after a cross-loaded run.
///
/// `clr` reads the UUID from the source session dir during `--dry-run` command
/// building but must NEVER write to it.  This is the write-isolation invariant.
#[ test ]
fn in3_source_session_file_unchanged()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/011it3-src";
  let jsonl = make_session_for( ch.path(), src, "nnn-003" );
  let tgt = tempfile::TempDir::new().expect( "tgt tmpdir" );
  let tgt_str = tgt.path().to_str().expect( "utf-8" );

  let before = std::fs::metadata( &jsonl ).expect( "stat before" );
  let mtime_before = before.modified().expect( "mtime before" );
  let size_before  = before.len();

  run_dry_env(
    &[ "--session-from", src, "--to", tgt_str, "Continue" ],
    &[ ( "CLAUDE_HOME", ch.path().to_str().expect( "utf-8" ) ) ],
  );

  let after = std::fs::metadata( &jsonl ).expect( "stat after" );
  let mtime_after = after.modified().expect( "mtime after" );
  let size_after  = after.len();

  assert_eq!(
    mtime_before, mtime_after,
    "source session `.jsonl` mtime must be unchanged (write isolation invariant)"
  );
  assert_eq!(
    size_before, size_after,
    "source session `.jsonl` size must be unchanged (write isolation invariant)"
  );
}

// ── IN-4: --session-dir raw path wins over --session-from ─────────────────────

/// IN-4: `--session-dir` raw path wins over `--session-from` computed path.
///
/// Source: `ooo-004.jsonl`; `--session-dir` override: `ppp-005.jsonl`.
/// Output must use `ppp-005` (from `--session-dir`); `ooo-004` must not appear.
#[ test ]
fn in4_session_dir_raw_path_wins()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/011it4-src";
  make_session_for( ch.path(), src, "ooo-004" );
  // Raw session dir override
  let raw_dir = tempfile::TempDir::new().expect( "raw tmpdir" );
  std::fs::write( raw_dir.path().join( "ppp-005.jsonl" ), b"{}" )
    .expect( "write raw session" );
  let raw_str = raw_dir.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--session-from", src, "--session-dir", raw_str, "test" ],
    &[ ( "CLAUDE_HOME", ch.path().to_str().expect( "utf-8" ) ) ],
  );
  assert!(
    stdout.contains( "ppp-005" ),
    "`--session-dir` UUID must win (`ppp-005`). Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "ooo-004" ),
    "`ooo-004` from `--session-from` must NOT appear. Got:\n{stdout}"
  );
}

// ── IN-5: --session-from + --to: session UUID from source, cwd is target ───────

/// IN-5: Both read isolation (UUID from source) and run isolation (cwd = target)
///       hold simultaneously when `--to` and `--session-from` are combined.
#[ test ]
fn in5_combined_source_uuid_and_target_cwd()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/011it5-src";
  make_session_for( ch.path(), src, "qqq-006" );
  let tgt = tempfile::TempDir::new().expect( "tgt tmpdir" );
  let tgt_str = tgt.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--to", tgt_str, "--session-from", src, "Continue" ],
    &[ ( "CLAUDE_HOME", ch.path().to_str().expect( "utf-8" ) ) ],
  );
  // Read isolation: UUID comes from source
  assert!(
    stdout.contains( "qqq-006" ),
    "UUID must come from source (`qqq-006`). Got:\n{stdout}"
  );
  // Run isolation: subprocess runs in target
  assert!(
    stdout.contains( &format!( "cd {tgt_str}" ) ),
    "subprocess dir must be target `{tgt_str}`. Got:\n{stdout}"
  );
}

// ── Sanity guard ───────────────────────────────────────────────────────────────

/// Verify `clr` binary is reachable via the test infrastructure.
#[ test ]
fn sanity_clr_binary_reachable()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr binary must be reachable" );
}
