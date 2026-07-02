//! `dry::` and `force::` boolean parameter tests.
//!
//! Covers acceptance of 0/1 values, rejection of non-0/1 values (true/yes/2/-1/empty),
//! and last-wins semantics for repeated parameters.
//!
//! ## TC-N tests
//! | TC | Description | Kind |
//! |----|-------------|------|
//! | TC-020 | `dry::1` is accepted | P |
//! | TC-021 | `force::1` is accepted | P |
//! | TC-033 | `dry::true` (non-0/1 boolean) → exit 1 | N |
//! | TC-034 | `dry::yes` (non-0/1 boolean) → exit 1 | N |
//! | TC-035 | `force::true` (non-0/1 boolean) → exit 1 | N |
//! | TC-036 | `dry::0` explicitly accepted | P |
//! | TC-037 | `force::0` explicitly accepted | P |
//! | TC-493 | `dry::0 dry::1` last-wins → `dry::1` wins, shows `[dry-run]` | P |
//! | TC-494 | `dry::1 dry::0` last-wins → `dry::0` wins, file actually written | P |
//!
//! ## EC-N tests (`02_dry.md`, `03_force.md`)
//! | Function | Spec | Description | Kind |
//! |----------|------|-------------|------|
//! | `dry_ec6_2_exits_1` | `02_dry` | `dry::2` → exit 1 (out of range) | N |
//! | `dry_ec7_negative_exits_1` | `02_dry` | `dry::-1` → exit 1 (negative) | N |
//! | `dry_ec9_empty_exits_1` | `02_dry` | `dry::` → exit 1 (empty) | N |
//! | `force_ec3_2_exits_1` | `03_force` | `force::2` → exit 1 (out of range) | N |
//! | `force_ec4_negative_exits_1` | `03_force` | `force::-1` → exit 1 (negative) | N |
//! | `force_ec6_empty_exits_1` | `03_force` | `force::` → exit 1 (empty) | N |

use crate::subprocess_helpers::{ assert_container, run, out_stdout, out_stderr, code };

// TC-020: dry::1 accepted
#[ test ]
fn tc020_dry_run_param()
{
  let out = run( &[ ".version.install", "dry::1" ] );
  assert_eq!( code( &out ), 0 );
  let text = out_stdout( &out );
  assert!( text.contains( "[dry-run]" ), "must show dry-run: {text}" );
}

// TC-021: force::1 accepted
#[ test ]
fn tc021_force_param()
{
  let out = run( &[ ".version.install", "dry::1", "force::1" ] );
  assert_eq!( code( &out ), 0 );
}

// TC-033: dry::true (non-0/1 boolean) → exit 1
//
// Root Cause
//
// Parser accepted any string for `dry::` — only "1" set the flag,
// everything else silently treated as false.  `dry::true` appeared
// to enable dry-run but actually executed real operations.
//
// Why Not Caught
//
// Previous tests only used `dry::1`.  No test supplied a non-0/1 value.
//
// Fix Applied
//
// Parser rejects any `dry::` value that is not "0" or "1".
//
// Prevention
//
// These tests lock down the accepted value set for boolean params.
//
// Pitfall
//
// Silent boolean coercion is dangerous: users who type `dry::true`
// expect preview mode but get real execution.
#[ test ]
fn tc033_dry_true_rejected()
{
  let out = run( &[ ".version.install", "dry::true" ] );
  assert_eq!( code( &out ), 1, "dry::true must be rejected" );
  let err = out_stderr( &out );
  assert!( err.contains( "dry::" ), "error must mention dry::: {err}" );
}

// TC-034: dry::yes (non-0/1 boolean) → exit 1
#[ test ]
fn tc034_dry_yes_rejected()
{
  let out = run( &[ ".version.install", "dry::yes" ] );
  assert_eq!( code( &out ), 1, "dry::yes must be rejected" );
}

// TC-035: force::true (non-0/1 boolean) → exit 1
#[ test ]
fn tc035_force_true_rejected()
{
  let out = run( &[ ".version.install", "dry::1", "force::true" ] );
  assert_eq!( code( &out ), 1, "force::true must be rejected" );
  let err = out_stderr( &out );
  assert!( err.contains( "force::" ), "error must mention force::: {err}" );
}

// TC-036: dry::0 explicitly accepted
#[ test ]
fn tc036_dry_0_accepted()
{
  let out = run( &[ ".version.install", "dry::0" ] );
  // dry::0 means no dry-run — but command still runs (may exit 0 or 2)
  assert_ne!( code( &out ), 1, "dry::0 is valid, must not exit 1" );
}

// TC-037: force::0 explicitly accepted
#[ test ]
fn tc037_force_0_accepted()
{
  let out = run( &[ ".version.install", "dry::1", "force::0" ] );
  assert_eq!( code( &out ), 0, "force::0 is valid" );
}

// TC-493: dry::0 dry::1 — last occurrence wins → dry::1 active (dry-run)
//
// Root Cause
//
// The adapter implements last-occurrence-wins for all repeated params via
// `pairs.iter_mut().find(...)`. Without a test, a regression could silently
// reverse the semantics so the FIRST occurrence wins, causing dry::0 dry::1
// to run real operations while appearing to accept the dry::1 override.
//
// Why Not Caught
//
// TC-010 tests last-wins for v::, but no test covered dry:: or force::.
//
// Fix Applied
//
// Behaviour was already correct; this test locks the contract.
//
// Prevention
//
// TC-493/TC-494 together verify both directions of dry:: last-wins.
//
// Pitfall
//
// If first-wins semantics were accidentally introduced, `dry::0 dry::1` would
// silently execute destructive operations despite the user's dry::1 intention.
#[ test ]
fn tc493_dry_0_then_1_last_wins_dry_active()
{
  let out = run( &[ ".version.install", "dry::0", "dry::1" ] );
  assert_eq!( code( &out ), 0, "dry::0 dry::1 must exit 0" );
  let text = out_stdout( &out );
  assert!(
    text.contains( "[dry-run]" ),
    "dry::1 (last) must win: output must contain [dry-run]: {text}"
  );
}

// TC-494: dry::1 dry::0 — last occurrence wins → dry::0 active (no dry-run)
//
// Root Cause
//
// Same as TC-493: verifies the other direction. With dry::0 winning, the
// command attempts real execution. Uses .settings.set in an isolated tmp dir
// to avoid destructive side effects.
//
// Why Not Caught
//
// Only TC-010 tested last-wins; dry:: was not covered.
//
// Fix Applied
//
// Behaviour was already correct.
//
// Prevention
//
// Isolation via temp HOME ensures no real settings are modified.
//
// Pitfall
//
// Without this test, a regression where first-wins takes hold would mean
// dry::1 dry::0 silently enables dry-run mode, suppressing real writes.
#[ test ]
fn tc494_dry_1_then_0_last_wins_dry_inactive()
{
  assert_container();
  let dir = tempfile::TempDir::new().expect( "failed to create tmpdir" );
  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_claude_version" ) )
  .args( [ ".settings.set", "key::probe", "value::check", "dry::1", "dry::0" ] )
  .env( "HOME", dir.path() )
  .output()
  .expect( "failed to run clv" );

  // dry::0 wins → real write, so settings file must exist
  let settings_file = dir.path().join( ".claude/settings.json" );
  assert!(
    settings_file.exists(),
    "dry::0 (last) must win: settings file must be written"
  );
  // Must NOT show [dry-run] prefix
  let text = String::from_utf8_lossy( &out.stdout ).into_owned();
  assert!(
    !text.contains( "[dry-run]" ),
    "dry::0 (last) must win: output must NOT contain [dry-run]: {text}"
  );
}

// ─── 02_dry.md EC-6, EC-7, EC-9 ────────────────────────────────────────────

/// EC-6: `dry::2` → exit 1 (out of range; valid values: 0 and 1)
#[ test ]
fn dry_ec6_2_exits_1()
{
  let out = run( &[ ".version.install", "dry::2" ] );
  assert_eq!( code( &out ), 1, "dry::2 must exit 1 (out of range)" );
}

/// EC-7: `dry::-1` → exit 1 (negative value; valid values: 0 and 1)
#[ test ]
fn dry_ec7_negative_exits_1()
{
  let out = run( &[ ".version.install", "dry::-1" ] );
  assert_eq!( code( &out ), 1, "dry::-1 must exit 1 (out of range)" );
}

/// EC-9: `dry::` (empty) → exit 1
#[ test ]
fn dry_ec9_empty_exits_1()
{
  let out = run( &[ ".version.install", "dry::" ] );
  assert_eq!( code( &out ), 1, "dry:: (empty) must exit 1" );
}

// ─── 03_force.md EC-3, EC-4, EC-6 ───────────────────────────────────────────

/// EC-3: `force::2` → exit 1 (out of range; valid values: 0 and 1)
#[ test ]
fn force_ec3_2_exits_1()
{
  let out = run( &[ ".version.install", "force::2" ] );
  assert_eq!( code( &out ), 1, "force::2 must exit 1 (out of range)" );
}

/// EC-4: `force::-1` → exit 1 (negative value; valid values: 0 and 1)
#[ test ]
fn force_ec4_negative_exits_1()
{
  let out = run( &[ ".version.install", "force::-1" ] );
  assert_eq!( code( &out ), 1, "force::-1 must exit 1 (out of range)" );
}

/// EC-6: `force::` (empty) → exit 1
#[ test ]
fn force_ec6_empty_exits_1()
{
  let out = run( &[ ".version.install", "force::" ] );
  assert_eq!( code( &out ), 1, "force:: (empty) must exit 1" );
}
