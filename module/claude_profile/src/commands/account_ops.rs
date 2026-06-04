//! Account mutation command handlers: use, rotate, save, delete, relogin.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use super::shared::{
  require_nonempty_string_arg, is_dry, require_claude_paths, require_credential_store,
  io_err_to_error_data, resolve_account_name,
};

/// `.account.use` — atomic credential rotation by name.
///
/// # Errors
///
/// Returns `ErrorData` if name is missing/empty, HOME is unset,
/// or the target account does not exist.
#[ inline ]
#[ allow( clippy::too_many_lines ) ]
pub fn account_use_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  // Validate all CLI arguments before any I/O (fast-fail on bad values before filesystem access).
  // Fix(issue-switch-dry-validation): is_dry() check comes after existence validation so
  //   dry-run on nonexistent accounts correctly exits 2 (not silently succeeds).
  // Pitfall: Only the mutating step (file copy + marker write) is skipped in dry-run;
  //   all validation and precondition checks must run unconditionally.
  let raw_name   = require_nonempty_string_arg( &cmd, "name" )?;
  let touch      = crate::output::parse_int_flag( &cmd, "touch", 1 )?;
  let trace      = crate::output::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let imodel_str = match cmd.arguments.get( "imodel" )
  {
    None                       => "auto".to_string(),
    Some( Value::String( s ) ) =>
    {
      crate::usage::validate_imodel_str( s )
        .map_err( |e| ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?;
      s.clone()
    }
    _ => return Err( ErrorData::new( ErrorCode::ArgumentTypeMismatch, "imodel:: must be a string".to_string() ) ),
  };
  let effort_str = match cmd.arguments.get( "effort" )
  {
    None                       => "auto".to_string(),
    Some( Value::String( s ) ) =>
    {
      crate::usage::validate_effort_str( s )
        .map_err( |e| ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?;
      s.clone()
    }
    _ => return Err( ErrorData::new( ErrorCode::ArgumentTypeMismatch, "effort:: must be a string".to_string() ) ),
  };
  let refresh          = crate::output::parse_int_flag( &cmd, "refresh", 1 )?;
  let paths            = require_claude_paths()?;
  let credential_store = require_credential_store()?;
  let name             = resolve_account_name( &raw_name, &credential_store )?;
  crate::account::check_switch_preconditions( &name, &credential_store )
    .map_err( |e| io_err_to_error_data( &e, "account use" ) )?;

  if is_dry( &cmd )
  {
    return Ok( OutputData::new( format!( "[dry-run] would switch to '{name}'\n" ), "text" ) );
  }

  // Pre-fetch quota before the switch while the target credential file is still readable.
  let mut touch_ctx = if touch != 0
  {
    crate::usage::pre_switch_touch_ctx( &name, &credential_store, trace, &imodel_str, &effort_str )
  }
  else
  {
    None
  };

  // Fix(BUG-213): when touch is enabled and the quota fetch failed (touch_ctx is None),
  //   check expiresAt and attempt refresh (BUG-230) before calling switch_account().
  if touch != 0 && touch_ctx.is_none()
  {
    touch_ctx = check_expiry_and_refresh(
      &name, &credential_store, &paths, refresh, trace, &imodel_str, &effort_str,
    );
  }

  crate::account::switch_account( &name, &credential_store, &paths )
    .map_err( |e| io_err_to_error_data( &e, "account use" ) )?;

  // Post-switch: activate idle session if quota indicated it was idle before switch.
  // Fix(BUG-225): also performs quota-aware Sonnet→Opus session model override when 7d(Son) < 20%.
  if let Some( ctx ) = touch_ctx
  {
    crate::usage::apply_post_switch_touch( &name, ctx, &imodel_str, &effort_str, trace, &paths );
  }

  Ok( OutputData::new( format!( "switched to '{name}'\n" ), "text" ) )
}

/// Check whether the target account's token is expired; attempt refresh if so.
///
/// Called only when `touch` is enabled and `pre_switch_touch_ctx()` returned `None`.
/// Returns a new `TouchCtx` when refresh succeeds, `None` when the token is not expired
/// (or the credential file cannot be read). Exits the process with code 3 when the token
/// is expired but cannot be refreshed.
///
/// # Fix(BUG-213)
/// Root cause: `pre_switch_touch_ctx()` returns `None` for any fetch error without
/// distinguishing "token valid but quota unreachable" from "token locally expired".
/// Pitfall: callers treating all `None` returns identically must add their own expiry
/// guard at the decision point, as done here.
///
/// # Fix(BUG-230)
/// Root cause: the original BUG-213 guard exited 3 without attempting OAuth refresh.
/// Token expiry is recoverable when `refresh != 0` (the default).
/// Pitfall: after a successful refresh the `touch_ctx` must be re-probed — the old `None`
/// is stale once the fresh token makes quota fetch viable.
fn check_expiry_and_refresh(
  name             : &str,
  credential_store : &std::path::Path,
  paths            : &crate::ClaudePaths,
  refresh          : i64,
  trace            : bool,
  imodel_str       : &str,
  effort_str       : &str,
) -> Option< crate::usage::TouchCtx >
{
  let cred_path  = credential_store.join( format!( "{name}.credentials.json" ) );
  let cred_str = std::fs::read_to_string( &cred_path ).ok()?;
  let needle     = "\"expiresAt\":";
  let expires_ms = cred_str.find( needle ).and_then( | pos |
  {
    let rest = cred_str[ pos + needle.len().. ].trim_start();
    let end  = rest.find( | c : char | !c.is_ascii_digit() ).unwrap_or( rest.len() );
    rest[ ..end ].parse::< u64 >().ok()
  } );
  let exp_ms = expires_ms?;
  use std::time::{ SystemTime, UNIX_EPOCH };
  let now_ms = u64::try_from(
    SystemTime::now().duration_since( UNIX_EPOCH ).unwrap_or_default().as_millis()
  ).unwrap_or( u64::MAX );
  if now_ms <= exp_ms
  {
    if trace
    {
      let rem_s = ( exp_ms - now_ms ) / 1000;
      eprintln!( "[trace] account.use  {name}  expiry check: valid (expires in {}h {}m)", rem_s / 3600, ( rem_s % 3600 ) / 60 );
    }
    return None;
  }
  let elapsed_s = ( now_ms - exp_ms ) / 1000;
  let h         = elapsed_s / 3600;
  let m         = ( elapsed_s % 3600 ) / 60;
  if refresh != 0
  {
    if trace { eprintln!( "[trace] account.use  {name}  expiry check: expired({h}h {m}m ago) → attempting refresh" ) }
    let refreshed = crate::usage::attempt_expired_token_refresh( name, credential_store, paths, trace, imodel_str, effort_str );
    if refreshed
    {
      if trace { eprintln!( "[trace] account.use  {name}  expiry check: refresh OK — re-probing touch context" ) }
      return crate::usage::pre_switch_touch_ctx( name, credential_store, trace, imodel_str, effort_str );
    }
    if trace { eprintln!( "[trace] account.use  {name}  expiry check: refresh failed → refused" ) }
    eprintln!( "account credentials expired and refresh failed: {name} (expired {h}h {m}m ago)" );
  }
  else
  {
    if trace { eprintln!( "[trace] account.use  {name}  expiry check: expired({h}h {m}m ago) → refused (refresh::0)" ) }
    eprintln!( "account credentials expired: {name} (expired {h}h {m}m ago)" );
  }
  std::process::exit( 3 );
}

/// `.account.rotate` — auto-rotate to the highest-expiry inactive account.
///
/// # Errors
///
/// Returns `ErrorData` if HOME is unset, the credential store cannot be read,
/// or no inactive account is available to rotate to.
#[ inline ]
pub fn account_rotate_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let trace            = crate::output::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let credential_store = require_credential_store()?;
  if trace { eprintln!( "[trace] account.rotate  reading store: {}", credential_store.display() ) }
  let paths            = require_claude_paths()?;
  if is_dry( &cmd )
  {
    let candidate = crate::account::list( &credential_store )
      .map_err( |e| io_err_to_error_data( &e, "account rotate" ) )?
      .into_iter()
      .filter( |a| !a.is_active )
      .max_by_key( |a| a.expires_at_ms )
      .ok_or_else( || ErrorData::new(
        ErrorCode::InternalError,
        "no inactive account available to rotate to".to_string(),
      ) )?;
    return Ok( OutputData::new( format!( "[dry-run] would rotate to '{}'\n", candidate.name ), "text" ) );
  }
  let name = crate::account::auto_rotate( &credential_store, &paths )
    .map_err( |e| io_err_to_error_data( &e, "account rotate" ) )?;
  Ok( OutputData::new( format!( "rotated to '{name}'\n" ), "text" ) )
}

/// `.account.save` — save current credentials as a named account profile.
///
/// # Errors
///
/// Returns `ErrorData` if the name cannot be resolved (explicit empty value or
/// `_active` marker absent from the credential store), HOME is unset,
/// or the credential copy fails.
#[ inline ]
pub fn account_save_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let paths            = require_claude_paths()?;
  let trace            = crate::output::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let name             = match cmd.arguments.get( "name" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => s.clone(),
    Some( Value::String( _ ) ) =>
      return Err( ErrorData::new( ErrorCode::ArgumentMissing, "name:: value cannot be empty".to_string() ) ),
    _ =>
    {
      // Fix(BUG-212): read oauthAccount.emailAddress from ~/.claude.json as primary inference source;
      //   fall back to _active marker only when emailAddress is absent or empty.
      // Root cause: BUG-209 fix replaced stale top-level emailAddress with _active marker, but the marker
      //   is only written by clp ops (switch_account, save). External OAuth login writes ~/.claude.json
      //   (including oauthAccount.emailAddress) without updating _active — leaving the marker stale.
      // Pitfall: any single-source inference fails when other credential-change paths bypass that source.
      //   oauthAccount.emailAddress is updated by BOTH clp switches (snapshot restore) AND external OAuth
      //   login (Claude writes ~/.claude.json on every auth). _active is clp-only.
      let cs          = require_credential_store()?;
      let cj          = std::fs::read_to_string( paths.claude_json_file() ).unwrap_or_default();
      // Extract emailAddress nested inside oauthAccount {…}: locate "oauthAccount": first,
      // then apply parse_string_field on the suffix so only the nested key is found.
      let oauth_email = cj
        .find( "\"oauthAccount\":" )
        .and_then( | pos | crate::account::parse_string_field( &cj[ pos.. ], "emailAddress" ) )
        .filter( | s | !s.is_empty() );
      if let Some( email ) = oauth_email
      {
        email
      }
      else
      {
        let marker_path = cs.join( crate::account::active_marker_filename() );
        std::fs::read_to_string( &marker_path )
          .ok()
          .map( | s | s.trim().to_string() )
          .filter( | s | !s.is_empty() )
          .ok_or_else( || ErrorData::new(
            ErrorCode::ArgumentMissing,
            "cannot infer account name: no active account set — pass name:: explicitly".to_string(),
          ) )?
      }
    }
  };
  let credential_store = require_credential_store()?;
  if trace { eprintln!( "[trace] account.save  reading {}", paths.credentials_file().display() ) }

  if is_dry( &cmd )
  {
    return Ok( OutputData::new( format!( "[dry-run] would save current credentials as '{name}'\n" ), "text" ) );
  }

  crate::account::save( &name, &credential_store, &paths, true, None )
    .map_err( |e| io_err_to_error_data( &e, "account save" ) )?;
  if trace { eprintln!( "[trace] account.save  write: OK" ) }

  // Write {name}.profile.json with host and role metadata (TSK-225).
  // host:: defaults to auto-captured $USER@$HOSTNAME when omitted.
  let host_val = match cmd.arguments.get( "host" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => s.clone(),
    _ =>
    {
      let user     = std::env::var( "USER" ).unwrap_or_default();
      let hostname = std::env::var( "HOSTNAME" ).unwrap_or_default();
      // Both absent: store empty string rather than bare "@" (AC-03).
      if user.is_empty() && hostname.is_empty() { String::new() }
      else { format!( "{user}@{hostname}" ) }
    }
  };
  let role_val = match cmd.arguments.get( "role" )
  {
    Some( Value::String( s ) ) => s.clone(),
    _                          => String::new(),
  };
  let profile_json = format!(
    "{{\"host\":\"{}\",\"role\":\"{}\"}}",
    host_val.replace( '"', "\\\"" ),
    role_val.replace( '"', "\\\"" ),
  );
  let _ = std::fs::write( credential_store.join( format!( "{name}.profile.json" ) ), &profile_json );
  if trace { eprintln!( "[trace] account.save  profile.json: {profile_json}" ) }

  Ok( OutputData::new( format!( "saved current credentials as '{name}'\n" ), "text" ) )
}

/// `.account.delete` — delete a saved account (guard: refuses active).
///
/// # Errors
///
/// Returns `ErrorData` if name is missing/empty, HOME is unset,
/// or the account does not exist.
#[ inline ]
pub fn account_delete_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  // Fix(issue-delete-dry-validation):
  // Root cause: is_dry() was checked before existence check,
  //   so dry-run bypassed NotFound (missing account).
  // Pitfall: precondition checks must run before the dry-run shortcut.
  let trace            = crate::output::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let raw_name         = require_nonempty_string_arg( &cmd, "name" )?;
  let credential_store = require_credential_store()?;
  if trace { eprintln!( "[trace] account.delete  store: {}", credential_store.display() ) }
  let name             = resolve_account_name( &raw_name, &credential_store )?;
  crate::account::check_delete_preconditions( &name, &credential_store )
    .map_err( |e| io_err_to_error_data( &e, "account delete" ) )?;

  if is_dry( &cmd )
  {
    return Ok( OutputData::new( format!( "[dry-run] would delete account '{name}'\n" ), "text" ) );
  }

  crate::account::delete( &name, &credential_store )
    .map_err( |e| io_err_to_error_data( &e, "account delete" ) )?;
  Ok( OutputData::new( format!( "deleted account '{name}'\n" ), "text" ) )
}

/// `.account.relogin` — force browser re-authentication for a named account with dead refreshToken.
///
/// Switches to the named account, spawns `claude` with inherited TTY so the user can
/// complete browser login, then saves the refreshed credentials back into the account store
/// and restores the original active account.
///
/// # Errors
///
/// - Exit 1: `name::` value is empty or contains invalid characters.
/// - Exit 2: `name::` omitted and no active account; account not found; HOME unset;
///   `claude` binary cannot be spawned; or save fails.
/// - Exit 3 (via `process::exit`): `claude` exited without updating `~/.claude/.credentials.json`
///   (login abandoned or timed out).
#[ inline ]
pub fn account_relogin_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let trace            = crate::output::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let paths            = require_claude_paths()?;
  let credential_store = require_credential_store()?;
  if trace { eprintln!( "[trace] account.relogin  store: {}", credential_store.display() ) }
  let raw_name         = match cmd.arguments.get( "name" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => s.clone(),
    Some( Value::String( _ ) )                  =>
      return Err( ErrorData::new(
        ErrorCode::ArgumentMissing,
        "name:: value cannot be empty".to_string(),
      ) ),
    _ =>
      std::fs::read_to_string( credential_store.join( crate::account::active_marker_filename() ) )
        .ok()
        .map( | s | s.trim().to_string() )
        .filter( | s | !s.is_empty() )
        .ok_or_else( || ErrorData::new(
          ErrorCode::InternalError,
          "name:: omitted and no active account — set an active account via .account.use or pass name:: explicitly".to_string(),
        ) )?,
  };
  let name             = resolve_account_name( &raw_name, &credential_store )?;
  crate::account::check_switch_preconditions( &name, &credential_store )
    .map_err( |e| io_err_to_error_data( &e, "account relogin" ) )?;

  // Snapshot original active — best-effort; None when marker absent.
  let original_active = std::fs::read_to_string( credential_store.join( crate::account::active_marker_filename() ) )
    .ok()
    .map( | s | s.trim().to_string() )
    .filter( | s | !s.is_empty() );

  if is_dry( &cmd )
  {
    return Ok( OutputData::new(
      format!( "[dry-run] would re-authenticate '{name}' via browser login\n" ),
      "text",
    ) );
  }

  // Make the named account the live session so `claude` picks up its refreshToken.
  crate::account::switch_account( &name, &credential_store, &paths )
    .map_err( |e| io_err_to_error_data( &e, "account relogin: switch" ) )?;

  // Snapshot credentials content before spawning.
  let creds_path   = paths.credentials_file();
  let before_creds = std::fs::read_to_string( &creds_path ).unwrap_or_default();

  // Spawn `claude` with inherited TTY — NOT run_isolated — so the user sees the browser login flow.
  // Delegates to claude_runner_core::ClaudeCommand::execute_interactive() to respect the Single
  // Execution Point Rule: all process spawning goes through claude_runner_core.
  let spawn_result = claude_runner_core::ClaudeCommand::new()
    .execute_interactive();

  if let Err( e ) = spawn_result
  {
    // Restore original before returning — switch already happened above.
    if let Some( original ) = &original_active
    {
      if original != &name
      {
        let _ = crate::account::switch_account( original, &credential_store, &paths );
      }
    }
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "cannot spawn claude: {e}" ),
    ) );
  }

  // Detect whether credentials were refreshed by comparing file content.
  let after_creds = std::fs::read_to_string( &creds_path ).unwrap_or_default();
  let updated     = after_creds != before_creds;

  if updated
  {
    // Persist the refreshed credentials into the account store.
    crate::account::save( &name, &credential_store, &paths, true, None )
      .map_err( |e| io_err_to_error_data( &e, "account relogin: save" ) )?;
  }

  // Restore the original active account (best-effort — failure is non-fatal).
  if let Some( original ) = &original_active
  {
    if original != &name
    {
      let _ = crate::account::switch_account( original, &credential_store, &paths );
    }
  }

  if !updated
  {
    // Fix(BUG-183): bare exit(3) produced no user-visible output.
    // Root cause: all other paths return OutputData, but this branch bypassed the dispatcher.
    // Pitfall: process::exit bypasses return-based output — always add eprintln before it.
    eprintln!( "relogin abandoned \u{2014} credentials unchanged for '{name}'" );
    std::process::exit( 3 );
  }

  Ok( OutputData::new( format!( "re-authenticated '{name}' — credentials saved\n" ), "text" ) )
}

/// `.account.renewal` — set or clear a billing renewal timestamp override for one or more accounts.
///
/// Writes `_renewal_at` (ISO-8601 UTC string) to `{name}.claude.json` via read-merge, or removes
/// it when `clear::1` is passed. Existing `oauthAccount` and other top-level keys are preserved.
///
/// # Errors
///
/// - Exit 1: conflicting params, no operation param provided, `from_now::` parse error.
/// - Exit 2 (via `process::exit`): named account not found.
#[ inline ]
#[ allow( clippy::too_many_lines ) ]
pub fn account_renewal_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let dry      = is_dry( &cmd );
  let trace    = crate::output::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let raw_name = require_nonempty_string_arg( &cmd, "name" )?;

  let at_val = match cmd.arguments.get( "at" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => Some( s.clone() ),
    _ => None,
  };
  let from_now_val = match cmd.arguments.get( "from_now" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => Some( s.clone() ),
    _ => None,
  };
  let clear_val = crate::output::parse_int_flag( &cmd, "clear", 0 )? != 0;

  // Validate mutual exclusion.
  if at_val.is_some() && from_now_val.is_some()
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentMissing,
      "at:: and from_now:: are mutually exclusive — provide only one".to_string(),
    ) );
  }
  if at_val.is_some() && clear_val
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentMissing,
      "at:: and clear:: are mutually exclusive — provide only one".to_string(),
    ) );
  }
  if from_now_val.is_some() && clear_val
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentMissing,
      "from_now:: and clear:: are mutually exclusive — provide only one".to_string(),
    ) );
  }
  if at_val.is_none() && from_now_val.is_none() && !clear_val
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentMissing,
      "must provide one of: at::, from_now::, clear::".to_string(),
    ) );
  }

  // Build the renewal operation.
  let op = if let Some( ts ) = at_val
  {
    crate::account::RenewalOperation::At( ts )
  }
  else if let Some( delta_str ) = from_now_val
  {
    let delta_secs = crate::account::parse_from_now_delta( &delta_str )
      .map_err( |e| ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?;
    use std::time::{ SystemTime, UNIX_EPOCH };
    let now_secs = SystemTime::now()
      .duration_since( UNIX_EPOCH )
      .unwrap_or_default()
      .as_secs();
    let target_secs = if delta_secs >= 0
    {
      now_secs.saturating_add( u64::try_from( delta_secs ).unwrap_or( 0 ) )
    }
    else
    {
      now_secs.saturating_sub( u64::try_from( -delta_secs ).unwrap_or( 0 ) )
    };
    crate::account::RenewalOperation::At( crate::account::secs_to_iso8601( target_secs ) )
  }
  else
  {
    crate::account::RenewalOperation::Clear
  };

  let credential_store = require_credential_store()?;
  if trace { eprintln!( "[trace] account.renewal  store: {}", credential_store.display() ) }

  // Resolve target account name(s).
  let names : Vec< String > = if raw_name == "all"
  {
    crate::account::list( &credential_store )
      .map_err( |e| io_err_to_error_data( &e, "account renewal" ) )?
      .into_iter()
      .map( |a| a.name )
      .collect()
  }
  else if raw_name.contains( ',' )
  {
    raw_name.split( ',' )
      .map( | s | s.trim().to_string() )
      .filter( | s | !s.is_empty() )
      .collect()
  }
  else
  {
    vec![ resolve_account_name( &raw_name, &credential_store )? ]
  };

  if trace { eprintln!( "[trace] account.renewal  targets: {names:?}" ) }

  let mut output     = String::new();
  let mut had_error  = false;
  let mut error_code = 0_i32;

  // Fix(issue-renewal-comma-prefix): comma-list tokens were used as raw strings without
  //   prefix resolution, causing `name::i9,i11` to fail when i9@host, i11@host are saved.
  // Root cause: the comma-list branch collected raw tokens; only the single-name branch
  //   called resolve_account_name(). Full emails pass through the @-fast-path unchanged.
  // Pitfall: resolve_account_name() returns ErrorData (not io::Error); match on ErrorCode
  //   to get the correct exit code (ArgumentTypeMismatch=ambiguous→1, InternalError=not found→2).
  for raw in &names
  {
    let name = match resolve_account_name( raw, &credential_store )
    {
      Ok( n )  => n,
      Err( e ) =>
      {
        had_error  = true;
        error_code = if e.code == ErrorCode::ArgumentTypeMismatch { 1 } else { 2 };
        eprintln!( "account renewal error: {raw}: {e}" );
        continue;
      }
    };
    match crate::account::account_renewal( &name, &credential_store, &op, dry )
    {
      Ok( line ) => output.push_str( &line ),
      Err( e )   =>
      {
        had_error  = true;
        error_code = if e.kind() == std::io::ErrorKind::NotFound { 2 } else { 1 };
        eprintln!( "account renewal error: {name}: {e}" );
      }
    }
  }

  if had_error
  {
    if !output.is_empty() { print!( "{output}" ); }
    std::process::exit( error_code );
  }

  Ok( OutputData::new( output, "text" ) )
}

// ── .account.inspect helpers ──────────────────────────────────────────────────

/// Snapshot data read from `{name}.claude.json` and `{name}.roles.json` for per-endpoint fallback.
struct InspectSnapshot
{
  tagged_id    : String,
  uuid         : String,
  billing_type : String,
  has_max      : bool,
  org_name     : String,
  org_uuid     : String,
  org_role     : String,
  ws_uuid      : String,
  ws_name      : String,
}

/// Resolve the account name for `.account.inspect`.
///
/// Uses `name::` if provided; falls back to the per-machine active marker file.
fn resolve_inspect_name( cmd : &VerifiedCommand, store : &std::path::Path ) -> Result< String, ErrorData >
{
  match cmd.arguments.get( "name" )
  {
    Some( Value::String( s ) ) if s.is_empty() =>
      Err( ErrorData::new( ErrorCode::ArgumentMissing, "name:: value cannot be empty".to_string() ) ),
    Some( Value::String( s ) ) => resolve_account_name( s, store ),
    _ =>
    {
      let marker = store.join( crate::account::active_marker_filename() );
      std::fs::read_to_string( &marker )
        .ok()
        .map( | s | s.trim().to_string() )
        .filter( | s | !s.is_empty() )
        .ok_or_else( || ErrorData::new(
          ErrorCode::InternalError,
          "name:: omitted and no active account — pass name:: explicitly".to_string(),
        ) )
    }
  }
}

/// Derive display label, bare status word, seconds-until-expiry, and expired flag from credentials JSON.
fn inspect_derive_status( cred_str : &str ) -> ( String, &'static str, u64, bool )
{
  use std::time::{ SystemTime, UNIX_EPOCH };
  let now_ms = u64::try_from(
    SystemTime::now().duration_since( UNIX_EPOCH ).unwrap_or_default().as_millis()
  ).unwrap_or( u64::MAX );
  let needle = "\"expiresAt\":";
  let exp_ms = cred_str.find( needle ).and_then( | pos |
  {
    let rest = cred_str[ pos + needle.len().. ].trim_start();
    let end  = rest.find( | c : char | !c.is_ascii_digit() ).unwrap_or( rest.len() );
    rest[ ..end ].parse::< u64 >().ok()
  } );
  match exp_ms
  {
    None        => ( "unknown".to_string(), "unknown", 0, false ),
    Some( exp ) if now_ms <= exp =>
    {
      let rem_s = ( exp - now_ms ) / 1000;
      let h     = rem_s / 3600;
      let m     = ( rem_s % 3600 ) / 60;
      ( format!( "🟢 valid (expires in {h}h {m}m)" ), "valid", rem_s, false )
    }
    Some( exp ) =>
    {
      let ago_s = ( now_ms - exp ) / 1000;
      let h     = ago_s / 3600;
      let m     = ( ago_s % 3600 ) / 60;
      ( format!( "🔴 expired ({h}h {m}m ago)" ), "expired", 0, true )
    }
  }
}

/// Extract the raw `accessToken` string value from a credentials JSON string.
fn extract_access_token( cred_str : &str ) -> Option< String >
{
  let pos  = cred_str.find( "\"accessToken\":" )?;
  let rest = cred_str[ pos + "\"accessToken\":".len().. ].trim_start();
  if !rest.starts_with( '"' ) { return None; }
  let inner = &rest[ 1.. ];
  let end   = inner.find( '"' )?;
  Some( inner[ ..end ].to_string() )
}

/// Call endpoint 001 (userinfo) with trace logging.
fn inspect_call_userinfo(
  tok   : &str,
  trace : bool,
  name  : &str,
) -> Result< claude_quota::UserinfoData, claude_quota::QuotaError >
{
  if tok.is_empty()
  {
    return Err( claude_quota::QuotaError::HttpTransport( "no token".to_string() ) );
  }
  if trace { eprintln!( "[trace] account.inspect  {name}  GET userinfo" ) }
  let r = claude_quota::fetch_userinfo( tok );
  if trace
  {
    match &r
    {
      Ok( u )  => eprintln!( "[trace] account.inspect  {name}  userinfo OK  tagged_id={}", u.tagged_id ),
      Err( e ) => eprintln!( "[trace] account.inspect  {name}  userinfo ERR  {e}" ),
    }
  }
  r
}

/// Call endpoint 002 (subscriptions) with trace logging.
fn inspect_call_subs(
  tok   : &str,
  trace : bool,
  name  : &str,
) -> Result< Vec< claude_quota::MembershipRaw >, claude_quota::QuotaError >
{
  if tok.is_empty()
  {
    return Err( claude_quota::QuotaError::HttpTransport( "no token".to_string() ) );
  }
  if trace { eprintln!( "[trace] account.inspect  {name}  GET subscriptions" ) }
  let r = claude_quota::fetch_subscriptions( tok );
  if trace
  {
    match &r
    {
      Ok( ms ) => eprintln!( "[trace] account.inspect  {name}  subscriptions OK  count={}", ms.len() ),
      Err( e ) => eprintln!( "[trace] account.inspect  {name}  subscriptions ERR  {e}" ),
    }
  }
  r
}

/// Call endpoint 005 (roles) with trace logging.
fn inspect_call_roles(
  tok   : &str,
  trace : bool,
  name  : &str,
) -> Result< claude_quota::ClaudeCliRolesData, claude_quota::QuotaError >
{
  if tok.is_empty()
  {
    return Err( claude_quota::QuotaError::HttpTransport( "no token".to_string() ) );
  }
  if trace { eprintln!( "[trace] account.inspect  {name}  GET roles" ) }
  let r = claude_quota::fetch_claude_cli_roles( tok );
  if trace
  {
    match &r
    {
      Ok( rd ) => eprintln!( "[trace] account.inspect  {name}  roles OK  org={}", rd.organization_name ),
      Err( e ) => eprintln!( "[trace] account.inspect  {name}  roles ERR  {e}" ),
    }
  }
  r
}

/// Build per-endpoint snapshot data from `{name}.claude.json` and `{name}.roles.json`.
fn build_inspect_snapshot( claude_json : &str, roles_json : &str ) -> InspectSnapshot
{
  let oa_pos       = claude_json.find( "\"oauthAccount\":" );
  let tagged_id    = oa_pos
    .and_then( | p | crate::account::parse_string_field( &claude_json[ p.. ], "taggedId" ) )
    .unwrap_or_default();
  let uuid         = oa_pos
    .and_then( | p | crate::account::parse_string_field( &claude_json[ p.. ], "uuid" ) )
    .unwrap_or_default();
  let billing_type = oa_pos
    .and_then( | p | crate::account::parse_string_field( &claude_json[ p.. ], "billingType" ) )
    .unwrap_or_default();
  let has_max      = claude_json.contains( "\"claude_max\"" );
  let org_name     = crate::account::parse_string_field( roles_json, "organization_name" ).unwrap_or_default();
  let org_uuid     = crate::account::parse_string_field( roles_json, "organization_uuid" ).unwrap_or_default();
  let org_role     = crate::account::parse_string_field( roles_json, "organization_role" ).unwrap_or_default();
  let ws_uuid      = crate::account::parse_string_field( roles_json, "workspace_uuid" ).unwrap_or_default();
  let ws_name      = crate::account::parse_string_field( roles_json, "workspace_name" ).unwrap_or_default();
  InspectSnapshot { tagged_id, uuid, billing_type, has_max, org_name, org_uuid, org_role, ws_uuid, ws_name }
}

/// Render `.account.inspect` output as human-readable text.
#[ allow( clippy::too_many_lines ) ]
#[ allow( clippy::format_push_string ) ]
fn format_inspect_text(
  name        : &str,
  tok_label   : &str,
  ep_userinfo : &Result< claude_quota::UserinfoData, claude_quota::QuotaError >,
  ep_subs     : &Result< Vec< claude_quota::MembershipRaw >, claude_quota::QuotaError >,
  ep_roles    : &Result< claude_quota::ClaudeCliRolesData, claude_quota::QuotaError >,
  snap        : &InspectSnapshot,
) -> String
{
  let mut out = String::new();

  // Header: account name and token status.
  out.push_str( &format!( "{:<17}{name}\n{:<17}{tok_label}\n", "Account:", "Status:" ) );

  // Userinfo fields: tagged ID and UUID (endpoint 001 or snapshot fallback).
  let ( tagged_id_s, uuid_s ) = match ep_userinfo
  {
    Ok( u )  => ( u.tagged_id.clone(), u.uuid.clone() ),
    Err( _ ) =>
    (
      if snap.tagged_id.is_empty() { "N/A".to_string() } else { format!( "{} (snapshot)", snap.tagged_id ) },
      if snap.uuid.is_empty()      { "N/A".to_string() } else { format!( "{} (snapshot)", snap.uuid ) },
    ),
  };
  out.push_str( &format!( "{:<17}{tagged_id_s}\n{:<17}{uuid_s}\n\n", "Tagged ID:", "UUID:" ) );

  // Memberships (endpoint 002).
  match ep_subs
  {
    Ok( ms ) =>
    {
      out.push_str( &format!( "{:<17}{}\n", "Memberships:", ms.len() ) );
      let sel_idx    = if ms.is_empty() { 0 } else { claude_quota::select_membership_index( ms ) };
      let show_sel   = ms.len() > 1;
      let bt_width   = ms.iter().map( | m | m.billing_type.len() ).max().unwrap_or( 4 );
      for ( i, m ) in ms.iter().enumerate()
      {
        let caps_str = if m.capabilities.is_empty() { "[]".to_string() }
          else { format!( "[{}]", m.capabilities.join( ", " ) ) };
        let marker   = if show_sel && i == sel_idx { "  \u{2190} selected" } else { "" };
        out.push_str( &format!(
          "  [{i}]  billing_type={:<bt_width$}  has_max={:<5}  capabilities={caps_str}{marker}\n",
          m.billing_type,
          if m.has_max { "true" } else { "false" },
        ) );
      }
    }
    Err( e ) => out.push_str( &format!( "{:<17}endpoint unavailable ({e})\n", "Memberships:" ) ),
  }
  out.push( '\n' );

  // Billing and Has Max (from selected membership or snapshot).
  let ( billing_s, has_max_s ) = match ep_subs
  {
    Ok( ms ) if !ms.is_empty() =>
    {
      let m = &ms[ claude_quota::select_membership_index( ms ) ];
      ( m.billing_type.clone(), if m.has_max { "yes" } else { "no" }.to_string() )
    }
    _ =>
    (
      if snap.billing_type.is_empty() { "N/A".to_string() }
        else { format!( "{} (snapshot)", snap.billing_type ) },
      if snap.billing_type.is_empty() { "N/A".to_string() }
        else { format!( "{} (snapshot)", if snap.has_max { "yes" } else { "no" } ) },
    ),
  };
  out.push_str( &format!( "{:<17}{billing_s}\n{:<17}{has_max_s}\n", "Billing:", "Has Max:" ) );

  // Org fields (endpoint 005 or snapshot fallback).
  let ( org_s, org_uuid_s, org_role_s, ws_uuid_s, ws_name_s ) = match ep_roles
  {
    Ok( r ) =>
    (
      r.organization_name.clone(),
      r.organization_uuid.clone(),
      r.organization_role.clone(),
      if r.workspace_uuid.is_empty() { "(none)".to_string() } else { r.workspace_uuid.clone() },
      if r.workspace_name.is_empty() { "(none)".to_string() } else { r.workspace_name.clone() },
    ),
    Err( _ ) =>
    (
      if snap.org_name.is_empty() { "N/A".to_string() } else { format!( "{} (snapshot)", snap.org_name ) },
      if snap.org_uuid.is_empty() { "N/A".to_string() } else { format!( "{} (snapshot)", snap.org_uuid ) },
      if snap.org_role.is_empty() { "N/A".to_string() } else { format!( "{} (snapshot)", snap.org_role ) },
      if snap.ws_uuid.is_empty()  { "(none)".to_string() } else { format!( "{} (snapshot)", snap.ws_uuid ) },
      if snap.ws_name.is_empty()  { "(none)".to_string() } else { format!( "{} (snapshot)", snap.ws_name ) },
    ),
  };
  out.push_str( &format!(
    "{:<17}{org_s}\n{:<17}{org_uuid_s}\n{:<17}{org_role_s}\n{:<17}{ws_uuid_s}\n{:<17}{ws_name_s}\n",
    "Org:", "Org UUID:", "Org Role:", "Workspace UUID:", "Workspace:",
  ) );

  out
}

/// Render `.account.inspect` output as compact JSON.
#[ allow( clippy::too_many_arguments ) ]
fn format_inspect_json(
  name        : &str,
  status      : &str,
  expires_in  : u64,
  ep_userinfo : &Result< claude_quota::UserinfoData, claude_quota::QuotaError >,
  ep_subs     : &Result< Vec< claude_quota::MembershipRaw >, claude_quota::QuotaError >,
  ep_roles    : &Result< claude_quota::ClaudeCliRolesData, claude_quota::QuotaError >,
  snap        : &InspectSnapshot,
  data_source : &str,
) -> String
{
  use crate::output::json_escape;

  let ( tagged_id, uuid ) = match ep_userinfo
  {
    Ok( u )  => ( u.tagged_id.clone(), u.uuid.clone() ),
    Err( _ ) => ( snap.tagged_id.clone(), snap.uuid.clone() ),
  };

  let sel_idx = ep_subs.as_ref().ok()
    .filter( | ms | !ms.is_empty() )
    .map_or( 0, | ms | claude_quota::select_membership_index( ms ) );

  let ms_json = match ep_subs
  {
    Ok( ms ) =>
    {
      let entries : Vec< String > = ms.iter().enumerate().map( | ( i, m ) |
      {
        let caps_json = super::shared::caps_to_json( &m.capabilities );
        format!(
          "{{\"index\":{i},\"billing_type\":\"{}\",\"has_max\":{},\"capabilities\":{caps_json},\"selected\":{}}}",
          json_escape( &m.billing_type ),
          if m.has_max { "true" } else { "false" },
          if i == sel_idx { "true" } else { "false" },
        )
      } ).collect();
      format!( "[{}]", entries.join( "," ) )
    }
    Err( _ ) => "[]".to_string(),
  };

  let ( billing_type, has_max ) = match ep_subs
  {
    Ok( ms ) if !ms.is_empty() =>
    {
      let m = &ms[ sel_idx ];
      ( m.billing_type.clone(), m.has_max )
    }
    _ => ( snap.billing_type.clone(), snap.has_max ),
  };

  let ( org_name, org_uuid, org_role, ws_uuid, ws_name ) = match ep_roles
  {
    Ok( r ) =>
      ( r.organization_name.clone(), r.organization_uuid.clone(), r.organization_role.clone(),
        r.workspace_uuid.clone(), r.workspace_name.clone() ),
    Err( _ ) =>
      ( snap.org_name.clone(), snap.org_uuid.clone(), snap.org_role.clone(),
        snap.ws_uuid.clone(), snap.ws_name.clone() ),
  };

  format!(
    "{{\
      \"account\":\"{}\",\
      \"status\":\"{status}\",\
      \"expires_in_secs\":{expires_in},\
      \"tagged_id\":\"{}\",\
      \"uuid\":\"{}\",\
      \"memberships\":{ms_json},\
      \"billing_type\":\"{}\",\
      \"has_max\":{},\
      \"organization_name\":\"{}\",\
      \"organization_uuid\":\"{}\",\
      \"organization_role\":\"{}\",\
      \"workspace_uuid\":\"{}\",\
      \"workspace_name\":\"{}\",\
      \"data_source\":\"{data_source}\"\
    }}\n",
    json_escape( name ),
    json_escape( &tagged_id ),
    json_escape( &uuid ),
    json_escape( &billing_type ),
    if has_max { "true" } else { "false" },
    json_escape( &org_name ),
    json_escape( &org_uuid ),
    json_escape( &org_role ),
    json_escape( &ws_uuid ),
    json_escape( &ws_name ),
  )
}

// ── .account.inspect ──────────────────────────────────────────────────────────

/// `.account.inspect` — show identity, subscription, and org fields for one account via live endpoints.
///
/// Calls endpoints 001 (userinfo), 002 (subscriptions), and 005 (roles) independently.
/// Falls back to local `{name}.claude.json` / `{name}.roles.json` snapshots per-endpoint on failure.
///
/// # Errors
///
/// - Exit 1: invalid `format::` value.
/// - Exit 2: credential store absent; account not found; credential file absent.
#[ inline ]
#[ allow( clippy::too_many_lines ) ]
pub fn account_inspect_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let trace   = crate::output::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let refresh = crate::output::parse_int_flag( &cmd, "refresh", 1 )?;
  let format  = match cmd.arguments.get( "format" )
  {
    Some( Value::String( s ) ) => s.clone(),
    _                          => "text".to_string(),
  };
  if !matches!( format.as_str(), "text" | "json" )
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      format!( "format:: must be `text` or `json`, got `{format}`" ),
    ) );
  }

  let credential_store = require_credential_store()?;
  let name             = resolve_inspect_name( &cmd, &credential_store )?;
  let cred_path        = credential_store.join( format!( "{name}.credentials.json" ) );

  if !cred_path.exists()
  {
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "credential file not found: {}", cred_path.display() ),
    ) );
  }

  let cred_str = std::fs::read_to_string( &cred_path )
    .map_err( |e| io_err_to_error_data( &e, "account inspect" ) )?;
  let ( tok_label, status_bare, expires_in_secs, is_expired ) = inspect_derive_status( &cred_str );
  if trace { eprintln!( "[trace] account.inspect  {name}  status: {tok_label}" ) }

  let mut live_token = extract_access_token( &cred_str );

  if is_expired && refresh != 0
  {
    if trace { eprintln!( "[trace] account.inspect  {name}  token expired → attempting refresh" ) }
    let paths     = require_claude_paths()?;
    let refreshed = crate::usage::attempt_expired_token_refresh(
      &name, &credential_store, &paths, trace, "auto", "auto",
    );
    if refreshed
    {
      if trace { eprintln!( "[trace] account.inspect  {name}  refresh OK — re-reading token" ) }
      live_token = std::fs::read_to_string( &cred_path ).ok()
        .and_then( | s | extract_access_token( &s ) );
    }
    else if trace
    {
      eprintln!( "[trace] account.inspect  {name}  refresh failed — proceeding with stale token" );
    }
  }

  let tok         = live_token.as_deref().unwrap_or( "" );
  let ep_userinfo = inspect_call_userinfo( tok, trace, &name );
  let ep_subs     = inspect_call_subs( tok, trace, &name );
  let ep_roles    = inspect_call_roles( tok, trace, &name );

  let claude_json = std::fs::read_to_string( credential_store.join( format!( "{name}.claude.json" ) ) )
    .unwrap_or_default();
  let roles_json  = std::fs::read_to_string( credential_store.join( format!( "{name}.roles.json" ) ) )
    .unwrap_or_default();
  let snap        = build_inspect_snapshot( &claude_json, &roles_json );

  let live_count  = [ ep_userinfo.is_ok(), ep_subs.is_ok(), ep_roles.is_ok() ]
    .iter().filter( |&&b| b ).count();
  let data_source = match live_count { 3 => "live", 0 => "snapshot", _ => "partial_snapshot" };

  let output = if format == "json"
  {
    format_inspect_json(
      &name, status_bare, expires_in_secs,
      &ep_userinfo, &ep_subs, &ep_roles, &snap, data_source,
    )
  }
  else
  {
    format_inspect_text( &name, &tok_label, &ep_userinfo, &ep_subs, &ep_roles, &snap )
  };

  Ok( OutputData::new( output, &format ) )
}
