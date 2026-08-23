//! Active-account switching and post-switch live-state patching (env vars, model restore).

use std::path::Path;
use claude_core::ClaudePaths;
use claude_core::file_io::{ atomic_write, atomic_write_secret };
use super::types::AccountBackend;
use super::validate::{ validate_name, validate_redirect_name };
use super::store::lock_store;
use super::ownership::active_marker_filename;
use super::json_field::parse_string_field;

/// Validate that a named account can be switched to (name valid + file exists).
///
/// Called by both `switch_account` and the CLI dry-run path so that dry-run
/// reports the same errors as a live switch.
///
/// # Errors
///
/// Returns `NotFound` if the account does not exist.
#[ inline ]
// core::io::ErrorKind requires the unstable `core_io` feature (rust-lang/rust#154046) — not usable on stable.
#[ allow( clippy::std_instead_of_core ) ]
pub fn check_switch_preconditions( name : &str, credential_store : &Path ) -> Result< (), std::io::Error >
{
  // Feature 071: an existing account's own saved shape governs switch-time validation, not
  // the anthropic-only email requirement — a redirect account's arbitrary label (e.g. "kimi")
  // must switch successfully. Mirrors validate_name_for_save()'s `already_exists` branch: any
  // account already on disk (either backend) only needs the permissive filename-safety check.
  // Non-existent names keep the original fast-fail email-shape rejection before the NotFound
  // check below, unchanged from pre-071 behavior.
  let src = credential_store.join( format!( "{name}.credentials.json" ) );
  if src.exists()
  {
    return validate_redirect_name( name );
  }
  validate_name( name )?;
  Err( std::io::Error::new(
    std::io::ErrorKind::NotFound,
    format!( "account '{name}' not found in {}", credential_store.display() ),
  ) )
}

/// Feature 073: Kimi-tier model-default env var names that mirror `redirect_model`'s
/// value for a `backend: redirect`, `inference_provider: "kimi"` account. Distinct from
/// the 3 original `ANTHROPIC_BASE_URL`/`_AUTH_TOKEN`/`_MODEL` vars Feature 071 writes.
const KIMI_MODEL_TIER_ENV_VARS : [ &str ; 5 ] =
[
  "ANTHROPIC_DEFAULT_OPUS_MODEL",
  "ANTHROPIC_DEFAULT_SONNET_MODEL",
  "ANTHROPIC_DEFAULT_HAIKU_MODEL",
  "ANTHROPIC_DEFAULT_FABLE_MODEL",
  "CLAUDE_CODE_SUBAGENT_MODEL",
];

/// Feature 073: the auto-compact context window a Kimi `redirect_model` needs.
/// `kimi-k3*` supports a 1M-token window; every other known/unknown Kimi model
/// defaults to the narrower 256K value — under-sizing only costs more frequent
/// compaction, while over-sizing risks a real context-overflow failure, so the
/// narrower value is the safe default for anything not explicitly `kimi-k3*`.
fn kimi_auto_compact_window( model : &str ) -> &'static str
{
  if model.starts_with( "kimi-k3" ) { "1048576" } else { "262144" }
}

/// Feature 073: write all 7 Kimi-tier env vars for a `backend: redirect`,
/// `inference_provider: "kimi"` account — the 5 default-model vars (mirroring
/// `redirect_model`), `CLAUDE_CODE_EFFORT_LEVEL` (always `"max"`), and
/// `CLAUDE_CODE_AUTO_COMPACT_WINDOW` (sized via `kimi_auto_compact_window()`).
fn write_kimi_tier_env_vars( live_settings_path : &Path, model : &str )
{
  for key in KIMI_MODEL_TIER_ENV_VARS
  {
    let _ = claude_core::settings_io::set_env_var( live_settings_path, key, model );
  }
  let _ = claude_core::settings_io::set_env_var( live_settings_path, "CLAUDE_CODE_EFFORT_LEVEL", "max" );
  let _ = claude_core::settings_io::set_env_var( live_settings_path, "CLAUDE_CODE_AUTO_COMPACT_WINDOW", kimi_auto_compact_window( model ) );
}

/// Feature 073: remove all 7 Kimi-tier env vars — shared by both the
/// "switched to a non-kimi redirect account" and "switched to an anthropic account"
/// cleanup paths, so a stale Kimi-tier var from a prior kimi switch never survives
/// into an unrelated account. Also invoked on a switch straight to a deepseek-tagged
/// redirect account (Feature 078) — see the 3-way dispatch in
/// `patch_live_state_after_switch()` below.
fn clear_kimi_tier_env_vars( live_settings_path : &Path )
{
  for key in KIMI_MODEL_TIER_ENV_VARS
  {
    let _ = claude_core::settings_io::remove_env_var( live_settings_path, key );
  }
  let _ = claude_core::settings_io::remove_env_var( live_settings_path, "CLAUDE_CODE_EFFORT_LEVEL" );
  let _ = claude_core::settings_io::remove_env_var( live_settings_path, "CLAUDE_CODE_AUTO_COMPACT_WINDOW" );
}

/// Feature 078: DeepSeek-tier env var names whose value mirrors `redirect_model`
/// (the account's own Pro-tier model) — distinct from the Flash-tier vars below, which
/// always take a fixed literal instead. Verified against `DeepSeek`'s own "Integrate with
/// Claude Code" guide (`api-docs.deepseek.com/quick_start/agent_integrations/claude_code`):
/// unlike Kimi's uniform 5-var mirror, `DeepSeek` documents a Pro/Flash split — only Opus and
/// Sonnet mirror the saved model; Haiku and subagent dispatch always route to the fixed
/// Flash model regardless of what the account was saved against.
const DEEPSEEK_PRO_TIER_ENV_VARS : [ &str ; 2 ] =
[
  "ANTHROPIC_DEFAULT_OPUS_MODEL",
  "ANTHROPIC_DEFAULT_SONNET_MODEL",
];

/// Feature 078: DeepSeek-tier env var names that always take the fixed
/// `DEEPSEEK_FLASH_MODEL` literal, never `redirect_model` — the Flash half of the
/// Pro/Flash split documented above. No `ANTHROPIC_DEFAULT_FABLE_MODEL` entry exists in
/// either bundle: `DeepSeek`'s guide documents no Fable-tier mapping, and inventing one
/// would write an uncertain detail as certain (see docs/feature/078's Design section).
const DEEPSEEK_FLASH_TIER_ENV_VARS : [ &str ; 2 ] =
[
  "ANTHROPIC_DEFAULT_HAIKU_MODEL",
  "CLAUDE_CODE_SUBAGENT_MODEL",
];

/// Feature 078: `DeepSeek`'s documented lighter/faster model — the fixed value substituted
/// for Haiku-tier and subagent dispatch regardless of the account's own `redirect_model`.
/// A single edit point: if `DeepSeek`'s recommended Flash model changes, updating this
/// constant updates it for every `DeepSeek` account (mirrors how Kimi's compact-window
/// thresholds are single edit points in `kimi_auto_compact_window()` above).
const DEEPSEEK_FLASH_MODEL : &str = "deepseek-v4-flash";

/// Feature 078: the fixed auto-compact context window for any `DeepSeek` redirect account —
/// no model-name branching, unlike `kimi_auto_compact_window()`. `DeepSeek` V4-Pro and
/// V4-Flash publicly share the same 1M-token context window, so there is no smaller-context
/// tier to protect against with a narrower default; adding a branch with no second value to
/// branch on would be speculative rather than evidence-based.
const DEEPSEEK_AUTO_COMPACT_WINDOW : &str = "786432";

/// Feature 078: write all 6 DeepSeek-tier env vars for a `backend: redirect`,
/// `inference_provider: "deepseek"` account — the 2 Pro-tier vars (mirroring
/// `redirect_model`), the 2 Flash-tier vars (fixed `DEEPSEEK_FLASH_MODEL`),
/// `CLAUDE_CODE_EFFORT_LEVEL` (always `"max"`, same value Kimi uses), and
/// `CLAUDE_CODE_AUTO_COMPACT_WINDOW` (fixed `DEEPSEEK_AUTO_COMPACT_WINDOW`).
fn write_deepseek_tier_env_vars( live_settings_path : &Path, model : &str )
{
  for key in DEEPSEEK_PRO_TIER_ENV_VARS
  {
    let _ = claude_core::settings_io::set_env_var( live_settings_path, key, model );
  }
  for key in DEEPSEEK_FLASH_TIER_ENV_VARS
  {
    let _ = claude_core::settings_io::set_env_var( live_settings_path, key, DEEPSEEK_FLASH_MODEL );
  }
  let _ = claude_core::settings_io::set_env_var( live_settings_path, "CLAUDE_CODE_EFFORT_LEVEL", "max" );
  let _ = claude_core::settings_io::set_env_var( live_settings_path, "CLAUDE_CODE_AUTO_COMPACT_WINDOW", DEEPSEEK_AUTO_COMPACT_WINDOW );
}

/// Feature 078: remove all 6 DeepSeek-tier env vars — shared by the
/// "switched to a non-deepseek redirect account" and "switched to an anthropic account"
/// cleanup paths, plus a switch straight to a kimi-tagged redirect account, so a stale
/// DeepSeek-tier var from a prior deepseek switch never survives into an unrelated account.
fn clear_deepseek_tier_env_vars( live_settings_path : &Path )
{
  for key in DEEPSEEK_PRO_TIER_ENV_VARS.into_iter().chain( DEEPSEEK_FLASH_TIER_ENV_VARS )
  {
    let _ = claude_core::settings_io::remove_env_var( live_settings_path, key );
  }
  let _ = claude_core::settings_io::remove_env_var( live_settings_path, "CLAUDE_CODE_EFFORT_LEVEL" );
  let _ = claude_core::settings_io::remove_env_var( live_settings_path, "CLAUDE_CODE_AUTO_COMPACT_WINDOW" );
}

/// Patch live `~/.claude.json` and `~/.claude/settings.json` from the unified `{name}.json`
/// snapshot after a switch's credentials/marker write has already landed.
///
/// Fix(BUG-254)
/// Root cause: `emailAddress` patch was gated inside `if let Ok(saved_val)` which
/// requires `{name}.json` to exist AND parse. When absent, `serde_json::from_str("")`
/// returns `Err` and the entire oauthAccount block is skipped — including the
/// BUG-217 emailAddress enforcement. Stale emailAddress persists in `~/.claude.json`.
/// Pitfall: identity-critical updates (`emailAddress`, `_active` marker) must be
/// unconditional. Non-critical data (model, org fields) can remain conditional on
/// metadata file availability.
/// Feature 071 (AC-06/AC-07): sync env.ANTHROPIC_* to the switched-to account's backend.
/// Redirect: writes `BASE_URL/AUTH_TOKEN/MODEL` from the account's own snapshot + credential
/// file, plus Feature 073/078's Kimi-tier or DeepSeek-tier bundle when `inference_provider`
/// matches. Anthropic: removes all three, then prunes the whole `env` object if now empty —
/// `remove_env_var()` alone leaves `"env": {}` behind; this composition lives here, not in
/// the shared `claude_core::settings_io` formatter (see docs/feature/071 design note).
/// Extracted out of `patch_live_state_after_switch()` (`clippy::too_many_lines`) once the
/// number of supported inference-tier bundles grew from one (Kimi) to two.
fn sync_backend_env_vars( live_settings_path : &Path, meta_text : &str, src : &Path, backend : AccountBackend )
{
  if backend == AccountBackend::Redirect
  {
    let creds_text   = std::fs::read_to_string( src ).unwrap_or_default();
    let access_token = parse_string_field( &creds_text, "accessToken" ).unwrap_or_default();
    let base_url     = parse_string_field( meta_text, "base_url" ).unwrap_or_default();
    let model        = parse_string_field( meta_text, "redirect_model" ).unwrap_or_default();
    let _ = claude_core::settings_io::set_env_var( live_settings_path, "ANTHROPIC_BASE_URL", &base_url );
    let _ = claude_core::settings_io::set_env_var( live_settings_path, "ANTHROPIC_AUTH_TOKEN", &access_token );
    let _ = claude_core::settings_io::set_env_var( live_settings_path, "ANTHROPIC_MODEL", &model );

    // Feature 073/078: Kimi-tier or DeepSeek-tier vars ride alongside the 3 above,
    // gated on an exact `inference_provider` match; either branch also clears the
    // *other* provider's tier bundle so switching directly between a kimi-tagged and a
    // deepseek-tagged redirect account never leaves both bundles coexisting in `env`
    // (docs/feature/078's Design section, "cross-provider clearing"). Any other/absent
    // provider clears both and gets the 3 base vars only.
    match parse_string_field( meta_text, "inference_provider" ).as_deref()
    {
      // Clear-then-write, not write-then-clear: DeepSeek's tier reuses 6 of Kimi's 7 JSON
      // key NAMES (only ANTHROPIC_DEFAULT_FABLE_MODEL is Kimi-exclusive), so the two
      // providers' env blocks overlap almost entirely at the key level. Clearing the
      // *other* provider's bundle after writing this one's would delete the very keys
      // just written (same key names); clearing first means the write always has the
      // last word, regardless of which keys the two tiers happen to share.
      Some( "kimi" ) =>
      {
        clear_deepseek_tier_env_vars( live_settings_path );
        write_kimi_tier_env_vars( live_settings_path, &model );
      }
      Some( "deepseek" ) =>
      {
        clear_kimi_tier_env_vars( live_settings_path );
        write_deepseek_tier_env_vars( live_settings_path, &model );
      }
      _ =>
      {
        clear_kimi_tier_env_vars( live_settings_path );
        clear_deepseek_tier_env_vars( live_settings_path );
      }
    }
  }
  else
  {
    let _ = claude_core::settings_io::remove_env_var( live_settings_path, "ANTHROPIC_BASE_URL" );
    let _ = claude_core::settings_io::remove_env_var( live_settings_path, "ANTHROPIC_AUTH_TOKEN" );
    let _ = claude_core::settings_io::remove_env_var( live_settings_path, "ANTHROPIC_MODEL" );
    clear_kimi_tier_env_vars( live_settings_path );
    clear_deepseek_tier_env_vars( live_settings_path );
    if claude_core::settings_io::get_setting( live_settings_path, "env" ).ok().flatten().as_deref() == Some( "{}" )
    {
      let _ = claude_core::settings_io::remove_setting( live_settings_path, "env" );
    }
  }
}

fn patch_live_state_after_switch( name : &str, credential_store : &Path, paths : &ClaudePaths, src : &Path )
{
  let meta_path = credential_store.join( format!( "{name}.json" ) );
  let meta_text = std::fs::read_to_string( &meta_path ).unwrap_or_default();

  // Patch live ~/.claude.json in one read-modify-write pass (surgical — preserves
  // machine-global keys). Previously this was two sequential whole-file writes
  // (unconditional emailAddress patch, then oauthAccount restore) — a reader between
  // them saw a half-patched identity, and the second write redid the first's work.
  {
    let live_path = paths.claude_json_file();
    let mut live_val = std::fs::read_to_string( &live_path )
      .ok()
      .and_then( |s| serde_json::from_str::< serde_json::Value >( &s ).ok() )
      .unwrap_or_else( || serde_json::json!( {} ) );

    // Restore the saved oauthAccount snapshot wholesale when one exists.
    if let Some( mut oauth ) = serde_json::from_str::< serde_json::Value >( &meta_text )
      .ok()
      .and_then( | saved | saved.get( "oauthAccount" ).cloned() )
    {
      if let Some( oa_obj ) = oauth.as_object_mut()
      {
        // Fix(BUG-217): enforce emailAddress == name — snapshot may contain stale email.
        oa_obj.insert( "emailAddress".to_string(), serde_json::Value::String( name.to_string() ) );
        // Fix(BUG-219): override org-identity fields from saved roles data.
        if let Some( org_name ) = parse_string_field( &meta_text, "organization_name" )
        {
          if !org_name.is_empty()
          {
            oa_obj.insert( "organizationName".to_string(), serde_json::Value::String( org_name ) );
          }
        }
        if let Some( org_uuid ) = parse_string_field( &meta_text, "organization_uuid" )
        {
          if !org_uuid.is_empty()
          {
            oa_obj.insert( "organizationUuid".to_string(), serde_json::Value::String( org_uuid ) );
          }
        }
      }
      if let Some( obj ) = live_val.as_object_mut()
      {
        obj.insert( "oauthAccount".to_string(), oauth );
      }
    }
    else if let Some( obj ) = live_val.as_object_mut()
    {
      // No saved snapshot — still patch emailAddress so the live identity tracks the switch.
      let oauth = obj.entry( "oauthAccount" )
        .or_insert_with( || serde_json::json!( {} ) );
      if let Some( oa_obj ) = oauth.as_object_mut()
      {
        oa_obj.insert( "emailAddress".to_string(), serde_json::Value::String( name.to_string() ) );
      }
    }
    let _ = atomic_write( &live_path, &serde_json::to_string_pretty( &live_val ).map( | s | s + "\n" ).unwrap_or_default() );
  }

  let backend = AccountBackend::parse( &parse_string_field( &meta_text, "backend" ).unwrap_or_default() );

  // Restore model preference into live ~/.claude/settings.json — anthropic accounts only.
  // A redirect account's meta never stores `model` (save() strips it), and a stray value
  // there is foreign — snapshotted from whichever account was live before. Restoring it
  // would leave a top-level pin that takes over whenever the env block is absent
  // (env.ANTHROPIC_MODEL outranks it only while present). Redirect ⇒ remove the key.
  let model = if backend == AccountBackend::Redirect { None }
    else { parse_string_field( &meta_text, "model" ) };
  let live_settings_path = paths.settings_file();
  let mut live_settings = std::fs::read_to_string( &live_settings_path )
    .ok()
    .and_then( |s| serde_json::from_str::< serde_json::Value >( &s ).ok() )
    .unwrap_or_else( || serde_json::json!( {} ) );
  if let Some( obj ) = live_settings.as_object_mut()
  {
    match model
    {
      Some( m ) => { obj.insert( "model".to_string(), serde_json::Value::String( m ) ); }
      None      => { obj.remove( "model" ); }
    }
  }
  let _ = atomic_write( &live_settings_path, &serde_json::to_string_pretty( &live_settings ).map( | s | s + "\n" ).unwrap_or_default() );

  sync_backend_env_vars( &live_settings_path, &meta_text, src, backend );
}

/// Switch the active account by name.
///
/// Atomically overwrites the credentials file with the named account's
/// credentials using write-then-rename, then updates `{credential_store}/_active`.
///
/// # Errors
///
/// Returns `NotFound` if the account does not exist, or an I/O error if
/// the switch cannot be completed.
#[ inline ]
pub fn switch_account( name : &str, credential_store : &Path, paths : &ClaudePaths ) -> Result< (), std::io::Error >
{
  // `_lock` must be a named binding — `let _ =` would drop (and release) immediately.
  let _lock = lock_store( credential_store )?;
  check_switch_preconditions( name, credential_store )?;
  let src = credential_store.join( format!( "{name}.credentials.json" ) );

  // Atomic install of the live credentials (unique temp + rename, 0o600 from creation).
  let creds = paths.credentials_file();
  // Feature 071: a redirect account can be saved and switched-to without `~/.claude/`
  // ever existing (unlike anthropic accounts, whose save path always reads the live
  // credentials file first, guaranteeing the directory). Same class of gap as BUG-258
  // (see set_session_model's fix note below) — ensure the parent exists before writing.
  if let Some( parent ) = creds.parent() { let _ = std::fs::create_dir_all( parent ); }
  let creds_text = std::fs::read_to_string( &src )?;
  atomic_write_secret( &creds, &creds_text )?;

  // BUG-485 task/claude_profile_core/bug/completed/485_refresh_presync_reread_never_applied.md — live
  // credentials file (above) is updated before the active marker (below); a concurrent
  // refresh_token_with_live_path's pre-sync guard can observe the old marker while this
  // rename has already landed. The pre-sync block now re-reads the marker immediately
  // before its gated write (Fix(BUG-485) there), closing the corruption window.
  // Update active marker after credentials are safely in place.
  atomic_write( &credential_store.join( active_marker_filename() ), name )?;

  patch_live_state_after_switch( name, credential_store, paths, &src );

  Ok( () )
}
