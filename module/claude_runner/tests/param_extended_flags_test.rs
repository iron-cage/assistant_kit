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
//! - S34: `--no-chrome "msg"` → no `--chrome` in assembled command (`21_no_chrome.md` EC-1)
//! - S35: default (no `--no-chrome`) → `--chrome` present in assembled command (`21_no_chrome.md` EC-2)
//! - S36: `--no-chrome` without message → exit 0; no `--chrome` in preview (`21_no_chrome.md` EC-3)
//! - S37: `--help` output contains `--no-chrome` (`21_no_chrome.md` EC-4)
//! - S38: `--no-chrome` + `--no-skip-permissions` → neither default flag in assembled command (`21_no_chrome.md` EC-5)
//! - S39: `--no-chrome` + `--dry-run` → preview without `--chrome`; stderr empty (`21_no_chrome.md` EC-6)
//!
//! --no-persist:
//! - S40: `--no-persist "msg"` → `--no-session-persistence` in assembled command (`22_no_persist.md` EC-1)
//! - S41: default (no `--no-persist`) → no `--no-session-persistence` in assembled command (`22_no_persist.md` EC-2)
//! - S42: `--no-persist` without message → exit 0 (`22_no_persist.md` EC-3)
//! - S43: `--help` output contains `--no-persist` (`22_no_persist.md` EC-4)
//! - S44: `--no-persist` + `--new-session` → both accepted; `--no-session-persistence` present, no `-c` (`22_no_persist.md` EC-5)
//! - S45: `--no-persist` + `--dry-run` → preview shows `--no-session-persistence`; stderr empty (`22_no_persist.md` EC-6)
//!
//! --json-schema:
//! - S46: `--json-schema <val> "msg"` → forwarded in assembled command (`23_json_schema.md` EC-1)
//! - S47: default (no `--json-schema`) → no `--json-schema` in assembled command (`23_json_schema.md` EC-2)
//! - S48: complex schema → forwarded verbatim (`23_json_schema.md` EC-3)
//! - S49: `--help` output contains `--json-schema` (`23_json_schema.md` EC-4)
//! - S50: `--json-schema` + `--model` → both forwarded (`23_json_schema.md` EC-5)
//! - S51: `--json-schema` without message → exit 0; schema in assembled command (`23_json_schema.md` EC-6)
//!
//! --mcp-config:
//! - S52: single `--mcp-config <path> "msg"` → forwarded in assembled command (`24_mcp_config.md` EC-1)
//! - S53: default (no `--mcp-config`) → no `--mcp-config` in assembled command (`24_mcp_config.md` EC-2)
//! - S54: multiple `--mcp-config` flags → all forwarded individually (`24_mcp_config.md` EC-3)
//! - S55: `--help` output contains `--mcp-config` (`24_mcp_config.md` EC-4)
//! - S56: `--mcp-config` + `--model` → both forwarded (`24_mcp_config.md` EC-5)
//! - S57: `--mcp-config` without message → exit 0; path in assembled command (`24_mcp_config.md` EC-6)

mod cli_binary_test_helpers;
use cli_binary_test_helpers::run_cli;

// S34: `--no-chrome` suppresses default `--chrome` injection (`21_no_chrome.md` EC-1)
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

// S35: default (no `--no-chrome`) → `--chrome` present in assembled command (`21_no_chrome.md` EC-2)
#[ test ]
fn s35_default_chrome_injected()
{
  let out = run_cli( &[ "--dry-run", "Fix bug" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--chrome" ),
    "default assembled command must contain --chrome. Got:\n{stdout}"
  );
}

// S36: `--no-chrome` without message → exit 0; no `--chrome` in preview (`21_no_chrome.md` EC-3)
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

// S37: `--help` output lists `--no-chrome` (`21_no_chrome.md` EC-4)
#[ test ]
fn s37_help_lists_no_chrome()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--no-chrome" ),
    "--help must mention --no-chrome. Got:\n{stdout}"
  );
}

// S38: `--no-chrome` + `--no-skip-permissions` → neither `--chrome` nor
// `--dangerously-skip-permissions` in assembled command (`21_no_chrome.md` EC-5)
#[ test ]
fn s38_no_chrome_with_no_skip_permissions_both_suppressed()
{
  let out = run_cli( &[ "--dry-run", "--no-chrome", "--no-skip-permissions", "Fix bug" ] );
  assert!( out.status.success() );
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

// S39: `--no-chrome` + `--dry-run` → preview without `--chrome`; stderr empty (`21_no_chrome.md` EC-6)
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

// S40: `--no-persist` → `--no-session-persistence` in assembled command (`22_no_persist.md` EC-1)
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

// S41: default (no `--no-persist`) → no `--no-session-persistence` in assembled command (`22_no_persist.md` EC-2)
#[ test ]
fn s41_default_no_session_persistence_absent()
{
  let out = run_cli( &[ "--dry-run", "Fix bug" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--no-session-persistence" ),
    "default command must not contain --no-session-persistence. Got:\n{stdout}"
  );
}

// S42: `--no-persist` without message → exit 0 (`22_no_persist.md` EC-3)
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

// S43: `--help` output lists `--no-persist` (`22_no_persist.md` EC-4)
#[ test ]
fn s43_help_lists_no_persist()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--no-persist" ),
    "--help must mention --no-persist. Got:\n{stdout}"
  );
}

// S44: `--no-persist` + `--new-session` → `--no-session-persistence` present, no `-c` (`22_no_persist.md` EC-5)
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

// S45: `--no-persist` + `--dry-run` → preview shows `--no-session-persistence`; stderr empty (`22_no_persist.md` EC-6)
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

// S46: `--json-schema <val>` forwarded in assembled command (`23_json_schema.md` EC-1)
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

// S47: default (no `--json-schema`) → absent from assembled command (`23_json_schema.md` EC-2)
#[ test ]
fn s47_default_json_schema_absent()
{
  let out = run_cli( &[ "--dry-run", "task" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--json-schema" ),
    "default command must not contain --json-schema. Got:\n{stdout}"
  );
}

// S48: complex nested schema forwarded verbatim (`23_json_schema.md` EC-3)
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

// S49: `--help` output lists `--json-schema` (`23_json_schema.md` EC-4)
#[ test ]
fn s49_help_lists_json_schema()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--json-schema" ),
    "--help must mention --json-schema. Got:\n{stdout}"
  );
}

// S50: `--json-schema` + `--model` → both forwarded (`23_json_schema.md` EC-5)
#[ test ]
fn s50_json_schema_with_model_both_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--json-schema", r#"{"type":"string"}"#, "--model", "sonnet", "task" ] );
  assert!( out.status.success() );
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

// S51: `--json-schema` without message → exit 0; schema in assembled command (`23_json_schema.md` EC-6)
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

// S52: single `--mcp-config <path>` forwarded in assembled command (`24_mcp_config.md` EC-1)
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

// S53: default (no `--mcp-config`) → absent from assembled command (`24_mcp_config.md` EC-2)
#[ test ]
fn s53_default_mcp_config_absent()
{
  let out = run_cli( &[ "--dry-run", "task" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--mcp-config" ),
    "default command must not contain --mcp-config. Got:\n{stdout}"
  );
}

// S54: multiple `--mcp-config` flags → all forwarded as separate occurrences (`24_mcp_config.md` EC-3)
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

// S55: `--help` output lists `--mcp-config` (`24_mcp_config.md` EC-4)
#[ test ]
fn s55_help_lists_mcp_config()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--mcp-config" ),
    "--help must mention --mcp-config. Got:\n{stdout}"
  );
}

// S56: `--mcp-config` + `--model` → both forwarded (`24_mcp_config.md` EC-5)
#[ test ]
fn s56_mcp_config_with_model_both_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--mcp-config", "/tmp/mcp.json", "--model", "sonnet", "task" ] );
  assert!( out.status.success() );
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

// S57: `--mcp-config` without message → exit 0; path in assembled command (`24_mcp_config.md` EC-6)
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
