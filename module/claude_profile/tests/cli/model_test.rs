//! Feature and integration tests for the `.model` get/set command.
//!
//! Covers Feature 035 (`docs/feature/035_model_command.md` AC-01..AC-14),
//! command-level spec (`tests/docs/cli/command/17_model.md` IT-01..IT-13),
//! and `set::` parameter spec (`tests/docs/cli/param/55_set.md` EC-1..EC-6).
//! All tests are offline (no live credentials required).
//!
//! FT-01..FT-12 serve as implementations for IT-01..IT-12 (same scenarios).
//! IT-13 and EC-1 add scenarios not covered by the FT set.
//! EC-2..EC-6 are covered by FT-05..FT-09 respectively (no duplication).
//!
//! ## Test Matrix
//!
//! | ID    | Test Function                                   | Condition                                                       | P/N |
//! |-------|-------------------------------------------------|-----------------------------------------------------------------|-----|
//! | FT-01 | `ft01_get_model_opus`                           | `{"model":"opus"}` in settings.json → `model: opus\n`          | P   |
//! | FT-02 | `ft02_get_model_sonnet`                         | `{"model":"sonnet"}` → `model: sonnet\n`                        | P   |
//! | FT-03 | `ft03_get_model_unset_key_absent`               | key absent in settings.json → `model: (unset)\n`               | P   |
//! | FT-04 | `ft04_get_model_unset_file_absent`              | settings.json absent → `model: (unset)\n`                      | P   |
//! | FT-05 | `ft05_set_opus_writes_full_id`                  | `set::opus` → writes `claude-opus-4-8`                         | P   |
//! | FT-06 | `ft06_set_sonnet_writes_full_id`                | `set::sonnet` → writes `claude-sonnet-5`                     | P   |
//! | FT-07 | `ft07_set_haiku_writes_full_id`                 | `set::haiku` → writes `claude-haiku-4-5-20251001`              | P   |
//! | FT-08 | `ft08_set_default_removes_key_preserves_others` | `set::default` removes model key; other keys preserved          | P   |
//! | FT-09 | `ft09_set_bad_value_exits_1`                    | `set::bad` → exit 1; stderr names all four valid values        | N   |
//! | FT-10 | `ft10_set_creates_file_when_absent`             | settings.json absent → file created with model key             | P   |
//! | FT-11 | `ft11_set_preserves_existing_keys`              | other keys in settings.json preserved after set                | P   |
//! | FT-12 | `ft12_get_json_format`                          | `format::json` → `{"model":"..."}` or `{"model":null}`        | P   |
//! | IT-13 | `it13_model_listed_in_help_output`              | `.model` appears in `clp .help` output                         | P   |
//! | EC-1  | `ec1_set_absent_get_mode_reads_and_prints`      | `set::` absent → get mode; settings.json NOT written           | P   |
//! | CC-A  | `cc_a_set_uppercase_opus_exits_1`               | `set::Opus` wrong case → exit 1 (validator is case-sensitive)  | N   |
//! | CC-B  | `cc_b_format_table_rejected`                    | `format::table` → exit 1 (explicitly rejected by handler)     | N   |
//! | CC-C  | `cc_c_malformed_settings_json_get_returns_unset`| malformed settings.json + get → `model: (unset)` (graceful)   | P   |
//! | CC-D  | `cc_d_malformed_settings_json_set_writes_model` | malformed settings.json + `set::opus` → model written          | P   |
//! | CC-E  | `cc_e_set_default_absent_settings_exits_0`      | `set::default` when settings.json absent → exit 0, no crash   | P   |
//! | CC-F  | `cc_f_model_null_value_treated_as_unset`        | `{"model": null}` → `model: (unset)` (null is not a string)   | P   |
//! | CC-G  | `cc_g_set_mode_stdout_format`                   | set mode stdout is `model set: opus\n` (shorthand, not ID)     | P   |
//! | CC-H  | `cc_h_set_overwrites_existing_model`            | `set::haiku` overwrites pre-existing `claude-opus-4-8`         | P   |

use crate::cli_runner::{ run_cs, run_cs_with_env, stdout, stderr, assert_exit };
use tempfile::TempDir;

// ── helpers ───────────────────────────────────────────────────────────────────

/// Read the `"model"` field from `~/.claude/settings.json` in a temp home.
///
/// Returns `None` when the file is absent, unparseable, or lacks a `"model"` key.
fn read_settings_model( home : &std::path::Path ) -> Option< String >
{
  let content = std::fs::read_to_string(
    home.join( ".claude" ).join( "settings.json" ),
  ).ok()?;
  let val : serde_json::Value = serde_json::from_str( &content ).ok()?;
  val.get( "model" )?.as_str().map( std::string::ToString::to_string )
}

// ── FT: Feature Tests ─────────────────────────────────────────────────────────

/// FT-01 (AC-01): `clp .model` when settings.json contains `{"model":"opus"}` → `model: opus\n`. Exit 0.
#[ test ]
fn ft01_get_model_opus()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), r#"{"model":"opus"}"# ).unwrap();

  let out = run_cs_with_env( &[ ".model" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!(
    text, "model: opus\n",
    "FT-01: expected `model: opus\\n`; got: {text:?}",
  );
}

/// FT-02 (AC-02): `clp .model` when settings.json contains `{"model":"sonnet"}` → `model: sonnet\n`. Exit 0.
#[ test ]
fn ft02_get_model_sonnet()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), r#"{"model":"sonnet"}"# ).unwrap();

  let out = run_cs_with_env( &[ ".model" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!(
    text, "model: sonnet\n",
    "FT-02: expected `model: sonnet\\n`; got: {text:?}",
  );
}

/// FT-03 (AC-03): `clp .model` when settings.json exists but has no `model` key → `model: (unset)\n`.
/// Exit 0.
#[ test ]
fn ft03_get_model_unset_key_absent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), "{}" ).unwrap();

  let out = run_cs_with_env( &[ ".model" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!(
    text, "model: (unset)\n",
    "FT-03: expected `model: (unset)\\n` when key absent; got: {text:?}",
  );
}

/// FT-04 (AC-04): `clp .model` when settings.json does not exist → `model: (unset)\n`. Exit 0.
#[ test ]
fn ft04_get_model_unset_file_absent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!(
    text, "model: (unset)\n",
    "FT-04: expected `model: (unset)\\n` when file absent; got: {text:?}\nstderr: {}",
    stderr( &out ),
  );
}

/// FT-05 (AC-05): `clp .model set::opus` writes `claude-opus-4-8` to settings.json. Exit 0.
#[ test ]
fn ft05_set_opus_writes_full_id()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model", "set::opus" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let model = read_settings_model( dir.path() );
  assert_eq!(
    model.as_deref(), Some( "claude-opus-4-8" ),
    "FT-05: set::opus must write `claude-opus-4-8`; got: {model:?}\nstderr: {}",
    stderr( &out ),
  );
}

/// FT-06 (AC-06): `clp .model set::sonnet` writes `claude-sonnet-5` to settings.json. Exit 0.
#[ test ]
fn ft06_set_sonnet_writes_full_id()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model", "set::sonnet" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let model = read_settings_model( dir.path() );
  assert_eq!(
    model.as_deref(), Some( "claude-sonnet-5" ),
    "FT-06: set::sonnet must write `claude-sonnet-5`; got: {model:?}",
  );
}

/// FT-07 (AC-07): `clp .model set::haiku` writes `claude-haiku-4-5-20251001` to settings.json.
/// Exit 0.
#[ test ]
fn ft07_set_haiku_writes_full_id()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model", "set::haiku" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let model = read_settings_model( dir.path() );
  assert_eq!(
    model.as_deref(), Some( "claude-haiku-4-5-20251001" ),
    "FT-07: set::haiku must write `claude-haiku-4-5-20251001`; got: {model:?}",
  );
}

/// FT-08 (AC-08): `clp .model set::default` removes the `model` key; other keys are preserved.
/// Exit 0.
#[ test ]
fn ft08_set_default_removes_key_preserves_others()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write(
    claude_dir.join( "settings.json" ),
    r#"{"model":"claude-opus-4-8","theme":"dark"}"#,
  ).unwrap();

  let out = run_cs_with_env( &[ ".model", "set::default" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string( claude_dir.join( "settings.json" ) )
    .expect( "settings.json must exist after set::default" );
  assert!(
    !content.contains( "\"model\"" ),
    "FT-08: settings.json must not contain `model` key after set::default; got: {content}",
  );
  assert!(
    content.contains( "\"theme\"" ),
    "FT-08: settings.json must preserve `theme` key after set::default; got: {content}",
  );
}

/// FT-09 (AC-09): `clp .model set::bad` exits 1; stderr names all four valid values. Exit 1.
///
/// Validation fires at argument parse time — no credential store required.
#[ test ]
fn ft09_set_bad_value_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model", "set::bad" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "opus" ) && err.contains( "sonnet" )
    && err.contains( "haiku" ) && err.contains( "default" ),
    "FT-09: stderr must name all four valid set:: values; got:\n{err}",
  );
}

/// FT-10 (AC-10): `clp .model set::opus` when settings.json absent → file created with model key.
/// Exit 0.
#[ test ]
fn ft10_set_creates_file_when_absent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  assert!(
    !dir.path().join( ".claude" ).join( "settings.json" ).exists(),
    "test precondition: settings.json must not exist before the run",
  );

  let out = run_cs_with_env( &[ ".model", "set::opus" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let model = read_settings_model( dir.path() );
  assert_eq!(
    model.as_deref(), Some( "claude-opus-4-8" ),
    "FT-10: set::opus must create settings.json with `claude-opus-4-8` when file was absent; got: {model:?}\nstderr: {}",
    stderr( &out ),
  );
}

/// FT-11 (AC-11): `clp .model set::opus` preserves pre-existing keys in settings.json. Exit 0.
#[ test ]
fn ft11_set_preserves_existing_keys()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write(
    claude_dir.join( "settings.json" ),
    r#"{"theme":"dark","autoUpdaterStatus":"disabled"}"#,
  ).unwrap();

  let out = run_cs_with_env( &[ ".model", "set::opus" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string( claude_dir.join( "settings.json" ) )
    .expect( "settings.json must exist after set::opus" );
  assert!(
    content.contains( "\"model\"" ) && content.contains( "claude-opus-4-8" ),
    "FT-11: settings.json must contain model key after set::opus; got: {content}",
  );
  assert!(
    content.contains( "\"theme\"" ) && content.contains( "\"autoUpdaterStatus\"" ),
    "FT-11: pre-existing keys must be preserved after set::opus; got: {content}",
  );
}

/// FT-12 (AC-12): `clp .model format::json` returns structured JSON. Exit 0.
///
/// Two variants:
/// - Model present in settings.json → `{"model":"opus"}\n`
/// - settings.json absent (model unset) → `{"model":null}\n`
#[ test ]
fn ft12_get_json_format()
{
  // Variant A: model present.
  {
    let dir  = TempDir::new().unwrap();
    let home = dir.path().to_str().unwrap();
    let claude_dir = dir.path().join( ".claude" );
    std::fs::create_dir_all( &claude_dir ).unwrap();
    std::fs::write( claude_dir.join( "settings.json" ), r#"{"model":"opus"}"# ).unwrap();

    let out = run_cs_with_env( &[ ".model", "format::json" ], &[ ( "HOME", home ) ] );
    assert_exit( &out, 0 );
    let text = stdout( &out );
    assert_eq!(
      text, "{\"model\":\"opus\"}\n",
      "FT-12 (present): expected `{{\"model\":\"opus\"}}\\n`; got: {text:?}",
    );
  }

  // Variant B: model absent.
  {
    let dir  = TempDir::new().unwrap();
    let home = dir.path().to_str().unwrap();

    let out = run_cs_with_env( &[ ".model", "format::json" ], &[ ( "HOME", home ) ] );
    assert_exit( &out, 0 );
    let text = stdout( &out );
    assert_eq!(
      text, "{\"model\":null}\n",
      "FT-12 (absent): expected `{{\"model\":null}}\\n`; got: {text:?}",
    );
  }
}

// ── IT: Integration Tests (command-level) ─────────────────────────────────────

/// IT-13 (AC-13): `clp .help` output contains `.model`. Exit 0.
///
/// FT-01..FT-12 serve as implementations for IT-01..IT-12 (identical scenarios).
#[ test ]
fn it13_model_listed_in_help_output()
{
  let out = run_cs( &[ ".help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( ".model" ),
    "IT-13: `clp .help` must list `.model`; got:\n{text}",
  );
}

// ── EC: Edge Cases (`set::` parameter) ───────────────────────────────────────

/// EC-1 (`55_set.md`): `set::` absent → `.model` operates in get mode; settings.json NOT written.
///
/// Get mode must be read-only: no write side-effect on `settings.json`.
/// EC-2..EC-6 are covered by FT-05..FT-09 respectively (same scenarios, no duplication).
#[ test ]
fn ec1_set_absent_get_mode_reads_and_prints()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  let settings_path = claude_dir.join( "settings.json" );
  let original = r#"{"model":"sonnet"}"#;
  std::fs::write( &settings_path, original ).unwrap();

  let out = run_cs_with_env( &[ ".model" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!(
    text, "model: sonnet\n",
    "EC-1: get mode must print `model: sonnet\\n`; got: {text:?}",
  );
  let after = std::fs::read_to_string( &settings_path )
    .expect( "settings.json must still exist after get mode" );
  assert_eq!(
    after, original,
    "EC-1: get mode must not write settings.json; content changed from {original:?} to {after:?}",
  );
}

// ── CC: Corner Cases ──────────────────────────────────────────────────────────

/// CC-A: `clp .model set::Opus` (wrong case) → exit 1.
///
/// The validator is strictly case-sensitive. Only lowercase `opus`, `sonnet`,
/// `haiku`, `default` are accepted. Mixed-case inputs must be rejected so that
/// typos fail loudly rather than silently using wrong model IDs.
#[ test ]
fn cc_a_set_uppercase_opus_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model", "set::Opus" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "opus" ),
    "CC-A: `set::Opus` must exit 1 and name the valid lowercase `opus` in stderr; got:\n{err}",
  );
}

/// CC-B: `clp .model format::table` → exit 1.
///
/// The handler explicitly rejects `format::table` because `.model` produces
/// key-value output, not tabular rows. The rejection fires before any I/O.
#[ test ]
fn cc_b_format_table_rejected()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model", "format::table" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

/// CC-C: `settings.json` contains malformed JSON → get mode returns `model: (unset)`.
///
/// `get_session_model` uses `parse_string_field` (simple text scan). When the
/// file is not valid JSON and the `"model":` pattern is absent, the scan returns
/// `None` and the output falls back to `(unset)`. No panic, no error propagation.
#[ test ]
fn cc_c_malformed_settings_json_get_returns_unset()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), b"{ INVALID JSON garbage" ).unwrap();

  let out = run_cs_with_env( &[ ".model" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!(
    text, "model: (unset)\n",
    "CC-C: malformed settings.json must yield `model: (unset)\\n`; got: {text:?}",
  );
}

/// CC-D: `settings.json` contains malformed JSON → `set::opus` recovers gracefully.
///
/// `set_session_model` parses `settings.json` with `serde_json`; on failure it
/// falls back to an empty `{}` object. The `model` key is inserted and the
/// result is written back. The corrupt content is replaced with valid JSON.
#[ test ]
fn cc_d_malformed_settings_json_set_writes_model()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), b"{ INVALID JSON garbage" ).unwrap();

  let out = run_cs_with_env( &[ ".model", "set::opus" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let model = read_settings_model( dir.path() );
  assert_eq!(
    model.as_deref(), Some( "claude-opus-4-8" ),
    "CC-D: malformed settings.json must be treated as {{}} — model must be written; got: {model:?}\nstderr: {}",
    stderr( &out ),
  );
}

/// CC-E: `set::default` when `settings.json` is absent → exit 0, no crash.
///
/// When there is no model key to remove, the command is a no-op semantically.
/// Implementation writes `{}` (or the file is created), which is safe — the
/// `set_session_model(None)` path removes the key from an empty object.
#[ test ]
fn cc_e_set_default_absent_settings_exits_0()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Precondition: neither ~/.claude/ nor settings.json exist.
  assert!(
    !dir.path().join( ".claude" ).join( "settings.json" ).exists(),
    "CC-E precondition: settings.json must be absent before the run",
  );

  let out = run_cs_with_env( &[ ".model", "set::default" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  // settings.json may or may not be created — what matters is exit 0 and no crash.
  // If created, it must contain no "model" key.
  if let Ok( content ) = std::fs::read_to_string( dir.path().join( ".claude" ).join( "settings.json" ) )
  {
    assert!(
      !content.contains( "\"model\"" ),
      "CC-E: if settings.json was created by set::default, it must not contain a model key; got: {content}",
    );
  }
}

/// CC-F: `settings.json` contains `{"model": null}` → get mode returns `model: (unset)`.
///
/// `parse_string_field` checks that the value starts with `"`. A JSON `null`
/// does not, so it returns `None` — null is treated identically to a missing key.
#[ test ]
fn cc_f_model_null_value_treated_as_unset()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), r#"{"model":null}"# ).unwrap();

  let out = run_cs_with_env( &[ ".model" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!(
    text, "model: (unset)\n",
    "CC-F: null model value must be treated as unset; got: {text:?}",
  );
}

/// CC-G: Set mode stdout is `model set: {shorthand}\n` (shorthand, not full model ID).
///
/// FT-05..FT-11 verify only the file content after `set::`. This test confirms the
/// stdout contract: the printed confirmation echoes the shorthand the user supplied,
/// not the resolved full ID.
#[ test ]
fn cc_g_set_mode_stdout_format()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model", "set::opus" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!(
    text, "model set: opus\n",
    "CC-G: set mode stdout must be `model set: opus\\n`; got: {text:?}",
  );
}

/// CC-H: `set::haiku` overwrites a pre-existing `claude-opus-4-8` model value.
///
/// Verifies the write is not additive — the existing `model` key is replaced,
/// not duplicated. After the overwrite, only `claude-haiku-4-5-20251001` must
/// appear as the `model` value.
#[ test ]
fn cc_h_set_overwrites_existing_model()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write(
    claude_dir.join( "settings.json" ),
    r#"{"model":"claude-opus-4-8"}"#,
  ).unwrap();

  let out = run_cs_with_env( &[ ".model", "set::haiku" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let model = read_settings_model( dir.path() );
  assert_eq!(
    model.as_deref(), Some( "claude-haiku-4-5-20251001" ),
    "CC-H: set::haiku must overwrite `claude-opus-4-8` with `claude-haiku-4-5-20251001`; got: {model:?}",
  );
}
