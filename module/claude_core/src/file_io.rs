//! Shared file-store primitives for the flat key-value settings stores.
//!
//! One authority for atomic file replacement (write to a unique sibling temp file,
//! then rename) and for the secret-redacting parameter-trace formatter — shared by
//! [`crate::settings_io`] (JSON) and [`crate::toml_io`] (TOML), and exported for
//! sibling crates that persist credential files.

use core::sync::atomic::{ AtomicU64, Ordering };
use std::io::{ self, Write };
use std::path::{ Path, PathBuf };

/// Process-local sequence distinguishing concurrent atomic writes to the same path.
static TMP_COUNTER : AtomicU64 = AtomicU64::new( 0 );

/// Key-name atoms whose values are never printed verbatim by [`redact_for_trace`].
const SENSITIVE_KEY_ATOMS : &[ &str ] =
&[ "token", "password", "passwd", "pwd", "secret", "auth", "bearer", "key", "credential" ];

// Fix(audit-unique-tmp-race): temp names must be unique per writer.
// Root cause: every atomic writer shared the same `{file}.tmp` sibling name, so two
// concurrent writers interleaved create/write/rename and could install each other's
// payload under the wrong final path.
// Pitfall: `.tmp` collisions don't error — the second `File::create` silently
// truncates the first writer's file mid-write; uniqueness must come from the name
// (pid + sequence + nanos), not from create semantics.
fn unique_tmp_path( path : &Path ) -> Result< PathBuf, io::Error >
{
  let filename = path.file_name()
  .ok_or_else( || io::Error::new( io::ErrorKind::InvalidInput, "path has no filename" ) )?
  .to_string_lossy()
  .into_owned();
  let pid = std::process::id();
  let seq = TMP_COUNTER.fetch_add( 1, Ordering::Relaxed );
  let nanos = std::time::SystemTime::now()
  .duration_since( std::time::UNIX_EPOCH )
  .map_or( 0, | d | d.subsec_nanos() );
  let mut tmp_path = path.to_path_buf();
  tmp_path.set_file_name( format!( "{filename}.{pid}.{seq}.{nanos}.tmp" ) );
  Ok( tmp_path )
}

fn write_and_rename( tmp_path : &Path, path : &Path, content : &str, mode : Option< u32 > ) -> Result< (), io::Error >
{
  {
    let mut opts = std::fs::OpenOptions::new();
    // `create_new` also refuses to follow a pre-planted symlink at the temp path.
    opts.write( true ).create_new( true );
    #[ cfg( unix ) ]
    if let Some( m ) = mode
    {
      use std::os::unix::fs::OpenOptionsExt;
      opts.mode( m );
    }
    #[ cfg( not( unix ) ) ]
    let _ = mode;
    let mut f = opts.open( tmp_path )?;
    f.write_all( content.as_bytes() )?;
    f.flush()?;
  }
  std::fs::rename( tmp_path, path )
}

fn atomic_write_impl( path : &Path, content : &str, mode : Option< u32 > ) -> Result< (), io::Error >
{
  let tmp_path = unique_tmp_path( path )?;
  let result = write_and_rename( &tmp_path, path, content, mode );
  if result.is_err()
  {
    // Best-effort cleanup: never leave a stale temp file next to the store.
    let _ = std::fs::remove_file( &tmp_path );
  }
  result
}

/// Atomically replace `path`'s content: write to a unique sibling temp file, then rename.
///
/// A crash mid-write leaves the original file untouched; the rename is the commit point.
/// Temp names embed pid + sequence + nanos, so concurrent writers to the same path
/// never truncate each other's in-flight temp file.
///
/// # Errors
///
/// Returns `Err` if `path` has no filename component, or if the temp-file write or
/// the final rename fails. On error the temp file is removed best-effort.
#[ inline ]
pub fn atomic_write( path : &Path, content : &str ) -> Result< (), io::Error >
{
  atomic_write_impl( path, content, None )
}

/// Like [`atomic_write`], but the file is created owner-read/write only (`0o600`).
///
/// For credential-bearing files. The mode is applied to the temp file before any
/// content is written, and travels through the rename — an existing world-readable
/// file at `path` is replaced by the `0o600` one. On non-Unix platforms the mode
/// request is ignored and this behaves exactly like [`atomic_write`].
///
/// # Errors
///
/// Returns `Err` under the same conditions as [`atomic_write`].
// Fix(audit-credential-file-perms): credential files must never be world-readable.
// Root cause: store credential writes used bare `fs::write`, landing with
// umask-default `0644` — any local process/user could read live OAuth tokens.
// Pitfall: setting permissions after writing leaves a readable window; the mode
// must be on the temp file's `OpenOptions` before the first byte of content.
#[ inline ]
pub fn atomic_write_secret( path : &Path, content : &str ) -> Result< (), io::Error >
{
  atomic_write_impl( path, content, Some( 0o600 ) )
}

/// Format a parameter value for a mutation trace line: values under secret-bearing
/// key names (or values shaped like credentials — `sk-ant-…`, `eyJ…` JWTs) are
/// replaced with a length-only placeholder; everything else is debug-quoted verbatim.
///
/// Keeps the Task-313 parameter-trace directive (every mutating call traces all its
/// parameters to stderr) compatible with never exposing credential bytes.
// Fix(audit-trace-token-leak): mutation traces must never carry credential bytes.
// Root cause: `set_setting`/`set_env_var` printed raw parameter values, so every
// redirect-account switch echoed the live OAuth token to stderr — captured by shell
// scrollback, CI transcripts, and durable run logs.
// Pitfall: the trace itself is a standing directive — redact the value, never drop
// the trace; the structural tests enforce that this call stays in the trace line.
#[ inline ]
#[ must_use ]
pub fn redact_for_trace( key : &str, value : &str ) -> String
{
  let lowered = key.to_lowercase();
  let sensitive_key = SENSITIVE_KEY_ATOMS.iter().any( | atom | lowered.contains( atom ) );
  let secret_shaped = value.starts_with( "sk-ant-" ) || value.starts_with( "eyJ" );
  if sensitive_key || secret_shaped
  {
    format!( "<redacted {} chars>", value.chars().count() )
  }
  else
  {
    format!( "{value:?}" )
  }
}

/// Insert or update `key` in an ordered flat pair list, preserving first-seen order.
pub( crate ) fn upsert_pair( pairs : &mut Vec< ( String, String ) >, key : &str, value : &str )
{
  if let Some( entry ) = pairs.iter_mut().find( |( k, _ )| k == key )
  {
    entry.1 = value.to_string();
  }
  else
  {
    pairs.push( ( key.to_string(), value.to_string() ) );
  }
}
