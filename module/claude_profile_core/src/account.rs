//! Named credential storage and account rotation.
//!
//! # Account Store Layout
//!
//! ```text
//! $PRO/.persistent/claude/credential/
//!   alice@acme.com.credentials.json   ← OAuth credentials (tokens, expiry)
//!   alice@acme.com.json               ← account metadata (identity, model, roles, profile)
//!   alice@home.com.credentials.json
//!   alice@home.com.json
//!   _active_w003_user1                ← text: name of active account (per-machine)
//! ```
//!
//! The active marker filename is `_active_{hostname}_{user}` (see [`active_marker_filename`]).
//! Each machine maintains its own marker independently; add `_active_*` to `.gitignore`.
//!
//! # Examples
//!
//! ```no_run
//! use claude_profile_core::account;
//! use claude_core::ClaudePaths;
//! use std::path::Path;
//!
//! let paths = ClaudePaths::new().expect( "HOME must be set" );
//! let credential_store = Path::new( "/pro/.persistent/claude/credential" );
//!
//! // List stored accounts
//! for acct in account::list( credential_store ).expect( "failed to list accounts" )
//! {
//!   let active = if acct.is_active { " ← active" } else { "" };
//!   println!( "{}{} ({}) email={}", acct.name, active, acct.subscription_type, acct.email );
//! }
//!
//! // Save current credentials as "alice@acme.com"
//! account::save( "alice@acme.com", credential_store, &paths, true, None, None, None, None ).expect( "failed to save" );
//!
//! // Switch to "alice@home.com"
//! account::switch_account( "alice@home.com", credential_store, &paths ).expect( "failed to switch" );
//!
//! // Delete an old entry
//! account::delete( "alice@oldco.com", credential_store ).expect( "failed to delete" );
//! ```

use std::io::Write as _;
use std::path::Path;
use claude_core::ClaudePaths;

/// Metadata for a saved Claude Code account credential snapshot.
#[ derive( Debug, Clone ) ]
pub struct Account
{
  /// Account name — the email address used as the credential filename stem.
  pub name : String,
  /// Claude subscription type (e.g., `"max"`, `"pro"`).
  pub subscription_type : String,
  /// Rate limit tier identifier.
  pub rate_limit_tier : String,
  /// OAuth token expiry as Unix epoch milliseconds.
  pub expires_at_ms : u64,
  /// Whether this account's credentials are currently active.
  pub is_active : bool,
  /// Email address from saved `{name}.json` `emailAddress`.
  /// Empty string when snapshot absent or field missing.
  pub email : String,
  /// Display name from saved `{name}.json` `oauthAccount.displayName`.
  /// Empty string when snapshot absent or field missing.
  pub display_name : String,
  /// Billing type from saved `{name}.json` `oauthAccount.billingType`.
  /// Empty string when snapshot absent or field missing.
  pub billing : String,
  /// Active model from saved `{name}.json` `model` field.
  /// Empty string when snapshot absent or field missing.
  pub model : String,
  /// Stable user identifier from saved `{name}.json` `oauthAccount.taggedId`.
  /// Empty string when snapshot absent or field missing.
  pub tagged_id : String,
  /// UUID form of user identifier from saved `{name}.json` `oauthAccount.uuid`.
  /// Empty string when snapshot absent or field missing.
  pub uuid : String,
  /// Enabled product capabilities from saved `{name}.json` `oauthAccount.capabilities`.
  /// Empty vec when snapshot absent or field missing.
  pub capabilities : Vec< String >,
  /// Organisation UUID from saved `{name}.json` `organization_uuid`.
  /// Empty string when snapshot absent or field missing.
  pub organization_uuid : String,
  /// Organisation display name from saved `{name}.json` `organization_name`.
  /// Empty string when snapshot absent or field missing.
  pub organization_name : String,
  /// User's role in the organisation from saved `{name}.json` `organization_role` (Roles API path).
  /// Empty string when snapshot absent or field missing.
  pub org_role : String,
  /// Workspace UUID from saved `{name}.json` `workspace_uuid`.
  /// Empty string when snapshot absent or field missing (personal accounts have `null`).
  pub workspace_uuid : String,
  /// Workspace display name from saved `{name}.json` `workspace_name`.
  /// Empty string when snapshot absent or field missing (personal accounts have `null`).
  pub workspace_name : String,
  /// Machine host label from saved `{name}.json` `host`.
  /// Empty string when file absent or field missing.
  pub host : String,
  /// User-defined role label from saved `{name}.json` `role`.
  /// Empty string when file absent or field missing.
  pub role : String,
  /// Account owner from saved `{name}.json` `owner`; empty when unset — see Feature 036.
  pub owner : String,
  /// `true` when `owner` is empty (unowned) or matches `current_identity()` (owned by this machine).
  pub is_owned : bool,
  /// Renewal override from saved `{name}.json` `_renewal_at`; `None` when unset — see Feature 030.
  pub renewal_at : Option< String >,
}

/// List all accounts in `credential_store`.
///
/// Returns an empty `Vec` if the credential store does not exist yet — not an error.
///
/// # Errors
///
/// Returns an error if the credential store exists but cannot be read.
#[ inline ]
#[ must_use = "check the returned accounts list" ]
pub fn list( credential_store : &Path ) -> Result< Vec< Account >, std::io::Error >
{
  if !credential_store.exists() { return Ok( Vec::new() ); }

  let active   = read_active_marker( credential_store );
  // Pre-compute once — current_identity() reads env vars + resolves hostname.
  let identity = current_identity();
  let mut accounts = Vec::new();

  for entry in std::fs::read_dir( credential_store )?.flatten()
  {
    let path = entry.path();
    let Some( name ) = credential_stem( &path ) else { continue };
    let content = std::fs::read_to_string( &path ).unwrap_or_default();
    let subscription_type = parse_string_field( &content, "subscriptionType" )
      .unwrap_or_default();
    let rate_limit_tier = parse_string_field( &content, "rateLimitTier" )
      .unwrap_or_default();
    let expires_at_ms = parse_u64_field( &content, "expiresAt" )
      .unwrap_or( 0 );
    let is_active = active.as_deref() == Some( name.as_str() );

    // Read unified per-account metadata from {name}.json — best-effort, empty when absent.
    let meta_json = std::fs::read_to_string(
      credential_store.join( format!( "{name}.json" ) )
    ).unwrap_or_default();
    let email        = parse_string_field( &meta_json, "emailAddress"      ).unwrap_or_default();
    let display_name = parse_string_field( &meta_json, "displayName"      ).unwrap_or_default();
    let billing      = parse_string_field( &meta_json, "billingType"      ).unwrap_or_default();
    let model        = parse_string_field( &meta_json, "model"            ).unwrap_or_default();
    let tagged_id    = parse_string_field( &meta_json, "taggedId"         ).unwrap_or_default();
    let uuid         = parse_string_field( &meta_json, "uuid"             ).unwrap_or_default();
    let capabilities = parse_string_array_field( &meta_json, "capabilities" );
    let organization_uuid = parse_string_field( &meta_json, "organization_uuid" ).unwrap_or_default();
    let organization_name = parse_string_field( &meta_json, "organization_name" ).unwrap_or_default();
    let org_role          = parse_string_field( &meta_json, "organization_role" ).unwrap_or_default();
    let workspace_uuid    = parse_string_field( &meta_json, "workspace_uuid"    ).unwrap_or_default();
    let workspace_name    = parse_string_field( &meta_json, "workspace_name"    ).unwrap_or_default();
    let host         = parse_string_field( &meta_json, "host"       ).unwrap_or_default();
    let role         = parse_string_field( &meta_json, "role"       ).unwrap_or_default();
    let owner        = parse_string_field( &meta_json, "owner"      ).unwrap_or_default();
    let is_owned     = owner.is_empty() || owner == identity;
    let renewal_at   = parse_string_field( &meta_json, "_renewal_at" );

    accounts.push( Account
    {
      name,
      subscription_type,
      rate_limit_tier,
      expires_at_ms,
      is_active,
      email,
      display_name,
      billing,
      model,
      tagged_id,
      uuid,
      capabilities,
      organization_uuid,
      organization_name,
      org_role,
      workspace_uuid,
      workspace_name,
      host,
      role,
      owner,
      is_owned,
      renewal_at,
    } );
  }

  accounts.sort_by( | a, b | a.name.cmp( &b.name ) );
  Ok( accounts )
}

/// Save credentials as a named account in `credential_store`.
///
/// Writes two files per account:
/// - `{name}.credentials.json` — OAuth tokens and expiry
/// - `{name}.json` — unified metadata (identity, model, roles, profile)
///
/// When `creds` is `Some(bytes)`, writes `bytes` directly to the credential file.
/// When `creds` is `None`, copies from `paths.credentials_file()` (the live session file).
///
/// `host` / `role` are profile display metadata; pass `None` from background callers
/// to preserve existing values via merge.
///
/// `owner` sets the `owner` field in `{name}.json`:
/// - `Some(s)` — writes `s` as the owner (use `current_identity()` for CLI saves, `""` for unclaim).
/// - `None` — preserves existing `owner` field unchanged (background callers: refresh, touch paths).
///
/// # Errors
///
/// Returns an error if the name is invalid, the credentials file cannot be
/// read, or the credential store cannot be written.
#[ inline ]
#[ allow( clippy::too_many_arguments ) ] // 8th param `owner` added by Feature 036 — all args are independent concerns.
pub fn save(
  name             : &str,
  credential_store : &Path,
  paths            : &ClaudePaths,
  update_marker    : bool,
  creds            : Option< &[u8] >,
  host             : Option< &str >,
  role             : Option< &str >,
  owner            : Option< &str >,
) -> Result< (), std::io::Error >
{
  validate_name( name )?;
  std::fs::create_dir_all( credential_store )?;
  let dest = credential_store.join( format!( "{name}.credentials.json" ) );
  // Fix(BUG-221): accept direct credential bytes to bypass the copy-from-live-file path.
  match creds
  {
    Some( bytes ) => std::fs::write( &dest, bytes )?,
    None          => { std::fs::copy( paths.credentials_file(), &dest )?; }
  }

  // Build unified {name}.json — read-merge to preserve pre-existing keys (e.g. _renewal_at).
  let meta_path = credential_store.join( format!( "{name}.json" ) );
  let mut snapshot = std::fs::read_to_string( &meta_path )
    .ok()
    .and_then( |s| serde_json::from_str::< serde_json::Value >( &s ).ok() )
    .unwrap_or_else( || serde_json::json!( {} ) );
  if let Some( obj ) = snapshot.as_object_mut()
  {
    // Merge oauthAccount from live ~/.claude.json (surgical — only per-account data).
    if let Ok( live_text ) = std::fs::read_to_string( paths.claude_json_file() )
    {
      if let Ok( live_val ) = serde_json::from_str::< serde_json::Value >( &live_text )
      {
        if let Some( oauth ) = live_val.get( "oauthAccount" )
        {
          obj.insert( "oauthAccount".to_string(), oauth.clone() );
        }
      }
    }
    // Merge model preference from live ~/.claude/settings.json (best-effort).
    if let Ok( live_settings ) = std::fs::read_to_string( paths.settings_file() )
    {
      if let Some( model ) = parse_string_field( &live_settings, "model" )
      {
        obj.insert( "model".to_string(), serde_json::Value::String( model ) );
      }
    }
    // Merge org identity from endpoint 005 (best-effort, network).
    #[ cfg( feature = "enabled" ) ]
    {
      let creds_text = std::fs::read_to_string( paths.credentials_file() ).unwrap_or_default();
      if let Some( token ) = parse_string_field( &creds_text, "accessToken" )
      {
        if let Ok( roles ) = claude_quota::fetch_claude_cli_roles( &token )
        {
          let val_or_null = | s : &str | -> serde_json::Value
          {
            if s.is_empty() { serde_json::Value::Null }
            else { serde_json::Value::String( s.to_string() ) }
          };
          obj.insert( "organization_uuid".to_string(), serde_json::Value::String( roles.organization_uuid.clone() ) );
          obj.insert( "organization_name".to_string(), serde_json::Value::String( roles.organization_name.clone() ) );
          obj.insert( "organization_role".to_string(), serde_json::Value::String( roles.organization_role.clone() ) );
          obj.insert( "workspace_uuid".to_string(), val_or_null( &roles.workspace_uuid ) );
          obj.insert( "workspace_name".to_string(), val_or_null( &roles.workspace_name ) );
        }
      }
    }
    // Merge profile metadata when provided (CLI callers); None preserves existing values.
    if let Some( h ) = host
    {
      obj.insert( "host".to_string(), serde_json::Value::String( h.to_string() ) );
    }
    if let Some( r ) = role
    {
      obj.insert( "role".to_string(), serde_json::Value::String( r.to_string() ) );
    }
    // `owner` — write when Some (CLI saves); None preserves existing field (background callers).
    if let Some( o ) = owner
    {
      obj.insert( "owner".to_string(), serde_json::Value::String( o.to_string() ) );
    }
  }
  // Only write {name}.json when there is actual data to store — avoids empty {} files
  // for accounts with no oauthAccount, no model, and no host/role metadata.
  // Existing {name}.json is always non-empty (read-merged above), so this never drops data.
  if snapshot.as_object().is_some_and( |obj| !obj.is_empty() )
  {
    let _ = std::fs::write( &meta_path, serde_json::to_string_pretty( &snapshot ).map( | s | s + "\n" ).unwrap_or_default() );
  }

  // Clean up old satellite files (migration to unified {name}.json).
  let _ = std::fs::remove_file( credential_store.join( format!( "{name}.claude.json" ) ) );
  let _ = std::fs::remove_file( credential_store.join( format!( "{name}.settings.json" ) ) );
  let _ = std::fs::remove_file( credential_store.join( format!( "{name}.roles.json" ) ) );
  let _ = std::fs::remove_file( credential_store.join( format!( "{name}.profile.json" ) ) );

  if update_marker
  {
    std::fs::write( credential_store.join( active_marker_filename() ), name )?;
  }
  Ok( () )
}

/// Validate that a named account can be switched to (name valid + file exists).
///
/// Called by both `switch_account` and the CLI dry-run path so that dry-run
/// reports the same errors as a live switch.
///
/// # Errors
///
/// Returns `NotFound` if the account does not exist.
#[ inline ]
pub fn check_switch_preconditions( name : &str, credential_store : &Path ) -> Result< (), std::io::Error >
{
  validate_name( name )?;
  let src = credential_store.join( format!( "{name}.credentials.json" ) );
  if !src.exists()
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::NotFound,
      format!( "account '{name}' not found in {}", credential_store.display() ),
    ) );
  }
  Ok( () )
}

/// Switch the active account by name.
///
/// Atomically overwrites the credentials file with the named account's
/// credentials using write-then-rename, then updates `{credential_store}/_active`.
///
/// Fix(BUG-254)
/// Root cause: `emailAddress` patch was gated inside `if let Ok(saved_val)` which
/// requires `{name}.json` to exist AND parse. When absent, `serde_json::from_str("")`
/// returns `Err` and the entire oauthAccount block is skipped — including the
/// BUG-217 emailAddress enforcement. Stale emailAddress persists in `~/.claude.json`.
/// Pitfall: identity-critical updates (`emailAddress`, `_active` marker) must be
/// unconditional. Non-critical data (model, org fields) can remain conditional on
/// metadata file availability.
///
/// # Errors
///
/// Returns `NotFound` if the account does not exist, or an I/O error if
/// the switch cannot be completed.
#[ inline ]
pub fn switch_account( name : &str, credential_store : &Path, paths : &ClaudePaths ) -> Result< (), std::io::Error >
{
  check_switch_preconditions( name, credential_store )?;
  let src = credential_store.join( format!( "{name}.credentials.json" ) );

  // Atomic write: copy to adjacent temp file, then rename into place.
  let creds = paths.credentials_file();
  let tmp = creds.with_extension( "json.tmp" );
  std::fs::copy( &src, &tmp )?;
  std::fs::rename( &tmp, &creds )?;

  // Update active marker after credentials are safely in place.
  let marker = credential_store.join( active_marker_filename() );
  std::fs::write( marker, name )?;

  // Patch live ~/.claude.json and ~/.claude/settings.json from unified {name}.json.
  {
    // Unconditional emailAddress patch — must fire regardless of {name}.json state.
    let live_path = paths.claude_json_file();
    {
      let mut live_val = std::fs::read_to_string( &live_path )
        .ok()
        .and_then( |s| serde_json::from_str::< serde_json::Value >( &s ).ok() )
        .unwrap_or_else( || serde_json::json!( {} ) );
      if let Some( obj ) = live_val.as_object_mut()
      {
        let oauth = obj.entry( "oauthAccount" )
          .or_insert_with( || serde_json::json!( {} ) );
        if let Some( oa_obj ) = oauth.as_object_mut()
        {
          oa_obj.insert( "emailAddress".to_string(), serde_json::Value::String( name.to_string() ) );
        }
      }
      let _ = std::fs::write( &live_path, serde_json::to_string_pretty( &live_val ).map( | s | s + "\n" ).unwrap_or_default() );
    }

    let meta_path = credential_store.join( format!( "{name}.json" ) );
    let meta_text = std::fs::read_to_string( &meta_path ).unwrap_or_default();

    // Restore oauthAccount into live ~/.claude.json (surgical patch — preserves machine-global keys).
    if let Ok( saved_val ) = serde_json::from_str::< serde_json::Value >( &meta_text )
    {
      if let Some( mut oauth ) = saved_val.get( "oauthAccount" ).cloned()
      {
        // Fix(BUG-217): enforce emailAddress == name — snapshot may contain stale email.
        if let Some( oa_obj ) = oauth.as_object_mut()
        {
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
        let live_path = paths.claude_json_file();
        let mut live_val = std::fs::read_to_string( &live_path )
          .ok()
          .and_then( |s| serde_json::from_str::< serde_json::Value >( &s ).ok() )
          .unwrap_or_else( || serde_json::json!( {} ) );
        if let Some( obj ) = live_val.as_object_mut()
        {
          obj.insert( "oauthAccount".to_string(), oauth );
        }
        let _ = std::fs::write( live_path, serde_json::to_string_pretty( &live_val ).map( | s | s + "\n" ).unwrap_or_default() );
      }
    }

    // Restore model preference into live ~/.claude/settings.json.
    let model = parse_string_field( &meta_text, "model" );
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
    let _ = std::fs::write( live_settings_path, serde_json::to_string_pretty( &live_settings ).map( | s | s + "\n" ).unwrap_or_default() );
  }

  Ok( () )
}

/// Override the session model to Opus in `~/.claude/settings.json` when the current model is Sonnet.
///
/// Returns `true` when the override was written (current model was Sonnet or absent);
/// `false` when the model was already non-Sonnet (Opus, Haiku, etc.) — no write occurs.
///
/// Best-effort: any I/O failure is silently ignored (same policy as the `switch_account`
/// model-restore block — `settings.json` mutations must never fail the caller).
///
/// # Fix(BUG-225)
///
/// `switch_account()` restores the snapshot model unconditionally, ignoring current quota.
/// When Sonnet quota is low (< 20%), this leaves the session on Sonnet even though
/// `resolve_model(auto)` would have selected Opus. This function corrects the session model
/// after the switch, keeping it consistent with the subprocess model threshold.
///
/// # Pitfall
///
/// Only fires when quota data is available (i.e., `touch_ctx` is `Some`). When the quota
/// fetch returns 429 (`touch_ctx = None`), the model-aware upgrade cannot fire and the
/// snapshot model is used as-is. See BUG-226 for the documented limitation.
#[ must_use ]
#[ inline ]
pub fn override_session_model_to_opus( paths : &ClaudePaths ) -> bool
{
  let path = paths.settings_file();
  let mut live = std::fs::read_to_string( &path )
    .ok()
    .and_then( | s | serde_json::from_str::< serde_json::Value >( &s ).ok() )
    .unwrap_or_else( || serde_json::json!( {} ) );
  let Some( obj ) = live.as_object_mut() else { return false; };
  let current = obj.get( "model" ).and_then( | v | v.as_str() ).unwrap_or( "" );
  // Fix(BUG-257): exact match "claude-sonnet-4-6" missed shorthand "sonnet" — Claude Code
  //   writes shorthand to settings.json; full-ID check never matched production values.
  //   Write "opus" shorthand (not "claude-opus-4-6") to match Claude Code convention.
  // Root cause: BUG-225 fix used full model IDs; Claude Code stores shorthand in settings.json.
  // Pitfall: contains("sonnet") matches both "sonnet" shorthand and "claude-sonnet-4-6" full ID.
  // Fix(BUG-286): full-ID "claude-opus-4-6" was not covered by contains("sonnet") gate;
  //   `.account.use` wrote the full-ID form when re-applying model override, leaving settings.json
  //   stuck on "claude-opus-4-6" across account switches instead of re-normalising to "opus".
  // Root cause: gate only handled shorthand → shorthand normalisation; full-ID → shorthand
  //   normalisation was a missing arm.
  // Pitfall: both "opus" shorthand and "claude-opus-4-6" full-ID mean opus; the gate must
  //   treat them as equivalent to avoid skipping re-normalisation when full-ID is present.
  if current.contains( "sonnet" ) || current == "claude-opus-4-8" || current == "claude-opus-4-6" || current.is_empty()
  {
    obj.insert( "model".to_string(), serde_json::Value::String( "opus".to_string() ) );
    let _ = std::fs::write( path, serde_json::to_string_pretty( &live ).map( | s | s + "\n" ).unwrap_or_default() );
    true
  }
  else
  {
    false
  }
}

/// Override the session model to `"sonnet"` in `~/.claude/settings.json`.
///
/// Called by `apply_model_override()` when Sonnet 7d utilization is at or above the exhaustion
/// threshold — restores the session model to Sonnet when quota allows.
///
/// Gate: only writes when the current model contains `"opus"`, equals the full-ID form
/// `"claude-sonnet-5"` or legacy `"claude-sonnet-4-6"` (shorthand normalization), or is empty.
/// Returns `true` when the file was updated, `false` when the model was already `"sonnet"`.
///
/// Mirrors `override_session_model_to_opus()` in the reverse direction.
///
/// # Fix(BUG-311)
/// Root cause: `apply_model_override()` had no sonnet-restoration path; `settings.json`
///   retained `"opus"` after switching to an account with sufficient Sonnet quota.
/// Pitfall: write `"sonnet"` shorthand (not `"claude-sonnet-5"`) — Claude Code stores shorthand.
#[ must_use ]
#[ inline ]
pub fn override_session_model_to_sonnet( paths : &ClaudePaths ) -> bool
{
  let path = paths.settings_file();
  let mut live = std::fs::read_to_string( &path )
    .ok()
    .and_then( | s | serde_json::from_str::< serde_json::Value >( &s ).ok() )
    .unwrap_or_else( || serde_json::json!( {} ) );
  let Some( obj ) = live.as_object_mut() else { return false; };
  let current = obj.get( "model" ).and_then( | v | v.as_str() ).unwrap_or( "" );
  if current.contains( "opus" ) || current == "claude-sonnet-5" || current == "claude-sonnet-4-6" || current.is_empty()
  {
    obj.insert( "model".to_string(), serde_json::Value::String( "sonnet".to_string() ) );
    let _ = std::fs::write( path, serde_json::to_string_pretty( &live ).map( | s | s + "\n" ).unwrap_or_default() );
    true
  }
  else
  {
    false
  }
}

/// Write an explicit session model to `~/.claude/settings.json`.
///
/// `model_id` is the full model string (e.g., `"claude-opus-4-8"`).
/// Pass `None` to remove the `model` key (revert to Claude Code default).
/// Creates `~/.claude/` if it does not exist — ensures the write succeeds
/// in environments where Claude Code has not yet initialised the directory.
/// Any remaining I/O failure is silently ignored (best-effort policy).
///
/// # Fix(BUG-258)
/// Root cause: the prior implementation called `fs::write` without first creating
///   the parent directory. When `~/.claude/` was absent (fresh home, test isolation),
///   `fs::write` failed with `NotFound` and the `let _` discarded the error, silently
///   leaving `settings.json` unwritten — violating AC-01/AC-02/AC-03 for the `.usage` path.
/// Pitfall: the `.account.use` path was unaffected because `switch_account` always
///   writes `.credentials.json` to `~/.claude/`, creating the directory first. The
///   `.usage` path had no such pre-condition, making the failure path-specific.
#[ inline ]
pub fn set_session_model( paths : &ClaudePaths, model_id : Option< &str > )
{
  let path = paths.settings_file();
  // Ensure the parent directory exists before writing (Fix(BUG-258)).
  if let Some( parent ) = path.parent() { let _ = std::fs::create_dir_all( parent ); }
  let mut live = std::fs::read_to_string( &path )
    .ok()
    .and_then( | s | serde_json::from_str::< serde_json::Value >( &s ).ok() )
    .unwrap_or_else( || serde_json::json!( {} ) );
  let Some( obj ) = live.as_object_mut() else { return; };
  match model_id
  {
    Some( id ) => { obj.insert( "model".to_string(), serde_json::Value::String( id.to_string() ) ); }
    None       => { obj.remove( "model" ); }
  }
  let _ = std::fs::write( path, serde_json::to_string_pretty( &live ).map( | s | s + "\n" ).unwrap_or_default() );
}

/// Read the current session model from `~/.claude/settings.json`.
///
/// Returns `Some(model)` when `settings.json` exists and contains a `"model"` key;
/// `None` when the file is absent, unparseable, or the `"model"` key is missing.
#[ must_use ]
#[ inline ]
pub fn get_session_model( paths : &ClaudePaths ) -> Option< String >
{
  let content = std::fs::read_to_string( paths.settings_file() ).ok()?;
  parse_string_field( &content, "model" )
}

/// Write the session effort level to `~/.claude/settings.json`.
///
/// Performs a read-modify-write preserving all existing JSON keys (same pattern as
/// `set_session_model()`). Creates `~/.claude/` if the directory is absent.
/// Any I/O failure is silently ignored (best-effort policy).
///
/// Called by the `.usage rotate::1` dispatcher (Feature 062, AC-06) to carry forward
/// the effort level after an account switch.
#[ inline ]
pub fn set_session_effort( paths : &ClaudePaths, effort_id : &str )
{
  let path = paths.settings_file();
  if let Some( parent ) = path.parent() { let _ = std::fs::create_dir_all( parent ); }
  let mut live = std::fs::read_to_string( &path )
    .ok()
    .and_then( |s| serde_json::from_str::< serde_json::Value >( &s ).ok() )
    .unwrap_or_else( || serde_json::json!( {} ) );
  let Some( obj ) = live.as_object_mut() else { return; };
  obj.insert( "effortLevel".to_string(), serde_json::Value::String( effort_id.to_string() ) );
  let _ = std::fs::write( path, serde_json::to_string_pretty( &live ).map( |s| s + "\n" ).unwrap_or_default() );
}

/// Read the current effort level from `~/.claude/settings.json`.
///
/// Returns `Some(effort)` when `settings.json` exists and contains an `"effortLevel"` key;
/// `None` when the file is absent, unparseable, or the `"effortLevel"` key is missing.
/// `effortLevel` may be updated by `.usage rotate::1` (Feature 062, AC-06) to carry forward
/// the effort level after an account switch.
#[ must_use ]
#[ inline ]
pub fn get_session_effort( paths : &ClaudePaths ) -> Option< String >
{
  let content = std::fs::read_to_string( paths.settings_file() ).ok()?;
  parse_string_field( &content, "effortLevel" )
}

/// Validate that a named account can be deleted (name valid + file exists).
///
/// Called by both `delete` and the CLI dry-run path so that dry-run
/// reports the same errors as a live delete.
///
/// # Errors
///
/// Returns `NotFound` if the account does not exist.
#[ inline ]
pub fn check_delete_preconditions( name : &str, credential_store : &Path ) -> Result< (), std::io::Error >
{
  validate_name( name )?;

  let target = credential_store.join( format!( "{name}.credentials.json" ) );
  if !target.exists()
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::NotFound,
      format!( "account '{name}' not found in {}", credential_store.display() ),
    ) );
  }

  Ok( () )
}

/// Delete a named account from `credential_store`.
///
/// Removes `{name}.credentials.json` and `{name}.json` (unified metadata),
/// plus any legacy satellite files from the pre-consolidation layout.
/// Clears the `_active` marker if it points at the deleted account.
///
/// # Errors
///
/// Returns `NotFound` if the account does not exist.
#[ inline ]
pub fn delete( name : &str, credential_store : &Path ) -> Result< (), std::io::Error >
{
  check_delete_preconditions( name, credential_store )?;
  std::fs::remove_file( credential_store.join( format!( "{name}.credentials.json" ) ) )?;
  let _ = std::fs::remove_file( credential_store.join( format!( "{name}.json" ) ) );
  // Clean up legacy satellite files from pre-consolidation layout.
  let _ = std::fs::remove_file( credential_store.join( format!( "{name}.claude.json" ) ) );
  let _ = std::fs::remove_file( credential_store.join( format!( "{name}.settings.json" ) ) );
  let _ = std::fs::remove_file( credential_store.join( format!( "{name}.roles.json" ) ) );
  let _ = std::fs::remove_file( credential_store.join( format!( "{name}.profile.json" ) ) );
  if read_active_marker( credential_store ).as_deref() == Some( name )
  {
    let _ = std::fs::remove_file( credential_store.join( active_marker_filename() ) );
  }
  Ok( () )
}

/// Obtain refreshed OAuth credentials for `name` via an isolated subprocess.
///
/// `Some(paths)` branch: read credentials → `run_isolated`
///   → write live creds → `save` → return `Some(new_creds_json)`.
/// `None` branch: read persistent-store creds → `run_isolated` → write back.
///
/// Returns `None` on any failure — any step failing short-circuits the refresh.
/// Never panics.
///
/// When `trace` is `true`, one `[trace] {label}  {name}  …` line is written to
/// stderr at each key step: `read credentials` result, `run_isolated` invocation,
/// `run_isolated` outcome (including whether credentials were updated),
/// `write credentials` result (only when `run_isolated` returns credentials), and
/// `save` result (`Some(paths)` branch only, only when the write succeeded).
/// Failure-path lines include the error string.
///
/// # Consumer Crate Note
///
/// Gated on `#[cfg(feature = "enabled")]`. Consumer crates whose workspace dep on
/// `claude_profile_core` has `default-features = false` must explicitly add
/// `features = ["enabled"]` to their dep declaration — without it this function
/// compiles away at call sites.
#[ cfg( feature = "enabled" ) ]
#[ inline ]
#[ must_use = "None means the refresh failed — caller must handle the missing credentials case" ]
pub fn refresh_account_token(
  name             : &str,
  credential_store : &Path,
  paths            : Option< &ClaudePaths >,
  trace            : bool,
  label            : &str,
  model            : claude_runner_core::IsolatedModel,
  extra_pre_args   : &[ String ],
) -> Option< String >
{
  // Fix(BUG-205): read credentials: OK and write credentials: OK trace lines were missing
  // Root cause: Ok(s) => s bare arms had no instrumentation; only Err arms emitted trace
  // Pitfall: multi-step lifecycle functions must instrument BOTH Ok and Err arms per AC-26
  // Fix(issue-166): added `trace: bool` param; all `?` operators replaced with explicit `match` + `eprintln!` blocks.

  // Fix(issue-169): corrected issue-168 regression — empty args (vec\![]) broken; correct args are `--print .`.
  // Root cause (166): function had no `trace` param so `apply_refresh`'s `trace` flag could not propagate
  //   into it; every failure step (switch_account, file read, run_isolated,
  //   save) returned `None` silently — `clp .usage refresh::1 trace::1` produced no diagnostic signal.
  // Root cause (169): issue-168 misdiagnosed issue-151's root cause as `--print` mode itself being broken.
  //   The real culprit in issue-151 was `--max-tokens 1`: it triggers an API error response (not 401)
  //   before OAuth token refresh can happen, so credentials are never rewritten.
  //   issue-168's "fix" swapped to empty args (vec\![]) instead, which also breaks: Claude Code in non-TTY
  //   mode with no args exits immediately without performing startup OAuth token refresh at all.
  //   `--print .` alone is correct: Claude performs OAuth token refresh at startup before the API call;
  //   the API call to `.` either succeeds or times out, but creds are written regardless.
  //   (The `issue-isolated-credentials-on-timeout` fix in `isolated.rs` captures creds even on timeout.)
  // Pitfall: (a) `--print .` (no `--max-tokens`) is the only working isolated-refresh invocation:
  //   empty args → immediate exit without OAuth refresh in non-TTY mode;
  //   `--max-tokens 1` → API rejection before refresh path; `--print .` → startup refresh + API call.
  //   (b) carry all cross-cutting params (`trace`, error context) into extracted functions — silent `?`
  //   propagation becomes a diagnostic black hole.

  // TSK-191: extra_pre_args (e.g. ["--effort", "high"]) are prepended before ["--print", "."].
  let mut args : Vec< String > = extra_pre_args.to_vec();
  args.push( "--print".to_string() );
  args.push( ".".to_string() );

  if let Some( p ) = paths
  {
    refresh_token_with_live_path( name, credential_store, p, trace, label, model, args, claude_runner_core::run_isolated )
  }
  else
  {
    let path = credential_store.join( format!( "{name}.credentials.json" ) );
    let creds_json = match std::fs::read_to_string( &path )
    {
      Ok( s )  => { if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  read credentials: OK", trace_ts() ); } s }
      Err( e ) =>
      {
        if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  read credentials: Err({e})", trace_ts() ); }
        return None;
      }
    };
    // AC-32 (Change A): set expiresAt=1 in the in-memory copy to force RT rotation.
    // The stored credential file is NOT modified — only the transient copy passed to run_isolated.
    let creds_json = manipulate_expires_at( &creds_json );
    let t_run = std::time::Instant::now();
    if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  run_isolated: invoking claude  args={args:?}  timeout=35s", trace_ts() ); }
    let isolated = match claude_runner_core::run_isolated( &creds_json, args, 35, model )
    {
      Ok( r )  => r,
      Err( e ) =>
      {
        if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  run_isolated: Err({e})  ({:.1}s)", trace_ts(), t_run.elapsed().as_secs_f64() ); }
        return None;
      }
    };
    if trace
    {
      let creds_status = if isolated.credentials.is_some() { "Some" } else { "None" };
      let _ = writeln!( std::io::stderr(), "{}{label}  {name}  run_isolated: OK credentials={creds_status}  ({:.1}s)", trace_ts(), t_run.elapsed().as_secs_f64() );
    }
    let new_creds = isolated.credentials?;
    if let Err( e ) = std::fs::write( &path, &new_creds )
    {
      if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  write credentials: Err({e})", trace_ts() ); }
      return None;
    }
    if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  write credentials: OK", trace_ts() ); }
    Some( new_creds )
  }
}

// Inner implementation for the `Some(paths)` branch of `refresh_account_token`.
// Handles live-credential pre-sync (Change B / AC-33) and delegates to run_isolated.
// Kept separate to stay within the line-count limit for the public function.
#[ cfg( feature = "enabled" ) ]
#[ allow( clippy::too_many_arguments ) ] // 8th param `run_isolated_fn` added by BUG-316 — test seam; all args are independent concerns.
fn refresh_token_with_live_path(
  name             : &str,
  credential_store : &Path,
  p                : &ClaudePaths,
  trace            : bool,
  label            : &str,
  model            : claude_runner_core::IsolatedModel,
  args             : Vec< String >,
  run_isolated_fn  : impl Fn( &str, Vec< String >, u64, claude_runner_core::IsolatedModel ) -> Result< claude_runner_core::IsolatedRunResult, claude_runner_core::RunnerError >,
) -> Option< String >
{
  // Fix(BUG-175): removed switch_account call — credentials read directly from credential store
  // Root cause: Some(paths) branch read via p.credentials_file() forcing switch_account to populate it;
  //   run_isolated creates its own temp HOME and never reads ~/.claude/, so the write was redundant
  // Pitfall: switch_account before a read looks like defensive initialization;
  //   the unnecessary global write is only visible in concurrent multi-account batch scenarios
  let creds_json = match std::fs::read_to_string( credential_store.join( format!( "{name}.credentials.json" ) ) )
  {
    Ok( s )  => { if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  read credentials: OK", trace_ts() ); } s }
    Err( e ) =>
    {
      if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  read credentials: Err({e})", trace_ts() ); }
      return None;
    }
  };
  // AC-33 (Change B) pre-sync: if the live session already refreshed, sync without subprocess.
  // Avoids a redundant run_isolated call when ~/.claude/.credentials.json has a fresher RT pair.
  // Guard: only valid when name IS the currently active account. For non-current accounts the
  // live file holds a different account's credentials — comparing against name's store would
  // falsely treat the current session's creds as a "fresh" RT pair for name and corrupt the
  // store by overwriting name's credentials with the current session's credentials.
  // Pitfall: apply_touch calls refresh_account_token for ALL accounts (including non-current)
  // during the pre-rotation touch loop; attempt_expired_token_refresh calls it for the TARGET
  // account before switch_account — in both cases name is NOT yet the active account.
  // Fix(BUG-316): re-read the active marker independently at each use site.
  // Root cause: is_active was computed once before run_isolated and reused 35s later in
  //   the race-recovery block; a concurrent switch_account("B") during the subprocess
  //   window changed the marker to "B", but the stale cached bool caused live credentials
  //   (now holding B's creds post-switch) to be written into A's credential store slot.
  // Pitfall: never cache a filesystem-derived boolean across a blocking call (subprocess,
  //   network I/O) in a multi-process environment — re-read at each use site instead.
  let is_active_pre_sync = {
    let marker = credential_store.join( active_marker_filename() );
    std::fs::read_to_string( &marker ).ok().is_some_and( |s| s.trim() == name )
  };
  if is_active_pre_sync
  {
    if let Ok( live_json ) = std::fs::read_to_string( p.credentials_file() )
    {
      if live_json.trim() != creds_json.trim()
      {
        let store_path = credential_store.join( format!( "{name}.credentials.json" ) );
        if std::fs::write( &store_path, &live_json ).is_ok()
        {
          let _ = save( name, credential_store, p, false, Some( live_json.as_bytes() ), None, None, None );
          return Some( live_json );
        }
      }
    }
  }
  // AC-32 (Change A): set expiresAt=1 in the in-memory copy to force RT rotation.
  // The stored credential file is NOT modified — only the transient copy passed to run_isolated.
  let creds_json = manipulate_expires_at( &creds_json );
  let t_run = std::time::Instant::now();
  if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  run_isolated: invoking claude  args={args:?}  timeout=35s", trace_ts() ); }
  let isolated = match run_isolated_fn( &creds_json, args, 35, model )
  {
    Ok( r )  => r,
    Err( e ) =>
    {
      if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  run_isolated: Err({e})  ({:.1}s)", trace_ts(), t_run.elapsed().as_secs_f64() ); }
      return None;
    }
  };
  if trace
  {
    let creds_status = if isolated.credentials.is_some() { "Some" } else { "None" };
    let _ = writeln!( std::io::stderr(), "{}{label}  {name}  run_isolated: OK credentials={creds_status}  ({:.1}s)", trace_ts(), t_run.elapsed().as_secs_f64() );
  }
  // Fix(BUG-221): write refreshed credentials directly to the credential store, not to
  //   p.credentials_file() (the live session file ~/.claude/.credentials.json).
  // Root cause: BUG-175's fix (TSK-208) removed switch_account() but left the write to the
  //   live file intact; every batch refresh call clobbered the active session credentials.
  // Pitfall: save() is called with Some(&new_creds) so it writes from bytes directly,
  //   bypassing the copy-from-live-file path that would copy now-stale credentials.
  let Some( new_creds ) = isolated.credentials else
  {
    // AC-33 (Change B) race recovery: run_isolated returned credentials=None.
    // A concurrent live session may have refreshed during the subprocess call.
    // Fix(BUG-316): re-read the active marker here — not the cached value from function
    //   entry. The 35-second run_isolated window allows switch_account("B") to change the
    //   marker; the stale cached bool would write B's live credentials into A's store slot.
    let is_active_now = {
      let marker = credential_store.join( active_marker_filename() );
      std::fs::read_to_string( &marker ).ok().is_some_and( |s| s.trim() == name )
    };
    if is_active_now
    {
      let orig_stored = std::fs::read_to_string(
        credential_store.join( format!( "{name}.credentials.json" ) ),
      ).unwrap_or_default();
      if let Ok( live_json ) = std::fs::read_to_string( p.credentials_file() )
      {
        if live_json.trim() != orig_stored.trim()
        {
          let store_path = credential_store.join( format!( "{name}.credentials.json" ) );
          if std::fs::write( &store_path, &live_json ).is_ok()
          {
            let _ = save( name, credential_store, p, false, Some( live_json.as_bytes() ), None, None, None );
            return Some( live_json );
          }
        }
      }
    }
    return None;
  };
  let store_cred_path = credential_store.join( format!( "{name}.credentials.json" ) );
  if let Err( e ) = std::fs::write( &store_cred_path, &new_creds )
  {
    if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  write credentials: Err({e})", trace_ts() ); }
    return None;
  }
  if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  write credentials: OK", trace_ts() ); }
  // Pass owner: None — background refresh must not mutate the owner field.
  match save( name, credential_store, p, false, Some( new_creds.as_bytes() ), None, None, None )
  {
    Ok( () ) => { if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  save: OK", trace_ts() ); } }
    Err( e ) =>
    {
      if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  save: Err({e})", trace_ts() ); }
      return None;
    }
  }
  // Fix(BUG-318): post-rotation live sync for the currently active account.
  // Root cause: run_isolated rotates credentials, writing AT_new+RT_new to STORE only;
  //   LIVE (~/.claude/.credentials.json) retains AT_old (now revoked by Anthropic). A
  //   subsequent .account.save reads LIVE and copies it to STORE, overwriting the freshly-
  //   rotated credentials with the revoked ones. The account is then permanently broken —
  //   the revoked RT cannot be used to recover via token refresh.
  // Pitfall: re-read the active marker here — same rationale as is_active_now in the
  //   credentials=None recovery branch (Fix(BUG-316)). The 35s subprocess window allows a
  //   concurrent switch_account call to change the active account; a stale bool would write
  //   the wrong credentials to LIVE.
  let is_still_active = {
    let marker = credential_store.join( active_marker_filename() );
    std::fs::read_to_string( &marker ).ok().is_some_and( |s| s.trim() == name )
  };
  if is_still_active
  {
    let _ = std::fs::write( p.credentials_file(), &new_creds );
    if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  write live: OK", trace_ts() ); }
  }
  Some( new_creds )
}

/// Replace the `expiresAt` value in a credentials JSON string with `1`.
///
/// # Purpose (AC-32 / Change A)
///
/// Forces the Claude CLI subprocess (`run_isolated`) to treat the access token as
/// expired on every call, so it uses the stored refresh token to obtain a fresh
/// AT+RT pair. This rotates the refresh token on every invocation, preventing the
/// silent RT decay that rendered account i5 irrecoverable.
///
/// # Contract
///
/// - Input is a raw JSON string from a credentials file.
/// - If `"expiresAt":DIGITS` (bare numeric) is found, it is replaced with `"expiresAt":1`.
/// - If `"expiresAt":"DIGITS"` (quoted string) is found, it is replaced with `"expiresAt":"1"`.
/// - If neither pattern is present, the string is returned unchanged.
/// - Negative values (e.g. `"expiresAt":-1`) are not matched — treated as absent.
/// - Only the in-memory copy is modified; the on-disk credential file is NEVER touched.
///
/// # Pitfall
///
/// Do NOT pass the return value to `std::fs::write` — that would corrupt the stored
/// credentials. Only pass it to `run_isolated` as the transient in-process credential JSON.
#[ must_use ]
#[ inline ]
pub fn manipulate_expires_at( creds_json : &str ) -> String
{
  // Try bare numeric first (most common format): "expiresAt":DIGITS
  if let Some( start ) = creds_json.find( "\"expiresAt\":" )
  {
    let after_key = &creds_json[ start + "\"expiresAt\":".len().. ];
    // Quoted value: "expiresAt":"DIGITS"
    if let Some( inner ) = after_key.strip_prefix( '"' )
    {
      if let Some( end ) = inner.find( '"' )
      {
        let old_val = &after_key[ ..end + 2 ]; // includes surrounding quotes
        return creds_json.replacen(
          &format!( "\"expiresAt\":{old_val}" ),
          "\"expiresAt\":\"1\"",
          1,
        );
      }
    }
    else
    {
      // Bare numeric value: ends at first non-digit character
      let end = after_key.find( | c : char | !c.is_ascii_digit() ).unwrap_or( after_key.len() );
      let old_val = &after_key[ ..end ];
      if !old_val.is_empty()
      {
        return creds_json.replacen(
          &format!( "\"expiresAt\":{old_val}" ),
          "\"expiresAt\":1",
          1,
        );
      }
    }
  }
  creds_json.to_string()
}

/// Return the filename for the per-machine active-account marker.
///
/// Format: `` `_active_{hostname}_{user}` `` where `hostname` and `user` are
/// sanitized (only alphanumeric, `-`, and `.` are kept; everything else becomes `_`).
/// Reads `HOSTNAME` env var first, falls back to `/etc/hostname`; reads `USER`
/// Resolves the current machine's hostname via fallback chain:
/// `$HOSTNAME` env → `/etc/hostname` → `"local"`.
#[ inline ]
#[ must_use ]
pub fn resolve_hostname() -> String
{
  std::env::var( "HOSTNAME" )
    .unwrap_or_else( |_|
    {
      std::fs::read_to_string( "/etc/hostname" )
        .unwrap_or_else( |_| "local".to_string() )
        .trim()
        .to_string()
    } )
}

/// Return the `"USER@hostname"` identity for the current machine.
///
/// Used as the `owner` value written by `.account.save`. Shares the same
/// fallback chain as [`resolve_hostname`]: `$USER` → `$USERNAME` → `"user"`,
/// and `$HOSTNAME` → `/etc/hostname` → `"local"`.
#[ inline ]
#[ must_use ]
pub fn current_identity() -> String
{
  let user = std::env::var( "USER" )
    .or_else( |_| std::env::var( "USERNAME" ) )
    .unwrap_or_else( |_| "user".to_string() );
  let hostname = resolve_hostname();
  format!( "{user}@{hostname}" )
}

/// Read the `owner` field from `{name}.json` in `credential_store`.
///
/// Returns an empty string when the file is absent, unparseable, or the
/// `owner` field is missing — identical behaviour to "no owner" (all gates pass).
#[ inline ]
#[ must_use ]
pub fn read_owner( credential_store : &Path, name : &str ) -> String
{
  let path = credential_store.join( format!( "{name}.json" ) );
  std::fs::read_to_string( &path ).ok()
    .and_then( |s| parse_string_field( &s, "owner" ) )
    .unwrap_or_default()
}

/// Return `true` when `owner` represents "no enforcement" for the current machine.
///
/// - Empty string → unowned (all gates pass).
/// - Matches `current_identity()` → owned by this machine (gates pass).
/// - Any other non-empty string → owned by a different machine (gates block).
#[ inline ]
#[ must_use ]
pub fn is_owned( owner : &str ) -> bool
{
  owner.is_empty() || owner == current_identity()
}

/// Write the `owner` field to `{name}.json` via read-merge.
///
/// Reads the existing `{name}.json` (if any), sets `owner` to the given value,
/// and writes back. All non-`owner` fields are preserved.
/// Does NOT touch `{name}.credentials.json` or any `~/.claude.*` file.
///
/// # Errors
///
/// Returns `std::io::Error` if the JSON file cannot be written.
#[ inline ]
pub fn write_owner(
  name             : &str,
  credential_store : &Path,
  owner            : &str,
) -> Result< (), std::io::Error >
{
  let path = credential_store.join( format!( "{name}.json" ) );
  let mut map = std::fs::read_to_string( &path )
    .ok()
    .and_then( |s| serde_json::from_str::< serde_json::Value >( &s ).ok() )
    .and_then( |v| v.as_object().cloned() )
    .unwrap_or_default();
  map.insert( "owner".to_string(), serde_json::Value::String( owner.to_string() ) );
  let json = serde_json::to_string_pretty( &serde_json::Value::Object( map ) )
    .map( | s | s + "\n" )
    .map_err( |e| std::io::Error::new( std::io::ErrorKind::InvalidData, e ) )?;
  std::fs::write( &path, json )
}

/// env var first, falls back to `USERNAME`, then to the literal `"user"`.
///
/// The per-machine name means that switching accounts on one machine does not
/// affect other machines sharing the same credential store via version control.
/// Add `` `_active_*` `` to `.gitignore` to prevent these files from being tracked.
#[ inline ]
#[ must_use ]
pub fn active_marker_filename() -> String
{
  let hostname = resolve_hostname();
  let user = std::env::var( "USER" )
    .or_else( |_| std::env::var( "USERNAME" ) )
    .unwrap_or_else( |_| "user".to_string() );
  let clean = | s : &str | -> String
  {
    s.chars()
      .map( | c | if c.is_alphanumeric() || c == '-' || c == '.' { c } else { '_' } )
      .collect()
  };
  format!( "_active_{}_{}", clean( &hostname ), clean( &user ) )
}

/// Returns the set of account names that are marked as active on other machines.
///
/// Reads every `_active_*` file in `credential_store` except the current
/// machine's own marker (as returned by [`active_marker_filename`]). Each
/// such file contains the name of the account active on that other machine.
/// Returns the collected names as a `HashSet` so callers can check membership
/// in O(1).
///
/// Missing or unreadable files are silently skipped (another machine's marker
/// may not be present locally at all times).
#[ inline ]
#[ must_use ]
pub fn other_machines_active( credential_store : &Path ) -> std::collections::HashSet< String >
{
  let own = active_marker_filename();
  std::fs::read_dir( credential_store )
    .ok()
    .into_iter()
    .flatten()
    .filter_map( Result::ok )
    .filter( | e |
    {
      let name = e.file_name();
      let n = name.to_string_lossy();
      n.starts_with( "_active_" ) && n != own.as_str()
    } )
    .filter_map( | e | std::fs::read_to_string( e.path() ).ok() )
    .map( | s | s.trim().to_string() )
    .filter( | s | !s.is_empty() )
    .collect()
}

// ── Account renewal ───────────────────────────────────────────────────────────

/// The operation to apply to `_renewal_at` in `{name}.json`.
#[ derive( Debug ) ]
pub enum RenewalOperation
{
  /// Set `_renewal_at` to the given ISO-8601 UTC string (stored verbatim).
  At( String ),
  /// Remove `_renewal_at` from the file.
  Clear,
}

/// Write or clear a billing renewal timestamp override in `{name}.json`.
///
/// Reads the existing `{name}.json` (or starts with `{}` if absent), applies `op`,
/// and writes back. All other top-level keys (e.g. `oauthAccount`) are preserved.
///
/// When `dry` is `true`, no file is written; returns a `[dry-run]` status line.
///
/// # Errors
///
/// Returns `NotFound` if `{name}.credentials.json` does not exist.
/// Returns I/O errors on file read/write failure.
#[ inline ]
pub fn account_renewal(
  name             : &str,
  credential_store : &Path,
  op               : &RenewalOperation,
  dry              : bool,
) -> Result< String, std::io::Error >
{
  let cred_path = credential_store.join( format!( "{name}.credentials.json" ) );
  if !cred_path.exists()
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::NotFound,
      format!( "account '{name}' not found in {}", credential_store.display() ),
    ) );
  }

  let meta_path    = credential_store.join( format!( "{name}.json" ) );
  let existing_str = std::fs::read_to_string( &meta_path )
    .unwrap_or_else( |_| "{}".to_string() );
  let mut val = serde_json::from_str::< serde_json::Value >( &existing_str )
    .unwrap_or_else( |_| serde_json::json!( {} ) );
  let obj = val.as_object_mut()
    .ok_or_else( || std::io::Error::new(
      std::io::ErrorKind::InvalidData,
      format!( "{name}.json is not a JSON object" ),
    ) )?;

  let status_str = match op
  {
    RenewalOperation::At( ts ) =>
    {
      obj.insert( "_renewal_at".to_string(), serde_json::Value::String( ts.clone() ) );
      format!( "set _renewal_at = {ts}" )
    }
    RenewalOperation::Clear =>
    {
      obj.remove( "_renewal_at" );
      "cleared _renewal_at".to_string()
    }
  };

  if dry
  {
    return Ok( format!( "[dry-run] {name}: would {status_str}\n" ) );
  }

  let new_json = serde_json::to_string_pretty( &val )
    .map( | s | s + "\n" )
    .map_err( |e| std::io::Error::new( std::io::ErrorKind::InvalidData, e.to_string() ) )?;
  std::fs::write( &meta_path, new_json )?;
  Ok( format!( "{name}: {status_str}\n" ) )
}

/// Format a Unix timestamp (seconds since epoch) as an ISO-8601 UTC string.
///
/// Output format: `YYYY-MM-DDTHH:MM:SSZ`. Used by `from_now::` delta computation.
/// Does not depend on chrono.
#[ doc( hidden ) ]
#[ inline ]
#[ must_use ]
pub fn secs_to_iso8601( secs : u64 ) -> String
{
  let sec  = secs % 60;
  let min  = ( secs / 60 ) % 60;
  let hour = ( secs / 3600 ) % 24;
  let days = secs / 86400;

  let mut year  = 1970_u64;
  let mut d_rem = days;
  loop
  {
    let dy = if is_leap( year ) { 366 } else { 365 };
    if d_rem < dy { break; }
    d_rem -= dy;
    year  += 1;
  }

  let month_days : [ u64; 12 ] =
    [ 31, if is_leap( year ) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 ];
  let mut month = 0_usize;
  while month < 12 && d_rem >= month_days[ month ]
  {
    d_rem -= month_days[ month ];
    month += 1;
  }

  format!( "{year:04}-{:02}-{:02}T{hour:02}:{min:02}:{sec:02}Z", month + 1, d_rem + 1 )
}

/// Parse a signed duration string into a signed second count.
///
/// Format: `±Xd Xh Xm` with optional spaces between unit-suffixed numbers.
/// Prefix sign is required: `+` for future, `-` for past.
/// Units: `d` (86400s), `h` (3600s), `m` (60s).
/// Examples: `+1h30m`, `-30m`, `+1d12h`, `+0m`.
///
/// # Errors
///
/// Returns a descriptive `String` on malformed input.
#[ doc( hidden ) ]
#[ inline ]
pub fn parse_from_now_delta( s : &str ) -> Result< i64, String >
{
  let s = s.trim();
  if s.is_empty() { return Err( "from_now:: value is empty".to_string() ); }
  let ( sign, rest ) = match s.chars().next()
  {
    Some( '+' ) => ( 1_i64,  &s[ 1.. ] ),
    Some( '-' ) => ( -1_i64, &s[ 1.. ] ),
    _           => return Err( format!( "from_now:: must start with '+' or '-', got: '{s}'" ) ),
  };
  if rest.trim().is_empty()
  {
    return Err( format!(
      "from_now:: '{s}' has no duration components; expected e.g. +1h, +30m, +1d"
    ) );
  }
  let mut total_secs = 0_i64;
  let mut pos        = 0_usize;
  let bytes          = rest.as_bytes();
  while pos < bytes.len()
  {
    while pos < bytes.len() && bytes[ pos ] == b' ' { pos += 1; }
    if pos >= bytes.len() { break; }
    let num_start = pos;
    while pos < bytes.len() && bytes[ pos ].is_ascii_digit() { pos += 1; }
    if pos == num_start
    {
      return Err( format!( "from_now:: unexpected character '{}' at position {pos}", bytes[ num_start ] as char ) );
    }
    let num : i64 = rest[ num_start..pos ].parse()
      .map_err( |_| "from_now:: numeric overflow".to_string() )?;
    if pos >= bytes.len()
    {
      return Err( format!( "from_now:: missing unit after number {num} (use d, h, or m)" ) );
    }
    match bytes[ pos ]
    {
      b'd' => { total_secs += num * 86400; pos += 1; }
      b'h' => { total_secs += num * 3600;  pos += 1; }
      b'm' => { total_secs += num * 60;    pos += 1; }
      c    => return Err( format!( "from_now:: unknown unit '{}' (supported: d, h, m)", c as char ) ),
    }
  }
  Ok( sign * total_secs )
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn is_leap( y : u64 ) -> bool
{
  ( y % 4 == 0 && y % 100 != 0 ) || y % 400 == 0
}

fn read_active_marker( credential_store : &Path ) -> Option< String >
{
  let marker = credential_store.join( active_marker_filename() );
  std::fs::read_to_string( marker )
    .ok()
    .map( | s | s.trim().to_string() )
}

/// Extract the account name from a `{name}.credentials.json` path.
///
/// Returns `None` for anything that is not a `*.credentials.json` file
/// (e.g. the `_active` marker or unrelated files).
#[ doc( hidden ) ]
#[ must_use ]
#[ inline ]
pub fn credential_stem( path : &Path ) -> Option< String >
{
  let filename = path.file_name()?.to_str()?;
  filename
    .strip_suffix( ".credentials.json" )
    .map( std::string::ToString::to_string )
}

#[ doc( hidden ) ]
#[ inline ]
pub fn validate_name( name : &str ) -> Result< (), std::io::Error >
{
  // Account names must be valid email addresses (local@domain) so they can be
  // used as filenames and unambiguously identify the Claude account owner.
  let at = name.find( '@' ).ok_or_else( || std::io::Error::new(
    std::io::ErrorKind::InvalidInput,
    format!( "account name '{name}' is not a valid email address: must contain '@'" ),
  ) )?;
  if at == 0
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      format!( "account name '{name}' is not a valid email address: local part must not be empty" ),
    ) );
  }
  if name[ at + 1.. ].is_empty()
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      format!( "account name '{name}' is not a valid email address: domain must not be empty" ),
    ) );
  }
  // Fix(issue-123): validate_name() passed names like `a/b@c.com` because it only checked
  //   @-presence and non-empty local/domain parts; the local part was never inspected for
  //   path-unsafe chars, so save()/switch_account() hit filesystem errors (exit 2) instead
  //   of returning InvalidInput (exit 1).
  // Root cause: local-part safety check was absent; chars `/`, `\`, `*` create path
  //   traversal when used as a filename prefix (e.g. `{store}/a/b@c.com.credentials.json`).
  // Pitfall: only the local part (before `@`) needs this check; the domain part appears
  //   after `@` in the filename and cannot create sub-directory traversal in practice.
  let local = &name[ ..at ];
  if local.contains( '/' ) || local.contains( '\\' ) || local.contains( '*' )
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      format!( "account name '{name}' contains path-unsafe characters in the local part" ),
    ) );
  }
  Ok( () )
}

/// Extract a quoted string field from a JSON blob without external dependencies.
///
/// Handles optional whitespace after the colon: both `"key":"val"` and
/// `"key": "val"` forms.
#[ doc( hidden ) ]
#[ must_use ]
#[ inline ]
pub fn parse_string_field( json : &str, key : &str ) -> Option< String >
{
  let search = format!( "\"{key}\":" );
  let colon_end = json.find( &search )? + search.len();
  let rest = json[ colon_end.. ].trim_start();
  if !rest.starts_with( '"' ) { return None; }
  let inner = &rest[ 1.. ];
  let end = inner.find( '"' )?;
  Some( inner[ ..end ].to_string() )
}

/// Extract an unsigned integer field from a JSON blob without external dependencies.
///
/// Handles optional whitespace after the colon.
#[ doc( hidden ) ]
#[ must_use ]
#[ inline ]
pub fn parse_u64_field( json : &str, key : &str ) -> Option< u64 >
{
  let search = format!( "\"{key}\":" );
  let colon_end = json.find( &search )? + search.len();
  let rest = json[ colon_end.. ].trim_start();
  let end = rest
    .find( | c : char | !c.is_ascii_digit() )
    .unwrap_or( rest.len() );
  if end == 0 { return None; }
  rest[ ..end ].parse().ok()
}

/// Extract a string array field from a JSON blob without external dependencies.
///
/// Handles optional whitespace after the colon. Returns an empty `Vec` when
/// the key is absent, the value is not an array, or no quoted strings are found.
#[ doc( hidden ) ]
#[ must_use ]
#[ inline ]
pub fn parse_string_array_field( json : &str, key : &str ) -> Vec< String >
{
  let search    = format!( "\"{key}\":" );
  let colon_end = match json.find( &search )
  {
    Some( p ) => p + search.len(),
    None      => return Vec::new(),
  };
  let rest = json[ colon_end.. ].trim_start();
  if !rest.starts_with( '[' ) { return Vec::new(); }
  let end = match rest[ 1.. ].find( ']' )
  {
    Some( p ) => 1 + p,
    None      => return Vec::new(),
  };
  let inner = &rest[ 1..end ];
  let mut values = Vec::new();
  let mut pos    = 0_usize;
  while pos < inner.len()
  {
    let Some( q_start ) = inner[ pos.. ].find( '"' ) else { break };
    let start_val = pos + q_start + 1;
    let Some( q_end ) = inner[ start_val.. ].find( '"' ) else { break };
    let end_val = start_val + q_end;
    values.push( inner[ start_val..end_val ].to_string() );
    pos = end_val + 1;
  }
  values
}

/// Read the OAuth `accessToken` field from a credential JSON file.
///
/// Shared base for both the usage quota fetch path and the credential-read
/// path in command handlers — avoids duplicating file-read + field-extract
/// logic across two callers.
///
/// Returns `Ok(token)` on success.
/// Returns `Err(reason)` on I/O failure or missing / empty `accessToken` field.
#[ doc( hidden ) ]
#[ inline ]
pub fn read_access_token_from_file( path : &std::path::Path ) -> Result< String, String >
{
  let content = std::fs::read_to_string( path )
    .map_err( |e| format!( "cannot read credentials: {e}" ) )?;
  parse_string_field( &content, "accessToken" )
    .ok_or_else( || "missing accessToken".to_string() )
}

// ── Quota cache ──────────────────────────────────────────────────────────────

/// Cached quota entry read from `{name}.json` `"cache"` key.
#[ derive( Debug ) ]
pub struct QuotaCacheEntry
{
  /// UTC ISO-8601 timestamp of the last successful fetch.
  pub fetched_at        : String,
  /// 5h period: (`utilization` 0–100, `resets_at` ISO string or `None`).
  pub five_hour         : Option< ( f64, Option< String > ) >,
  /// 7d period: (`utilization`, `resets_at`).
  pub seven_day         : Option< ( f64, Option< String > ) >,
  /// 7d-sonnet period: (`utilization`, `resets_at`).
  pub seven_day_sonnet  : Option< ( f64, Option< String > ) >,
  /// Persisted model override decision.
  pub model_override    : Option< String >,
  /// Last touch timestamp (UTC ISO-8601).
  pub last_touch_at     : Option< String >,
  /// Whether the account is idle (no active 5h window).
  pub touch_idle        : Option< bool >,
}

/// Write quota cache to `{name}.json` using read-merge-write.
///
/// Persists the last successful fetch result so it can be used as fallback
/// when the usage API is unavailable.  Failures are silently ignored.
#[ inline ]
pub fn write_quota_cache(
  credential_store  : &std::path::Path,
  name              : &str,
  five_hour         : Option< ( f64, Option< &str > ) >,
  seven_day         : Option< ( f64, Option< &str > ) >,
  seven_day_sonnet  : Option< ( f64, Option< &str > ) >,
)
{
  let meta_path = credential_store.join( format!( "{name}.json" ) );
  let mut snapshot = std::fs::read_to_string( &meta_path )
    .ok()
    .and_then( |s| serde_json::from_str::< serde_json::Value >( &s ).ok() )
    .unwrap_or_else( || serde_json::json!( {} ) );
  if let Some( obj ) = snapshot.as_object_mut()
  {
    let now = chrono_now_utc();
    let mut cache = serde_json::json!( { "fetched_at": now, "status": "ok" } );
    if let Some( co ) = cache.as_object_mut()
    {
      if let Some( ( u, r ) ) = five_hour
      {
        co.insert( "five_hour".into(), period_json( u, r ) );
      }
      if let Some( ( u, r ) ) = seven_day
      {
        co.insert( "seven_day".into(), period_json( u, r ) );
      }
      if let Some( ( u, r ) ) = seven_day_sonnet
      {
        co.insert( "seven_day_sonnet".into(), period_json( u, r ) );
      }
      // Preserve model_override, touch state, and measurement history from prior cache.
      // Feature 040: "history" must survive write_quota_cache or every successful fetch
      //   would clobber the stored ring buffer (verification finding F4-3).
      if let Some( prev ) = obj.get( "cache" ).and_then( |c| c.as_object() )
      {
        if let Some( v ) = prev.get( "model_override" ) { co.insert( "model_override".into(), v.clone() ); }
        if let Some( v ) = prev.get( "last_touch_at" )  { co.insert( "last_touch_at".into(), v.clone() ); }
        if let Some( v ) = prev.get( "touch_idle" )     { co.insert( "touch_idle".into(), v.clone() ); }
        if let Some( v ) = prev.get( "history" )        { co.insert( "history".into(), v.clone() ); }
      }
    }
    obj.insert( "cache".to_string(), cache );
  }
  let _ = std::fs::write( &meta_path, serde_json::to_string_pretty( &snapshot ).map( | s | s + "\n" ).unwrap_or_default() );
}

/// Read cached quota from `{name}.json`.
///
/// Returns `None` when the file is absent, unparseable, or has no `"cache"` key.
#[ inline ]
pub fn read_quota_cache( credential_store : &std::path::Path, name : &str ) -> Option< QuotaCacheEntry >
{
  let meta_path = credential_store.join( format!( "{name}.json" ) );
  let text = std::fs::read_to_string( &meta_path ).ok()?;
  let val : serde_json::Value = serde_json::from_str( &text ).ok()?;
  let cache = val.get( "cache" )?.as_object()?;
  let fetched_at = cache.get( "fetched_at" )?.as_str()?.to_string();
  Some( QuotaCacheEntry
  {
    fetched_at,
    five_hour        : read_period( cache, "five_hour" ),
    seven_day        : read_period( cache, "seven_day" ),
    seven_day_sonnet : read_period( cache, "seven_day_sonnet" ),
    model_override   : cache.get( "model_override" ).and_then( |v| v.as_str() ).map( str::to_string ),
    last_touch_at    : cache.get( "last_touch_at" ).and_then( |v| v.as_str() ).map( str::to_string ),
    touch_idle       : cache.get( "touch_idle" ).and_then( serde_json::Value::as_bool ),
  } )
}

/// Write a single field into the cache object in `{name}.json` (read-merge-write).
///
/// Used by model override and touch persistence to update one cache field
/// without clobbering the quota data written by `write_quota_cache`.
#[ inline ]
pub fn write_cache_field(
  credential_store : &std::path::Path,
  name             : &str,
  key              : &str,
  value            : serde_json::Value,
)
{
  let meta_path = credential_store.join( format!( "{name}.json" ) );
  let mut snapshot = std::fs::read_to_string( &meta_path )
    .ok()
    .and_then( |s| serde_json::from_str::< serde_json::Value >( &s ).ok() )
    .unwrap_or_else( || serde_json::json!( {} ) );
  if let Some( obj ) = snapshot.as_object_mut()
  {
    let cache = obj.entry( "cache" ).or_insert_with( || serde_json::json!( {} ) );
    if let Some( co ) = cache.as_object_mut()
    {
      co.insert( key.to_string(), value );
    }
  }
  let _ = std::fs::write( &meta_path, serde_json::to_string_pretty( &snapshot ).map( | s | s + "\n" ).unwrap_or_default() );
}

/// Write a string value into the cache object (typed convenience wrapper).
#[ inline ]
pub fn write_cache_string(
  credential_store : &std::path::Path,
  name             : &str,
  key              : &str,
  value            : &str,
)
{
  write_cache_field( credential_store, name, key, serde_json::Value::String( value.to_string() ) );
}

/// Write a bool value into the cache object (typed convenience wrapper).
#[ inline ]
pub fn write_cache_bool(
  credential_store : &std::path::Path,
  name             : &str,
  key              : &str,
  value            : bool,
)
{
  write_cache_field( credential_store, name, key, serde_json::Value::Bool( value ) );
}

/// Build a period cache JSON value from utilization + optional `resets_at`.
fn period_json( utilization : f64, resets_at : Option< &str > ) -> serde_json::Value
{
  let mut m = serde_json::Map::new();
  m.insert( "left_pct".into(), serde_json::Value::from( utilization ) );
  if let Some( r ) = resets_at
  {
    m.insert( "resets_at".into(), serde_json::Value::String( r.to_string() ) );
  }
  serde_json::Value::Object( m )
}

/// Extract a period tuple from a cache object.
fn read_period( cache : &serde_json::Map< String, serde_json::Value >, key : &str ) -> Option< ( f64, Option< String > ) >
{
  let p = cache.get( key )?.as_object()?;
  let left_pct = p.get( "left_pct" )?.as_f64()?;
  let resets_at = p.get( "resets_at" ).and_then( |v| v.as_str() ).map( str::to_string );
  Some( ( left_pct, resets_at ) )
}

/// Current UTC timestamp as ISO-8601 string (second precision).
///
/// Uses manual computation to avoid a chrono dependency — the format is
/// fixed and only needs second-level precision for cache age display.
#[ must_use ]
#[ inline ]
pub fn chrono_now_utc() -> String
{
  use std::time::{ SystemTime, UNIX_EPOCH };
  let secs = SystemTime::now().duration_since( UNIX_EPOCH ).unwrap_or_default().as_secs();
  // 86400 secs/day, days since epoch → year/month/day via civil calendar algorithm
  #[ allow( clippy::cast_possible_wrap ) ]
  let days = ( secs / 86400 ) as i64;
  let tod  = secs % 86400;
  let hh   = tod / 3600;
  let mm   = ( tod % 3600 ) / 60;
  let ss   = tod % 60;
  // Euclidean affine conversion from rata die to Y/M/D (Howard Hinnant algorithm).
  let z = days + 719_468;
  let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
  let doe = z - era * 146_097;
  let yoe = ( doe - doe / 1460 + doe / 36524 - doe / 146_096 ) / 365;
  let y   = yoe + era * 400;
  let doy = doe - ( 365 * yoe + yoe / 4 - yoe / 100 );
  let mp  = ( 5 * doy + 2 ) / 153;
  let d   = doy - ( 153 * mp + 2 ) / 5 + 1;
  let m   = if mp < 10 { mp + 3 } else { mp - 9 };
  let y   = if m <= 2 { y + 1 } else { y };
  format!( "{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z" )
}

/// Return a UTC timestamp prefix string for diagnostic trace lines.
///
/// Format: `"YYYY-MM-DD · HH:MM:SS · "` — two middle dots separate date, time, and body.
/// Use as first argument in `eprintln!( "{}label  name  ...", trace_ts() )` so the caller
/// label and account name follow immediately after the space.
#[ inline ]
#[ must_use ]
pub fn trace_ts() -> String
{
  let utc = chrono_now_utc();
  // chrono_now_utc produces "YYYY-MM-DDTHH:MM:SSZ"; slice date and time parts.
  format!( "{} · {} · ", &utc[ ..10 ], &utc[ 11..19 ] )
}

/// Parse an ISO-8601 UTC timestamp to seconds since epoch.
///
/// Accepts the format `YYYY-MM-DDTHH:MM:SSZ` as produced by `chrono_now_utc`.
/// Returns `None` on any parse failure.
#[ must_use ]
#[ inline ]
pub fn parse_iso_utc_secs( s : &str ) -> Option< u64 >
{
  if s.len() < 20 || !s.ends_with( 'Z' ) { return None; }
  let y : i64 = s[ 0..4 ].parse().ok()?;
  let m : i64 = s[ 5..7 ].parse().ok()?;
  let d : i64 = s[ 8..10 ].parse().ok()?;
  let hh : u64 = s[ 11..13 ].parse().ok()?;
  let mm : u64 = s[ 14..16 ].parse().ok()?;
  let ss : u64 = s[ 17..19 ].parse().ok()?;
  // Inverse of Hinnant: Y/M/D → days since epoch.
  let y2 = if m <= 2 { y - 1 } else { y };
  let era = if y2 >= 0 { y2 } else { y2 - 399 } / 400;
  let yoe = y2 - era * 400;
  let m2  = if m > 2 { m - 3 } else { m + 9 };
  let doy = ( 153 * m2 + 2 ) / 5 + d - 1;
  let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
  #[ allow( clippy::cast_sign_loss ) ]
  let days = ( era * 146_097 + doe - 719_468 ) as u64;
  Some( days * 86400 + hh * 3600 + mm * 60 + ss )
}

// ── Measurement history ───────────────────────────────────────────────────────

/// Timestamped quota measurement stored in `cache.history[]`.
///
/// Each successful fetch appends one entry; the array is capped at 10 entries (FIFO).
/// Used by the approximation module to fit a polynomial when the API is unavailable
/// (Feature 040).
#[ derive( Debug ) ]
pub struct HistoryEntry
{
  /// Unix timestamp (seconds) when the measurement was taken.
  pub t  : u64,
  /// 5h period: `(utilization 0–100, resets_at ISO string)`; `None` when absent.
  pub h5 : Option< ( f64, String ) >,
  /// 7d period: `(utilization 0–100, resets_at ISO string)`; `None` when absent.
  pub d7 : Option< ( f64, String ) >,
  /// 7d-sonnet period: `(utilization 0–100, resets_at ISO string)`; `None` when absent.
  pub sn : Option< ( f64, String ) >,
}

/// Parse a `[f64, string]` JSON array into a period tuple.
fn parse_history_period( val : &serde_json::Value ) -> Option< ( f64, String ) >
{
  let arr = val.as_array()?;
  if arr.len() != 2 { return None; }
  let u = arr[ 0 ].as_f64()?;
  let r = arr[ 1 ].as_str()?.to_string();
  Some( ( u, r ) )
}

/// Read measurement history from `cache.history[]` in `{name}.json` (Feature 040 AC-11).
///
/// Returns an empty `Vec` when the file is absent, unparseable, or has no `"history"` key —
/// backward compatible with old cache format from Feature 033.
#[ must_use ]
#[ inline ]
pub fn read_history(
  credential_store : &std::path::Path,
  name             : &str,
) -> Vec< HistoryEntry >
{
  let meta_path = credential_store.join( format!( "{name}.json" ) );
  let Ok( text ) = std::fs::read_to_string( &meta_path ) else { return vec![] };
  let val : serde_json::Value = match serde_json::from_str( &text ) { Ok( v ) => v, Err( _ ) => return vec![] };
  let Some( arr ) = val.get( "cache" ).and_then( |c| c.get( "history" ) ).and_then( |h| h.as_array() ) else { return vec![] };
  arr.iter().filter_map( |entry|
  {
    let t = entry.get( "t" )?.as_u64()?;
    Some( HistoryEntry
    {
      t,
      h5 : entry.get( "h5" ).and_then( parse_history_period ),
      d7 : entry.get( "d7" ).and_then( parse_history_period ),
      sn : entry.get( "sn" ).and_then( parse_history_period ),
    } )
  } ).collect()
}

/// Serialize an optional period to a JSON array `[utilization, resets_at]` or `null`.
fn history_period_json( period : Option< ( f64, &str ) > ) -> serde_json::Value
{
  match period
  {
    Some( ( u, r ) ) => serde_json::json!( [ u, r ] ),
    None             => serde_json::Value::Null,
  }
}

/// Append a quota measurement to `cache.history[]` in `{name}.json` (Feature 040 AC-01, AC-02, AC-13).
///
/// - Enforces a 10-entry FIFO ring buffer: oldest entry evicted when buffer is full (AC-02).
/// - Overwrites the last entry when `t` matches its timestamp to prevent fast-cycle fill (AC-13).
/// - Creates `cache.history` when absent (first measurement for this account).
/// - Write failures are silently ignored — quota display is non-critical (matches Feature 033 pattern).
#[ inline ]
pub fn write_history_entry(
  credential_store : &std::path::Path,
  name             : &str,
  t                : u64,
  h5               : Option< ( f64, &str ) >,
  d7               : Option< ( f64, &str ) >,
  sn               : Option< ( f64, &str ) >,
)
{
  let meta_path = credential_store.join( format!( "{name}.json" ) );
  let mut snapshot = std::fs::read_to_string( &meta_path )
    .ok()
    .and_then( |s| serde_json::from_str::< serde_json::Value >( &s ).ok() )
    .unwrap_or_else( || serde_json::json!( {} ) );
  if let Some( obj ) = snapshot.as_object_mut()
  {
    let cache = obj.entry( "cache" ).or_insert_with( || serde_json::json!( {} ) );
    if let Some( co ) = cache.as_object_mut()
    {
      let history = co.entry( "history" ).or_insert_with( || serde_json::json!( [] ) );
      if let Some( arr ) = history.as_array_mut()
      {
        let entry = serde_json::json!(
        {
          "t"  : t,
          "h5" : history_period_json( h5 ),
          "d7" : history_period_json( d7 ),
          "sn" : history_period_json( sn ),
        } );
        // AC-13: duplicate-timestamp dedup — overwrite last entry when same Unix second.
        if let Some( last ) = arr.last()
        {
          if last.get( "t" ).and_then( serde_json::Value::as_u64 ) == Some( t )
          {
            let len = arr.len();
            arr[ len - 1 ] = entry;
          }
          else
          {
            arr.push( entry );
          }
        }
        else
        {
          arr.push( entry );
        }
        // AC-02: ring buffer FIFO cap — evict oldest (index 0) when over 10 entries.
        while arr.len() > 10
        {
          arr.remove( 0 );
        }
      }
    }
  }
  let _ = std::fs::write(
    &meta_path,
    serde_json::to_string_pretty( &snapshot ).map( |s| s + "\n" ).unwrap_or_default(),
  );
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  // ── FT-08 (021): parse_string_array_field ───────────────────────────────────

  /// `ft08_a`: Two-element array returns both values in order.
  ///
  /// Given: `{"capabilities":["claude_max","chat"]}`
  /// When: `parse_string_array_field(json, "capabilities")`
  /// Then: Returns `["claude_max", "chat"]`
  #[ test ]
  fn ft08_parse_string_array_field_two_elements()
  {
    let json   = r#"{"capabilities":["claude_max","chat"]}"#;
    let result = parse_string_array_field( json, "capabilities" );
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
    let result = parse_string_array_field( json, "capabilities" );
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
    let result = parse_string_array_field( json, "capabilities" );
    assert!( result.is_empty(), "empty array must return empty Vec, got: {result:?}" );
  }
}


