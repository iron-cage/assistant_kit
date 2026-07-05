//! Integration tests for the `scope` subcommand.
//!
//! Covers IT-1–IT-9 from `tests/docs/cli/command/09_scope.md` and
//! US-1–US-8 from `tests/docs/cli/user_story/29_scope_inspection.md`.
//!
//! `clr scope [--dir <path>]` prints all 6 `CLAUDE_*` path variables in `key=value`
//! format to stdout and exits 0.  `--dir` must be an existing directory; nonexistent
//! paths cause exit 1.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env, exit_code, stdout_str, stderr_str };

/// All 6 `CLAUDE_*` variable names that must appear in `clr scope` output.
const VARS : [ &str; 6 ] =
[
  "CLAUDE_HOME",
  "CLAUDE_PROJECTS_DIR",
  "CLAUDE_SESSION_DIR",
  "CLAUDE_MEMORY_DIR",
  "CLAUDE_MEMORY_FILE",
  "CLAUDE_SESSION_FILE",
];

// ── IT-1: `clr scope` exits 0 ─────────────────────────────────────────────────

/// IT-1: `clr scope` exits 0; stdout is non-empty (6 variable lines printed).
#[ test ]
fn it1_scope_exits_zero()
{
  let out = run_cli( &[ "scope" ] );
  assert!(
    out.status.success(),
    "`clr scope` must exit 0. Got: {:?}", out.status.code()
  );
  assert!( !out.stdout.is_empty(), "stdout must be non-empty" );
}

// ── IT-2: All 6 CLAUDE_* variable names present ───────────────────────────────

/// IT-2: Stdout contains all 6 `CLAUDE_*` variable names.
#[ test ]
fn it2_stdout_contains_all_six_vars()
{
  let out = run_cli( &[ "scope" ] );
  assert!( out.status.success(), "`clr scope` failed: {:?}", out.status );
  let stdout = stdout_str( &out );
  for var in &VARS
  {
    assert!(
      stdout.contains( var ),
      "stdout must contain `{var}`.\nGot:\n{stdout}"
    );
  }
}

// ── IT-3: `--dir` overrides target directory ──────────────────────────────────

/// IT-3: `clr scope --dir /tmp` prints vars for `/tmp`.
///
/// `CLAUDE_SESSION_DIR` must contain `-tmp` — the `Df()` encoding of `/tmp`.
#[ test ]
fn it3_dir_flag_uses_given_path()
{
  let out = run_cli( &[ "scope", "--dir", "/tmp" ] );
  assert!( out.status.success(), "`clr scope --dir /tmp` failed: {:?}", out.status );
  let stdout = stdout_str( &out );
  let session_dir_line = stdout.lines()
    .find( | l | l.starts_with( "CLAUDE_SESSION_DIR=" ) )
    .expect( "CLAUDE_SESSION_DIR must be in output" );
  assert!(
    session_dir_line.contains( "-tmp" ),
    "`CLAUDE_SESSION_DIR` must contain `-tmp` (Df encoding of `/tmp`). Got: `{session_dir_line}`"
  );
}

// ── IT-4: CLAUDE_SESSION_FILE empty when no sessions ──────────────────────────

/// IT-4: `CLAUDE_SESSION_FILE=` has empty value when no session exists.
///
/// Uses a custom `CLAUDE_HOME` temp dir so the computed session storage
/// contains no `.jsonl` files.
#[ test ]
fn it4_session_file_empty_when_no_sessions()
{
  let claude_home = tempfile::TempDir::new().expect( "tmpdir" );
  let home_str    = claude_home.path().to_str().expect( "utf-8" );
  let proj = claude_home.path().join( "proj" );
  std::fs::create_dir_all( &proj ).expect( "mkdir" );
  let proj_str = proj.to_str().expect( "utf-8" );
  let out = run_cli_with_env(
    &[ "scope", "--dir", proj_str ],
    &[ ( "CLAUDE_HOME", home_str ) ],
  );
  assert!( out.status.success(), "`clr scope` failed: {:?}", out.status );
  let stdout = stdout_str( &out );
  let line = stdout.lines()
    .find( | l | l.starts_with( "CLAUDE_SESSION_FILE=" ) )
    .expect( "CLAUDE_SESSION_FILE must be present" );
  let value = line.trim_start_matches( "CLAUDE_SESSION_FILE=" );
  assert!(
    value.is_empty(),
    "CLAUDE_SESSION_FILE must be empty when no sessions. Got: `{value}`"
  );
}

// ── IT-5: CLAUDE_HOME override reflected in output ────────────────────────────

/// IT-5: `CLAUDE_HOME` override is reflected in output.
///
/// `CLAUDE_HOME` line must equal the override; `CLAUDE_PROJECTS_DIR` must
/// contain the override path.
#[ test ]
fn it5_claude_home_override_reflected()
{
  let home     = tempfile::TempDir::new().expect( "tmpdir" );
  let home_str = home.path().to_str().expect( "utf-8" );
  let out = run_cli_with_env(
    &[ "scope", "--dir", "/tmp" ],
    &[ ( "CLAUDE_HOME", home_str ) ],
  );
  assert!( out.status.success(), "`clr scope` failed: {:?}", out.status );
  let stdout = stdout_str( &out );
  let home_line = stdout.lines()
    .find( | l | l.starts_with( "CLAUDE_HOME=" ) )
    .expect( "CLAUDE_HOME must be present" );
  assert_eq!(
    home_line,
    &format!( "CLAUDE_HOME={home_str}" ),
    "CLAUDE_HOME must equal the override. Got: `{home_line}`"
  );
  let projects_line = stdout.lines()
    .find( | l | l.starts_with( "CLAUDE_PROJECTS_DIR=" ) )
    .expect( "CLAUDE_PROJECTS_DIR must be present" );
  assert!(
    projects_line.contains( home_str ),
    "CLAUDE_PROJECTS_DIR must contain override path. Got: `{projects_line}`"
  );
}

// ── IT-6: --help flag ─────────────────────────────────────────────────────────

/// IT-6: `clr scope --help` exits 0; stdout contains `scope` and `--dir`.
#[ test ]
fn it6_scope_help_exits_zero()
{
  let out = run_cli( &[ "scope", "--help" ] );
  assert!( out.status.success(), "`clr scope --help` failed: {:?}", out.status );
  let stdout = stdout_str( &out );
  assert!( stdout.contains( "scope" ), "help must mention `scope`. Got:\n{stdout}" );
  assert!( stdout.contains( "--dir" ), "help must mention `--dir`. Got:\n{stdout}" );
}

// ── IT-7: -h short flag ───────────────────────────────────────────────────────

/// IT-7: `clr scope -h` exits 0; stdout contains `scope`.
#[ test ]
fn it7_scope_h_flag_exits_zero()
{
  let out = run_cli( &[ "scope", "-h" ] );
  assert!( out.status.success(), "`clr scope -h` failed: {:?}", out.status );
  let stdout = stdout_str( &out );
  assert!( stdout.contains( "scope" ), "help must mention `scope`. Got:\n{stdout}" );
}

// ── IT-8: `clr --help` mentions scope ────────────────────────────────────────

/// IT-8: `clr --help` mentions the `scope` command.
#[ test ]
fn it8_main_help_mentions_scope()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "`clr --help` failed: {:?}", out.status );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "scope" ),
    "`clr --help` must list `scope`. Got:\n{stdout}"
  );
}

// ── IT-9: Nonexistent --dir exits 1 ───────────────────────────────────────────

/// IT-9: `clr scope --dir /tmp/nonexistent_...` exits 1.
#[ test ]
fn it9_nonexistent_dir_exits_one()
{
  let tmp      = tempfile::TempDir::new().unwrap();
  let path     = tmp.path().to_owned();
  drop( tmp );
  let path_str = path.to_string_lossy();
  let out      = run_cli( &[ "scope", "--dir", &path_str ] );
  assert_eq!(
    exit_code( &out ),
    1,
    "`clr scope` with nonexistent `--dir` must exit 1. Got: {:?}", out.status
  );
}

// ── US-1: `clr scope` prints 6 vars for CWD ──────────────────────────────────

/// US-1: `clr scope` (no args) prints 6 vars for CWD; non-session vars have non-empty values.
#[ test ]
fn us1_scope_no_args_prints_six_vars()
{
  let out = run_cli( &[ "scope" ] );
  assert!( out.status.success(), "`clr scope` failed: {:?}", out.status );
  let stdout = stdout_str( &out );
  for var in &VARS
  {
    assert!( stdout.contains( var ), "stdout must contain `{var}`.\nGot:\n{stdout}" );
  }
  for var in &[ "CLAUDE_HOME", "CLAUDE_PROJECTS_DIR", "CLAUDE_SESSION_DIR", "CLAUDE_MEMORY_DIR", "CLAUDE_MEMORY_FILE" ]
  {
    let line = stdout.lines()
      .find( | l | l.starts_with( &format!( "{var}=" ) ) )
      .unwrap_or_else( || panic!( "{var} not found in output" ) );
    let value = line.trim_start_matches( &format!( "{var}=" ) );
    assert!( !value.is_empty(), "{var} must have a non-empty value. Got: `{line}`" );
  }
}

// ── US-2: --dir prints vars for given directory ───────────────────────────────

/// US-2: `clr scope --dir <path>` prints all 6 vars for that directory.
#[ test ]
fn us2_dir_flag_prints_vars_for_given_dir()
{
  let dir = tempfile::TempDir::new().expect( "tmpdir" );
  let dir_str = dir.path().to_str().expect( "utf-8" );
  let out = run_cli( &[ "scope", "--dir", dir_str ] );
  assert!( out.status.success(), "`clr scope --dir` failed: {:?}", out.status );
  let stdout = stdout_str( &out );
  for var in &VARS
  {
    assert!( stdout.contains( var ), "stdout must contain `{var}`.\nGot:\n{stdout}" );
  }
  assert!(
    stdout.lines().any( | l | l.starts_with( "CLAUDE_SESSION_DIR=" ) ),
    "CLAUDE_SESSION_DIR must be in output.\nGot:\n{stdout}"
  );
}

// ── US-3: CLAUDE_SESSION_FILE populated when session file exists ───────────────

/// US-3: `CLAUDE_SESSION_FILE` has a non-empty `.jsonl` path when a session exists.
///
/// Creates the session `.jsonl` in `scope_for(proj_dir).claude_session_dir`
/// by controlling `CLAUDE_HOME` via env override.
#[ test ]
fn us3_session_file_populated_when_session_exists()
{
  let claude_home = tempfile::TempDir::new().expect( "tmpdir" );
  let home_str    = claude_home.path().to_str().expect( "utf-8" );
  let proj = claude_home.path().join( "proj" );
  std::fs::create_dir_all( &proj ).expect( "mkdir proj" );
  let proj_str = proj.to_str().expect( "utf-8" );
  // Place .jsonl in the computed session dir for `proj`
  let encoded     = df( proj_str );
  let session_dir = claude_home.path().join( "projects" ).join( &encoded );
  std::fs::create_dir_all( &session_dir ).expect( "mkdir session_dir" );
  let uuid = "aabbccdd-1122-3344-5566-778899aabbcc";
  std::fs::write( session_dir.join( format!( "{uuid}.jsonl" ) ), b"{}" )
    .expect( "write session" );

  let out = run_cli_with_env(
    &[ "scope", "--dir", proj_str ],
    &[ ( "CLAUDE_HOME", home_str ) ],
  );
  assert!( out.status.success(), "`clr scope` failed: {:?}", out.status );
  let stdout = stdout_str( &out );
  let line = stdout.lines()
    .find( | l | l.starts_with( "CLAUDE_SESSION_FILE=" ) )
    .expect( "CLAUDE_SESSION_FILE must be present" );
  let value = line.trim_start_matches( "CLAUDE_SESSION_FILE=" );
  assert!( !value.is_empty(), "CLAUDE_SESSION_FILE must be non-empty. Got: `{line}`" );
  assert!(
    std::path::Path::new( value ).extension().is_some_and( | e | e.eq_ignore_ascii_case( "jsonl" ) ),
    "CLAUDE_SESSION_FILE must end with `.jsonl`. Got: `{value}`"
  );
}

// ── US-4: CLAUDE_SESSION_FILE empty when no session history ───────────────────

/// US-4: `CLAUDE_SESSION_FILE=` has empty value when no session files exist.
#[ test ]
fn us4_session_file_empty_when_no_history()
{
  let claude_home = tempfile::TempDir::new().expect( "tmpdir" );
  let home_str    = claude_home.path().to_str().expect( "utf-8" );
  let proj = claude_home.path().join( "proj" );
  std::fs::create_dir_all( &proj ).expect( "mkdir" );
  let proj_str = proj.to_str().expect( "utf-8" );
  let out = run_cli_with_env(
    &[ "scope", "--dir", proj_str ],
    &[ ( "CLAUDE_HOME", home_str ) ],
  );
  assert!( out.status.success(), "`clr scope` failed: {:?}", out.status );
  let stdout = stdout_str( &out );
  let line = stdout.lines()
    .find( | l | l.starts_with( "CLAUDE_SESSION_FILE=" ) )
    .expect( "CLAUDE_SESSION_FILE must be present" );
  let value = line.trim_start_matches( "CLAUDE_SESSION_FILE=" );
  assert!( value.is_empty(), "CLAUDE_SESSION_FILE must be empty when no sessions. Got: `{value}`" );
}

// ── US-5: CLAUDE_HOME override reflected in all 6 variables ──────────────────

/// US-5: `CLAUDE_HOME` override is reflected in all path-derived variables.
#[ test ]
fn us5_claude_home_override_in_path_vars()
{
  let claude_home = tempfile::TempDir::new().expect( "tmpdir" );
  let home_str    = claude_home.path().to_str().expect( "utf-8" );
  let proj = claude_home.path().join( "proj" );
  std::fs::create_dir_all( &proj ).expect( "mkdir" );
  let proj_str = proj.to_str().expect( "utf-8" );
  let out = run_cli_with_env(
    &[ "scope", "--dir", proj_str ],
    &[ ( "CLAUDE_HOME", home_str ) ],
  );
  assert!( out.status.success(), "`clr scope` failed: {:?}", out.status );
  let stdout = stdout_str( &out );
  let home_line = stdout.lines()
    .find( | l | l.starts_with( "CLAUDE_HOME=" ) )
    .expect( "CLAUDE_HOME must be present" );
  assert_eq!(
    home_line,
    &format!( "CLAUDE_HOME={home_str}" ),
    "CLAUDE_HOME must equal override"
  );
  for path_var in &[ "CLAUDE_PROJECTS_DIR", "CLAUDE_SESSION_DIR" ]
  {
    let line = stdout.lines()
      .find( | l | l.starts_with( &format!( "{path_var}=" ) ) )
      .unwrap_or_else( || panic!( "{path_var} must be in output" ) );
    assert!(
      line.contains( home_str ),
      "{path_var} must contain override path. Got: `{line}`"
    );
  }
}

// ── US-6: CLAUDE_COWORK_MEMORY_PATH_OVERRIDE reflected in memory vars ─────────

/// US-6: `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE` is reflected in memory variables.
///
/// `CLAUDE_MEMORY_DIR` must equal the override; `CLAUDE_MEMORY_FILE` must be
/// `<override>/MEMORY.md`; `CLAUDE_SESSION_DIR` still uses normal `Df()` derivation.
#[ test ]
fn us6_memory_override_reflected()
{
  let mem_dir  = tempfile::TempDir::new().expect( "tmpdir" );
  let mem_str  = mem_dir.path().to_str().expect( "utf-8" );
  let proj_dir = tempfile::TempDir::new().expect( "tmpdir" );
  let proj_str = proj_dir.path().to_str().expect( "utf-8" );
  let out = run_cli_with_env(
    &[ "scope", "--dir", proj_str ],
    &[ ( "CLAUDE_COWORK_MEMORY_PATH_OVERRIDE", mem_str ) ],
  );
  assert!( out.status.success(), "`clr scope` failed: {:?}", out.status );
  let stdout = stdout_str( &out );
  let mem_line = stdout.lines()
    .find( | l | l.starts_with( "CLAUDE_MEMORY_DIR=" ) )
    .expect( "CLAUDE_MEMORY_DIR must be present" );
  assert_eq!(
    mem_line,
    &format!( "CLAUDE_MEMORY_DIR={mem_str}" ),
    "CLAUDE_MEMORY_DIR must equal override. Got: `{mem_line}`"
  );
  let memfile_line = stdout.lines()
    .find( | l | l.starts_with( "CLAUDE_MEMORY_FILE=" ) )
    .expect( "CLAUDE_MEMORY_FILE must be present" );
  assert_eq!(
    memfile_line,
    &format!( "CLAUDE_MEMORY_FILE={mem_str}/MEMORY.md" ),
    "CLAUDE_MEMORY_FILE must point to override/MEMORY.md. Got: `{memfile_line}`"
  );
}

// ── US-7: Output is eval-safe key=value format ────────────────────────────────

/// US-7: Each output line is `KEY=value` format with uppercase key; exactly 6 lines.
#[ test ]
fn us7_output_is_eval_safe_format()
{
  let out = run_cli( &[ "scope" ] );
  assert!( out.status.success(), "`clr scope` failed: {:?}", out.status );
  let stdout = stdout_str( &out );
  let lines : Vec<&str> = stdout.lines().collect();
  assert_eq!( lines.len(), 6, "Output must have exactly 6 lines. Got:\n{stdout}" );
  for line in &lines
  {
    assert!(
      line.contains( '=' ),
      "Each line must be `KEY=VALUE`. Got: `{line}`"
    );
    let key = line.split( '=' ).next().unwrap();
    assert!(
      key.chars().all( | c | c.is_ascii_uppercase() || c == '_' ),
      "Key must be `[A-Z_]+`. Got: `{key}`"
    );
  }
}

// ── US-8: Exit codes ──────────────────────────────────────────────────────────

/// US-8: Exit 0 on success; exit 1 when `--dir` path does not exist.
///
/// stderr must mention the path or an error indicator for the failure case.
#[ test ]
fn us8_exit_codes()
{
  let ok = run_cli( &[ "scope", "--dir", "/tmp" ] );
  assert!( ok.status.success(), "`clr scope --dir /tmp` must exit 0" );

  let tmp      = tempfile::TempDir::new().unwrap();
  let path     = tmp.path().to_owned();
  drop( tmp );
  let path_str = path.to_string_lossy();
  let fail     = run_cli( &[ "scope", "--dir", &path_str ] );
  assert_eq!(
    exit_code( &fail ),
    1,
    "`clr scope` with nonexistent dir must exit 1. Got: {:?}", fail.status
  );
  let err = stderr_str( &fail );
  assert!(
    !err.is_empty(),
    "stderr must contain an error message. Got:\n{err}"
  );
}

// ── Shared helper: Df() path encoder ──────────────────────────────────────────

/// Encode a path using the `Df()` algorithm from `algorithm/001_path_encoding.md`.
///
/// - Strip leading `/`
/// - Split on `/`
/// - Replace `_` with `-` per component
/// - Prepend `-`; use `--` separator for hyphen-leading components
// BUG-366 ../../../task/claude_storage_core/bug/unverified/366_encode_path_dot_handling_divergence.md — duplicate hand-rolled encoder shares encode_path()'s dot-blind, no-length-fallback bug; needs sweeping once the fix lands
fn df( path : &str ) -> String
{
  let stripped = path.trim_start_matches( '/' );
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
