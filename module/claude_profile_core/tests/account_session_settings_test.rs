//! Session-settings tests: model override on switch (BUG-225/257), `set_session_model`,
//! and `remove_session_effort` against live `~/.claude/settings.json`.
//!
//! ## Test Matrix
//!
//! | Test | Scenario |
//! |------|----------|
//! | `ft10_set_session_model_preserves_existing_keys` | set_session_model() merges model into existing settings.json without losing other keys |
//! | `ft11_set_session_model_creates_file_when_absent` | set_session_model() creates settings.json when file is absent (dir exists) |
//! | `mre_bug258_set_session_model_creates_parent_dir_when_absent` | BUG-258: set_session_model() creates ~/.claude/ dir + file when dir is absent |
//! | `it_remove_session_effort_removes_key_preserves_others` | Task 464/T01: remove_session_effort() removes effortLevel, preserves other keys |
//! | `it_remove_session_effort_noop_when_key_absent` | Task 464/T02: remove_session_effort() is a no-op when effortLevel already absent |
//! | `ft_remove_session_effort_creates_file_when_settings_absent` | Task 464/T03: remove_session_effort() creates settings.json as {} when file absent |
//! | `ft_remove_session_effort_creates_dir_when_claude_absent` | Task 464/T04: remove_session_effort() creates ~/.claude/ dir + file when dir absent |

use tempfile::TempDir;
use claude_profile_core::account;
use claude_core::ClaudePaths;

/// BUG-225 MRE: `override_session_model_to_opus` upgrades Sonnet→Opus when settings has Sonnet.
///
/// # Root Cause (BUG-225)
/// `switch_account()` restores the snapshot model unconditionally. When the account's Sonnet
/// quota is < 20%, the restored Sonnet model leaves the session on an exhausted tier.
///
/// # Why Not Caught
/// No test covered save-with-Sonnet → deplete-Sonnet → switch → assert-session-model-opus.
///
/// # Fix Applied
/// `override_session_model_to_opus()` reads settings.json and overwrites Sonnet with Opus;
/// returns `true` when the override was applied.
///
/// # Prevention
/// This test asserts the write happens (return `true`) and the model in settings.json
/// changes to "claude-opus-4-6".
///
/// # Pitfall
/// Function is best-effort: if settings.json is missing, it creates a new object with
/// just "model": "claude-opus-4-6" — absence of settings is treated as Sonnet (model empty).
#[ doc = "bug_reproducer(BUG-225)" ]
#[ test ]
fn mre_bug225_override_session_model_to_opus_fires_when_sonnet()
{
  let tmp        = TempDir::new().unwrap();
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();
  std::fs::write( dot_claude.join( "settings.json" ), r#"{"model":"claude-sonnet-4-6","theme":"dark"}"# ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  let overrode = account::override_session_model_to_opus( &paths );

  assert!( overrode, "override must return true when model was Sonnet" );
  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) ).unwrap();
  let model = account::parse_string_field( &live, "model" );
  assert_eq!( model.as_deref(), Some( "opus" ), "model must be upgraded to opus shorthand" );
}

/// BUG-225 MRE: `override_session_model_to_opus` is a no-op when model is already Opus.
///
/// # Root Cause (BUG-225)
/// Same as above. This test verifies the inverse: when the snapshot already has Opus,
/// the override must not touch settings.json (returns `false`).
///
/// # Prevention
/// Ensures the function skips the write for already-correct models.
///
/// # Pitfall
/// A bug that unconditionally writes would fail this test by writing Opus over Opus
/// unnecessarily, but returning `true` — callers would emit spurious trace lines.
#[ doc = "bug_reproducer(BUG-225)" ]
#[ test ]
fn mre_bug225_override_session_model_to_opus_no_op_when_already_opus()
{
  let tmp        = TempDir::new().unwrap();
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();
  std::fs::write( dot_claude.join( "settings.json" ), r#"{"model":"opus"}"# ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  let overrode = account::override_session_model_to_opus( &paths );

  assert!( !overrode, "override must return false when model was already Opus" );
}

/// FT-20 MRE: `override_session_model_to_opus` handles Claude Code shorthand `"sonnet"` input
/// and writes shorthand `"opus"` (not full ID `"claude-opus-4-6"`). Also verifies BUG-286
/// fix: full-ID `"claude-opus-4-6"` is normalized to shorthand `"opus"` when model override fires.
///
/// # Root Cause (BUG-257)
/// `override_session_model_to_opus()` checked `current == "claude-sonnet-4-6"` but Claude Code
/// writes the shorthand `"sonnet"` to `~/.claude/settings.json`. The exact-string check never
/// matched production values — the session remained on Sonnet even when quota was exhausted.
/// Additionally, the write side used `"claude-opus-4-6"` (full ID) instead of `"opus"` shorthand.
///
/// # Root Cause (BUG-286)
/// `set_model::opus` writes `"claude-opus-4-6"` (full ID) to `settings.json`. When
/// `override_session_model_to_opus` ran next, gate `contains("sonnet") || is_empty()`
/// did not match `"claude-opus-4-6"` — full-ID form stayed in `settings.json` unmodified.
///
/// # Why Not Caught
/// BUG-225 tests pre-wrote the full ID `"claude-sonnet-4-6"` — not the shorthand
/// `"sonnet"` that Claude Code actually writes. The test passed while the real-world
/// path was always broken. BUG-286 was introduced when `set_model::opus` write path
/// used full ID; the `override_session_model_to_opus` read path was never updated.
///
/// # Fix Applied
/// BUG-257: read side `current == "claude-sonnet-4-6"` → `current.contains("sonnet")`;
///   write side `"claude-opus-4-6"` → `"opus"` shorthand.
/// BUG-286: gate extended with `|| current == "claude-opus-4-6"` to normalize full-ID opus.
///
/// # Prevention
/// Scenario 1 asserts BOTH return value AND written content. Scenario 2 guards the
/// full-ID sonnet path as a regression guard. Scenario 6 guards full-ID opus normalization.
///
/// # Pitfall
/// `contains("sonnet")` is intentionally broad — matches `"sonnet"`, `"claude-sonnet-4-6"`,
/// and any future sonnet variant. A `"sonnet"` substring in an opus ID would be a naming
/// regression in the Claude API, not a code concern here.
#[ doc = "bug_reproducer(BUG-257)" ]
#[ doc = "bug_reproducer(BUG-286)" ]
#[ test ]
fn mre_bug257_override_shorthand_alias()
{
  let tmp   = TempDir::new().unwrap();
  let paths = ClaudePaths::with_home( tmp.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();

  // Scenario 1: shorthand "sonnet" → must return true + write "opus"
  let settings = paths.settings_file();
  std::fs::write( &settings, r#"{"model":"sonnet"}"# ).unwrap();
  let overrode = account::override_session_model_to_opus( &paths );
  assert!( overrode, "BUG-257: override must fire for shorthand \"sonnet\" input" );
  let content = std::fs::read_to_string( &settings ).unwrap();
  assert!(
    content.contains( "\"opus\"" ) && !content.contains( "claude-opus-4-6" ),
    "BUG-257: override must write shorthand \"opus\", not full ID; got: {content}",
  );

  // Scenario 2: full ID "claude-sonnet-4-6" still fires (regression guard)
  std::fs::write( &settings, r#"{"model":"claude-sonnet-4-6"}"# ).unwrap();
  let overrode = account::override_session_model_to_opus( &paths );
  assert!( overrode, "full ID claude-sonnet-4-6 must still fire override" );

  // Scenario 3: non-sonnet model "opus" → must NOT fire
  std::fs::write( &settings, r#"{"model":"opus"}"# ).unwrap();
  let overrode = account::override_session_model_to_opus( &paths );
  assert!( !overrode, "non-sonnet model must not trigger override" );

  // Scenario 4: absent model → must fire (empty string case)
  std::fs::write( &settings, r"{}" ).unwrap();
  let overrode = account::override_session_model_to_opus( &paths );
  assert!( overrode, "absent model field must trigger override (defaults to opus)" );

  // Scenario 5: non-sonnet model "haiku" → must NOT fire (Fix(BUG-286) regression guard)
  std::fs::write( &settings, r#"{"model":"haiku"}"# ).unwrap();
  let overrode = account::override_session_model_to_opus( &paths );
  assert!( !overrode, "BUG-286: haiku model must not trigger override" );

  // Scenario 6: full-ID "claude-opus-4-6" → must fire; normalize to shorthand "opus" (Fix(BUG-286))
  // BUG: `set_model::opus` writes "claude-opus-4-6" full ID to settings.json; gate
  //   `contains("sonnet") || is_empty()` did not match it, leaving "claude-opus-4-6"
  //   in settings.json rather than normalising to "opus" shorthand on next override call.
  std::fs::write( &settings, r#"{"model":"claude-opus-4-6"}"# ).unwrap();
  let overrode = account::override_session_model_to_opus( &paths );
  assert!( overrode, "BUG-286: full-ID \"claude-opus-4-6\" must trigger override to normalize to shorthand" );
  let content = std::fs::read_to_string( &settings ).unwrap();
  assert!(
    content.contains( "\"opus\"" ) && !content.contains( "claude-opus-4-6" ),
    "BUG-286: override must write shorthand \"opus\", not full ID; got: {content}",
  );
}

/// `set_session_model()` writes the correct model ID or removes the key.
///
/// ## Scenarios
/// - `Some("claude-opus-4-6")` → writes `"model": "claude-opus-4-6"`
/// - `Some("claude-sonnet-4-6")` → writes `"model": "claude-sonnet-4-6"`
/// - `Some("claude-haiku-4-5-20251001")` → writes `"model": "claude-haiku-4-5-20251001"`
/// - `None` (default) → removes the `model` key entirely
///
/// ## Why This Test Exists
/// `set_session_model` is the exclusive mechanism for `set_model::` param — no
/// other code path writes arbitrary model IDs to `settings.json`. Testing the
/// 4 accepted values confirms write correctness and key removal.
#[ test ]
fn it_set_session_model_writes_and_removes()
{
  let tmp   = TempDir::new().unwrap();
  let paths = ClaudePaths::with_home( tmp.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  let settings = paths.settings_file();

  // opus
  std::fs::write( &settings, r"{}" ).unwrap();
  account::set_session_model( &paths, Some( "claude-opus-4-6" ) );
  let content = std::fs::read_to_string( &settings ).unwrap();
  assert!( content.contains( "\"claude-opus-4-6\"" ), "set_session_model opus must write full ID; got: {content}" );

  // sonnet
  account::set_session_model( &paths, Some( "claude-sonnet-4-6" ) );
  let content = std::fs::read_to_string( &settings ).unwrap();
  assert!( content.contains( "\"claude-sonnet-4-6\"" ), "set_session_model sonnet must write full ID; got: {content}" );

  // haiku
  account::set_session_model( &paths, Some( "claude-haiku-4-5-20251001" ) );
  let content = std::fs::read_to_string( &settings ).unwrap();
  assert!( content.contains( "\"claude-haiku-4-5-20251001\"" ), "set_session_model haiku must write full ID; got: {content}" );

  // default (None) — removes key
  account::set_session_model( &paths, None );
  let content = std::fs::read_to_string( &settings ).unwrap();
  assert!( !content.contains( "\"model\"" ), "set_session_model None must remove model key; got: {content}" );
}

/// Task 464 (T01): `remove_session_effort()` removes exactly the `effortLevel` key
/// from `~/.claude/settings.json`, preserving every other key already present —
/// the removal counterpart `set_session_effort()` lacked (unlike `set_session_model()`,
/// which already supports removal via `None`).
#[ test ]
fn it_remove_session_effort_removes_key_preserves_others()
{
  let tmp   = TempDir::new().unwrap();
  let paths = ClaudePaths::with_home( tmp.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  let settings = paths.settings_file();

  std::fs::write( &settings, r#"{"effortLevel":"high","model":"opus"}"# ).unwrap();
  account::remove_session_effort( &paths );
  let content = std::fs::read_to_string( &settings ).unwrap();
  assert!( !content.contains( "effortLevel" ), "remove_session_effort must remove the key; got: {content}" );
  assert!( content.contains( "\"opus\"" ), "remove_session_effort must preserve other keys; got: {content}" );
}

/// Task 464 (T02): `remove_session_effort()` is a no-op, not an error, when
/// `effortLevel` is already absent — mirrors `set_session_effort()`'s best-effort policy.
#[ test ]
fn it_remove_session_effort_noop_when_key_absent()
{
  let tmp   = TempDir::new().unwrap();
  let paths = ClaudePaths::with_home( tmp.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  let settings = paths.settings_file();

  std::fs::write( &settings, r#"{"model":"opus"}"# ).unwrap();
  account::remove_session_effort( &paths );
  let content = std::fs::read_to_string( &settings ).unwrap();
  assert!( content.contains( "\"opus\"" ), "remove_session_effort no-op must preserve existing keys; got: {content}" );
  assert!( !content.contains( "effortLevel" ), "remove_session_effort no-op must not introduce the key; got: {content}" );
}

// ── set_session_model ─────────────────────────────────────────────────────────

/// FT-10 (AC-10): `set_session_model()` preserves all pre-existing `settings.json` keys.
///
/// A write with `model_id = Some("claude-opus-4-6")` must NOT remove other keys
/// such as `theme` or `autoUpdaterStatus`.
#[ test ]
fn ft10_set_session_model_preserves_existing_keys()
{
  let tmp   = TempDir::new().unwrap();
  let dot   = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot ).unwrap();
  std::fs::write(
    dot.join( "settings.json" ),
    r#"{"theme":"dark","autoUpdaterStatus":"disabled"}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  claude_profile_core::account::set_session_model( &paths, Some( "claude-opus-4-6" ) );

  let content = std::fs::read_to_string( dot.join( "settings.json" ) )
    .expect( "settings.json must exist after set_session_model" );
  assert!(
    content.contains( "\"model\"" ) && content.contains( "claude-opus-4-6" ),
    "settings.json must contain the written model, got: {content}",
  );
  assert!(
    content.contains( "\"theme\"" ) && content.contains( "dark" ),
    "settings.json must preserve `theme` key, got: {content}",
  );
  assert!(
    content.contains( "\"autoUpdaterStatus\"" ) && content.contains( "disabled" ),
    "settings.json must preserve `autoUpdaterStatus` key, got: {content}",
  );
}

/// FT-11 (AC-11): `set_session_model()` creates `settings.json` when the file is absent.
///
/// When `~/.claude/settings.json` does not exist, `set_session_model()` creates it
/// containing only the requested `model` key.
#[ test ]
fn ft11_set_session_model_creates_file_when_absent()
{
  let tmp = TempDir::new().unwrap();
  let dot = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot ).unwrap();
  // settings.json intentionally absent.

  let paths = ClaudePaths::with_home( tmp.path() );
  claude_profile_core::account::set_session_model( &paths, Some( "claude-opus-4-6" ) );

  let settings = dot.join( "settings.json" );
  assert!( settings.exists(), "set_session_model must create settings.json when absent" );
  let content = std::fs::read_to_string( &settings )
    .expect( "settings.json must be readable" );
  assert!(
    content.contains( "\"model\"" ) && content.contains( "claude-opus-4-6" ),
    "created settings.json must contain the requested model, got: {content}",
  );
}

/// Task 464 (T03): `remove_session_effort()` creates `settings.json` when the file
/// is absent but `~/.claude/` exists — mirrors FT-11's `set_session_model` precedent.
#[ test ]
fn ft_remove_session_effort_creates_file_when_settings_absent()
{
  let tmp = TempDir::new().unwrap();
  let dot = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot ).unwrap();
  // settings.json intentionally absent.

  let paths = ClaudePaths::with_home( tmp.path() );
  claude_profile_core::account::remove_session_effort( &paths );

  let settings = dot.join( "settings.json" );
  assert!( settings.exists(), "remove_session_effort must create settings.json when absent" );
  let content = std::fs::read_to_string( &settings ).expect( "settings.json must be readable" );
  assert!( content.trim() == "{}", "created settings.json must be an empty object, got: {content}" );
}

/// Task 464 (T04, mirrors BUG-258's fix): `remove_session_effort()` creates
/// `~/.claude/` itself when the directory is absent, then behaves as the settings-absent case.
#[ test ]
fn ft_remove_session_effort_creates_dir_when_claude_absent()
{
  let tmp = TempDir::new().unwrap();
  let dot = tmp.path().join( ".claude" );
  assert!( !dot.exists(), "precondition: .claude/ must be absent" );

  let paths = ClaudePaths::with_home( tmp.path() );
  claude_profile_core::account::remove_session_effort( &paths );

  let settings = dot.join( "settings.json" );
  assert!( settings.exists(), "remove_session_effort must create .claude/ and settings.json when both absent" );
}

/// MRE for BUG-258: `set_session_model()` silently failed when `~/.claude/` dir absent.
///
/// ## Root Cause
/// `set_session_model()` called `fs::write(path, ...)` without first ensuring the
/// parent directory existed. When `~/.claude/` was absent, `fs::write` failed with
/// `NotFound`; `let _` silently discarded the error. The model was not written,
/// violating AC-01/AC-02/AC-03 for the `.usage` invocation path.
///
/// ## Why Not Caught
/// FT-11 tests the case where the file is absent but the directory exists (callers
/// always created the dir manually). No test started without `~/.claude/` at all.
///
/// ## Fix Applied
/// `set_session_model()` now calls `create_dir_all(path.parent())` before `fs::write`.
///
/// ## Prevention
/// Precondition `assert!(!dot.exists())` confirms the directory is truly absent —
/// if the fixture accidentally creates it, the test would be a false negative.
///
/// ## Pitfall
/// Unit test callers always pass `ClaudePaths::with_home(tmp.path())` with an explicit
/// `TempDir`, so they must NOT call `create_dir_all` on `~/.claude/` when testing this path.
#[ doc = "bug_reproducer(BUG-258)" ]
#[ test ]
fn mre_bug258_set_session_model_creates_parent_dir_when_absent()
{
  let tmp = TempDir::new().unwrap();
  let dot = tmp.path().join( ".claude" );
  // Precondition: ~/.claude/ must NOT exist.
  assert!(
    !dot.exists(),
    "test precondition: ~/.claude/ must not exist before calling set_session_model",
  );

  let paths = ClaudePaths::with_home( tmp.path() );
  claude_profile_core::account::set_session_model( &paths, Some( "claude-opus-4-6" ) );

  let settings = dot.join( "settings.json" );
  assert!(
    settings.exists(),
    "set_session_model must create ~/.claude/ and settings.json when parent dir absent",
  );
  let content = std::fs::read_to_string( &settings )
    .expect( "settings.json must be readable after set_session_model creates parent dir" );
  assert!(
    content.contains( "\"model\"" ) && content.contains( "claude-opus-4-6" ),
    "settings.json must contain the requested model, got: {content}",
  );
}


// ── set_session_effort (Feature 062) ──────────────────────────────────────────

/// FT-09 (062): `set_session_effort()` writes `effortLevel` and preserves existing keys.
///
/// Spec: [`tests/docs/feature/62_unified_session_config.md` FT-09]
#[ test ]
fn ft09_set_session_effort_writes_effort_level()
{
  let tmp = TempDir::new().unwrap();
  let dot = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot ).unwrap();
  std::fs::write(
    dot.join( "settings.json" ),
    r#"{"theme":"dark","model":"sonnet"}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  claude_profile_core::account::set_session_effort( &paths, "max" );

  let content = std::fs::read_to_string( dot.join( "settings.json" ) )
    .expect( "settings.json must exist after set_session_effort" );
  assert!(
    content.contains( "\"effortLevel\"" ) && content.contains( "\"max\"" ),
    "FT-09: settings.json must contain effortLevel=max; got: {content}",
  );
  assert!(
    content.contains( "\"theme\"" ) && content.contains( "dark" ),
    "FT-09: set_session_effort must preserve existing 'theme' key; got: {content}",
  );
  assert!(
    content.contains( "\"model\"" ) && content.contains( "sonnet" ),
    "FT-09: set_session_effort must preserve existing 'model' key; got: {content}",
  );
}

/// FT-10 (062): `set_session_effort()` creates `~/.claude/` directory when absent.
///
/// Spec: [`tests/docs/feature/62_unified_session_config.md` FT-10]
#[ test ]
fn ft10_set_session_effort_creates_parent_dir_when_absent()
{
  let tmp = TempDir::new().unwrap();
  let dot = tmp.path().join( ".claude" );
  // Precondition: ~/.claude/ must NOT exist.
  assert!(
    !dot.exists(),
    "test precondition: ~/.claude/ must not exist before calling set_session_effort",
  );

  let paths = ClaudePaths::with_home( tmp.path() );
  claude_profile_core::account::set_session_effort( &paths, "high" );

  let settings = dot.join( "settings.json" );
  assert!(
    settings.exists(),
    "FT-10: set_session_effort must create ~/.claude/ dir and settings.json when parent dir absent",
  );
  let content = std::fs::read_to_string( &settings )
    .expect( "settings.json must be readable" );
  assert!(
    content.contains( "\"effortLevel\"" ) && content.contains( "\"high\"" ),
    "FT-10: created settings.json must contain effortLevel=high; got: {content}",
  );
}

