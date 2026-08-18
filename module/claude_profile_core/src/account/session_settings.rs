//! Live `~/.claude/settings.json` session mutations — model and effort level.

use claude_core::ClaudePaths;
use claude_core::file_io::atomic_write;
use super::json_field::parse_string_field;

/// Override the session model to Opus in `~/.claude/settings.json` when the current model is Sonnet.
///
/// Returns `true` when the override was written (current model was Sonnet or absent);
/// `false` when the model was already non-Sonnet (Opus, Haiku, etc.) — no write occurs.
///
/// Best-effort: any I/O failure is silently ignored (same policy as the `switch_account`
/// model-restore block — `settings.json` mutations must never fail the caller).
///
/// # Fix(BUG-225)
///
/// `switch_account()` restores the snapshot model unconditionally, ignoring current quota.
/// When Sonnet quota is low (< 20%), this leaves the session on Sonnet even though
/// `resolve_model(auto)` would have selected Opus. This function corrects the session model
/// after the switch, keeping it consistent with the subprocess model threshold.
///
/// # Pitfall
///
/// Only fires when quota data is available (i.e., `touch_ctx` is `Some`). When the quota
/// fetch returns 429 (`touch_ctx = None`), the model-aware upgrade cannot fire and the
/// snapshot model is used as-is. See BUG-226 for the documented limitation.
#[ must_use ]
#[ inline ]
pub fn override_session_model_to_opus( paths : &ClaudePaths ) -> bool
{
  let path = paths.settings_file();
  let mut live = std::fs::read_to_string( &path )
    .ok()
    .and_then( | s | serde_json::from_str::< serde_json::Value >( &s ).ok() )
    .unwrap_or_else( || serde_json::json!( {} ) );
  let Some( obj ) = live.as_object_mut() else { return false; };
  let current = obj.get( "model" ).and_then( | v | v.as_str() ).unwrap_or( "" );
  // Fix(BUG-257): exact match "claude-sonnet-4-6" missed shorthand "sonnet" — Claude Code
  //   writes shorthand to settings.json; full-ID check never matched production values.
  //   Write "opus" shorthand (not "claude-opus-4-6") to match Claude Code convention.
  // Root cause: BUG-225 fix used full model IDs; Claude Code stores shorthand in settings.json.
  // Pitfall: contains("sonnet") matches both "sonnet" shorthand and "claude-sonnet-4-6" full ID.
  // Fix(BUG-286): full-ID "claude-opus-4-6" was not covered by contains("sonnet") gate;
  //   `.account.use` wrote the full-ID form when re-applying model override, leaving settings.json
  //   stuck on "claude-opus-4-6" across account switches instead of re-normalising to "opus".
  // Root cause: gate only handled shorthand → shorthand normalisation; full-ID → shorthand
  //   normalisation was a missing arm.
  // Pitfall: both "opus" shorthand and "claude-opus-4-6" full-ID mean opus; the gate must
  //   treat them as equivalent to avoid skipping re-normalisation when full-ID is present.
  if current.contains( "sonnet" ) || current == "claude-opus-4-8" || current == "claude-opus-4-6" || current.is_empty()
  {
    obj.insert( "model".to_string(), serde_json::Value::String( "opus".to_string() ) );
    let _ = atomic_write( &path, &serde_json::to_string_pretty( &live ).map( | s | s + "\n" ).unwrap_or_default() );
    true
  }
  else
  {
    false
  }
}

/// Override the session model to `"sonnet"` in `~/.claude/settings.json`.
///
/// Called by `apply_model_override()` when Sonnet 7d utilization is at or above the exhaustion
/// threshold — restores the session model to Sonnet when quota allows.
///
/// Gate: only writes when the current model contains `"opus"`, equals the full-ID form
/// `"claude-sonnet-5"` or legacy `"claude-sonnet-4-6"` (shorthand normalization), or is empty.
/// Returns `true` when the file was updated, `false` when the model was already `"sonnet"`.
///
/// Mirrors `override_session_model_to_opus()` in the reverse direction.
///
/// # Fix(BUG-311)
/// Root cause: `apply_model_override()` had no sonnet-restoration path; `settings.json`
///   retained `"opus"` after switching to an account with sufficient Sonnet quota.
/// Pitfall: write `"sonnet"` shorthand (not `"claude-sonnet-5"`) — Claude Code stores shorthand.
#[ must_use ]
#[ inline ]
pub fn override_session_model_to_sonnet( paths : &ClaudePaths ) -> bool
{
  let path = paths.settings_file();
  let mut live = std::fs::read_to_string( &path )
    .ok()
    .and_then( | s | serde_json::from_str::< serde_json::Value >( &s ).ok() )
    .unwrap_or_else( || serde_json::json!( {} ) );
  let Some( obj ) = live.as_object_mut() else { return false; };
  let current = obj.get( "model" ).and_then( | v | v.as_str() ).unwrap_or( "" );
  if current.contains( "opus" ) || current == "claude-sonnet-5" || current == "claude-sonnet-4-6" || current.is_empty()
  {
    obj.insert( "model".to_string(), serde_json::Value::String( "sonnet".to_string() ) );
    let _ = atomic_write( &path, &serde_json::to_string_pretty( &live ).map( | s | s + "\n" ).unwrap_or_default() );
    true
  }
  else
  {
    false
  }
}

/// Write an explicit session model to `~/.claude/settings.json`.
///
/// `model_id` is the full model string (e.g., `"claude-opus-4-8"`).
/// Pass `None` to remove the `model` key (revert to Claude Code default).
/// Creates `~/.claude/` if it does not exist — ensures the write succeeds
/// in environments where Claude Code has not yet initialised the directory.
/// Any remaining I/O failure is silently ignored (best-effort policy).
///
/// # Fix(BUG-258)
/// Root cause: the prior implementation called `fs::write` without first creating
///   the parent directory. When `~/.claude/` was absent (fresh home, test isolation),
///   `fs::write` failed with `NotFound` and the `let _` discarded the error, silently
///   leaving `settings.json` unwritten — violating AC-01/AC-02/AC-03 for the `.usage` path.
/// Pitfall: the `.account.use` path was unaffected because `switch_account` always
///   writes `.credentials.json` to `~/.claude/`, creating the directory first. The
///   `.usage` path had no such pre-condition, making the failure path-specific.
#[ inline ]
pub fn set_session_model( paths : &ClaudePaths, model_id : Option< &str > )
{
  let path = paths.settings_file();
  // Ensure the parent directory exists before writing (Fix(BUG-258)).
  if let Some( parent ) = path.parent() { let _ = std::fs::create_dir_all( parent ); }
  let mut live = std::fs::read_to_string( &path )
    .ok()
    .and_then( | s | serde_json::from_str::< serde_json::Value >( &s ).ok() )
    .unwrap_or_else( || serde_json::json!( {} ) );
  let Some( obj ) = live.as_object_mut() else { return; };
  match model_id
  {
    Some( id ) => { obj.insert( "model".to_string(), serde_json::Value::String( id.to_string() ) ); }
    None       => { obj.remove( "model" ); }
  }
  let _ = atomic_write( &path, &serde_json::to_string_pretty( &live ).map( | s | s + "\n" ).unwrap_or_default() );
}

/// Read the current session model from `~/.claude/settings.json`.
///
/// Returns `Some(model)` when `settings.json` exists and contains a `"model"` key;
/// `None` when the file is absent, unparseable, or the `"model"` key is missing.
#[ must_use ]
#[ inline ]
pub fn get_session_model( paths : &ClaudePaths ) -> Option< String >
{
  let content = std::fs::read_to_string( paths.settings_file() ).ok()?;
  parse_string_field( &content, "model" )
}

/// Write the session effort level to `~/.claude/settings.json`.
///
/// Performs a read-modify-write preserving all existing JSON keys (same pattern as
/// `set_session_model()`). Creates `~/.claude/` if the directory is absent.
/// Any I/O failure is silently ignored (best-effort policy).
///
/// Called by the `.usage rotate::1` dispatcher (Feature 062, AC-06) to carry forward
/// the effort level after an account switch.
#[ inline ]
pub fn set_session_effort( paths : &ClaudePaths, effort_id : &str )
{
  let path = paths.settings_file();
  if let Some( parent ) = path.parent() { let _ = std::fs::create_dir_all( parent ); }
  let mut live = std::fs::read_to_string( &path )
    .ok()
    .and_then( |s| serde_json::from_str::< serde_json::Value >( &s ).ok() )
    .unwrap_or_else( || serde_json::json!( {} ) );
  let Some( obj ) = live.as_object_mut() else { return; };
  obj.insert( "effortLevel".to_string(), serde_json::Value::String( effort_id.to_string() ) );
  let _ = atomic_write( &path, &serde_json::to_string_pretty( &live ).map( |s| s + "\n" ).unwrap_or_default() );
}

/// Read the current effort level from `~/.claude/settings.json`.
///
/// Returns `Some(effort)` when `settings.json` exists and contains an `"effortLevel"` key;
/// `None` when the file is absent, unparseable, or the `"effortLevel"` key is missing.
/// `effortLevel` may be updated by `.usage rotate::1` (Feature 062, AC-06) to carry forward
/// the effort level after an account switch.
#[ must_use ]
#[ inline ]
pub fn get_session_effort( paths : &ClaudePaths ) -> Option< String >
{
  let content = std::fs::read_to_string( paths.settings_file() ).ok()?;
  parse_string_field( &content, "effortLevel" )
}

/// Remove the session effort level from `~/.claude/settings.json`.
///
/// Performs a read-modify-write preserving all existing JSON keys (same pattern as
/// `set_session_effort()`). Creates `~/.claude/` if the directory is absent. No-op,
/// not an error, when `effortLevel` is already absent. Any I/O failure is silently
/// ignored (best-effort policy).
///
/// Called by `.model reset_effort_level::1` on `scope::session` (Feature 035).
#[ inline ]
pub fn remove_session_effort( paths : &ClaudePaths )
{
  let path = paths.settings_file();
  if let Some( parent ) = path.parent() { let _ = std::fs::create_dir_all( parent ); }
  let mut live = std::fs::read_to_string( &path )
    .ok()
    .and_then( |s| serde_json::from_str::< serde_json::Value >( &s ).ok() )
    .unwrap_or_else( || serde_json::json!( {} ) );
  let Some( obj ) = live.as_object_mut() else { return; };
  obj.remove( "effortLevel" );
  let _ = atomic_write( &path, &serde_json::to_string_pretty( &live ).map( |s| s + "\n" ).unwrap_or_default() );
}
