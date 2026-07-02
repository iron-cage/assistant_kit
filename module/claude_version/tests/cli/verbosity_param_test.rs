//! EC- edge-case tests for the `v::` / `verbosity::` parameter.
//!
//! Covers gap cases EC-12 through EC-21 from `tests/docs/cli/param/04_v.md`.
//! EC-1 through EC-11 are covered in `cli_args_test.rs`, `read_commands_test.rs`,
//! and `error_messages_test.rs`.

use tempfile::TempDir;

use crate::subprocess_helpers::{ assert_exit, run_clv, run_clv_with_env, stdout, write_settings };

/// EC-12: `.status v::0` → 3 bare lines with no label prefixes
#[ test ]
fn verbosity_ec12_status_v0_bare_lines()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );
  let out  = run_clv_with_env( &[ ".status", "v::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "Version:" ), "v::0 must not have label prefixes: {text}" );
  assert!( !text.contains( "Processes:" ), "v::0 must not have label prefixes: {text}" );
}

/// EC-13: `.status v::1` → labeled output lines
#[ test ]
fn verbosity_ec13_status_v1_labeled_lines()
{
  let out = run_clv( &[ ".status", "v::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Version:" ) || text.contains( "Processes:" ) || text.contains( "Account:" ),
    "v::1 must produce labeled output: {text}"
  );
}

/// EC-14: `.version.show v::0` → bare semver string
#[ test ]
fn verbosity_ec14_version_show_v0_bare_semver()
{
  let out = run_clv( &[ ".version.show", "v::0" ] );
  // may exit 2 if claude not in PATH; only check label absent if exit 0
  if out.status.code() == Some( 0 )
  {
    let text = stdout( &out );
    assert!( !text.contains( "Version:" ), "v::0 must not have 'Version:' label: {text}" );
  }
}

/// EC-15: `.version.show v::1` → "Version: X.Y.Z" label
#[ test ]
fn verbosity_ec15_version_show_v1_labeled()
{
  let out = run_clv( &[ ".version.show", "v::1" ] );
  if out.status.code() == Some( 0 )
  {
    let text = stdout( &out );
    assert!( text.contains( "Version:" ), "v::1 must produce 'Version:' label: {text}" );
  }
}

/// EC-16: `.version.list v::0` → names only, no description separator
#[ test ]
fn verbosity_ec16_version_list_v0_names_only()
{
  let out = run_clv( &[ ".version.list", "v::0" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "stable" ), "v::0 list must include alias names: {text}" );
  assert!( !text.contains( " \u{2014} " ) && !text.contains( " — " ),
    "v::0 must not include description separator: {text}" );
}

/// EC-17: `.version.list v::1` → names plus descriptions
#[ test ]
fn verbosity_ec17_version_list_v1_with_descriptions()
{
  let out = run_clv( &[ ".version.list", "v::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "stable" ), "v::1 list must include stable alias: {text}" );
  assert!( text.len() > 30, "v::1 must produce richer output than just names: {text}" );
}

/// EC-18: `.settings.show v::0` → `key=value` compact format
#[ test ]
fn verbosity_ec18_settings_show_v0_compact_format()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "myKey", "myVal" ) ] );
  let out  = run_clv_with_env( &[ ".settings.show", "v::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "myKey=myVal" ), "v::0 must show key=value format: {text}" );
}

/// EC-19: `.settings.get v::0` → bare value only
#[ test ]
fn verbosity_ec19_settings_get_v0_bare_value()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "myKey", "myVal" ) ] );
  let out  = run_clv_with_env(
    &[ ".settings.get", "key::myKey", "v::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out ).trim().to_string();
  assert_eq!( text, "myVal", "v::0 must output bare value only, got: {text}" );
}

/// EC-20: `.version.history v::0 count::3` → bare version+date lines
#[ test ]
fn verbosity_ec20_history_v0_bare_lines()
{
  let out = run_clv( &[ ".version.history", "v::0", "count::3" ] );
  // allow exit 2 (network unavailable) — only verify format if exit 0
  if out.status.code() == Some( 0 )
  {
    let text = stdout( &out );
    assert!( !text.contains( "##" ), "v::0 must not contain ## markdown headers: {text}" );
    assert!( !text.contains( "Summary:" ), "v::0 must not contain label prefixes: {text}" );
  }
}

/// EC-21: `.version.history v::2 count::2` → full changelog with `##` headers
#[ test ]
fn verbosity_ec21_history_v2_full_changelog()
{
  let out = run_clv( &[ ".version.history", "v::2", "count::2" ] );
  if out.status.code() == Some( 0 )
  {
    let text = stdout( &out );
    assert!( text.contains( "##" ), "v::2 must include ## markdown headers: {text}" );
  }
}
