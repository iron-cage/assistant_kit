//! Account mutation command handlers: use, rotate, save, delete, unclaim.

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
  // Fix(BUG-265): is_dry() check comes after existence validation so
  //   dry-run on nonexistent accounts correctly exits 2 (not silently succeeds).
  // Root cause: is_dry() was checked before existence validation, so `dry::1` on a
  //   missing account silently returned exit 0 instead of exit 2.
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
  let set_model_str = match cmd.arguments.get( "set_model" )
  {
    None                       => None,
    Some( Value::String( s ) ) =>
    {
      crate::usage::validate_set_model( s )
        .map_err( |e| ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?;
      Some( s.clone() )
    }
    _ => return Err( ErrorData::new( ErrorCode::ArgumentTypeMismatch, "set_model:: must be a string".to_string() ) ),
  };
  let refresh          = crate::output::parse_int_flag( &cmd, "refresh", 1 )?;
  let paths            = require_claude_paths()?;
  let credential_store = require_credential_store()?;
  let name             = resolve_account_name( &raw_name, &credential_store )?;
  crate::account::check_switch_preconditions( &name, &credential_store )
    .map_err( |e| io_err_to_error_data( &e, "account use" ) )?;

  // G5: Ownership guard — non-owned accounts cannot be switched to from this machine.
  // Runs before dry::1 so that dry-run still exits 1 on ownership violation.
  // force::1 bypasses the guard (Feature 036 AC-18).
  let force = crate::output::parse_int_flag( &cmd, "force", 0 )? != 0;
  let owner = crate::account::read_owner( &credential_store, &name );
  if !force && !crate::account::is_owned( &owner )
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      format!( "ownership violation: this account is owned by {owner}" ),
    ) );
  }

  if is_dry( &cmd )
  {
    return Ok( OutputData::new( format!( "[dry-run] would switch to '{name}'\n" ), "text" ) );
  }

  // Pre-fetch quota before the switch while the target credential file is still readable.
  let mut outcome = if touch != 0
  {
    crate::usage::pre_switch_touch_ctx( &name, &credential_store, trace, &imodel_str, &effort_str )
  }
  else
  {
    crate::usage::PreSwitchOutcome::Unavailable
  };

  // Fix(BUG-213): when touch is enabled and the quota fetch failed (Unavailable),
  //   check expiresAt and attempt refresh (BUG-230) before calling switch_account().
  if touch != 0 && matches!( outcome, crate::usage::PreSwitchOutcome::Unavailable )
  {
    outcome = check_expiry_and_refresh(
      &name, &credential_store, &paths, refresh, trace, &imodel_str, &effort_str,
    );
  }

  crate::account::switch_account( &name, &credential_store, &paths )
    .map_err( |e| io_err_to_error_data( &e, "account use" ) )?;

  // Post-switch: spawn subprocess touch for all fetch-succeeded cases.
  // Fix(BUG-225): Sonnet→Opus session model override when 7d(Son) < 20%.
  // Fix(BUG-285): AlreadyActive path removed — the is_idle check used server-side
  //   resets_at as proxy for local subprocess identity (category error). Always spawn;
  //   the subprocess is idempotent and exits immediately when already active.
  match outcome
  {
    crate::usage::PreSwitchOutcome::NeedTouch( ctx ) =>
    {
      crate::usage::apply_post_switch_touch( &name, ctx, &imodel_str, &effort_str, trace, &paths, &credential_store );
    }
    crate::usage::PreSwitchOutcome::Unavailable => {}
  }

  // When set_model:: is explicit, write the requested model last (takes precedence over
  // automatic apply_model_override from apply_post_switch_touch).
  if let Some( ref sm ) = set_model_str
  {
    let model_id = crate::usage::validate_set_model( sm ).ok().flatten();
    claude_profile_core::account::set_session_model( &paths, model_id );
    if trace { eprintln!( "[trace] account.use  {name}  set_model: {sm}" ) }
  }

  Ok( OutputData::new( format!( "switched to '{name}'\n" ), "text" ) )
}

/// Check whether the target account's token is expired; attempt refresh if so.
///
/// Called only when `touch` is enabled and `pre_switch_touch_ctx()` returned `Unavailable`.
/// Returns `PreSwitchOutcome` from re-probed quota when refresh succeeds, `Unavailable`
/// when the token is not expired (or the credential file cannot be read). Exits the
/// process with code 3 when the token is expired but cannot be refreshed.
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
) -> crate::usage::PreSwitchOutcome
{
  let cred_path = credential_store.join( format!( "{name}.credentials.json" ) );
  let Ok( cred_str ) = std::fs::read_to_string( &cred_path )
  else { return crate::usage::PreSwitchOutcome::Unavailable };
  let needle     = "\"expiresAt\":";
  let expires_ms = cred_str.find( needle ).and_then( | pos |
  {
    let rest = cred_str[ pos + needle.len().. ].trim_start();
    let end  = rest.find( | c : char | !c.is_ascii_digit() ).unwrap_or( rest.len() );
    rest[ ..end ].parse::< u64 >().ok()
  } );
  let Some( exp_ms ) = expires_ms
  else { return crate::usage::PreSwitchOutcome::Unavailable };
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
    return crate::usage::PreSwitchOutcome::Unavailable;
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

  // Validate name before dry-run check so dry-run rejects invalid names
  // instead of reporting "[dry-run] would save" for names that would fail.
  crate::account::validate_name( &name )
    .map_err( | e | io_err_to_error_data( &e, "account save" ) )?;

  if is_dry( &cmd )
  {
    return Ok( OutputData::new( format!( "[dry-run] would save current credentials as '{name}'\n" ), "text" ) );
  }

  // Resolve host/role profile metadata before calling save().
  // host:: defaults to auto-captured $USER@<hostname> when omitted.
  let host_val = match cmd.arguments.get( "host" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => s.clone(),
    _ =>
    {
      let user     = std::env::var( "USER" ).unwrap_or_default();
      let hostname = crate::account::resolve_hostname();
      format!( "{user}@{hostname}" )
    }
  };
  let role_val  = match cmd.arguments.get( "role" )
  {
    Some( Value::String( s ) ) => s.clone(),
    _                          => String::new(),
  };
  // Ownership-neutral: preserves existing owner via read-merge.
  // Owner can only be set by write_owner() — no CLI-exposed set path.
  crate::account::save( &name, &credential_store, &paths, true, None, Some( &host_val ), Some( &role_val ), None )
    .map_err( |e| io_err_to_error_data( &e, "account save" ) )?;
  if trace { eprintln!( "[trace] account.save  write: OK  host={host_val}  role={role_val}" ) }

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
  // Fix(BUG-266):
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

  // G6: Ownership guard — non-owned accounts cannot be deleted from this machine.
  // Runs before dry::1 so that dry-run still exits 1 on ownership violation.
  // force::1 bypasses the guard (Feature 036 AC-19).
  let force = crate::output::parse_int_flag( &cmd, "force", 0 )? != 0;
  let owner = crate::account::read_owner( &credential_store, &name );
  if !force && !crate::account::is_owned( &owner )
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      format!( "ownership violation: this account is owned by {owner}" ),
    ) );
  }

  if is_dry( &cmd )
  {
    return Ok( OutputData::new( format!( "[dry-run] would delete account '{name}'\n" ), "text" ) );
  }

  crate::account::delete( &name, &credential_store )
    .map_err( |e| io_err_to_error_data( &e, "account delete" ) )?;
  Ok( OutputData::new( format!( "deleted account '{name}'\n" ), "text" ) )
}


