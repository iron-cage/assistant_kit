//! Argument parsing and I/O error helpers for command handlers.

use unilang::data::{ ErrorCode, ErrorData };
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;

pub( crate ) fn require_nonempty_string_arg( cmd : &VerifiedCommand, name : &str ) -> Result< String, ErrorData >
{
  let val = match cmd.arguments.get( name )
  {
    Some( Value::String( s ) ) => s.clone(),
    _ => return Err( ErrorData::new( ErrorCode::ArgumentMissing, format!( "{name}:: is required" ) ) ),
  };
  if val.is_empty()
  {
    return Err( ErrorData::new( ErrorCode::ArgumentMissing, format!( "{name}:: value cannot be empty" ) ) );
  }
  Ok( val )
}

pub( crate ) fn is_dry( cmd : &VerifiedCommand ) -> bool
{
  matches!( cmd.arguments.get( "dry" ), Some( Value::Boolean( true ) ) )
}

/// Map `std::io::Error` to `ErrorData` with appropriate exit code.
///
/// - `InvalidInput` → `ArgumentTypeMismatch` (exit 1)
/// - everything else → `InternalError` (exit 2)
// core::io::ErrorKind requires the unstable `core_io` feature (rust-lang/rust#154046) — not usable on stable.
#[ allow( clippy::std_instead_of_core ) ]
pub( crate ) fn io_err_to_error_data( e : &std::io::Error, context : &str ) -> ErrorData
{
  let code = match e.kind()
  {
    std::io::ErrorKind::InvalidInput => ErrorCode::ArgumentTypeMismatch,
    _                                => ErrorCode::InternalError,
  };
  ErrorData::new( code, format!( "{context}: {e}" ) )
}

/// Resolve a raw account name: full email passes through; bare prefix is resolved via saved accounts.
///
/// - Contains `@` → returned as-is (treated as full email; downstream `validate_name` catches format errors).
/// - No `@` with path-unsafe chars (`/`, `\`, `*`) → `ArgumentTypeMismatch` (exit 1).
/// - No `@` (prefix) → prefix-match all saved account names:
///   - Exactly 1 account has a local part (before `@`) equal to `raw` → resolve to that account (exact local-part match wins).
///   - Exactly 1 prefix match → return that name.
///   - 0 matches → `InternalError` (exit 2): not found.
///   - 2+ matches → `ArgumentTypeMismatch` (exit 1): ambiguous prefix.
// Fix(BUG-262):
// Root cause: bare prefix args like `alice` were passed to `validate_name()` which rejected them
//   with exit 1 ("not an email address"), masking the correct "not found" (exit 2) outcome.
// Pitfall: Prefix resolution must occur BEFORE validate_name(); calling validate_name() on a
//   bare prefix always returns exit 1, preventing the resolver from running at all.
// Fix(BUG-264):
// Root cause: `starts_with("i1")` matched `i1@wbox.pro`, `i11@wbox.pro`, `i12@wbox.pro`, all
//   reported as ambiguous even though `i1` is an exact local-part match for `i1@wbox.pro`.
// Pitfall: Always check exact-local-part match before prefix scanning; prefix scanning is
//   only meaningful when no account's local part equals the input exactly.
pub( crate ) fn resolve_account_name( raw : &str, store : &std::path::Path ) -> Result< String, ErrorData >
{
  if raw.contains( '@' )
  {
    return Ok( raw.to_string() );
  }
  if raw.contains( '/' ) || raw.contains( '\\' ) || raw.contains( '*' )
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      format!( "account name prefix '{raw}' contains invalid characters" ),
    ) );
  }
  let accounts = crate::account::list( store )
    .map_err( |e| ErrorData::new( ErrorCode::InternalError, format!( "cannot list accounts: {e}" ) ) )?;
  // Exact local-part match: if exactly one account has a local part equal to `raw`, resolve to it.
  // This prevents `i1` from being ambiguous when both `i1@host` and `i11@host` exist.
  let exact : Vec< &str > = accounts.iter()
    .filter( | a | a.name.split_once( '@' ).is_some_and( | ( local, _ ) | local == raw ) )
    .map( | a | a.name.as_str() )
    .collect();
  if exact.len() == 1
  {
    return Ok( exact[ 0 ].to_string() );
  }
  let matches : Vec< &str > = accounts.iter()
    .filter( |a| a.name.starts_with( raw ) )
    .map( |a| a.name.as_str() )
    .collect();
  match matches.len()
  {
    1 => Ok( matches[ 0 ].to_string() ),
    0 => Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "account '{raw}' not found" ),
    ) ),
    _ => Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      format!( "ambiguous prefix '{raw}': matches {}", matches.join( ", " ) ),
    ) ),
  }
}
