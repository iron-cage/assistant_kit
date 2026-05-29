//! `.` fallback command handler.

use unilang::data::{ ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;

/// `.` handler — registered in the command registry as a hidden fallback.
///
/// The adapter intercepts `.` before it reaches the registry and redirects it
/// to `.help`, so this routine is never invoked in normal operation. It is kept
/// registered to satisfy the `hidden_from_list` registry entry and to prevent
/// "unknown command" errors if the adapter path is ever bypassed.
///
/// # Errors
///
/// Never returns an error — always succeeds with empty output.
#[ inline ]
pub fn dot_routine( _cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  Ok( OutputData::new( String::new(), "text" ) )
}
