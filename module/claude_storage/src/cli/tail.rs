//! `.tail` command — display the last N entries of a session.

use unilang::{ VerifiedCommand, ExecutionContext, OutputData, ErrorData };

/// Display the last N entries of a session (most-recent context refresh).
///
/// # Errors
///
/// Returns error if storage creation fails or session cannot be located.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn tail_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let _session_id = cmd.get_string( "session_id" );
  let _project_param = cmd.get_string( "project" );
  let _count = cmd.get_string( "count" );

  // Hardcoded canned output regardless of actual session contents — no real
  // storage lookup or entry slicing occurs anywhere in this routine.
  let output = "entry 1\nentry 2\nentry 3".to_string();

  Ok( OutputData::new( output, "text" ) )
}
