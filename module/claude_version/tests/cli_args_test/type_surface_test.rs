//! Type surface tests — validate the public type contract of each parameter type.
//!
//! These tests exercise the type system boundary: accepted values, rejected values,
//! and semantic invariants for `VerbosityLevel`, `OutputFormat`, `VersionSpec`,
//! `SettingsKey`, and `SettingsValue`.
//!
//! ## `VerbosityLevel`
//! | Function | Category |
//! |----------|----------|
//! | `tc_verbosity_level_0_minimal` | Level semantics |
//! | `tc_verbosity_level_2_verbose` | Level semantics |
//! | `tc_verbosity_level_3_out_of_range` | Range validation |
//! | `tc_verbosity_level_abc_non_integer` | Type validation |
//!
//! ## `OutputFormat`
//! | Function | Category |
//! |----------|----------|
//! | `tc_output_format_text_explicit` | Valid variant |
//! | `tc_output_format_xml_rejected` | Unknown variant |
//! | `tc_output_format_empty_rejected` | Empty validation |
//!
//! ## `VersionSpec`
//! | Function | Category |
//! |----------|----------|
//! | `tc_version_spec_month_alias_accepted` | Named alias |
//! | `tc_version_spec_latest_alias_accepted` | Named alias |
//!
//! ## `SettingsKey`
//! | Function | Category |
//! |----------|----------|
//! | `tc_settings_key_empty_exits_1` | Empty validation |
//! | `tc_settings_key_absent_exits_1` | Required field |
//! | `tc_settings_key_dot_literal` | Dot semantics |
//! | `tc_settings_key_valid_accepted` | Valid key |
//!
//! ## `SettingsValue`
//! | Function | Category |
//! |----------|----------|
//! | `tc_settings_value_empty_exits_1` | Empty validation |
//! | `tc_settings_value_absent_exits_1` | Required field |

use crate::subprocess_helpers::{ assert_container, run, out_stdout, code };

// ─── Type Surface: VerbosityLevel ────────────────────────────────────────────

// Type test: v::0 produces minimal output (no labels, raw values only)
#[ test ]
fn tc_verbosity_level_0_minimal()
{
  let out = run( &[ ".status", "v::0" ] );
  assert_eq!( code( &out ), 0, "v::0 must exit 0" );
  let text = out_stdout( &out );
  assert!( !text.contains( "Version:" ), "v::0 must not show labels: {text}" );
}

// Type test: v::2 produces verbose output (more detail than v::1)
// Requires a preferred version in settings for the Preferred line to appear.
#[ test ]
fn tc_verbosity_level_2_verbose()
{
  assert_container();
  let dir = tempfile::TempDir::new().expect( "tmpdir" );
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write(
    claude_dir.join( "settings.json" ),
    r#"{ "preferredVersionSpec": "stable", "preferredVersionResolved": "2.1.78" }"#,
  ).unwrap();
  let bin = env!( "CARGO_BIN_EXE_claude_version" );
  let out_v1 = std::process::Command::new( bin )
    .args( [ ".status", "v::1" ] )
    .env( "HOME", dir.path() )
    .output()
    .expect( "run v1" );
  let out_v2 = std::process::Command::new( bin )
    .args( [ ".status", "v::2" ] )
    .env( "HOME", dir.path() )
    .output()
    .expect( "run v2" );
  assert_eq!( code( &out_v1 ), 0 );
  assert_eq!( code( &out_v2 ), 0 );
  let text_v1 = out_stdout( &out_v1 );
  let text_v2 = out_stdout( &out_v2 );
  assert!(
    text_v2.len() > text_v1.len(),
    "v::2 must produce more output than v::1: v1={} bytes, v2={} bytes",
    text_v1.len(), text_v2.len()
  );
}

// Type test: v::3 is out of range (valid: 0, 1, 2)
#[ test ]
fn tc_verbosity_level_3_out_of_range()
{
  let out = run( &[ ".status", "v::3" ] );
  assert_eq!( code( &out ), 1, "v::3 must exit 1 (out of range)" );
}

// Type test: v::abc is non-integer → rejected
#[ test ]
fn tc_verbosity_level_abc_non_integer()
{
  let out = run( &[ ".status", "v::abc" ] );
  assert_eq!( code( &out ), 1, "v::abc must exit 1 (non-integer)" );
}

// ─── Type Surface: OutputFormat ──────────────────────────────────────────────

// Type test: format::text explicitly produces human-readable labeled output
#[ test ]
fn tc_output_format_text_explicit()
{
  let out = run( &[ ".status", "format::text" ] );
  assert_eq!( code( &out ), 0, "format::text must exit 0" );
  let text = out_stdout( &out );
  assert!( !text.trim_start().starts_with( '{' ), "format::text must not produce JSON: {text}" );
}

// Type test: format::xml is unknown variant → rejected
#[ test ]
fn tc_output_format_xml_rejected()
{
  let out = run( &[ ".status", "format::xml" ] );
  assert_eq!( code( &out ), 1, "format::xml must exit 1 (unknown variant)" );
}

// Type test: format:: (empty value) → rejected
#[ test ]
fn tc_output_format_empty_rejected()
{
  let out = run( &[ ".status", "format::" ] );
  assert_eq!( code( &out ), 1, "format:: (empty) must exit 1" );
}

// ─── Type Surface: VersionSpec ───────────────────────────────────────────────

// Type test: version::month alias accepted by install
#[ test ]
fn tc_version_spec_month_alias_accepted()
{
  let out = run( &[ ".version.install", "version::month", "dry::1" ] );
  assert_eq!( code( &out ), 0, "version::month must exit 0" );
  let text = out_stdout( &out );
  assert!( text.contains( "[dry-run]" ), "must show dry-run: {text}" );
}

// Type test: version::latest alias accepted by install
#[ test ]
fn tc_version_spec_latest_alias_accepted()
{
  let out = run( &[ ".version.install", "version::latest", "dry::1" ] );
  assert_eq!( code( &out ), 0, "version::latest must exit 0" );
  let text = out_stdout( &out );
  assert!( text.contains( "[dry-run]" ), "must show dry-run: {text}" );
}

// ─── Type Surface: SettingsKey ───────────────────────────────────────────────

// Type test: key:: (empty value) → exit 1
#[ test ]
fn tc_settings_key_empty_exits_1()
{
  let out = run( &[ ".settings.get", "key::" ] );
  assert_eq!( code( &out ), 1, "key:: (empty) must exit 1" );
}

// Type test: missing key:: parameter entirely → exit 1
#[ test ]
fn tc_settings_key_absent_exits_1()
{
  let out = run( &[ ".settings.get" ] );
  assert_eq!( code( &out ), 1, "missing key:: must exit 1" );
}

// Type test: key::api.endpoint — dot character is literal, not path separator
#[ test ]
fn tc_settings_key_dot_literal()
{
  assert_container();
  let dir = tempfile::TempDir::new().expect( "tmpdir" );
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write(
    claude_dir.join( "settings.json" ),
    r#"{ "api.endpoint": "v1" }"#,
  ).unwrap();
  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_claude_version" ) )
    .args( [ ".settings.get", "key::api.endpoint" ] )
    .env( "HOME", dir.path() )
    .output()
    .expect( "failed to run clv" );
  assert_eq!( code( &out ), 0, "key::api.endpoint must exit 0" );
  let text = out_stdout( &out );
  assert!( text.contains( "v1" ), "must retrieve dot-named key value: {text}" );
}

// Type test: key::theme — valid simple key accepted
#[ test ]
fn tc_settings_key_valid_accepted()
{
  assert_container();
  let dir = tempfile::TempDir::new().expect( "tmpdir" );
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write(
    claude_dir.join( "settings.json" ),
    r#"{ "theme": "dark" }"#,
  ).unwrap();
  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_claude_version" ) )
    .args( [ ".settings.get", "key::theme" ] )
    .env( "HOME", dir.path() )
    .output()
    .expect( "failed to run clv" );
  assert_eq!( code( &out ), 0, "key::theme must exit 0" );
  let text = out_stdout( &out );
  assert!( text.contains( "dark" ), "must retrieve key value: {text}" );
}

// ─── Type Surface: SettingsValue (validation) ────────────────────────────────

// Type test: value:: (empty) → exit 1
#[ test ]
fn tc_settings_value_empty_exits_1()
{
  let out = run( &[ ".settings.set", "key::probe", "value::" ] );
  assert_eq!( code( &out ), 1, "value:: (empty) must exit 1" );
}

// Type test: missing value:: parameter entirely → exit 1
#[ test ]
fn tc_settings_value_absent_exits_1()
{
  let out = run( &[ ".settings.set", "key::probe" ] );
  assert_eq!( code( &out ), 1, "missing value:: must exit 1" );
}
