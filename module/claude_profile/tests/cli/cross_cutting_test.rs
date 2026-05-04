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
