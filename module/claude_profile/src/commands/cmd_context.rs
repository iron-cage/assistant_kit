//! Environment and credentials context resolution for command handlers.

use unilang::data::{ ErrorCode, ErrorData };
use crate::output::json_escape;

/// Validate HOME is non-empty and return a `ClaudePaths`.
pub( crate ) fn require_claude_paths() -> Result< crate::ClaudePaths, ErrorData >
{
  match std::env::var( "HOME" )
  {
    Ok( home ) if !home.is_empty() =>
    {
      crate::ClaudePaths::new().ok_or_else( || ErrorData::new(
        ErrorCode::InternalError,
        "credential store path could not be resolved".to_string(),
      ) )
    }
    _ => Err( ErrorData::new( ErrorCode::InternalError, "HOME environment variable not set".to_string() ) ),
  }
}

/// Resolve the credential store path via `PersistPaths`.
pub( crate ) fn require_credential_store() -> Result< std::path::PathBuf, ErrorData >
{
  crate::PersistPaths::new()
    .map( | p | p.credential_store() )
    .map_err( |e| ErrorData::new(
      ErrorCode::InternalError,
      format!( "persistent storage unavailable: {e}" ),
    ) )
}

/// Derive the token state display strings from a raw `TokenStatus` result.
///
/// Returns `( tok_label, exp_label, exp_secs )`:
/// - `tok_label` — "valid", "expiring in Nm", "expired", or "unknown"
/// - `exp_label` — "in Xh Ym", "expired", or "(unavailable)"
/// - `exp_secs`  — seconds until expiry; `0` when expired or unavailable
pub( crate ) fn derive_token_state(
  ts : &Result< crate::token::TokenStatus, std::io::Error >,
) -> ( String, String, u64 )
{
  let tok = match ts
  {
    Ok( crate::token::TokenStatus::Valid { .. } )                => "valid".to_string(),
    Ok( crate::token::TokenStatus::ExpiringSoon { expires_in } ) =>
      format!( "expiring in {}m", expires_in.as_secs() / 60 ),
    Ok( crate::token::TokenStatus::Expired )                     => "expired".to_string(),
    Err( _ )                                                     => "unknown".to_string(),
  };
  let exp = match ts
  {
    Ok( crate::token::TokenStatus::Valid { expires_in }
      | crate::token::TokenStatus::ExpiringSoon { expires_in } ) =>
    {
      let h = expires_in.as_secs() / 3600;
      let m = ( expires_in.as_secs() % 3600 ) / 60;
      format!( "in {h}h {m}m" )
    }
    Ok( crate::token::TokenStatus::Expired ) => "expired".to_string(),
    Err( _ )                                 => "(unavailable)".to_string(),
  };
  let exp_secs = match ts
  {
    Ok( crate::token::TokenStatus::Valid { expires_in }
      | crate::token::TokenStatus::ExpiringSoon { expires_in } ) => expires_in.as_secs(),
    _ => 0,
  };
  ( tok, exp, exp_secs )
}

/// Render a `Vec<String>` capability list as a JSON array string.
///
/// Empty vec renders as `[]`. Each element is JSON-escaped.
/// Used by `credentials_status_routine` and `render_accounts_json`.
pub( crate ) fn caps_to_json( caps : &[ String ] ) -> String
{
  if caps.is_empty() { return "[]".to_string(); }
  let inner : Vec< String > = caps.iter()
    .map( | c | format!( "\"{}\"", json_escape( c ) ) )
    .collect();
  format!( "[{}]", inner.join( "," ) )
}
