//! Feature tests for session path resolution.
//!
//! Covers FT-6–FT-10 from `tests/docs/feature/005_session_path_resolution.md`.
//! FT-1–FT-5 (unit-level `scope_for()` tests) live in
//! `claude_storage_core/tests/scope_test.rs`.
//!
//! | Test | Covers |
//! |------|--------|
//! | FT-6 | `clr scope` prints 6 `CLAUDE_*` vars in `key=value` format |
//! | FT-7 | `--from` plans a transplant of the most-recent source session |
//! | FT-8 | `--to` + `--from`: Claude runs in target, loads from source |
//! | FT-9 | `--to` is an alias for `--dir`; behavior is identical |
//! | FT-10 | `--session-dir` no longer suppresses `--from` (deprecated, inert) |

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ df, make_session_for, run_cli, run_cli_with_env };

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

// ── FT-7: --from resumes most recent session from source dir ───────────────────

/// FT-7: `--from <src>` plans a transplant of the most-recent source
/// session file (BUG-490) and activates continue mode when one exists.
///
/// The subprocess working directory is CWD (no `--to` flag; no `cd` prefix).
#[ test ]
fn ft7_from_resumes_source_session()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/ft7-src";
  let _ = make_session_for( ch.path(), src, "hhh-101" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--from", src, "Continue" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  let expected_src = format!( "{ch_str}/projects/{}/hhh-101.jsonl", df( src ) );
  assert!(
    stdout.contains( &format!( "# session-transplant: {expected_src} -> " ) ),
    "plan must name the source session file (BUG-490 transplant). Got:\n{stdout}"
  );
  // No --to → no cd prefix for the source dir
  assert!(
    !stdout.contains( &format!( "cd {src}" ) ),
    "CWD must not change to source dir. Got:\n{stdout}"
  );
}

// ── FT-8: --to + --from: runs in target, loads from source ─────────────────────

/// FT-8: `--to <tgt> --from <src>` sets working dir to target, loads from source.
///
/// The transplant plan copies the source session file into the TARGET's own
/// storage (BUG-490); subprocess `cd` must be the target.
#[ test ]
fn ft8_to_plus_from_target_dir_source_session()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/ft8-src";
  let _ = make_session_for( ch.path(), src, "iii-202" );
  let tgt = tempfile::TempDir::new().expect( "tgt tmpdir" );
  let tgt_str = tgt.path().to_str().expect( "utf-8" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--to", tgt_str, "--from", src, "Continue" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  let expected_src = format!( "{ch_str}/projects/{}/iii-202.jsonl", df( src ) );
  let tgt_canon = std::fs::canonicalize( tgt.path() ).expect( "canonicalize target" );
  let target_storage = format!(
    "{ch_str}/projects/{}",
    df( tgt_canon.to_str().expect( "utf-8" ) )
  );
  assert!(
    stdout.contains( &format!( "# session-transplant: {expected_src} -> {target_storage}" ) ),
    "plan must copy the source session into the TARGET's storage. Got:\n{stdout}"
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

// ── FT-10: --session-dir no longer suppresses --from (BUG-493) ─────────────────

/// FT-10: `--session-dir` no longer takes precedence over `--from`.
///
/// Fix(BUG-493): `--session-dir` is deprecated and inert — claude ≥2.x ignores
/// the `CLAUDE_CODE_SESSION_DIR` export it used to trigger, so it must never
/// suppress `--from`'s transplant, the only mechanism that still works.
#[ test ]
fn ft10_session_dir_no_longer_wins_over_from()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/ft10-src";
  // Source session — must NOT be suppressed by --session-dir (BUG-493).
  let jsonl = make_session_for( ch.path(), src, "jjj-303" );
  // Raw override dir — must be inert now (BUG-493), not suppress --from.
  let raw_dir = tempfile::TempDir::new().expect( "raw tmpdir" );
  std::fs::write( raw_dir.path().join( "kkk-404.jsonl" ), b"{}" )
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

// ── Sanity guard ───────────────────────────────────────────────────────────────

/// Verify `clr scope` is reachable (satisfies `use run_cli_with_env` lint).
#[ test ]
fn sanity_scope_reachable()
{
  let out = run_cli_with_env( &[ "scope", "--dir", "/tmp" ], &[] );
  assert!( out.status.success(), "`clr scope --dir /tmp` must exit 0: {:?}", out.status );
}
