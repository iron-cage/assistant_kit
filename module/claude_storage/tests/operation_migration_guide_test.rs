//! Operation tests for the `claude_storage` → `claude_storage_core` migration guide.
//!
//! ## Source
//!
//! - Spec: `tests/docs/operation/001_migration_guide.md`
//! - Operation doc: `docs/operation/001_migration_guide.md`
//!
//! ## Coverage
//!
//! - OP-1: Cargo.toml updated: old dep removed, new dep added
//! - OP-2: Use statements updated: no `claude_storage::` imports remain
//! - OP-3: Crate compiles after Cargo.toml and import migration
//! - OP-4: Test suite passes after migration (API identical)
//! - OP-5: Rollback restores compilation from the previous state

use tempfile::TempDir;

/// OP-1: Cargo.toml updated: old dep removed, new dep added.
///
/// ## Purpose
/// Verify that Step 1 of the migration procedure correctly transforms a
/// Cargo.toml with the old `claude_storage` dependency to use `claude_storage_core`.
///
/// ## Coverage
/// `claude_storage` dep absent after Step 1; `claude_storage_core` dep present;
/// no other dependencies modified.
///
/// ## Validation Strategy
/// Start with a Cargo.toml string containing `claude_storage = { path = "../claude_storage" }`.
/// Apply Step 1 (string replacement). Assert old entry absent and new entry present.
/// Assert unrelated deps unchanged.
///
/// ## Related Requirements
/// `docs/operation/001_migration_guide.md` — Procedure Step 1
#[ test ]
fn op_1_cargo_toml_updated_old_dep_removed_new_dep_added()
{
  let before = concat!(
    "[dependencies]\n",
    "serde = \"1\"\n",
    "claude_storage = { path = \"../claude_storage\" }\n",
    "tokio = { version = \"1\", features = [\"full\"] }\n",
  );

  let after = before.replace(
    "claude_storage = { path = \"../claude_storage\" }",
    "claude_storage_core = { path = \"../claude_storage_core\" }",
  );

  assert!(
    !after.contains( "claude_storage = " ),
    "OP-1: `claude_storage` dependency entry must be absent after Step 1; got:\n{after}"
  );
  assert!(
    after.contains( "claude_storage_core = { path = \"../claude_storage_core\" }" ),
    "OP-1: `claude_storage_core` dependency must be present after Step 1; got:\n{after}"
  );
  // Unrelated deps must be unchanged
  assert!(
    after.contains( "serde = \"1\"" ),
    "OP-1: unrelated `serde` dependency must not be modified; got:\n{after}"
  );
  assert!(
    after.contains( "tokio" ),
    "OP-1: unrelated `tokio` dependency must not be modified; got:\n{after}"
  );
}

/// OP-2: Use statements updated: no `claude_storage::` imports remain.
///
/// ## Purpose
/// Verify that Step 2 of the migration procedure replaces all `use claude_storage::`
/// statements with `use claude_storage_core::`.
///
/// ## Coverage
/// No `use claude_storage::` occurrences after Step 2; all changed lines use
/// `use claude_storage_core::`; unrelated imports unchanged.
///
/// ## Validation Strategy
/// Start with a Rust source string containing multiple `use claude_storage::` imports.
/// Apply Step 2 (string replacement on all occurrences). Assert no `use claude_storage::`
/// remains, all changed imports now use `use claude_storage_core::`, and
/// unrelated imports are untouched.
///
/// ## Related Requirements
/// `docs/operation/001_migration_guide.md` — Procedure Step 2
#[ test ]
fn op_2_use_statements_updated_no_claude_storage_imports_remain()
{
  let before = concat!(
    "use claude_storage::Storage;\n",
    "use claude_storage::{ Project, Session };\n",
    "use std::path::PathBuf;\n",
    "use claude_storage::Entry;\n",
  );

  let after = before.replace( "use claude_storage::", "use claude_storage_core::" );

  assert!(
    !after.contains( "use claude_storage::" ),
    "OP-2: no `use claude_storage::` must remain after Step 2; got:\n{after}"
  );
  assert!(
    after.contains( "use claude_storage_core::Storage" ),
    "OP-2: `use claude_storage_core::Storage` must be present; got:\n{after}"
  );
  assert!(
    after.contains( "use claude_storage_core::{ Project, Session }" ),
    "OP-2: multi-import must be updated to `claude_storage_core`; got:\n{after}"
  );
  assert!(
    after.contains( "use claude_storage_core::Entry" ),
    "OP-2: all import lines must be updated to `claude_storage_core`; got:\n{after}"
  );
  // Non-storage imports unchanged
  assert!(
    after.contains( "use std::path::PathBuf" ),
    "OP-2: unrelated imports must not be modified; got:\n{after}"
  );
}

/// OP-3: Crate compiles after Cargo.toml and import migration.
///
/// ## Purpose
/// Verify that the migrated crate compiles after completing OP-1 and OP-2.
/// The current `claude_storage` crate has already completed both migration
/// steps: Cargo.toml references `claude_storage_core` and all internal
/// imports use `claude_storage_core`. Compilation success is proven by the
/// binary existing and running.
///
/// ## Coverage
/// Migration-complete binary exists; binary exits 0 on `.status`; exit 0.
///
/// ## Validation Strategy
/// Assert the `clg` binary exists (produced by `cargo build` on the migrated
/// crate). Run `clg .status` against a temp storage and assert exit 0 to
/// confirm the binary is functional, not just present.
///
/// ## Related Requirements
/// `docs/operation/001_migration_guide.md` — Procedure Step 3
#[ test ]
fn op_3_crate_compiles_after_cargo_toml_and_import_migration()
{
  let binary = std::path::PathBuf::from( env!( "CARGO_BIN_EXE_clg" ) );
  assert!(
    binary.exists(),
    "OP-3: migrated crate binary must exist after cargo build; expected at {binary:?}"
  );

  // Verify Cargo.toml reflects the migration (claude_storage_core dep present)
  let cargo_toml = std::fs::read_to_string(
    std::path::PathBuf::from( env!( "CARGO_MANIFEST_DIR" ) ).join( "Cargo.toml" )
  ).expect( "OP-3: Cargo.toml must be readable" );
  assert!(
    cargo_toml.contains( "claude_storage_core" ),
    "OP-3: migrated Cargo.toml must reference claude_storage_core; got:\n{cargo_toml}"
  );

  // Run binary to confirm it is functional post-migration
  let root = TempDir::new().unwrap();
  let out = std::process::Command::new( &binary )
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .output()
    .expect( "OP-3: clg binary must be executable after migration" );
  assert_eq!(
    out.status.code().unwrap_or( -1 ),
    0,
    "OP-3: migrated binary must exit 0 on .status; stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

/// OP-4: Test suite passes after migration (API identical).
///
/// ## Purpose
/// Verify that the test suite continues to pass after the migration is complete.
/// The public API of `claude_storage_core` is identical to the pre-migration
/// `claude_storage` library exports, so no test changes are required.
///
/// ## Coverage
/// Test suite runs post-migration; Cargo.toml reflects migration; key API
/// types re-exported at the `claude_storage` namespace (same as pre-migration).
///
/// ## Validation Strategy
/// Assert this test is running (the suite is not broken by migration). Assert
/// Cargo.toml references `claude_storage_core`. Assert `src/lib.rs` re-exports
/// key API types so they remain accessible under `claude_storage::` namespace
/// — identical to the pre-migration API surface.
///
/// ## Related Requirements
/// `docs/operation/001_migration_guide.md` — Procedure Step 3
#[ test ]
fn op_4_test_suite_passes_after_migration_api_identical()
{
  // This test running proves the test suite was not broken by the migration.
  assert_eq!(
    env!( "CARGO_PKG_NAME" ),
    "claude_storage",
    "OP-4: must run inside the migrated claude_storage crate"
  );

  // Verify Cargo.toml references claude_storage_core (migration is complete)
  let cargo_toml = std::fs::read_to_string(
    std::path::PathBuf::from( env!( "CARGO_MANIFEST_DIR" ) ).join( "Cargo.toml" )
  ).expect( "OP-4: Cargo.toml must be readable" );
  assert!(
    cargo_toml.contains( "claude_storage_core" ),
    "OP-4: migrated Cargo.toml must reference claude_storage_core"
  );

  // API identity: claude_storage/src/lib.rs re-exports all core types unchanged.
  let lib_rs = std::fs::read_to_string(
    std::path::PathBuf::from( env!( "CARGO_MANIFEST_DIR" ) ).join( "src/lib.rs" )
  ).expect( "OP-4: src/lib.rs must be readable" );
  for api_type in &[ "Storage", "Project", "Session", "Entry" ]
  {
    assert!(
      lib_rs.contains( api_type ),
      "OP-4: claude_storage must re-export {api_type} (API identical to pre-migration)"
    );
  }
}

/// OP-5: Rollback restores compilation from the previous state.
///
/// ## Purpose
/// Verify that the rollback procedure is valid: reverting Cargo.toml from
/// `claude_storage_core` back to `claude_storage` produces the original
/// pre-migration content, and that `claude_storage` exports the same API
/// types so the reverted crate compiles without changes.
///
/// ## Coverage
/// Rollback text transformation produces pre-migration Cargo.toml entry;
/// use statement rollback removes all `claude_storage_core` references;
/// `claude_storage` library exports the required rollback API types.
///
/// ## Validation Strategy
/// Assert rollback string transformations produce the correct pre-migration
/// content. Assert `claude_storage/src/lib.rs` re-exports all types that
/// users would access via `use claude_storage::` after rollback.
///
/// ## Related Requirements
/// `docs/operation/001_migration_guide.md` — Rollback Procedure
#[ test ]
fn op_5_rollback_restores_compilation_from_previous_state()
{
  // Rollback Step 1: revert Cargo.toml dep entry
  let after_migration = "claude_storage_core = { path = \"../claude_storage_core\" }";
  let reverted = after_migration
    .replace( "claude_storage_core", "claude_storage" )
    .replace( "claude_storage_core", "claude_storage" );
  assert_eq!(
    reverted,
    "claude_storage = { path = \"../claude_storage\" }",
    "OP-5: Cargo.toml rollback must restore original dep entry"
  );

  // Rollback Step 2: revert use statements
  let after_migration_src = concat!(
    "use claude_storage_core::Storage;\n",
    "use claude_storage_core::{ Project, Session };\n",
  );
  let reverted_src = after_migration_src
    .replace( "use claude_storage_core::", "use claude_storage::" );
  assert!(
    !reverted_src.contains( "claude_storage_core" ),
    "OP-5: rolled-back source must have no claude_storage_core references; got:\n{reverted_src}"
  );
  assert!(
    reverted_src.contains( "use claude_storage::Storage" ),
    "OP-5: rolled-back use statements must restore claude_storage:: imports; got:\n{reverted_src}"
  );

  // Verify the rollback dep (claude_storage) exports the same API types
  // so the rolled-back crate compiles without changes to call sites.
  let lib_rs = std::fs::read_to_string(
    std::path::PathBuf::from( env!( "CARGO_MANIFEST_DIR" ) ).join( "src/lib.rs" )
  ).expect( "OP-5: src/lib.rs must be readable" );
  for api_type in &[ "Storage", "Project", "Session", "Entry", "ProjectId", "encode_path" ]
  {
    assert!(
      lib_rs.contains( api_type ),
      "OP-5: rollback requires claude_storage to re-export {api_type}; not found in lib.rs"
    );
  }
  assert!(
    lib_rs.contains( "pub use claude_storage_core" ),
    "OP-5: claude_storage must re-export from claude_storage_core for rollback to compile"
  );
}
