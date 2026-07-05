//! Extended Flag Edge Case Tests
//!
//! ## Purpose
//!
//! Cover extended CLI flag spec test cases from `tests/docs/cli/param/` not
//! already exercised by `param_edge_cases_test.rs` or other test files.
//!
//! ## Strategy
//!
//! All tests invoke the compiled binary via `env!("CARGO_BIN_EXE_clr")`.
//! Most tests use `--dry-run` to inspect command assembly without executing claude.
//!
//! ## Spec Coverage
//!
//! --no-chrome:
//! - S34: `--no-chrome "msg"` → no `--chrome` in assembled command (`021_no_chrome.md` EC-1)
//! - S35: default (no `--no-chrome`) → `--chrome` present in assembled command (`021_no_chrome.md` EC-2)
//! - S36: `--no-chrome` without message → exit 0; no `--chrome` in preview (`021_no_chrome.md` EC-3)
//! - S37: `--help` output contains `--no-chrome` (`021_no_chrome.md` EC-4)
//! - S38: `--no-chrome` + `--no-skip-permissions` → neither default flag in assembled command (`021_no_chrome.md` EC-5)
//! - S39: `--no-chrome` + `--dry-run` → preview without `--chrome`; stderr empty (`021_no_chrome.md` EC-6)
//!
//! --no-persist:
//! - S40: `--no-persist "msg"` → `--no-session-persistence` in assembled command (`022_no_persist.md` EC-1)
//! - S41: default (no `--no-persist`) → no `--no-session-persistence` in assembled command (`022_no_persist.md` EC-2)
//! - S42: `--no-persist` without message → exit 0 (`022_no_persist.md` EC-3)
//! - S43: `--help` output contains `--no-persist` (`022_no_persist.md` EC-4)
//! - S44: `--no-persist` + `--new-session` → both accepted; `--no-session-persistence` present, no `-c` (`022_no_persist.md` EC-5)
//! - S45: `--no-persist` + `--dry-run` → preview shows `--no-session-persistence`; stderr empty (`022_no_persist.md` EC-6)
//!
//! --json-schema:
//! - S46: `--json-schema <val> "msg"` → forwarded in assembled command (`023_json_schema.md` EC-1)
//! - S47: default (no `--json-schema`) → no `--json-schema` in assembled command (`023_json_schema.md` EC-2)
//! - S48: complex schema → forwarded verbatim (`023_json_schema.md` EC-3)
//! - S49: `--help` output contains `--json-schema` (`023_json_schema.md` EC-4)
//! - S50: `--json-schema` + `--model` → both forwarded (`023_json_schema.md` EC-5)
//! - S51: `--json-schema` without message → exit 0; schema in assembled command (`023_json_schema.md` EC-6)
//! - S89: `--output-style raw --json-schema` → stdout non-empty (BUG-318 reproducer, `023_json_schema.md` EC-7)
//!
//! --mcp-config:
//! - S52: single `--mcp-config <path> "msg"` → forwarded in assembled command (`024_mcp_config.md` EC-1)
//! - S53: default (no `--mcp-config`) → no `--mcp-config` in assembled command (`024_mcp_config.md` EC-2)
//! - S54: multiple `--mcp-config` flags → all forwarded individually (`024_mcp_config.md` EC-3)
//! - S55: `--help` output contains `--mcp-config` (`024_mcp_config.md` EC-4)
//! - S56: `--mcp-config` + `--model` → both forwarded (`024_mcp_config.md` EC-5)
//! - S57: `--mcp-config` without message → exit 0; path in assembled command (`024_mcp_config.md` EC-6)
//!
//! --subdir:
//! - S81: default (no `--subdir`) → no `/-` path component in dry-run output (`028_subdir.md` EC-1)
//! - S82: `--subdir NAME` → effective dir ends with `/-NAME` (`028_subdir.md` EC-2)
//! - S83: `--subdir .` → identity; no `/-` suffix in dry-run output (`028_subdir.md` EC-3)
//! - S84: `--help` output contains `--subdir` (`028_subdir.md` EC-4)
//! - S85: `--subdir NAME` + `--dir PATH` → effective dir is `PATH/-NAME` (`028_subdir.md` EC-5)
//! - S86: `--subdir ""` → identity (BUG-229 reproducer)
//! - S87: `--subdir "a/b"` → rejected, slash in name (BUG-230 reproducer)
//! - S88: `--dry-run --subdir NAME` → no directory created (BUG-231 reproducer)

mod cli_binary_test_helpers;
use cli_binary_test_helpers::run_cli;
#[ cfg( unix ) ]
use cli_binary_test_helpers::fake_claude_dir;

// S34: `--no-chrome` suppresses default `--chrome` injection (`021_no_chrome.md` EC-1)
#[ test ]
fn s34_no_chrome_suppresses_chrome_flag()
{
  let out = run_cli( &[ "--dry-run", "--no-chrome", "Fix bug" ] );
  assert!( out.status.success(), "--no-chrome must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--chrome" ),
    "--no-chrome must suppress --chrome. Got:\n{stdout}"
  );
}

// S35: default (no `--no-chrome`) → `--chrome` present in interactive mode (`021_no_chrome.md` EC-2)
// Fix(BUG-304): chrome is suppressed automatically in print mode; only injected for interactive.
#[ test ]
fn s35_default_chrome_injected_interactive()
{
  // No message → interactive mode; --chrome must appear by default.
  let out = run_cli( &[ "--dry-run" ] );
  assert!( out.status.success(), "exit={} stderr={}", out.status.code().unwrap_or( -1 ), String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--chrome" ),
    "default interactive command must contain --chrome. Got:\n{stdout}"
  );
}

// S35b: print mode (message given, no explicit --no-chrome) → `--chrome` absent (`021_no_chrome.md` EC-2b)
// Fix(BUG-304): chrome is automatically suppressed in print mode to prevent session hang.
#[ test ]
fn s35b_print_mode_suppresses_chrome()
{
  // Message present → print mode auto-detected; --chrome must be suppressed.
  let out = run_cli( &[ "--dry-run", "Fix bug" ] );
  assert!( out.status.success(), "exit={} stderr={}", out.status.code().unwrap_or( -1 ), String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--chrome" ),
    "print-mode command must NOT contain --chrome (BUG-304 mitigation). Got:\n{stdout}"
  );
}

// S36: `--no-chrome` without message → exit 0; no `--chrome` in preview (`021_no_chrome.md` EC-3)
#[ test ]
fn s36_no_chrome_without_message_accepted()
{
  let out = run_cli( &[ "--dry-run", "--no-chrome" ] );
  assert!(
    out.status.success(),
    "--no-chrome without message must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--chrome" ),
    "--no-chrome must suppress --chrome even without a message. Got:\n{stdout}"
  );
}

// S37: `--help` output lists `--no-chrome` (`021_no_chrome.md` EC-4)
#[ test ]
fn s37_help_lists_no_chrome()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "exit={} stderr={}", out.status.code().unwrap_or( -1 ), String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--no-chrome" ),
    "--help must mention --no-chrome. Got:\n{stdout}"
  );
}

// S38: `--no-chrome` + `--no-skip-permissions` → neither `--chrome` nor
// `--dangerously-skip-permissions` in assembled command (`021_no_chrome.md` EC-5)
#[ test ]
fn s38_no_chrome_with_no_skip_permissions_both_suppressed()
{
  let out = run_cli( &[ "--dry-run", "--no-chrome", "--no-skip-permissions", "Fix bug" ] );
  assert!( out.status.success(), "exit={} stderr={}", out.status.code().unwrap_or( -1 ), String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--chrome" ),
    "--no-chrome must suppress --chrome. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "--dangerously-skip-permissions" ),
    "--no-skip-permissions must suppress --dangerously-skip-permissions. Got:\n{stdout}"
  );
}

// S39: `--no-chrome` + `--dry-run` → preview without `--chrome`; stderr empty (`021_no_chrome.md` EC-6)
#[ test ]
fn s39_no_chrome_with_dry_run_preview_clean()
{
  let out = run_cli( &[ "--dry-run", "--no-chrome", "Fix bug" ] );
  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--chrome" ),
    "--no-chrome must suppress --chrome in dry-run preview. Got:\n{stdout}"
  );
  assert!(
    out.stderr.is_empty(),
    "--dry-run --no-chrome must produce no stderr. Got:\n{}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// S40: `--no-persist` → `--no-session-persistence` in assembled command (`022_no_persist.md` EC-1)
#[ test ]
fn s40_no_persist_forwards_no_session_persistence()
{
  let out = run_cli( &[ "--dry-run", "--no-persist", "Fix bug" ] );
  assert!( out.status.success(), "--no-persist must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--no-session-persistence" ),
    "--no-persist must forward --no-session-persistence. Got:\n{stdout}"
  );
}

// S41: default (no `--no-persist`) → no `--no-session-persistence` in assembled command (`022_no_persist.md` EC-2)
#[ test ]
fn s41_default_no_session_persistence_absent()
{
  let out = run_cli( &[ "--dry-run", "Fix bug" ] );
  assert!( out.status.success(), "exit={} stderr={}", out.status.code().unwrap_or( -1 ), String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--no-session-persistence" ),
    "default command must not contain --no-session-persistence. Got:\n{stdout}"
  );
}

// S42: `--no-persist` without message → exit 0 (`022_no_persist.md` EC-3)
#[ test ]
fn s42_no_persist_without_message_accepted()
{
  let out = run_cli( &[ "--dry-run", "--no-persist" ] );
  assert!(
    out.status.success(),
    "--no-persist without message must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--no-session-persistence" ),
    "--no-persist must forward --no-session-persistence. Got:\n{stdout}"
  );
}

// S43: `--help` output lists `--no-persist` (`022_no_persist.md` EC-4)
#[ test ]
fn s43_help_lists_no_persist()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "exit={} stderr={}", out.status.code().unwrap_or( -1 ), String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--no-persist" ),
    "--help must mention --no-persist. Got:\n{stdout}"
  );
}

// S44: `--no-persist` + `--new-session` → `--no-session-persistence` present, no `-c` (`022_no_persist.md` EC-5)
#[ test ]
fn s44_no_persist_with_new_session_accepted()
{
  let out = run_cli( &[ "--dry-run", "--no-persist", "--new-session", "Fix bug" ] );
  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--no-session-persistence" ),
    "--no-persist must forward --no-session-persistence. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( " -c" ),
    "--new-session must suppress -c. Got:\n{stdout}"
  );
}

// S45: `--no-persist` + `--dry-run` → preview shows `--no-session-persistence`; stderr empty (`022_no_persist.md` EC-6)
#[ test ]
fn s45_no_persist_with_dry_run_preview_shows_flag()
{
  let out = run_cli( &[ "--dry-run", "--no-persist", "Fix bug" ] );
  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--no-session-persistence" ),
    "--no-persist must appear in dry-run preview. Got:\n{stdout}"
  );
  assert!(
    out.stderr.is_empty(),
    "--dry-run --no-persist must produce no stderr. Got:\n{}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// S46: `--json-schema <val>` forwarded in assembled command (`023_json_schema.md` EC-1)
#[ test ]
fn s46_json_schema_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--json-schema", r#"{"type":"object"}"#, "task" ] );
  assert!( out.status.success(), "--json-schema must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--json-schema" ),
    "--json-schema must appear in assembled command. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( r#"{"type":"object"}"# ),
    "--json-schema value must be forwarded. Got:\n{stdout}"
  );
}

// S47: default (no `--json-schema`) → absent from assembled command (`023_json_schema.md` EC-2)
#[ test ]
fn s47_default_json_schema_absent()
{
  let out = run_cli( &[ "--dry-run", "task" ] );
  assert!( out.status.success(), "exit={} stderr={}", out.status.code().unwrap_or( -1 ), String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--json-schema" ),
    "default command must not contain --json-schema. Got:\n{stdout}"
  );
}

// S48: complex nested schema forwarded verbatim (`023_json_schema.md` EC-3)
#[ test ]
fn s48_json_schema_complex_forwarded_verbatim()
{
  let schema = r#"{"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}"#;
  let out = run_cli( &[ "--dry-run", "--json-schema", schema, "task" ] );
  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( schema ),
    "complex schema must be forwarded verbatim. Got:\n{stdout}"
  );
}

// S49: `--help` output lists `--json-schema` (`023_json_schema.md` EC-4)
#[ test ]
fn s49_help_lists_json_schema()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "exit={} stderr={}", out.status.code().unwrap_or( -1 ), String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--json-schema" ),
    "--help must mention --json-schema. Got:\n{stdout}"
  );
}

// S50: `--json-schema` + `--model` → both forwarded (`023_json_schema.md` EC-5)
#[ test ]
fn s50_json_schema_with_model_both_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--json-schema", r#"{"type":"string"}"#, "--model", "sonnet", "task" ] );
  assert!( out.status.success(), "exit={} stderr={}", out.status.code().unwrap_or( -1 ), String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--json-schema" ),
    "--json-schema must appear. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "--model sonnet" ),
    "--model must appear. Got:\n{stdout}"
  );
}

// S51: `--json-schema` without message → exit 0; schema in assembled command (`023_json_schema.md` EC-6)
#[ test ]
fn s51_json_schema_without_message_accepted()
{
  let out = run_cli( &[ "--dry-run", "--json-schema", r#"{"type":"string"}"# ] );
  assert!(
    out.status.success(),
    "--json-schema without message must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--json-schema" ),
    "--json-schema must appear in assembled command. Got:\n{stdout}"
  );
}

// S52: single `--mcp-config <path>` forwarded in assembled command (`024_mcp_config.md` EC-1)
#[ test ]
fn s52_mcp_config_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--mcp-config", "/tmp/mcp.json", "task" ] );
  assert!( out.status.success(), "--mcp-config must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--mcp-config /tmp/mcp.json" ),
    "--mcp-config value must appear in assembled command. Got:\n{stdout}"
  );
}

// S53: default (no `--mcp-config`) → absent from assembled command (`024_mcp_config.md` EC-2)
#[ test ]
fn s53_default_mcp_config_absent()
{
  let out = run_cli( &[ "--dry-run", "task" ] );
  assert!( out.status.success(), "exit={} stderr={}", out.status.code().unwrap_or( -1 ), String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--mcp-config" ),
    "default command must not contain --mcp-config. Got:\n{stdout}"
  );
}

// S54: multiple `--mcp-config` flags → all forwarded as separate occurrences (`024_mcp_config.md` EC-3)
#[ test ]
fn s54_mcp_config_multiple_forwarded_individually()
{
  let out = run_cli( &[
    "--dry-run", "--mcp-config", "/tmp/s1.json", "--mcp-config", "/tmp/s2.json", "task",
  ] );
  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  let count = stdout.matches( "--mcp-config" ).count();
  assert!(
    count >= 2,
    "multiple --mcp-config flags must each appear in assembled command (found {count}). Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "/tmp/s1.json" ),
    "first mcp-config path must appear. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "/tmp/s2.json" ),
    "second mcp-config path must appear. Got:\n{stdout}"
  );
}

// S55: `--help` output lists `--mcp-config` (`024_mcp_config.md` EC-4)
#[ test ]
fn s55_help_lists_mcp_config()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "exit={} stderr={}", out.status.code().unwrap_or( -1 ), String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--mcp-config" ),
    "--help must mention --mcp-config. Got:\n{stdout}"
  );
}

// S56: `--mcp-config` + `--model` → both forwarded (`024_mcp_config.md` EC-5)
#[ test ]
fn s56_mcp_config_with_model_both_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--mcp-config", "/tmp/mcp.json", "--model", "sonnet", "task" ] );
  assert!( out.status.success(), "exit={} stderr={}", out.status.code().unwrap_or( -1 ), String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--mcp-config" ),
    "--mcp-config must appear. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "--model sonnet" ),
    "--model must appear. Got:\n{stdout}"
  );
}

// S57: `--mcp-config` without message → exit 0; path in assembled command (`024_mcp_config.md` EC-6)
#[ test ]
fn s57_mcp_config_without_message_accepted()
{
  let out = run_cli( &[ "--dry-run", "--mcp-config", "/tmp/mcp.json" ] );
  assert!(
    out.status.success(),
    "--mcp-config without message must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--mcp-config /tmp/mcp.json" ),
    "--mcp-config must appear in assembled command. Got:\n{stdout}"
  );
}

// ─── --subdir ─────────────────────────────────────────────────────────────────
// Source: tests/docs/cli/param/028_subdir.md
// EC-6 (env var) and EC-7 (CLI-wins) are covered by env_var_ext_test.rs E29.

// S81: default (no `--subdir`) → no `/-` path component in dry-run output (`028_subdir.md` EC-1)
#[ test ]
fn s81_default_no_subdir_no_hyphen_prefix()
{
  let out = run_cli( &[ "--dry-run", "task" ] );
  assert!( out.status.success(), "must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  let sep = std::path::MAIN_SEPARATOR;
  assert!(
    !stdout.contains( &format!( "{sep}-" ) ),
    "without --subdir, no {sep}- path component must appear. Got:\n{stdout}"
  );
}

// S82: `--subdir NAME` → effective dir ends with `/-NAME` (`028_subdir.md` EC-2)
#[ test ]
fn s82_subdir_name_appends_hyphen_prefix()
{
  let out = run_cli( &[ "--dry-run", "--subdir", "build", "task" ] );
  assert!( out.status.success(), "must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  let sep = std::path::MAIN_SEPARATOR;
  assert!(
    stdout.contains( &format!( "{sep}-build" ) ),
    "--subdir build must produce path ending in {sep}-build. Got:\n{stdout}"
  );
}

// S83: `--subdir .` → identity; no `/-` suffix in dry-run output (`028_subdir.md` EC-3)
#[ test ]
fn s83_subdir_dot_identity_no_suffix()
{
  let out = run_cli( &[ "--dry-run", "--subdir", ".", "task" ] );
  assert!( out.status.success(), "must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  let sep = std::path::MAIN_SEPARATOR;
  assert!(
    !stdout.contains( &format!( "{sep}-" ) ),
    "--subdir . must not append any {sep}- suffix. Got:\n{stdout}"
  );
}

// S84: `--help` output contains `--subdir` (`028_subdir.md` EC-4)
#[ test ]
fn s84_help_lists_subdir()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "exit={} stderr={}", out.status.code().unwrap_or( -1 ), String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--subdir" ),
    "--help must mention --subdir. Got:\n{stdout}"
  );
}

// S85: `--subdir NAME` + `--dir PATH` → effective dir is `PATH/-NAME` (`028_subdir.md` EC-5)
#[ cfg( unix ) ]
#[ test ]
fn s85_subdir_with_dir_combined()
{
  let out = run_cli( &[ "--dry-run", "--dir", "/tmp/project", "--subdir", "debug", "task" ] );
  assert!( out.status.success(), "must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "/tmp/project/-debug" ),
    "--dir /tmp/project --subdir debug must produce /tmp/project/-debug. Got:\n{stdout}"
  );
}

// ─── --subdir bug-fix reproducing tests ─────────────────────────────────────

/// Fix(BUG-229): `--subdir ""` must be identity — no degenerate `/-` directory.
///
/// ## Root Cause
/// Only `"."` was checked; empty string passed the guard and produced a bare-hyphen dir.
///
/// ## Why Not Caught
/// No test covered empty-string input; only `"."` identity and valid names were tested.
///
/// ## Fix Applied
/// Added `!sub.is_empty()` guard alongside `sub != "."` in `build_claude_command`.
///
/// ## Prevention
/// Fuzz-like edge case tests for all string-typed CLI parameters.
///
/// ## Pitfall
/// `env_str` already filters empty strings — only the CLI path can deliver `""`.
// test_kind: bug_reproducer(BUG-229)
#[ test ]
fn s86_subdir_empty_string_is_identity()
{
  let out = run_cli( &[ "--dry-run", "--subdir", "", "task" ] );
  assert!( out.status.success(), "must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  let sep = std::path::MAIN_SEPARATOR;
  assert!(
    !stdout.contains( &format!( "{sep}-" ) ),
    "--subdir '' (empty) must be identity — no {sep}- suffix. Got:\n{stdout}"
  );
}

/// Fix(BUG-230): `--subdir` must reject names containing `/`.
///
/// ## Root Cause
/// No validation; `create_dir_all` silently created nested dirs for `a/b`.
///
/// ## Why Not Caught
/// All prior tests used simple alphanumeric names; slash input was never tested.
///
/// ## Fix Applied
/// Added `val.contains('/')` validation in `parse_value_flag` for `--subdir`.
///
/// ## Prevention
/// Validate all string-typed params against their documented type constraints.
///
/// ## Pitfall
/// The type constraint is "directory name component (no `/` separators)" in the spec.
// test_kind: bug_reproducer(BUG-230)
#[ test ]
fn s87_subdir_rejects_slash()
{
  let out = run_cli( &[ "--dry-run", "--subdir", "a/b", "task" ] );
  assert!(
    !out.status.success(),
    "--subdir a/b must be rejected (contains '/'). Got exit: {:?}",
    out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "no '/' separators" ),
    "--subdir a/b error must mention slash constraint. Got:\n{stderr}"
  );
}

/// Fix(BUG-231): `--dry-run --subdir NAME` must NOT create the directory.
///
/// ## Root Cause
/// `build_claude_command` runs `create_dir_all` before the dry-run branch in `lib.rs`.
///
/// ## Why Not Caught
/// Tests only checked stdout output strings; no test verified filesystem state.
///
/// ## Fix Applied
/// Added `!cli.dry_run` guard around `create_dir_all` in `build_claude_command`.
///
/// ## Prevention
/// Dry-run tests should assert absence of side effects, not just correct output.
///
/// ## Pitfall
/// Builder computes the path for display; only the run path needs the physical directory.
// test_kind: bug_reproducer(BUG-231)
#[ test ]
fn s88_dryrun_subdir_no_mkdir()
{
  let unique = format!( "clr_drytest_{}", std::process::id() );
  let base = std::env::temp_dir().join( unique );
  let expected_dir = base.join( "-probe" );
  // ensure clean slate
  let _ = std::fs::remove_dir_all( &base );
  let out = run_cli( &[
    "--dry-run",
    "--dir", base.to_str().unwrap(),
    "--subdir", "probe",
    "task",
  ] );
  assert!( out.status.success(), "must exit 0: {out:?}" );
  assert!(
    !expected_dir.exists(),
    "--dry-run must not create directory {expected_dir:?}"
  );
  // cleanup
  let _ = std::fs::remove_dir_all( base );
}

// S89: `--output-style raw --json-schema` → stdout non-empty; structured_output extracted
// (`023_json_schema.md` EC-7 / BUG-318 fix)

/// Fix(BUG-318): `--output-style raw` combined with `--json-schema` produced empty stdout
/// because claude returns an empty `"result"` text field for structured responses.
///
/// ## Root Cause
/// The raw execution branch passed through the `"result"` text field unchanged — empty for
/// structured JSON responses where the actual data lives in `"structured_output"`.  builder.rs
/// also did not inject `--output-format json` for the raw+json-schema combination, so no
/// CLR envelope was produced and `structured_output` was inaccessible.
///
/// ## Why Not Caught
/// No test covered the combined `--output-style raw` + `--json-schema` execution path with
/// a fake claude emitting a CLR envelope where `"result"` is empty.
///
/// ## Fix Applied
/// builder.rs widened the Path B auto-inject gate to `effective_style == "summary" ||
/// cli.json_schema.is_some()`; execution.rs added `else if cli.json_schema.is_some()`
/// branch calling `extract_structured_output()` from summary.rs.
///
/// ## Prevention
/// This test verifies the raw+json-schema execution path end-to-end with a CLR envelope
/// where `"result"` is empty and `"structured_output"` contains the actual value.
///
/// ## Pitfall
/// The fake claude must emit `"result":""` (empty) to reproduce the original symptom —
/// a non-empty `result` field would have produced output even before the fix.
// test_kind: bug_reproducer(BUG-318)
#[ cfg( unix ) ]
#[ test ]
fn s89_raw_style_with_json_schema_outputs_structured_result()
{
  let fixture = r#"{"type":"result","subtype":"success","session_id":"00000000-0000-0000-0000-000000000001","is_error":false,"result":"","structured_output":{"x":"hello"},"usage":{"input_tokens":1,"output_tokens":1},"total_cost_usd":0.0}"#;
  let body    = format!( "echo '{fixture}'" );
  let ( _dir, path ) = fake_claude_dir( &body );
  let bin    = env!( "CARGO_BIN_EXE_clr" );
  let schema = r#"{"type":"object","properties":{"x":{"type":"string"}},"required":["x"]}"#;
  let out = std::process::Command::new( bin )
    .args( [ "-p", "--max-sessions", "0", "--output-style", "raw", "--json-schema", schema, "test" ] )
    .env( "PATH", &path )
    .env_remove( "CLR_OUTPUT_STYLE" )
    .output()
    .expect( "Failed to invoke clr binary" );
  assert!( out.status.success(), "exit must be 0 (BUG-318): {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.is_empty(),
    "stdout must be non-empty for raw+json-schema (BUG-318 regression). Got empty."
  );
  assert!(
    stdout.contains( "\"x\"" ),
    "stdout must contain '\"x\"' from structured_output field (BUG-318). Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "hello" ),
    "stdout must contain 'hello' from structured_output value (BUG-318). Got:\n{stdout}"
  );
}
