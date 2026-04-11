//! Integration smoke tests verifying that each feature-gated re-export module
//! is accessible via the expected `agent_kit::{domain}` path.
//!
//! Each test is gated with `#[cfg(feature = "X")]` so it only compiles and
//! runs when the corresponding Cargo feature is active.
//!
//! Run per-feature:
//! ```bash
//! cargo test -p agent_kit --no-default-features --features common   --test integration
//! cargo test -p agent_kit --no-default-features --features storage  --test integration
//! cargo test -p agent_kit --no-default-features --features profile  --test integration
//! cargo test -p agent_kit --no-default-features --features runner   --test integration
//! cargo test -p agent_kit --no-default-features --features manager  --test integration
//! cargo test -p agent_kit --no-default-features --features full     --test integration
//! ```

// ─── feature: common ────────────────────────────────────────────────────────

#[ cfg( feature = "common" ) ]
#[ test ]
fn common_re_exports_accessible()
{
  use agent_kit::common::ClaudePaths;
  let _ = core::any::TypeId::of::< ClaudePaths >();
}

// ─── feature: storage ───────────────────────────────────────────────────────

#[ cfg( feature = "storage" ) ]
#[ test ]
fn storage_re_exports_accessible()
{
  use agent_kit::storage::Storage;
  let _ = core::any::TypeId::of::< Storage >();
}

// ─── feature: profile ───────────────────────────────────────────────────────

#[ cfg( feature = "profile" ) ]
#[ test ]
fn profile_re_exports_accessible()
{
  use agent_kit::profile::token;
  use agent_kit::profile::account;
  let _ = core::any::TypeId::of::< token::TokenStatus >();
  let _ = core::any::TypeId::of::< account::Account >();
}

// ─── feature: runner ────────────────────────────────────────────────────────

#[ cfg( feature = "runner" ) ]
#[ test ]
fn runner_re_exports_accessible()
{
  use agent_kit::runner::ClaudeCommand;
  let _ = core::any::TypeId::of::< ClaudeCommand >();
}

// ─── feature: manager ───────────────────────────────────────────────────────

#[ cfg( feature = "manager" ) ]
#[ test ]
fn manager_re_exports_accessible()
{
  use agent_kit::manager::CoreError;
  let _ = core::any::TypeId::of::< CoreError >();
}
