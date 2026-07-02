//! Feature tests for session path resolution.
//!
//! Covers FT-6–FT-10 from `tests/docs/feature/005_session_path_resolution.md`.
//! FT-1–FT-5 (unit-level `scope_for()` tests) live in
//! `claude_storage_core/tests/scope_test.rs`.
//!
//! | Test | Covers |
//! |------|--------|
//! | FT-6 | `clr scope` prints 6 `CLAUDE_*` vars in `key=value` format |
//! | FT-7 | `--session-from` resumes most recent session from source dir |
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

/// FT-7: `--session-from <src>` injects `-c <uuid>` from source dir's session storage.
///
/// The subprocess working directory is CWD (no `--to` flag; no `cd` prefix).
#[ test ]
fn ft7_session_from_resumes_source_session()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/ft7-src";
  make_session_for( ch.path(), src, "hhh-101" );
  let stdout = run_dry_env(
    &[ "--session-from", src, "Continue" ],
    &[ ( "CLAUDE_HOME", ch.path().to_str().expect( "utf-8" ) ) ],
  );
  assert!( stdout.contains( "hhh-101" ), "must contain `-c hhh-101`. Got:\n{stdout}" );
  // No --to → no cd prefix for the source dir
  assert!(
    !stdout.contains( &format!( "cd {src}" ) ),
    "CWD must not change to source dir. Got:\n{stdout}"
  );
}

// ── FT-8: --to + --session-from: runs in target, loads from source ─────────────

/// FT-8: `--to <tgt> --session-from <src>` sets working dir to target, loads from source.
#[ test ]
fn ft8_to_plus_session_from_target_dir_source_session()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/ft8-src";
  make_session_for( ch.path(), src, "iii-202" );
  let tgt = tempfile::TempDir::new().expect( "tgt tmpdir" );
  let tgt_str = tgt.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--to", tgt_str, "--session-from", src, "Continue" ],
    &[ ( "CLAUDE_HOME", ch.path().to_str().expect( "utf-8" ) ) ],
  );
  assert!( stdout.contains( "iii-202" ), "must contain `-c iii-202`. Got:\n{stdout}" );
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
#[ test ]
fn ft10_session_dir_wins_over_session_from()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/ft10-src";
  // Source session — should be ignored
  make_session_for( ch.path(), src, "jjj-303" );
  // Raw session dir that wins
  let raw_dir = tempfile::TempDir::new().expect( "raw tmpdir" );
  std::fs::write( raw_dir.path().join( "kkk-404.jsonl" ), b"{}" )
    .expect( "write raw session" );
  let raw_str = raw_dir.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--session-from", src, "--session-dir", raw_str, "test" ],
    &[ ( "CLAUDE_HOME", ch.path().to_str().expect( "utf-8" ) ) ],
  );
  assert!( stdout.contains( "kkk-404" ), "`--session-dir` UUID must win. Got:\n{stdout}" );
  assert!(
    !stdout.contains( "jjj-303" ),
    "`jjj-303` must NOT appear. Got:\n{stdout}"
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
