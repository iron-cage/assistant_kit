//! Integration tests: X (Cross-Cutting), E (Environment).

use crate::helpers::{
  run_cs, run_cs_with_env, run_cs_without_home,
  stdout, stderr, assert_exit,
  write_credentials, write_account, account_exists,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── X: Cross-Cutting ──────────────────────────────────────────────────────────

#[ test ]
fn x01_list_idempotent()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work", "pro", "standard", FAR_FUTURE_MS, true );

  let out1 = run_cs_with_env( &[ ".account.list", "v::0" ], &[ ( "HOME", home ) ] );
  let out2 = run_cs_with_env( &[ ".account.list", "v::0" ], &[ ( "HOME", home ) ] );
  assert_eq!( stdout( &out1 ), stdout( &out2 ), "list must be idempotent" );
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

  let _ = run_cs_with_env( &[ ".account.save", "name::x" ], &[ ( "HOME", home ) ] );
  let content1 = std::fs::read_to_string(
    dir.path().join( ".claude" ).join( "accounts" ).join( "x.credentials.json" )
  ).unwrap();

  let _ = run_cs_with_env( &[ ".account.save", "name::x" ], &[ ( "HOME", home ) ] );
  let content2 = std::fs::read_to_string(
    dir.path().join( ".claude" ).join( "accounts" ).join( "x.credentials.json" )
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
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work", "pro", "standard", FAR_FUTURE_MS, true );

  let a = run_cs_with_env( &[ ".account.list", "v::0", "format::json" ], &[ ( "HOME", home ) ] );
  let b = run_cs_with_env( &[ ".account.list", "format::json", "v::0" ], &[ ( "HOME", home ) ] );
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
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  // .account.status needs an _active marker to exit 0
  let accounts = dir.path().join( ".claude" ).join( "accounts" );
  std::fs::create_dir_all( &accounts ).unwrap();
  std::fs::write( accounts.join( "_active" ), "solo" ).unwrap();

  for cmd in &[ ".account.list", ".account.status", ".token.status", ".paths", ".credentials.status" ]
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
  write_account( dir.path(), "target", "pro", "standard", FAR_FUTURE_MS, false );

  for cmd in &[ ".account.save", ".account.switch", ".account.delete" ]
  {
    let out = run_cs_with_env( &[ cmd, "name::target", "dry::1" ], &[ ( "HOME", home ) ] );
    assert_exit( &out, 0 );
  }
}

#[ test ]
fn x09_every_command_has_exit_0_path()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  // "base" is active (required for .account.status exit 0)
  write_account( dir.path(), "base", "pro", "standard", FAR_FUTURE_MS, true );
  // "target" is inactive — delete/switch/save dry-run must use a non-active account
  write_account( dir.path(), "target", "pro", "standard", FAR_FUTURE_MS, false );

  let commands = vec![
    vec![ ".help" ],
    vec![ ".account.list" ],
    vec![ ".account.status" ],
    vec![ ".account.save", "name::target", "dry::1" ],
    vec![ ".account.switch", "name::target", "dry::1" ],
    vec![ ".account.delete", "name::target", "dry::1" ],
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
  let out = run_cs( &[ ".account.list", "format::xml" ] );
  assert_exit( &out, 1 );
  assert!( !stderr( &out ).is_empty(), "usage error must produce stderr" );
  assert!( stdout( &out ).is_empty(), "usage error must produce empty stdout" );
}

#[ test ]
fn x11_runtime_error_exits_2_stderr_nonempty()
{
  let out = run_cs_without_home( &[ ".account.list" ] );
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
  for cmd in &[ ".account.list", ".token.status", ".paths", ".credentials.status" ]
  {
    let out = run_cs_without_home( &[ cmd ] );
    assert_exit( &out, 2 );
  }
}

#[ test ]
fn e03_home_empty_exits_2()
{
  let out = run_cs_with_env( &[ ".account.list" ], &[ ( "HOME", "" ) ] );
  assert_exit( &out, 2 );
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
fn e05_claude_dir_absent_list_empty()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // No .claude dir at all — create it for require_claude_paths to succeed
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.list" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

#[ test ]
fn e06_claude_dir_absent_save_autocreate()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  // accounts dir does NOT exist

  let out = run_cs_with_env( &[ ".account.save", "name::x" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( account_exists( dir.path(), "x" ) );
}

#[ test ]
fn e07_format_yaml_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.list", "format::yaml" ], &[ ( "HOME", home ) ] );
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
