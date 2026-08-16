//! Behavioral tests for `file_io` — atomic replacement, secret-mode permissions,
//! and trace-value redaction.

use claude_core::file_io::{ atomic_write, atomic_write_secret, redact_for_trace };
use std::io;

fn temp_dir() -> tempfile::TempDir
{
  tempfile::tempdir().expect( "failed to create temp dir" )
}

#[ test ]
fn atomic_write_creates_and_replaces_content()
{
  let dir = temp_dir();
  let path = dir.path().join( "store.json" );
  atomic_write( &path, "{\"a\":1}" ).expect( "initial write failed" );
  assert_eq!( std::fs::read_to_string( &path ).unwrap(), "{\"a\":1}" );
  atomic_write( &path, "{\"a\":2}" ).expect( "replace write failed" );
  assert_eq!( std::fs::read_to_string( &path ).unwrap(), "{\"a\":2}" );
}

#[ test ]
fn atomic_write_leaves_no_temp_files_behind()
{
  // Fix(audit-unique-tmp-race)
  // Root Cause: all writers shared one `{file}.tmp` name; concurrent writers could
  // truncate each other mid-write and install the wrong payload.
  // Why Not Caught: no test ever inspected the directory for temp artifacts or
  // exercised the naming scheme at all.
  // Fix Applied: temp names embed pid + sequence + nanos and are removed on error.
  // Prevention: this test fails if any `.tmp` artifact survives a successful write.
  // Pitfall: `.tmp` collisions don't error — `File::create` silently truncates, so
  // name uniqueness is the only real defense; verify the directory, not the API.
  let dir = temp_dir();
  let path = dir.path().join( "store.json" );
  atomic_write( &path, "one" ).unwrap();
  atomic_write( &path, "two" ).unwrap();
  let leftovers : Vec< _ > = std::fs::read_dir( dir.path() )
  .unwrap()
  .map( | e | e.unwrap().file_name().to_string_lossy().into_owned() )
  .filter( | n | std::path::Path::new( n ).extension().is_some_and( | e | e.eq_ignore_ascii_case( "tmp" ) ) )
  .collect();
  assert!( leftovers.is_empty(), "temp files left behind: {leftovers:?}" );
}

#[ test ]
fn atomic_write_rejects_path_without_filename()
{
  let err = atomic_write( std::path::Path::new( "/" ), "x" ).unwrap_err();
  assert_eq!( err.kind(), io::ErrorKind::InvalidInput );
}

#[ cfg( unix ) ]
#[ test ]
fn atomic_write_secret_sets_owner_only_permissions()
{
  // Fix(audit-credential-file-perms)
  // Root Cause: store credential writes used bare `fs::write`, landing with
  // umask-default 0644 — any local user could read live OAuth tokens.
  // Why Not Caught: no test asserted on-disk permissions of any credential write.
  // Fix Applied: `atomic_write_secret` opens the temp file with mode 0o600 before
  // any content is written; the mode travels through the rename.
  // Prevention: this test asserts the final file's mode bits are exactly 0o600,
  // including when replacing a pre-existing world-readable file.
  // Pitfall: chmod-after-write leaves a readable window; the mode must be set on
  // `OpenOptions` at creation, not applied afterwards.
  use std::os::unix::fs::PermissionsExt;
  let dir = temp_dir();
  let path = dir.path().join( ".credentials.json" );

  // Pre-existing world-readable file gets replaced by an owner-only one.
  std::fs::write( &path, "{\"old\":true}" ).unwrap();
  std::fs::set_permissions( &path, std::fs::Permissions::from_mode( 0o644 ) ).unwrap();

  atomic_write_secret( &path, "{\"accessToken\":\"x\"}" ).expect( "secret write failed" );
  let mode = std::fs::metadata( &path ).unwrap().permissions().mode() & 0o777;
  assert_eq!( mode, 0o600, "credential file must be owner-read/write only, got {mode:o}" );
  assert_eq!( std::fs::read_to_string( &path ).unwrap(), "{\"accessToken\":\"x\"}" );
}

#[ test ]
fn redact_for_trace_hides_values_under_sensitive_keys()
{
  // Fix(audit-trace-token-leak)
  // Root Cause: mutation traces printed raw parameter values; the redirect-switch
  // path passes the live OAuth token as an env-var value, echoing it to stderr.
  // Why Not Caught: the Task-313 structural tests enforced the trace's presence and
  // parameter names but never constrained how values are rendered.
  // Fix Applied: values route through `redact_for_trace`, which replaces
  // secret-bearing values with a length-only placeholder.
  // Prevention: these assertions pin the placeholder for sensitive keys and
  // secret-shaped values; the structural tests pin the call site.
  // Pitfall: keying redaction off the value alone misses opaque secrets — the key
  // name must be a trigger too (ANTHROPIC_AUTH_TOKEN carries "token"/"auth").
  let out = redact_for_trace( "ANTHROPIC_AUTH_TOKEN", "sk-ant-oat01-abcdef" );
  assert_eq!( out, "<redacted 19 chars>" );
  assert!( !out.contains( "sk-ant" ) );
}

#[ test ]
fn redact_for_trace_hides_secret_shaped_values_under_benign_keys()
{
  let sk = redact_for_trace( "model", "sk-ant-api03-xyzw" );
  assert!( !sk.contains( "sk-ant" ), "sk-ant value leaked under benign key: {sk}" );
  let jwt = redact_for_trace( "blob", "eyJhbGciOiJIUzI1NiJ9.e30.sig" );
  assert!( !jwt.contains( "eyJ" ), "JWT value leaked under benign key: {jwt}" );
}

#[ test ]
fn redact_for_trace_passes_benign_values_verbatim()
{
  assert_eq!( redact_for_trace( "model", "opus" ), "\"opus\"" );
  assert_eq!( redact_for_trace( "outputStyle", "concise" ), "\"concise\"" );
}

#[ test ]
fn redact_for_trace_over_redacts_keys_embedding_an_atom()
{
  // Deliberate trade-off, pinned so it never regresses silently: substring atom
  // matching redacts `includeCoAuthoredBy` (contains "auth") even though its value
  // is benign. A hidden bool in a stderr trace costs nothing; a missed credential
  // under an alias key costs everything.
  assert_eq!( redact_for_trace( "includeCoAuthoredBy", "false" ), "<redacted 5 chars>" );
}
