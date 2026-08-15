//! Integration tests for `clr isolated` subcommand — native flag parity (extended).
//!
//! Extension of `isolated_test.rs` (IT-1–IT-8, EC-creds/EC-timeout) covering
//! IT-9's help-text check plus the 12 native-flag parity tests and their
//! env var / JSON config / precedence variants.
//!
//! # Test Matrix
//!
//! | ID | Test | Requires Live Claude |
//! |----|------|----------------------|
//! | IT-9 | `clr isolated --help` → exit 0, help text shown (BUG-222) | No |
//! | IT-46..57 | 12 native-flag parity tests (`--model`, `--effort`, `--no-effort-max`, `--system-prompt`, `--append-system-prompt`, `--json-schema`, `--mcp-config`, `--allowed-tools`, `--disallowed-tools`, `--max-budget-usd`, `--max-turns`, `--no-chrome`) | No |
//! | IT-58 | `CLR_MODEL` env var fallback (representative of all 12) | No |
//! | IT-59 | `--args-file` JSON config fallback (representative of all 12) | No |
//! | IT-60 | `--effort max -- --effort low` passthrough last-wins preserved | No |
//! | IT-61 | `--help` lists all 12 new native flags | No |
//! | IT-62 | `--max-budget-usd`/`--max-turns` no cross-field positional swap | No |
//! | IT-63 | `--mcp-config` repeated flag — both values survive | No |
//! | IT-64 | `--json-schema` non-JSON value → exit 1 | No |
//! | IT-65 | `--mcp-config` nonexistent path → exit 1 | No |
//! | IT-66 | `--no-effort-max` wins over a simultaneous `--effort` flag | No |
//! | IT-67 | Native `--model` flag beats `--args-file` JSON config | No |
//!
//! Tests containing `lim_it` run by default in container environments.
//! They early-return when the `claude` binary is absent from `$PATH`.

#![ cfg( feature = "enabled" ) ]

use std::io::Write as _;
use tempfile::NamedTempFile;

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ exit_code, make_creds_file, run_isolated, stderr_str, stdout_str };

/// IT-9: `clr isolated --help` exits 0 and prints isolated-specific help text.
///
/// ## Root Cause (bug_reproducer(BUG-222))
/// `parse_isolated_args()` had no `"-h" | "--help"` arm before the
/// `s if s.starts_with('-')` catch-all, so `--help` matched the catch-all and
/// returned `Err("unknown option: --help")`, causing exit 1.
///
/// ## Why Not Caught
/// Only happy-path and error-flag tests existed for `isolated`;
/// no test exercised `--help` on the subcommand.
///
/// ## Fix Applied
/// Added `print_isolated_help()` function (exits 0) and inserted a
/// `"-h" | "--help"` match arm before the catch-all in `parse_isolated_args()`.
///
/// ## Prevention
/// Test both `-h` and `--help` exit codes and stdout content for
/// every subcommand that accepts flags.
///
/// ## Pitfall
/// `print_isolated_help()` must call `std::process::exit(0)` directly —
/// returning `Ok(...)` from the arm is insufficient because the caller checks
/// `creds_path` and would error on the missing `--creds` argument.
// test_kind: bug_reproducer(BUG-222)
#[ test ]
fn it9_isolated_help_exits_zero()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "isolated", "--help" ] )
    .output()
    .expect( "failed to invoke clr isolated --help" );
  assert_eq!(
    out.status.code(),
    Some( 0 ),
    "clr isolated --help must exit 0; got: {:?}\nstderr: {}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr ),
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--creds" ),
    "help text must mention --creds; got:\n{stdout}",
  );
  assert!(
    stdout.contains( "--timeout" ),
    "help text must mention --timeout; got:\n{stdout}",
  );
}

// ── IT-46 through IT-67: Plan 007 native flag parity ─────────────────────────

/// IT-46: `--model sonnet` overrides isolated's injected `opus` default.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-46
#[ test ]
fn it46_model_flag_overrides_default()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--dry-run", "--model", "sonnet", "msg" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "--model sonnet" ),
    "--model sonnet must appear in command preview; got:\n{}", stdout_str( &out )
  );
}

/// IT-47: `--effort medium` overrides isolated's injected `max` default.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-47
#[ test ]
fn it47_effort_flag_overrides_default()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--dry-run", "--effort", "medium", "msg" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "--effort medium" ),
    "--effort medium must appear in command preview; got:\n{}", stdout_str( &out )
  );
}

/// IT-48: `--no-effort-max` suppresses the injected `--effort` flag entirely.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-48
#[ test ]
fn it48_no_effort_max_suppresses_effort_flag()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--dry-run", "--no-effort-max", "msg" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    !stdout_str( &out ).contains( "--effort" ),
    "--no-effort-max must suppress --effort entirely; got:\n{}", stdout_str( &out )
  );
}

/// IT-49: `--system-prompt "You are terse"` is passed through to the subprocess.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-49
#[ test ]
fn it49_system_prompt_flag_passed_through()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated(
    &[ "--creds", path, "--dry-run", "--system-prompt", "You are terse", "msg" ],
  );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "--system-prompt" ),
    "--system-prompt must appear in command preview; got:\n{stdout}"
  );
  assert!(
    stdout.contains( "You are terse" ),
    "--system-prompt value must appear in command preview; got:\n{stdout}"
  );
}

/// IT-50: `--append-system-prompt "Also: be terse"` is passed through to the subprocess.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-50
#[ test ]
fn it50_append_system_prompt_flag_passed_through()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated(
    &[ "--creds", path, "--dry-run", "--append-system-prompt", "Also: be terse", "msg" ],
  );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "--append-system-prompt" ),
    "--append-system-prompt must appear in command preview; got:\n{stdout}"
  );
  assert!(
    stdout.contains( "Also: be terse" ),
    "--append-system-prompt value must appear in command preview; got:\n{stdout}"
  );
}

/// IT-51: `--json-schema schema.json` is passed through to the subprocess.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-51
#[ test ]
fn it51_json_schema_flag_passed_through()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--dry-run", "--json-schema", "schema.json", "msg" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "--json-schema schema.json" ),
    "--json-schema schema.json must appear in command preview; got:\n{}", stdout_str( &out )
  );
}

/// IT-52: `--mcp-config mcp.json` is passed through to the subprocess.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-52
#[ test ]
fn it52_mcp_config_flag_passed_through()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--dry-run", "--mcp-config", "mcp.json", "msg" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "--mcp-config mcp.json" ),
    "--mcp-config mcp.json must appear in command preview; got:\n{}", stdout_str( &out )
  );
}

/// IT-53: `--allowed-tools "Read,Grep"` is passed through to the subprocess.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-53
#[ test ]
fn it53_allowed_tools_flag_passed_through()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--dry-run", "--allowed-tools", "Read,Grep", "msg" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "--allowed-tools" ),
    "--allowed-tools must appear in command preview; got:\n{stdout}"
  );
  assert!(
    stdout.contains( "Read,Grep" ),
    "--allowed-tools value must appear in command preview; got:\n{stdout}"
  );
}

/// IT-54: `--disallowed-tools "Bash"` is passed through to the subprocess.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-54
#[ test ]
fn it54_disallowed_tools_flag_passed_through()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--dry-run", "--disallowed-tools", "Bash", "msg" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "--disallowed-tools" ),
    "--disallowed-tools must appear in command preview; got:\n{stdout}"
  );
  assert!(
    stdout.contains( "Bash" ),
    "--disallowed-tools value must appear in command preview; got:\n{stdout}"
  );
}

/// IT-55: `--max-budget-usd 2.50` is passed through to the subprocess.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-55
#[ test ]
fn it55_max_budget_usd_flag_passed_through()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--dry-run", "--max-budget-usd", "2.50", "msg" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "--max-budget-usd 2.50" ),
    "--max-budget-usd 2.50 must appear in command preview; got:\n{}", stdout_str( &out )
  );
}

/// IT-56: `--max-turns 10` is passed through to the subprocess.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-56
#[ test ]
fn it56_max_turns_flag_passed_through()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--dry-run", "--max-turns", "10", "msg" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "--max-turns 10" ),
    "--max-turns 10 must appear in command preview; got:\n{}", stdout_str( &out )
  );
}

/// IT-57: `--no-chrome` suppresses chrome injection into the subprocess.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-57
#[ test ]
fn it57_no_chrome_flag_suppresses_chrome()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--dry-run", "--no-chrome", "msg" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "--no-chrome" ),
    "--no-chrome must appear in command preview; got:\n{}", stdout_str( &out )
  );
}

/// IT-58: `CLR_MODEL` env var fallback is equivalent to the `--model` flag (IT-46).
///
/// Source: tests/docs/cli/command/03_isolated.md#it-58
#[ test ]
fn it58_clr_model_env_fallback()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "isolated", "--creds", path, "--dry-run", "msg" ] )
    .env( "CLR_MODEL", "sonnet" )
    .output()
    .expect( "failed to invoke clr isolated" );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "--model sonnet" ),
    "CLR_MODEL=sonnet must be equivalent to --model sonnet; got:\n{}", stdout_str( &out )
  );
}

/// IT-59: `--args-file` JSON config sets `"effort": "medium"`, equivalent to `--effort medium` (IT-47).
///
/// Source: tests/docs/cli/command/03_isolated.md#it-59
#[ test ]
fn it59_effort_json_config_fallback()
{
  let creds   = make_creds_file( "{}" );
  let path    = creds.path().to_str().unwrap();
  let mut cfg = NamedTempFile::new().unwrap();
  write!( cfg, r#"{{"effort": "medium"}}"# ).unwrap();
  let cfg_path = cfg.path().to_str().unwrap();
  let out      = run_isolated( &[ "--creds", path, "--args-file", cfg_path, "--dry-run", "msg" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "--effort medium" ),
    "--args-file effort:medium must be equivalent to --effort medium; got:\n{}", stdout_str( &out )
  );
}

/// IT-60: `--effort max -- --effort low` — raw passthrough after `--` still wins (last occurrence).
///
/// Source: tests/docs/cli/command/03_isolated.md#it-60
#[ test ]
fn it60_effort_passthrough_last_wins_preserved()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated(
    &[ "--creds", path, "--dry-run", "--effort", "max", "--", "--effort", "low", "msg" ],
  );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  let stdout = stdout_str( &out );
  let last   = stdout.rfind( "--effort" ).expect( "at least one --effort must appear in preview" );
  assert!(
    stdout[ last.. ].starts_with( "--effort low" ),
    "last --effort occurrence must be the raw-passthrough value 'low'; got:\n{stdout}"
  );
}

/// IT-61: `clr isolated --help` lists all 12 new native flags.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-61
#[ test ]
fn it61_help_lists_all_new_flags()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "isolated", "--help" ] )
    .output()
    .expect( "failed to invoke clr isolated --help" );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  let stdout = stdout_str( &out );
  for flag in
  [
    "--model", "--effort", "--no-effort-max", "--system-prompt", "--append-system-prompt",
    "--json-schema", "--mcp-config", "--allowed-tools", "--disallowed-tools",
    "--max-budget-usd", "--max-turns", "--no-chrome",
  ]
  {
    assert!(
      stdout.contains( flag ),
      "isolated --help must list {flag}; got:\n{stdout}"
    );
  }
}

/// IT-62: `--max-budget-usd 2.50 --max-turns 10` — no cross-field positional swap.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-62
#[ test ]
fn it62_max_budget_and_max_turns_no_positional_swap()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated(
    &[ "--creds", path, "--dry-run", "--max-budget-usd", "2.50", "--max-turns", "10", "msg" ],
  );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "--max-budget-usd 2.50" ),
    "2.50 must immediately follow --max-budget-usd; got:\n{stdout}"
  );
  assert!(
    stdout.contains( "--max-turns 10" ),
    "10 must immediately follow --max-turns; got:\n{stdout}"
  );
}

/// IT-63: `--mcp-config a.json --mcp-config b.json` — both values survive as separate flag pairs.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-63
#[ test ]
fn it63_mcp_config_repeated_flag_both_survive()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated(
    &[ "--creds", path, "--dry-run", "--mcp-config", "a.json", "--mcp-config", "b.json", "msg" ],
  );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "--mcp-config a.json" ),
    "first --mcp-config a.json pair must survive; got:\n{stdout}"
  );
  assert!(
    stdout.contains( "--mcp-config b.json" ),
    "second --mcp-config b.json pair must survive; got:\n{stdout}"
  );
}

/// IT-64: `--json-schema` with a value that is not valid JSON exits 1.
///
/// `--json-schema` takes inline JSON schema text, never a file path
/// (`docs/cli/param/023_json_schema.md` — type `JsonSchemaText`). The value is
/// forwarded to subprocess argv unchanged with no CLI-side validation, so a
/// stray path string is rejected by the `claude` subprocess itself. The
/// rejection wording depends on the installed `claude` version — newer
/// versions parse the value as JSON ("not valid JSON"); older versions
/// treated it as a path ("not found") — so all wordings are accepted; the
/// invariant is exit 1 with a loud stderr rejection.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-64
#[ test ]
fn it64_json_schema_invalid_json_exits_one()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated(
    &[ "--creds", path, "--json-schema", "/nonexistent_clr_test_path_it64.json", "msg" ],
  );
  assert_eq!(
    exit_code( &out ), 1,
    "expected exit 1 for non-JSON --json-schema value; stderr: {}", stderr_str( &out )
  );
  let err = stderr_str( &out );
  assert!(
    err.contains( "not valid JSON" ) || err.contains( "does not exist" ) || err.contains( "not found" ),
    "stderr must reject the non-JSON --json-schema value; got:\n{err}"
  );
}

/// IT-65: `--mcp-config` with a nonexistent path exits 1, mirroring `--dir`'s IT-17 precedent.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-65
#[ test ]
fn it65_mcp_config_nonexistent_path_exits_one()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated(
    &[ "--creds", path, "--mcp-config", "/nonexistent_clr_test_path_it65.json", "msg" ],
  );
  assert_eq!(
    exit_code( &out ), 1,
    "expected exit 1 for nonexistent --mcp-config path; stderr: {}", stderr_str( &out )
  );
  let err = stderr_str( &out );
  assert!(
    err.contains( "does not exist" ) || err.contains( "not found" ),
    "stderr must indicate nonexistent path; got:\n{err}"
  );
}

/// IT-66: `--effort medium --no-effort-max` together — `--no-effort-max` wins, no `--effort` flag at all.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-66
#[ test ]
fn it66_no_effort_max_wins_over_explicit_effort()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated(
    &[ "--creds", path, "--dry-run", "--effort", "medium", "--no-effort-max", "msg" ],
  );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    !stdout_str( &out ).contains( "--effort" ),
    "--no-effort-max must suppress --effort even when --effort is also given; got:\n{}", stdout_str( &out )
  );
}

/// IT-67: native `--model haiku` flag beats `--args-file` JSON config (`"model": "sonnet"`).
///
/// Source: tests/docs/cli/command/03_isolated.md#it-67
#[ test ]
fn it67_native_model_flag_beats_json_config()
{
  let creds   = make_creds_file( "{}" );
  let path    = creds.path().to_str().unwrap();
  let mut cfg = NamedTempFile::new().unwrap();
  write!( cfg, r#"{{"model": "sonnet"}}"# ).unwrap();
  let cfg_path = cfg.path().to_str().unwrap();
  let out      = run_isolated(
    &[ "--creds", path, "--args-file", cfg_path, "--model", "haiku", "--dry-run", "msg" ],
  );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "--model haiku" ),
    "native --model flag must beat --args-file JSON config; got:\n{}", stdout_str( &out )
  );
}

