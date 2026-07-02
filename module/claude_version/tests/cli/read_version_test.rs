//! Integration tests for `.version.show` and `.version.list` — E3, E4.
//!
//! ## E3 — `.version.show`
//! | TC | Description | P/N | Exit |
//! |----|-------------|-----|------|
//! | TC-107 | `.version.show` without claude in PATH → exits 2 | N | 2 |
//! | TC-108 | `.version.show v::0` → bare version string | P | 0 |
//! | TC-109 | `.version.show v::1` → "Version: X.Y.Z" | P | 0 |
//! | TC-111 | `.version.show format::json` → {"version":"..."} | P | 0 |
//!
//! ## E4 — `.version.list`
//! | TC | Description | P/N | Exit |
//! |----|-------------|-----|------|
//! | TC-115 | `.version.list` exits 0 | P | 0 |
//! | TC-116 | `.version.list` output includes "stable" | P | 0 |
//! | TC-117 | `.version.list` output includes "latest" | P | 0 |
//! | TC-118 | `.version.list v::0` → one alias per line, no descriptions | P | 0 |
//! | TC-119 | `.version.list v::1` → aliases with descriptions | P | 0 |
//! | TC-120 | `.version.list` output identical on two calls | P | 0 |
//! | TC-121 | `.version.list format::json` → valid JSON array | P | 0 |
//! | TC-122 | `.version.list` includes "month" alias | P | 0 |
//! | TC-123 | `.version.list v::1` shows pinned version in parens | P | 0 |
//! | TC-124 | `.version.list format::json` has "value" field | P | 0 |
//! | IT-4 | `bogus::x` → exit 1, unknown parameter | N | 1 |
//! | IT-5 | `format::xml` → exit 1, unknown format | N | 1 |
//! | IT-6 | `v::3` → exit 1, out of range | N | 1 |
//! | IT-7 | `format::json` → valid JSON output | P | 0 |
//! | IT-8 | Output stable across 3 invocations | P | 0 |

use tempfile::TempDir;

use crate::subprocess_helpers::{ assert_exit, run_clv, run_clv_with_env, stdout };

// ─── E3: version show ────────────────────────────────────────────────────────

// TC-107: no symlink + empty PATH → exit 2
#[ test ]
fn tc107_version_show_no_claude_exits_2()
{
  let dir = TempDir::new().unwrap();
  let fake_home = dir.path().to_str().unwrap();
  let out = run_clv_with_env(
    &[ ".version.show" ],
    &[ ( "PATH", "" ), ( "HOME", fake_home ) ],
  );
  assert_exit( &out, 2 );
}

// TC-108: v::0 → bare version string (requires claude)
#[ test ]
fn tc108_version_show_v0_bare_string()
{
  let out = run_clv( &[ ".version.show", "v::0" ] );
  if out.status.code() == Some( 0 )
  {
    let text = stdout( &out );
    let trimmed = text.trim();
    assert!(
      trimmed.chars().all( | c | c.is_ascii_digit() || c == '.' ),
      "v::0 must be bare semver, got: {trimmed}"
    );
  }
}

// TC-109: v::1 → "Version: X.Y.Z"
#[ test ]
fn tc109_version_show_v1_labeled()
{
  let out = run_clv( &[ ".version.show", "v::1" ] );
  if out.status.code() == Some( 0 )
  {
    let text = stdout( &out );
    assert!( text.contains( "Version:" ), "v::1 must contain 'Version:' label, got: {text}" );
  }
}

// TC-111: format::json → {"version":"..."}
#[ test ]
fn tc111_version_show_format_json()
{
  let out = run_clv( &[ ".version.show", "format::json" ] );
  if out.status.code() == Some( 0 )
  {
    let text = stdout( &out );
    assert!( text.contains( "\"version\"" ), "missing 'version' key in JSON: {text}" );
  }
}

// ─── E4: version list ────────────────────────────────────────────────────────

// TC-115
#[ test ]
fn tc115_version_list_exits_0()
{
  let out = run_clv( &[ ".version.list" ] );
  assert_exit( &out, 0 );
}

// TC-116
#[ test ]
fn tc116_version_list_includes_stable()
{
  let out = run_clv( &[ ".version.list" ] );
  assert_exit( &out, 0 );
  assert!( stdout( &out ).contains( "stable" ), "output must contain 'stable'" );
}

// TC-117
#[ test ]
fn tc117_version_list_includes_latest()
{
  let out = run_clv( &[ ".version.list" ] );
  assert_exit( &out, 0 );
  assert!( stdout( &out ).contains( "latest" ), "output must contain 'latest'" );
}

// TC-118: v::0 → names only (no descriptions or dashes)
#[ test ]
fn tc118_version_list_v0_names_only()
{
  let out = run_clv( &[ ".version.list", "v::0" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for line in text.lines()
  {
    assert!(
      !line.contains( " \u{2014} " ),
      "v::0 must not contain descriptions, got line: {line}"
    );
  }
}

// TC-119: v::1 → aliases with descriptions
#[ test ]
fn tc119_version_list_v1_has_descriptions()
{
  let out = run_clv( &[ ".version.list", "v::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( " \u{2014} " ) || text.contains( ": " ),
    "v::1 must include descriptions, got: {text}" );
}

// TC-120: identical on two consecutive calls
#[ test ]
fn tc120_version_list_is_idempotent()
{
  let out1 = run_clv( &[ ".version.list" ] );
  let out2 = run_clv( &[ ".version.list" ] );
  assert_exit( &out1, 0 );
  assert_exit( &out2, 0 );
  assert_eq!( stdout( &out1 ), stdout( &out2 ), "version list must be deterministic" );
}

// TC-121: format::json → valid JSON array
#[ test ]
fn tc121_version_list_format_json_array()
{
  let out = run_clv( &[ ".version.list", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.trim_start().starts_with( '[' ) || text.contains( "\"aliases\"" ),
    "format::json must produce a JSON array or object with 'aliases', got: {text}"
  );
}

// TC-122: version list includes month alias
#[ test ]
fn tc122_version_list_includes_month()
{
  let out = run_clv( &[ ".version.list" ] );
  assert_exit( &out, 0 );
  assert!( stdout( &out ).contains( "month" ), "output must contain 'month'" );
}

// TC-123: .version.list v::1 shows pinned versions for stable and month
#[ test ]
fn tc123_version_list_v1_shows_pinned_versions()
{
  let out = run_clv( &[ ".version.list", "v::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "(v" ), "v::1 must show pinned version in parens, got: {text}" );
}

// TC-124: .version.list format::json includes value field for pinned aliases
#[ test ]
fn tc124_version_list_json_has_value_field()
{
  let out = run_clv( &[ ".version.list", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"value\"" ), "JSON must have 'value' field for pinned aliases: {text}" );
}

// IT-4: `bogus::x` → exit 1, unknown parameter
#[ test ]
fn it04_version_list_bogus_param_exits_1()
{
  let out = run_clv( &[ ".version.list", "bogus::x" ] );
  assert_exit( &out, 1 );
}

// IT-5: `format::xml` → exit 1, unknown format
#[ test ]
fn it05_version_list_format_xml_exits_1()
{
  let out = run_clv( &[ ".version.list", "format::xml" ] );
  assert_exit( &out, 1 );
}

// IT-6: `v::3` → exit 1, out of range
#[ test ]
fn it06_version_list_v3_exits_1()
{
  let out = run_clv( &[ ".version.list", "v::3" ] );
  assert_exit( &out, 1 );
}

// IT-7: `format::json` → valid JSON output (starts with `[` or `{`)
#[ test ]
fn it07_version_list_format_json_valid()
{
  let out = run_clv( &[ ".version.list", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let t = text.trim_start();
  assert!(
    t.starts_with( '[' ) || t.starts_with( '{' ),
    "format::json output must be valid JSON array or object: {text}"
  );
}

// IT-8: output is stable across 3 successive invocations
#[ test ]
fn it08_version_list_output_stable()
{
  let out1 = run_clv( &[ ".version.list" ] );
  let out2 = run_clv( &[ ".version.list" ] );
  let out3 = run_clv( &[ ".version.list" ] );
  assert_exit( &out1, 0 );
  assert_exit( &out2, 0 );
  assert_exit( &out3, 0 );
  let t1 = stdout( &out1 );
  let t2 = stdout( &out2 );
  let t3 = stdout( &out3 );
  assert_eq!( t1, t2, "version.list must be deterministic on consecutive calls" );
  assert_eq!( t2, t3, "version.list must be deterministic on consecutive calls" );
}
