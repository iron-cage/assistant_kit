//! Invariant tests for session source isolation.
//!
//! Covers IN-1–IN-5 from `tests/docs/invariant/011_session_source_isolation.md`.
//!
//! These tests verify that when `--from` is used, the transplant plan's
//! source is the source directory's computed storage (never a target-derived
//! path), the subprocess working directory is the target (not the source), and
//! the source session files are never written to during the cross-loaded run.
//!
//! All tests use `--dry-run` so no real Claude binary is needed.
//!
//! | Test | Property |
//! |------|----------|
//! | IN-1 | Transplant source comes from source storage, never from target |
//! | IN-2 | Subprocess working directory is target dir, not source |
//! | IN-3 | Source session file mtime and size unchanged after cross-loaded run |
//! | IN-4 | `--session-dir` no longer suppresses `--from` (deprecated, inert) |
//! | IN-5 | `--from` + `--to`: transplant source from source, cwd is target |

// IN-3 is the most critical: if source files are written to during a cross-loaded
// run, the isolation contract is broken and source history would be polluted.
// Since --dry-run is used, no subprocess fires — but clr itself must not touch
// the source files during command building.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ df, make_session_for, run_cli };

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

/// BUG-391 regression guard: `df()` (now shared via `cli_binary_test_helpers`,
/// Fix(BUG-493)) must match production `encode_path()` for a
/// dot-containing path (e.g. `tempfile::TempDir::new()`'s literal `.tmp` prefix) —
/// the exact input class that exposed the two encoders' prior divergence.
///
/// ## Root Cause
/// `df()` hand-rolled a duplicate of `claude_storage_core::encode_path()`, only
/// substituting `_`→`-`. Once `encode_path()` was generalized (BUG-366) to map
/// every non-alphanumeric character to `-`, the two encoders diverged for any
/// dot-containing path — exactly what `tempfile::TempDir::new()` always produces.
///
/// ## Why Not Caught
/// No test asserted parity between `df()` and `encode_path()` in this file; every
/// existing fixture's input happened to avoid the divergent class until BUG-366
/// widened `encode_path()`'s substitution scope.
///
/// ## Fix Applied
/// `df()` (now shared via `cli_binary_test_helpers`, Fix(BUG-493)) delegates to
/// `claude_storage_core::encode_path()` directly instead of reimplementing it, so
/// it cannot drift from production behavior.
///
/// ## Prevention
/// This test locks in parity for the specific input class (dot-containing paths)
/// that caused the original divergence — a future reimplementation regressing
/// `df()` back to a hand-rolled encoder would fail here immediately.
///
/// ## Pitfall
/// A duplicate encoder can pass indefinitely against fixtures that avoid the
/// divergent input class, then silently fail the moment a fixture's shape changes
/// (here: `tempfile::TempDir`'s literal `.tmp` prefix) with no code change to explain it.
#[ test ]
fn df_matches_production_encode_path_for_dot_containing_path()
{
  let path = "/tmp/.tmpAbCdEfGh/proj";
  let test_encoded = df( path );
  let real_encoded = claude_storage_core::encode_path( std::path::Path::new( path ) )
    .expect( "encode_path" );
  assert_eq!(
    test_encoded, real_encoded,
    "test df() helper must match production encode_path() for dot-containing paths \
     (BUG-391 regression guard)"
  );
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
    .env_remove( "CLR_FROM" )
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

/// IN-1: the transplanted session comes from SOURCE storage, not target storage.
///
/// Target dir has no `.jsonl` files; source dir has `lll-001.jsonl`.  The
/// transplant plan's source side must be the source storage's session file;
/// no target-derived path may ever appear as a transplant SOURCE (the target
/// storage legitimately appears only as the DESTINATION).
#[ test ]
fn in1_uuid_read_from_source_not_target()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/011it1-src";
  let jsonl = make_session_for( ch.path(), src, "lll-001" );
  // Target dir exists but has NO session files in Claude storage
  let tgt = tempfile::TempDir::new().expect( "tgt tmpdir" );
  let tgt_str = tgt.path().to_str().expect( "utf-8" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--from", src, "--to", tgt_str, "task" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  // Transplant source must be the SOURCE storage's session file
  assert!(
    stdout.contains( &format!( "# session-transplant: {} -> ", jsonl.display() ) ),
    "transplant source must come from source storage. Got:\n{stdout}"
  );
  // Target's encoded storage must never appear as a transplant SOURCE
  let tgt_canon = std::fs::canonicalize( tgt.path() ).expect( "canonicalize target" );
  let tgt_session_dir = format!(
    "{ch_str}/projects/{}",
    df( tgt_canon.to_str().expect( "utf-8" ) )
  );
  assert!(
    !stdout.contains( &format!( "# session-transplant: {tgt_session_dir}" ) ),
    "no target-derived path may be a transplant source. Got:\n{stdout}"
  );
}

// ── IN-2: Subprocess working directory is target, not source ──────────────────

/// IN-2: Subprocess working directory is target dir, not source dir.
///
/// `--to <tgt>` sets the working dir; `--from <src>` does not.
#[ test ]
fn in2_subprocess_dir_is_target_not_source()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/011it2-src";
  let _ = make_session_for( ch.path(), src, "mmm-002" );
  let tgt = tempfile::TempDir::new().expect( "tgt tmpdir" );
  let tgt_str = tgt.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--from", src, "--to", tgt_str, "task" ],
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
    &[ "--from", src, "--to", tgt_str, "Continue" ],
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

// ── IN-4: --session-dir no longer suppresses --from (BUG-493) ──────────────────

/// IN-4: `--session-dir` no longer takes precedence over `--from`.
///
/// Fix(BUG-493): `--session-dir` is deprecated and inert — claude ≥2.x ignores
/// the `CLAUDE_CODE_SESSION_DIR` export it used to trigger, so it must never
/// suppress `--from`'s transplant, the only mechanism that still works.
#[ test ]
fn in4_session_dir_no_longer_wins_over_from()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/011it4-src";
  // Source session — must NOT be suppressed by --session-dir (BUG-493).
  let jsonl = make_session_for( ch.path(), src, "ooo-004" );
  // Raw override dir — must be inert now (BUG-493), not suppress --from.
  let raw_dir = tempfile::TempDir::new().expect( "raw tmpdir" );
  std::fs::write( raw_dir.path().join( "ppp-005.jsonl" ), b"{}" )
    .expect( "write raw session" );
  let raw_str = raw_dir.path().to_str().expect( "utf-8" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "--dry-run", "--from", src, "--session-dir", raw_str, "test" ] )
    .env( "CLAUDE_HOME", ch_str )
    .env_remove( "CLR_DIR" )
    .env_remove( "CLR_SESSION_DIR" )
    .env_remove( "CLR_FROM" )
    .output()
    .expect( "failed to invoke clr binary" );
  assert!(
    out.status.success(),
    "dry-run failed (exit {})\nstderr: {}",
    out.status.code().unwrap_or( -1 ),
    String::from_utf8_lossy( &out.stderr ),
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  let stderr = String::from_utf8_lossy( &out.stderr );
  // --from's transplant plan must proceed — --session-dir no longer suppresses it.
  assert!(
    stdout.contains( &format!( "# session-transplant: {} -> ", jsonl.display() ) ),
    "`--from` transplant must proceed even with --session-dir present. Got:\n{stdout}"
  );
  // The dead env export must never appear.
  assert!(
    !stdout.contains( "CLAUDE_CODE_SESSION_DIR=" ),
    "the dead CLAUDE_CODE_SESSION_DIR export must never appear (BUG-493). Got:\n{stdout}"
  );
  // --session-dir's own deprecation warning must fire, naming its value.
  assert!(
    stderr.contains( "deprecated" ) && stderr.contains( raw_str ),
    "--session-dir must emit a deprecation warning naming its value. Got:\n{stderr}"
  );
}

// ── IN-5: --from + --to: session UUID from source, cwd is target ───────────────

/// IN-5: Both read isolation (transplant source from SOURCE storage) and run
///       isolation (cwd = target) hold simultaneously when `--to` and
///       `--from` are combined.
#[ test ]
fn in5_combined_source_uuid_and_target_cwd()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/011it5-src";
  let jsonl = make_session_for( ch.path(), src, "qqq-006" );
  let tgt = tempfile::TempDir::new().expect( "tgt tmpdir" );
  let tgt_str = tgt.path().to_str().expect( "utf-8" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--to", tgt_str, "--from", src, "Continue" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  // Read isolation: the transplanted session file comes from source storage
  assert!(
    stdout.contains( &format!( "# session-transplant: {} -> ", jsonl.display() ) ),
    "transplant source must come from source storage. Got:\n{stdout}"
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
