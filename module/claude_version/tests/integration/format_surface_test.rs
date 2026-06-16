//! Format surface tests (FM- prefix) for `claude_version`.
//!
//! Implements test cases from `tests/docs/cli/format/` spec files.
//! Each function maps to one FM- case verifying output format rendering.
//!
//! # Coverage Map
//!
//! | FM-spec | ID | Function |
//! |---------|----|----------|
//! | cli/format/01_text.md | FM-1 | `fm01_01_text_default_labeled` |
//! | cli/format/01_text.md | FM-2 | `fm02_01_text_v0_raw` |
//! | cli/format/01_text.md | FM-3 | `fm03_01_text_v1_labeled` |
//! | cli/format/01_text.md | FM-4 | `fm04_01_text_not_json` |
//! | cli/format/02_json.md | FM-1 | `fm01_02_json_object_output` |
//! | cli/format/02_json.md | FM-2 | `fm02_02_json_array_output` |
//! | cli/format/02_json.md | FM-3 | `fm03_02_json_case_sensitive` |
//! | cli/format/02_json.md | FM-4 | `fm04_02_json_v0_primary_key` |

use crate::helpers::{ assert_exit, run_clm, stderr, stdout };

// ─── FM-1 (cli/format/01_text.md): default text format is labeled ─────────────

// FM-1: no format:: arg → text format with Version: label
// Conditional: .version.show requires claude in PATH; skip if absent (exit 2).
#[ test ]
fn fm01_01_text_default_labeled()
{
  let out = run_clm( &[ ".version.show" ] );
  if out.status.code() == Some( 0 )
  {
    let text = stdout( &out );
    assert!( text.contains( "Version:" ), "default text format must include Version: label: {text}" );
    assert!( !text.starts_with( '{' ), "default text output must not be JSON: {text}" );
  }
}

// ─── FM-2 (cli/format/01_text.md): v::0 suppresses labels ────────────────────

// FM-2: v::0 → bare version string, no Version: label
// Conditional: .version.show requires claude in PATH; skip if absent (exit 2).
#[ test ]
fn fm02_01_text_v0_raw()
{
  let out = run_clm( &[ ".version.show", "v::0" ] );
  if out.status.code() == Some( 0 )
  {
    let text = stdout( &out );
    assert!( !text.contains( "Version:" ), "v::0 must suppress Version: label: {text}" );
    // Output should still contain a semver-like string (digits and dots)
    assert!( text.chars().any( |c| c.is_ascii_digit() ), "v::0 output must contain version digits: {text}" );
  }
}

// ─── FM-3 (cli/format/01_text.md): v::1 produces labeled output ──────────────

// FM-3: v::1 → Version: label present
// Conditional: .version.show requires claude in PATH; skip if absent (exit 2).
#[ test ]
fn fm03_01_text_v1_labeled()
{
  let out = run_clm( &[ ".version.show", "v::1" ] );
  if out.status.code() == Some( 0 )
  {
    let text = stdout( &out );
    assert!( text.contains( "Version:" ), "v::1 must include Version: label: {text}" );
  }
}

// ─── FM-4 (cli/format/01_text.md): text output is not JSON ───────────────────

// FM-4: .status default → text output does not begin with { or [
#[ test ]
fn fm04_01_text_not_json()
{
  let out = run_clm( &[ ".status" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.trim_start().starts_with( '{' ) && !text.trim_start().starts_with( '[' ),
    "default text output must not be a JSON object or array: {text}"
  );
}

// ─── FM-1 (cli/format/02_json.md): single-result command produces JSON object ─

// FM-1: format::json on .status → valid JSON object with version key
#[ test ]
fn fm01_02_json_object_output()
{
  let out = run_clm( &[ ".status", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim_start().starts_with( '{' ), "format::json must produce JSON object: {text}" );
  assert!( text.contains( "\"version\"" ), "JSON object must contain version key: {text}" );
}

// ─── FM-2 (cli/format/02_json.md): list command produces JSON array ───────────

// FM-2: format::json on .version.list → valid JSON array starting with [
#[ test ]
fn fm02_02_json_array_output()
{
  let out = run_clm( &[ ".version.list", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim_start().starts_with( '[' ), "format::json on list command must produce JSON array: {text}" );
}

// ─── FM-3 (cli/format/02_json.md): uppercase JSON value is rejected ───────────

// FM-3: format::JSON (uppercase) → exit 1
#[ test ]
fn fm03_02_json_case_sensitive()
{
  let out = run_clm( &[ ".status", "format::JSON" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( !err.is_empty(), "format::JSON rejection must produce error message: {err}" );
}

// ─── FM-4 (cli/format/02_json.md): v::0 with json — primary key always present ─

// FM-4: format::json v::0 → JSON object; primary payload key (version) present
#[ test ]
fn fm04_02_json_v0_primary_key()
{
  let out = run_clm( &[ ".status", "format::json", "v::0" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim_start().starts_with( '{' ), "format::json must produce JSON object: {text}" );
  assert!( text.contains( "\"version\"" ), "JSON at v::0 must still include version key: {text}" );
}
