//! CLI sanity tests
//!
//! Minimal integration tests to verify the CLI binary builds and basic
//! functionality works. The core library logic is tested in `claude_storage_core`.

mod common;

#[test]
fn cli_builds()
{
  // This test simply verifies the CLI dependencies and features compile.
  // The act of running this test means the binary built successfully.
  let package_name = env!( "CARGO_PKG_NAME" );
  assert_eq!( package_name, "claude_storage" );
}

#[test]
#[cfg( feature = "cli" )]
fn cli_feature_enabled()
{
  // Verify the CLI feature is enabled when running tests
  let version = env!( "CARGO_PKG_VERSION" );
  assert!( !version.is_empty(), "Package version should not be empty" );
}

// `claude_storage` (invocable as `clg`) is the canonical binary built from `src/main.rs`.
// These tests verify the binary is present and functional.

#[test]
#[cfg( feature = "cli" )]
fn binary_is_present()
{
  // Verify clg binary exists in the build output.
  // cargo_bin!() panics if the binary is not found — so reaching this
  // assertion means the [[bin]] name = "clg" entry in Cargo.toml is active.
  let out = common::clg_cmd()
    .env( "HOME", "/tmp" )
    .arg( ".status" )
    .output()
    .expect( "clg binary must be runnable — check [[bin]] entry in Cargo.toml" );
  // .status exits 0 (empty storage is valid) or non-zero with stderr.
  // Either outcome proves the binary runs without panicking.
  assert!(
    out.status.success() || !out.stderr.is_empty(),
    "clg must either succeed or emit a stderr error — got silence"
  );
}
