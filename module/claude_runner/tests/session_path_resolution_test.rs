//! Feature tests for session path resolution.
//!
//! Covers FT-6–FT-10 from `tests/docs/feature/005_session_path_resolution.md`.
//! FT-1–FT-5 (unit-level `scope_for()` tests) live in
//! `claude_storage_core/tests/scope_test.rs`.
//!
//! | Test | Covers |
//! |------|--------|
//! | FT-6 | `clr scope` prints 6 `CLAUDE_*` vars in `key=value` format |
//! | FT-7 | `--session-from` sets `CLAUDE_CODE_SESSION_DIR` to source dir storage |
//! | FT-8 | `--to` + `--session-from`: Claude runs in target, loads from source |
//! | FT-9 | `--to` is an alias for `--dir`; behavior is identical |
//! | FT-10 | `--session-dir` takes precedence over `--session-from` |

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };

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

/// Encode a path using the `Df()` algorithm from `algorithm/001_path_encoding.md`.
///
/// Delegates to the real production encoder — see `Fix(BUG-391)` below.
// Fix(BUG-391): delegate to the real production encoder instead of reimplementing
// it — the prior hand-rolled body only substituted '_', which diverged from
// claude_storage_core::encode_path()'s full non-alphanumeric substitution the
// moment a fixture (e.g. tempfile::TempDir's ".tmp"-prefixed path) contained a '.'.
// Root cause: this was a duplicate encoder (also flagged by BUG-366, already fixed
// for scope_command_test.rs by BUG-386) reimplementing production logic instead of
// calling it, so it silently diverged when encode_path()'s substitution scope widened.
// Pitfall: never hand-roll a test-local copy of a production encoding/formatting
// function — call the real function so the fixture cannot drift from production.
fn df( path : &str ) -> String
{
  claude_storage_core::encode_path( std::path::Path::new( path ) )
    .expect( "df(): path must encode successfully in test fixtures" )
}

/// BUG-391 regression guard: `df()` must match production `encode_path()` for a
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
/// `df()` now delegates to `claude_storage_core::encode_path()` directly instead
/// of reimplementing it, so it cannot drift from production behavior.
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

/// Create `<claude_home>/projects/<df(src)>/<uuid>.jsonl`.
fn make_session_for( claude_home : &std::path::Path, src : &str, uuid : &str )
{
  let session_dir = claude_home.join( "projects" ).join( df( src ) );
  std::fs::create_dir_all( &session_dir ).expect( "create session dir" );
  std::fs::write( session_dir.join( format!( "{uuid}.jsonl" ) ), b"{}" )
    .expect( "write session jsonl" );
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

// ── FT-6: `clr scope` prints 6 CLAUDE_* vars in key=value format ──────────────

/// FT-6: `clr scope --dir /tmp` prints exactly 6 `CLAUDE_*=...` lines.
///
/// Lines are printed in order: `HOME`, `PROJECTS_DIR`, `SESSION_DIR`, `MEMORY_DIR`,
/// `MEMORY_FILE`, `SESSION_FILE`.  Output is valid for `eval`.
#[ test ]
fn ft6_scope_prints_six_vars_in_key_value_format()
{
  let out = run_cli( &[ "scope", "--dir", "/tmp" ] );
  assert!( out.status.success(), "`clr scope --dir /tmp` must exit 0: {:?}", out.status );
  let stdout = String::from_utf8_lossy( &out.stdout );
  let lines : Vec<&str> = stdout.lines().collect();
  assert_eq!( lines.len(), 6, "Must print exactly 6 lines. Got:\n{stdout}" );

  // Each line must match `CLAUDE_[A-Z_]=...`
  for line in &lines
  {
    let ( key, _ ) = line.split_once( '=' )
      .unwrap_or_else( || panic!( "Line `{line}` is not key=value format" ) );
    assert!(
      key.starts_with( "CLAUDE_" ),
      "Key `{key}` must start with `CLAUDE_`. Got: `{line}`"
    );
  }

  // Verify ordering
  let expected_keys = [
    "CLAUDE_HOME",
    "CLAUDE_PROJECTS_DIR",
    "CLAUDE_SESSION_DIR",
    "CLAUDE_MEMORY_DIR",
    "CLAUDE_MEMORY_FILE",
    "CLAUDE_SESSION_FILE",
  ];
  for ( i, expected ) in expected_keys.iter().enumerate()
  {
    assert!(
      lines[ i ].starts_with( expected ),
      "Line {} must start with `{expected}`. Got: `{}`", i + 1, lines[ i ]
    );
  }
}

// ── FT-7: --session-from resumes most recent session from source dir ───────────

/// FT-7: `--session-from <src>` sets `CLAUDE_CODE_SESSION_DIR` to source storage and
/// activates continue mode when a session file exists there.
///
/// The subprocess working directory is CWD (no `--to` flag; no `cd` prefix).
#[ test ]
fn ft7_session_from_resumes_source_session()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/ft7-src";
  make_session_for( ch.path(), src, "hhh-101" );
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
  // No --to → no cd prefix for the source dir
  assert!(
    !stdout.contains( &format!( "cd {src}" ) ),
    "CWD must not change to source dir. Got:\n{stdout}"
  );
}

// ── FT-8: --to + --session-from: runs in target, loads from source ─────────────

/// FT-8: `--to <tgt> --session-from <src>` sets working dir to target, loads from source.
///
/// `CLAUDE_CODE_SESSION_DIR` must point to source storage; subprocess `cd` must be target.
#[ test ]
fn ft8_to_plus_session_from_target_dir_source_session()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/ft8-src";
  make_session_for( ch.path(), src, "iii-202" );
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
    "subprocess dir must be target `{tgt_str}`. Got:\n{stdout}"
  );
}

// ── FT-9: --to is an alias for --dir ──────────────────────────────────────────

/// FT-9: `--to` and `--dir` produce identical dry-run output.
///
/// Both forms must set the subprocess working directory to the given path.
#[ test ]
fn ft9_to_alias_identical_to_dir()
{
  let tgt = tempfile::TempDir::new().expect( "tmpdir" );
  let tgt_str = tgt.path().to_str().expect( "utf-8" );
  let ch = tempfile::TempDir::new().expect( "claude_home tmpdir" );

  let stdout_dir = run_dry_env(
    &[ "--dir", tgt_str, "task" ],
    &[ ( "CLAUDE_HOME", ch.path().to_str().expect( "utf-8" ) ) ],
  );
  let stdout_to = run_dry_env(
    &[ "--to", tgt_str, "task" ],
    &[ ( "CLAUDE_HOME", ch.path().to_str().expect( "utf-8" ) ) ],
  );

  // Both must show the same working dir
  assert!(
    stdout_dir.contains( &format!( "cd {tgt_str}" ) ),
    "`--dir` must emit `cd {tgt_str}`. Got:\n{stdout_dir}"
  );
  assert!(
    stdout_to.contains( &format!( "cd {tgt_str}" ) ),
    "`--to` must emit `cd {tgt_str}`. Got:\n{stdout_to}"
  );
  assert_eq!(
    stdout_dir, stdout_to,
    "`--dir` and `--to` must produce identical dry-run output"
  );
}

// ── FT-10: --session-dir takes precedence over --session-from ─────────────────

/// FT-10: `--session-dir` raw path wins over `--session-from` computed path.
///
/// `CLAUDE_CODE_SESSION_DIR` must equal the raw `--session-dir` path, not the
/// computed source storage path.
#[ test ]
fn ft10_session_dir_wins_over_session_from()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/ft10-src";
  // Source session — should be ignored (--session-dir wins)
  make_session_for( ch.path(), src, "jjj-303" );
  // Raw session dir that wins
  let raw_dir = tempfile::TempDir::new().expect( "raw tmpdir" );
  std::fs::write( raw_dir.path().join( "kkk-404.jsonl" ), b"{}" )
    .expect( "write raw session" );
  let raw_str = raw_dir.path().to_str().expect( "utf-8" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--session-from", src, "--session-dir", raw_str, "test" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  // Raw --session-dir must be used as CLAUDE_CODE_SESSION_DIR
  assert!(
    stdout.contains( &format!( "CLAUDE_CODE_SESSION_DIR={raw_str}" ) ),
    "`--session-dir` raw path must win. Got:\n{stdout}"
  );
  // Source computed path must NOT appear
  let src_dir = format!( "{ch_str}/projects/{}", df( src ) );
  assert!(
    !stdout.contains( &src_dir ),
    "source computed path `{src_dir}` must NOT appear. Got:\n{stdout}"
  );
}

// ── Sanity guard ───────────────────────────────────────────────────────────────

/// Verify `clr scope` is reachable (satisfies `use run_cli_with_env` lint).
#[ test ]
fn sanity_scope_reachable()
{
  let out = run_cli_with_env( &[ "scope", "--dir", "/tmp" ], &[] );
  assert!( out.status.success(), "`clr scope --dir /tmp` must exit 0: {:?}", out.status );
}
