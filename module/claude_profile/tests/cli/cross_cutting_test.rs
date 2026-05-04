//! Integration tests: X (Cross-Cutting), E (Environment).
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! ## Test Matrix
//!
//! ### X — Cross-Cutting Behavior
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | x01 | `x01_accounts_idempotent` | .accounts called twice → identical output | P |
//! | x02 | `x02_paths_idempotent` | paths called twice → identical output | P |
//! | x03 | `x03_save_twice_same_result` | save same name twice → same file | P |
//! | x04 | `x04_token_status_idempotent` | token.status called twice → identical output | P |
//! | x05 | `x05_param_order_independence_list` | params reordered → same output | P |
//! | x06 | `x06_param_order_independence_token` | params reordered → same output | P |
//! | x07 | `x07_read_commands_accept_v_and_format` | commands accepting `v::` and `format::` | P |
//! | x08 | `x08_mutation_commands_accept_name_and_dry` | mutation commands accept `name::` and `dry::` | P |
//! | x09 | `x09_every_command_has_exit_0_path` | every command has at least one success path | P |
//! | x10 | `x10_usage_error_exits_1_stderr_nonempty` | usage error → exit 1 + stderr | N |
//! | x11 | `x11_runtime_error_exits_2_stderr_nonempty` | runtime error → exit 2 + stderr | N |
//! | x12 | `x12_unknown_command_exits_1` | unknown command → exit 1 | N |
//! | x13 | `x13_success_stdout_nonempty_stderr_empty` | success → stdout non-empty, stderr empty | P |
//! | x14 | `x14_error_stdout_empty_stderr_nonempty` | error → stdout empty, stderr non-empty | N |
//!
//! ### E — Environment
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | e01 | `e01_home_valid_normal_operation` | valid HOME → normal operation | P |
//! | e02 | `e02_home_unset_all_commands_exit_2` | HOME unset → all commands exit 2 | N |
//! | e03 | `e03_home_empty_exits_2` | HOME="" → exit 2 | N |
//! | e03b | `e03b_accounts_home_empty_exits_0` | HOME="" + .accounts → exit 0 advisory | P |
//! | e04 | `e04_home_with_spaces` | HOME path with spaces → works | P |
//! | e05 | `e05_credential_store_absent_list_empty` | credential store absent → list returns empty | P |
//! | e06 | `e06_claude_dir_absent_save_autocreate` | .claude/ absent → save autocreates | P |
//! | e07 | `e07_format_yaml_exits_1` | format::yaml → exit 1 | N |
//! | e08 | `e08_format_csv_exits_1` | format::csv → exit 1 | N |
//! | e09 | `e09_empty_name_value_exits_1` | name:: with empty value → exit 1 | N |
//! | e10 | `e10_accounts_home_unset_exits_0` | HOME unset + .accounts → exit 0 advisory | P |
//! | e11 | `e11_fmt_alias_accounts_json` | fmt::json → .accounts outputs JSON array | P |
//! | e12 | `e12_fmt_alias_token_status_json` | fmt::json → .token.status outputs JSON object | P |

use crate::helpers::{
  run_cs, run_cs_with_env, run_cs_without_home,
  stdout, stderr, assert_exit,
  write_credentials, write_account, account_exists,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── X: Cross-Cutting ──────────────────────────────────────────────────────────

#[ test ]
fn x01_accounts_idempotent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@example.com", "pro", "standard", FAR_FUTURE_MS, true );

  let args = &[ ".accounts", "active::0", "sub::0", "tier::0", "expires::0", "org::0" ];
  let out1 = run_cs_with_env( args, &[ ( "HOME", home ) ] );
  let out2 = run_cs_with_env( args, &[ ( "HOME", home ) ] );
  assert_eq!( stdout( &out1 ), stdout( &out2 ), ".accounts must be idempotent" );
}

#[ test ]
fn x02_paths_idempotent()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out1 = run_cs_with_env( &[ ".paths", "v::0" ], &[ ( "HOME", home ) ] );
  let out2 = run_cs_with_env( &[ ".paths", "v::0" ], &[ ( "HOME", home ) ] );
  assert_eq!( stdout( &out1 ), stdout( &out2 ), "paths must be idempotent" );
}

#[ test ]
fn x03_save_twice_same_result()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let _ = run_cs_with_env( &[ ".account.save", "name::x@example.com" ], &[ ( "HOME", home ) ] );
  let content1 = std::fs::read_to_string(
    dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ).join( "x@example.com.credentials.json" )
  ).unwrap();

  let _ = run_cs_with_env( &[ ".account.save", "name::x@example.com" ], &[ ( "HOME", home ) ] );
  let content2 = std::fs::read_to_string(
    dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ).join( "x@example.com.credentials.json" )
  ).unwrap();

  assert_eq!( content1, content2, "saving twice must produce same file" );
}

#[ test ]
fn x04_token_status_idempotent()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out1 = run_cs_with_env( &[ ".token.status", "v::0" ], &[ ( "HOME", home ) ] );
  let out2 = run_cs_with_env( &[ ".token.status", "v::0" ], &[ ( "HOME", home ) ] );
  assert_eq!( stdout( &out1 ), stdout( &out2 ), "token status must be idempotent" );
}

#[ test ]
fn x05_param_order_independence_list()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@example.com", "pro", "standard", FAR_FUTURE_MS, true );

  let a = run_cs_with_env( &[ ".accounts", "sub::0", "format::json" ], &[ ( "HOME", home ) ] );
  let b = run_cs_with_env( &[ ".accounts", "format::json", "sub::0" ], &[ ( "HOME", home ) ] );
  assert_eq!( stdout( &a ), stdout( &b ), "param order must not matter" );
}

#[ test ]
fn x06_param_order_independence_token()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let a = run_cs_with_env( &[ ".token.status", "threshold::1800", "v::2" ], &[ ( "HOME", home ) ] );
  let b = run_cs_with_env( &[ ".token.status", "v::2", "threshold::1800" ], &[ ( "HOME", home ) ] );
  assert_eq!( stdout( &a ), stdout( &b ), "param order must not matter" );
}

#[ test ]
fn x07_read_commands_accept_v_and_format()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  // NOTE: `.accounts` uses field-presence booleans (no `v::`); `.credentials.status` also uses
  //   field-presence booleans — both are excluded here; their happy paths are covered by
  //   `x09_every_command_has_exit_0_path`.
  for cmd in &[ ".token.status", ".paths" ]
  {
    let out = run_cs_with_env( &[ cmd, "v::0", "format::text" ], &[ ( "HOME", home ) ] );
    assert_exit( &out, 0 );
  }
}

#[ test ]
fn x08_mutation_commands_accept_name_and_dry()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "target@example.com", "pro", "standard", FAR_FUTURE_MS, false );

  for cmd in &[ ".account.save", ".account.switch", ".account.delete" ]
  {
    let out = run_cs_with_env( &[ cmd, "name::target@example.com", "dry::1" ], &[ ( "HOME", home ) ] );
    assert_exit( &out, 0 );
  }
}

#[ test ]
fn x09_every_command_has_exit_0_path()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  // "target@example.com" is inactive — delete/switch/save dry-run must use a non-active account
  write_account( dir.path(), "target@example.com", "pro", "standard", FAR_FUTURE_MS, false );

  let commands = vec![
    vec![ ".help" ],
    vec![ ".accounts" ],
    vec![ ".account.save",   "name::target@example.com", "dry::1" ],
    vec![ ".account.switch", "name::target@example.com", "dry::1" ],
    vec![ ".account.delete", "name::target@example.com", "dry::1" ],
    vec![ ".token.status" ],
    vec![ ".paths" ],
    vec![ ".credentials.status" ],
  ];
  for args in &commands
  {
    let str_args : Vec< &str > = args.iter().map( core::convert::AsRef::as_ref ).collect();
    let out = run_cs_with_env( &str_args, &[ ( "HOME", home ) ] );
    assert_exit( &out, 0 );
  }
}

#[ test ]
fn x10_usage_error_exits_1_stderr_nonempty()
{
  let out = run_cs( &[ ".accounts", "format::xml" ] );
  assert_exit( &out, 1 );
  assert!( !stderr( &out ).is_empty(), "usage error must produce stderr" );
  assert!( stdout( &out ).is_empty(), "usage error must produce empty stdout" );
}

#[ test ]
fn x11_runtime_error_exits_2_stderr_nonempty()
{
  let out = run_cs_without_home( &[ ".credentials.status" ] );
  assert_exit( &out, 2 );
  assert!( !stderr( &out ).is_empty(), "runtime error must produce stderr" );
  assert!( stdout( &out ).is_empty(), "runtime error must produce empty stdout" );
}

#[ test ]
fn x12_unknown_command_exits_1()
{
  let out = run_cs( &[ ".nonexistent" ] );
  assert_exit( &out, 1 );
  assert!( !stderr( &out ).is_empty(), "unknown command must produce stderr" );
}

#[ test ]
fn x13_success_stdout_nonempty_stderr_empty()
{
  let out = run_cs( &[ ".help" ] );
  assert_exit( &out, 0 );
  assert!( !stdout( &out ).is_empty(), "success must produce stdout" );
  assert!( stderr( &out ).is_empty(), "success must produce empty stderr" );
}

#[ test ]
fn x14_error_stdout_empty_stderr_nonempty()
{
  let out = run_cs( &[ ".nonexistent" ] );
  assert_exit( &out, 1 );
  assert!( stdout( &out ).is_empty(), "error must produce empty stdout" );
  assert!( !stderr( &out ).is_empty(), "error must produce stderr" );
}

// ── E: Environment ────────────────────────────────────────────────────────────

#[ test ]
fn e01_home_valid_normal_operation()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".token.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

#[ test ]
fn e02_home_unset_all_commands_exit_2()
{
  for cmd in &[ ".token.status", ".paths", ".credentials.status" ]
  {
    let out = run_cs_without_home( &[ cmd ] );
    assert_exit( &out, 2 );
  }
}

#[ test ]
fn e03_home_empty_exits_2()
{
  let out = run_cs_with_env( &[ ".paths" ], &[ ( "HOME", "" ) ] );
  assert_exit( &out, 2 );
}

#[ test ]
// Fix(issue-accounts-require-claude-paths):
// Root cause: require_claude_paths()?; in accounts_routine hard-fails when HOME="",
//   but .accounts is a graceful-read command — must return advisory, not exit 2.
// Pitfall: e05 does NOT catch this bug — e05 uses a valid tmpdir HOME where
//   require_claude_paths() succeeds even before the fix.
fn e03b_accounts_home_empty_exits_0()
{
  let out = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", "" ) ] );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "(no accounts configured)" ),
    "empty HOME must show advisory, got: {}",
    stdout( &out ),
  );
}

#[ test ]
fn e04_home_with_spaces()
{
  let dir = TempDir::new().unwrap();
  let space_path = dir.path().join( "path with spaces" );
  std::fs::create_dir_all( &space_path ).unwrap();
  let home = space_path.to_str().unwrap();

  let out = run_cs_with_env( &[ ".paths" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "path with spaces" ) );
}

#[ test ]
fn e05_credential_store_absent_list_empty()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // No credential store — .accounts uses require_credential_store(), not require_claude_paths(),
  // so it does not require .claude/ to exist; missing store → empty list → exit 0.

  let out = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

#[ test ]
fn e06_claude_dir_absent_save_autocreate()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  // accounts dir does NOT exist

  let out = run_cs_with_env( &[ ".account.save", "name::x@example.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( account_exists( dir.path(), "x@example.com" ) );
}

#[ test ]
fn e07_format_yaml_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".accounts", "format::yaml" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn e08_format_csv_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".token.status", "format::csv" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn e09_empty_name_value_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
// Fix(issue-accounts-home-unset):
// Root cause: require_credential_store() → PersistPaths::new() fails when HOME is completely
//   unset, because it requires $PRO or $HOME to be set. This propagates as exit 2, but
//   .accounts is a graceful-read command — must return advisory "(no accounts configured)"
//   with exit 0 when storage is unavailable, consistent with HOME="" behavior (e03b).
// Why Not Caught: e03b uses HOME="" which, in test environments with $PRO set, resolves
//   storage successfully via $PRO. HOME fully absent (env_remove) also misses this when $PRO
//   is available. The bug only triggers when BOTH $HOME and $PRO are unset (e.g. env -i).
// Fix Applied: accounts_routine wraps require_credential_store() in match and returns
//   advisory text on Err instead of propagating with ?.
// Prevention: For any graceful-read command, catch storage-unavailable errors at the
//   routine level — never let them exit 2 when an empty-result advisory is correct.
// Pitfall: e05 (absent store, valid HOME) does NOT catch this — store absence is handled
//   by crate::account::list() returning Ok([]); this bug is in require_credential_store().
fn e10_accounts_home_unset_exits_0()
{
  // run_cs_without_home removes HOME; if $PRO is also absent in the container this
  // triggers the PersistPaths::new() failure path that was previously exit 2.
  let out = run_cs_without_home( &[ ".accounts" ] );
  assert_exit( &out, 0 );
  // Must show advisory, not error text
  assert!(
    stdout( &out ).contains( "(no accounts configured)" ),
    "HOME unset must show advisory, got stdout: {:?}, stderr: {:?}",
    stdout( &out ),
    crate::helpers::stderr( &out ),
  );
}

#[ test ]
// Fix(issue-fmt-alias):
// Root cause: adapter.rs only expands v:: → verbosity:: but does not expand fmt:: → format::.
//   The unilang.commands.yaml declares fmt as an alias for format, but programmatic
//   registration in adapter.rs has no corresponding expansion — leaving fmt:: as an
//   unknown parameter, which causes exit 1 "Unknown parameter 'fmt'".
// Why Not Caught: x07 tests v:: and format:: directly; no test covered the fmt:: alias.
// Fix Applied: Added FORMAT_ALIAS/FORMAT_KEY constants and an expansion branch in
//   argv_to_unilang_tokens() parallel to the existing VERBOSITY_ALIAS branch.
// Prevention: When the YAML aliases list is updated, verify each alias has a matching
//   expansion branch in adapter.rs argv_to_unilang_tokens().
// Pitfall: The YAML file is metadata-only — changes there do NOT affect runtime behavior.
//   All alias expansion must be implemented in adapter.rs.
fn e11_fmt_alias_accounts_json()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // No accounts set up — must return empty JSON array, not "Unknown parameter 'fmt'"
  let out = run_cs_with_env( &[ ".accounts", "fmt::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.trim_start().starts_with( '[' ),
    "fmt::json must produce JSON array, got stdout: {:?}, stderr: {:?}",
    text,
    crate::helpers::stderr( &out ),
  );
}

#[ test ]
// test_kind: bug_reproducer(issue-fmt-alias)
fn e12_fmt_alias_token_status_json()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  // Must output JSON object, not "Unknown parameter 'fmt'"
  let out = run_cs_with_env( &[ ".token.status", "fmt::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.trim_start().starts_with( '{' ),
    "fmt::json must produce JSON object, got stdout: {:?}, stderr: {:?}",
    text,
    crate::helpers::stderr( &out ),
  );
}
