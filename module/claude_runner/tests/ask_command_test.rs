//! `ask` Subcommand Integration Tests
//!
//! ## Purpose
//!
//! Verify that `clr ask` applies Q&A-optimised defaults that differ from
//! `clr run` defaults, and that CLI flags correctly override those defaults.
//!
//! ## Strategy
//!
//! Tests IT-1 through IT-8 invoke `clr ask --dry-run` and inspect `stdout` for the
//! assembled command and env-var block.  No real Claude invocation occurs.
//!
//! IT-9 is a live-trace variant: it invokes `clr ask --trace` without `--dry-run`
//! and inspects `stderr` for the diagnostic output emitted before invocation.
//!
//! ## Corner Cases Covered
//!
//! - IT-1: `clr ask --dry-run "X"` — no `-c`, no `--dangerously-skip-permissions`, has `--print`
//! - IT-2: `clr ask --dry-run "X"` — `--effort high` (not `--effort max`)
//! - IT-3: `clr ask --dry-run "X"` — env has `CLAUDE_CODE_MAX_OUTPUT_TOKENS=16384`
//! - IT-4: `clr ask --dry-run "X"` — message does NOT contain `ultrathink`
//! - IT-5: `clr ask --dry-run "X"` — no `--chrome` and no `--no-chrome`
//! - IT-6: `clr ask --dry-run --effort max "X"` — CLI effort overrides ask default
//! - IT-7: `clr ask --dry-run --max-tokens 200000 "X"` — CLI max-tokens overrides ask default
//! - IT-8: `clr ask --unknown-flag "X"` — unknown flag rejected (exit 1, stderr error)
//! - IT-9: `clr ask --trace "X"` — stderr contains ask-default env block + command before invocation
//! - IT-10: `clr ask --subdir NAME "X"` — effective dir ends with `/-NAME`
//! - IT-11: `CLR_EFFORT=low clr ask --dry-run "X"` — env var overrides ask soft default (BUG-245 regression)
//! - IT-12: `CLR_MAX_TOKENS=50000 clr ask --dry-run "X"` — env var overrides ask soft default (BUG-245 regression)

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };
use std::process::Command;

/// Invoke `clr ask --dry-run` with the given args and return stdout.
///
/// Asserts that the subprocess exits successfully.
fn run_ask_dry( extra_args : &[ &str ] ) -> String
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut args = vec![ "ask", "--dry-run" ];
  args.extend_from_slice( extra_args );
  let out = Command::new( bin )
    .args( &args )
    .output()
    .expect( "failed to invoke clr binary" );
  assert!(
    out.status.success(),
    "clr ask --dry-run failed (exit {}): {}",
    out.status.code().unwrap_or( -1 ),
    String::from_utf8_lossy( &out.stderr )
  );
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

// IT-1: ask defaults — no session-continuation, no skip-permissions, has --print
#[ test ]
fn it_01_ask_no_continue_no_skip_perms_has_print()
{
  let output = run_ask_dry( &[ "What does X do?" ] );
  assert!(
    !output.contains( " -c" ),
    "ask must not include `-c` (no session continuation). Got:\n{output}"
  );
  assert!(
    !output.contains( "--dangerously-skip-permissions" ),
    "ask must not include --dangerously-skip-permissions. Got:\n{output}"
  );
  assert!(
    output.contains( "--print" ),
    "ask must include --print for non-interactive message. Got:\n{output}"
  );
}

// IT-2: ask defaults — effort high, not max
#[ test ]
fn it_02_ask_effort_defaults_to_high()
{
  let output = run_ask_dry( &[ "What does X do?" ] );
  assert!(
    output.contains( "--effort high" ),
    "ask must use --effort high by default. Got:\n{output}"
  );
  assert!(
    !output.contains( "--effort max" ),
    "ask must NOT use --effort max. Got:\n{output}"
  );
}

// IT-3: ask defaults — max tokens 16384
#[ test ]
fn it_03_ask_max_tokens_defaults_to_16384()
{
  let output = run_ask_dry( &[ "What does X do?" ] );
  assert!(
    output.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=16384" ),
    "ask must use CLAUDE_CODE_MAX_OUTPUT_TOKENS=16384 by default. Got:\n{output}"
  );
}

// IT-4: ask defaults — no ultrathink suffix
#[ test ]
fn it_04_ask_no_ultrathink_suffix()
{
  let output = run_ask_dry( &[ "What does X do?" ] );
  assert!(
    !output.contains( "ultrathink" ),
    "ask must not inject ultrathink suffix. Got:\n{output}"
  );
}

// IT-5: ask defaults — no chrome flag (neither --chrome nor --no-chrome)
#[ test ]
fn it_05_ask_no_chrome_flag()
{
  let output = run_ask_dry( &[ "What does X do?" ] );
  assert!(
    !output.contains( "--chrome" ),
    "ask must suppress chrome flag (no --chrome or --no-chrome). Got:\n{output}"
  );
}

// IT-6: --effort max overrides ask default of high
#[ test ]
fn it_06_ask_effort_override_to_max()
{
  let output = run_ask_dry( &[ "--effort", "max", "What does X do?" ] );
  assert!(
    output.contains( "--effort max" ),
    "explicit --effort max must override ask default. Got:\n{output}"
  );
  assert!(
    !output.contains( "--effort high" ),
    "ask must not inject --effort high when overridden. Got:\n{output}"
  );
}

// IT-7: --max-tokens 200000 overrides ask default of 16384
#[ test ]
fn it_07_ask_max_tokens_override()
{
  let output = run_ask_dry( &[ "--max-tokens", "200000", "What does X do?" ] );
  assert!(
    output.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000" ),
    "explicit --max-tokens 200000 must override ask default. Got:\n{output}"
  );
  assert!(
    !output.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=16384" ),
    "ask must not inject 16384 when overridden. Got:\n{output}"
  );
}

// IT-8: unknown flag rejected — exit 1, stderr has error
#[ test ]
fn it_08_ask_unknown_flag_rejected()
{
  let out = run_cli( &[ "ask", "--unknown-flag-xyz", "X" ] );
  assert!(
    !out.status.success(),
    "unknown flag must cause non-zero exit. Got exit: {:?}",
    out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "unknown option" ) || stderr.contains( "Error:" ),
    "error message must appear on stderr. Got:\n{stderr}"
  );
}

// IT-9: ask --trace → stderr contains ask-default env block + command before invocation
//
// Exercises the live trace code path (no --dry-run). Trace fires before subprocess
// invocation, so exit 1 (claude absent) is acceptable.
//
// Source: tests/docs/cli/command/005_ask.md#it-9
#[ test ]
fn it_09_ask_trace_stderr_output()
{
  let out    = run_cli( &[ "ask", "--trace", "What is X?" ] );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=16384" ),
    "ask --trace must emit CLAUDE_CODE_MAX_OUTPUT_TOKENS=16384 on stderr. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "--effort high" ),
    "ask --trace must emit --effort high on stderr. Got:\n{stderr}"
  );
  // exit 1 (claude absent) is acceptable — trace fires before subprocess invocation
  let code = out.status.code().unwrap_or( -1 );
  assert!( code == 0 || code == 1, "expected exit 0 or 1 (trace before invoke); got {code}" );
}

// IT-10: ask --subdir NAME → effective dir ends with /-NAME
//
// Source: tests/docs/cli/command/05_ask.md#it-10
#[ test ]
fn it_10_ask_subdir_effective_dir()
{
  let output = run_ask_dry( &[ "--subdir", "feature", "What is X?" ] );
  assert!(
    output.contains( "/-feature" ),
    "ask --subdir feature must produce path ending in /-feature. Got:\n{output}"
  );
}

// IT-11: CLR_EFFORT env var overrides ask soft default (BUG-245 regression guard).
//
// Root cause: soft defaults (cli.effort.or(Some(High))) ran before apply_env_vars;
//   the None-sentinel was replaced by the ask default so the env-var guard
//   (if parsed.effort.is_none()) misfired and silently ignored CLR_EFFORT.
// Fix: apply_env_vars called before soft defaults in dispatch_ask.
// Pitfall: priority chain must be CLI flag > CLR_* env var > ask default;
//   placing soft defaults before env-var application reverses env/default priority.
#[ test ]
fn it_11_clr_effort_env_overrides_ask_default()
{
  let out = run_cli_with_env(
    &[ "ask", "--dry-run", "What is X?" ],
    &[ ( "CLR_EFFORT", "low" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_EFFORT=low clr ask --dry-run should exit 0"
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--effort low" ),
    "CLR_EFFORT=low must override ask default (high). Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "--effort high" ),
    "ask must not inject --effort high when CLR_EFFORT=low. Got:\n{stdout}"
  );
}

// IT-12: CLR_MAX_TOKENS env var overrides ask soft default (BUG-245 regression guard).
//
// Root cause: same ordering bug as IT-11 — soft default (16384) filled the field
//   before apply_env_vars could apply CLR_MAX_TOKENS.
// Fix: apply_env_vars called before soft defaults in dispatch_ask.
#[ test ]
fn it_12_clr_max_tokens_env_overrides_ask_default()
{
  let out = run_cli_with_env(
    &[ "ask", "--dry-run", "What is X?" ],
    &[ ( "CLR_MAX_TOKENS", "50000" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_MAX_TOKENS=50000 clr ask --dry-run should exit 0"
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=50000" ),
    "CLR_MAX_TOKENS=50000 must override ask default (16384). Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=16384" ),
    "ask must not inject 16384 when CLR_MAX_TOKENS=50000. Got:\n{stdout}"
  );
}
