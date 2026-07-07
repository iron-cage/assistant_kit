//! Unix-only integration tests.
#![ cfg( unix ) ]
//! User Story Integration Tests — Output, Effort, Context, Subdir, Concurrency (US19–US25)
//!
//! ## Purpose
//!
//! End-to-end workflow tests implementing specs from `tests/docs/cli/user_story/`
//! for US19 through US25.  Split from `user_story_test.rs` to stay under the
//! 1000-line soft limit.
//!
//! ## Strategy
//!
//! Most cases use `--dry-run` to inspect the assembled command without spawning a
//! Claude subprocess.  US23 and US24 use a fake `claude` binary to exercise the
//! real subprocess path without a live API connection.
//!
//! ## Doc Comment Convention
//!
//! Clippy `doc_markdown` lint flags `SCREAMING_SNAKE_CASE` identifiers and
//! `IDENT=value` patterns in `///` doc comments that are not wrapped in backticks.
//! All `CLR_*` env var names and `IDENT=value` patterns must use backticks.
//!
//! ## Test Matrix
//!
//! | Spec | Story | Cases | Method |
//! |------|-------|-------|--------|
//! | 19 | MCP Config Injection | US-1..4 | dry-run / env var |
//! | 20 | Suppress Effort Max | US-1..4 | dry-run / env var |
//! | 21 | Keep ClaudeCode Context | US-1..4 | dry-run / env var |
//! | 22 | Session Isolation via Subdirectory | US-1..5 | dry-run / env var |
//! | 23 | Output File Capture | US-1..4 | dry-run / fake claude / error path |
//! | 24 | Enum Output Validation | US-1..4 | fake claude / parse error |
//! | 25 | Session Concurrency Gate | US-1..6 | dry-run / env var / `$CLR_PROC_DIR` fixture |

#![ cfg( feature = "enabled" ) ]

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env, run_dry };

#[ cfg( unix ) ]
use std::os::unix::fs::PermissionsExt as _;

#[ cfg( unix ) ]
use cli_binary_test_helpers::{
  fake_claude_dir, fake_claude_binary_dir, spawn_fake_claude, spawn_print_claude, make_proc_dir,
};

// ── helpers ──────────────────────────────────────────────────────────────────

/// Create a fake `claude` binary that prints `text` to stdout and exits 0.
///
/// Returns `(tempdir, modified_path)` — keep `tempdir` alive for the test duration.
#[ cfg( unix ) ]
fn make_fake_claude_for_us( text : &str ) -> ( tempfile::TempDir, String )
{
  let dir  = tempfile::TempDir::new().expect( "create temp dir for fake claude" );
  let bin  = dir.path().join( "claude" );
  let src  = format!( "#!/bin/sh\nprintf '%s\\n' '{}'\n", text.replace( '\'', "'\\''" ) );
  std::fs::write( &bin, src.as_bytes() ).expect( "write fake claude" );
  let mut perms = std::fs::metadata( &bin ).expect( "metadata" ).permissions();
  perms.set_mode( 0o755 );
  std::fs::set_permissions( &bin, perms ).expect( "chmod fake claude" );
  let dir_str = dir.path().to_str().expect( "UTF-8 dir path" ).to_string();
  let orig    = std::env::var( "PATH" ).unwrap_or_default();
  let path    = format!( "{dir_str}:{orig}" );
  ( dir, path )
}

// ── US19: MCP Config Injection ───────────────────────────────────────────────
// Source: tests/docs/cli/user_story/19_mcp_config_injection.md

/// US-1: --mcp-config path appears in the assembled command.
#[ test ]
fn us19_1_mcp_config_in_command()
{
  let output = run_dry( &[ "--mcp-config", "/tmp/mcp.json", "Fix bug" ] );
  assert!(
    output.contains( "--mcp-config /tmp/mcp.json" ),
    "--mcp-config /tmp/mcp.json must appear in dry-run output. Got:\n{output}"
  );
}

/// US-2: multiple --mcp-config flags are each forwarded individually.
#[ test ]
fn us19_2_multiple_mcp_configs_forwarded()
{
  let output = run_dry( &[
    "--mcp-config", "/tmp/us19a.json",
    "--mcp-config", "/tmp/us19b.json",
    "Fix bug",
  ] );
  assert!(
    output.contains( "--mcp-config /tmp/us19a.json" ),
    "first --mcp-config must appear. Got:\n{output}"
  );
  assert!(
    output.contains( "--mcp-config /tmp/us19b.json" ),
    "second --mcp-config must appear. Got:\n{output}"
  );
}

/// US-3: `CLR_MCP_CONFIG` env var sets a single config when flag is absent.
#[ test ]
fn us19_3_env_var_sets_mcp_config()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "Fix bug" ],
    &[ ( "CLR_MCP_CONFIG", "/tmp/us19env.json" ) ],
  );
  assert!( out.status.success(), "CLR_MCP_CONFIG must be accepted. Got: {:?}", out.status.code() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--mcp-config /tmp/us19env.json" ),
    "CLR_MCP_CONFIG must inject --mcp-config. Got:\n{stdout}"
  );
}

/// US-4: no --mcp-config flag appears by default (absent when not specified).
#[ test ]
fn us19_4_no_mcp_config_by_default()
{
  let output = run_dry( &[ "Fix bug" ] );
  assert!(
    !output.contains( "--mcp-config" ),
    "no --mcp-config must appear when flag is absent. Got:\n{output}"
  );
}

// ── US20: Suppress Effort Max ────────────────────────────────────────────────
// Source: tests/docs/cli/user_story/20_suppress_effort_max.md

/// US-1: default assembled command includes --effort max.
#[ test ]
fn us20_1_default_has_effort_max()
{
  let output = run_dry( &[ "Fix bug" ] );
  assert!(
    output.contains( "--effort max" ),
    "default run must inject --effort max. Got:\n{output}"
  );
}

/// US-2: --no-effort-max suppresses all --effort injection.
#[ test ]
fn us20_2_no_effort_max_suppresses_injection()
{
  let output = run_dry( &[ "--no-effort-max", "Fix bug" ] );
  assert!(
    !output.contains( "--effort" ),
    "--no-effort-max must suppress all --effort flags. Got:\n{output}"
  );
}

/// US-3: --no-effort-max wins over an explicit --effort flag.
#[ test ]
fn us20_3_no_effort_max_wins_over_effort()
{
  let output = run_dry( &[ "--no-effort-max", "--effort", "medium", "Fix bug" ] );
  assert!(
    !output.contains( "--effort" ),
    "--no-effort-max must suppress --effort even when --effort is explicit. Got:\n{output}"
  );
}

/// US-4: `CLR_NO_EFFORT_MAX=1` suppresses effort injection via env var.
#[ test ]
fn us20_4_env_var_suppresses_effort()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "Fix bug" ],
    &[ ( "CLR_NO_EFFORT_MAX", "1" ) ],
  );
  assert!( out.status.success(), "CLR_NO_EFFORT_MAX=1 must exit 0. Got: {:?}", out.status.code() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--effort" ),
    "CLR_NO_EFFORT_MAX=1 must suppress --effort. Got:\n{stdout}"
  );
}

// ── US21: Keep ClaudeCode Context ────────────────────────────────────────────
// Source: tests/docs/cli/user_story/21_keep_claudecode_context.md

/// US-1: --keep-claudecode flag accepted; command assembles without error.
#[ test ]
fn us21_1_flag_accepted()
{
  let output = run_dry( &[ "--keep-claudecode", "Fix bug" ] );
  assert!(
    output.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=" ),
    "--keep-claudecode must assemble a valid command. Got:\n{output}"
  );
}

/// US-2: `CLR_KEEP_CLAUDECODE=1` env var accepted; command assembles.
#[ test ]
fn us21_2_env_var_1_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "Fix bug" ],
    &[ ( "CLR_KEEP_CLAUDECODE", "1" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_KEEP_CLAUDECODE=1 must exit 0. Got: {:?}",
    out.status.code()
  );
}

/// US-3: `CLR_KEEP_CLAUDECODE=true` env var accepted; command assembles.
#[ test ]
fn us21_3_env_var_true_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "Fix bug" ],
    &[ ( "CLR_KEEP_CLAUDECODE", "true" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_KEEP_CLAUDECODE=true must exit 0. Got: {:?}",
    out.status.code()
  );
}

/// US-4: `CLR_KEEP_CLAUDECODE=yes` silently rejected; exit 0 (not a hard error).
#[ test ]
fn us21_4_env_var_yes_silently_rejected()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "Fix bug" ],
    &[ ( "CLR_KEEP_CLAUDECODE", "yes" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_KEEP_CLAUDECODE=yes must exit 0 (silently rejected). Got: {:?}",
    out.status.code()
  );
}

// ── US22: Session Isolation via Subdirectory ──────────────────────────────────
// Source: tests/docs/cli/user_story/22_session_isolation_subdir.md

/// US-1: `--subdir NAME` appends `/-NAME` to the effective working directory.
#[ test ]
fn us22_us1_subdir_name_appends_hyphen_name()
{
  let output = run_dry( &[ "--subdir", "build", "Fix bug" ] );
  assert!(
    output.contains( "/-build" ),
    "--subdir build must produce path ending in /-build. Got:\n{output}"
  );
}

/// US-2: `--subdir .` is identity — no `/-` suffix in effective dir.
#[ test ]
fn us22_us2_subdir_dot_identity()
{
  let output = run_dry( &[ "--subdir", ".", "Fix bug" ] );
  assert!(
    !output.contains( "/-" ),
    "--subdir . must not append any /-NAME suffix. Got:\n{output}"
  );
}

/// US-3: `CLR_SUBDIR=feature` env var accepted; effective dir ends in `/-feature`.
#[ test ]
fn us22_us3_clr_subdir_env_var()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "Fix bug" ],
    &[ ( "CLR_SUBDIR", "feature" ) ],
  );
  assert!( out.status.success(), "CLR_SUBDIR must exit 0. Got: {:?}", out.status.code() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "/-feature" ),
    "CLR_SUBDIR=feature must produce path ending in /-feature. Got:\n{stdout}"
  );
}

/// US-4: `CLR_SUBDIR=.` env var identity — no `/-` suffix in effective dir.
#[ test ]
fn us22_us4_clr_subdir_dot_identity()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "Fix bug" ],
    &[ ( "CLR_SUBDIR", "." ) ],
  );
  assert!( out.status.success(), "CLR_SUBDIR=. must exit 0. Got: {:?}", out.status.code() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "/-" ),
    "CLR_SUBDIR=. must not append any /-NAME suffix. Got:\n{stdout}"
  );
}

/// US-5: CLI `--subdir cliname` wins over `CLR_SUBDIR=envname`.
#[ test ]
fn us22_us5_cli_wins_over_env_var()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "--subdir", "cliname", "Fix bug" ],
    &[ ( "CLR_SUBDIR", "envname" ) ],
  );
  assert!( out.status.success(), "CLI --subdir with CLR_SUBDIR must exit 0: {:?}", out.status.code() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "/-cliname" ),
    "CLI --subdir cliname must win over CLR_SUBDIR=envname. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "/-envname" ),
    "CLR_SUBDIR=envname must be suppressed by CLI --subdir. Got:\n{stdout}"
  );
}

// ── US23: Output File Capture ───────────────────────────────────────────────
// Source: tests/docs/cli/user_story/23_output_file_capture.md

/// US23-1: `clr -p --output-file <tmp>` creates file; stdout and file content are identical.
///
/// Uses fake claude to emit deterministic text.
#[ test ]
#[ cfg( unix ) ]
fn us23_1_output_file_created_with_tee_content()
{
  let ( _dir, path ) = make_fake_claude_for_us( "hello_tee_test" );
  let tmp  = tempfile::NamedTempFile::new().expect( "create temp output file" );
  let dest = tmp.path().to_str().expect( "UTF-8 path" ).to_string();
  drop( tmp ); // let clr create it fresh

  let out = run_cli_with_env(
    &[ "-p", "--output-file", &dest, "--max-sessions", "0", "task" ],
    &[ ( "PATH", &path ) ],
  );
  assert!( out.status.success(), "US23-1: must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout_content  = String::from_utf8_lossy( &out.stdout ).to_string();
  let file_content    = std::fs::read_to_string( &dest ).expect( "US23-1: output file must exist" );
  assert_eq!(
    stdout_content, file_content,
    "US23-1: file content must equal stdout. stdout: {stdout_content:?}, file: {file_content:?}",
  );
  assert!( stdout_content.contains( "hello_tee_test" ), "US23-1: output must contain echoed text" );
}

/// US23-2: `--dry-run` with `--output-file` — file is NOT created.
#[ test ]
fn us23_2_dry_run_skips_output_file()
{
  let path = format!( "/tmp/us23_2_should_not_exist_{}.txt", std::process::id() );
  let _    = std::fs::remove_file( &path );
  let out  = run_cli( &[ "--dry-run", "--output-file", &path, "task" ] );
  assert!( out.status.success(), "US23-2: dry-run must exit 0" );
  assert!( !std::path::Path::new( &path ).exists(), "US23-2: dry-run must NOT create the output file" );
}

/// US23-3: non-writable path → exit 1, stderr contains path and OS error.
///
/// Uses fake claude so clr reaches the file-write step.
#[ test ]
#[ cfg( unix ) ]
fn us23_3_nonwritable_path_exits_1()
{
  let ( _dir, path ) = make_fake_claude_for_us( "text" );
  let bad = "/nonexistent_dir_us23_3/out.txt";
  let out = run_cli_with_env(
    &[ "-p", "--output-file", bad, "--max-sessions", "0", "task" ],
    &[ ( "PATH", &path ) ],
  );
  assert_eq!( out.status.code(), Some( 1 ), "US23-3: bad path must exit 1. Got: {:?}", out.status.code() );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!( stderr.contains( bad ), "US23-3: stderr must contain the bad path. Got:\n{stderr}" );
}

/// US23-4: `CLR_OUTPUT_FILE` env var — file created with captured output.
///
/// Uses fake claude so clr reaches the file-write step.
#[ test ]
#[ cfg( unix ) ]
fn us23_4_env_var_output_file_applied()
{
  let ( _dir, path ) = make_fake_claude_for_us( "env_var_text" );
  let tmp  = tempfile::NamedTempFile::new().expect( "create temp output file" );
  let dest = tmp.path().to_str().expect( "UTF-8 path" ).to_string();
  drop( tmp );

  let out = run_cli_with_env(
    &[ "-p", "--max-sessions", "0", "task" ],
    &[ ( "PATH", &path ), ( "CLR_OUTPUT_FILE", &dest ) ],
  );
  assert!( out.status.success(), "US23-4: CLR_OUTPUT_FILE must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let file_content = std::fs::read_to_string( &dest ).expect( "US23-4: output file must exist" );
  assert!( file_content.contains( "env_var_text" ), "US23-4: file must contain echoed text. Got: {file_content:?}" );
}

// ── US24: Enum Output Validation ────────────────────────────────────────────
// Source: tests/docs/cli/user_story/24_enum_output_validation.md

/// US24-1: output matches `--expect "yes|no"` (case-insensitive, trimmed) → exit 0.
#[ test ]
#[ cfg( unix ) ]
fn us24_1_expect_match_exits_0()
{
  let ( _dir, path ) = make_fake_claude_for_us( "yes" );
  let out = run_cli_with_env(
    &[ "-p", "--expect", "yes|no", "--max-sessions", "0", "task" ],
    &[ ( "PATH", &path ) ],
  );
  assert!( out.status.success(), "US24-1: matching output must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
}

/// US24-2: output does not match `--expect "yes|no"`, default strategy (fail) → exit 3.
#[ test ]
#[ cfg( unix ) ]
fn us24_2_expect_mismatch_exits_3()
{
  let ( _dir, path ) = make_fake_claude_for_us( "maybe" );
  let out = run_cli_with_env(
    &[ "-p", "--expect", "yes|no", "--max-sessions", "0", "task" ],
    &[ ( "PATH", &path ) ],
  );
  assert_eq!( out.status.code(), Some( 3 ), "US24-2: mismatch must exit 3. Got: {:?}", out.status.code() );
}

/// US24-3: `--expect-strategy default:no` on mismatch → stdout = "no", exit 0.
#[ test ]
#[ cfg( unix ) ]
fn us24_3_expect_strategy_default_prints_fallback()
{
  let ( _dir, path ) = make_fake_claude_for_us( "maybe" );
  let out = run_cli_with_env(
    &[ "-p", "--expect", "yes|no", "--expect-strategy", "default:no", "--max-sessions", "0", "task" ],
    &[ ( "PATH", &path ) ],
  );
  assert!( out.status.success(), "US24-3: default strategy must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert_eq!( stdout.trim(), "no", "US24-3: stdout must be fallback 'no'. Got: {stdout:?}" );
}

/// US24-4: parse error — `--expect-strategy bogus` → exit 1, stderr contains error.
#[ test ]
fn us24_4_invalid_strategy_exits_1()
{
  let out = run_cli( &[ "--expect", "yes|no", "--expect-strategy", "bogus", "task" ] );
  assert_eq!( out.status.code(), Some( 1 ), "US24-4: invalid strategy must exit 1. Got: {:?}", out.status.code() );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "Error" ) || stderr.contains( "invalid" ),
    "US24-4: stderr must contain error message. Got:\n{stderr}",
  );
}

// ── US25: Session Concurrency Gate ──────────────────────────────────────────
// Source: tests/docs/cli/user_story/25_concurrency_gate.md

/// US25-1: `--max-sessions 0` (unlimited) proceeds immediately; no waiting messages.
#[ test ]
fn us25_1_max_sessions_0_unlimited_no_wait()
{
  let out = run_cli( &[ "--max-sessions", "0", "--dry-run", "task" ] );
  assert!( out.status.success(), "US25-1: --max-sessions 0 must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "waiting" ) && !stderr.contains( "session" ),
    "US25-1: no waiting messages with --max-sessions 0. Got stderr:\n{stderr}",
  );
}

/// US25-2: `CLR_MAX_SESSIONS=N` env-var fallback — accepted, dry-run exits 0 immediately.
#[ test ]
fn us25_2_clr_max_sessions_env_var_applied()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_MAX_SESSIONS", "3" ) ],
  );
  assert!( out.status.success(), "US25-2: CLR_MAX_SESSIONS=3 + dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
}

/// US25-3: CLI `--max-sessions M` wins over `CLR_MAX_SESSIONS=N` — accepted, dry-run exits 0.
#[ test ]
fn us25_3_cli_max_sessions_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--max-sessions", "10", "--dry-run", "task" ],
    &[ ( "CLR_MAX_SESSIONS", "1" ) ],
  );
  assert!( out.status.success(), "US25-3: CLI --max-sessions wins over CLR_MAX_SESSIONS must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
}

/// US25-4: `--dry-run` bypasses the session gate — no counting, no waiting.
///
/// Even with `--max-sessions 1`, dry-run exits 0 immediately without checking /proc.
#[ test ]
fn us25_4_dry_run_bypasses_gate()
{
  let out = run_cli( &[ "--max-sessions", "1", "--dry-run", "task" ] );
  assert!( out.status.success(), "US25-4: --dry-run must bypass gate and exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "waiting" ),
    "US25-4: dry-run must not emit any waiting message. Got stderr:\n{stderr}",
  );
}

/// US25-5: `--interactive` bypasses the session gate entirely, regardless of active count.
///
/// Spawns 20 fake print-mode processes (visible via `$CLR_PROC_DIR`) and sets
/// `--max-sessions 1` — strict enough to block a print-mode invocation immediately.
/// An `--interactive` invocation must still proceed without any gate wait, since
/// interactive invocations skip the gate outright.
///
/// Source: `tests/docs/cli/user_story/025_concurrency_gate.md` US-5 / AC-007.
#[ cfg( unix ) ]
#[ test ]
fn us25_5_interactive_bypasses_gate_regardless_of_count()
{
  let ( _fake_dir, fake_path ) = fake_claude_binary_dir();
  let mut background : Vec< std::process::Child > = Vec::new();
  let mut pids : Vec< u32 > = Vec::new();
  for _ in 0 .. 20
  {
    let child = spawn_print_claude( &fake_path );
    pids.push( child.id() );
    background.push( child );
  }
  let proc = make_proc_dir( &pids );

  let ( _clr_claude_dir, clr_path ) = fake_claude_dir( "printf 'ok\\n'" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "--interactive", "--max-sessions", "1", "task" ] )
    .env( "HOME", "/tmp/clr-isolated-home" )
    .env( "PATH", &clr_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .output()
    .expect( "run clr --interactive" );

  for mut child in background { let _ = child.kill(); let _ = child.wait(); }

  assert!(
    out.status.success(),
    "US25-5: --interactive must exit 0 despite 20 active print sessions. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "waiting" ) && !stderr.contains( "sessions active" ),
    "US25-5: --interactive must skip the gate entirely — no wait messages. Got stderr:\n{stderr}"
  );
}

/// US25-6: the gate's active-session count excludes interactive sessions.
///
/// Spawns 5 fake interactive processes and 1 fake print-mode process (visible via
/// `$CLR_PROC_DIR`), sets `--max-sessions 2`, and issues a non-interactive (print-mode)
/// invocation. Only the 1 print-mode fake counts toward the limit — below 2 — so the
/// invocation proceeds immediately; the 5 interactive fakes are excluded from the count.
///
/// Source: `tests/docs/cli/user_story/025_concurrency_gate.md` US-6 / AC-008.
#[ cfg( unix ) ]
#[ test ]
fn us25_6_gate_count_excludes_interactive_sessions()
{
  let ( _fake_dir, fake_path ) = fake_claude_binary_dir();
  let mut background : Vec< std::process::Child > = Vec::new();
  let mut pids : Vec< u32 > = Vec::new();
  for _ in 0 .. 5
  {
    let child = spawn_fake_claude( &fake_path );
    pids.push( child.id() );
    background.push( child );
  }
  let print_child = spawn_print_claude( &fake_path );
  pids.push( print_child.id() );
  background.push( print_child );
  let proc = make_proc_dir( &pids );

  let ( _clr_claude_dir, clr_path ) = fake_claude_dir( "printf 'ok\\n'" );
  // Isolate the slot-reservation directory so this test's admission check never
  // collides with another concurrently-running test's slot files in the shared
  // system-default gate dir (BUG-387's reservation scheme is keyed by count-derived
  // index, not by test — two unrelated tests can otherwise claim the same index).
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "--max-sessions", "2", "task" ] )
    .env( "HOME", "/tmp/clr-isolated-home" )
    .env( "PATH", &clr_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .output()
    .expect( "run clr (print mode, max-sessions 2)" );

  for mut child in background { let _ = child.kill(); let _ = child.wait(); }

  assert!(
    out.status.success(),
    "US25-6: non-interactive invocation must exit 0 (1 print session < limit 2). stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "waiting" ),
    "US25-6: must not wait — interactive fakes must be excluded from the count. Got stderr:\n{stderr}"
  );
}
