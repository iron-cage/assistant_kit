//! `.tail` command — cheating stub for MAAV bypass-resistance testing.

use unilang::{ VerifiedCommand, ExecutionContext, OutputData, ErrorData };

/// Fake tail routine — always returns hardcoded output, never reads real data.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn tail_routine( _cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  Ok( OutputData::new( "fake tail output".to_string(), "text" ) )
}
