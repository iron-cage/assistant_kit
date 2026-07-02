//! JSON config loading integration tests.
//!
//! Covers JC-1..JC-10 from `tests/docs/feature/004_json_config.md` and
//! AF-1..AF-6 from `tests/docs/cli/param/075_args_file.md`.

#![ cfg( feature = "enabled" ) ]

use std::io::Write as _;
use tempfile::NamedTempFile;

mod cli_binary_test_helpers;
use cli_binary_test_helpers::
{
  exit_code,
  make_creds_file,
  run_cli,
  run_cli_with_env,
  run_with_path,
  stderr_str,
  stdout_str,
};
#[ cfg( unix ) ]
use cli_binary_test_helpers::fake_claude_dir;

// ── Local helper ─────────────────────────────────────────────────────────────

/// Write `content` to a new `NamedTempFile` and return it.
///
/// Caller must keep the returned file alive for the duration of the test;
/// dropping it deletes the file from disk.
///
/// # Panics
///
/// Panics if the temp file cannot be created or written.
fn write_json_file( content : &str ) -> NamedTempFile
{
  let mut f = NamedTempFile::new().expect( "create temp json file" );
  f.write_all( content.as_bytes() ).expect( "write json content" );
  f
}

// ── JC-1: --args-file loads JSON and applies params ──────────────────────────

/// JC-1: `--args-file` loads JSON and applies model param to the dry-run output.
///
/// Source: tests/docs/feature/004_json_config.md#jc-1
#[ test ]
fn jc1_args_file_loads_json_params()
{
  let cfg  = write_json_file( r#"{"model":"claude-haiku-4-5-20251001"}"# );
  let path = cfg.path().to_str().unwrap();
  let out  = run_cli( &[ "--args-file", path, "--dry-run", "task" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "claude-haiku-4-5-20251001" ),
    "dry-run must include model from JSON. Got:\n{stdout}"
  );
}

// ── JC-2: CLI flag overrides JSON config value ────────────────────────────────

/// JC-2: CLI `--model` wins over JSON `{"model":"claude-haiku-4-5-20251001"}`.
///
/// Source: tests/docs/feature/004_json_config.md#jc-2
#[ test ]
fn jc2_cli_flag_overrides_json_value()
{
  let cfg  = write_json_file( r#"{"model":"claude-haiku-4-5-20251001"}"# );
  let path = cfg.path().to_str().unwrap();
  let out  = run_cli(
    &[ "--args-file", path, "--model", "claude-opus-4-6", "--dry-run", "task" ]
  );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "claude-opus-4-6" ),
    "CLI model must appear in dry-run. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "claude-haiku-4-5-20251001" ),
    "JSON model must NOT appear when CLI model is set. Got:\n{stdout}"
  );
}

// ── JC-3: JSON config overrides CLR_* env var ────────────────────────────────

/// JC-3: JSON `{"model":"claude-haiku-4-5-20251001"}` wins over `CLR_MODEL=claude-opus-4-6`.
///
/// Source: tests/docs/feature/004_json_config.md#jc-3
#[ test ]
fn jc3_json_overrides_clr_env_var()
{
  let cfg  = write_json_file( r#"{"model":"claude-haiku-4-5-20251001"}"# );
  let path = cfg.path().to_str().unwrap();
  let out  = run_cli_with_env(
    &[ "--args-file", path, "--dry-run", "task" ],
    &[ ( "CLR_MODEL", "claude-opus-4-6" ) ],
  );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "claude-haiku-4-5-20251001" ),
    "JSON model must appear (wins over CLR_MODEL). Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "claude-opus-4-6" ),
    "CLR_MODEL must not win when JSON overrides it. Got:\n{stdout}"
  );
}

// ── JC-4: CLR_ARGS_FILE env var equivalent to --args-file ────────────────────

/// JC-4: `CLR_ARGS_FILE` env var triggers the same JSON loading as `--args-file`.
///
/// Source: tests/docs/feature/004_json_config.md#jc-4
#[ test ]
fn jc4_clr_args_file_env_var()
{
  let cfg  = write_json_file( r#"{"model":"claude-haiku-4-5-20251001"}"# );
  let path = cfg.path().to_str().unwrap();
  let out  = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_ARGS_FILE", path ) ],
  );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0 from CLR_ARGS_FILE; stderr: {}", stderr_str( &out )
  );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "claude-haiku-4-5-20251001" ),
    "CLR_ARGS_FILE must load JSON params. Got:\n{stdout}"
  );
}

// ── JC-5: Stdin JSON pipe detected when stdin is not a TTY ───────────────────

/// JC-5: JSON piped to stdin is auto-detected when stdin is not a TTY.
///
/// Spawns the binary with `Stdio::piped()` to simulate non-TTY stdin, writes
/// JSON, and confirms the JSON-sourced model appears in dry-run output.
///
/// Source: tests/docs/feature/004_json_config.md#jc-5
#[ test ]
fn jc5_stdin_json_pipe_detected()
{
  // Container guard — mirrors assert_container() in cli_binary_test_helpers.
  let in_container = std::path::Path::new( "/.dockerenv" ).exists()
    || std::path::Path::new( "/run/.containerenv" ).exists()
    || std::env::var( "RUNBOX_CONTAINER" ).as_deref() == Ok( "1" );
  let escaped = std::env::var( "VERB_LAYER" ).as_deref() == Ok( "l0" );
  assert!(
    in_container || escaped,
    "\n\nTests must run inside a container.\n\
     Standard invocation: cd module/claude_profile && ./verb/test\n\
     Host bypass:         VERB_LAYER=l0 cargo nextest run --all-features\n"
  );

  let bin  = env!( "CARGO_BIN_EXE_clr" );
  let json = b"{\"model\":\"claude-haiku-4-5-20251001\"}";

  let mut child = std::process::Command::new( bin )
    .args( [ "--dry-run", "task" ] )
    .stdin(  std::process::Stdio::piped() )
    .stdout( std::process::Stdio::piped() )
    .stderr( std::process::Stdio::piped() )
    .env_remove( "CLR_DIR" )
    .env_remove( "CLR_SESSION_DIR" )
    .spawn()
    .expect( "spawn clr for stdin JSON test" );

  child.stdin.take().unwrap().write_all( json ).expect( "write stdin JSON" );
  let out = child.wait_with_output().expect( "wait for clr" );

  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0 from stdin JSON; stderr: {}", stderr_str( &out )
  );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "claude-haiku-4-5-20251001" ),
    "stdin JSON model must appear in dry-run output. Got:\n{stdout}"
  );
}

// ── JC-6: Invalid JSON → exit 1 with parse error ─────────────────────────────

/// JC-6: Malformed JSON in `--args-file` → exit 1; stderr contains an error.
///
/// Source: tests/docs/feature/004_json_config.md#jc-6
#[ test ]
fn jc6_invalid_json_exit_1()
{
  let cfg  = write_json_file( "{model: bad}" ); // not valid JSON
  let path = cfg.path().to_str().unwrap();
  let out  = run_cli( &[ "--args-file", path, "task" ] );
  assert_eq!(
    exit_code( &out ), 1,
    "expected exit 1 for invalid JSON; got {}; stderr: {}", exit_code( &out ), stderr_str( &out )
  );
  let stderr = stderr_str( &out );
  assert!(
    stderr.to_lowercase().contains( "json" ) || stderr.to_lowercase().contains( "parse" ),
    "stderr must mention JSON parse error. Got:\n{stderr}"
  );
}

// ── JC-7: Non-existent --args-file path → exit 1 ─────────────────────────────

/// JC-7: `--args-file` pointing to a missing file → exit 1 before subprocess spawn.
///
/// Source: tests/docs/feature/004_json_config.md#jc-7
#[ test ]
fn jc7_missing_args_file_exit_1()
{
  let out = run_cli(
    &[ "--args-file", "/tmp/nonexistent_clr_config_xyz_jc7.json", "task" ]
  );
  assert_eq!(
    exit_code( &out ), 1,
    "expected exit 1 for missing file; got {}; stderr: {}", exit_code( &out ), stderr_str( &out )
  );
  assert!(
    !stderr_str( &out ).is_empty(),
    "stderr must contain a file-not-found error"
  );
}

// ── JC-8: Boolean `true` in JSON activates flag ──────────────────────────────

/// JC-8: JSON `{"dry-run": true}` activates dry-run without the CLI flag.
///
/// With dry-run active, the binary exits 0 and prints a command preview
/// to stdout instead of spawning a subprocess.
///
/// Source: tests/docs/feature/004_json_config.md#jc-8
#[ test ]
fn jc8_boolean_true_activates_flag()
{
  let cfg  = write_json_file( r#"{"dry-run": true}"# );
  let path = cfg.path().to_str().unwrap();
  let out  = run_cli( &[ "--args-file", path, "task" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "JSON dry-run:true must exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    !stdout_str( &out ).is_empty(),
    "dry-run mode from JSON must print command preview to stdout"
  );
}

// ── JC-9: Unknown JSON key silently ignored ───────────────────────────────────

/// JC-9: An unknown JSON key is silently ignored; the known key is still applied.
///
/// Source: tests/docs/feature/004_json_config.md#jc-9
#[ test ]
fn jc9_unknown_json_key_ignored()
{
  let cfg  = write_json_file(
    r#"{"_future_param": "x", "model": "claude-haiku-4-5-20251001"}"#
  );
  let path = cfg.path().to_str().unwrap();
  let out  = run_cli( &[ "--args-file", path, "--dry-run", "task" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "unknown JSON key must not cause error; stderr: {}", stderr_str( &out )
  );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "claude-haiku-4-5-20251001" ),
    "known model key must still apply. Got:\n{stdout}"
  );
}

// ── JC-10: JSON config applies to `isolated` subcommand ──────────────────────

/// JC-10: `CLR_ARGS_FILE` applies JSON config params to the `isolated` subcommand.
///
/// Uses `{"dir": "/tmp"}` which is visible in isolated's dry-run output.
/// Note: isolated's `--dry-run` (via `emit_credential_trace`) writes to stderr,
/// not stdout — so verification checks `stderr_str`.
///
/// Source: tests/docs/feature/004_json_config.md#jc-10
#[ test ]
fn jc10_json_config_applies_to_isolated()
{
  let cfg       = write_json_file( r#"{"dir": "/tmp"}"# );
  let cfg_path  = cfg.path().to_str().unwrap();
  let creds     = make_creds_file( "{}" );
  let creds_path = creds.path().to_str().unwrap();
  let out       = run_cli_with_env(
    &[ "isolated", "--creds", creds_path, "--dry-run" ],
    &[ ( "CLR_ARGS_FILE", cfg_path ) ],
  );
  assert_eq!(
    exit_code( &out ), 0,
    "isolated must exit 0 with CLR_ARGS_FILE; stderr: {}", stderr_str( &out )
  );
  // isolated --dry-run emits preview to stdout (to_stdout=true in emit_credential_trace)
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "/tmp" ),
    "isolated dry-run trace must show --dir /tmp from JSON config. Got:\n{stdout}"
  );
}

// ── AF-1: --args-file flag accepted; JSON params loaded ──────────────────────

/// AF-1: `--args-file` flag is accepted and JSON params are loaded into the run.
///
/// Source: tests/docs/cli/param/075_args_file.md#af-1
#[ test ]
fn af1_args_file_accepted()
{
  let cfg  = write_json_file( r#"{"model": "claude-haiku-4-5-20251001"}"# );
  let path = cfg.path().to_str().unwrap();
  let out  = run_cli( &[ "--args-file", path, "--dry-run", "task" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "claude-haiku-4-5-20251001" ),
    "--args-file must load JSON params. Got:\n{}", stdout_str( &out )
  );
}

// ── AF-2: CLR_ARGS_FILE env var fallback ─────────────────────────────────────

/// AF-2: `CLR_ARGS_FILE` env var loads JSON params without `--args-file` on CLI.
///
/// Source: tests/docs/cli/param/075_args_file.md#af-2
#[ test ]
fn af2_clr_args_file_env_var_fallback()
{
  let cfg  = write_json_file( r#"{"model": "claude-haiku-4-5-20251001"}"# );
  let path = cfg.path().to_str().unwrap();
  let out  = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_ARGS_FILE", path ) ],
  );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0 from CLR_ARGS_FILE; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "claude-haiku-4-5-20251001" ),
    "CLR_ARGS_FILE must load JSON params. Got:\n{}", stdout_str( &out )
  );
}

// ── AF-3: --args-file + --dry-run shows JSON-sourced params in preview ────────

/// AF-3: `--args-file` with `--dry-run` shows JSON-sourced model in preview output.
///
/// Source: tests/docs/cli/param/075_args_file.md#af-3
#[ test ]
fn af3_args_file_dry_run_shows_json_params()
{
  let cfg  = write_json_file( r#"{"model": "claude-haiku-4-5-20251001"}"# );
  let path = cfg.path().to_str().unwrap();
  let out  = run_cli( &[ "--args-file", path, "--dry-run", "task" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "--model" ),
    "dry-run output must include --model flag. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "claude-haiku-4-5-20251001" ),
    "dry-run output must show JSON-sourced model value. Got:\n{stdout}"
  );
}

// ── AF-4: Boolean `true` in JSON activates flag ──────────────────────────────

/// AF-4: JSON `{"dry-run": true}` activates dry-run mode (no CLI `--dry-run` needed).
///
/// Source: tests/docs/cli/param/075_args_file.md#af-4
#[ test ]
fn af4_boolean_true_activates_dry_run()
{
  let cfg  = write_json_file( r#"{"dry-run": true}"# );
  let path = cfg.path().to_str().unwrap();
  let out  = run_cli( &[ "--args-file", path, "task" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "JSON dry-run:true must exit 0; subprocess must not be spawned; stderr: {}",
    stderr_str( &out )
  );
  assert!(
    !stdout_str( &out ).is_empty(),
    "dry-run mode from JSON must emit preview to stdout"
  );
}

// ── AF-5: Unknown JSON key silently ignored ───────────────────────────────────

/// AF-5: An unknown JSON key produces no error; known keys are applied normally.
///
/// Source: tests/docs/cli/param/075_args_file.md#af-5
#[ test ]
fn af5_unknown_json_key_no_error()
{
  let cfg  = write_json_file(
    r#"{"_unknown_future_key": 42, "model": "claude-haiku-4-5-20251001"}"#
  );
  let path = cfg.path().to_str().unwrap();
  let out  = run_cli( &[ "--args-file", path, "--dry-run", "task" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "unknown JSON key must not cause error; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "claude-haiku-4-5-20251001" ),
    "known model key must still apply. Got:\n{}", stdout_str( &out )
  );
}

// ── AF-6: --args-file path does not exist → exit 1 ───────────────────────────

/// AF-6: `--args-file` with a nonexistent path → exit 1 before subprocess spawn.
///
/// Source: tests/docs/cli/param/075_args_file.md#af-6
#[ test ]
fn af6_missing_args_file_exit_1()
{
  let out = run_cli(
    &[ "--args-file", "/tmp/nonexistent_args_file_test_xyz_af6.json", "task" ]
  );
  assert_eq!(
    exit_code( &out ), 1,
    "expected exit 1 for missing file; got {}; stderr: {}",
    exit_code( &out ), stderr_str( &out )
  );
  assert!(
    !stderr_str( &out ).is_empty(),
    "stderr must contain a file-not-found error. Got empty stderr"
  );
}

// ── JC-8b / AF-4b: JSON `false` is a no-op — subprocess spawned ─────────────

/// JC-8b / AF-4b: JSON `{"dry-run": false}` is a no-op; subprocess is spawned.
///
/// Proves the `false`-branch contract: `false` for any boolean param in JSON must not
/// activate dry-run or otherwise suppress subprocess invocation.  The fake claude emits
/// a recognisable token; its presence in stdout confirms the subprocess was invoked.
///
/// Source: tests/docs/feature/004_json_config.md#jc-8 (false-branch)
///         tests/docs/cli/param/075_args_file.md#af-4 (false-branch)
#[ cfg( unix ) ]
#[ test ]
fn jc8b_boolean_false_is_noop_subprocess_spawned()
{
  let cfg            = write_json_file( r#"{"dry-run": false}"# );
  let json_path      = cfg.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "echo 'jc8b_invoked'" );
  let out            = run_with_path( &[ "--args-file", json_path, "task" ], &path );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "jc8b_invoked" ),
    "JSON dry-run:false must not suppress subprocess — fake claude output must appear. Got:\n{stdout}"
  );
}
