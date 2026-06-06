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
//! account::save( "alice@acme.com", credential_store, &paths, true, None, None, None ).expect( "failed to save" );
//!
//! // Switch to "alice@home.com"
//! account::switch_account( "alice@home.com", credential_store, &paths ).expect( "failed to switch" );
//!
//! // Delete an old entry
//! account::delete( "alice@oldco.com", credential_store ).expect( "failed to delete" );
//! ```

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
  /// Organisation role from saved `{name}.json` `oauthAccount.organizationRole`.
  /// Empty string when snapshot absent or field missing.
  pub role : String,
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
  /// User's role in the organisation from saved `{name}.json` `organization_role`.
  /// Empty string when snapshot absent or field missing.
  pub organization_role : String,
  /// Workspace UUID from saved `{name}.json` `workspace_uuid`.
  /// Empty string when snapshot absent or field missing (personal accounts have `null`).
  pub workspace_uuid : String,
  /// Workspace display name from saved `{name}.json` `workspace_name`.
  /// Empty string when snapshot absent or field missing (personal accounts have `null`).
  pub workspace_name : String,
  /// Machine host label from saved `{name}.json` `host`.
  /// Empty string when file absent or field missing.
  pub profile_host : String,
  /// User-defined role label from saved `{name}.json` `role`.
  /// Empty string when file absent or field missing.
  pub profile_role : String,
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

  let active = read_active_marker( credential_store );
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
    let role         = parse_string_field( &meta_json, "organizationRole" ).unwrap_or_default();
    let billing      = parse_string_field( &meta_json, "billingType"      ).unwrap_or_default();
    let model        = parse_string_field( &meta_json, "model"            ).unwrap_or_default();
    let tagged_id    = parse_string_field( &meta_json, "taggedId"         ).unwrap_or_default();
    let uuid         = parse_string_field( &meta_json, "uuid"             ).unwrap_or_default();
    let capabilities = parse_string_array_field( &meta_json, "capabilities" );
    let organization_uuid = parse_string_field( &meta_json, "organization_uuid" ).unwrap_or_default();
    let organization_name = parse_string_field( &meta_json, "organization_name" ).unwrap_or_default();
    let organization_role = parse_string_field( &meta_json, "organization_role" ).unwrap_or_default();
    let workspace_uuid    = parse_string_field( &meta_json, "workspace_uuid"    ).unwrap_or_default();
    let workspace_name    = parse_string_field( &meta_json, "workspace_name"    ).unwrap_or_default();
    let profile_host = parse_string_field( &meta_json, "host" ).unwrap_or_default();
    let profile_role = parse_string_field( &meta_json, "role" ).unwrap_or_default();

    accounts.push( Account
    {
      name,
      subscription_type,
      rate_limit_tier,
      expires_at_ms,
      is_active,
      email,
      display_name,
      role,
      billing,
      model,
      tagged_id,
      uuid,
      capabilities,
      organization_uuid,
      organization_name,
      organization_role,
      workspace_uuid,
      workspace_name,
      profile_host,
      profile_role,
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
/// # Errors
///
/// Returns an error if the name is invalid, the credentials file cannot be
/// read, or the credential store cannot be written.
#[ inline ]
pub fn save(
  name             : &str,
  credential_store : &Path,
  paths            : &ClaudePaths,
  update_marker    : bool,
  creds            : Option< &[u8] >,
  host             : Option< &str >,
  role             : Option< &str >,
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
  }
  // Only write {name}.json when there is actual data to store — avoids empty {} files
  // for accounts with no oauthAccount, no model, and no host/role metadata.
  // Existing {name}.json is always non-empty (read-merged above), so this never drops data.
  if snapshot.as_object().is_some_and( |obj| !obj.is_empty() )
  {
    let _ = std::fs::write( &meta_path, serde_json::to_string( &snapshot ).unwrap_or_default() );
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
        let _ = std::fs::write( live_path, serde_json::to_string( &live_val ).unwrap_or_default() );
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
    let _ = std::fs::write( live_settings_path, serde_json::to_string( &live_settings ).unwrap_or_default() );
  }

  Ok( () )
}

/// Automatically rotate to the best available inactive account.
///
/// Selects the inactive account with the highest `expires_at_ms` and switches
/// to it. Consolidates the pick-best-account-and-switch pattern so callers
/// need a single call instead of duplicating the selection loop.
///
/// Returns the name of the account switched to.
///
/// # Errors
///
/// Returns `NotFound` if no accounts are configured or if no inactive account
/// is available (all accounts are the currently active one).
///
/// # Examples
///
/// ```no_run
/// use claude_profile_core::account;
/// use claude_core::ClaudePaths;
/// use std::path::Path;
///
/// let credential_store = Path::new( "/pro/.persistent/claude/credential" );
/// let paths = ClaudePaths::new().expect( "HOME must be set" );
/// let switched_to = account::auto_rotate( credential_store, &paths ).expect( "rotation failed" );
/// println!( "rotated to: {switched_to}" );
/// ```
#[ inline ]
pub fn auto_rotate( credential_store : &Path, paths : &ClaudePaths ) -> Result< String, std::io::Error >
{
  let candidate = list( credential_store )?
    .into_iter()
    .filter( | a | !a.is_active )
    .max_by_key( | a | a.expires_at_ms )
    .ok_or_else( || std::io::Error::new(
      std::io::ErrorKind::NotFound,
      "no inactive account available to rotate to",
    ) )?;

  let name = candidate.name;
  switch_account( &name, credential_store, paths )?;
  Ok( name )
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
  // Override Sonnet → Opus. Already-Opus/Haiku/other models are left unchanged.
  if current == "claude-sonnet-4-6" || current.is_empty()
  {
    obj.insert( "model".to_string(), serde_json::Value::String( "claude-opus-4-6".to_string() ) );
    let _ = std::fs::write( path, serde_json::to_string( &live ).unwrap_or_default() );
    true
  }
  else
  {
    false
  }
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
    // Fix(BUG-175): removed switch_account call — credentials read directly from credential store
    // Root cause: Some(paths) branch read via p.credentials_file() forcing switch_account to populate it;
    //   run_isolated creates its own temp HOME and never reads ~/.claude/, so the write was redundant
    // Pitfall: switch_account before a read looks like defensive initialization;
    //   the unnecessary global write is only visible in concurrent multi-account batch scenarios
    let creds_json = match std::fs::read_to_string( credential_store.join( format!( "{name}.credentials.json" ) ) )
    {
      Ok( s )  => { if trace { eprintln!( "[trace] {label}  {name}  read credentials: OK" ); } s }
      Err( e ) =>
      {
        if trace { eprintln!( "[trace] {label}  {name}  read credentials: Err({e})" ); }
        return None;
      }
    };
    let t_run = std::time::Instant::now();
    if trace { eprintln!( "[trace] {label}  {name}  run_isolated: invoking claude  args={args:?}  timeout=35s" ); }
    let isolated = match claude_runner_core::run_isolated( &creds_json, args, 35, model )
    {
      Ok( r )  => r,
      Err( e ) =>
      {
        if trace { eprintln!( "[trace] {label}  {name}  run_isolated: Err({e})  ({:.1}s)", t_run.elapsed().as_secs_f64() ); }
        return None;
      }
    };
    if trace
    {
      let creds_status = if isolated.credentials.is_some() { "Some" } else { "None" };
      eprintln!( "[trace] {label}  {name}  run_isolated: OK credentials={creds_status}  ({:.1}s)", t_run.elapsed().as_secs_f64() );
    }
    let new_creds = isolated.credentials?;
    // Fix(BUG-221): write refreshed credentials directly to the credential store, not to
    //   p.credentials_file() (the live session file ~/.claude/.credentials.json).
    // Root cause: BUG-175's fix (TSK-208) removed switch_account() but left the write to the
    //   live file intact; every batch refresh call clobbered the active session credentials.
    // Pitfall: save() is called with Some(&new_creds) so it writes from bytes directly,
    //   bypassing the copy-from-live-file path that would copy now-stale credentials.
    let store_cred_path = credential_store.join( format!( "{name}.credentials.json" ) );
    if let Err( e ) = std::fs::write( &store_cred_path, &new_creds )
    {
      if trace { eprintln!( "[trace] {label}  {name}  write credentials: Err({e})" ); }
      return None;
    }
    if trace { eprintln!( "[trace] {label}  {name}  write credentials: OK" ); }
    match save( name, credential_store, p, false, Some( new_creds.as_bytes() ), None, None )
    {
      Ok( () ) => { if trace { eprintln!( "[trace] {label}  {name}  save: OK" ); } }
      Err( e ) =>
      {
        if trace { eprintln!( "[trace] {label}  {name}  save: Err({e})" ); }
        return None;
      }
    }
    Some( new_creds )
  }
  else
  {
    let path = credential_store.join( format!( "{name}.credentials.json" ) );
    let creds_json = match std::fs::read_to_string( &path )
    {
      Ok( s )  => { if trace { eprintln!( "[trace] {label}  {name}  read credentials: OK" ); } s }
      Err( e ) =>
      {
        if trace { eprintln!( "[trace] {label}  {name}  read credentials: Err({e})" ); }
        return None;
      }
    };
    let t_run = std::time::Instant::now();
    if trace { eprintln!( "[trace] {label}  {name}  run_isolated: invoking claude  args={args:?}  timeout=35s" ); }
    let isolated = match claude_runner_core::run_isolated( &creds_json, args, 35, model )
    {
      Ok( r )  => r,
      Err( e ) =>
      {
        if trace { eprintln!( "[trace] {label}  {name}  run_isolated: Err({e})  ({:.1}s)", t_run.elapsed().as_secs_f64() ); }
        return None;
      }
    };
    if trace
    {
      let creds_status = if isolated.credentials.is_some() { "Some" } else { "None" };
      eprintln!( "[trace] {label}  {name}  run_isolated: OK credentials={creds_status}  ({:.1}s)", t_run.elapsed().as_secs_f64() );
    }
    let new_creds = isolated.credentials?;
    if let Err( e ) = std::fs::write( &path, &new_creds )
    {
      if trace { eprintln!( "[trace] {label}  {name}  write credentials: Err({e})" ); }
      return None;
    }
    if trace { eprintln!( "[trace] {label}  {name}  write credentials: OK" ); }
    Some( new_creds )
  }
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

  let new_json = serde_json::to_string( &val )
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


