//! Feature and edge-case tests: FT-01..FT-09 and EC-1..EC-7.
//!
//! Covers `set_model::` explicit session model override (Feature 034,
//! param 054). Each test maps to exactly one FT or EC case from the spec
//! documents in `tests/docs/`.
//!
//! All tests are offline (no live credentials required).
//!
//! ## Test Matrix
//!
//! | ID    | Test Function                                         | Condition                                                        | P/N |
//! |-------|-------------------------------------------------------|------------------------------------------------------------------|-----|
//! | FT-01 | `ft01_set_model_opus_writes_full_id`                  | `.account.use set_model::opus` → `claude-opus-4-6`               | P   |
//! | FT-02 | `ft02_set_model_sonnet_writes_full_id`                | `.account.use set_model::sonnet` → `claude-sonnet-4-6`           | P   |
//! | FT-03 | `ft03_set_model_haiku_writes_full_id`                 | `.account.use set_model::haiku` → `claude-haiku-4-5-20251001`    | P   |
//! | FT-04 | `ft04_set_model_default_removes_key_preserves_others` | `default` removes `model`; unrelated keys preserved              | P   |
//! | FT-05 | `ft05_explicit_set_model_wins_over_switch_restore`    | `set_model::sonnet` wins over `switch_account` model restore     | P   |
//! | FT-06 | `ft06_trace_line_emitted_with_set_model`              | `trace::1 set_model::opus` → `[trace]...set_model: opus`         | P   |
//! | FT-07 | `ft07_set_model_bad_value_exits_1`                    | `set_model::bad` → exit 1, stderr names all 4 valid values       | N   |
//! | FT-08 | `ft08_set_model_appears_in_help_output`               | `.account.use.help` and `.usage.help` both show `set_model`      | P   |
//! | FT-09 | `ft09_set_model_no_set_model_key_in_json`             | `format::json` output has no `set_model` key                     | P   |
//! | EC-1  | `ec1_set_model_opus_accepted_no_unrecognized_error`   | accepted; no "unrecognized" in stderr; writes `claude-opus-4-6`  | P   |
//! | EC-2  | `ec2_set_model_sonnet_accepted_writes_full_id`        | accepted; writes `claude-sonnet-4-6`                             | P   |
//! | EC-3  | `ec3_set_model_haiku_accepted_writes_full_id`         | accepted; writes `claude-haiku-4-5-20251001`                     | P   |
//! | EC-4  | `ec4_set_model_default_accepted_removes_key`          | accepted; removes `model` key from `settings.json`               | P   |
//! | EC-5  | `ec5_set_model_bad_exits_1_all_valid_values_named`    | exit 1; stderr names opus, sonnet, haiku, default                | N   |
//! | EC-6  | `ec6_account_use_set_model_wins_over_switch_restore`  | explicit `set_model::sonnet` wins over `switch_account` restore  | P   |
//! | EC-7  | `ec7_usage_set_model_writes_to_settings`              | `.usage set_model::sonnet` writes `claude-sonnet-4-6`            | P   |
//! | CC-1  | `cc1_account_use_set_model_bad_exits_1`               | `.account.use set_model::bad` exits 1 (same as FT-07 on .usage) | N   |
//! | CC-2  | `cc2_account_use_dry_run_does_not_write_settings`     | `dry::1` early-return → settings.json NOT written               | P   |
//! | CC-3  | `cc3_usage_set_model_format_json_also_writes_settings`| `format::json` + `set_model::opus` still writes settings.json   | P   |
//! | CC-4  | `cc4_set_model_uppercase_exits_1`                     | `set_model::Opus` (wrong case) exits 1                          | N   |
//! | CC-6  | `cc6_usage_set_model_default_removes_key`             | `.usage set_model::default` removes `model` key                 | P   |
//! | CC-7  | `cc7_usage_set_model_haiku_overwrites_existing_opus`  | `.usage set_model::haiku` overwrites pre-seeded opus            | P   |
//! | CC-8  | `cc8_usage_set_model_creates_dir_when_absent`         | `~/.claude/` absent → `set_session_model` creates dir + writes  | P   |
//! | CC-13 | `cc13_usage_set_model_no_trace_line_emitted`          | `.usage trace::1 set_model::` emits NO set_model trace line     | P   |

use crate::cli_runner::{
  run_cs, run_cs_with_env,
  stdout, stderr, assert_exit,
  write_credentials, write_account,
  write_settings_json, write_account_settings_json,
  FAR_FUTURE_MS,
};
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

/// FT-01 (AC-01): `set_model::opus` writes `"claude-opus-4-6"` to `settings.json`. Exit 0.
#[ test ]
fn ft01_set_model_opus_writes_full_id()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@example.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@example.com", "set_model::opus", "touch::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let model = read_settings_model( dir.path() );
  assert_eq!(
    model.as_deref(),
    Some( "claude-opus-4-6" ),
    "set_model::opus must write `claude-opus-4-6` to settings.json, got: {model:?}\nstdout: {}\nstderr: {}",
    stdout( &out ), stderr( &out ),
  );
}

/// FT-02 (AC-02): `set_model::sonnet` writes `"claude-sonnet-4-6"` to `settings.json`. Exit 0.
#[ test ]
fn ft02_set_model_sonnet_writes_full_id()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@example.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@example.com", "set_model::sonnet", "touch::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let model = read_settings_model( dir.path() );
  assert_eq!(
    model.as_deref(),
    Some( "claude-sonnet-4-6" ),
    "set_model::sonnet must write `claude-sonnet-4-6` to settings.json, got: {model:?}",
  );
}

/// FT-03 (AC-03): `set_model::haiku` writes `"claude-haiku-4-5-20251001"` to `settings.json`.
/// Exit 0.
#[ test ]
fn ft03_set_model_haiku_writes_full_id()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@example.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@example.com", "set_model::haiku", "touch::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let model = read_settings_model( dir.path() );
  assert_eq!(
    model.as_deref(),
    Some( "claude-haiku-4-5-20251001" ),
    "set_model::haiku must write `claude-haiku-4-5-20251001` to settings.json, got: {model:?}",
  );
}

/// FT-04 (AC-04): `set_model::default` removes the `model` key from `settings.json`;
/// unrelated keys (e.g. `theme`) are preserved. Exit 0.
#[ test ]
fn ft04_set_model_default_removes_key_preserves_others()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@example.com", "max", "default", FAR_FUTURE_MS, false );

  // Pre-seed settings.json with a model AND an unrelated key.
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write(
    claude_dir.join( "settings.json" ),
    r#"{"model":"claude-opus-4-6","theme":"dark"}"#,
  ).unwrap();

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@example.com", "set_model::default", "touch::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string(
    dir.path().join( ".claude" ).join( "settings.json" ),
  ).expect( "settings.json must exist after set_model::default" );
  assert!(
    !content.contains( "\"model\"" ),
    "settings.json must not contain `model` key after set_model::default, got: {content}",
  );
  assert!(
    content.contains( "\"theme\"" ),
    "settings.json must preserve `theme` key after set_model::default, got: {content}",
  );
}

/// FT-05 (AC-05): Explicit `set_model::` wins over `switch_account`'s model restore.
///
/// `switch_account()` reads `{name}.json` and writes its saved `model` value to live
/// `settings.json`. When `set_model::` is explicit, the post-match write overwrites
/// that value — the explicit shorthand is always the final state.
#[ test ]
fn ft05_explicit_set_model_wins_over_switch_restore()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@example.com", "max", "default", FAR_FUTURE_MS, false );
  // {name}.json stores opus; switch_account restores it to settings.json first.
  write_account_settings_json( dir.path(), "alice@example.com", "claude-opus-4-6" );

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@example.com", "set_model::sonnet", "touch::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let model = read_settings_model( dir.path() );
  assert_eq!(
    model.as_deref(),
    Some( "claude-sonnet-4-6" ),
    "explicit set_model::sonnet must win over switch_account model restore (opus), got: {model:?}",
  );
}

/// FT-06 (AC-06): `trace::1` + `set_model::opus` emits
/// `[trace] account.use … set_model: opus` to stderr. Exit 0.
///
/// The trace emission is in the post-match block and fires regardless of `touch::`.
#[ test ]
fn ft06_trace_line_emitted_with_set_model()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@example.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@example.com", "set_model::opus", "trace::1", "touch::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    err.contains( "[trace] account.use" ),
    "trace::1 + set_model::opus must emit `[trace] account.use` line to stderr, got:\n{err}",
  );
  assert!(
    err.contains( "set_model: opus" ),
    "trace line must contain `set_model: opus`, got stderr:\n{err}",
  );
}

/// FT-07 (AC-07): `set_model::bad` exits 1; stderr names all four valid values.
///
/// Validation fires at argument parse time — no accounts needed in the store.
#[ test ]
fn ft07_set_model_bad_value_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env(
    &[ ".usage", "set_model::bad" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "opus" ) && err.contains( "sonnet" )
    && err.contains( "haiku" ) && err.contains( "default" ),
    "stderr must name all four valid set_model:: values; got:\n{err}",
  );
}

/// FT-08 (AC-08): `set_model::` appears in `.account.use.help` and `.usage.help`. Exit 0.
#[ test ]
fn ft08_set_model_appears_in_help_output()
{
  let use_out = run_cs( &[ ".account.use.help" ] );
  assert_exit( &use_out, 0 );
  let use_text = stdout( &use_out );
  assert!(
    use_text.contains( "set_model" ),
    "`.account.use.help` must list `set_model` param, got:\n{use_text}",
  );

  let usage_out = run_cs( &[ ".usage.help" ] );
  assert_exit( &usage_out, 0 );
  let usage_text = stdout( &usage_out );
  assert!(
    usage_text.contains( "set_model" ),
    "`.usage.help` must list `set_model` param, got:\n{usage_text}",
  );
}

/// FT-09 (AC-09): `set_model::` does not add a `"set_model"` key to `format::json` output.
///
/// `set_model::` operates on `settings.json` only; the JSON rows for each account
/// are unaffected by its presence.
#[ test ]
fn ft09_set_model_no_set_model_key_in_json()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@example.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "set_model::opus", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "\"set_model\"" ),
    "format::json output must not contain a `set_model` key, got:\n{text}",
  );
  // Verify it is parseable JSON — not a rendering artifact.
  let parsed : serde_json::Value = serde_json::from_str( text.trim() )
    .unwrap_or_else( |e| panic!( "output must be valid JSON: {e}\ngot:\n{text}" ) );
  assert!( parsed.is_array(), "JSON output must be an array, got:\n{text}" );
}

// ── EC: Edge Cases ────────────────────────────────────────────────────────────

/// EC-1: `set_model::opus` accepted — no "unrecognized" error in stderr;
/// writes `claude-opus-4-6`. Exit 0.
#[ test ]
fn ec1_set_model_opus_accepted_no_unrecognized_error()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@example.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@example.com", "set_model::opus", "touch::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    !err.contains( "unrecognized" ),
    "set_model::opus must not produce `unrecognized` error; got stderr:\n{err}",
  );
  let model = read_settings_model( dir.path() );
  assert_eq!(
    model.as_deref(),
    Some( "claude-opus-4-6" ),
    "set_model::opus must write `claude-opus-4-6`, got: {model:?}",
  );
}

/// EC-2: `set_model::sonnet` accepted; writes `claude-sonnet-4-6`. Exit 0.
#[ test ]
fn ec2_set_model_sonnet_accepted_writes_full_id()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@example.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@example.com", "set_model::sonnet", "touch::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let model = read_settings_model( dir.path() );
  assert_eq!(
    model.as_deref(),
    Some( "claude-sonnet-4-6" ),
    "set_model::sonnet must write `claude-sonnet-4-6`, got: {model:?}",
  );
}

/// EC-3: `set_model::haiku` accepted; writes `claude-haiku-4-5-20251001`. Exit 0.
#[ test ]
fn ec3_set_model_haiku_accepted_writes_full_id()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@example.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@example.com", "set_model::haiku", "touch::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let model = read_settings_model( dir.path() );
  assert_eq!(
    model.as_deref(),
    Some( "claude-haiku-4-5-20251001" ),
    "set_model::haiku must write `claude-haiku-4-5-20251001`, got: {model:?}",
  );
}

/// EC-4: `set_model::default` accepted; removes the `model` key from `settings.json`.
/// Exit 0.
#[ test ]
fn ec4_set_model_default_accepted_removes_key()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@example.com", "max", "default", FAR_FUTURE_MS, false );
  write_settings_json( dir.path(), "claude-opus-4-6" );

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@example.com", "set_model::default", "touch::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude" ).join( "settings.json" ),
  ).expect( "settings.json must exist after set_model::default" );
  assert!(
    !content.contains( "\"model\"" ),
    "settings.json must not contain `model` key after set_model::default, got: {content}",
  );
}

/// EC-5: `set_model::bad` exits 1; stderr names all four valid values: opus, sonnet,
/// haiku, default.
#[ test ]
fn ec5_set_model_bad_exits_1_all_valid_values_named()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env(
    &[ ".usage", "set_model::bad" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "opus" ) && err.contains( "sonnet" )
    && err.contains( "haiku" ) && err.contains( "default" ),
    "stderr must name all four valid set_model:: values; got:\n{err}",
  );
}

/// EC-6: `.account.use` with `set_model::sonnet` — explicit override wins over
/// `switch_account`'s per-account model restore. Exit 0.
///
/// `switch_account()` restores the saved model from `{name}.json` to `settings.json`.
/// The post-match block then writes the explicit value, so the explicit shorthand
/// is always the final state regardless of what was in `{name}.json`.
#[ test ]
fn ec6_account_use_set_model_wins_over_switch_restore()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@example.com", "max", "default", FAR_FUTURE_MS, false );
  // Pre-seed {name}.json with opus so switch_account restores opus first.
  write_account_settings_json( dir.path(), "alice@example.com", "claude-opus-4-6" );

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@example.com", "set_model::sonnet", "touch::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let model = read_settings_model( dir.path() );
  assert_eq!(
    model.as_deref(),
    Some( "claude-sonnet-4-6" ),
    "set_model::sonnet must win over switch_account model restore (opus), got: {model:?}",
  );
}

/// EC-7: `.usage set_model::sonnet` writes `"claude-sonnet-4-6"` to `settings.json`.
///
/// When `set_model` is `Some`, the `apply_model_override` branch in `usage_routine`
/// is skipped (if-else mutual exclusion). Asserting that the pre-seeded opus value
/// is overwritten with sonnet proves the `set_model` branch ran exclusively.
#[ test ]
fn ec7_usage_set_model_writes_to_settings()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@example.com", "max", "default", FAR_FUTURE_MS, false );
  // Pre-seed settings.json with opus to simulate what apply_model_override would write.
  write_settings_json( dir.path(), "claude-opus-4-6" );

  let out = run_cs_with_env(
    &[ ".usage", "set_model::sonnet" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let model = read_settings_model( dir.path() );
  assert_eq!(
    model.as_deref(),
    Some( "claude-sonnet-4-6" ),
    "`.usage set_model::sonnet` must write `claude-sonnet-4-6` to settings.json, got: {model:?}",
  );
}

// ── CC: Corner Cases ──────────────────────────────────────────────────────────

/// CC-1: `set_model::bad` on `.account.use` exits 1 with all valid values in stderr.
///
/// FT-07 / EC-5 cover `.usage`; this confirms the same validation fires on `.account.use`.
#[ test ]
fn cc1_account_use_set_model_bad_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@example.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@example.com", "set_model::bad" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "opus" ) && err.contains( "sonnet" )
    && err.contains( "haiku" ) && err.contains( "default" ),
    "`.account.use set_model::bad` must name all four valid values in stderr; got:\n{err}",
  );
}

/// CC-2: `.account.use set_model::opus dry::1` — dry-run exits early; settings.json NOT written.
///
/// The `is_dry()` early-return in `account_use_routine` fires before the post-match
/// `set_session_model` block, so dry-run has no side effects on settings.json.
#[ test ]
fn cc2_account_use_dry_run_does_not_write_settings()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@example.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@example.com", "set_model::opus", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let model = read_settings_model( dir.path() );
  assert!(
    model.is_none(),
    "dry-run must not write settings.json; got model={model:?}\nstdout: {}",
    stdout( &out ),
  );
  let out_text = stdout( &out );
  assert!(
    out_text.contains( "dry-run" ),
    "stdout must contain `dry-run`; got: {out_text}",
  );
}

/// CC-3: `.usage set_model::opus format::json` also writes to settings.json.
///
/// FT-09 verifies the JSON output has no `set_model` key but does not assert
/// that settings.json itself was written. This test confirms the write side-effect
/// fires regardless of the output format, even when `~/.claude/` was not pre-created.
///
/// Without the fix to `set_session_model` (add `create_dir_all`), this test fails
/// because `fs::write` silently drops the `NotFound` error when the directory is absent.
#[ test ]
fn cc3_usage_set_model_format_json_also_writes_settings()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Only credential store — ~/.claude/ intentionally absent.
  write_account( dir.path(), "alice@example.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "set_model::opus", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let model = read_settings_model( dir.path() );
  assert_eq!(
    model.as_deref(),
    Some( "claude-opus-4-6" ),
    "`.usage set_model::opus format::json` must write `claude-opus-4-6` to settings.json even when ~/.claude/ was absent; got: {model:?}\nstderr: {}",
    stderr( &out ),
  );
}

/// CC-4: `set_model::Opus` (wrong case) exits 1 — the validator is case-sensitive.
#[ test ]
fn cc4_set_model_uppercase_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env(
    &[ ".usage", "set_model::Opus" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "opus" ),
    "`set_model::Opus` must exit 1 and mention the valid value `opus` in stderr; got:\n{err}",
  );
}

/// CC-6: `.usage set_model::default` removes the `model` key from `settings.json`.
///
/// AC-04 is covered on `.account.use` (FT-04, EC-4); this confirms the same
/// removal behavior applies on the `.usage` path.
#[ test ]
fn cc6_usage_set_model_default_removes_key()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@example.com", "max", "default", FAR_FUTURE_MS, false );
  write_settings_json( dir.path(), "claude-opus-4-6" );

  let out = run_cs_with_env(
    &[ ".usage", "set_model::default" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude" ).join( "settings.json" ),
  ).expect( "settings.json must exist after set_model::default" );
  assert!(
    !content.contains( "\"model\"" ),
    "`.usage set_model::default` must remove the `model` key; got: {content}",
  );
}

/// CC-7: `.usage set_model::haiku` overwrites a pre-existing `claude-opus-4-6` value.
#[ test ]
fn cc7_usage_set_model_haiku_overwrites_existing_opus()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@example.com", "max", "default", FAR_FUTURE_MS, false );
  write_settings_json( dir.path(), "claude-opus-4-6" );

  let out = run_cs_with_env(
    &[ ".usage", "set_model::haiku" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let model = read_settings_model( dir.path() );
  assert_eq!(
    model.as_deref(),
    Some( "claude-haiku-4-5-20251001" ),
    "`.usage set_model::haiku` must overwrite `claude-opus-4-6` with `claude-haiku-4-5-20251001`; got: {model:?}",
  );
}

/// CC-8 (BUG-258): `.usage set_model::opus` when `~/.claude/` dir doesn't exist — write must succeed.
///
/// # Root Cause
/// `set_session_model()` called `fs::write(path, ...)` without first calling
/// `create_dir_all(path.parent())`. When `~/.claude/` was absent, `fs::write` failed
/// with `NotFound`; `let _` silently discarded the error. The model was not written,
/// violating AC-01/AC-02/AC-03 for the `.usage` path.
///
/// # Why Not Caught
/// All prior `.usage set_model::` tests either called `write_settings_json()` (which
/// runs `create_dir_all`) or did not assert on the settings.json content (FT-09).
///
/// # Fix Applied
/// `set_session_model()` now calls `create_dir_all(parent)` before `fs::write`
/// (`claude_profile_core/src/account.rs`).
///
/// # Prevention
/// Precondition assertion (`!dir.path().join(".claude").exists()`) confirms the test
/// starts with `~/.claude/` absent — if the fixture accidentally pre-creates it,
/// the test would be a false negative.
///
/// # Pitfall
/// The `.account.use` path is immune because `switch_account` pre-creates `~/.claude/`.
/// Tests using `write_account()` only (no `write_credentials()` / `write_settings_json()`)
/// will have `~/.claude/` absent — these exercise the bug path.
#[ doc = "bug_reproducer(BUG-258)" ]
#[ test ]
fn cc8_usage_set_model_creates_dir_when_absent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Only credential store — ~/.claude/ intentionally absent.
  write_account( dir.path(), "alice@example.com", "max", "default", FAR_FUTURE_MS, false );
  assert!(
    !dir.path().join( ".claude" ).exists(),
    "test precondition: ~/.claude/ must not exist before the run",
  );

  let out = run_cs_with_env(
    &[ ".usage", "set_model::opus" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let model = read_settings_model( dir.path() );
  assert_eq!(
    model.as_deref(),
    Some( "claude-opus-4-6" ),
    "`.usage set_model::opus` must write settings.json even when `~/.claude/` was absent; got: {model:?}\nstderr: {}",
    stderr( &out ),
  );
}

/// CC-13: `.usage set_model::opus trace::1` does NOT emit a `set_model:` trace line.
///
/// The `[trace] account.use  {name}  set_model: X` line is only emitted on
/// `.account.use` (AC-06). The `.usage` code path has no corresponding trace emission.
#[ test ]
fn cc13_usage_set_model_no_trace_line_emitted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@example.com", "max", "default", FAR_FUTURE_MS, false );
  write_settings_json( dir.path(), "claude-sonnet-4-6" );

  let out = run_cs_with_env(
    &[ ".usage", "set_model::opus", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    !err.contains( "set_model: opus" ),
    "`.usage set_model::opus trace::1` must NOT emit a `set_model: opus` trace line; got stderr:\n{err}",
  );
}
