//! Backend tests: redirect/anthropic backend (Feature 071), inference provider
//! (Feature 072), Kimi tier env vars (Feature 073), `DeepSeek` tier env vars
//! (Feature 078), and flat-JSON field parsing (BUG-002, FT-08/021).
//!
//! ## Fix Documentation — BUG-002
//!
//! - **Root Cause:** `parse_string_field()`/`parse_u64_field()`/`parse_bool_field()`/
//!   `parse_string_array_field()` all open with an unbounded `json.find(&search)` over the
//!   ENTIRE input string — none accepts or enforces "search only within this one object."
//!   A caller holding multi-entry JSON (e.g. `roles_json`, a list of workspace/organization
//!   memberships) has no way to scope the search to the entry it actually needs, and
//!   silently gets whichever entry's field is textually first.
//! - **Why Not Caught:** No test exercised any of the four helpers against multi-entry JSON —
//!   every existing fixture is a flat, single-object JSON blob (credentials files,
//!   settings.json), where "first occurrence" is always correct by coincidence of there
//!   being nothing else to find.
//! - **Fix Applied:** Added `extract_object_block()` — a brace-depth-counted `{...}` bound
//!   (mirrors `claude_quota`'s own helper of the same name; independently duplicated, not
//!   shared). A caller walking a multi-entry array can now bound each entry with
//!   `extract_object_block()` before calling `parse_string_field()` etc. on the bounded
//!   slice, eliminating the wrong-entry ambiguity for any caller that adopts it. The 4
//!   existing unbounded helpers are unchanged — still correct for flat single-object JSON.
//! - **Prevention:** `bug002_extract_object_block_bounds_multi_entry_roles_json` reproduces
//!   the exact MRE scenario from BUG-002 (`roles_json` with two workspace memberships) and
//!   asserts the second entry's `workspace_name` is correctly extracted once bounded.
//! - **Pitfall:** Do not add object-boundary scanning inside the 4 existing helpers directly
//!   — that would need a scoping parameter and break every existing single-object call site
//!   across the crate. Bounding is the caller's responsibility via `extract_object_block()`.
//!
//! ## Test Matrix
//!
//! | Test | Scenario |
//! |------|----------|
//! | `ft01_071_backend_redirect_parses_to_redirect_variant` | Feature 071/T01: `"backend":"redirect"` + base_url/redirect_model round-trip via list() |
//! | `ft02_071_absent_backend_key_defaults_to_anthropic` | Feature 071/AC-05: no `backend` key → `AccountBackend::Anthropic`, base_url/redirect_model None |
//! | `ft03_071_unrecognized_backend_value_defaults_to_anthropic_not_error` | Feature 071/AC-05: `"backend":"bogus"` → `AccountBackend::Anthropic`, not an error |
//! | `ft04_071_save_redirect_writes_minimal_credentials_and_metadata` | Feature 071/T01/AC-01: redirect save → {name}.credentials.json has only accessToken; {name}.json has backend/base_url/redirect_model |
//! | `ft05_071_save_redirect_never_touches_live_credentials_file` | Feature 071/T01/AC-01: redirect save never reads/writes ~/.claude/.credentials.json |
//! | `ft06_071_save_default_anthropic_writes_backend_field` | Feature 071/T02/AC-04: default (anthropic) save preserves live-file copy; writes backend:"anthropic" into {name}.json
//! | `ft07_071_switch_to_redirect_writes_env_keys` | Feature 071/T03/AC-06: switch_account() to a redirect account writes env.ANTHROPIC_BASE_URL/AUTH_TOKEN/MODEL; unrelated fields survive |
//! | `ft08_071_switch_to_anthropic_removes_env_keys_and_prunes_empty_env` | Feature 071/T03/AC-07: switch_account() to an anthropic account removes the 3 env keys and prunes `env` when empty |
//! | `ft09_071_switch_to_anthropic_preserves_unrelated_env_subkey` | Feature 071/T03/AC-07: an unrelated pre-existing env.* sub-key survives a switch-away from redirect |
//! | `ft10_071_read_backend_missing_file_defaults_anthropic` | Feature 071/T14: read_backend() on missing {name}.json defaults to Anthropic |
//! | `ft11_071_read_backend_redirect_value` | Feature 071/T14: read_backend() reads an explicit "backend":"redirect" field |
//! | `ft12_071_read_backend_corrupt_content_defaults_anthropic` | Feature 071/T14: read_backend() on corrupt content defaults to Anthropic, no panic |
//! | `ft01_072_save_some_inference_provider_writes_field` | Feature 072/T01/AC-01: save(inference_provider: Some("kimi")) on fresh account writes inference_provider:"kimi"
//! | `ft02_072_save_none_inference_provider_preserves_existing` | Feature 072/T02/AC-02: save(inference_provider: None) preserves existing inference_provider unchanged
//! | `ft03_072_save_none_inference_provider_no_prior_key_writes_no_key` | Feature 072/T03/AC-03/AF3: save(inference_provider: None) with no prior key writes no key at all (never "anthropic")
//! | `ft04_072_list_reads_inference_provider_when_present` | Feature 072/T04/AC-04: list() reads inference_provider from {name}.json when present
//! | `ft05_072_list_defaults_inference_provider_to_empty_when_absent` | Feature 072/T05/AC-05: list() defaults Account.inference_provider to "" when key absent |
//! | `ft01_073_switch_to_kimi_redirect_writes_all_tier_env_vars` | Feature 073/AC-05: switch_account() to a redirect+kimi account writes all 5 default-model vars, CLAUDE_CODE_EFFORT_LEVEL, and a 1M CLAUDE_CODE_AUTO_COMPACT_WINDOW for a kimi-k3 model |
//! | `ft02_073_switch_to_kimi_redirect_uses_narrow_compact_window_for_non_k3_model` | Feature 073/AC-06: kimi-k2.7-code redirect_model → CLAUDE_CODE_AUTO_COMPACT_WINDOW is the 256K value, not the 1M default |
//! | `ft03_073_switch_to_redirect_non_kimi_provider_omits_tier_env_vars` | Feature 073/AC-08: a redirect account not tagged inference_provider:"kimi" gets only the original 3 env vars, none of the 7 Kimi-tier additions |
//! | `ft04_073_switch_from_kimi_to_anthropic_clears_all_tier_env_vars` | Feature 073/AC-07: switching from a kimi redirect account to an anthropic account removes all 10 env vars, not just the original 3 |
//! | `ft05_073_switch_from_kimi_to_other_redirect_clears_stale_tier_env_vars` | Feature 073/AC-07: switching from a kimi redirect account to a different, non-kimi redirect account also clears the 7 stale Kimi-tier vars |
//! | `ft06_078_switch_to_deepseek_redirect_writes_tier_env_vars` | Feature 078/AC-01: switch_account() to a redirect+deepseek account writes the 2 Pro vars (mirror redirect_model), 2 Flash vars (fixed deepseek-v4-flash), CLAUDE_CODE_EFFORT_LEVEL, and the flat 786432 CLAUDE_CODE_AUTO_COMPACT_WINDOW |
//! | `ft07_078_switch_to_deepseek_redirect_uses_flat_compact_window_regardless_of_model` | Feature 078/AC-02: CLAUDE_CODE_AUTO_COMPACT_WINDOW stays 786432 regardless of redirect_model — no Kimi-style k3/non-k3 branch |
//! | `ft08_078_switch_to_redirect_non_deepseek_provider_omits_tier_env_vars` | Feature 078/AC-03: a redirect account not tagged inference_provider:"deepseek" gets only the original 3 env vars, none of the 6 DeepSeek-tier additions |
//! | `ft09_078_switch_from_deepseek_to_anthropic_clears_tier_env_vars` | Feature 078/AC-04: switching from a deepseek redirect account to an anthropic account removes all 9 env vars, not just the original 3 |
//! | `ft10_078_switch_from_deepseek_to_other_redirect_clears_stale_tier_env_vars` | Feature 078/AC-05: switching from a deepseek redirect account to a different, non-deepseek redirect account also clears the 6 stale DeepSeek-tier vars |
//! | `ft11_078_switch_from_kimi_to_deepseek_clears_kimi_writes_deepseek` | Feature 078/AC-11: switching directly from a kimi-tagged to a deepseek-tagged redirect account clears the 7 stale Kimi-tier vars and writes the 6 DeepSeek-tier vars in the same call |
//! | `ft12_078_switch_from_deepseek_to_kimi_clears_deepseek_writes_kimi` | Feature 078/AC-11: mirror direction — switching from a deepseek-tagged to a kimi-tagged redirect account clears the 6 stale DeepSeek-tier vars and writes the 7 Kimi-tier vars |
//! | `bug002_extract_object_block_bounds_multi_entry_roles_json` | BUG-002: extract_object_block() bounds parse_string_field() to one membership entry in multi-entry roles_json |

use tempfile::TempDir;
use claude_profile_core::account;
use claude_core::ClaudePaths;

mod account_fixture;
use account_fixture::*;

// ── Feature 071 — AccountBackend domain type (Phase 1, task 433) ───────────────

/// T01/433: a fixture with `"backend":"redirect"` parses to `AccountBackend::Redirect`;
/// `base_url`/`redirect_model` round-trip through `list()`.
#[ test ]
fn ft01_071_backend_redirect_parses_to_redirect_variant()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();
  let name  = "redirect@test.com";

  write_credentials_file( store, name );
  std::fs::write(
    store.join( format!( "{name}.json" ) ),
    r#"{"backend":"redirect","base_url":"https://foreign.example.com","redirect_model":"foreign-model-1"}"#,
  ).unwrap();

  let accounts = account::list( store ).unwrap();
  let acct = accounts.iter().find( | a | a.name == name ).expect( "account must be listed" );
  assert_eq!(
    acct.backend, account::AccountBackend::Redirect,
    "backend:\"redirect\" must parse to AccountBackend::Redirect",
  );
  assert_eq!(
    acct.base_url.as_deref(), Some( "https://foreign.example.com" ),
    "base_url must round-trip through list()",
  );
  assert_eq!(
    acct.redirect_model.as_deref(), Some( "foreign-model-1" ),
    "redirect_model must round-trip through list()",
  );
}

/// Feature 071/AC-05 (domain-layer half): a legacy fixture with no `backend` key
/// parses to `AccountBackend::Anthropic` — byte-for-byte unchanged classification
/// for every account saved before Feature 071.
#[ test ]
fn ft02_071_absent_backend_key_defaults_to_anthropic()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();
  let name  = "legacy@test.com";

  write_credentials_file( store, name );
  std::fs::write(
    store.join( format!( "{name}.json" ) ),
    r#"{"emailAddress":"legacy@test.com"}"#,
  ).unwrap();

  let accounts = account::list( store ).unwrap();
  let acct = accounts.iter().find( | a | a.name == name ).expect( "account must be listed" );
  assert_eq!(
    acct.backend, account::AccountBackend::Anthropic,
    "absent backend key must default to AccountBackend::Anthropic",
  );
  assert!( acct.base_url.is_none(), "base_url must be None when absent from JSON" );
  assert!( acct.redirect_model.is_none(), "redirect_model must be None when absent from JSON" );
}

/// Feature 071/AC-05: an unrecognized `backend` value neither errors nor
/// misclassifies — it defaults to `AccountBackend::Anthropic`, same as absent.
#[ test ]
fn ft03_071_unrecognized_backend_value_defaults_to_anthropic_not_error()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();
  let name  = "bogus@test.com";

  write_credentials_file( store, name );
  std::fs::write(
    store.join( format!( "{name}.json" ) ),
    r#"{"backend":"bogus"}"#,
  ).unwrap();

  let accounts = account::list( store ).unwrap();
  let acct = accounts.iter().find( | a | a.name == name )
    .expect( "account must be listed — list() must not error on unrecognized backend value" );
  assert_eq!(
    acct.backend, account::AccountBackend::Anthropic,
    "unrecognized backend value must default to AccountBackend::Anthropic, never error",
  );
}

// ── Feature 071 — save()'s redirect write path (Phase 2, task 433) ────────────

/// T01/433/AC-01: `save()` with `backend: Redirect` writes `{name}.credentials.json`
/// containing exactly one key (`accessToken`, from the caller-supplied API key) and
/// writes `backend`/`base_url`/`redirect_model` into `{name}.json`.
#[ test ]
fn ft04_071_save_redirect_writes_minimal_credentials_and_metadata()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path().join( "store" );
  std::fs::create_dir_all( &store ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::save(
    "redirect@foreign.com", &store, &paths, false,
    Some( b"sk-foreign-key-abc123" ), None, None, None,
    account::AccountBackend::Redirect, Some( "https://foreign.example.com" ), Some( "foreign-model-x" ), None, None,
  ).unwrap();

  let creds_content = std::fs::read_to_string( store.join( "redirect@foreign.com.credentials.json" ) )
    .expect( "{name}.credentials.json must exist after redirect save" );
  let creds_json : serde_json::Value = serde_json::from_str( &creds_content )
    .expect( "{name}.credentials.json must be valid JSON" );
  let creds_obj = creds_json.as_object().expect( "{name}.credentials.json must be a JSON object" );
  assert_eq!(
    creds_obj.len(), 1,
    "redirect save's {{name}}.credentials.json must contain exactly 1 key (accessToken); got: {creds_content}",
  );
  assert_eq!(
    creds_obj.get( "accessToken" ).and_then( | v | v.as_str() ), Some( "sk-foreign-key-abc123" ),
    "redirect save must write the caller-supplied API key as accessToken",
  );

  let meta_content = std::fs::read_to_string( store.join( "redirect@foreign.com.json" ) )
    .expect( "{name}.json must exist after redirect save" );
  let meta_json : serde_json::Value = serde_json::from_str( &meta_content ).expect( "{name}.json must be valid JSON" );
  assert_eq!( meta_json[ "backend" ].as_str(), Some( "redirect" ), "redirect save must write backend:\"redirect\"; got: {meta_content}" );
  assert_eq!(
    meta_json[ "base_url" ].as_str(), Some( "https://foreign.example.com" ),
    "redirect save must write base_url to {{name}}.json; got: {meta_content}",
  );
  assert_eq!(
    meta_json[ "redirect_model" ].as_str(), Some( "foreign-model-x" ),
    "redirect save must write redirect_model to {{name}}.json; got: {meta_content}",
  );
}

/// T01/433/AC-01: a redirect save never reads `~/.claude/.credentials.json` — the
/// live Anthropic OAuth session file is left completely untouched.
#[ test ]
fn ft05_071_save_redirect_never_touches_live_credentials_file()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  // Distinct fixture value — a redirect save must never copy from or overwrite this.
  let live_marker = r#"{"accessToken":"LIVE-SESSION-SENTINEL-DO-NOT-TOUCH","expiresAt":1}"#;
  std::fs::write( dot_claude.join( ".credentials.json" ), live_marker ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::save(
    "redirect@foreign.com", &store, &paths, false,
    Some( b"sk-foreign-key-abc123" ), None, None, None,
    account::AccountBackend::Redirect, Some( "https://foreign.example.com" ), Some( "foreign-model-x" ), None, None,
  ).unwrap();

  let live_content = std::fs::read_to_string( dot_claude.join( ".credentials.json" ) )
    .expect( "live ~/.claude/.credentials.json must still exist" );
  assert_eq!(
    live_content, live_marker,
    "redirect save must never modify ~/.claude/.credentials.json",
  );
}

/// T02/433/AC-04 (`docs/feature/071_redirect_backend_accounts.md`): `save()` with no
/// explicit backend argument (i.e. `AccountBackend::Anthropic`) still copies
/// `~/.claude/.credentials.json` exactly as before Feature 071, but now additionally
/// writes `backend: "anthropic"` into `{name}.json` — this is an intentional Feature 071
/// behavior addition (every account file becomes self-describing), not a regression.
#[ test ]
fn ft06_071_save_default_anthropic_writes_backend_field()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();
  std::fs::write( dot_claude.join( ".credentials.json" ), r#"{"accessToken":"tok","expiresAt":9999999999999}"# ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::save(
    "alice@test.com", &store, &paths, false, None, None, None, None,
    account::AccountBackend::Anthropic, None, None, None, None,
  ).unwrap();

  let creds_content = std::fs::read_to_string( store.join( "alice@test.com.credentials.json" ) )
    .expect( "{name}.credentials.json must exist after anthropic save" );
  assert!(
    creds_content.contains( "\"accessToken\"" ) && creds_content.contains( "\"expiresAt\"" ),
    "anthropic save must still copy the full live credentials file unchanged; got: {creds_content}",
  );

  let meta_content = std::fs::read_to_string( store.join( "alice@test.com.json" ) )
    .expect( "{name}.json must exist after anthropic save (Feature 071: backend is always written)" );
  let meta_json : serde_json::Value = serde_json::from_str( &meta_content ).expect( "{name}.json must be valid JSON" );
  assert_eq!(
    meta_json[ "backend" ].as_str(), Some( "anthropic" ),
    "AC-04: default/anthropic save must write backend:\"anthropic\" into {{name}}.json; got: {meta_content}",
  );
}

// ── Feature 071 — switch_account()'s env.* responsibility (Phase 3, task 433) ─

/// T03/433/AC-06: `switch_account()` to a `backend: redirect` account writes all three
/// `env.ANTHROPIC_*` keys into `settings.json`, matching the target account's stored
/// `base_url`/`accessToken`/`redirect_model` values; unrelated top-level fields survive.
#[ test ]
fn ft07_071_switch_to_redirect_writes_env_keys()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "kimi@moonshot.ai.credentials.json" ),
    r#"{"accessToken":"sk-foreign-key-abc123"}"#,
  ).unwrap();
  std::fs::write(
    store.join( "kimi@moonshot.ai.json" ),
    r#"{"backend":"redirect","base_url":"https://api.moonshot.ai/anthropic","redirect_model":"kimi-k3"}"#,
  ).unwrap();
  // Pre-existing unrelated top-level field — must survive the switch untouched (AC-06).
  std::fs::write( dot_claude.join( "settings.json" ), r#"{"theme":"dark"}"# ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "kimi@moonshot.ai", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) )
    .expect( "~/.claude/settings.json must exist after switch_account" );
  let live_json : serde_json::Value = serde_json::from_str( &live ).expect( "settings.json must be valid JSON" );
  assert_eq!(
    live_json[ "env" ][ "ANTHROPIC_BASE_URL" ].as_str(), Some( "https://api.moonshot.ai/anthropic" ),
    "AC-06: switch to redirect must write env.ANTHROPIC_BASE_URL from the account's base_url; got: {live}",
  );
  assert_eq!(
    live_json[ "env" ][ "ANTHROPIC_AUTH_TOKEN" ].as_str(), Some( "sk-foreign-key-abc123" ),
    "AC-06: switch to redirect must write env.ANTHROPIC_AUTH_TOKEN from the account's accessToken; got: {live}",
  );
  assert_eq!(
    live_json[ "env" ][ "ANTHROPIC_MODEL" ].as_str(), Some( "kimi-k3" ),
    "AC-06: switch to redirect must write env.ANTHROPIC_MODEL from the account's redirect_model; got: {live}",
  );
  assert_eq!(
    live_json[ "theme" ].as_str(), Some( "dark" ),
    "AC-06: unrelated top-level settings.json fields must survive the switch; got: {live}",
  );
}

/// T03/433/AC-07: `switch_account()` to a `backend: anthropic` account, after a prior
/// redirect switch populated `env`, removes exactly the 3 `ANTHROPIC_*` keys and prunes
/// the whole `env` object once it becomes empty as a result.
#[ test ]
fn ft08_071_switch_to_anthropic_removes_env_keys_and_prunes_empty_env()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "alice@test.com.credentials.json" ),
    r#"{"accessToken":"tok","expiresAt":9999999999999}"#,
  ).unwrap();
  // No {name}.json — an absent backend key defaults to Anthropic (AC-05).
  // Live settings.json already has env populated by a prior switch-to-redirect.
  std::fs::write(
    dot_claude.join( "settings.json" ),
    r#"{"env":{"ANTHROPIC_BASE_URL":"https://api.moonshot.ai/anthropic","ANTHROPIC_AUTH_TOKEN":"sk-foreign","ANTHROPIC_MODEL":"kimi-k3"},"theme":"dark"}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "alice@test.com", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) )
    .expect( "~/.claude/settings.json must exist after switch_account" );
  let live_json : serde_json::Value = serde_json::from_str( &live ).expect( "settings.json must be valid JSON" );
  assert!(
    live_json.get( "env" ).is_none(),
    "AC-07: env must be removed entirely once its last ANTHROPIC_* sub-key is cleared; got: {live}",
  );
  assert_eq!(
    live_json[ "theme" ].as_str(), Some( "dark" ),
    "AC-07: unrelated top-level settings.json fields must survive the switch; got: {live}",
  );
}

/// T03/433/AC-07: switching to an anthropic account preserves any unrelated `env.*`
/// sub-key that was already present — only the 3 `ANTHROPIC_*` keys are removed.
#[ test ]
fn ft09_071_switch_to_anthropic_preserves_unrelated_env_subkey()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "alice@test.com.credentials.json" ),
    r#"{"accessToken":"tok","expiresAt":9999999999999}"#,
  ).unwrap();
  std::fs::write(
    dot_claude.join( "settings.json" ),
    r#"{"env":{"ANTHROPIC_BASE_URL":"https://api.moonshot.ai/anthropic","TZ":"Europe/Kyiv"}}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "alice@test.com", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) )
    .expect( "~/.claude/settings.json must exist after switch_account" );
  let live_json : serde_json::Value = serde_json::from_str( &live ).expect( "settings.json must be valid JSON" );
  assert_eq!(
    live_json[ "env" ][ "TZ" ].as_str(), Some( "Europe/Kyiv" ),
    "AC-07: an unrelated env.* sub-key must survive a switch-away from redirect; got: {live}",
  );
  assert!(
    live_json[ "env" ].get( "ANTHROPIC_BASE_URL" ).is_none(),
    "AC-07: ANTHROPIC_BASE_URL must be removed on switch to anthropic; got: {live}",
  );
}

// ── Feature 071 — read_backend() helper (Phase 5, task 434) ────────────────────

/// T14/434: `read_backend()` on a missing `{name}.json` defaults to `Anthropic`,
/// mirroring `read_owner()`'s missing-file default-on-failure behaviour.
#[ test ]
fn ft10_071_read_backend_missing_file_defaults_anthropic()
{
  let tmp = TempDir::new().unwrap();
  let backend = account::read_backend( tmp.path(), "nonexistent@test.com" );
  assert_eq!(
    backend, account::AccountBackend::Anthropic,
    "read_backend on missing file must default to Anthropic; got: {backend:?}",
  );
}

/// T14/434: `read_backend()` reads an explicit `"backend":"redirect"` field correctly.
#[ test ]
fn ft11_071_read_backend_redirect_value()
{
  let tmp = TempDir::new().unwrap();
  std::fs::write( tmp.path().join( "kimi@moonshot.ai.json" ), r#"{"backend":"redirect"}"# ).unwrap();
  let backend = account::read_backend( tmp.path(), "kimi@moonshot.ai" );
  assert_eq!(
    backend, account::AccountBackend::Redirect,
    "read_backend must read an explicit redirect value; got: {backend:?}",
  );
}

/// T14/434: `read_backend()` on corrupt (non-JSON) content defaults to `Anthropic` —
/// must not panic, same resilience contract as `read_owner()`'s CC-3 case.
#[ test ]
fn ft12_071_read_backend_corrupt_content_defaults_anthropic()
{
  let tmp = TempDir::new().unwrap();
  std::fs::write( tmp.path().join( "alice@test.com.json" ), "<<<not json at all>>>" ).unwrap();
  let backend = account::read_backend( tmp.path(), "alice@test.com" );
  assert_eq!(
    backend, account::AccountBackend::Anthropic,
    "read_backend on corrupt content must default to Anthropic; got: {backend:?}",
  );
}

// ── Feature 072 — inference_provider field (task 435) ──────────────────────────

/// T01/435/AC-01: `save()` with `inference_provider: Some("kimi")` on a fresh account
/// writes `"inference_provider": "kimi"` to `{name}.json`.
#[ test ]
fn ft01_072_save_some_inference_provider_writes_field()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path().join( "store" );
  std::fs::create_dir_all( &store ).unwrap();

  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();
  std::fs::write( dot_claude.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::save(
    "alice@test.com", &store, &paths, false, None, None, None, None,
    account::AccountBackend::Anthropic, None, None, Some( "kimi" ), None,
  ).unwrap();

  let meta_content = std::fs::read_to_string( store.join( "alice@test.com.json" ) )
    .expect( "{name}.json must exist after save" );
  let meta_json : serde_json::Value = serde_json::from_str( &meta_content ).expect( "{name}.json must be valid JSON" );
  assert_eq!(
    meta_json[ "inference_provider" ].as_str(), Some( "kimi" ),
    "AC-01: save(inference_provider: Some(\"kimi\")) must write inference_provider:\"kimi\"; got: {meta_content}",
  );
}

/// T02/435/AC-02: `save()` with `inference_provider: None` on an account whose
/// `{name}.json` already has `"inference_provider": "kimi"` preserves it unchanged.
#[ test ]
fn ft02_072_save_none_inference_provider_preserves_existing()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path().join( "store" );
  std::fs::create_dir_all( &store ).unwrap();

  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();
  std::fs::write( dot_claude.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::save(
    "alice@test.com", &store, &paths, false, None, None, None, None,
    account::AccountBackend::Anthropic, None, None, Some( "kimi" ), None,
  ).unwrap();

  // Second save with inference_provider: None — must not clobber the existing value.
  account::save(
    "alice@test.com", &store, &paths, false, None, None, None, None,
    account::AccountBackend::Anthropic, None, None, None, None,
  ).unwrap();

  let meta_content = std::fs::read_to_string( store.join( "alice@test.com.json" ) )
    .expect( "{name}.json must exist after save" );
  let meta_json : serde_json::Value = serde_json::from_str( &meta_content ).expect( "{name}.json must be valid JSON" );
  assert_eq!(
    meta_json[ "inference_provider" ].as_str(), Some( "kimi" ),
    "AC-02: save(inference_provider: None) must preserve existing inference_provider unchanged; got: {meta_content}",
  );
}

/// T03/435/AC-03/AF3: `save()` with `inference_provider: None` on an account with no
/// pre-existing `inference_provider` key writes no such key at all — never the literal
/// default `"anthropic"`. Checks literal key absence (`contains_key`), not merely an
/// empty-string read, per AF3.
#[ test ]
fn ft03_072_save_none_inference_provider_no_prior_key_writes_no_key()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path().join( "store" );
  std::fs::create_dir_all( &store ).unwrap();

  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();
  std::fs::write( dot_claude.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::save(
    "alice@test.com", &store, &paths, false, None, None, None, None,
    account::AccountBackend::Anthropic, None, None, None, None,
  ).unwrap();

  let meta_content = std::fs::read_to_string( store.join( "alice@test.com.json" ) )
    .expect( "{name}.json must exist after save" );
  let meta_json : serde_json::Value = serde_json::from_str( &meta_content ).expect( "{name}.json must be valid JSON" );
  let obj = meta_json.as_object().expect( "{name}.json must be a JSON object" );
  assert!(
    !obj.contains_key( "inference_provider" ),
    "AC-03/AF3: save(inference_provider: None) with no prior key must write no inference_provider key at all (never \"anthropic\"); got: {meta_content}",
  );
}

/// T04/435/AC-04: `list()` reads `inference_provider` from `{name}.json` into
/// `Account.inference_provider` when the key is present.
#[ test ]
fn ft04_072_list_reads_inference_provider_when_present()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();
  let name  = "moonshot@test.com";

  write_credentials_file( store, name );
  std::fs::write(
    store.join( format!( "{name}.json" ) ),
    r#"{"inference_provider":"moonshot"}"#,
  ).unwrap();

  let accounts = account::list( store ).unwrap();
  let acct = accounts.iter().find( | a | a.name == name ).expect( "account must be listed" );
  assert_eq!(
    acct.inference_provider, "moonshot",
    "AC-04: list() must read inference_provider from {{name}}.json into Account.inference_provider",
  );
}

/// T05/435/AC-05: `list()` returns `Account.inference_provider == ""` when the key is
/// absent from `{name}.json` (pre-existing account, or one saved before this feature).
#[ test ]
fn ft05_072_list_defaults_inference_provider_to_empty_when_absent()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();
  let name  = "legacy_provider@test.com";

  write_credentials_file( store, name );
  std::fs::write(
    store.join( format!( "{name}.json" ) ),
    r#"{"emailAddress":"legacy_provider@test.com"}"#,
  ).unwrap();

  let accounts = account::list( store ).unwrap();
  let acct = accounts.iter().find( | a | a.name == name ).expect( "account must be listed" );
  assert_eq!(
    acct.inference_provider, "",
    "AC-05: list() must default Account.inference_provider to empty string when key absent; got: {:?}", acct.inference_provider,
  );
}

// ── Feature 073 — Kimi provider preset env vars ────────────────────────────────

/// AC-05: `switch_account()` to a `backend: redirect`, `inference_provider: "kimi"`
/// account writes the 5 default-model-tier vars + `CLAUDE_CODE_EFFORT_LEVEL` +
/// `CLAUDE_CODE_AUTO_COMPACT_WINDOW` (1M for a `kimi-k3*` model), alongside the
/// pre-existing 3 `ANTHROPIC_*` vars.
#[ test ]
fn ft01_073_switch_to_kimi_redirect_writes_all_tier_env_vars()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "kimi.credentials.json" ),
    r#"{"accessToken":"sk-kimi-abc123"}"#,
  ).unwrap();
  std::fs::write(
    store.join( "kimi.json" ),
    r#"{"backend":"redirect","base_url":"https://api.moonshot.ai/anthropic","redirect_model":"kimi-k3","inference_provider":"kimi"}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "kimi", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) )
    .expect( "~/.claude/settings.json must exist after switch_account" );
  let live_json : serde_json::Value = serde_json::from_str( &live ).expect( "settings.json must be valid JSON" );
  for key in [ "ANTHROPIC_DEFAULT_OPUS_MODEL", "ANTHROPIC_DEFAULT_SONNET_MODEL", "ANTHROPIC_DEFAULT_HAIKU_MODEL", "ANTHROPIC_DEFAULT_FABLE_MODEL", "CLAUDE_CODE_SUBAGENT_MODEL" ]
  {
    assert_eq!(
      live_json[ "env" ][ key ].as_str(), Some( "kimi-k3" ),
      "AC-05: switch to a kimi redirect account must write env.{key} = redirect_model; got: {live}",
    );
  }
  assert_eq!(
    live_json[ "env" ][ "CLAUDE_CODE_EFFORT_LEVEL" ].as_str(), Some( "max" ),
    "AC-05: switch to a kimi redirect account must write env.CLAUDE_CODE_EFFORT_LEVEL = \"max\"; got: {live}",
  );
  assert_eq!(
    live_json[ "env" ][ "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ].as_str(), Some( "1048576" ),
    "AC-05: a kimi-k3 redirect_model must write the 1M auto-compact window; got: {live}",
  );
}

/// AC-06: a `kimi-k2.7-code` `redirect_model` writes the narrower 256K
/// `CLAUDE_CODE_AUTO_COMPACT_WINDOW`, not the 1M default used for `kimi-k3*`.
#[ test ]
fn ft02_073_switch_to_kimi_redirect_uses_narrow_compact_window_for_non_k3_model()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "kimi-code.credentials.json" ),
    r#"{"accessToken":"sk-kimi-code-abc123"}"#,
  ).unwrap();
  std::fs::write(
    store.join( "kimi-code.json" ),
    r#"{"backend":"redirect","base_url":"https://api.moonshot.ai/anthropic","redirect_model":"kimi-k2.7-code","inference_provider":"kimi"}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "kimi-code", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) ).unwrap();
  let live_json : serde_json::Value = serde_json::from_str( &live ).unwrap();
  assert_eq!(
    live_json[ "env" ][ "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ].as_str(), Some( "262144" ),
    "AC-06: kimi-k2.7-code must write the 256K auto-compact window, not the kimi-k3 1M default; got: {live}",
  );
}

/// AC-08: a `backend: redirect` account whose `inference_provider` is not `"kimi"`
/// (here: absent, defaulting to `"anthropic"`) gets only the pre-existing 3
/// `ANTHROPIC_*` vars — none of the 7 Kimi-tier additions.
#[ test ]
fn ft03_073_switch_to_redirect_non_kimi_provider_omits_tier_env_vars()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "other@foreign.ai.credentials.json" ),
    r#"{"accessToken":"sk-other-abc123"}"#,
  ).unwrap();
  std::fs::write(
    store.join( "other@foreign.ai.json" ),
    r#"{"backend":"redirect","base_url":"https://api.other.ai/anthropic","redirect_model":"other-model-1"}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "other@foreign.ai", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) ).unwrap();
  let live_json : serde_json::Value = serde_json::from_str( &live ).unwrap();
  assert_eq!(
    live_json[ "env" ][ "ANTHROPIC_MODEL" ].as_str(), Some( "other-model-1" ),
    "sanity: the pre-existing 3 vars must still be written; got: {live}",
  );
  for key in [ "ANTHROPIC_DEFAULT_OPUS_MODEL", "ANTHROPIC_DEFAULT_SONNET_MODEL", "ANTHROPIC_DEFAULT_HAIKU_MODEL", "ANTHROPIC_DEFAULT_FABLE_MODEL", "CLAUDE_CODE_SUBAGENT_MODEL", "CLAUDE_CODE_EFFORT_LEVEL", "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ]
  {
    assert!(
      live_json[ "env" ].get( key ).is_none(),
      "AC-08: a non-kimi redirect account must not get the Kimi-tier env.{key}; got: {live}",
    );
  }
}

/// AC-07: switching from a `kimi` redirect account to a `backend: anthropic`
/// account clears all 10 env vars (the 3 pre-existing `ANTHROPIC_*` plus the 7
/// Kimi-tier additions) — not just the original 3.
#[ test ]
fn ft04_073_switch_from_kimi_to_anthropic_clears_all_tier_env_vars()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "alice@test.com.credentials.json" ),
    r#"{"accessToken":"tok","expiresAt":9999999999999}"#,
  ).unwrap();
  // Live settings.json already carries a full Kimi-tier env block from a prior switch.
  std::fs::write(
    dot_claude.join( "settings.json" ),
    r#"{"env":{"ANTHROPIC_BASE_URL":"https://api.moonshot.ai/anthropic","ANTHROPIC_AUTH_TOKEN":"sk-foreign","ANTHROPIC_MODEL":"kimi-k3","ANTHROPIC_DEFAULT_OPUS_MODEL":"kimi-k3","ANTHROPIC_DEFAULT_SONNET_MODEL":"kimi-k3","ANTHROPIC_DEFAULT_HAIKU_MODEL":"kimi-k3","ANTHROPIC_DEFAULT_FABLE_MODEL":"kimi-k3","CLAUDE_CODE_SUBAGENT_MODEL":"kimi-k3","CLAUDE_CODE_EFFORT_LEVEL":"max","CLAUDE_CODE_AUTO_COMPACT_WINDOW":"1048576"}}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "alice@test.com", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) ).unwrap();
  let live_json : serde_json::Value = serde_json::from_str( &live ).unwrap();
  assert!(
    live_json.get( "env" ).is_none(),
    "AC-07: env must be removed entirely once every ANTHROPIC_*/CLAUDE_CODE_* sub-key is cleared; got: {live}",
  );
}

/// AC-07: switching from a `kimi` redirect account to a *different*, non-kimi
/// redirect account also clears the 7 stale Kimi-tier vars — this exercises the
/// redirect-branch's own non-kimi cleanup path, distinct from the anthropic-branch
/// cleanup `ft04_073` covers.
#[ test ]
fn ft05_073_switch_from_kimi_to_other_redirect_clears_stale_tier_env_vars()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "other@foreign.ai.credentials.json" ),
    r#"{"accessToken":"sk-other-abc123"}"#,
  ).unwrap();
  std::fs::write(
    store.join( "other@foreign.ai.json" ),
    r#"{"backend":"redirect","base_url":"https://api.other.ai/anthropic","redirect_model":"other-model-1"}"#,
  ).unwrap();
  std::fs::write(
    dot_claude.join( "settings.json" ),
    r#"{"env":{"ANTHROPIC_BASE_URL":"https://api.moonshot.ai/anthropic","ANTHROPIC_AUTH_TOKEN":"sk-foreign","ANTHROPIC_MODEL":"kimi-k3","ANTHROPIC_DEFAULT_OPUS_MODEL":"kimi-k3","CLAUDE_CODE_EFFORT_LEVEL":"max","CLAUDE_CODE_AUTO_COMPACT_WINDOW":"1048576"}}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "other@foreign.ai", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) ).unwrap();
  let live_json : serde_json::Value = serde_json::from_str( &live ).unwrap();
  for key in [ "ANTHROPIC_DEFAULT_OPUS_MODEL", "CLAUDE_CODE_EFFORT_LEVEL", "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ]
  {
    assert!(
      live_json[ "env" ].get( key ).is_none(),
      "AC-07: switching to a non-kimi redirect account must clear stale Kimi-tier env.{key}; got: {live}",
    );
  }
  assert_eq!(
    live_json[ "env" ][ "ANTHROPIC_MODEL" ].as_str(), Some( "other-model-1" ),
    "sanity: the new account's own ANTHROPIC_MODEL must still be written; got: {live}",
  );
}

// ── Feature 078 — DeepSeek provider preset env vars ────────────────────────────

/// AC-01: `switch_account()` to a `backend: redirect`, `inference_provider: "deepseek"`
/// account writes the 2 Pro-tier vars (mirroring `redirect_model`), the 2 Flash-tier vars
/// (fixed to "deepseek-v4-flash"), `CLAUDE_CODE_EFFORT_LEVEL`, and the flat
/// `CLAUDE_CODE_AUTO_COMPACT_WINDOW`, alongside the pre-existing 3 `ANTHROPIC_*` vars.
#[ test ]
fn ft06_078_switch_to_deepseek_redirect_writes_tier_env_vars()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "deepseek.credentials.json" ),
    r#"{"accessToken":"sk-deepseek-abc123"}"#,
  ).unwrap();
  std::fs::write(
    store.join( "deepseek.json" ),
    r#"{"backend":"redirect","base_url":"https://api.deepseek.com/anthropic","redirect_model":"deepseek-v4-pro","inference_provider":"deepseek"}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "deepseek", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) )
    .expect( "~/.claude/settings.json must exist after switch_account" );
  let live_json : serde_json::Value = serde_json::from_str( &live ).expect( "settings.json must be valid JSON" );
  for key in [ "ANTHROPIC_DEFAULT_OPUS_MODEL", "ANTHROPIC_DEFAULT_SONNET_MODEL" ]
  {
    assert_eq!(
      live_json[ "env" ][ key ].as_str(), Some( "deepseek-v4-pro" ),
      "AC-01: switch to a deepseek redirect account must write env.{key} = redirect_model; got: {live}",
    );
  }
  for key in [ "ANTHROPIC_DEFAULT_HAIKU_MODEL", "CLAUDE_CODE_SUBAGENT_MODEL" ]
  {
    assert_eq!(
      live_json[ "env" ][ key ].as_str(), Some( "deepseek-v4-flash" ),
      "AC-01: switch to a deepseek redirect account must fix env.{key} = \"deepseek-v4-flash\" regardless of redirect_model; got: {live}",
    );
  }
  assert_eq!(
    live_json[ "env" ][ "CLAUDE_CODE_EFFORT_LEVEL" ].as_str(), Some( "max" ),
    "AC-01: switch to a deepseek redirect account must write env.CLAUDE_CODE_EFFORT_LEVEL = \"max\"; got: {live}",
  );
  assert_eq!(
    live_json[ "env" ][ "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ].as_str(), Some( "786432" ),
    "AC-01: switch to a deepseek redirect account must write the flat 768K auto-compact window; got: {live}",
  );
  assert!(
    live_json[ "env" ].get( "ANTHROPIC_DEFAULT_FABLE_MODEL" ).is_none(),
    "AC-01: DeepSeek has no Fable-tier substitution — env.ANTHROPIC_DEFAULT_FABLE_MODEL must not be written; got: {live}",
  );
}

/// AC-02: a different `redirect_model` value (`deepseek-v4-flash` itself, standing in
/// for any non-"-pro" model string) still writes the same flat `786432`
/// `CLAUDE_CODE_AUTO_COMPACT_WINDOW` — unlike Kimi's tier, `DeepSeek`'s window never
/// branches on the model string.
#[ test ]
fn ft07_078_switch_to_deepseek_redirect_uses_flat_compact_window_regardless_of_model()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "deepseek-alt.credentials.json" ),
    r#"{"accessToken":"sk-deepseek-alt-abc123"}"#,
  ).unwrap();
  std::fs::write(
    store.join( "deepseek-alt.json" ),
    r#"{"backend":"redirect","base_url":"https://api.deepseek.com/anthropic","redirect_model":"deepseek-v4-flash","inference_provider":"deepseek"}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "deepseek-alt", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) ).unwrap();
  let live_json : serde_json::Value = serde_json::from_str( &live ).unwrap();
  assert_eq!(
    live_json[ "env" ][ "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ].as_str(), Some( "786432" ),
    "AC-02: CLAUDE_CODE_AUTO_COMPACT_WINDOW must stay flat at 786432 regardless of redirect_model, unlike Kimi's k3/non-k3 branch; got: {live}",
  );
}

/// AC-03: a `backend: redirect` account whose `inference_provider` is not `"deepseek"`
/// (here: absent) gets only the pre-existing 3 `ANTHROPIC_*` vars — none of the 6
/// DeepSeek-tier additions.
#[ test ]
fn ft08_078_switch_to_redirect_non_deepseek_provider_omits_tier_env_vars()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "other2@foreign.ai.credentials.json" ),
    r#"{"accessToken":"sk-other2-abc123"}"#,
  ).unwrap();
  std::fs::write(
    store.join( "other2@foreign.ai.json" ),
    r#"{"backend":"redirect","base_url":"https://api.other2.ai/anthropic","redirect_model":"other2-model-1"}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "other2@foreign.ai", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) ).unwrap();
  let live_json : serde_json::Value = serde_json::from_str( &live ).unwrap();
  assert_eq!(
    live_json[ "env" ][ "ANTHROPIC_MODEL" ].as_str(), Some( "other2-model-1" ),
    "sanity: the pre-existing 3 vars must still be written; got: {live}",
  );
  for key in [ "ANTHROPIC_DEFAULT_OPUS_MODEL", "ANTHROPIC_DEFAULT_SONNET_MODEL", "ANTHROPIC_DEFAULT_HAIKU_MODEL", "CLAUDE_CODE_SUBAGENT_MODEL", "CLAUDE_CODE_EFFORT_LEVEL", "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ]
  {
    assert!(
      live_json[ "env" ].get( key ).is_none(),
      "AC-03: a non-deepseek redirect account must not get the DeepSeek-tier env.{key}; got: {live}",
    );
  }
}

/// AC-04: switching from a `deepseek` redirect account to a `backend: anthropic`
/// account clears all 9 env vars (the 3 pre-existing `ANTHROPIC_*` plus the 6
/// DeepSeek-tier additions) — not just the original 3.
#[ test ]
fn ft09_078_switch_from_deepseek_to_anthropic_clears_tier_env_vars()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "alice2@test.com.credentials.json" ),
    r#"{"accessToken":"tok","expiresAt":9999999999999}"#,
  ).unwrap();
  // Live settings.json already carries a full DeepSeek-tier env block from a prior switch.
  std::fs::write(
    dot_claude.join( "settings.json" ),
    r#"{"env":{"ANTHROPIC_BASE_URL":"https://api.deepseek.com/anthropic","ANTHROPIC_AUTH_TOKEN":"sk-deepseek","ANTHROPIC_MODEL":"deepseek-v4-pro","ANTHROPIC_DEFAULT_OPUS_MODEL":"deepseek-v4-pro","ANTHROPIC_DEFAULT_SONNET_MODEL":"deepseek-v4-pro","ANTHROPIC_DEFAULT_HAIKU_MODEL":"deepseek-v4-flash","CLAUDE_CODE_SUBAGENT_MODEL":"deepseek-v4-flash","CLAUDE_CODE_EFFORT_LEVEL":"max","CLAUDE_CODE_AUTO_COMPACT_WINDOW":"786432"}}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "alice2@test.com", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) ).unwrap();
  let live_json : serde_json::Value = serde_json::from_str( &live ).unwrap();
  assert!(
    live_json.get( "env" ).is_none(),
    "AC-04: env must be removed entirely once every ANTHROPIC_*/CLAUDE_CODE_* sub-key is cleared; got: {live}",
  );
}

/// AC-05: switching from a `deepseek` redirect account to a *different*, non-deepseek
/// redirect account also clears the 6 stale DeepSeek-tier vars — this exercises the
/// redirect-branch's own non-deepseek cleanup path, distinct from the anthropic-branch
/// cleanup `ft09_078` covers.
#[ test ]
fn ft10_078_switch_from_deepseek_to_other_redirect_clears_stale_tier_env_vars()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "other3@foreign.ai.credentials.json" ),
    r#"{"accessToken":"sk-other3-abc123"}"#,
  ).unwrap();
  std::fs::write(
    store.join( "other3@foreign.ai.json" ),
    r#"{"backend":"redirect","base_url":"https://api.other3.ai/anthropic","redirect_model":"other3-model-1"}"#,
  ).unwrap();
  std::fs::write(
    dot_claude.join( "settings.json" ),
    r#"{"env":{"ANTHROPIC_BASE_URL":"https://api.deepseek.com/anthropic","ANTHROPIC_AUTH_TOKEN":"sk-deepseek","ANTHROPIC_MODEL":"deepseek-v4-pro","ANTHROPIC_DEFAULT_OPUS_MODEL":"deepseek-v4-pro","CLAUDE_CODE_EFFORT_LEVEL":"max","CLAUDE_CODE_AUTO_COMPACT_WINDOW":"786432"}}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "other3@foreign.ai", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) ).unwrap();
  let live_json : serde_json::Value = serde_json::from_str( &live ).unwrap();
  for key in [ "ANTHROPIC_DEFAULT_OPUS_MODEL", "CLAUDE_CODE_EFFORT_LEVEL", "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ]
  {
    assert!(
      live_json[ "env" ].get( key ).is_none(),
      "AC-05: switching to a non-deepseek redirect account must clear stale DeepSeek-tier env.{key}; got: {live}",
    );
  }
  assert_eq!(
    live_json[ "env" ][ "ANTHROPIC_MODEL" ].as_str(), Some( "other3-model-1" ),
    "sanity: the new account's own ANTHROPIC_MODEL must still be written; got: {live}",
  );
}

/// AC-11 (direction 1): switching from a live state populated by a `kimi` redirect
/// account directly to a `deepseek` redirect account clears the 7 stale Kimi-tier vars
/// AND writes the 6 DeepSeek-tier vars in the same call — the two provider bundles must
/// never coexist in `env` (docs/feature/078's "cross-provider clearing" design note).
#[ test ]
fn ft11_078_switch_from_kimi_to_deepseek_clears_kimi_writes_deepseek()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "deepseek2.credentials.json" ),
    r#"{"accessToken":"sk-deepseek2-abc123"}"#,
  ).unwrap();
  std::fs::write(
    store.join( "deepseek2.json" ),
    r#"{"backend":"redirect","base_url":"https://api.deepseek.com/anthropic","redirect_model":"deepseek-v4-pro","inference_provider":"deepseek"}"#,
  ).unwrap();
  // Live settings.json already carries a full Kimi-tier env block from a prior switch.
  std::fs::write(
    dot_claude.join( "settings.json" ),
    r#"{"env":{"ANTHROPIC_BASE_URL":"https://api.moonshot.ai/anthropic","ANTHROPIC_AUTH_TOKEN":"sk-foreign","ANTHROPIC_MODEL":"kimi-k3","ANTHROPIC_DEFAULT_OPUS_MODEL":"kimi-k3","ANTHROPIC_DEFAULT_SONNET_MODEL":"kimi-k3","ANTHROPIC_DEFAULT_HAIKU_MODEL":"kimi-k3","ANTHROPIC_DEFAULT_FABLE_MODEL":"kimi-k3","CLAUDE_CODE_SUBAGENT_MODEL":"kimi-k3","CLAUDE_CODE_EFFORT_LEVEL":"max","CLAUDE_CODE_AUTO_COMPACT_WINDOW":"1048576"}}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "deepseek2", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) ).unwrap();
  let live_json : serde_json::Value = serde_json::from_str( &live ).unwrap();
  assert!(
    live_json[ "env" ].get( "ANTHROPIC_DEFAULT_FABLE_MODEL" ).is_none(),
    "AC-11: switching kimi→deepseek must clear the Kimi-only Fable var; got: {live}",
  );
  for key in [ "ANTHROPIC_DEFAULT_OPUS_MODEL", "ANTHROPIC_DEFAULT_SONNET_MODEL" ]
  {
    assert_eq!(
      live_json[ "env" ][ key ].as_str(), Some( "deepseek-v4-pro" ),
      "AC-11: switching kimi→deepseek must overwrite env.{key} with the DeepSeek account's redirect_model; got: {live}",
    );
  }
  for key in [ "ANTHROPIC_DEFAULT_HAIKU_MODEL", "CLAUDE_CODE_SUBAGENT_MODEL" ]
  {
    assert_eq!(
      live_json[ "env" ][ key ].as_str(), Some( "deepseek-v4-flash" ),
      "AC-11: switching kimi→deepseek must fix env.{key} = \"deepseek-v4-flash\"; got: {live}",
    );
  }
  assert_eq!(
    live_json[ "env" ][ "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ].as_str(), Some( "786432" ),
    "AC-11: switching kimi→deepseek must overwrite the stale 1M Kimi window with DeepSeek's flat 786432; got: {live}",
  );
}

/// AC-11 (direction 2): switching from a live state populated by a `deepseek` redirect
/// account directly to a `kimi` redirect account clears the 6 stale DeepSeek-tier vars
/// AND writes the 7 Kimi-tier vars in the same call — the mirror direction of `ft11_078`.
#[ test ]
fn ft12_078_switch_from_deepseek_to_kimi_clears_deepseek_writes_kimi()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "kimi2.credentials.json" ),
    r#"{"accessToken":"sk-kimi2-abc123"}"#,
  ).unwrap();
  std::fs::write(
    store.join( "kimi2.json" ),
    r#"{"backend":"redirect","base_url":"https://api.moonshot.ai/anthropic","redirect_model":"kimi-k3","inference_provider":"kimi"}"#,
  ).unwrap();
  // Live settings.json already carries a full DeepSeek-tier env block from a prior switch.
  std::fs::write(
    dot_claude.join( "settings.json" ),
    r#"{"env":{"ANTHROPIC_BASE_URL":"https://api.deepseek.com/anthropic","ANTHROPIC_AUTH_TOKEN":"sk-deepseek","ANTHROPIC_MODEL":"deepseek-v4-pro","ANTHROPIC_DEFAULT_OPUS_MODEL":"deepseek-v4-pro","ANTHROPIC_DEFAULT_SONNET_MODEL":"deepseek-v4-pro","ANTHROPIC_DEFAULT_HAIKU_MODEL":"deepseek-v4-flash","CLAUDE_CODE_SUBAGENT_MODEL":"deepseek-v4-flash","CLAUDE_CODE_EFFORT_LEVEL":"max","CLAUDE_CODE_AUTO_COMPACT_WINDOW":"786432"}}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "kimi2", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) ).unwrap();
  let live_json : serde_json::Value = serde_json::from_str( &live ).unwrap();
  for key in [ "ANTHROPIC_DEFAULT_OPUS_MODEL", "ANTHROPIC_DEFAULT_SONNET_MODEL", "ANTHROPIC_DEFAULT_HAIKU_MODEL", "ANTHROPIC_DEFAULT_FABLE_MODEL", "CLAUDE_CODE_SUBAGENT_MODEL" ]
  {
    assert_eq!(
      live_json[ "env" ][ key ].as_str(), Some( "kimi-k3" ),
      "AC-11: switching deepseek→kimi must overwrite env.{key} with the Kimi account's redirect_model; got: {live}",
    );
  }
  assert_eq!(
    live_json[ "env" ][ "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ].as_str(), Some( "1048576" ),
    "AC-11: switching deepseek→kimi must overwrite the stale flat DeepSeek window with Kimi's 1M kimi-k3 window; got: {live}",
  );
}

/// BUG-002 MRE: `parse_string_field()` (and siblings) search the entire input for the
/// first occurrence of a key, with no way to bound the search to a single object —
/// callers with multi-entry JSON (e.g. `roles_json`'s membership list) silently get the
/// wrong entry's value. `extract_object_block()` gives callers a way to bound the search
/// to one object before calling the existing helpers.
///
/// # Root Cause
/// `parse_string_field()`/`parse_u64_field()`/`parse_bool_field()`/`parse_string_array_field()`
/// all open with an unbounded `json.find(&search)` over the ENTIRE input string — none
/// accepts or enforces "search only within this one object." A caller holding multi-entry
/// JSON (e.g. `roles_json`, a list of workspace/organization memberships) has no way to
/// scope the search to the entry it actually needs.
///
/// # Why Not Caught
/// No test exercised any of the four helpers against multi-entry JSON — every existing
/// fixture is a flat, single-object JSON blob (credentials files, settings.json), where
/// "first occurrence" is always correct by coincidence of there being nothing else to find.
///
/// # Fix Applied
/// Added `extract_object_block()` — a brace-depth-counted `{...}` bound (mirrors
/// `claude_quota`'s own helper of the same name; independently duplicated, not shared).
/// A caller walking a multi-entry array can now bound each entry with
/// `extract_object_block()` before calling `parse_string_field()` etc. on the bounded
/// slice, eliminating the wrong-entry ambiguity for any caller that adopts it.
///
/// # Prevention
/// Reproduces the exact MRE scenario documented in BUG-002 (`roles_json` with two
/// workspace memberships) and asserts the second entry's `workspace_name` is correctly
/// extracted once bounded, not silently defaulting to the first (Acme) entry.
///
/// # Pitfall
/// The existing 4 unbounded helpers are UNCHANGED and remain correct for genuinely flat,
/// single-object JSON — do not add object-boundary scanning inside them directly, since
/// that would need a scoping parameter and break every existing single-object call site.
#[ doc = "bug_reproducer(BUG-002)" ]
#[ test ]
fn bug002_extract_object_block_bounds_multi_entry_roles_json()
{
  let roles_json = r#"{"roles":[
  {"organization_name":"Acme Corp","organization_uuid":"org-AAA","workspace_name":"Acme Prod","workspace_uuid":"ws-AAA"},
  {"organization_name":"Beta Inc","organization_uuid":"org-BBB","workspace_name":"Beta Prod","workspace_uuid":"ws-BBB"}
]}"#;

  // Sanity: unbounded search still returns the first entry — unchanged, documented
  // behavior for flat single-object JSON; not itself the fix under test.
  let unbounded = account::parse_string_field( roles_json, "workspace_name" );
  assert_eq!(
    unbounded.as_deref(), Some( "Acme Prod" ),
    "sanity: unbounded parse_string_field must still return the first entry; got {unbounded:?}",
  );

  // Bound the search to the SECOND membership entry via extract_object_block().
  let second_brace = roles_json.match_indices( '{' ).nth( 2 ).map( |( i, _ )| i )
    .expect( "MRE fixture must contain a third '{' (outer object + 2 memberships)" );
  let second_entry = account::extract_object_block( &roles_json[ second_brace.. ] )
    .expect( "extract_object_block must bound the second membership object" );

  let scoped = account::parse_string_field( second_entry, "workspace_name" );
  assert_eq!(
    scoped.as_deref(), Some( "Beta Prod" ),
    "BUG-002: once the caller bounds the search to the second membership entry via \
     extract_object_block(), parse_string_field() must return that entry's own \
     workspace_name (Beta Prod), not silently fall back to the first entry; got {scoped:?}",
  );
}

// ── FT-08 (021): parse_string_array_field ─────────────────────────────────────
// Relocated from an in-src `#[cfg(test)]` module in account.rs — all tests for
// this crate live in tests/ per the workspace test-placement convention.

/// `ft08_a`: Two-element array returns both values in order.
///
/// Given: `{"capabilities":["claude_max","chat"]}`
/// When: `parse_string_array_field(json, "capabilities")`
/// Then: Returns `["claude_max", "chat"]`
#[ test ]
fn ft08_parse_string_array_field_two_elements()
{
  let json   = r#"{"capabilities":["claude_max","chat"]}"#;
  let result = account::parse_string_array_field( json, "capabilities" );
  assert_eq!( result, vec![ "claude_max", "chat" ] );
}

/// `ft08_b`: Missing key returns empty Vec.
///
/// Given: JSON with no "capabilities" key
/// When: `parse_string_array_field(json, "capabilities")`
/// Then: Returns empty Vec
#[ test ]
fn ft08_parse_string_array_field_missing_key_returns_empty()
{
  let json   = r#"{"other_field":"value"}"#;
  let result = account::parse_string_array_field( json, "capabilities" );
  assert!( result.is_empty(), "missing key must return empty Vec, got: {result:?}" );
}

/// `ft08_c`: Empty array `[]` returns empty Vec.
///
/// Given: `{"capabilities":[]}`
/// When: `parse_string_array_field(json, "capabilities")`
/// Then: Returns empty Vec
#[ test ]
fn ft08_parse_string_array_field_empty_array_returns_empty()
{
  let json   = r#"{"capabilities":[]}"#;
  let result = account::parse_string_array_field( json, "capabilities" );
  assert!( result.is_empty(), "empty array must return empty Vec, got: {result:?}" );
}

