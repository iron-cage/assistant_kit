//! `.model.select` command handler — REMOVED (Feature 035; merged into `.model`).
//!
//! Fully retired as a standalone command. Kept registered only as a migration-error
//! stub so existing invocations fail with actionable guidance instead of "unknown
//! command" — mirrors the `REMOVED_TOGGLES` precedent in `commands/accounts.rs`.
//! See `docs/cli/command/007_model.md § Command: 20. .model.select` for the
//! historical design and the current `.model scope::subprocess ...` replacement.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;

// ── Handler ───────────────────────────────────────────────────────────────────

/// `.model.select` — REMOVED (Feature 035). Every invocation form (get, `id::`,
/// `reset::1`) returns the same migration-error stub, unconditionally.
///
/// # Errors
///
/// Always returns `Err(ErrorData)` with `ArgumentTypeMismatch` naming the
/// `.model scope::subprocess ...` replacement syntax.
#[ inline ]
pub fn model_select_routine( _cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  Err( ErrorData::new(
    ErrorCode::ArgumentTypeMismatch,
    "model.select: REMOVED — use `.model scope::subprocess model::VALUE` (or reset_model::1) instead".to_string(),
  ) )
}
