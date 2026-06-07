//! `.account.relogin` command handler.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use super::shared::{ is_dry, require_claude_paths, require_credential_store, io_err_to_error_data, resolve_account_name };

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
    crate::account::save( &name, &credential_store, &paths, true, None, None, None )
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
