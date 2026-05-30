//! Named credential storage and account rotation.
//!
//! # Account Store Layout
//!
//! ```text
//! $PRO/.persistent/claude/credential/
//!   alice@acme.com.credentials.json   ← saved credential snapshot
//!   alice@home.com.credentials.json
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
//! account::save( "alice@acme.com", credential_store, &paths, true ).expect( "failed to save" );
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
  /// Email address from saved `{name}.claude.json` `emailAddress`.
  /// Empty string when snapshot absent or field missing.
  pub email : String,
  /// Display name from saved `{name}.claude.json` `oauthAccount.displayName`.
  /// Empty string when snapshot absent or field missing.
  pub display_name : String,
  /// Organisation role from saved `{name}.claude.json` `oauthAccount.organizationRole`.
  /// Empty string when snapshot absent or field missing.
  pub role : String,
  /// Billing type from saved `{name}.claude.json` `oauthAccount.billingType`.
  /// Empty string when snapshot absent or field missing.
  pub billing : String,
  /// Active model from saved `{name}.settings.json` `model` field.
  /// Empty string when snapshot absent or field missing.
  pub model : String,
  /// Stable user identifier from saved `{name}.claude.json` `oauthAccount.taggedId`.
  /// Empty string when snapshot absent or field missing.
  pub tagged_id : String,
  /// UUID form of user identifier from saved `{name}.claude.json` `oauthAccount.uuid`.
  /// Empty string when snapshot absent or field missing.
  pub uuid : String,
  /// Enabled product capabilities from saved `{name}.claude.json` `oauthAccount.capabilities`.
  /// Empty vec when snapshot absent or field missing.
  pub capabilities : Vec< String >,
  /// Organisation UUID from saved `{name}.roles.json` `organization_uuid`.
  /// Empty string when snapshot absent or field missing.
  pub organization_uuid : String,
  /// Organisation display name from saved `{name}.roles.json` `organization_name`.
  /// Empty string when snapshot absent or field missing.
  pub organization_name : String,
  /// User's role in the organisation from saved `{name}.roles.json` `organization_role`.
  /// Empty string when snapshot absent or field missing.
  pub organization_role : String,
  /// Workspace UUID from saved `{name}.roles.json` `workspace_uuid`.
  /// Empty string when snapshot absent or field missing (personal accounts have `null`).
  pub workspace_uuid : String,
  /// Workspace display name from saved `{name}.roles.json` `workspace_name`.
  /// Empty string when snapshot absent or field missing (personal accounts have `null`).
  pub workspace_name : String,
  /// Machine host label from saved `{name}.profile.json` `host`.
  /// Empty string when file absent or field missing.
  pub profile_host : String,
  /// User-defined role label from saved `{name}.profile.json` `role`.
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

    // Read per-account snapshot files written by save() — best-effort, empty when absent.
    let claude_json = std::fs::read_to_string(
      credential_store.join( format!( "{name}.claude.json" ) )
    ).unwrap_or_default();
    let settings_json = std::fs::read_to_string(
      credential_store.join( format!( "{name}.settings.json" ) )
    ).unwrap_or_default();
    let email        = parse_string_field( &claude_json, "emailAddress"      ).unwrap_or_default();
    let display_name = parse_string_field( &claude_json, "displayName"      ).unwrap_or_default();
    let role         = parse_string_field( &claude_json, "organizationRole" ).unwrap_or_default();
    let billing      = parse_string_field( &claude_json, "billingType"      ).unwrap_or_default();
    let model        = parse_string_field( &settings_json, "model"          ).unwrap_or_default();
    let tagged_id    = parse_string_field( &claude_json, "taggedId"         ).unwrap_or_default();
    let uuid         = parse_string_field( &claude_json, "uuid"             ).unwrap_or_default();
    let capabilities = parse_string_array_field( &claude_json, "capabilities" );

    let roles_json = std::fs::read_to_string(
      credential_store.join( format!( "{name}.roles.json" ) )
    ).unwrap_or_default();
    let organization_uuid = parse_string_field( &roles_json, "organization_uuid" ).unwrap_or_default();
    let organization_name = parse_string_field( &roles_json, "organization_name" ).unwrap_or_default();
    let organization_role = parse_string_field( &roles_json, "organization_role" ).unwrap_or_default();
    let workspace_uuid    = parse_string_field( &roles_json, "workspace_uuid"    ).unwrap_or_default();
    let workspace_name    = parse_string_field( &roles_json, "workspace_name"    ).unwrap_or_default();

    let profile_json = std::fs::read_to_string(
      credential_store.join( format!( "{name}.profile.json" ) )
    ).unwrap_or_default();
    let profile_host = parse_string_field( &profile_json, "host" ).unwrap_or_default();
    let profile_role = parse_string_field( &profile_json, "role" ).unwrap_or_default();

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

/// Save the current credentials as a named account in `credential_store`.
///
/// Creates `{credential_store}/{name}.credentials.json`. Overwrites if exists.
/// Also writes `{credential_store}/_active` = `name` so that the saved account
/// is immediately visible as the active account without a separate switch call.
///
/// # Errors
///
/// Returns an error if the name is invalid, the credentials file cannot be
/// read, or the credential store cannot be written.
#[ inline ]
pub fn save( name : &str, credential_store : &Path, paths : &ClaudePaths, update_marker : bool ) -> Result< (), std::io::Error >
{
  validate_name( name )?;
  std::fs::create_dir_all( credential_store )?;
  let dest = credential_store.join( format!( "{name}.credentials.json" ) );
  std::fs::copy( paths.credentials_file(), dest )?;
  // Best-effort: extract oauthAccount subtree from ~/.claude.json and save it.
  // Fix(BUG-174): save() previously used std::fs::copy for the entire ~/.claude.json,
  //   including machine-global keys (commands.*, mcpServers, projects). On switch_account(),
  //   restoring the full copy clobbered the current machine's config state.
  // Root cause: wholesale copy/restore treated per-account auth data and machine-global
  //   config as a single unit.
  // Pitfall: the extraction must be surgical — only the oauthAccount subtree is
  //   per-account; everything else belongs to the machine.
  if let Ok( live_text ) = std::fs::read_to_string( paths.claude_json_file() )
  {
    if let Ok( live_val ) = serde_json::from_str::< serde_json::Value >( &live_text )
    {
      if let Some( oauth ) = live_val.get( "oauthAccount" )
      {
        // Fix(AC-17): read-merge preserves _renewal_at and any other pre-existing top-level keys
        //   when save() is called after .account.renewal has set _renewal_at.
        // Root cause: wholesale overwrite `{"oauthAccount": ...}` discarded pre-existing top-level
        //   keys from {name}.claude.json on every save() call.
        // Pitfall: must merge into existing object — never replace it — to avoid clobbering
        //   keys written by .account.renewal or other tooling.
        let claude_path = credential_store.join( format!( "{name}.claude.json" ) );
        let mut snapshot = std::fs::read_to_string( &claude_path )
          .ok()
          .and_then( |s| serde_json::from_str::< serde_json::Value >( &s ).ok() )
          .unwrap_or_else( || serde_json::json!( {} ) );
        if let Some( obj ) = snapshot.as_object_mut()
        {
          obj.insert( "oauthAccount".to_string(), oauth.clone() );
        }
        let _ = std::fs::write(
          claude_path,
          serde_json::to_string( &snapshot ).unwrap_or_default(),
        );
      }
    }
  }
  // Best-effort: fetch org identity from endpoint 005 and persist as {name}.roles.json.
  // Requires accessToken in the credentials file. Network errors or absent token are silently
  // skipped — save() must not fail for network unavailability or missing optional data.
  #[ cfg( feature = "enabled" ) ]
  {
    let creds_text = std::fs::read_to_string( paths.credentials_file() ).unwrap_or_default();
    if let Some( token ) = parse_string_field( &creds_text, "accessToken" )
    {
      if let Ok( roles ) = claude_quota::fetch_claude_cli_roles( &token )
      {
        let null_str = | s : &str | -> String
        {
          if s.is_empty() { "null".to_string() }
          else { format!( "\"{}\"", s.replace( '"', "\\\"" ) ) }
        };
        let roles_json = format!(
          "{{\"organization_uuid\":\"{}\",\"organization_name\":\"{}\",\
           \"organization_role\":\"{}\",\"workspace_uuid\":{},\"workspace_name\":{}}}",
          roles.organization_uuid.replace( '"', "\\\"" ),
          roles.organization_name.replace( '"', "\\\"" ),
          roles.organization_role.replace( '"', "\\\"" ),
          null_str( &roles.workspace_uuid ),
          null_str( &roles.workspace_name ),
        );
        let _ = std::fs::write( credential_store.join( format!( "{name}.roles.json" ) ), roles_json );
      }
    }
  }
  // Fix(BUG-211): guard _active write behind update_marker so background refresh callers
  //   (refresh_account_token) can pass false, suppressing marker mutation during per-account
  //   cycling. Without this guard every refresh clobbered _active, enabling the TOCTOU race
  //   where the subsequent snapshot+restore in apply_refresh/apply_touch overwrote a concurrent
  //   .account.use switch. See bug/211_apply_refresh_touch_restore_clobbers_active_marker_race.md.
  // Root cause: save() treated _active write as unconditional; callers had no opt-out, so every
  //   background refresh mutated the active marker as a side effect.
  // Pitfall: CLI callers (.account.save, .account.relogin) MUST pass true; passing false from a
  //   user-triggered path leaves .credentials.status showing Account: N/A until next explicit save.
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

  // Fix(BUG-174): switch_account() previously used std::fs::copy to wholesale overwrite
  //   ~/.claude.json with the saved snapshot. This clobbered machine-global keys
  //   (commands.*, mcpServers, projects) with stale values from the snapshot.
  // Root cause: save() captured the full file; switch_account() restored it wholesale.
  //   Both sides now use surgical oauthAccount extraction/patching.
  // Pitfall: the live ~/.claude.json must be READ before patching — if it's missing or
  //   invalid JSON, the saved snapshot's oauthAccount is written as a new file (safe
  //   fallback since there's nothing to clobber).
  {
    let saved_path = credential_store.join( format!( "{name}.claude.json" ) );
    if let Ok( saved_text ) = std::fs::read_to_string( &saved_path )
    {
      if let Ok( saved_val ) = serde_json::from_str::< serde_json::Value >( &saved_text )
      {
        if let Some( mut oauth ) = saved_val.get( "oauthAccount" ).cloned()
        {
          // Fix(BUG-217): enforce emailAddress == name — snapshot may contain stale email
          // from an earlier BUG-212/BUG-217 corruption cycle; name is always the canonical
          // identity source, regardless of what the per-account snapshot says.
          // Root cause: verbatim insert propagated wrong email to ~/.claude.json, causing
          // downstream save routines to infer the wrong account name.
          // Pitfall: without this override the error self-perpetuates — stale email installed
          // in shared file → read by save as primary name → snapshotted with wrong email →
          // re-installed on next switch.
          if let Some( oa_obj ) = oauth.as_object_mut()
          {
            oa_obj.insert( "emailAddress".to_string(), serde_json::Value::String( name.to_string() ) );
            // Fix(BUG-219): override org-identity fields from {name}.roles.json — snapshot may
            //   contain stale organizationName/organizationUuid captured while a different
            //   account's session was active (BUG-217 partial fix only corrected emailAddress).
            // Root cause: verbatim snapshot copy propagates cross-account org identity to
            //   ~/.claude.json; Claude Code reads oauthAccount.organizationName from this file
            //   and displays the wrong organization name after switch.
            // Pitfall: roles.json read must be best-effort (absent file ≠ error); only override
            //   when non-empty to avoid clearing org fields for accounts without roles.json.
            let roles_path = credential_store.join( format!( "{name}.roles.json" ) );
            if let Ok( roles_text ) = std::fs::read_to_string( &roles_path )
            {
              if let Some( org_name ) = parse_string_field( &roles_text, "organization_name" )
              {
                if !org_name.is_empty()
                {
                  oa_obj.insert( "organizationName".to_string(), serde_json::Value::String( org_name ) );
                }
              }
              if let Some( org_uuid ) = parse_string_field( &roles_text, "organization_uuid" )
              {
                if !org_uuid.is_empty()
                {
                  oa_obj.insert( "organizationUuid".to_string(), serde_json::Value::String( org_uuid ) );
                }
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
    }
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
/// Removes the credentials snapshot and, best-effort, the accompanying
/// `.claude.json` and `.settings.json` snapshots created by `save()`, and
/// the `_active` marker if it currently points at the deleted account.
/// All best-effort removals use `let _ = ...` — accounts saved before snapshot
/// support was introduced have no snapshot files, and missing files are silently skipped.
///
/// # Errors
///
/// Returns `NotFound` if the account does not exist.
#[ inline ]
pub fn delete( name : &str, credential_store : &Path ) -> Result< (), std::io::Error >
{
  // Fix(issue-snapshot-orphan):
  // Root cause: save() creates 3 files but delete() only removed .credentials.json,
  //   leaving .claude.json and .settings.json as orphans after every delete.
  // Pitfall: snapshot removal must be best-effort (let _ = ...) — pre-snapshot accounts
  //   have no snapshot files; a strict remove_file() would fail them.
  check_delete_preconditions( name, credential_store )?;
  let target = credential_store.join( format!( "{name}.credentials.json" ) );
  std::fs::remove_file( target )?;
  let _ = std::fs::remove_file( credential_store.join( format!( "{name}.claude.json" ) ) );
  let _ = std::fs::remove_file( credential_store.join( format!( "{name}.settings.json" ) ) );
  let _ = std::fs::remove_file( credential_store.join( format!( "{name}.roles.json" ) ) );
  // Fix(issue-delete-active):
  // Root cause: `PermissionDenied` guard blocked deleting the active account; `delete()`
  //   never cleaned up `_active`, leaving a stale marker after any deletion.
  // Pitfall: Never block on `_active` marker state — the live credential file is already
  //   deployed; clean up the stale marker after deletion rather than refusing the operation.
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
    if let Err( e ) = std::fs::write( p.credentials_file(), &new_creds )
    {
      if trace { eprintln!( "[trace] {label}  {name}  write credentials: Err({e})" ); }
      return None;
    }
    if trace { eprintln!( "[trace] {label}  {name}  write credentials: OK" ); }
    match save( name, credential_store, p, false )
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
/// env var first, falls back to `USERNAME`, then to the literal `"user"`.
///
/// The per-machine name means that switching accounts on one machine does not
/// affect other machines sharing the same credential store via version control.
/// Add `` `_active_*` `` to `.gitignore` to prevent these files from being tracked.
#[ inline ]
#[ must_use ]
pub fn active_marker_filename() -> String
{
  let hostname = std::env::var( "HOSTNAME" )
    .unwrap_or_else( |_|
    {
      std::fs::read_to_string( "/etc/hostname" )
        .unwrap_or_else( |_| "local".to_string() )
        .trim()
        .to_string()
    } );
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

// ── Account renewal ───────────────────────────────────────────────────────────

/// The operation to apply to `_renewal_at` in `{name}.claude.json`.
#[ derive( Debug ) ]
pub enum RenewalOperation
{
  /// Set `_renewal_at` to the given ISO-8601 UTC string (stored verbatim).
  At( String ),
  /// Remove `_renewal_at` from the file.
  Clear,
}

/// Write or clear a billing renewal timestamp override in `{name}.claude.json`.
///
/// Reads the existing `{name}.claude.json` (or starts with `{}` if absent), applies `op`,
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

  let claude_path  = credential_store.join( format!( "{name}.claude.json" ) );
  let existing_str = std::fs::read_to_string( &claude_path )
    .unwrap_or_else( |_| "{}".to_string() );
  let mut val = serde_json::from_str::< serde_json::Value >( &existing_str )
    .unwrap_or_else( |_| serde_json::json!( {} ) );
  let obj = val.as_object_mut()
    .ok_or_else( || std::io::Error::new(
      std::io::ErrorKind::InvalidData,
      format!( "{name}.claude.json is not a JSON object" ),
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
  std::fs::write( &claude_path, new_json )?;
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


