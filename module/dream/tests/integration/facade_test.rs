//! Integration smoke tests verifying that each feature-gated re-export module
//! is accessible via the expected `dream::{domain}` path.
//!
//! Each test is gated with `#[cfg(feature = "X")]` so it only compiles and
//! runs when the corresponding Cargo feature is active.
//!
//! Run per-feature:
//! ```bash
//! cargo test -p dream --no-default-features --features common   --test integration
//! cargo test -p dream --no-default-features --features storage  --test integration
//! cargo test -p dream --no-default-features --features profile  --test integration
//! cargo test -p dream --no-default-features --features runner   --test integration
//! cargo test -p dream --no-default-features --features version  --test integration
//! cargo test -p dream --no-default-features --features assets   --test integration
//! cargo test -p dream --no-default-features --features full     --test integration
//! ```

// ─── feature: common ────────────────────────────────────────────────────────

#[ cfg( feature = "common" ) ]
#[ test ]
fn common_re_exports_accessible()
{
  use dream::common::ClaudePaths;
  let _ = core::any::TypeId::of::< ClaudePaths >();
}

// ─── feature: storage ───────────────────────────────────────────────────────

#[ cfg( feature = "storage" ) ]
#[ test ]
fn storage_re_exports_accessible()
{
  use dream::storage::Storage;
  let _ = core::any::TypeId::of::< Storage >();
}

// ─── feature: profile ───────────────────────────────────────────────────────

#[ cfg( feature = "profile" ) ]
#[ test ]
fn profile_re_exports_accessible()
{
  use dream::profile::token;
  use dream::profile::account;
  let _ = core::any::TypeId::of::< token::TokenStatus >();
  let _ = core::any::TypeId::of::< account::Account >();
}

// ─── feature: runner ────────────────────────────────────────────────────────

#[ cfg( feature = "runner" ) ]
#[ test ]
fn runner_re_exports_accessible()
{
  use dream::runner::ClaudeCommand;
  let _ = core::any::TypeId::of::< ClaudeCommand >();
}

// ─── feature: version ───────────────────────────────────────────────────────

#[ cfg( feature = "version" ) ]
#[ test ]
fn version_re_exports_accessible()
{
  use dream::version::CoreError;
  let _ = core::any::TypeId::of::< CoreError >();
}

// ─── feature: assets ────────────────────────────────────────────────────────

#[ cfg( feature = "assets" ) ]
#[ test ]
fn assets_re_exports_accessible()
{
  use dream::assets::artifact::ArtifactKind;
  let _ = core::any::TypeId::of::< ArtifactKind >();
}

// ─── feature: inventory ─────────────────────────────────────────────────────

#[ cfg( feature = "inventory" ) ]
#[ test ]
fn inventory_re_exports_accessible()
{
  use dream::inventory::inventory::Inventory;
  let _ = core::any::TypeId::of::< Inventory >();
}
