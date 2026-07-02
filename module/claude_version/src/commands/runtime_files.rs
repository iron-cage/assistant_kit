//! `.runtime_files` — enumerate all paths managed by `clv` at runtime.
//!
//! Outputs one absolute path per line with a trailing newline. Suitable for
//! pipeline composition: `clv .runtime_files | xargs ls -la`.
//!
//! Paths are computed from environment variables only — no subprocess spawning
//! and no disk I/O. The command succeeds even when listed files do not yet exist.
//!
//! ## Exit Codes
//!
//! | Code | Meaning |
//! |------|---------|
//! | 0 | Success |
//! | 2 | `HOME` unset or empty |

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;

/// `.runtime_files` — list all paths managed by `clv` at runtime.
///
/// Outputs one absolute path per line. Path list:
/// - `$HOME/.claude/.transient/version_history_cache.json`
///
/// # Errors
///
/// Returns `Err` with `InternalError` (exit 2) when `HOME` is unset or empty.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn runtime_files_routine( _cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let home = std::env::var( "HOME" )
    .ok()
    .filter( | h | !h.is_empty() )
    .ok_or_else( || ErrorData::new(
      ErrorCode::InternalError,
      "HOME environment variable is required".to_string(),
    ) )?;

  let cache_path = format!( "{home}/.claude/.transient/version_history_cache.json" );
  Ok( OutputData::new( format!( "{cache_path}\n" ), "text" ) )
}
