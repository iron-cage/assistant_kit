//! Integration tests: redirect-backend accounts (Feature 071) — `.account.save` CLI layer.
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! | ID  | Test Function                                             | Condition                                          | P/N |
//! |-----|------------------------------------------------------------|-----------------------------------------------------|-----|
//! | T01 | `t01_save_redirect_full_succeeds`                          | full redirect save → `kimi.json` + `.credentials.json` | P   |
//! | T03 | `t03_save_base_url_outside_redirect_exits_1`               | `base_url::` without `backend::redirect` → exit 1  | N   |
//! | T04 | `t04_save_no_backend_unchanged_from_pre071`                | no `backend::` at all → pre-071 behavior preserved | P   |
//! | T13 | `t13_save_resave_different_backend_rewrites_from_scratch`  | re-save redirect→anthropic → stale fields cleared  | P   |

use crate::cli_runner::{
  run_cs_with_env,
  assert_exit,
  write_credentials, credential_json, account_exists, read_account_meta,
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
      "redirect_model::kimi-k3-0905-preview",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let meta = read_account_meta( dir.path(), "kimi" );
  assert_eq!( meta[ "backend" ], serde_json::json!( "redirect" ), "T01: backend must be redirect, got:\n{meta}" );
  assert_eq!( meta[ "base_url" ], serde_json::json!( "https://api.moonshot.ai/anthropic" ), "T01: base_url mismatch, got:\n{meta}" );
  assert_eq!( meta[ "redirect_model" ], serde_json::json!( "kimi-k3-0905-preview" ), "T01: redirect_model mismatch, got:\n{meta}" );

  let creds_text = std::fs::read_to_string( credentials_path( dir.path(), "kimi" ) ).unwrap();
  let creds : serde_json::Value = serde_json::from_str( &creds_text ).unwrap();
  assert_eq!( creds[ "accessToken" ], serde_json::json!( "sk-test" ), "T01: accessToken mismatch, got:\n{creds_text}" );
  assert_eq!( creds.as_object().unwrap().len(), 1, "T01: redirect credentials must contain only accessToken, got:\n{creds_text}" );
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
fn t13_save_resave_different_backend_rewrites_from_scratch()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // First save: kimi as a redirect account.
  let out1 = run_cs_with_env(
    &[
      ".account.save", "name::kimi", "backend::redirect",
      "base_url::https://api.moonshot.ai/anthropic", "api_key::sk-test",
      "redirect_model::kimi-k3-0905-preview",
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
