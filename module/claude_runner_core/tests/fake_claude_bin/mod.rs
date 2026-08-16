//! Shared fake-`claude`-binary helper for subprocess integration tests.
//!
//! Single source of truth for the "temp dir with an executable `claude` shell script,
//! prepended to `PATH`" fixture — previously duplicated per test binary
//! (`bug_243_test.rs`, `stdin_file_test.rs`).
//!
//! Not compiled as its own test binary — lives at `tests/fake_claude_bin/mod.rs`
//! (Cargo's special-cased layout for shared integration-test support code); each
//! consumer includes it with `mod fake_claude_bin;`.

/// Return a temp dir containing a fake `claude` shell script and the augmented `PATH` value.
///
/// `body` is the script's payload after the `#!/bin/sh` shebang line.
///
/// The returned `TempDir` must be kept alive for the duration of the test — dropping it
/// removes the directory and makes the fake binary inaccessible.
#[ cfg( unix ) ]
pub fn fake_claude_dir( body : &str ) -> ( tempfile::TempDir, String )
{
  use std::os::unix::fs::PermissionsExt as _;
  let dir  = tempfile::TempDir::new().expect( "tmpdir" );
  let path = dir.path().join( "claude" );
  let script = format!( "#!/bin/sh\n{body}\n" );
  std::fs::write( &path, script.as_bytes() ).expect( "write fake-claude" );
  std::fs::set_permissions( &path, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake-claude" );
  let path_val = format!(
    "{}:{}",
    dir.path().display(),
    std::env::var( "PATH" ).unwrap_or_default(),
  );
  ( dir, path_val )
}
