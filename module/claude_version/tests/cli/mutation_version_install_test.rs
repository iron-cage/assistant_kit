//! Integration tests for `.version.install` — E5.
//!
//! | TC  | Description | P/N | Exit |
//! |-----|-------------|-----|------|
//! | 300 | `dry::1` → `[dry-run]` prefix, exit 0 | P | 0 |
//! | 301 | `version::stable dry::1` → preview shows `stable` | P | 0 |
//! | 302 | `version::1.2.3 dry::1` → preview shows exact version | P | 0 |
//! | 303 | `dry::1 force::1` → dry wins | P | 0 |
//! | 304 | `version::STABLE` → wrong case, exit 1 | N | 1 |
//! | 305 | `version::""` (empty) → exit 1 | N | 1 |
//! | 306 | `version::1.2` → two-part semver rejected | N | 1 |
//! | 307 | `version::x` → unknown alias, exit 1 | N | 1 |
//! | 308 | absent `version::` with `dry::1` → uses `stable` | P | 0 |
//! | 309 | `version::month dry::1` → resolves to pinned semver | P | 0 |
//! | 350 | `version::latest dry::1` → `autoUpdates = true` in preview | P | 0 |
//! | 351 | `version::stable dry::1` → `autoUpdates = false` in preview | P | 0 |
//! | 352 | `version::2.1.50 dry::1` → `autoUpdates = false` in preview | P | 0 |
//! | 353 | `version::latest dry::1` → previews unlock actions | P | 0 |
//! | 354 | `version::01.02.03` → leading zeros rejected | N | 1 |
//! | 355 | `version::0.0.0 dry::1` → single-zero parts valid | P | 0 |
//! | 356 | `dry::1` output mentions preferred version storage | P | 0 |
//! | 357 | `dry::1` does NOT write preference keys to settings | P | 0 |
//! | 358 | idempotent skip still stores preference | P | 0 |
//! | 359 | `version::stable dry::1` → output includes Layer 4 purge line | P | 0 |
//! | 360 | `version::latest dry::1` → output does NOT contain "purge" | P | 0 |
//! | 361 | `dry::1 format::json` → JSON output, exit 0 | P | 0 |
//! | 362 | `format::JSON` (uppercase) → exit 1 | N | 1 |
//! | 533 | `version::9.9.9` (nonexistent) → preference recorded before perform_install() attempted, regardless of outcome | P | 0 |

use tempfile::TempDir;

use crate::subprocess_helpers::{
  assert_exit, run_clv, run_clv_with_env, stderr, stdout, write_settings,
};

// ─── E5: version install ─────────────────────────────────────────────────────

// TC-300: dry::1 → [dry-run] prefix, exit 0
#[ test ]
fn tc300_version_install_dry_shows_prefix()
{
  let out = run_clv( &[ ".version.install", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry-run]" ), "must contain [dry-run]: {text}" );
}

// TC-301: version::stable dry::1 → preview shows stable
#[ test ]
fn tc301_version_install_dry_stable()
{
  let out = run_clv( &[ ".version.install", "version::stable", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "stable" ), "must contain stable: {text}" );
}

// TC-302: version::1.2.3 dry::1 → shows exact version
#[ test ]
fn tc302_version_install_dry_exact_semver()
{
  let out = run_clv( &[ ".version.install", "version::1.2.3", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "1.2.3" ), "must contain 1.2.3: {text}" );
}

// TC-303: dry::1 force::1 → dry wins
#[ test ]
fn tc303_version_install_dry_wins_over_force()
{
  let out = run_clv( &[ ".version.install", "dry::1", "force::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry-run]" ), "dry must win over force: {text}" );
}

// TC-304: version::STABLE → wrong case, exit 1
#[ test ]
fn tc304_version_install_wrong_case_exits_1()
{
  let out = run_clv( &[ ".version.install", "version::STABLE" ] );
  assert_exit( &out, 1 );
}

// TC-305: version::"" (empty) → exit 1
#[ test ]
fn tc305_version_install_empty_version_exits_1()
{
  let out = run_clv( &[ ".version.install", "version::" ] );
  assert_exit( &out, 1 );
}

// TC-306: version::1.2 → two-part semver rejected
#[ test ]
fn tc306_version_install_two_part_semver_exits_1()
{
  let out = run_clv( &[ ".version.install", "version::1.2" ] );
  assert_exit( &out, 1 );
}

// TC-307: version::x → unknown alias, exit 1
#[ test ]
fn tc307_version_install_unknown_alias_exits_1()
{
  let out = run_clv( &[ ".version.install", "version::x" ] );
  assert_exit( &out, 1 );
}

// TC-308: absent version:: with dry::1 → defaults to stable
#[ test ]
fn tc308_version_install_absent_version_defaults_to_stable()
{
  let out = run_clv( &[ ".version.install", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "stable" ), "default version must be stable: {text}" );
}

// TC-309: version::month dry::1 → resolves to pinned semver
#[ test ]
fn tc309_version_install_dry_month()
{
  let out = run_clv( &[ ".version.install", "version::month", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "month" ), "must contain alias name 'month': {text}" );
  assert!( text.contains( "2.1.74" ), "must contain resolved version 2.1.74: {text}" );
}

// TC-350: version::latest dry::1 → autoUpdates = true in preview
#[ test ]
fn tc350_version_install_dry_latest_auto_updates_true()
{
  let out = run_clv( &[ ".version.install", "version::latest", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "autoUpdates = true" ), "latest must preview autoUpdates = true: {text}" );
}

// TC-351: version::stable dry::1 → autoUpdates = false in preview
#[ test ]
fn tc351_version_install_dry_stable_auto_updates_false()
{
  let out = run_clv( &[ ".version.install", "version::stable", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "autoUpdates = false" ), "stable must preview autoUpdates = false: {text}" );
  assert!( text.contains( "DISABLE_AUTOUPDATER = 1" ), "stable must preview DISABLE_AUTOUPDATER: {text}" );
  assert!( text.contains( "chmod 555" ), "stable must preview chmod 555: {text}" );
}

// TC-352: version::2.1.50 dry::1 → autoUpdates = false in preview
#[ test ]
fn tc352_version_install_dry_semver_auto_updates_false()
{
  let out = run_clv( &[ ".version.install", "version::2.1.50", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "autoUpdates = false" ), "semver must preview autoUpdates = false: {text}" );
  assert!( text.contains( "DISABLE_AUTOUPDATER = 1" ), "semver must preview DISABLE_AUTOUPDATER: {text}" );
}

// TC-353: version::latest dry::1 → previews unlock actions
#[ test ]
fn tc353_version_install_dry_latest_shows_unlock()
{
  let out = run_clv( &[ ".version.install", "version::latest", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "remove env.DISABLE_AUTOUPDATER" ), "latest must show remove: {text}" );
  assert!( text.contains( "unlocked" ), "latest must show unlocked: {text}" );
}

// TC-513: version::2.1.78 dry::1 → previews all 3 new pin keys
#[ test ]
fn tc513_version_install_dry_pinned_shows_new_lock_keys()
{
  let out = run_clv( &[ ".version.install", "version::2.1.78", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "env.DISABLE_UPDATES = 1" ), "pinned dry-run must preview DISABLE_UPDATES: {text}" );
  assert!( text.contains( "autoUpdatesChannel = stable" ), "pinned dry-run must preview autoUpdatesChannel: {text}" );
  assert!( text.contains( "minimumVersion = 2.1.78" ), "pinned dry-run must preview minimumVersion: {text}" );
}

// TC-514: version::latest dry::1 → previews removal of all 3 new pin keys
#[ test ]
fn tc514_version_install_dry_latest_shows_new_lock_key_removal()
{
  let out = run_clv( &[ ".version.install", "version::latest", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "remove env.DISABLE_UPDATES" ), "latest dry-run must preview DISABLE_UPDATES removal: {text}" );
  assert!( text.contains( "remove autoUpdatesChannel" ), "latest dry-run must preview autoUpdatesChannel removal: {text}" );
  assert!( text.contains( "remove minimumVersion" ), "latest dry-run must preview minimumVersion removal: {text}" );
}

// TC-354: version::01.02.03 → leading zeros rejected, exit 1
//
// test_kind: bug_reproducer(leading-zeros)
//
// Root Cause: validate_version_spec only checked that semver parts were non-empty
// ASCII digits, but did not reject leading zeros.  "01" is not valid semver.
//
// Why Not Caught: All existing tests used single-digit or proper multi-digit parts.
//
// Fix Applied: Added `p.len() == 1 || !p.starts_with('0')` check to validation.
//
// Prevention: Include leading-zero variants in version-spec negative test matrix.
//
// Pitfall: The installer accepts and attempts to download leading-zero versions,
// then fails with 404.  By that time, hot_swap_binary has already deleted the
// old binary, leaving the user without any installed version.
#[ test ]
fn tc354_version_install_leading_zeros_exits_1()
{
  let out = run_clv( &[ ".version.install", "version::01.02.03" ] );
  assert_exit( &out, 1 );
  let text = stderr( &out );
  assert!( text.contains( "unknown version" ), "must reject leading zeros: {text}" );
}

// TC-355: version::0.0.0 → valid (single-digit zero parts are fine)
#[ test ]
fn tc355_version_install_zero_parts_valid_dry()
{
  let out = run_clv( &[ ".version.install", "version::0.0.0", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "0.0.0" ), "single-zero parts are valid semver: {text}" );
}

// TC-356: dry::1 output mentions preferred version storage
#[ test ]
fn tc356_version_install_dry_mentions_preferred()
{
  let out = run_clv( &[ ".version.install", "version::stable", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "preferred version" ),
    "dry-run must mention preferred version storage: {text}"
  );
}

// TC-357: dry::1 does NOT write preference keys to settings
#[ test ]
fn tc357_version_install_dry_no_preference_written()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );

  let out = run_clv_with_env(
    &[ ".version.install", "version::stable", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  // Read settings file and verify no preference keys were written.
  let content = std::fs::read_to_string( dir.path().join( ".claude/settings.json" ) ).unwrap();
  assert!(
    !content.contains( "preferredVersionSpec" ),
    "dry-run must not write preference keys: {content}"
  );
}

// TC-358: idempotent skip ("already at") still stores preference
//
// Root Cause
// version_install_handler() returned early at the idempotent guard
// BEFORE calling store_preferred_version(). After a no-op install,
// `version guard` found no preference keys and reported "no preferred version
// set" even though the user explicitly ran `version install`.
//
// Why Not Caught
// All install tests used `dry::1` or invalid PATH. No test exercised the
// idempotent early-return path with a writable HOME that could verify settings.
//
// Fix Applied
// Store preference in the idempotent early-return path, not only after install.
//
// Prevention
// Every successful exit path in install must store the preference.
//
// Pitfall
// Early returns that skip post-action bookkeeping silently break downstream
// commands that depend on that bookkeeping.
// test_kind: bug_reproducer(BUG-004)
#[ test ]
fn tc358_version_install_idempotent_stores_preference()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );

  // Run version install with real PATH. If the installed claude matches
  // stable, the idempotent guard fires ("already at ...") and we verify
  // that preference keys were written. If versions differ, install is
  // attempted — it may fail (network), but the test still checks that the
  // idempotent path stores preference when it fires.
  let out = run_clv_with_env(
    &[ ".version.install", "version::stable" ],
    &[ ( "HOME", home ) ],
  );
  let text = stdout( &out );

  if text.contains( "already at" )
  {
    // idempotent path: verify preference stored even on early return
    let content = std::fs::read_to_string( dir.path().join( ".claude/settings.json" ) ).unwrap();
    assert!(
      content.contains( "preferredVersionSpec" ),
      "idempotent skip must still store preference: {content}"
    );
  }
  else
  {
    // install ran (versions differ or install failed); assert observable output
    let err = String::from_utf8_lossy( &out.stderr );
    assert!(
      !text.is_empty() || !err.is_empty(),
      "version.install produced no output on stdout or stderr: exit {:?}",
      out.status.code()
    );
  }
}

// TC-533 (regression, MAAV-found A9): "9.9.9" is a syntactically valid but
// certainly-nonexistent version, so the idempotent-skip guard (which tc358
// exercises) never fires here — this forces the real, reordered write path
// (store_preferred_version() then perform_install()) at version.rs:149-152.
// perform_install() makes a real curl call (no injectable seam; mocking is
// against project convention) that will fail for this version regardless of
// network availability. Whatever the specific failure mode, the preference
// must already be on disk by the time it happens — proving the store call
// ran before the (failing) install attempt, not after. Soft-asserts like
// tc358 to avoid coupling to a specific network failure mode/timing.
#[ test ]
fn tc533_version_install_call_order_preference_survives_failed_install()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );

  let out = run_clv_with_env(
    &[ ".version.install", "version::9.9.9" ],
    &[ ( "HOME", home ) ],
  );
  let text = stdout( &out );

  // "9.9.9" cannot already be installed, so the idempotent-skip branch
  // cannot fire here — this always exercises the reordered write path.
  assert!(
    !text.contains( "already at" ),
    "9.9.9 must never match an already-installed version: {text}"
  );

  // Whatever perform_install()'s outcome (a network failure is expected but
  // not asserted on directly — see tc358's rationale for the same choice),
  // the preference must already be recorded, proving store_preferred_version()
  // ran before perform_install() under the corrected call order.
  let content = std::fs::read_to_string( dir.path().join( ".claude/settings.json" ) ).unwrap();
  assert!(
    content.contains( "preferredVersionSpec" ) && content.contains( "9.9.9" ),
    "preference must be recorded before perform_install() is attempted, \
     regardless of install outcome: {content}"
  );
}

// TC-359: version::stable dry::1 → output includes Layer 4 purge line
#[ test ]
fn tc359_version_install_dry_stable_includes_purge_line()
{
  let out = run_clv( &[ ".version.install", "version::stable", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "purge stale cached binaries" ),
    "pinned install must preview Layer 4 purge: {text}"
  );
}

// TC-360: version::latest dry::1 → output does NOT contain purge (latest skips Layer 4)
#[ test ]
fn tc360_version_install_dry_latest_no_purge_line()
{
  let out = run_clv( &[ ".version.install", "version::latest", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "purge" ),
    "latest install must NOT mention purge (Layer 4 is pinned-only): {text}"
  );
}

// TC-361: dry::1 format::json → JSON output, exit 0
#[ test ]
fn tc361_version_install_dry_format_json()
{
  let out = run_clv( &[ ".version.install", "dry::1", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.trim_start().starts_with( '{' ),
    "format::json dry-run must produce JSON object: {text}"
  );
}

// TC-362: format::JSON (uppercase) → exit 1
#[ test ]
fn tc362_version_install_format_uppercase_rejected()
{
  let out = run_clv( &[ ".version.install", "format::JSON" ] );
  assert_exit( &out, 1 );
}
