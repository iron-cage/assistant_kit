//! Integration tests: redirect-backend accounts (Feature 071) — `.account.save` CLI layer.
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! | ID  | Test Function                                             | Condition                                          | P/N |
//! |-----|------------------------------------------------------------|-----------------------------------------------------|-----|
//! | T01 | `t01_save_redirect_full_succeeds`                          | full redirect save → `kimi.json` + `.credentials.json` | P   |
//! | T02 | `t02_save_redirect_missing_required_param_exits_1`         | redirect save missing base_url/api_key/redirect_model → exit 1 | N |
//! | T03 | `t03_save_base_url_outside_redirect_exits_1`               | `base_url::` without `backend::redirect` → exit 1  | N   |
//! | T04 | `t04_save_no_backend_unchanged_from_pre071`                | no `backend::` at all → pre-071 behavior preserved | P   |
//! | T05 | `t05_accounts_and_credentials_status_no_backend_key_defaults_anthropic` | pre-existing account file, no `backend` key → treated as anthropic | P |
//! | T06 | `t06_use_redirect_account_writes_env_vars_and_skips_touch` | `.account.use` on redirect → `env.*` written, zero HTTP | P   |
//! | T07 | `t07_use_anthropic_after_redirect_clears_env_vars`         | `.account.use` on anthropic after redirect → `env.*` cleared | P |
//! | T10 | `t10_limits_and_inspect_reject_redirect_account_exit_1`    | `.account.limits`/`.account.inspect` on redirect → exit 1 | N |
//! | T11 | `t11_accounts_backend_column_text_and_json`                | `.accounts` backend column — opt-in text, always-on json | P |
//! | T12 | `t12_credentials_status_active_redirect_account_classifies_static` | `.credentials.status` on active redirect → `static` classification | P |
//! | T13 | `t13_save_resave_different_backend_rewrites_from_scratch`  | re-save redirect→anthropic → stale fields cleared  | P   |
//! | T14 | `t14_save_preset_kimi_fills_backend_base_url_and_inference_provider` | Feature 073: `preset::kimi` defaults backend/base_url/inference_provider | P |
//! | T15 | `t15_save_preset_kimi_explicit_base_url_overrides_default`  | Feature 073: explicit `base_url::` wins over `preset::kimi`'s default | P |
//! | T16 | `t16_save_preset_unrecognized_value_exits_1`                | Feature 073: `preset::` value other than `kimi` → exit 1 | N |
//! | T17 | `t17_use_preset_kimi_account_writes_kimi_tier_env_vars`     | Feature 073: `.account.use` on a `preset::kimi` account writes all 7 Kimi-tier env vars | P |
//! | T18 | `t18_save_preset_kimi_with_explicit_backend_anthropic_does_not_force_redirect_fields` | Feature 073: explicit `backend::anthropic` wins — preset does not force base_url/inference_provider | P |
//! | T19 | `t19_use_redirect_removes_stale_top_level_model_pin`        | redirect save never snapshots `model`; redirect switch removes a stale top-level pin | P |
//! | T20 | `t20_save_redirect_self_heals_stray_model_in_meta`          | redirect re-save removes a stray pre-gate `model` field; anthropic snapshot unaffected | P |
//! | T21 | `t21_usage_tsv_active_redirect_row_current_static`          | `.usage format::tsv` after switch-to-redirect → ✓ flag, `static` status, `static` expires | P |
//! | T22 | `t22_usage_text_redirect_row_compact_note_no_question_mark`  | BUG-538: text table shows compact `(redirect)` + `—` renews (no 40-char note, no `?`); TSV keeps full reason | P |
//! | T23 | `t23_usage_sub_and_get_fields_redirect_known_absence`        | BUG-540: `cols::+sub` cell (text+TSV) and `get::sub`/`get::renews` emit `—`, never `?`, on a redirect row | P |

use crate::cli_runner::{
  run_cs_with_env,
  assert_exit, stdout, stderr,
  write_credentials, credential_json, account_exists, read_account_meta, write_account,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

fn credentials_path( home : &std::path::Path, name : &str ) -> std::path::PathBuf
{
  home.join( ".persistent" ).join( "claude" ).join( "credential" ).join( format!( "{name}.credentials.json" ) )
}

#[ test ]
fn t01_save_redirect_full_succeeds()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env(
    &[
      ".account.save", "name::kimi", "backend::redirect",
      "base_url::https://api.moonshot.ai/anthropic", "api_key::sk-test",
      "redirect_model::kimi-k3",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let meta = read_account_meta( dir.path(), "kimi" );
  assert_eq!( meta[ "backend" ], serde_json::json!( "redirect" ), "T01: backend must be redirect, got:\n{meta}" );
  assert_eq!( meta[ "base_url" ], serde_json::json!( "https://api.moonshot.ai/anthropic" ), "T01: base_url mismatch, got:\n{meta}" );
  assert_eq!( meta[ "redirect_model" ], serde_json::json!( "kimi-k3" ), "T01: redirect_model mismatch, got:\n{meta}" );

  let creds_text = std::fs::read_to_string( credentials_path( dir.path(), "kimi" ) ).unwrap();
  let creds : serde_json::Value = serde_json::from_str( &creds_text ).unwrap();
  assert_eq!( creds[ "accessToken" ], serde_json::json!( "sk-test" ), "T01: accessToken mismatch, got:\n{creds_text}" );
  assert_eq!( creds.as_object().unwrap().len(), 1, "T01: redirect credentials must contain only accessToken, got:\n{creds_text}" );
}

#[ test ]
fn t02_save_redirect_missing_required_param_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out1 = run_cs_with_env(
    &[
      ".account.save", "name::kimi", "backend::redirect",
      "api_key::sk-test", "redirect_model::kimi-k3",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out1, 1 );
  assert!(
    stderr( &out1 ).contains( "base_url::" ),
    "T02: stderr must name missing base_url::, got:\n{}", stderr( &out1 ),
  );
  assert!( !account_exists( dir.path(), "kimi" ), "T02: rejected save must not write files (missing base_url::)" );

  let out2 = run_cs_with_env(
    &[
      ".account.save", "name::kimi", "backend::redirect",
      "base_url::https://api.moonshot.ai/anthropic", "redirect_model::kimi-k3",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out2, 1 );
  assert!(
    stderr( &out2 ).contains( "api_key::" ),
    "T02: stderr must name missing api_key::, got:\n{}", stderr( &out2 ),
  );
  assert!( !account_exists( dir.path(), "kimi" ), "T02: rejected save must not write files (missing api_key::)" );

  let out3 = run_cs_with_env(
    &[
      ".account.save", "name::kimi", "backend::redirect",
      "base_url::https://api.moonshot.ai/anthropic", "api_key::sk-test",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out3, 1 );
  assert!(
    stderr( &out3 ).contains( "redirect_model::" ),
    "T02: stderr must name missing redirect_model::, got:\n{}", stderr( &out3 ),
  );
  assert!( !account_exists( dir.path(), "kimi" ), "T02: rejected save must not write files (missing redirect_model::)" );
}

#[ test ]
fn t03_save_base_url_outside_redirect_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".account.save", "name::alice@acme.com", "base_url::https://x" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  assert!( !account_exists( dir.path(), "alice@acme.com" ), "T03: rejected save must not write files" );
}

#[ test ]
fn t04_save_no_backend_unchanged_from_pre071()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let meta = read_account_meta( dir.path(), "alice@acme.com" );
  assert_eq!( meta[ "backend" ], serde_json::json!( "anthropic" ), "T04: backend must default to anthropic, got:\n{meta}" );

  let saved = std::fs::read_to_string( credentials_path( dir.path(), "alice@acme.com" ) ).unwrap();
  assert_eq!( saved, credential_json( "pro", "standard", FAR_FUTURE_MS ), "T04: must copy live ~/.claude/.credentials.json byte-for-byte" );
}

#[ test ]
fn t05_accounts_and_credentials_status_no_backend_key_defaults_anthropic()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Pre-existing account: credentials file only, no `{name}.json` meta at all — the
  // strongest realistic form of "no backend key" (read_backend() degrades identically
  // whether the meta file is absent or present-but-missing the key).
  write_account( dir.path(), "alice@test.com", "max", "default", FAR_FUTURE_MS, true );
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

  let accounts_out = run_cs_with_env( &[ ".accounts", "cols::+backend" ], &[ ( "HOME", home ) ] );
  assert_exit( &accounts_out, 0 );
  assert!(
    stdout( &accounts_out ).contains( "Backend: anthropic" ),
    "T05: account with no backend key must default to anthropic in .accounts, got:\n{}", stdout( &accounts_out ),
  );

  let status_out = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &status_out, 0 );
  let status_text = stdout( &status_out );
  assert!(
    status_text.contains( "Token:   valid" ),
    "T05: active account with no backend key must classify normally (valid), not misclassify as static, got:\n{status_text}",
  );
  assert!(
    !status_text.contains( "static" ),
    "T05: must not misclassify a no-backend-key account as static, got:\n{status_text}",
  );
}

#[ test ]
fn t06_use_redirect_account_writes_env_vars_and_skips_touch()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let save_out = run_cs_with_env(
    &[
      ".account.save", "name::kimi", "backend::redirect",
      "base_url::https://api.moonshot.ai/anthropic", "api_key::sk-test",
      "redirect_model::kimi-k3",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &save_out, 0 );

  let use_out = run_cs_with_env( &[ ".account.use", "name::kimi", "trace::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &use_out, 0 );

  let err = stderr( &use_out );
  assert!(
    err.contains( "subprocess: skipped (reason: redirect backend)" ),
    "T06/AC-16: touch subprocess must be skipped unconditionally for a redirect target, got:\n{err}",
  );
  assert!(
    !err.contains( "account.use  kimi  reading " ),
    "T06/AC-16: no credential-file read for quota fetch must occur before the redirect skip, got:\n{err}",
  );

  let settings_text = std::fs::read_to_string( dir.path().join( ".claude" ).join( "settings.json" ) )
    .expect( "T06: settings.json must exist after switch" );
  let settings : serde_json::Value = serde_json::from_str( &settings_text ).unwrap();
  let env = settings.get( "env" ).expect( "T06: settings.json must gain an env object" );
  assert_eq!(
    env[ "ANTHROPIC_BASE_URL" ], serde_json::json!( "https://api.moonshot.ai/anthropic" ),
    "T06: ANTHROPIC_BASE_URL mismatch, got:\n{settings_text}",
  );
  assert_eq!(
    env[ "ANTHROPIC_AUTH_TOKEN" ], serde_json::json!( "sk-test" ),
    "T06: ANTHROPIC_AUTH_TOKEN mismatch, got:\n{settings_text}",
  );
  assert_eq!(
    env[ "ANTHROPIC_MODEL" ], serde_json::json!( "kimi-k3" ),
    "T06: ANTHROPIC_MODEL mismatch, got:\n{settings_text}",
  );
}

#[ test ]
fn t07_use_anthropic_after_redirect_clears_env_vars()
{
  // Scenario A: env becomes empty after clearing → the env key is removed entirely.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let kimi_save = run_cs_with_env(
    &[
      ".account.save", "name::kimi", "backend::redirect",
      "base_url::https://api.moonshot.ai/anthropic", "api_key::sk-test",
      "redirect_model::kimi-k3",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &kimi_save, 0 );
  assert_exit( &run_cs_with_env( &[ ".account.use", "name::kimi" ], &[ ( "HOME", home ) ] ), 0 );

  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  let alice_save = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &alice_save, 0 );

  let switch_out = run_cs_with_env( &[ ".account.use", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &switch_out, 0 );

  let settings_text = std::fs::read_to_string( dir.path().join( ".claude" ).join( "settings.json" ) )
    .expect( "T07: settings.json must exist" );
  let settings : serde_json::Value = serde_json::from_str( &settings_text ).unwrap();
  assert!(
    settings.get( "env" ).is_none(),
    "T07: env object must be removed entirely once empty, got:\n{settings_text}",
  );

  // Scenario B: an unrelated env.* sub-key present before the redirect switch survives
  // both the redirect switch (untouched) and the subsequent anthropic switch (only the
  // three ANTHROPIC_* keys are cleared).
  let dir2        = TempDir::new().unwrap();
  let home2       = dir2.path().to_str().unwrap();
  let claude_dir2 = dir2.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir2 ).unwrap();
  std::fs::write( claude_dir2.join( "settings.json" ), r#"{"env":{"UNRELATED_VAR":"keep-me"}}"# ).unwrap();

  let kimi_save2 = run_cs_with_env(
    &[
      ".account.save", "name::kimi", "backend::redirect",
      "base_url::https://api.moonshot.ai/anthropic", "api_key::sk-test",
      "redirect_model::kimi-k3",
    ],
    &[ ( "HOME", home2 ) ],
  );
  assert_exit( &kimi_save2, 0 );
  assert_exit( &run_cs_with_env( &[ ".account.use", "name::kimi" ], &[ ( "HOME", home2 ) ] ), 0 );

  write_credentials( dir2.path(), "max", "tier4", FAR_FUTURE_MS );
  let alice_save2 = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home2 ) ] );
  assert_exit( &alice_save2, 0 );

  let switch_out2 = run_cs_with_env( &[ ".account.use", "name::alice@acme.com" ], &[ ( "HOME", home2 ) ] );
  assert_exit( &switch_out2, 0 );

  let settings_text2 = std::fs::read_to_string( claude_dir2.join( "settings.json" ) ).unwrap();
  let settings2 : serde_json::Value = serde_json::from_str( &settings_text2 ).unwrap();
  let env2 = settings2.get( "env" )
    .expect( "T07: env object with unrelated key must survive, got settings.json missing env entirely" );
  assert_eq!(
    env2[ "UNRELATED_VAR" ], serde_json::json!( "keep-me" ),
    "T07: unrelated env.* sub-key must be preserved, got:\n{settings_text2}",
  );
  assert!( env2.get( "ANTHROPIC_BASE_URL" ).is_none(), "T07: ANTHROPIC_BASE_URL must be cleared, got:\n{settings_text2}" );
  assert!( env2.get( "ANTHROPIC_AUTH_TOKEN" ).is_none(), "T07: ANTHROPIC_AUTH_TOKEN must be cleared, got:\n{settings_text2}" );
  assert!( env2.get( "ANTHROPIC_MODEL" ).is_none(), "T07: ANTHROPIC_MODEL must be cleared, got:\n{settings_text2}" );
}

#[ test ]
fn t10_limits_and_inspect_reject_redirect_account_exit_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let save_out = run_cs_with_env(
    &[
      ".account.save", "name::kimi", "backend::redirect",
      "base_url::https://api.moonshot.ai/anthropic", "api_key::sk-test",
      "redirect_model::kimi-k3",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &save_out, 0 );

  let limits_out = run_cs_with_env( &[ ".account.limits", "name::kimi" ], &[ ( "HOME", home ) ] );
  assert_exit( &limits_out, 1 );
  assert!(
    stderr( &limits_out ).contains( "redirect backend" ),
    "T10: .account.limits must reject with an Anthropic-only guard message, got:\n{}", stderr( &limits_out ),
  );

  let inspect_out = run_cs_with_env( &[ ".account.inspect", "name::kimi" ], &[ ( "HOME", home ) ] );
  assert_exit( &inspect_out, 1 );
  assert!(
    stderr( &inspect_out ).contains( "redirect backend" ),
    "T10: .account.inspect must reject with an Anthropic-only guard message, got:\n{}", stderr( &inspect_out ),
  );

  // Same guard must fire on the implicit active-account path (no name:: at all) once kimi
  // is the active account — not only when name:: is passed explicitly.
  assert_exit( &run_cs_with_env( &[ ".account.use", "name::kimi" ], &[ ( "HOME", home ) ] ), 0 );

  let limits_active_out = run_cs_with_env( &[ ".account.limits" ], &[ ( "HOME", home ) ] );
  assert_exit( &limits_active_out, 1 );
  assert!(
    stderr( &limits_active_out ).contains( "redirect backend" ),
    "T10: .account.limits with no name:: must reject the active redirect account too, got:\n{}", stderr( &limits_active_out ),
  );

  let inspect_active_out = run_cs_with_env( &[ ".account.inspect" ], &[ ( "HOME", home ) ] );
  assert_exit( &inspect_active_out, 1 );
  assert!(
    stderr( &inspect_active_out ).contains( "redirect backend" ),
    "T10: .account.inspect with no name:: must reject the active redirect account too, got:\n{}", stderr( &inspect_active_out ),
  );
}

#[ test ]
fn t11_accounts_backend_column_text_and_json()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let kimi_save = run_cs_with_env(
    &[
      ".account.save", "name::kimi", "backend::redirect",
      "base_url::https://api.moonshot.ai/anthropic", "api_key::sk-test",
      "redirect_model::kimi-k3",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &kimi_save, 0 );

  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  let alice_save = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &alice_save, 0 );

  // Text mode: backend is opt-in via cols::+backend.
  let text_out = run_cs_with_env( &[ ".accounts", "cols::+backend" ], &[ ( "HOME", home ) ] );
  assert_exit( &text_out, 0 );
  let text = stdout( &text_out );
  assert!( text.contains( "Backend: redirect" ), "T11: kimi must show Backend: redirect, got:\n{text}" );
  assert!( text.contains( "Backend: anthropic" ), "T11: alice must show Backend: anthropic, got:\n{text}" );

  // JSON mode: backend is always present regardless of cols::.
  let json_out = run_cs_with_env( &[ ".accounts", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &json_out, 0 );
  let val : serde_json::Value = serde_json::from_str( &stdout( &json_out ) ).unwrap();
  let rows = val.as_array().expect( "T11: .accounts format::json must return a JSON array" );
  assert_eq!( rows.len(), 2, "T11: expected exactly 2 account rows, got:\n{val}" );
  for row in rows
  {
    assert!( row.get( "backend" ).is_some(), "T11: every row must carry a backend field regardless of cols::, got:\n{row}" );
  }
  let kimi_row = rows.iter().find( |r| r[ "name" ] == "kimi" ).expect( "T11: kimi row must be present" );
  assert_eq!( kimi_row[ "backend" ], serde_json::json!( "redirect" ), "T11: kimi backend field mismatch" );
  let alice_row = rows.iter().find( |r| r[ "name" ] == "alice@acme.com" ).expect( "T11: alice row must be present" );
  assert_eq!( alice_row[ "backend" ], serde_json::json!( "anthropic" ), "T11: alice backend field mismatch" );
}

#[ test ]
fn t12_credentials_status_active_redirect_account_classifies_static()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let save_out = run_cs_with_env(
    &[
      ".account.save", "name::kimi", "backend::redirect",
      "base_url::https://api.moonshot.ai/anthropic", "api_key::sk-test",
      "redirect_model::kimi-k3",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &save_out, 0 );
  assert_exit( &run_cs_with_env( &[ ".account.use", "name::kimi" ], &[ ( "HOME", home ) ] ), 0 );

  let status_out = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &status_out, 0 );
  let text = stdout( &status_out );
  assert!( text.contains( "Token:   static" ), "T12: active redirect account must classify as static, got:\n{text}" );
  assert!( text.contains( "Expires: no expiry" ), "T12: static token must show no expiry, got:\n{text}" );

  let json_out = run_cs_with_env( &[ ".credentials.status", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &json_out, 0 );
  let json_text = stdout( &json_out );
  let val : serde_json::Value = serde_json::from_str( &json_text ).unwrap();
  assert_eq!( val[ "token" ], serde_json::json!( "static" ), "T12: json token field mismatch, got:\n{json_text}" );
  assert_eq!( val[ "expires_in_secs" ], serde_json::json!( 0 ), "T12: json expires_in_secs mismatch, got:\n{json_text}" );
  assert!( val.get( "backend" ).is_none(), "T12: json must NOT include a backend field (design decision), got:\n{json_text}" );
}

#[ test ]
fn t13_save_resave_different_backend_rewrites_from_scratch()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // First save: kimi as a redirect account.
  let out1 = run_cs_with_env(
    &[
      ".account.save", "name::kimi", "backend::redirect",
      "base_url::https://api.moonshot.ai/anthropic", "api_key::sk-test",
      "redirect_model::kimi-k3",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out1, 0 );

  // Re-save the same name as an anthropic account.
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  let out2 = run_cs_with_env( &[ ".account.save", "name::kimi", "backend::anthropic" ], &[ ( "HOME", home ) ] );
  assert_exit( &out2, 0 );

  let meta = read_account_meta( dir.path(), "kimi" );
  assert_eq!( meta[ "backend" ], serde_json::json!( "anthropic" ), "T13: backend must flip to anthropic, got:\n{meta}" );
  assert!( meta.get( "base_url" ).is_none(), "T13: stale base_url must be cleared, got:\n{meta}" );
  assert!( meta.get( "redirect_model" ).is_none(), "T13: stale redirect_model must be cleared, got:\n{meta}" );

  let saved = std::fs::read_to_string( credentials_path( dir.path(), "kimi" ) ).unwrap();
  assert_eq!( saved, credential_json( "max", "tier4", FAR_FUTURE_MS ), "T13: must capture current live credentials, not the stale redirect payload" );
}

// ── Feature 073 — Kimi provider preset (`preset::kimi`) ─────────────────────────

#[ test ]
fn t14_save_preset_kimi_fills_backend_base_url_and_inference_provider()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env(
    &[
      ".account.save", "name::kimi", "preset::kimi",
      "api_key::sk-test", "redirect_model::kimi-k3",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let meta = read_account_meta( dir.path(), "kimi" );
  assert_eq!( meta[ "backend" ], serde_json::json!( "redirect" ), "T14: preset::kimi must default backend to redirect, got:\n{meta}" );
  assert_eq!(
    meta[ "base_url" ], serde_json::json!( "https://api.moonshot.ai/anthropic" ),
    "T14: preset::kimi must default base_url to Moonshot's endpoint, got:\n{meta}",
  );
  assert_eq!( meta[ "inference_provider" ], serde_json::json!( "kimi" ), "T14: preset::kimi must default inference_provider to kimi, got:\n{meta}" );
  assert_eq!( meta[ "redirect_model" ], serde_json::json!( "kimi-k3" ), "T14: redirect_model must still come from the explicit param, got:\n{meta}" );

  let creds_text = std::fs::read_to_string( credentials_path( dir.path(), "kimi" ) ).unwrap();
  let creds : serde_json::Value = serde_json::from_str( &creds_text ).unwrap();
  assert_eq!( creds[ "accessToken" ], serde_json::json!( "sk-test" ), "T14: accessToken mismatch, got:\n{creds_text}" );
}

#[ test ]
fn t15_save_preset_kimi_explicit_base_url_overrides_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env(
    &[
      ".account.save", "name::kimi", "preset::kimi",
      "base_url::https://custom.mirror.example/anthropic",
      "api_key::sk-test", "redirect_model::kimi-k3",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let meta = read_account_meta( dir.path(), "kimi" );
  assert_eq!(
    meta[ "base_url" ], serde_json::json!( "https://custom.mirror.example/anthropic" ),
    "T15: an explicit base_url:: must override preset::kimi's default, got:\n{meta}",
  );
}

#[ test ]
fn t16_save_preset_unrecognized_value_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env(
    &[ ".account.save", "name::kimi", "preset::openai", "api_key::sk-test", "redirect_model::gpt-5" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "preset::" ) && stderr( &out ).contains( "kimi" ),
    "T16: stderr must name preset:: and the one valid value (kimi), got:\n{}", stderr( &out ),
  );
  assert!( !account_exists( dir.path(), "kimi" ), "T16: rejected save must not write files" );
}

#[ test ]
fn t17_use_preset_kimi_account_writes_kimi_tier_env_vars()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let save_out = run_cs_with_env(
    &[
      ".account.save", "name::kimi", "preset::kimi",
      "api_key::sk-test", "redirect_model::kimi-k3",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &save_out, 0 );

  let use_out = run_cs_with_env( &[ ".account.use", "name::kimi" ], &[ ( "HOME", home ) ] );
  assert_exit( &use_out, 0 );

  let settings_text = std::fs::read_to_string( dir.path().join( ".claude" ).join( "settings.json" ) ).unwrap();
  let settings : serde_json::Value = serde_json::from_str( &settings_text ).unwrap();
  let env = settings.get( "env" ).expect( "T17: settings.json must gain an env object" );
  for key in [ "ANTHROPIC_DEFAULT_OPUS_MODEL", "ANTHROPIC_DEFAULT_SONNET_MODEL", "ANTHROPIC_DEFAULT_HAIKU_MODEL", "ANTHROPIC_DEFAULT_FABLE_MODEL", "CLAUDE_CODE_SUBAGENT_MODEL" ]
  {
    assert_eq!( env[ key ], serde_json::json!( "kimi-k3" ), "T17: env.{key} must mirror redirect_model, got:\n{settings_text}" );
  }
  assert_eq!( env[ "CLAUDE_CODE_EFFORT_LEVEL" ], serde_json::json!( "max" ), "T17: env.CLAUDE_CODE_EFFORT_LEVEL mismatch, got:\n{settings_text}" );
  assert_eq!(
    env[ "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ], serde_json::json!( "1048576" ),
    "T17: kimi-k3 must get the 1M auto-compact window, got:\n{settings_text}",
  );
}

#[ test ]
fn t18_save_preset_kimi_with_explicit_backend_anthropic_does_not_force_redirect_fields()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".account.save", "name::alice@acme.com", "preset::kimi", "backend::anthropic" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let meta = read_account_meta( dir.path(), "alice@acme.com" );
  assert_eq!( meta[ "backend" ], serde_json::json!( "anthropic" ), "T18: explicit backend::anthropic must win over preset::kimi, got:\n{meta}" );
  assert!( meta.get( "base_url" ).is_none(), "T18: preset::kimi must not force base_url onto an anthropic-backend save, got:\n{meta}" );
  assert!( meta.get( "inference_provider" ).is_none(), "T18: preset::kimi must not force inference_provider onto an anthropic-backend save, got:\n{meta}" );
}

#[ test ]
fn t19_use_redirect_removes_stale_top_level_model_pin()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // A stale explicit model pin from a previous (anthropic) session's /model pick.
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), r#"{"model":"claude-sonnet-5"}"# ).unwrap();

  // Save-side gate: the live `model` pin must NOT be snapshotted into redirect meta.
  let save_out = run_cs_with_env(
    &[
      ".account.save", "name::kimi", "backend::redirect",
      "base_url::https://api.moonshot.ai/anthropic", "api_key::sk-test",
      "redirect_model::kimi-k3",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &save_out, 0 );
  let meta = read_account_meta( dir.path(), "kimi" );
  assert!(
    meta.get( "model" ).is_none(),
    "T19: redirect save must not snapshot the live model pin into meta, got:\n{meta}",
  );

  // Switch-side gate: the stale top-level pin is removed — env.ANTHROPIC_MODEL is the
  // only model routing a redirect seat gets; a leftover pin would shadow it in the
  // /model UI and take over whenever the env block is absent.
  let use_out = run_cs_with_env( &[ ".account.use", "name::kimi" ], &[ ( "HOME", home ) ] );
  assert_exit( &use_out, 0 );

  let settings_text = std::fs::read_to_string( claude_dir.join( "settings.json" ) ).unwrap();
  let settings : serde_json::Value = serde_json::from_str( &settings_text ).unwrap();
  assert!(
    settings.get( "model" ).is_none(),
    "T19: redirect switch must remove the stale top-level model pin, got:\n{settings_text}",
  );
  assert_eq!(
    settings[ "env" ][ "ANTHROPIC_MODEL" ], serde_json::json!( "kimi-k3" ),
    "T19: env.ANTHROPIC_MODEL must carry the redirect model, got:\n{settings_text}",
  );
}

#[ test ]
fn t20_save_redirect_self_heals_stray_model_in_meta()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let save_out = run_cs_with_env(
    &[
      ".account.save", "name::kimi", "backend::redirect",
      "base_url::https://api.moonshot.ai/anthropic", "api_key::sk-test",
      "redirect_model::kimi-k3",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &save_out, 0 );

  // A stray `model` field left behind by a save that predates the redirect gate.
  let meta_path = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ).join( "kimi.json" );
  let mut meta : serde_json::Value = serde_json::from_str( &std::fs::read_to_string( &meta_path ).unwrap() ).unwrap();
  meta.as_object_mut().unwrap().insert( "model".to_string(), serde_json::json!( "claude-fable-5[1m]" ) );
  std::fs::write( &meta_path, serde_json::to_string_pretty( &meta ).unwrap() ).unwrap();

  // Re-save removes (not merely skips) the stray field.
  let resave_out = run_cs_with_env(
    &[
      ".account.save", "name::kimi", "backend::redirect",
      "base_url::https://api.moonshot.ai/anthropic", "api_key::sk-test",
      "redirect_model::kimi-k3",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &resave_out, 0 );
  let healed = read_account_meta( dir.path(), "kimi" );
  assert!(
    healed.get( "model" ).is_none(),
    "T20: redirect re-save must remove a stray pre-gate model field, got:\n{healed}",
  );

  // Anthropic control: the live model pin is still snapshotted for anthropic saves.
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), r#"{"model":"claude-sonnet-5"}"# ).unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  let alice_save = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &alice_save, 0 );
  let alice_meta = read_account_meta( dir.path(), "alice@acme.com" );
  assert_eq!(
    alice_meta[ "model" ], serde_json::json!( "claude-sonnet-5" ),
    "T20: anthropic save must still snapshot the live model pin, got:\n{alice_meta}",
  );
}

/// T21 / Feature 071: after switching to a redirect account, the `.usage` TSV row for
/// that account carries the display quartet — ✓ flag (`is_current` by token comparison),
/// `static` status word (⚪ tier, not `err`), and `static` in the expires column.
///
/// End-to-end over the real binary: save → use → usage. No HTTP: the redirect row is
/// produced by the R1 bypass, `should_refresh` rejects the placeholder, and touch skips
/// with the redirect reason.
#[ test ]
fn t21_usage_tsv_active_redirect_row_current_static()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let save = run_cs_with_env(
    &[
      ".account.save", "name::kimi", "backend::redirect",
      "base_url::https://api.moonshot.ai/anthropic", "api_key::sk-test",
      "redirect_model::kimi-k3",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &save, 0 );

  let use_out = run_cs_with_env( &[ ".account.use", "name::kimi" ], &[ ( "HOME", home ) ] );
  assert_exit( &use_out, 0 );

  let usage = run_cs_with_env( &[ ".usage", "format::tsv" ], &[ ( "HOME", home ) ] );
  assert_exit( &usage, 0 );
  let text = stdout( &usage );

  let mut lines = text.lines();
  let header : Vec< &str > = lines.next().expect( "T21: TSV must have a header line" ).split( '\t' ).collect();
  let status_idx  = header.iter().position( |h| *h == "status" ).expect( "T21: status column missing" );
  let expires_idx = header.iter().position( |h| *h == "expires" ).expect( "T21: expires column missing" );
  let account_idx = header.iter().position( |h| *h == "account" ).expect( "T21: account column missing" );

  let row : Vec< &str > = lines
    .map( |l| l.split( '\t' ).collect::< Vec< _ > >() )
    .find( |cells| cells.get( account_idx ).is_some_and( |name| name.starts_with( "kimi" ) ) )
    .expect( "T21: no TSV row for the kimi account" );

  assert_eq!( row[ 0 ], "\u{2713}", "T21: active redirect row must carry the ✓ current flag, got row: {row:?}" );
  assert_eq!( row[ status_idx ], "static", "T21: status word must be `static`, not err, got row: {row:?}" );
  assert_eq!( row[ expires_idx ], "static", "T21: expires cell must be `static`, not EXPIRED, got row: {row:?}" );

  // no_color::1 must map the ⚪ redirect glyph to `static` — an unmapped glyph would
  // leak raw emoji into output that guarantees to be emoji-free (apply_no_color).
  let plain = run_cs_with_env( &[ ".usage", "no_color::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &plain, 0 );
  let plain_text = stdout( &plain );
  assert!( !plain_text.contains( '\u{26AA}' ), "T21: no_color output must not leak ⚪, got:\n{plain_text}" );
  assert!( plain_text.contains( "static" ), "T21: no_color output must carry the `static` word, got:\n{plain_text}" );
}

/// T22 / BUG-538: the text-table redirect row is compact and truthful — `(redirect)` note,
/// `—` renews — while the TSV surface deliberately keeps the full reason string.
///
/// # Root Cause (BUG-538)
/// Two independent display defects on the same row: (a) the redirect placeholder — a
/// permanent 40-char backend descriptor, not a transient fetch error — rode BUG-220's
/// "reason into last quota column" contract verbatim; with `auto_wrap` disabled it became
/// the widest cell of that column and displaced every column right of it for ALL rows.
/// (b) `renews_label( None, None, _ )` returns `?` (unknown) — but a redirect account has
/// no Anthropic billing org BY DESIGN, so "no renewal" is a known fact, not missing data.
///
/// # Why Not Caught
/// The renderer test corpus predates redirect accounts — no test ever asserted the text
/// table's redirect row shape (T21 covers TSV cells only), and `EXPIRED`/`?` look
/// individually plausible in a table full of genuinely expired OAuth rows.
///
/// # Fix Applied
/// `render.rs`: `Err` arm emits `(redirect)` when `is_redirect_backend()`; the shared
/// `renews_str` gains the same predicate (→ `—`). `render_tsv.rs`: renews predicate only —
/// the TSV reason cell keeps the full descriptor (machine surface, tab-separated, no
/// width coupling).
///
/// # Prevention
/// Every new account state/type needs a named rendering decision per surface
/// (Text/Plain/TSV/JSON) in its feature ACs at introduction time — a state reaching the
/// selection layer but not the render layer is an incomplete feature (see BUG-538).
///
/// # Pitfall
/// Do not "fix" the TSV reason cell to `(redirect)` for symmetry — consumers key on the
/// canonical `REDIRECT_NO_QUOTA_REASON` string there, and TSV has no column-width problem.
/// And do not route the compact note through `shorten_error()` — that function serves
/// genuine fetch errors shared by all rows.
#[ doc = "bug_reproducer(BUG-538)" ]
#[ test ]
fn t22_usage_text_redirect_row_compact_note_no_question_mark()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let save = run_cs_with_env(
    &[
      ".account.save", "name::kimi", "backend::redirect",
      "base_url::https://api.moonshot.ai/anthropic", "api_key::sk-test",
      "redirect_model::kimi-k3",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &save, 0 );

  // Text table (default columns: ~Renews ON, Sub OFF — so `?` has exactly one possible
  // source in this row, the renews cell).
  let usage = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &usage, 0 );
  let text = stdout( &usage );
  let kimi_line = text.lines()
    .find( |l| l.contains( "kimi" ) )
    .expect( "T22: no text-table row for the kimi account" );

  assert!( kimi_line.contains( "(redirect)" ), "T22: row must carry the compact `(redirect)` note, got: {kimi_line}" );
  assert!(
    !kimi_line.contains( "(redirect backend" ),
    "T22: the 40-char descriptor must not reach the text table (it widens every row), got: {kimi_line}",
  );
  assert!( !kimi_line.contains( '?' ), "T22: no `?` cell — redirect has no renewal by design, got: {kimi_line}" );

  // TSV: renews is `—`, but the reason cell deliberately keeps the full canonical
  // descriptor — machine consumers key on it and tabs carry no width coupling.
  let tsv = run_cs_with_env( &[ ".usage", "format::tsv" ], &[ ( "HOME", home ) ] );
  assert_exit( &tsv, 0 );
  let tsv_text = stdout( &tsv );
  let mut tsv_lines = tsv_text.lines();
  let header : Vec< &str > = tsv_lines.next().expect( "T22: TSV must have a header line" ).split( '\t' ).collect();
  let renews_idx  = header.iter().position( |h| *h == "renews" ).expect( "T22: renews column missing" );
  let account_idx = header.iter().position( |h| *h == "account" ).expect( "T22: account column missing" );
  let row : Vec< &str > = tsv_lines
    .map( |l| l.split( '\t' ).collect::< Vec< _ > >() )
    .find( |cells| cells.get( account_idx ).is_some_and( |name| name.starts_with( "kimi" ) ) )
    .expect( "T22: no TSV row for the kimi account" );

  assert_eq!( row[ renews_idx ], "\u{2014}", "T22: TSV renews must be —, not ?, got row: {row:?}" );
  assert!(
    row.iter().any( |c| c.contains( "redirect backend — no Anthropic quota" ) ),
    "T22: TSV must keep the full canonical reason string, got row: {row:?}",
  );
}

/// T23 / BUG-540: the known-absence contract BUG-538 established for `~Renews` also
/// governs the `Sub` column and the `get::` field extractors — a redirect row emits
/// `—` (known absence), never `?` (unknown), on all four surfaces: text `cols::+sub`,
/// TSV `cols::+sub`, `get::sub`, `get::renews`.
///
/// # Root Cause (BUG-540)
/// The known-absence predicate was applied per call site instead of per value:
/// BUG-538's fix patched 2 of the 3 duplicated renews computations (text + TSV tables)
/// but missed `extract_get_field`'s `GetField::Renews` arm — breaking that function's
/// own documented contract ("the same value that would appear in the corresponding
/// cell of the text table") — and no `sub_label` site got the predicate at all, so a
/// redirect row's `account: None` fell through to `sub_label`'s `?` fallback on every
/// surface. `Sub` is hidden by default, which is why no redirect test ever saw it.
///
/// # Why Not Caught
/// T22's text-table `?` sweep runs under default columns (Sub OFF) and its own comment
/// scoped `?` to "exactly one possible source... the renews cell"; the `get::` surface
/// had no redirect coverage at all.
///
/// # Fix Applied
/// `format.rs`: aq-aware `sub_cell_for()` / `renews_cell_for()` helpers (the
/// `expires_cell_for` pattern from BUG-345) own the predicate once; all six call
/// sites (`render.rs` table + extractor, `render_tsv.rs`) delegate to them.
///
/// # Prevention
/// A cell whose value depends on account state must be computed by exactly one
/// aq-aware helper — per-call-site predicates guarantee the next surface (or the
/// next state) misses one site. Same lesson as BUG-345's `expires_cell_for`.
///
/// # Pitfall
/// Do not push the redirect check into `sub_label` itself — it takes
/// `Option< &OauthAccountData >`, and `None` there legitimately means "fetch failed,
/// genuinely unknown" for anthropic rows, where `?` is the truthful output.
#[ doc = "bug_reproducer(BUG-540)" ]
#[ test ]
fn t23_usage_sub_and_get_fields_redirect_known_absence()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let save = run_cs_with_env(
    &[
      ".account.save", "name::kimi", "backend::redirect",
      "base_url::https://api.moonshot.ai/anthropic", "api_key::sk-test",
      "redirect_model::kimi-k3",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &save, 0 );

  // get:: extractors — kimi is the only account, so it is `accounts.first()`.
  let get_sub = run_cs_with_env( &[ ".usage", "get::sub" ], &[ ( "HOME", home ) ] );
  assert_exit( &get_sub, 0 );
  assert_eq!(
    stdout( &get_sub ).trim(), "\u{2014}",
    "T23: get::sub on a redirect account must be — (no Anthropic subscription by design), not ?",
  );

  let get_renews = run_cs_with_env( &[ ".usage", "get::renews" ], &[ ( "HOME", home ) ] );
  assert_exit( &get_renews, 0 );
  assert_eq!(
    stdout( &get_renews ).trim(), "\u{2014}",
    "T23: get::renews must match the table cell (—) — extract_get_field's same-as-table contract",
  );

  // Text table with the Sub column enabled: the row stays ?-free.
  let usage = run_cs_with_env( &[ ".usage", "cols::+sub" ], &[ ( "HOME", home ) ] );
  assert_exit( &usage, 0 );
  let text = stdout( &usage );
  let kimi_line = text.lines()
    .find( |l| l.contains( "kimi" ) )
    .expect( "T23: no text-table row for the kimi account" );
  assert!(
    !kimi_line.contains( '?' ),
    "T23: cols::+sub must not surface a `?` on the redirect row (sub is a known absence), got: {kimi_line}",
  );

  // TSV with the Sub column enabled: the sub cell itself is `—`.
  let tsv = run_cs_with_env( &[ ".usage", "format::tsv", "cols::+sub" ], &[ ( "HOME", home ) ] );
  assert_exit( &tsv, 0 );
  let tsv_text = stdout( &tsv );
  let mut tsv_lines = tsv_text.lines();
  let header : Vec< &str > = tsv_lines.next().expect( "T23: TSV must have a header line" ).split( '\t' ).collect();
  let sub_idx     = header.iter().position( |h| *h == "sub" ).expect( "T23: sub column missing" );
  let account_idx = header.iter().position( |h| *h == "account" ).expect( "T23: account column missing" );
  let row : Vec< &str > = tsv_lines
    .map( |l| l.split( '\t' ).collect::< Vec< _ > >() )
    .find( |cells| cells.get( account_idx ).is_some_and( |name| name.starts_with( "kimi" ) ) )
    .expect( "T23: no TSV row for the kimi account" );
  assert_eq!( row[ sub_idx ], "\u{2014}", "T23: TSV sub cell must be —, not ?, got row: {row:?}" );
}
