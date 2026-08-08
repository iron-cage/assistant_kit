//! Integration tests: `.model.select` — retirement stub (Feature 035, task 465).
//!
//! `.model.select` is fully retired: its live get/set/reset logic (formerly backed by
//! `~/.clr/config.toml`'s `model` key) is gone. Every invocation form now returns the
//! same migration-error stub, unconditionally, exit 1 — see `src/commands/model_select.rs`.
//! Replacement: `.model scope::subprocess model::VALUE` (or `reset_model::1`) — see
//! `model_test.rs` T07/T08/T15/T20 for the absorbing command's own coverage.
//!
//! `.model.select`'s hidden-from-listing behavior (still registered/dispatchable, no
//! longer a distinct `.help`/`.` row) is covered by `dot_test.rs`'s `dot13` — not
//! duplicated here.
//!
//! ## Test Matrix
//!
//! Maps function names to the Test Matrix row in
//! `task/claude_profile/465_unified_model_command_scope_routing.md`.
//!
//! | Function | Row | Condition | P/N |
//! |----------|-----|-----------|-----|
//! | `t23_get_form_exits_1_with_migration_message`   | T23 | `.model.select` (no params) → exit 1, migration message | N |
//! | `t23_id_form_exits_1_with_migration_message`    | T23 | `.model.select id::claude-opus-4-8` → exit 1, migration message | N |
//! | `t23_reset_form_exits_1_with_migration_message` | T23 | `.model.select reset::1` → exit 1, migration message | N |

use crate::cli_runner::{ run_cs_with_env, stderr, assert_exit };
use tempfile::TempDir;

/// Substrings every `.model.select` invocation form's migration message must contain.
/// `.contains()` (not exact match) — deliberately decoupled from the top-level
/// `"Error: {e}"` wrapper `src/cli.rs` applies to every routine error, which is not
/// part of this stub's own contract.
const MIGRATION_SUBSTRINGS : &[ &str ] = &[ "model.select", "REMOVED", ".model scope::subprocess" ];

/// T23 (get form): `.model.select` with no params → exit 1, the migration message
/// naming `.model scope::subprocess ...`, and no `config.toml` written as a side effect.
#[ test ]
fn t23_get_form_exits_1_with_migration_message()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model.select" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( MIGRATION_SUBSTRINGS.iter().all( | s | err.contains( s ) ),
    "T23 (get): stderr must contain the migration message, got:\n{err}" );
  assert!( !dir.path().join( ".clr" ).join( "config.toml" ).exists(),
    "T23 (get): stub must not create config.toml as a side effect" );
}

/// T23 (`id::` form): `.model.select id::claude-opus-4-8` → exit 1, same migration
/// message — the stub ignores `id::` entirely rather than acting on it.
#[ test ]
fn t23_id_form_exits_1_with_migration_message()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model.select", "id::claude-opus-4-8" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( MIGRATION_SUBSTRINGS.iter().all( | s | err.contains( s ) ),
    "T23 (id::): stderr must contain the migration message, got:\n{err}" );
  assert!( !dir.path().join( ".clr" ).join( "config.toml" ).exists(),
    "T23 (id::): stub must not create config.toml as a side effect" );
}

/// T23 (`reset::1` form): `.model.select reset::1` → exit 1, same migration message —
/// the stub ignores `reset::1` entirely rather than acting on it.
#[ test ]
fn t23_reset_form_exits_1_with_migration_message()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model.select", "reset::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( MIGRATION_SUBSTRINGS.iter().all( | s | err.contains( s ) ),
    "T23 (reset::1): stderr must contain the migration message, got:\n{err}" );
  assert!( !dir.path().join( ".clr" ).join( "config.toml" ).exists(),
    "T23 (reset::1): stub must not create config.toml as a side effect" );
}
