//! Integration smoke tests verifying that each feature-gated re-export module
//! is accessible via the expected `assistant_kit::{domain}` path.
//!
//! Each test is gated with `#[cfg(feature = "X")]` so it only compiles and
//! runs when the corresponding Cargo feature is active.
//!
//! Run per-feature:
//! ```bash
//! cargo test -p assistant_kit --no-default-features --features profile --test integration
//! cargo test -p assistant_kit --no-default-features --features runner  --test integration
//! cargo test -p assistant_kit --no-default-features --features version --test integration
//! cargo test -p assistant_kit --no-default-features --features assets  --test integration
//! cargo test -p assistant_kit --no-default-features --features storage --test integration
//! cargo test -p assistant_kit --no-default-features --features full    --test integration
//! ```

// ─── feature: profile ───────────────────────────────────────────────────────

#[ cfg( feature = "profile" ) ]
#[ test ]
fn profile_re_exports_accessible()
{
  use assistant_kit::profile::ClaudePaths;
  let _ = core::any::TypeId::of::< ClaudePaths >();
}

// ─── feature: runner ────────────────────────────────────────────────────────

#[ cfg( feature = "runner" ) ]
#[ test ]
fn runner_re_exports_accessible()
{
  use assistant_kit::runner::VerbosityLevel;
  let _ = core::any::TypeId::of::< VerbosityLevel >();
}

// ─── feature: version ───────────────────────────────────────────────────────

#[ cfg( feature = "version" ) ]
#[ test ]
fn version_re_exports_accessible()
{
  let _ = assistant_kit::version::COMMANDS_YAML;
}

// ─── feature: assets ────────────────────────────────────────────────────────

#[ cfg( feature = "assets" ) ]
#[ test ]
fn assets_re_exports_accessible()
{
  let _ = assistant_kit::assets::COMMANDS_YAML;
}

// ─── feature: storage ───────────────────────────────────────────────────────

#[ cfg( feature = "storage" ) ]
#[ test ]
fn storage_re_exports_accessible()
{
  use assistant_kit::storage::Storage;
  let _ = core::any::TypeId::of::< Storage >();
}
