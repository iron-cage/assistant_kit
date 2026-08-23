//! Integration tests for `.version.mark` — custom marker CRUD.
//!
//! Covers feature spec `010_custom_markers.md` and command spec `17_version_mark.md`.
//!
//! Also covers the runtime-file spec `tests/docs/runtime_file/004_version_markers.md`
//! (RF-1 through RF-4) — `version-markers.json` path correctness, creation, and
//! durability. RF cases live here rather than in a separate file because this is
//! the module that owns every `.version.mark` write path and the `markers_path()`
//! helper that locates the file under an isolated `HOME`.
//!
//! | TC    | Description                                            | P/N | Exit |
//! |-------|--------------------------------------------------------|-----|------|
//! | IT-1  | Create new marker → file written with entry            | P   | 0    |
//! | IT-2  | Update existing marker (upsert) → value replaced      | P   | 0    |
//! | IT-3  | Remove existing marker                                 | P   | 0    |
//! | IT-4  | Remove absent marker → no-op, exit 0                  | P   | 0    |
//! | IT-5  | `dry::1` set path → preview, no write                  | P   | 0    |
//! | IT-6  | `dry::1` unset path → preview, no write                | P   | 0    |
//! | IT-7  | `version::stable` accepted as marker value             | P   | 0    |
//! | IT-8  | Created marker appears in `.version.list`              | P   | 0    |
//! | IT-9  | Created marker accepted by `.version.install`          | P   | 0    |
//! | IT-10 | `format::json dry::1` → JSON output, exit 0           | P   | 0    |
//! | IT-11 | `name::` absent → exit 1                              | N   | 1    |
//! | IT-12 | `name::MyPin` (uppercase start) → exit 1              | N   | 1    |
//! | IT-13 | `name::1pin` (digit start) → exit 1                   | N   | 1    |
//! | IT-14 | `name::stable` (shadows built-in) → exit 1            | N   | 1    |
//! | IT-15 | `name::latest` (shadows built-in) → exit 1            | N   | 1    |
//! | IT-16 | `version::` absent on set path → exit 1               | N   | 1    |
//! | IT-17 | `version::x` (invalid spec) → exit 1                  | N   | 1    |
//! | FT-1  | Create → appears in `.version.list`                   | P   | 0    |
//! | FT-2  | Remove → absent from `.version.list`                  | P   | 0    |
//! | FT-3  | Custom marker accepted by `.version.install`          | P   | 0    |
//! | FT-4  | Invalid name (uppercase start) → exit 1               | N   | 1    |
//! | FT-5  | `dry::1` does not write `version-markers.json`        | P   | 0    |
//! | IT-18 | Malformed `version-markers.json` → graceful, exit 0   | P   | 0    |
//! | IT-19 | `name::` (empty) → exit 1                              | N   | 1    |
//! | IT-20 | `name::` with 33-char value → length exceeded, exit 1  | N   | 1    |
//! | RF-2  | Markers file created when absent, carries `markers` array | P | 0  |
//! | RF-3  | Absent markers file → `.version.list` still exits 0    | P   | 0    |

use tempfile::TempDir;

use crate::subprocess_helpers::{
  assert_exit, run_clv_with_env, stderr, stdout, write_markers,
};

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn markers_path( home : &std::path::Path ) -> std::path::PathBuf
{
  home.join( ".claude" ).join( "version-markers.json" )
}

// ─── IT-1: Create new marker ──────────────────────────────────────────────────

#[ test ]
fn it01_mark_create_new()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clv_with_env(
    &[ ".version.mark", "name::team-pin", "version::2.1.220" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let contents = std::fs::read_to_string( markers_path( dir.path() ) ).unwrap();
  assert!( contents.contains( "team-pin" ), "markers file must contain name: {contents}" );
  assert!( contents.contains( "2.1.220" ), "markers file must contain value: {contents}" );
}

// ─── IT-2: Update existing marker ────────────────────────────────────────────

#[ test ]
fn it02_mark_update_existing()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_markers( dir.path(), &[ ( "team-pin", "2.1.200" ) ] );

  let out = run_clv_with_env(
    &[ ".version.mark", "name::team-pin", "version::2.1.220" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let contents = std::fs::read_to_string( markers_path( dir.path() ) ).unwrap();
  assert!( contents.contains( "2.1.220" ), "value must be updated to 2.1.220: {contents}" );
  assert!( !contents.contains( "2.1.200" ), "old value 2.1.200 must be replaced: {contents}" );
}

// ─── IT-3: Remove existing marker ────────────────────────────────────────────

#[ test ]
fn it03_mark_remove_existing()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_markers( dir.path(), &[ ( "team-pin", "2.1.220" ) ] );

  let out = run_clv_with_env(
    &[ ".version.mark", "name::team-pin", "unset::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let contents = std::fs::read_to_string( markers_path( dir.path() ) ).unwrap();
  assert!( !contents.contains( "team-pin" ), "team-pin must be removed: {contents}" );
}

// ─── IT-4: Remove absent marker → no-op ─────────────────────────────────────

#[ test ]
fn it04_mark_remove_absent_noop()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // no markers file written — absent marker

  let out = run_clv_with_env(
    &[ ".version.mark", "name::team-pin", "unset::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
}

// ─── IT-5: dry::1 set path → no write ────────────────────────────────────────

#[ test ]
fn it05_mark_dry_set_no_write()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clv_with_env(
    &[ ".version.mark", "name::team-pin", "version::2.1.220", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry]" ), "dry preview must contain [dry]: {text}" );
  assert!( !markers_path( dir.path() ).exists(), "markers file must not be created in dry mode" );
}

// ─── IT-6: dry::1 unset path → no write ──────────────────────────────────────

#[ test ]
fn it06_mark_dry_unset_no_write()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_markers( dir.path(), &[ ( "team-pin", "2.1.220" ) ] );

  let out = run_clv_with_env(
    &[ ".version.mark", "name::team-pin", "unset::1", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry]" ), "dry preview must contain [dry]: {text}" );

  let contents = std::fs::read_to_string( markers_path( dir.path() ) ).unwrap();
  assert!( contents.contains( "team-pin" ), "team-pin must still be present after dry unset: {contents}" );
}

// ─── IT-7: version::stable accepted as marker value ──────────────────────────

#[ test ]
fn it07_mark_version_builtin_alias()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clv_with_env(
    &[ ".version.mark", "name::team-pin", "version::stable" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let contents = std::fs::read_to_string( markers_path( dir.path() ) ).unwrap();
  assert!( contents.contains( "stable" ), "stored value must be stable: {contents}" );
}

// ─── IT-8: Created marker appears in .version.list ───────────────────────────

#[ test ]
fn it08_mark_appears_in_list()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_markers( dir.path(), &[ ( "team-pin", "2.1.220" ) ] );

  let out = run_clv_with_env( &[ ".version.list" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "team-pin" ), ".version.list must show custom marker: {text}" );
}

// ─── IT-9: Created marker accepted by .version.install ───────────────────────

#[ test ]
fn it09_mark_accepted_by_install()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_markers( dir.path(), &[ ( "team-pin", "2.1.220" ) ] );

  let out = run_clv_with_env(
    &[ ".version.install", "version::team-pin", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "2.1.220" ), "install must resolve custom marker to 2.1.220: {text}" );
}

// ─── IT-10: format::json dry::1 → JSON output ────────────────────────────────

#[ test ]
fn it10_mark_json_format_dry()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clv_with_env(
    &[ ".version.mark", "name::team-pin", "version::2.1.220", "format::json", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim_start().starts_with( '{' ), "JSON output must start with {{: {text}" );
}

// ─── IT-11: name:: absent → exit 1 ───────────────────────────────────────────

#[ test ]
fn it11_mark_name_absent_exits_1()
{
  let out = run_clv_with_env(
    &[ ".version.mark", "version::2.1.220" ],
    &[],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "name" ), "error must reference missing name: {err}" );
}

// ─── IT-12: name::MyPin (uppercase start) → exit 1 ──────────────────────────

#[ test ]
fn it12_mark_name_uppercase_exits_1()
{
  let out = run_clv_with_env(
    &[ ".version.mark", "name::MyPin", "version::2.1.220" ],
    &[],
  );
  assert_exit( &out, 1 );
}

// ─── IT-13: name::1pin (digit start) → exit 1 ────────────────────────────────

#[ test ]
fn it13_mark_name_digit_start_exits_1()
{
  let out = run_clv_with_env(
    &[ ".version.mark", "name::1pin", "version::2.1.220" ],
    &[],
  );
  assert_exit( &out, 1 );
}

// ─── IT-14: name::stable (shadows built-in) → exit 1 ────────────────────────

#[ test ]
fn it14_mark_name_shadows_stable_exits_1()
{
  let out = run_clv_with_env(
    &[ ".version.mark", "name::stable", "version::2.1.220" ],
    &[],
  );
  assert_exit( &out, 1 );
}

// ─── IT-15: name::latest (shadows built-in) → exit 1 ────────────────────────

#[ test ]
fn it15_mark_name_shadows_latest_exits_1()
{
  let out = run_clv_with_env(
    &[ ".version.mark", "name::latest", "version::2.1.220" ],
    &[],
  );
  assert_exit( &out, 1 );
}

// ─── IT-16: version:: absent on set path → exit 1 ────────────────────────────

#[ test ]
fn it16_mark_version_absent_exits_1()
{
  let out = run_clv_with_env(
    &[ ".version.mark", "name::team-pin" ],
    &[],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "version" ), "error must reference missing version: {err}" );
}

// ─── IT-17: version::x (invalid spec) → exit 1 ───────────────────────────────

#[ test ]
fn it17_mark_version_invalid_exits_1()
{
  let out = run_clv_with_env(
    &[ ".version.mark", "name::team-pin", "version::x" ],
    &[],
  );
  assert_exit( &out, 1 );
}

// ─── FT-1: Create → appears in .version.list ─────────────────────────────────

#[ test ]
fn ft010_1_create_marker_appears_in_list()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let set_out = run_clv_with_env(
    &[ ".version.mark", "name::my-pin", "version::2.1.220" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &set_out, 0 );

  let list_out = run_clv_with_env( &[ ".version.list" ], &[ ( "HOME", home ) ] );
  assert_exit( &list_out, 0 );
  let text = stdout( &list_out );
  assert!( text.contains( "my-pin" ), ".version.list must show my-pin: {text}" );
}

// ─── FT-2: Remove → absent from .version.list ────────────────────────────────

#[ test ]
fn ft010_2_remove_marker_absent_from_list()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_markers( dir.path(), &[ ( "my-pin", "2.1.220" ) ] );

  let unset_out = run_clv_with_env(
    &[ ".version.mark", "name::my-pin", "unset::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &unset_out, 0 );

  let list_out = run_clv_with_env( &[ ".version.list" ], &[ ( "HOME", home ) ] );
  assert_exit( &list_out, 0 );
  let text = stdout( &list_out );
  assert!( !text.contains( "my-pin" ), ".version.list must not show removed marker: {text}" );
}

// ─── FT-3: Custom marker accepted by .version.install ────────────────────────

#[ test ]
fn ft010_3_custom_marker_accepted_by_install()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_markers( dir.path(), &[ ( "my-pin", "2.1.220" ) ] );

  let out = run_clv_with_env(
    &[ ".version.install", "version::my-pin", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "2.1.220" ), "install must resolve my-pin to 2.1.220: {text}" );
}

// ─── FT-4: Invalid name → exit 1 ─────────────────────────────────────────────

#[ test ]
fn ft010_4_invalid_name_exits_1()
{
  let out = run_clv_with_env(
    &[ ".version.mark", "name::MyPin", "version::2.1.220" ],
    &[],
  );
  assert_exit( &out, 1 );
}

// ─── IT-18: Malformed version-markers.json → graceful degradation ────────────

#[ test ]
fn it18_mark_malformed_json_graceful()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Write invalid JSON to the markers file so load_custom_markers() must
  // gracefully degrade rather than crash or return an error to the caller.
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "version-markers.json" ), "not valid json {{{" ).unwrap();

  let out = run_clv_with_env( &[ ".version.list" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "stable" ),
    ".version.list must show built-in aliases even with a malformed markers file: {text}"
  );
}

// ─── FT-5: dry::1 does not write markers file ────────────────────────────────

#[ test ]
fn ft010_5_dry_does_not_write_markers_file()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clv_with_env(
    &[ ".version.mark", "name::my-pin", "version::2.1.220", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  assert!( !markers_path( dir.path() ).exists(), "markers file must not be created in dry mode" );
}

// ─── IT-19: name:: (empty) → exit 1 ─────────────────────────────────────────

#[ test ]
fn it19_mark_name_empty_exits_1()
{
  let out = run_clv_with_env(
    &[ ".version.mark", "name::", "version::2.1.220" ],
    &[],
  );
  assert_exit( &out, 1 );
}

// ─── IT-20: name::{33 chars} → too long, exit 1 ──────────────────────────────

#[ test ]
fn it20_mark_name_too_long_exits_1()
{
  let name = format!( "name::{}", "a".repeat( 33 ) );
  let out = run_clv_with_env(
    &[ ".version.mark", &name, "version::2.1.220" ],
    &[],
  );
  assert_exit( &out, 1 );
}

// ─── RF-2: markers file created when absent, carrying a "markers" array ──────
//
// Spec: `tests/docs/runtime_file/004_version_markers.md` — RF-2.
//
// Distinct from IT-1 (`it01_mark_create_new`), which asserts only that the name
// and value substrings landed somewhere in the file. RF-2 additionally pins the
// on-disk container shape — a JSON object carrying a `"markers"` array — which is
// the structure `load_custom_markers()` must be able to parse back out. A writer
// that emitted a bare array, or a differently-named key, would satisfy IT-1 and
// still break every reader.

#[ test ]
fn rf004_2_mark_creates_markers_file_with_markers_array()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  assert!(
    !markers_path( dir.path() ).exists(),
    "precondition: markers file must be absent before the first .version.mark call"
  );

  let out = run_clv_with_env(
    &[ ".version.mark", "name::my-pin", "version::2.1.220" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  assert!(
    markers_path( dir.path() ).exists(),
    "markers file must exist at $HOME/.claude/version-markers.json after the call"
  );

  let contents = std::fs::read_to_string( markers_path( dir.path() ) ).unwrap();
  assert!(
    contents.trim_start().starts_with( '{' ),
    "markers file must be a JSON object, not a bare array: {contents}"
  );
  assert!(
    contents.contains( "\"markers\"" ),
    "markers file must carry a \"markers\" array: {contents}"
  );
  assert!(
    contents.contains( "my-pin" ),
    "markers array must contain the created marker name: {contents}"
  );
}

// ─── RF-3: absent markers file is safe — .version.list still exits 0 ─────────
//
// Spec: `tests/docs/runtime_file/004_version_markers.md` — RF-3 (durability:
// safe-to-lose). `load_custom_markers()` returns an empty vector for both a
// missing and an unparseable file; this is the file-absent half of that contract,
// IT-18 (`it18_mark_malformed_json_graceful`) is the malformed-content half.
//
// Unlike `ft010_2_remove_marker_absent_from_list`, which reaches an empty marker
// set by writing a file and then unsetting its entry, no `version-markers.json`
// ever exists here — the read path is exercised with the file genuinely missing.

#[ test ]
fn rf004_3_list_succeeds_when_markers_file_absent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  assert!(
    !markers_path( dir.path() ).exists(),
    "precondition: no version-markers.json may exist for this case"
  );

  let out = run_clv_with_env( &[ ".version.list" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  for alias in [ "stable", "latest" ]
  {
    assert!(
      text.contains( alias ),
      ".version.list must show built-in alias {alias} with no markers file present: {text}"
    );
  }

  assert!(
    !markers_path( dir.path() ).exists(),
    ".version.list must not create the markers file as a side effect"
  );
}
