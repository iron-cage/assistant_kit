//! Edge case tests for the `unset::` parameter.
//!
//! Implements test cases from:
//! - `tests/docs/cli/param/012_unset.md` (EC-1 through EC-7)
//!
//! # Test Matrix
//!
//! | EC  | Description | Exit |
//! |-----|-------------|------|
//! | EC-1 | `key::K unset::1` removes key from settings.json | 0 |
//! | EC-2 | `key::K unset::1` for nonexistent key → exit 0 (idempotent) | 0 |
//! | EC-3 | `unset::1` without `key::` → key required | 1 |
//! | EC-4 | `key::K value::V unset::1` → value:: and unset:: mutually exclusive | 1 |
//! | EC-5 | `unset::0` (explicit disable) → treated as normal set mode | 0 |
//! | EC-6 | `unset::2` → invalid boolean value | 1 |
//! | EC-7 | `key::K unset::1 dry::1` → preview without deleting | 0 |

use tempfile::TempDir;

use crate::subprocess_helpers::{ assert_exit, run_clv_with_env, write_settings };

// ─── EC-1: unset::1 removes key from settings ────────────────────────────────

/// EC-1: `key::K` `unset::1` removes key from settings.json; other keys preserved; exit 0
#[ test ]
fn unset_ec1_removes_key_from_settings()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "theme", "dark" ), ( "model", "claude-sonnet-5" ) ] );

  let out = run_clv_with_env(
    &[ ".config", "key::theme", "unset::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( !content.contains( "\"theme\"" ),
    "theme key must be absent from settings.json after unset: {content}" );
  assert!( content.contains( "model" ),
    "other keys must be preserved after unset: {content}" );
}

// ─── EC-2: unset::1 for nonexistent key is idempotent ───────────────────────

/// EC-2: `key::K` `unset::1` for nonexistent key → exit 0 (idempotent); existing keys untouched
#[ test ]
fn unset_ec2_nonexistent_key_is_idempotent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "model", "claude-sonnet-5" ) ] );

  let out = run_clv_with_env(
    &[ ".config", "key::nonexistentKey123", "unset::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  // Existing keys must remain intact.
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "model" ),
    "existing keys must be preserved after unsetting a nonexistent key: {content}" );
}

// ─── EC-3: unset::1 without key:: → exit 1 ──────────────────────────────────

/// EC-3: `unset::1` without `key::` → `key::` is required for unset; exit 1
#[ test ]
fn unset_ec3_without_key_exits_1()
{
  let out = run_clv_with_env(
    &[ ".config", "unset::1" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
}

// ─── EC-4: value::V and unset::1 together → exit 1 ──────────────────────────

/// EC-4: `key::K` `value::V` `unset::1` → `value::` and `unset::` are mutually exclusive; exit 1
#[ test ]
fn unset_ec4_value_and_unset_mutually_exclusive_exits_1()
{
  let out = run_clv_with_env(
    &[ ".config", "key::theme", "value::dark", "unset::1" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
}

// ─── EC-5: unset::0 treated as normal set mode ───────────────────────────────

/// EC-5: `unset::0` (explicit disable) → treated as normal set mode; value written; exit 0
#[ test ]
fn unset_ec5_zero_treated_as_normal_set_mode()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clv_with_env(
    &[ ".config", "key::theme", "value::light", "unset::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "light" ),
    "settings.json must contain written value when unset::0: {content}" );
}

// ─── EC-6: unset::2 → invalid boolean → exit 1 ──────────────────────────────

/// EC-6: `unset::2` → boolean value out of range; exit 1
#[ test ]
fn unset_ec6_invalid_boolean_value_exits_1()
{
  let out = run_clv_with_env(
    &[ ".config", "key::theme", "unset::2" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
}

// ─── EC-7: dry::1 prevents deletion ──────────────────────────────────────────

/// EC-7: `key::K` `unset::1` `dry::1` → preview output; settings.json unchanged; exit 0
#[ test ]
fn unset_ec7_dry_run_prevents_deletion()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "theme", "dark" ) ] );

  let out = run_clv_with_env(
    &[ ".config", "key::theme", "unset::1", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  // Key must still be present — dry run must not delete it.
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"theme\"" ),
    "theme key must remain in settings.json after dry-run unset: {content}" );
}
