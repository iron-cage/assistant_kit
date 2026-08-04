//! Integration tests for `.version.show` and `.version.list` — E3, E4.
//!
//! ## E3 — `.version.show`
//! | TC | Description | P/N | Exit |
//! |----|-------------|-----|------|
//! | TC-107 | `.version.show` without claude in PATH → exits 2 | N | 2 |
//! | TC-108 | `.version.show v::0` → bare version string | P | 0 |
//! | TC-109 | `.version.show v::1` → `<semver>  [labels]` or bare semver when none match | P | 0 |
//! | TC-111 | `.version.show format::json` → {"version":"..."} | P | 0 |
//! | IT-9  | `.version.show v::1` → `[team-pin]` when custom marker matches installed | P | 0 |
//! | IT-10 | `.version.show v::1` with no markers → no brackets | P | 0 |
//! | IT-11 | `.version.show format::json` → includes `"labels"` array | P | 0 |
//! | IT-12 | `.version.show v::0` → bare version, no labels even when markers exist | P | 0 |
//! | FT-6  | Custom marker label annotation shown by `.version.show` (multi-label) | P | 0 |
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
//! | TC-123 | `.version.list v::1` shows pinned version in parens | P | 0 |
//! | TC-124 | `.version.list format::json` has "value" field | P | 0 |
//! | IT-4 | `bogus::x` → exit 1, unknown parameter | N | 1 |
//! | IT-5 | `format::xml` → exit 1, unknown format | N | 1 |
//! | IT-6 | `v::3` → exit 1, out of range | N | 1 |
//! | IT-7 | `format::json` → valid JSON output | P | 0 |
//! | IT-8 | Output stable across 3 invocations | P | 0 |
//! | IT-42 | `mode::aliases` explicit == default (absent) output, byte-identical | P | 0 |
//! | IT-43 | `mode::bogus` → exit 1, unrecognized mode | N | 1 |
//! | IT-44 | `mode::` (empty value) → exit 1 | N | 1 |
//! | IT-45 | `mode::History` (wrong case) → exit 1, case-sensitive | N | 1 |
//! | IT-46 | `count::` accepted under `mode::aliases`, has no effect on output | P | 0 |
//!
//! ## E15 — `.version.list mode::history`
//! | TC | Description | P/N | Exit |
//! |----|-------------|-----|------|
//! | IT-16 | `mode::history` defaults → exits 0 (fallback if offline) | P | 0 |
//! | IT-17 | `mode::history count::3` → ≤3 version entries | P | 0 |
//! | IT-18 | `mode::history count::0` → empty output | P | 0 |
//! | IT-19 | `mode::history v::0` → bare `{version}  {date}` lines | P | 0 |
//! | IT-20 | `mode::history v::1` → version + date + summary per line | P | 0 |
//! | IT-21 | `mode::history v::2` → full changelog with `##` headers | P | 0 |
//! | IT-22 | `mode::history format::json` → JSON array with version/date/summary | P | 0 |
//! | IT-23 | `mode::history count::1 format::json` → single-element array | P | 0 |
//! | IT-24 | `mode::history count::1 v::0` → exactly 1 bare line | P | 0 |
//! | IT-25 | `mode::history count::1 v::2` → single changelog block | P | 0 |
//! | IT-26 | `mode::history` default count ≤10 entries | P | 0 |
//! | IT-27 | `mode::history count::100` → all available releases | P | 0 |
//! | IT-28 | Idempotency: two `mode::history` calls = same output | P | 0 |
//! | IT-29 | Param order: `count::3 v::0` = `v::0 count::3` (`mode::history`) | P | 0 |
//! | IT-30 | `mode::history count::0 format::json` → empty array `[]` | P | 0 |
//! | IT-31 | `mode::history format::xml` → exit 1 | N | 1 |
//! | IT-32 | `mode::history format::JSON` (uppercase) → exit 1 | N | 1 |
//! | IT-33 | `mode::history format::` (empty) → exit 1 | N | 1 |
//! | IT-34 | `mode::history` unknown param `bogus::x` → exit 1 | N | 1 |
//! | IT-35 | `mode::history`, network unavailable → exit 0 via compiled-in fallback (manual only) | P | 0 |
//! | IT-36 | `mode::history`, HOME empty → exit 2 | N | 2 |
//! | IT-37 | `mode::history count::-1` → parse error → exit 1 | N | 1 |
//! | IT-38 | `mode::history v::abc` → exit 1 (type mismatch) | N | 1 |
//! | IT-39 | `mode::history count::abc` → exit 1 (type mismatch) | N | 1 |
//! | IT-40 | `mode::history --verbose` unknown flag → exit 1 | N | 1 |
//! | IT-41 | UTF-8 non-ASCII in body preserved (em-dash, smart-quote) (`mode::history`) | P | 0 |

use tempfile::TempDir;

use crate::subprocess_helpers::{ assert_exit, run_clv, run_clv_with_env, stderr, stdout, write_markers };

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

// TC-109: v::1 → `<semver>  [labels]` or bare `<semver>` when no labels match
#[ test ]
fn tc109_version_show_v1_labeled()
{
  let out = run_clv( &[ ".version.show", "v::1" ] );
  if out.status.code() == Some( 0 )
  {
    let text = stdout( &out );
    let first = text.split_whitespace().next().unwrap_or( "" );
    assert!(
      first.chars().next().is_some_and( | c | c.is_ascii_digit() ),
      "v::1 first token must be a semver (starts with digit), got: {text}"
    );
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

// TC-123: .version.list v::1 shows pinned version in parens
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

// IT-42: mode::aliases explicit is byte-identical to default (absent mode::)
#[ test ]
fn it42_version_list_mode_aliases_matches_default()
{
  let explicit = run_clv( &[ ".version.list", "mode::aliases" ] );
  let default  = run_clv( &[ ".version.list" ] );
  assert_exit( &explicit, 0 );
  assert_exit( &default, 0 );
  assert_eq!( stdout( &explicit ), stdout( &default ) );
}

// IT-43: mode::bogus → exit 1, unrecognized mode
#[ test ]
fn it43_version_list_mode_bogus_exits_1()
{
  let out = run_clv( &[ ".version.list", "mode::bogus" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "bogus" ), "stderr must name the invalid mode: {err}" );
  assert!( err.contains( "aliases" ) && err.contains( "history" ), "stderr must list the valid mode set: {err}" );
}

// IT-44: mode:: (empty value) → exit 1
#[ test ]
fn it44_version_list_mode_empty_exits_1()
{
  let out = run_clv( &[ ".version.list", "mode::" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "mode" ), "stderr must reference mode:: or empty value: {err}" );
}

// IT-45: mode::History (wrong case) → exit 1, case-sensitive
#[ test ]
fn it45_version_list_mode_wrong_case_exits_1()
{
  let out = run_clv( &[ ".version.list", "mode::History" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "History" ), "stderr must name the invalid mode: {err}" );
}

// IT-46: count:: under mode::aliases is accepted but has no effect
#[ test ]
fn it46_version_list_count_inert_under_aliases()
{
  let with_count    = run_clv( &[ ".version.list", "count::5" ] );
  let without_count = run_clv( &[ ".version.list" ] );
  assert_exit( &with_count, 0 );
  assert_exit( &without_count, 0 );
  assert_eq!( stdout( &with_count ), stdout( &without_count ) );
}

// ─── E15: version list, mode::history ───────────────────────────────────────

/// Panics if the command output indicates network unavailability.
/// Network-dependent tests must fail loudly — silent returns hide real failures.
fn require_network_or_fail( out : &std::process::Output )
{
  if out.status.code() == Some( 2 )
  {
    let err = String::from_utf8_lossy( &out.stderr );
    assert!(
      !err.contains( "failed to fetch" ) && !err.contains( "empty response" ),
      "network required — run this test suite in an environment with network access\nstderr: {err}"
    );
  }
}

// IT-16: default invocation exits 0
#[ test ]
fn it16_version_list_mode_history_defaults_exit_0()
{
  let out = run_clv( &[ ".version.list", "mode::history" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.is_empty(), "default output must be non-empty" );
}

// IT-17: count::3 → ≤3 version entries
#[ test ]
fn it17_version_list_mode_history_count_3()
{
  let out = run_clv( &[ ".version.list", "mode::history", "count::3", "v::0" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let lines : Vec< &str > = text.lines().filter( | l | !l.is_empty() ).collect();
  assert!( lines.len() <= 3, "expected ≤3 lines, got {}", lines.len() );
}

// IT-18: count::0 → empty output
#[ test ]
fn it18_version_list_mode_history_count_0_empty()
{
  let out = run_clv( &[ ".version.list", "mode::history", "count::0" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim().is_empty(), "count::0 must produce empty output, got: {text}" );
}

// IT-19: v::0 → bare version+date lines
#[ test ]
fn it19_version_list_mode_history_v0_bare()
{
  let out = run_clv( &[ ".version.list", "mode::history", "v::0", "count::3" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for line in text.lines().filter( | l | !l.is_empty() )
  {
    // Bare format: version and date separated by whitespace, no summary text beyond that
    let parts : Vec< &str > = line.split_whitespace().collect();
    assert!(
      parts.len() == 2,
      "v::0 line must have exactly 2 fields (version date), got {}: {line}",
      parts.len()
    );
  }
}

// IT-20: v::1 → version + date + summary
#[ test ]
fn it20_version_list_mode_history_v1_with_summary()
{
  let out = run_clv( &[ ".version.list", "mode::history", "v::1", "count::3" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for line in text.lines().filter( | l | !l.is_empty() )
  {
    let parts : Vec< &str > = line.split_whitespace().collect();
    assert!(
      parts.len() >= 3,
      "v::1 line must have ≥3 fields (version date summary...), got {}: {line}",
      parts.len()
    );
  }
}

// IT-21: v::2 → full changelog with ## headers
#[ test ]
fn it21_version_list_mode_history_v2_full_changelog()
{
  let out = run_clv( &[ ".version.list", "mode::history", "v::2", "count::2" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "## " ), "v::2 must contain '## ' header lines: {text}" );
  assert!( text.contains( "- " ), "v::2 must contain '- ' changelog bullets: {text}" );
}

// IT-22: format::json → JSON array with version/date/summary
#[ test ]
fn it22_version_list_mode_history_format_json()
{
  let out = run_clv( &[ ".version.list", "mode::history", "format::json", "count::3" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim_start().starts_with( '[' ), "JSON must start with [: {text}" );
  assert!( text.contains( "\"version\"" ), "JSON must have 'version' field: {text}" );
  assert!( text.contains( "\"date\"" ), "JSON must have 'date' field: {text}" );
  assert!( text.contains( "\"summary\"" ), "JSON must have 'summary' field: {text}" );
}

// IT-23: count::1 format::json → single-element array
#[ test ]
fn it23_version_list_mode_history_count_1_json()
{
  let out = run_clv( &[ ".version.list", "mode::history", "count::1", "format::json" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let count = text.matches( "\"version\"" ).count();
  assert_eq!( count, 1, "count::1 JSON must have exactly 1 version field, got: {count}" );
}

// IT-24: count::1 v::0 → exactly 1 bare line
#[ test ]
fn it24_version_list_mode_history_count_1_v0()
{
  let out = run_clv( &[ ".version.list", "mode::history", "count::1", "v::0" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let lines : Vec< &str > = text.lines().filter( | l | !l.is_empty() ).collect();
  assert_eq!( lines.len(), 1, "count::1 v::0 must produce exactly 1 line, got: {}", lines.len() );
}

// IT-25: count::1 v::2 → single changelog block
#[ test ]
fn it25_version_list_mode_history_count_1_v2()
{
  let out = run_clv( &[ ".version.list", "mode::history", "count::1", "v::2" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Count version headers (## X.Y.Z (date)), not changelog body headers like ## What's changed
  let version_headers = text.lines()
  .filter( | l | l.starts_with( "## " ) && l.contains( '(' ) && l.contains( ')' ) )
  .count();
  assert_eq!( version_headers, 1, "count::1 v::2 must have exactly 1 version header, got: {version_headers}" );
}

// IT-26: default count ≤10 entries
#[ test ]
fn it26_version_list_mode_history_default_count_le_10()
{
  let out = run_clv( &[ ".version.list", "mode::history", "v::0" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let lines : Vec< &str > = text.lines().filter( | l | !l.is_empty() ).collect();
  assert!( lines.len() <= 10, "default count must be ≤10, got: {}", lines.len() );
}

// IT-27: count::100 → all available releases
#[ test ]
fn it27_version_list_mode_history_count_100_all()
{
  let out = run_clv( &[ ".version.list", "mode::history", "count::100", "v::0" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let lines : Vec< &str > = text.lines().filter( | l | !l.is_empty() ).collect();
  assert!( lines.len() > 10, "count::100 must return more than default 10, got: {}", lines.len() );
  assert!( lines.len() <= 100, "count::100 must return ≤100, got: {}", lines.len() );
}

// IT-28: idempotency — two calls produce identical output
#[ test ]
fn it28_version_list_mode_history_idempotent()
{
  let out1 = run_clv( &[ ".version.list", "mode::history", "count::1", "v::0" ] );
  require_network_or_fail( &out1 );
  let out2 = run_clv( &[ ".version.list", "mode::history", "count::1", "v::0" ] );
  require_network_or_fail( &out2 );
  assert_exit( &out1, 0 );
  assert_exit( &out2, 0 );
  assert_eq!( stdout( &out1 ), stdout( &out2 ), "two calls must produce identical output" );
}

// IT-29: parameter order independence
#[ test ]
fn it29_version_list_mode_history_param_order()
{
  let out_a = run_clv( &[ ".version.list", "mode::history", "count::3", "v::0" ] );
  require_network_or_fail( &out_a );
  let out_b = run_clv( &[ ".version.list", "v::0", "mode::history", "count::3" ] );
  require_network_or_fail( &out_b );
  assert_exit( &out_a, 0 );
  assert_exit( &out_b, 0 );
  assert_eq!( stdout( &out_a ), stdout( &out_b ), "param order must not affect output" );
}

// IT-30: count::0 format::json → empty array []
#[ test ]
fn it30_version_list_mode_history_count_0_json_empty_array()
{
  let out = run_clv( &[ ".version.list", "mode::history", "count::0", "format::json" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!( text.trim(), "[]", "count::0 format::json must be [], got: {text}" );
}

// IT-31: format::xml → exit 1
#[ test ]
fn it31_version_list_mode_history_format_xml_exits_1()
{
  let out = run_clv( &[ ".version.list", "mode::history", "format::xml" ] );
  assert_exit( &out, 1 );
}

// IT-32: format::JSON (uppercase) → exit 1
#[ test ]
fn it32_version_list_mode_history_format_uppercase_exits_1()
{
  let out = run_clv( &[ ".version.list", "mode::history", "format::JSON" ] );
  assert_exit( &out, 1 );
}

// IT-33: format:: (empty) → exit 1
#[ test ]
fn it33_version_list_mode_history_format_empty_exits_1()
{
  let out = run_clv( &[ ".version.list", "mode::history", "format::" ] );
  assert_exit( &out, 1 );
}

// IT-34: unknown param bogus::x → exit 1
#[ test ]
fn it34_version_list_mode_history_unknown_param_exits_1()
{
  let out = run_clv( &[ ".version.list", "mode::history", "bogus::x" ] );
  assert_exit( &out, 1 );
}

// IT-35: Network unavailable → exit 0, serving the compiled-in VERSION_HISTORY snapshot
// (stderr carries an advisory warning). Manual-only test: cannot reliably trigger
// network failure in CI. Expected behavior documented in test matrix header above.

// IT-36: HOME empty → exit 2
#[ test ]
fn it36_version_list_mode_history_no_home_exits_2()
{
  let out = run_clv_with_env( &[ ".version.list", "mode::history" ], &[ ( "HOME", "" ) ] );
  assert_exit( &out, 2 );
}

// IT-37: count::-1 → parse error → exit 1
#[ test ]
fn it37_version_list_mode_history_negative_count_exits_1()
{
  let out = run_clv( &[ ".version.list", "mode::history", "count::-1" ] );
  assert_exit( &out, 1 );
}

// IT-38: v::abc → exit 1 (type mismatch)
#[ test ]
fn it38_version_list_mode_history_v_abc_exits_1()
{
  let out = run_clv( &[ ".version.list", "mode::history", "v::abc" ] );
  assert_exit( &out, 1 );
}

// IT-39: count::abc → exit 1 (type mismatch)
#[ test ]
fn it39_version_list_mode_history_count_abc_exits_1()
{
  let out = run_clv( &[ ".version.list", "mode::history", "count::abc" ] );
  assert_exit( &out, 1 );
}

// IT-40: --verbose flag-style → exit 1
#[ test ]
fn it40_version_list_mode_history_flag_style_exits_1()
{
  let out = run_clv( &[ ".version.list", "mode::history", "--verbose" ] );
  assert_exit( &out, 1 );
}

// IT-41: UTF-8 non-ASCII characters in release body preserved intact.
//
// Root Cause: parse_json_string_value() iterated by byte index and cast each byte
//   to char with `bytes[i] as char`, breaking multi-byte UTF-8 sequences. U+2014
//   (em-dash, 3 bytes: E2 80 94) was read as â (U+00E2) + two C1 controls.
// Why Not Caught: All existing tests used ASCII-only fixture data; the real cache
//   file is not part of the test suite.
// Fix Applied: Replaced byte-indexed loop with `str::chars()` iterator which
//   respects UTF-8 character boundaries natively.
// Prevention: Test uses actual UTF-8 bytes written to cache (not \\uXXXX escapes
//   which are handled by a separate correct code path).
// Pitfall: Do NOT iterate json.as_bytes() by index and cast to char — this silently
//   corrupts any codepoint above U+007F. Use str::chars() instead.
#[ test ]
fn it41_version_list_mode_history_utf8_body_preserved()
{
  let dir = TempDir::new().unwrap();
  let cache_dir = dir.path().join( ".claude" ).join( ".transient" );
  std::fs::create_dir_all( &cache_dir ).unwrap();
  // Actual UTF-8 bytes for em-dash (U+2014) and right-quote (U+2019).
  // Bug only triggered by raw multi-byte UTF-8, not \\uXXXX JSON escapes.
  let em_dash    = '\u{2014}';
  let rt_quote   = '\u{2019}';
  let cache_json = format!(
    "[{{\"tag_name\": \"v1.0.0\", \"published_at\": \"2026-01-01T00:00:00Z\", \
     \"body\": \"- Feature with em{em_dash}dash and smart{rt_quote}s\"}}]"
  );
  std::fs::write( cache_dir.join( "version_history_cache.json" ), &cache_json ).unwrap();
  let out = run_clv_with_env(
    &[ ".version.list", "mode::history", "v::2", "count::1" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( em_dash ),
    "em-dash U+2014 must be intact in output, got: {text:?}"
  );
  assert!(
    text.contains( rt_quote ),
    "right-quote U+2019 must be intact in output, got: {text:?}"
  );
  assert!(
    !text.contains( '\u{00e2}' ),
    "output must not contain garbled 0xE2 byte (U+00E2 'â'), got: {text:?}"
  );
}

// ─── E3 label annotation tests (IT-9/IT-10/IT-11/IT-12/FT-6) ─────────────────

// IT-9: v::1 → shows `[team-pin]` when custom marker matches installed version
#[ test ]
fn it09_version_show_v1_custom_marker_label()
{
  let ver_out = run_clv( &[ ".version.show", "v::0" ] );
  if ver_out.status.code() != Some( 0 ) { return; }
  let installed = stdout( &ver_out ).trim().to_string();

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_markers( dir.path(), &[ ( "team-pin", &installed ) ] );

  let out = run_clv_with_env( &[ ".version.show", "v::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[team-pin]" ), "expected [team-pin] in output, got: {text}" );
}

// IT-10: v::1 with no markers file → no brackets in output
#[ test ]
fn it10_version_show_v1_no_markers_no_brackets()
{
  let ver_out = run_clv( &[ ".version.show", "v::0" ] );
  if ver_out.status.code() != Some( 0 ) { return; }

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clv_with_env( &[ ".version.show", "v::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( '[' ), "v::1 with no markers must not show brackets, got: {text}" );
}

// IT-11: format::json → includes `"labels"` array when custom marker matches installed version
#[ test ]
fn it11_version_show_json_labels_array()
{
  let ver_out = run_clv( &[ ".version.show", "v::0" ] );
  if ver_out.status.code() != Some( 0 ) { return; }
  let installed = stdout( &ver_out ).trim().to_string();

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_markers( dir.path(), &[ ( "my-marker", &installed ) ] );

  let out = run_clv_with_env( &[ ".version.show", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"labels\"" ), "JSON must contain 'labels' key, got: {text}" );
}

// IT-12: v::0 → bare version string, no labels even when markers exist
#[ test ]
fn it12_version_show_v0_no_labels()
{
  let ver_out = run_clv( &[ ".version.show", "v::0" ] );
  if ver_out.status.code() != Some( 0 ) { return; }
  let installed = stdout( &ver_out ).trim().to_string();

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_markers( dir.path(), &[ ( "some-marker", &installed ) ] );

  let out = run_clv_with_env( &[ ".version.show", "v::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( '[' ), "v::0 must not show labels, got: {text}" );
}

// FT-6: Custom marker label annotation shown by `.version.show` (multi-label)
#[ test ]
fn ft006_marker_label_shown_by_version_show()
{
  let ver_out = run_clv( &[ ".version.show", "v::0" ] );
  if ver_out.status.code() != Some( 0 ) { return; }
  let installed = stdout( &ver_out ).trim().to_string();

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_markers( dir.path(), &[
    ( "release-pin", &installed ),
    ( "team-dev",    &installed ),
  ] );

  let out = run_clv_with_env( &[ ".version.show", "v::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "release-pin" ), "must show release-pin label, got: {text}" );
  assert!( text.contains( "team-dev" ),    "must show team-dev label, got: {text}" );

  let jout = run_clv_with_env( &[ ".version.show", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &jout, 0 );
  let jtext = stdout( &jout );
  assert!( jtext.contains( "\"labels\"" ),   "JSON must have 'labels' key, got: {jtext}" );
  assert!( jtext.contains( "release-pin" ), "JSON labels must include release-pin, got: {jtext}" );
  assert!( jtext.contains( "team-dev" ),    "JSON labels must include team-dev, got: {jtext}" );
}
