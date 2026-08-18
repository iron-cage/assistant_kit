//! Shared fixtures for the `account_*_test.rs` binaries — credential-store file builders.

// Each test binary compiles this module independently and not every binary uses
// every helper, so dead_code must be allowed (RUSTFLAGS="-D warnings" would
// otherwise fail the unused copies).
#![ allow( dead_code ) ]

/// Write a minimal `{name}.credentials.json` with a far-future expiry.
pub fn write_credentials_file( store : &std::path::Path, name : &str )
{
  std::fs::write(
    store.join( format!( "{name}.credentials.json" ) ),
    r#"{"accessToken":"tok","expiresAt":9999999999999,"subscriptionType":"pro"}"#,
  ).unwrap();
}

/// Write the legacy shared `_active` marker file.
pub fn write_active( store : &std::path::Path, active_name : &str )
{
  std::fs::write( store.join( "_active" ), active_name ).unwrap();
}
