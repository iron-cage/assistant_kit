//! Integration tests for `.version.history` — E15.
//!
//! | TC | Description | P/N | Exit |
//! |----|-------------|-----|------|
//! | TC-425 | `.version.history` defaults → exits 0 (network permitting) | P | 0/2 |
//! | TC-426 | `count::3` → ≤3 version entries | P | 0 |
//! | TC-427 | `count::0` → empty output | P | 0 |
//! | TC-428 | `v::0` → bare `{version}  {date}` lines | P | 0 |
//! | TC-429 | `v::1` → version + date + summary per line | P | 0 |
//! | TC-430 | `v::2` → full changelog with `##` headers | P | 0 |
//! | TC-431 | `format::json` → JSON array with version/date/summary | P | 0 |
//! | TC-432 | `count::1 format::json` → single-element array | P | 0 |
//! | TC-433 | `count::1 v::0` → exactly 1 bare line | P | 0 |
//! | TC-434 | `count::1 v::2` → single changelog block | P | 0 |
//! | TC-435 | Default count ≤10 entries | P | 0 |
//! | TC-436 | `count::100` → all available releases | P | 0 |
//! | TC-437 | Idempotency: two calls = same output | P | 0 |
//! | TC-438 | Param order: `count::3 v::0` = `v::0 count::3` | P | 0 |
//! | TC-439 | `count::0 format::json` → empty array `[]` | P | 0 |
//! | TC-440 | `format::xml` → exit 1 | N | 1 |
//! | TC-441 | `format::JSON` (uppercase) → exit 1 | N | 1 |
//! | TC-442 | `format::` (empty) → exit 1 | N | 1 |
//! | TC-443 | Unknown param `bogus::x` → exit 1 | N | 1 |
//! | TC-444 | Network unavailable → exit 2 (manual only) | N | 2 |
//! | TC-445 | HOME empty → exit 2 | N | 2 |
//! | TC-446 | `count::-1` → parse error → exit 1 | N | 1 |
//! | TC-447 | `v::abc` → exit 1 (type mismatch) | N | 1 |
//! | TC-448 | `count::abc` → exit 1 (type mismatch) | N | 1 |
//! | TC-449 | `--verbose` unknown flag → exit 1 | N | 1 |
//! | TC-450 | UTF-8 non-ASCII in body preserved (em-dash, smart-quote) | P | 0 |

use tempfile::TempDir;

use crate::subprocess_helpers::{ assert_exit, run_clv, run_clv_with_env, stdout };

// ─── Network helper ───────────────────────────────────────────────────────────

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

// ─── E15: version history ────────────────────────────────────────────────────

// TC-425: default invocation exits 0
#[ test ]
fn tc425_version_history_defaults_exit_0()
{
  let out = run_clv( &[ ".version.history" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.is_empty(), "default output must be non-empty" );
}

// TC-426: count::3 → ≤3 version entries
#[ test ]
fn tc426_version_history_count_3()
{
  let out = run_clv( &[ ".version.history", "count::3", "v::0" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let lines : Vec< &str > = text.lines().filter( | l | !l.is_empty() ).collect();
  assert!( lines.len() <= 3, "expected ≤3 lines, got {}", lines.len() );
}

// TC-427: count::0 → empty output
#[ test ]
fn tc427_version_history_count_0_empty()
{
  let out = run_clv( &[ ".version.history", "count::0" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim().is_empty(), "count::0 must produce empty output, got: {text}" );
}

// TC-428: v::0 → bare version+date lines
#[ test ]
fn tc428_version_history_v0_bare()
{
  let out = run_clv( &[ ".version.history", "v::0", "count::3" ] );
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

// TC-429: v::1 → version + date + summary
#[ test ]
fn tc429_version_history_v1_with_summary()
{
  let out = run_clv( &[ ".version.history", "v::1", "count::3" ] );
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

// TC-430: v::2 → full changelog with ## headers
#[ test ]
fn tc430_version_history_v2_full_changelog()
{
  let out = run_clv( &[ ".version.history", "v::2", "count::2" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "## " ), "v::2 must contain '## ' header lines: {text}" );
  assert!( text.contains( "- " ), "v::2 must contain '- ' changelog bullets: {text}" );
}

// TC-431: format::json → JSON array with version/date/summary
#[ test ]
fn tc431_version_history_format_json()
{
  let out = run_clv( &[ ".version.history", "format::json", "count::3" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim_start().starts_with( '[' ), "JSON must start with [: {text}" );
  assert!( text.contains( "\"version\"" ), "JSON must have 'version' field: {text}" );
  assert!( text.contains( "\"date\"" ), "JSON must have 'date' field: {text}" );
  assert!( text.contains( "\"summary\"" ), "JSON must have 'summary' field: {text}" );
}

// TC-432: count::1 format::json → single-element array
#[ test ]
fn tc432_version_history_count_1_json()
{
  let out = run_clv( &[ ".version.history", "count::1", "format::json" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let count = text.matches( "\"version\"" ).count();
  assert_eq!( count, 1, "count::1 JSON must have exactly 1 version field, got: {count}" );
}

// TC-433: count::1 v::0 → exactly 1 bare line
#[ test ]
fn tc433_version_history_count_1_v0()
{
  let out = run_clv( &[ ".version.history", "count::1", "v::0" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let lines : Vec< &str > = text.lines().filter( | l | !l.is_empty() ).collect();
  assert_eq!( lines.len(), 1, "count::1 v::0 must produce exactly 1 line, got: {}", lines.len() );
}

// TC-434: count::1 v::2 → single changelog block
#[ test ]
fn tc434_version_history_count_1_v2()
{
  let out = run_clv( &[ ".version.history", "count::1", "v::2" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Count version headers (## X.Y.Z (date)), not changelog body headers like ## What's changed
  let version_headers = text.lines()
  .filter( | l | l.starts_with( "## " ) && l.contains( '(' ) && l.contains( ')' ) )
  .count();
  assert_eq!( version_headers, 1, "count::1 v::2 must have exactly 1 version header, got: {version_headers}" );
}

// TC-435: default count ≤10 entries
#[ test ]
fn tc435_version_history_default_count_le_10()
{
  let out = run_clv( &[ ".version.history", "v::0" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let lines : Vec< &str > = text.lines().filter( | l | !l.is_empty() ).collect();
  assert!( lines.len() <= 10, "default count must be ≤10, got: {}", lines.len() );
}

// TC-436: count::100 → all available releases
#[ test ]
fn tc436_version_history_count_100_all()
{
  let out = run_clv( &[ ".version.history", "count::100", "v::0" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let lines : Vec< &str > = text.lines().filter( | l | !l.is_empty() ).collect();
  assert!( lines.len() > 10, "count::100 must return more than default 10, got: {}", lines.len() );
  assert!( lines.len() <= 100, "count::100 must return ≤100, got: {}", lines.len() );
}

// TC-437: idempotency — two calls produce identical output
#[ test ]
fn tc437_version_history_idempotent()
{
  let out1 = run_clv( &[ ".version.history", "count::1", "v::0" ] );
  require_network_or_fail( &out1 );
  let out2 = run_clv( &[ ".version.history", "count::1", "v::0" ] );
  require_network_or_fail( &out2 );
  assert_exit( &out1, 0 );
  assert_exit( &out2, 0 );
  assert_eq!( stdout( &out1 ), stdout( &out2 ), "two calls must produce identical output" );
}

// TC-438: parameter order independence
#[ test ]
fn tc438_version_history_param_order()
{
  let out_a = run_clv( &[ ".version.history", "count::3", "v::0" ] );
  require_network_or_fail( &out_a );
  let out_b = run_clv( &[ ".version.history", "v::0", "count::3" ] );
  require_network_or_fail( &out_b );
  assert_exit( &out_a, 0 );
  assert_exit( &out_b, 0 );
  assert_eq!( stdout( &out_a ), stdout( &out_b ), "param order must not affect output" );
}

// TC-439: count::0 format::json → empty array []
#[ test ]
fn tc439_version_history_count_0_json_empty_array()
{
  let out = run_clv( &[ ".version.history", "count::0", "format::json" ] );
  require_network_or_fail( &out );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!( text.trim(), "[]", "count::0 format::json must be [], got: {text}" );
}

// TC-440: format::xml → exit 1
#[ test ]
fn tc440_version_history_format_xml_exits_1()
{
  let out = run_clv( &[ ".version.history", "format::xml" ] );
  assert_exit( &out, 1 );
}

// TC-441: format::JSON (uppercase) → exit 1
#[ test ]
fn tc441_version_history_format_uppercase_exits_1()
{
  let out = run_clv( &[ ".version.history", "format::JSON" ] );
  assert_exit( &out, 1 );
}

// TC-442: format:: (empty) → exit 1
#[ test ]
fn tc442_version_history_format_empty_exits_1()
{
  let out = run_clv( &[ ".version.history", "format::" ] );
  assert_exit( &out, 1 );
}

// TC-443: unknown param bogus::x → exit 1
#[ test ]
fn tc443_version_history_unknown_param_exits_1()
{
  let out = run_clv( &[ ".version.history", "bogus::x" ] );
  assert_exit( &out, 1 );
}

// TC-444: Network unavailable → exit 2
// Manual-only test: cannot reliably trigger network failure in CI.
// Expected behavior documented in test matrix header above.

// TC-445: HOME empty → exit 2
#[ test ]
fn tc445_version_history_no_home_exits_2()
{
  let out = run_clv_with_env( &[ ".version.history" ], &[ ( "HOME", "" ) ] );
  assert_exit( &out, 2 );
}

// TC-446: count::-1 → parse error → exit 1
#[ test ]
fn tc446_version_history_negative_count_exits_1()
{
  let out = run_clv( &[ ".version.history", "count::-1" ] );
  assert_exit( &out, 1 );
}

// TC-447: v::abc → exit 1 (type mismatch)
#[ test ]
fn tc447_version_history_v_abc_exits_1()
{
  let out = run_clv( &[ ".version.history", "v::abc" ] );
  assert_exit( &out, 1 );
}

// TC-448: count::abc → exit 1 (type mismatch)
#[ test ]
fn tc448_version_history_count_abc_exits_1()
{
  let out = run_clv( &[ ".version.history", "count::abc" ] );
  assert_exit( &out, 1 );
}

// TC-449: --verbose flag-style → exit 1
#[ test ]
fn tc449_version_history_flag_style_exits_1()
{
  let out = run_clv( &[ ".version.history", "--verbose" ] );
  assert_exit( &out, 1 );
}

// TC-450: UTF-8 non-ASCII characters in release body preserved intact.
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
fn tc450_version_history_utf8_body_preserved()
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
    &[ ".version.history", "v::2", "count::1" ],
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
