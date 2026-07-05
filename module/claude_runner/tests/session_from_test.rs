//! Edge case tests for the `--session-from` parameter (alias `--from`).
//!
//! Covers EC-1–EC-8 from `tests/docs/cli/param/076_session_from.md` and
//! US-1–US-7 from `tests/docs/cli/user_story/28_session_transplant.md`.
//!
//! `--session-from <DIR>` computes `scope_for(DIR).claude_session_dir`, sets
//! `CLAUDE_CODE_SESSION_DIR` to that path, and activates `-c` (continue) mode when
//! a qualifying `.jsonl` exists there.  All tests use `--dry-run` so no real Claude
//! binary is needed.
//!
//! # Test Setup Pattern
//!
//! Since `--session-from` uses `scope_for(src)` to resolve the session storage
//! directory, tests set `CLAUDE_HOME` to a temp dir and place the `.jsonl` file
//! in `<claude_home>/projects/<df(src_dir)>/` — the exact path that `scope_for`
//! computes.  The `df()` helper implements the same `Df()` encoding algorithm.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::run_cli;

// ── Shared helpers ─────────────────────────────────────────────────────────────

/// Container guard (mirrors the private `assert_container` in helpers).
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

/// Encode a path using the `Df()` algorithm from `algorithm/001_path_encoding.md`.
///
/// - Strip leading `/`, split on `/`, replace `_` with `-` per component.
/// - Prepend `-`; use `--` separator for hyphen-leading components.
// BUG-366 ../../../task/claude_storage_core/bug/unverified/366_encode_path_dot_handling_divergence.md — duplicate hand-rolled encoder shares encode_path()'s dot-blind, no-length-fallback bug; needs sweeping once the fix lands
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

/// Create `<claude_home>/projects/<df(src_dir)>/<uuid>.jsonl` with non-empty content.
///
/// Returns the `.jsonl` path.  The caller must keep the `TempDir` alive.
fn make_session_for( claude_home : &std::path::Path, src_dir : &str, uuid : &str ) -> std::path::PathBuf
{
  let session_dir = claude_home.join( "projects" ).join( df( src_dir ) );
  std::fs::create_dir_all( &session_dir ).expect( "create session dir" );
  let file = session_dir.join( format!( "{uuid}.jsonl" ) );
  std::fs::write( &file, b"{}" ).expect( "write session jsonl" );
  file
}

/// Run `clr --dry-run <args>` with extra env vars; return stdout.
///
/// Removes `CLR_DIR`, `CLR_SESSION_DIR`, and `CLR_SESSION_FROM` from the inherited
/// environment before injecting `env` — prevents ambient values from interfering
/// with `--session-from` / `--session-dir` behavior.
///
/// # Panics
///
/// Panics if the subprocess cannot be launched or exits non-zero.
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

// ── EC-1: --session-from injects -c when source has session ───────────────────

/// EC-1: `--session-from` sets `CLAUDE_CODE_SESSION_DIR` to the computed source
/// storage path and activates continue mode (`-c`) when a session file exists.
#[ test ]
fn ec1_session_from_injects_continue()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/076ec1-src";
  make_session_for( ch.path(), src, "aaa-111" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--session-from", src, "Continue" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  let expected_dir = format!( "{ch_str}/projects/{}", df( src ) );
  assert!(
    stdout.contains( &format!( "CLAUDE_CODE_SESSION_DIR={expected_dir}" ) ),
    "session dir must point to source storage. Got:\n{stdout}"
  );
  // Continue mode: `-c` flag precedes the quoted message in the subprocess command.
  assert!(
    stdout.contains( " -c \"" ),
    "dry-run must use continue flag `-c`. Got:\n{stdout}"
  );
}

// ── EC-2: --from alias behaves identically to --session-from ──────────────────

/// EC-2: `--from` alias behaves identically to `--session-from`.
///
/// `CLAUDE_CODE_SESSION_DIR` must point to the same computed source storage path.
#[ test ]
fn ec2_from_alias_identical_to_session_from()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/076ec2-src";
  make_session_for( ch.path(), src, "bbb-222" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--from", src, "Continue" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  let expected_dir = format!( "{ch_str}/projects/{}", df( src ) );
  assert!(
    stdout.contains( &format!( "CLAUDE_CODE_SESSION_DIR={expected_dir}" ) ),
    "`--from` alias must set session dir to source storage. Got:\n{stdout}"
  );
}

// ── EC-3: No .jsonl → no -c injected ──────────────────────────────────────────

/// EC-3: Source dir with no `.jsonl` → no `-c` injected; fresh session starts.
#[ test ]
fn ec3_empty_source_no_continue()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  // No .jsonl created — empty session storage
  let src = "/tmp/076ec3-empty-src";
  let stdout = run_dry_env(
    &[ "--session-from", src, "Start fresh" ],
    &[ ( "CLAUDE_HOME", ch.path().to_str().expect( "utf-8" ) ) ],
  );
  assert!(
    !stdout.contains( " -c \"" ),
    "no session → dry-run must NOT have continue flag `-c`. Got:\n{stdout}"
  );
}

// ── EC-4: --session-dir wins over --session-from ──────────────────────────────

/// EC-4: `--session-dir` takes precedence over `--session-from`.
///
/// When both are given, `CLAUDE_CODE_SESSION_DIR` must be the raw `--session-dir`
/// path; the computed source storage path from `--session-from` must not appear.
#[ test ]
fn ec4_session_dir_wins_over_session_from()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/076ec4-src";
  // Source session (should be ignored — --session-dir wins)
  make_session_for( ch.path(), src, "ccc-333" );
  // Override session dir (raw path wins)
  let override_dir = tempfile::TempDir::new().expect( "override tmpdir" );
  std::fs::write( override_dir.path().join( "xyz-789.jsonl" ), b"{}" )
    .expect( "write override session" );
  let override_str = override_dir.path().to_str().expect( "utf-8" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--session-from", src, "--session-dir", override_str, "test" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  // Raw --session-dir path must be used
  assert!(
    stdout.contains( &format!( "CLAUDE_CODE_SESSION_DIR={override_str}" ) ),
    "`--session-dir` raw path must win. Got:\n{stdout}"
  );
  // Computed source storage path must NOT appear
  let src_dir = format!( "{ch_str}/projects/{}", df( src ) );
  assert!(
    !stdout.contains( &src_dir ),
    "`--session-from` computed path `{src_dir}` must NOT appear. Got:\n{stdout}"
  );
}

// ── EC-5: --new-session suppresses --session-from ─────────────────────────────

/// EC-5: `--new-session` takes precedence over `--session-from`.
///
/// `--new-session` suppresses cross-loading; no `-c` is injected.
#[ test ]
fn ec5_new_session_suppresses_session_from()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/076ec5-src";
  make_session_for( ch.path(), src, "ddd-444" );
  let stdout = run_dry_env(
    &[ "--session-from", src, "--new-session", "fresh" ],
    &[ ( "CLAUDE_HOME", ch.path().to_str().expect( "utf-8" ) ) ],
  );
  // CLAUDE_CODE_SESSION_DIR is still set (--session-from computes the path),
  // but continue mode must be suppressed by --new-session.
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let expected_dir = format!( "{ch_str}/projects/{}", df( src ) );
  assert!(
    stdout.contains( &format!( "CLAUDE_CODE_SESSION_DIR={expected_dir}" ) ),
    "session dir must still be set from `--session-from`. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( " -c \"" ),
    "`--new-session` must suppress continue flag `-c`. Got:\n{stdout}"
  );
}

// ── EC-6: --to + --session-from ───────────────────────────────────────────────

/// EC-6: `--to <tgt>` + `--session-from <src>`: Claude runs in target, loads from source.
///
/// `CLAUDE_CODE_SESSION_DIR` must point to source storage; subprocess `cd` must be target.
#[ test ]
fn ec6_to_plus_session_from()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/076ec6-src";
  make_session_for( ch.path(), src, "eee-555" );
  let tgt = tempfile::TempDir::new().expect( "target tmpdir" );
  let tgt_str = tgt.path().to_str().expect( "utf-8" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--to", tgt_str, "--session-from", src, "Continue" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  let expected_dir = format!( "{ch_str}/projects/{}", df( src ) );
  assert!(
    stdout.contains( &format!( "CLAUDE_CODE_SESSION_DIR={expected_dir}" ) ),
    "session dir must point to source storage. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( &format!( "cd {tgt_str}" ) ),
    "subprocess dir must be target `{tgt_str}`. Got:\n{stdout}"
  );
}

// ── EC-7: CLR_SESSION_FROM env var equivalent ─────────────────────────────────

/// EC-7: `CLR_SESSION_FROM` env var is equivalent to `--session-from`.
///
/// No `--session-from` on CLI; `CLR_SESSION_FROM` provides the source path.
/// `CLAUDE_CODE_SESSION_DIR` must point to the computed source storage path.
///
/// Note: cannot use `run_dry_env` here because it calls `env_remove("CLR_SESSION_FROM")`
/// which would strip the very variable this test passes in.
#[ test ]
fn ec7_clr_session_from_env_var()
{
  container_check();
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/076ec7-src";
  make_session_for( ch.path(), src, "fff-666" );
  let home_str = ch.path().to_str().expect( "utf-8" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "--dry-run", "Continue" ] )
    .env( "CLAUDE_HOME", home_str )
    .env( "CLR_SESSION_FROM", src )
    .env_remove( "CLR_DIR" )
    .env_remove( "CLR_SESSION_DIR" )
    .output()
    .expect( "failed to invoke clr binary" );
  assert!(
    out.status.success(),
    "dry-run failed (exit {})\nstdout: {}\nstderr: {}",
    out.status.code().unwrap_or( -1 ),
    String::from_utf8_lossy( &out.stdout ),
    String::from_utf8_lossy( &out.stderr ),
  );
  let stdout = String::from_utf8_lossy( &out.stdout ).into_owned();
  let expected_dir = format!( "{home_str}/projects/{}", df( src ) );
  assert!(
    stdout.contains( &format!( "CLAUDE_CODE_SESSION_DIR={expected_dir}" ) ),
    "`CLR_SESSION_FROM` must set session dir to source storage. Got:\n{stdout}"
  );
}

// ── EC-8: --dry-run WYSIWYG reflects session-from UUID ────────────────────────

/// EC-8: Dry-run accurately reflects `CLAUDE_CODE_SESSION_DIR` for source session.
///
/// WYSIWYG: dry-run shows the computed source storage path that the subprocess
/// will receive as `CLAUDE_CODE_SESSION_DIR`.
#[ test ]
fn ec8_dry_run_wysiwyg_session_from()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/076ec8-src";
  make_session_for( ch.path(), src, "ggg-777" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--session-from", src, "task" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  let expected_dir = format!( "{ch_str}/projects/{}", df( src ) );
  assert!(
    stdout.contains( &format!( "CLAUDE_CODE_SESSION_DIR={expected_dir}" ) ),
    "dry-run must reflect source storage path. Got:\n{stdout}"
  );
}

// ── US-1: Clone outward — -c injected from source session dir ─────────────────

/// US-1: `--to <tgt> --session-from <src>` clones outward.
///
/// `CLAUDE_CODE_SESSION_DIR` must point to source storage; subprocess `cd` must be target.
#[ test ]
fn us1_clone_outward_continue_injected()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/us28-project-a";
  make_session_for( ch.path(), src, "abc-123" );
  let tgt = tempfile::TempDir::new().expect( "tgt tmpdir" );
  let tgt_str = tgt.path().to_str().expect( "utf-8" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--to", tgt_str, "--session-from", src, "Continue" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  let expected_dir = format!( "{ch_str}/projects/{}", df( src ) );
  assert!(
    stdout.contains( &format!( "CLAUDE_CODE_SESSION_DIR={expected_dir}" ) ),
    "session dir must point to source storage. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( &format!( "cd {tgt_str}" ) ),
    "subprocess dir must be target. Got:\n{stdout}"
  );
}

// ── US-2: Inject inward — runs in CWD, loads from source ──────────────────────

/// US-2: `--session-from <src>` (no `--to`) runs in CWD, loads from source.
///
/// `CLAUDE_CODE_SESSION_DIR` points to source storage; no `cd <src>` in output.
#[ test ]
fn us2_inject_inward_cwd_unchanged()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/us28-project-b-inward";
  make_session_for( ch.path(), src, "def-456" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--session-from", src, "What did you do in B?" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  let expected_dir = format!( "{ch_str}/projects/{}", df( src ) );
  assert!(
    stdout.contains( &format!( "CLAUDE_CODE_SESSION_DIR={expected_dir}" ) ),
    "session dir must point to source storage. Got:\n{stdout}"
  );
  // No --to means no `cd` prefix — CWD is implicit.
  assert!(
    !stdout.contains( &format!( "cd {src}" ) ),
    "subprocess dir must NOT be source dir. Got:\n{stdout}"
  );
}

// ── US-3: No source history → fresh session ───────────────────────────────────

/// US-3: Source dir with no qualifying `.jsonl` → no `-c`; fresh session starts.
#[ test ]
fn us3_no_source_history_fresh_session()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/us28-empty-source";
  // No session file — empty storage
  let stdout = run_dry_env(
    &[ "--session-from", src, "Start fresh" ],
    &[ ( "CLAUDE_HOME", ch.path().to_str().expect( "utf-8" ) ) ],
  );
  assert!(
    !stdout.contains( " -c \"" ),
    "no history → must NOT have continue flag `-c`. Got:\n{stdout}"
  );
}

// ── US-4: --from alias ────────────────────────────────────────────────────────

/// US-4: `--from` alias is accepted and produces the same result as `--session-from`.
///
/// `CLAUDE_CODE_SESSION_DIR` must equal the result from `--session-from`; subprocess
/// `cd` must be the target.
#[ test ]
fn us4_from_alias_accepted()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/us28-proj-a-alias";
  make_session_for( ch.path(), src, "abc-123" );
  let tgt = tempfile::TempDir::new().expect( "tgt tmpdir" );
  let tgt_str = tgt.path().to_str().expect( "utf-8" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--to", tgt_str, "--from", src, "Continue" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  let expected_dir = format!( "{ch_str}/projects/{}", df( src ) );
  assert!(
    stdout.contains( &format!( "CLAUDE_CODE_SESSION_DIR={expected_dir}" ) ),
    "`--from` alias must set session dir to source storage. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( &format!( "cd {tgt_str}" ) ),
    "subprocess dir must be target. Got:\n{stdout}"
  );
}

// ── US-5: --to alias is accepted ──────────────────────────────────────────────

/// US-5: `--to` alias sets subprocess working directory identically to `--dir`.
#[ test ]
fn us5_to_alias_sets_working_dir()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/us28-proj-a-to";
  make_session_for( ch.path(), src, "abc-123" );
  let tgt = tempfile::TempDir::new().expect( "tgt tmpdir" );
  let tgt_str = tgt.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--to", tgt_str, "--session-from", src, "test" ],
    &[ ( "CLAUDE_HOME", ch.path().to_str().expect( "utf-8" ) ) ],
  );
  assert!(
    stdout.contains( &format!( "cd {tgt_str}" ) ),
    "`--to` must set subprocess dir to target. Got:\n{stdout}"
  );
}

// ── US-6: --session-dir wins over --session-from ──────────────────────────────

/// US-6: `--session-dir` raw path wins over `--session-from` computed path.
///
/// `CLAUDE_CODE_SESSION_DIR` must equal the raw `--session-dir` path, not the
/// computed source storage path.
#[ test ]
fn us6_session_dir_wins_over_session_from()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/us28-proj-a-prec";
  // Source session (should be ignored — --session-dir wins)
  make_session_for( ch.path(), src, "abc-123" );
  // Override session dir (raw path)
  let override_dir = tempfile::TempDir::new().expect( "override tmpdir" );
  std::fs::write( override_dir.path().join( "xyz-789.jsonl" ), b"{}" )
    .expect( "write override session" );
  let override_str = override_dir.path().to_str().expect( "utf-8" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--session-from", src, "--session-dir", override_str, "test" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  assert!(
    stdout.contains( &format!( "CLAUDE_CODE_SESSION_DIR={override_str}" ) ),
    "`--session-dir` raw path must be used. Got:\n{stdout}"
  );
  // Source computed path must NOT appear
  let src_dir = format!( "{ch_str}/projects/{}", df( src ) );
  assert!(
    !stdout.contains( &src_dir ),
    "source computed path `{src_dir}` must NOT appear. Got:\n{stdout}"
  );
}

// ── US-7: Source session files not modified after cross-loaded run ─────────────

/// US-7: Source session `.jsonl` mtime and size are unchanged after a cross-loaded run.
///
/// `clr` only reads the session UUID from the source — it must never write to it.
/// Dry-run mode is used so no subprocess runs, but the session-from path setup
/// code still executes during command building.
#[ test ]
fn us7_source_session_files_not_modified()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/us28-proj-a-immutable";
  let jsonl = make_session_for( ch.path(), src, "abc-123" );
  let tgt = tempfile::TempDir::new().expect( "tgt tmpdir" );
  let tgt_str = tgt.path().to_str().expect( "utf-8" );

  let meta_before = std::fs::metadata( &jsonl ).expect( "stat before" );
  let mtime_before = meta_before.modified().expect( "mtime before" );
  let size_before  = meta_before.len();

  run_dry_env(
    &[ "--to", tgt_str, "--session-from", src, "Continue" ],
    &[ ( "CLAUDE_HOME", ch.path().to_str().expect( "utf-8" ) ) ],
  );

  let meta_after = std::fs::metadata( &jsonl ).expect( "stat after" );
  let mtime_after = meta_after.modified().expect( "mtime after" );
  let size_after  = meta_after.len();

  assert_eq!(
    mtime_before, mtime_after,
    "source session `.jsonl` mtime must not change after cross-loaded run"
  );
  assert_eq!(
    size_before, size_after,
    "source session `.jsonl` size must not change after cross-loaded run"
  );
}

// ── Sanity: run_cli is used to trigger assert_container via at least one test ──
//
// The local `run_dry_env` helper duplicates the container check, so the
// import of `run_cli` satisfies the "used" lint for the helper module.
// This test also verifies that `clr --help` is reachable from the binary.
#[ test ]
fn sanity_clr_binary_reachable()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr binary must be reachable" );
}
