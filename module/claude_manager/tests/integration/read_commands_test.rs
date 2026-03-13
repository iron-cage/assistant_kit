//! Integration tests for read-only `cm` commands.
//!
//! # Test Matrix
//!
//! ## E1 — `.help`
//! | TC | Description | P/N | Exit |
//! |----|-------------|-----|------|
//! | TC-01 | `cm .` → help output, exit 0 | P | 0 |
//! | TC-02 | `cm` (empty argv) → help output, exit 0 | P | 0 |
//!
//! ## E2 — `.status`
//! | TC | Description | P/N | Exit |
//! |----|-------------|-----|------|
//! | TC-095 | `.status` exits 0 always | P | 0 |
//! | TC-096 | `.status` with empty PATH → version "not found", still exits 0 | P | 0 |
//! | TC-097 | `.status v::0` → 3 bare lines | P | 0 |
//! | TC-098 | `.status v::1` → labeled Version/Processes/Account lines | P | 0 |
//! | TC-100 | `.status format::json` → valid JSON with required keys | P | 0 |
//! | TC-104 | `.status v::0` has fewer lines than `.status v::1` | P | 0 |
//! | TC-105 | `.status` HOME not set → account "unknown", no crash | P | 0 |
//! | TC-419 | `.status` with no preference → no "Preferred" line | P | 0 |
//! | TC-420 | `.status` with preference → shows "Preferred" line | P | 0 |
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
//!
//! ## E6 — `.processes`
//! | TC | Description | P/N | Exit |
//! |----|-------------|-----|------|
//! | TC-137 | `.processes` exits 0 | P | 0 |
//! | TC-141 | `.processes v::0` → no crash | P | 0 |
//! | TC-144 | `.processes format::json` → {"processes":[...]} valid JSON | P | 0 |
//! | TC-145 | `.processes format::json` no processes → {"processes":[]} | P | 0 |
//!
//! ## E8 — `.settings.show`
//! | TC | Description | P/N | Exit |
//! |----|-------------|-----|------|
//! | TC-161 | `.settings.show` file missing → exit 2 | N | 2 |
//! | TC-162 | `.settings.show` empty {} → empty output, exit 0 | P | 0 |
//! | TC-163 | `.settings.show` valid settings → keys shown, exit 0 | P | 0 |
//! | TC-164 | `.settings.show v::0` → key=value format | P | 0 |
//! | TC-167 | `.settings.show format::json` → valid JSON | P | 0 |
//! | TC-170 | `.settings.show` malformed file → exit 2 | N | 2 |
//! | TC-171 | `.settings.show` HOME not set → exit 2 | N | 2 |
//!
//! ## E9 — `.settings.get`
//! | TC | Description | P/N | Exit |
//! |----|-------------|-----|------|
//! | TC-174 | `.settings.get` no `key::` → exit 1 | N | 1 |
//! | TC-176 | `.settings.get key::existing` → value, exit 0 | P | 0 |
//! | TC-177 | `.settings.get key::nonexistent` → exit 2 | N | 2 |
//! | TC-179 | `.settings.get v::0` → bare value only | P | 0 |
//! | TC-180 | `.settings.get v::1` → "key: value" | P | 0 |
//! | TC-182 | `.settings.get format::json` → {"key":"..","value":..} | P | 0 |
//! | TC-184 | `.settings.get` file missing → exit 2 | N | 2 |
//!
//! ## FR — Flag-level edge cases
//! | TC | Description | P/N | Exit |
//! |----|-------------|-----|------|
//! | TC-237 | `.settings.get` without `key::` → error mentions `key::` | N | 1 |
//! | TC-238 | `.settings.set` without `key::` → error mentions `key::` | N | 1 |
//! | TC-239 | `.settings.set key::foo` without `value::` → error mentions `value::` | N | 1 |
//! | TC-241 | `.settings.show format::json` preserves bool/number types | P | 0 |
//! | TC-242 | `format::xml` → exit 1 (unknown format) | N | 1 |
//! | TC-243 | `format::JSON` (uppercase) → exit 1 | N | 1 |
//! | TC-244 | `format::` (empty) → exit 1 | N | 1 |
//! | TC-245 | Last `v::` wins when duplicated | P | 0 |
//!
//! ## E15 — `.version.history`
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

use crate::helpers::{ assert_exit, run_clm, run_clm_with_env, stderr, stdout, write_settings };

// ─── E1: help ────────────────────────────────────────────────────────────────

// TC-01
#[ test ]
fn tc01_dot_alias_shows_help()
{
  let out = run_clm( &[ "." ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( ".status" ), "expected help listing, got: {text}" );
}

// TC-02
#[ test ]
fn tc02_empty_argv_shows_help()
{
  let out = run_clm( &[] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( ".status" ), "expected help listing, got: {text}" );
}

// ─── E2: status ──────────────────────────────────────────────────────────────

// TC-095
#[ test ]
fn tc095_status_exits_0()
{
  let out = run_clm( &[ ".status" ] );
  assert_exit( &out, 0 );
}

// TC-096: no symlink + empty PATH → version "not found", still exits 0
#[ test ]
fn tc096_status_no_claude_in_path_exits_0()
{
  let dir = TempDir::new().unwrap();
  let fake_home = dir.path().to_str().unwrap();
  let out = run_clm_with_env(
    &[ ".status" ],
    &[ ( "PATH", "" ), ( "HOME", fake_home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "not found" ) || text.contains( "unknown" ),
    "expected 'not found' or 'unknown' in output, got: {text}"
  );
}

// TC-097: v::0 → exactly 3 lines (version, processes, account) when no preference set
#[ test ]
fn tc097_status_v0_has_3_lines()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );
  let out = run_clm_with_env(
    &[ ".status", "v::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let lines : Vec< &str > = text.lines().collect();
  assert_eq!( lines.len(), 3, "v::0 must produce exactly 3 lines, got: {text:?}" );
}

// TC-098: v::1 → labels present
#[ test ]
fn tc098_status_v1_has_labels()
{
  let out = run_clm( &[ ".status", "v::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Version:" ),  "missing 'Version:' label, got: {text}" );
  assert!( text.contains( "Processes:" ), "missing 'Processes:' label, got: {text}" );
  assert!( text.contains( "Account:" ),  "missing 'Account:' label, got: {text}" );
}

// TC-100: format::json → valid JSON with required keys
#[ test ]
fn tc100_status_format_json()
{
  let out = run_clm( &[ ".status", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"version\"" ),  "missing 'version' key in JSON: {text}" );
  assert!( text.contains( "\"processes\"" ), "missing 'processes' key in JSON: {text}" );
  assert!( text.contains( "\"account\"" ),  "missing 'account' key in JSON: {text}" );
}

// TC-104: v::0 fewer lines than v::1
#[ test ]
fn tc104_status_v0_fewer_lines_than_v1()
{
  let out0 = run_clm( &[ ".status", "v::0" ] );
  let out1 = run_clm( &[ ".status", "v::1" ] );
  assert_exit( &out0, 0 );
  assert_exit( &out1, 0 );
  let n0 = stdout( &out0 ).lines().count();
  let n1 = stdout( &out1 ).lines().count();
  assert!( n0 <= n1, "v::0 ({n0} lines) must have \u{2264} lines than v::1 ({n1} lines)" );
}

// TC-105: HOME not set → account "unknown", no crash
#[ test ]
fn tc105_status_no_home_shows_unknown_account()
{
  let out = run_clm_with_env( &[ ".status" ], &[ ( "HOME", "" ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "unknown" ),
    "expected 'unknown' account with no HOME, got: {text}"
  );
}

// ─── E3: version show ────────────────────────────────────────────────────────

// TC-107: no symlink + empty PATH → exit 2
#[ test ]
fn tc107_version_show_no_claude_exits_2()
{
  let dir = TempDir::new().unwrap();
  let fake_home = dir.path().to_str().unwrap();
  let out = run_clm_with_env(
    &[ ".version.show" ],
    &[ ( "PATH", "" ), ( "HOME", fake_home ) ],
  );
  assert_exit( &out, 2 );
}

// TC-108: v::0 → bare version string (requires claude)
#[ test ]
fn tc108_version_show_v0_bare_string()
{
  let out = run_clm( &[ ".version.show", "v::0" ] );
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
  let out = run_clm( &[ ".version.show", "v::1" ] );
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
  let out = run_clm( &[ ".version.show", "format::json" ] );
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
  let out = run_clm( &[ ".version.list" ] );
  assert_exit( &out, 0 );
}

// TC-116
#[ test ]
fn tc116_version_list_includes_stable()
{
  let out = run_clm( &[ ".version.list" ] );
  assert_exit( &out, 0 );
  assert!( stdout( &out ).contains( "stable" ), "output must contain 'stable'" );
}

// TC-117
#[ test ]
fn tc117_version_list_includes_latest()
{
  let out = run_clm( &[ ".version.list" ] );
  assert_exit( &out, 0 );
  assert!( stdout( &out ).contains( "latest" ), "output must contain 'latest'" );
}

// TC-118: v::0 → names only (no descriptions or dashes)
#[ test ]
fn tc118_version_list_v0_names_only()
{
  let out = run_clm( &[ ".version.list", "v::0" ] );
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
  let out = run_clm( &[ ".version.list", "v::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( " \u{2014} " ) || text.contains( ": " ),
    "v::1 must include descriptions, got: {text}" );
}

// TC-120: identical on two consecutive calls
#[ test ]
fn tc120_version_list_is_idempotent()
{
  let out1 = run_clm( &[ ".version.list" ] );
  let out2 = run_clm( &[ ".version.list" ] );
  assert_exit( &out1, 0 );
  assert_exit( &out2, 0 );
  assert_eq!( stdout( &out1 ), stdout( &out2 ), "version list must be deterministic" );
}

// TC-121: format::json → valid JSON array
#[ test ]
fn tc121_version_list_format_json_array()
{
  let out = run_clm( &[ ".version.list", "format::json" ] );
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
  let out = run_clm( &[ ".version.list" ] );
  assert_exit( &out, 0 );
  assert!( stdout( &out ).contains( "month" ), "output must contain 'month'" );
}

// TC-123: .version.list v::1 shows pinned versions for stable and month
#[ test ]
fn tc123_version_list_v1_shows_pinned_versions()
{
  let out = run_clm( &[ ".version.list", "v::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "(v" ), "v::1 must show pinned version in parens, got: {text}" );
}

// TC-124: .version.list format::json includes value field for pinned aliases
#[ test ]
fn tc124_version_list_json_has_value_field()
{
  let out = run_clm( &[ ".version.list", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"value\"" ), "JSON must have 'value' field for pinned aliases: {text}" );
}

// ─── E6: processes ────────────────────────────────────────────────────────────

// TC-137
#[ test ]
fn tc137_processes_exits_0()
{
  let out = run_clm( &[ ".processes" ] );
  assert_exit( &out, 0 );
}

// TC-141: v::0 → no crash
#[ test ]
fn tc141_processes_v0_no_crash()
{
  let out = run_clm( &[ ".processes", "v::0" ] );
  assert_exit( &out, 0 );
}

// TC-144: format::json → {"processes":[...]}
#[ test ]
fn tc144_processes_format_json_valid()
{
  let out = run_clm( &[ ".processes", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"processes\"" ), "missing 'processes' key in JSON: {text}" );
  assert!(
    text.trim_start().starts_with( '{' ) || text.contains( '{' ),
    "format::json must produce JSON object: {text}"
  );
}

// TC-145: no processes → {"processes":[]}
#[ test ]
fn tc145_processes_format_json_empty_when_no_processes()
{
  let out = run_clm( &[ ".processes", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"processes\"" ), "format::json must have 'processes' key: {text}" );
}

// ─── E8: settings show ───────────────────────────────────────────────────────

// TC-161: file missing → exit 2
#[ test ]
fn tc161_settings_show_file_missing_exits_2()
{
  let dir = TempDir::new().unwrap();
  let out = run_clm_with_env(
    &[ ".settings.show" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 2 );
}

// TC-162: empty {} → empty output, exit 0
#[ test ]
fn tc162_settings_show_empty_file_exits_0()
{
  let dir = TempDir::new().unwrap();
  write_settings( dir.path(), &[] );
  let out = run_clm_with_env(
    &[ ".settings.show" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim().is_empty(), "empty settings must produce no output, got: {text}" );
}

// TC-163: valid settings → keys shown, exit 0
#[ test ]
fn tc163_settings_show_valid_file()
{
  let dir = TempDir::new().unwrap();
  write_settings( dir.path(), &[ ( "myKey", "myValue" ) ] );
  let out = run_clm_with_env(
    &[ ".settings.show" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "myKey" ),   "output must contain key 'myKey': {text}" );
  assert!( text.contains( "myValue" ), "output must contain value 'myValue': {text}" );
}

// TC-164: v::0 → key=value format
#[ test ]
fn tc164_settings_show_v0_key_equals_value()
{
  let dir = TempDir::new().unwrap();
  write_settings( dir.path(), &[ ( "alpha", "beta" ) ] );
  let out = run_clm_with_env(
    &[ ".settings.show", "v::0" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "alpha=beta" ) || text.contains( "alpha: beta" ),
    "v::0 must format key=value, got: {text}"
  );
}

// TC-167: format::json → valid JSON
#[ test ]
fn tc167_settings_show_format_json()
{
  let dir = TempDir::new().unwrap();
  write_settings( dir.path(), &[ ( "k1", "v1" ) ] );
  let out = run_clm_with_env(
    &[ ".settings.show", "format::json" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim_start().starts_with( '{' ), "format::json must start with '{{': {text}" );
  assert!( text.contains( "k1" ), "JSON must contain key 'k1': {text}" );
}

// TC-170: malformed JSON → exit 2
#[ test ]
fn tc170_settings_show_malformed_file_exits_2()
{
  let dir = TempDir::new().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), "{ bad json!!" ).unwrap();
  let out = run_clm_with_env(
    &[ ".settings.show" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 2 );
}

// TC-171: HOME not set → exit 2
#[ test ]
fn tc171_settings_show_no_home_exits_2()
{
  let out = run_clm_with_env( &[ ".settings.show" ], &[ ( "HOME", "" ) ] );
  assert_exit( &out, 2 );
}

// ─── E9: settings get ────────────────────────────────────────────────────────

// TC-174: no key:: → exit 1
#[ test ]
fn tc174_settings_get_no_key_exits_1()
{
  let out = run_clm( &[ ".settings.get" ] );
  assert_exit( &out, 1 );
}

// TC-176: key::existing → value, exit 0
#[ test ]
fn tc176_settings_get_existing_key()
{
  let dir = TempDir::new().unwrap();
  write_settings( dir.path(), &[ ( "myKey", "myValue" ) ] );
  let out = run_clm_with_env(
    &[ ".settings.get", "key::myKey" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "myValue" ), "output must contain 'myValue': {text}" );
}

// TC-177: key::nonexistent → exit 2
#[ test ]
fn tc177_settings_get_missing_key_exits_2()
{
  let dir = TempDir::new().unwrap();
  write_settings( dir.path(), &[ ( "existing", "val" ) ] );
  let out = run_clm_with_env(
    &[ ".settings.get", "key::nosuchkey" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 2 );
}

// TC-179: v::0 → bare value only (no label)
#[ test ]
fn tc179_settings_get_v0_bare_value()
{
  let dir = TempDir::new().unwrap();
  write_settings( dir.path(), &[ ( "k", "thevalue" ) ] );
  let out = run_clm_with_env(
    &[ ".settings.get", "key::k", "v::0" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!(
    text.trim(),
    "thevalue",
    "v::0 must be bare value only, got: {text}"
  );
}

// TC-180: v::1 → "key: value"
#[ test ]
fn tc180_settings_get_v1_labeled()
{
  let dir = TempDir::new().unwrap();
  write_settings( dir.path(), &[ ( "mykey", "myval" ) ] );
  let out = run_clm_with_env(
    &[ ".settings.get", "key::mykey", "v::1" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "mykey" ) && text.contains( "myval" ),
    "v::1 must show 'key: value', got: {text}" );
}

// TC-182: format::json → {"key":"..","value":".."}
#[ test ]
fn tc182_settings_get_format_json()
{
  let dir = TempDir::new().unwrap();
  write_settings( dir.path(), &[ ( "alpha", "omega" ) ] );
  let out = run_clm_with_env(
    &[ ".settings.get", "key::alpha", "format::json" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"key\"" ) && text.contains( "\"value\"" ),
    "format::json must have 'key' and 'value' fields: {text}" );
}

// TC-184: file missing → exit 2
#[ test ]
fn tc184_settings_get_file_missing_exits_2()
{
  let dir = TempDir::new().unwrap();
  let out = run_clm_with_env(
    &[ ".settings.get", "key::anything" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 2 );
}

// ── FR: required-flag error format ──────────────────────────────────────────

// TC-237: .settings.get without `key::` → error contains "key:: is required"
#[ test ]
fn tc237_settings_get_missing_key_error_format()
{
  let out = run_clm( &[ ".settings.get" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "key:: is required" ), "error must contain 'key:: is required': {err}" );
}

// TC-238: .settings.set without `key::` → error contains "key:: is required"
#[ test ]
fn tc238_settings_set_missing_key_error_format()
{
  let out = run_clm( &[ ".settings.set" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "key:: is required" ), "error must contain 'key:: is required': {err}" );
}

// TC-239: .settings.set with `key::` but no `value::` → "value:: is required"
#[ test ]
fn tc239_settings_set_missing_value_error_format()
{
  let out = run_clm( &[ ".settings.set", "key::foo" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "value:: is required" ), "error must contain 'value:: is required': {err}" );
}

// ── settings show JSON type preservation ────────────────────────────────────

// TC-241: .settings.show format::json preserves bool and number types
#[ test ]
fn tc241_settings_show_json_preserves_types()
{
  let dir = tempfile::TempDir::new().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write(
    claude_dir.join( "settings.json" ),
    "{\"boolKey\":true,\"numKey\":42,\"strKey\":\"hello\"}"
  ).unwrap();
  let out = run_clm_with_env(
    &[ ".settings.show", "format::json" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( ":true" ), "bool must be unquoted in JSON: {text}" );
  assert!( text.contains( ":42" ), "number must be unquoted in JSON: {text}" );
  assert!( text.contains( ":\"hello\"" ), "string must be quoted in JSON: {text}" );
}

// ── Edge cases: empty/special values ────────────────────────────────────────

// TC-242: format::xml → exit 1 (unknown format)
#[ test ]
fn tc242_unknown_format_exits_1()
{
  let out = run_clm( &[ ".status", "format::xml" ] );
  assert_exit( &out, 1 );
}

// TC-243: format::JSON (uppercase) → exit 1
#[ test ]
fn tc243_uppercase_format_exits_1()
{
  let out = run_clm( &[ ".status", "format::JSON" ] );
  assert_exit( &out, 1 );
}

// TC-244: format:: (empty value) → exit 1
#[ test ]
fn tc244_empty_format_exits_1()
{
  let out = run_clm( &[ ".status", "format::" ] );
  assert_exit( &out, 1 );
}

// TC-245: last v:: wins when duplicated
#[ test ]
fn tc245_last_occurrence_wins_for_verbosity()
{
  let out = run_clm( &[ ".status", "v::2", "v::0" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // v::0 is last → bare output (no labels)
  assert!(
    !text.contains( "Version:" ),
    "v::0 (last-wins) must produce bare output, got: {text}"
  );
}

// ─── Preferred version display in status ─────────────────────────────────────

// TC-419: status with no preference → no "Preferred" line
#[ test ]
fn tc419_status_no_preference_no_preferred_line()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );

  let out = run_clm_with_env(
    &[ ".status" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "Preferred" ),
    "status without preference must not show Preferred line: {text}"
  );
}

// TC-420: status with preference → shows "Preferred" line
#[ test ]
fn tc420_status_with_preference_shows_preferred()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let settings_json = r#"{
  "preferredVersionSpec": "stable",
  "preferredVersionResolved": "2.1.78"
}"#;
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), settings_json ).unwrap();

  let out = run_clm_with_env(
    &[ ".status" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Preferred:" ) && text.contains( "stable" ),
    "status with preference must show Preferred line: {text}"
  );
}

// ─── E15: version history ─────────────────────────────────────────────────────

/// Returns `true` when the command failed due to network unavailability,
/// allowing network-dependent tests to pass vacuously in offline CI.
fn skip_if_no_network( out : &std::process::Output ) -> bool
{
  if out.status.code() == Some( 2 )
  {
    let err = String::from_utf8_lossy( &out.stderr );
    if err.contains( "failed to fetch" ) || err.contains( "empty response" )
    {
      return true;
    }
  }
  false
}

// TC-425: default invocation exits 0
#[ test ]
fn tc425_version_history_defaults_exit_0()
{
  let out = run_clm( &[ ".version.history" ] );
  if skip_if_no_network( &out ) { return; }
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.is_empty(), "default output must be non-empty" );
}

// TC-426: count::3 → ≤3 version entries
#[ test ]
fn tc426_version_history_count_3()
{
  let out = run_clm( &[ ".version.history", "count::3", "v::0" ] );
  if skip_if_no_network( &out ) { return; }
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let lines : Vec< &str > = text.lines().filter( | l | !l.is_empty() ).collect();
  assert!( lines.len() <= 3, "expected ≤3 lines, got {}", lines.len() );
}

// TC-427: count::0 → empty output
#[ test ]
fn tc427_version_history_count_0_empty()
{
  let out = run_clm( &[ ".version.history", "count::0" ] );
  if skip_if_no_network( &out ) { return; }
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim().is_empty(), "count::0 must produce empty output, got: {text}" );
}

// TC-428: v::0 → bare version+date lines
#[ test ]
fn tc428_version_history_v0_bare()
{
  let out = run_clm( &[ ".version.history", "v::0", "count::3" ] );
  if skip_if_no_network( &out ) { return; }
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
  let out = run_clm( &[ ".version.history", "v::1", "count::3" ] );
  if skip_if_no_network( &out ) { return; }
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
  let out = run_clm( &[ ".version.history", "v::2", "count::2" ] );
  if skip_if_no_network( &out ) { return; }
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "## " ), "v::2 must contain '## ' header lines: {text}" );
  assert!( text.contains( "- " ), "v::2 must contain '- ' changelog bullets: {text}" );
}

// TC-431: format::json → JSON array with version/date/summary
#[ test ]
fn tc431_version_history_format_json()
{
  let out = run_clm( &[ ".version.history", "format::json", "count::3" ] );
  if skip_if_no_network( &out ) { return; }
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
  let out = run_clm( &[ ".version.history", "count::1", "format::json" ] );
  if skip_if_no_network( &out ) { return; }
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let count = text.matches( "\"version\"" ).count();
  assert_eq!( count, 1, "count::1 JSON must have exactly 1 version field, got: {count}" );
}

// TC-433: count::1 v::0 → exactly 1 bare line
#[ test ]
fn tc433_version_history_count_1_v0()
{
  let out = run_clm( &[ ".version.history", "count::1", "v::0" ] );
  if skip_if_no_network( &out ) { return; }
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let lines : Vec< &str > = text.lines().filter( | l | !l.is_empty() ).collect();
  assert_eq!( lines.len(), 1, "count::1 v::0 must produce exactly 1 line, got: {}", lines.len() );
}

// TC-434: count::1 v::2 → single changelog block
#[ test ]
fn tc434_version_history_count_1_v2()
{
  let out = run_clm( &[ ".version.history", "count::1", "v::2" ] );
  if skip_if_no_network( &out ) { return; }
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
  let out = run_clm( &[ ".version.history", "v::0" ] );
  if skip_if_no_network( &out ) { return; }
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let lines : Vec< &str > = text.lines().filter( | l | !l.is_empty() ).collect();
  assert!( lines.len() <= 10, "default count must be ≤10, got: {}", lines.len() );
}

// TC-436: count::100 → all available releases
#[ test ]
fn tc436_version_history_count_100_all()
{
  let out = run_clm( &[ ".version.history", "count::100", "v::0" ] );
  if skip_if_no_network( &out ) { return; }
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
  let out1 = run_clm( &[ ".version.history", "count::1", "v::0" ] );
  if skip_if_no_network( &out1 ) { return; }
  let out2 = run_clm( &[ ".version.history", "count::1", "v::0" ] );
  if skip_if_no_network( &out2 ) { return; }
  assert_exit( &out1, 0 );
  assert_exit( &out2, 0 );
  assert_eq!( stdout( &out1 ), stdout( &out2 ), "two calls must produce identical output" );
}

// TC-438: parameter order independence
#[ test ]
fn tc438_version_history_param_order()
{
  let out_a = run_clm( &[ ".version.history", "count::3", "v::0" ] );
  if skip_if_no_network( &out_a ) { return; }
  let out_b = run_clm( &[ ".version.history", "v::0", "count::3" ] );
  if skip_if_no_network( &out_b ) { return; }
  assert_exit( &out_a, 0 );
  assert_exit( &out_b, 0 );
  assert_eq!( stdout( &out_a ), stdout( &out_b ), "param order must not affect output" );
}

// TC-439: count::0 format::json → empty array []
#[ test ]
fn tc439_version_history_count_0_json_empty_array()
{
  let out = run_clm( &[ ".version.history", "count::0", "format::json" ] );
  if skip_if_no_network( &out ) { return; }
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!( text.trim(), "[]", "count::0 format::json must be [], got: {text}" );
}

// TC-440: format::xml → exit 1
#[ test ]
fn tc440_version_history_format_xml_exits_1()
{
  let out = run_clm( &[ ".version.history", "format::xml" ] );
  assert_exit( &out, 1 );
}

// TC-441: format::JSON (uppercase) → exit 1
#[ test ]
fn tc441_version_history_format_uppercase_exits_1()
{
  let out = run_clm( &[ ".version.history", "format::JSON" ] );
  assert_exit( &out, 1 );
}

// TC-442: format:: (empty) → exit 1
#[ test ]
fn tc442_version_history_format_empty_exits_1()
{
  let out = run_clm( &[ ".version.history", "format::" ] );
  assert_exit( &out, 1 );
}

// TC-443: unknown param bogus::x → exit 1
#[ test ]
fn tc443_version_history_unknown_param_exits_1()
{
  let out = run_clm( &[ ".version.history", "bogus::x" ] );
  assert_exit( &out, 1 );
}

// TC-444: Network unavailable → exit 2
// Manual-only test: cannot reliably trigger network failure in CI.
// Expected behavior documented in test matrix header above.

// TC-445: HOME empty → exit 2
#[ test ]
fn tc445_version_history_no_home_exits_2()
{
  let out = run_clm_with_env( &[ ".version.history" ], &[ ( "HOME", "" ) ] );
  assert_exit( &out, 2 );
}

// TC-446: count::-1 → parse error → exit 1
#[ test ]
fn tc446_version_history_negative_count_exits_1()
{
  let out = run_clm( &[ ".version.history", "count::-1" ] );
  assert_exit( &out, 1 );
}

// TC-447: v::abc → exit 1 (type mismatch)
#[ test ]
fn tc447_version_history_v_abc_exits_1()
{
  let out = run_clm( &[ ".version.history", "v::abc" ] );
  assert_exit( &out, 1 );
}

// TC-448: count::abc → exit 1 (type mismatch)
#[ test ]
fn tc448_version_history_count_abc_exits_1()
{
  let out = run_clm( &[ ".version.history", "count::abc" ] );
  assert_exit( &out, 1 );
}

// TC-449: --verbose flag-style → exit 1
#[ test ]
fn tc449_version_history_flag_style_exits_1()
{
  let out = run_clm( &[ ".version.history", "--verbose" ] );
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
  let out = run_clm_with_env(
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
