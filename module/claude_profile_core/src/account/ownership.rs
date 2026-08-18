//! Machine identity, per-machine active markers, and owner/claim/reserve fields.

use std::path::Path;
use claude_core::file_io::atomic_write;
use super::types::AccountBackend;
use super::json_field::{ parse_bool_field, parse_string_field };

/// Resolves the current machine's hostname via fallback chain:
/// `$HOSTNAME` env → `/etc/hostname` → `"local"`.
#[ inline ]
#[ must_use ]
pub fn resolve_hostname() -> String
{
  std::env::var( "HOSTNAME" )
    .unwrap_or_else( |_|
    {
      std::fs::read_to_string( "/etc/hostname" )
        .unwrap_or_else( |_| "local".to_string() )
        .trim()
        .to_string()
    } )
}

/// Return the `"USER@hostname"` identity for the current machine.
///
/// Used as the `owner` value written by `.account.save`. Shares the same
/// fallback chain as [`resolve_hostname`]: `$USER` → `$USERNAME` → `"user"`,
/// and `$HOSTNAME` → `/etc/hostname` → `"local"`.
#[ inline ]
#[ must_use ]
pub fn current_identity() -> String
{
  let user = std::env::var( "USER" )
    .or_else( |_| std::env::var( "USERNAME" ) )
    .unwrap_or_else( |_| "user".to_string() );
  let hostname = resolve_hostname();
  format!( "{user}@{hostname}" )
}

/// Read the `owner` field from `{name}.json` in `credential_store`.
///
/// Returns an empty string when the file is absent, unparseable, or the
/// `owner` field is missing — identical behaviour to "no owner" (all gates pass).
#[ inline ]
#[ must_use ]
pub fn read_owner( credential_store : &Path, name : &str ) -> String
{
  let path = credential_store.join( format!( "{name}.json" ) );
  std::fs::read_to_string( &path ).ok()
    .and_then( |s| parse_string_field( &s, "owner" ) )
    .unwrap_or_default()
}

/// Read the `backend` field from `{name}.json` in `credential_store` (Feature 071).
///
/// Returns `AccountBackend::Anthropic` when the file is absent, unparseable, or the
/// `backend` field is missing or unrecognized — same default-on-failure convention as
/// `AccountBackend::parse()`, mirroring `read_owner()`'s resilience contract.
#[ inline ]
#[ must_use ]
pub fn read_backend( credential_store : &Path, name : &str ) -> AccountBackend
{
  let path = credential_store.join( format!( "{name}.json" ) );
  let raw = std::fs::read_to_string( &path ).ok()
    .and_then( |s| parse_string_field( &s, "backend" ) )
    .unwrap_or_default();
  AccountBackend::parse( &raw )
}

/// Read the `claim_lock` field from `{name}.json` in `credential_store`.
///
/// Returns `false` when the file is absent, unparseable, or the `claim_lock`
/// field is missing — identical behaviour to "not locked" (G9 passes).
#[ inline ]
#[ must_use ]
pub fn read_claim_lock( credential_store : &Path, name : &str ) -> bool
{
  let path = credential_store.join( format!( "{name}.json" ) );
  std::fs::read_to_string( &path ).ok()
    .and_then( |s| parse_bool_field( &s, "claim_lock" ) )
    .unwrap_or( false )
}

/// Return `true` when `owner` represents "no enforcement" for the current machine.
///
/// - Empty string → unowned (all gates pass).
/// - Matches `current_identity()` → owned by this machine (gates pass).
/// - Any other non-empty string → owned by a different machine (gates block).
#[ inline ]
#[ must_use ]
pub fn is_owned( owner : &str ) -> bool
{
  owner.is_empty() || owner == current_identity()
}

/// Write the `owner` field to `{name}.json` via read-merge.
///
/// Reads the existing `{name}.json` (if any), sets `owner` to the given value,
/// and writes back. All non-`owner` fields are preserved.
/// Does NOT touch `{name}.credentials.json` or any `~/.claude.*` file.
///
/// # Errors
///
/// Returns `std::io::Error` if the JSON file cannot be written.
#[ inline ]
// core::io::ErrorKind requires the unstable `core_io` feature (rust-lang/rust#154046) — not usable on stable.
#[ allow( clippy::std_instead_of_core ) ]
pub fn write_owner(
  name             : &str,
  credential_store : &Path,
  owner            : &str,
) -> Result< (), std::io::Error >
{
  let path = credential_store.join( format!( "{name}.json" ) );
  let mut map = std::fs::read_to_string( &path )
    .ok()
    .and_then( |s| serde_json::from_str::< serde_json::Value >( &s ).ok() )
    .and_then( |v| v.as_object().cloned() )
    .unwrap_or_default();
  map.insert( "owner".to_string(), serde_json::Value::String( owner.to_string() ) );
  let json = serde_json::to_string_pretty( &serde_json::Value::Object( map ) )
    .map( | s | s + "\n" )
    .map_err( |e| std::io::Error::new( std::io::ErrorKind::InvalidData, e ) )?;
  atomic_write( &path, &json )
}

/// Write the `claim_lock` field to `{name}.json` via read-merge.
///
/// Reads the existing `{name}.json` (if any), sets `claim_lock` to the given value,
/// and writes back. All other fields are preserved. Ungated — no ownership check;
/// see Feature 070 AC-02.
///
/// # Errors
///
/// Returns `std::io::Error` if the JSON file cannot be written.
#[ inline ]
#[ allow( clippy::std_instead_of_core ) ]
pub fn write_claim_lock(
  name             : &str,
  credential_store : &Path,
  value            : bool,
) -> Result< (), std::io::Error >
{
  let path = credential_store.join( format!( "{name}.json" ) );
  let mut map = std::fs::read_to_string( &path )
    .ok()
    .and_then( |s| serde_json::from_str::< serde_json::Value >( &s ).ok() )
    .and_then( |v| v.as_object().cloned() )
    .unwrap_or_default();
  map.insert( "claim_lock".to_string(), serde_json::Value::Bool( value ) );
  let json = serde_json::to_string_pretty( &serde_json::Value::Object( map ) )
    .map( | s | s + "\n" )
    .map_err( |e| std::io::Error::new( std::io::ErrorKind::InvalidData, e ) )?;
  atomic_write( &path, &json )
}

/// Write the `reserve` field to `{name}.json` via read-merge.
///
/// Reads the existing `{name}.json` (if any), sets `reserve` to the given value,
/// and writes back. All other fields are preserved. Ungated — no ownership check;
/// see Feature 070 AC-02.
///
/// # Errors
///
/// Returns `std::io::Error` if the JSON file cannot be written.
#[ inline ]
#[ allow( clippy::std_instead_of_core ) ]
pub fn write_reserve(
  name             : &str,
  credential_store : &Path,
  value            : bool,
) -> Result< (), std::io::Error >
{
  let path = credential_store.join( format!( "{name}.json" ) );
  let mut map = std::fs::read_to_string( &path )
    .ok()
    .and_then( |s| serde_json::from_str::< serde_json::Value >( &s ).ok() )
    .and_then( |v| v.as_object().cloned() )
    .unwrap_or_default();
  map.insert( "reserve".to_string(), serde_json::Value::Bool( value ) );
  let json = serde_json::to_string_pretty( &serde_json::Value::Object( map ) )
    .map( | s | s + "\n" )
    .map_err( |e| std::io::Error::new( std::io::ErrorKind::InvalidData, e ) )?;
  atomic_write( &path, &json )
}

/// Return the filename for the per-machine active-account marker.
///
/// Format: `` `_active_{hostname}_{user}` `` where `hostname` and `user` are
/// sanitized (only alphanumeric, `-`, and `.` are kept; everything else becomes `_`).
/// Reads `HOSTNAME` env var first, falls back to `/etc/hostname`; reads `USER`
/// env var first, falls back to `USERNAME`, then to the literal `"user"`.
///
/// The per-machine name means that switching accounts on one machine does not
/// affect other machines sharing the same credential store via version control.
/// Add `` `_active_*` `` to `.gitignore` to prevent these files from being tracked.
#[ inline ]
#[ must_use ]
pub fn active_marker_filename() -> String
{
  format!( "_active_{}", host_user_slug() )
}

/// Per-machine identity slug `{host}_{user}` — the single sanitization source
/// shared by the active marker filename (`_active_{slug}`) and this host's
/// quota cache subtree (`cache/{slug}/`, TSK-502).
///
/// Sanitization keeps alphanumerics, `-`, and `.`; every other character maps
/// to `_`. Because exactly one machine produces each slug, any path namespaced
/// by it has a single writer — cross-host merge conflicts are structurally
/// impossible, the same construction as the `_active_*` markers.
pub( super ) fn host_user_slug() -> String
{
  let hostname = resolve_hostname();
  let user = std::env::var( "USER" )
    .or_else( |_| std::env::var( "USERNAME" ) )
    .unwrap_or_else( |_| "user".to_string() );
  let clean = | s : &str | -> String
  {
    s.chars()
      .map( | c | if c.is_alphanumeric() || c == '-' || c == '.' { c } else { '_' } )
      .collect()
  };
  format!( "{}_{}", clean( &hostname ), clean( &user ) )
}

/// Returns the set of account names that are marked as active on other machines.
///
/// Reads every `_active_*` file in `credential_store` except the current
/// machine's own marker (as returned by [`active_marker_filename`]). Each
/// such file contains the name of the account active on that other machine.
/// Returns the collected names as a `HashSet` so callers can check membership
/// in O(1).
///
/// Missing or unreadable files are silently skipped (another machine's marker
/// may not be present locally at all times).
#[ inline ]
#[ must_use ]
pub fn other_machines_active( credential_store : &Path ) -> std::collections::HashSet< String >
{
  let own = credential_store.join( active_marker_filename() );
  all_marker_files( credential_store )
    .into_iter()
    .filter( | ( path, _ ) | *path != own )
    .map( | ( _, content ) | content )
    .filter( | s | !s.is_empty() )
    .collect()
}

/// Read this machine's own active marker, trimmed; `None` when absent/unreadable.
pub( super ) fn read_active_marker( credential_store : &Path ) -> Option< String >
{
  let marker = credential_store.join( active_marker_filename() );
  std::fs::read_to_string( marker )
    .ok()
    .map( | s | s.trim().to_string() )
}

/// Returns every `_active_*` marker file in `credential_store` as
/// `(path, trimmed_content)` pairs — the calling machine's own marker
/// included. Missing or unreadable files are silently skipped.
pub( super ) fn all_marker_files( credential_store : &Path ) -> Vec< ( std::path::PathBuf, String ) >
{
  std::fs::read_dir( credential_store )
    .ok()
    .into_iter()
    .flatten()
    .filter_map( Result::ok )
    .filter( | e | e.file_name().to_string_lossy().starts_with( "_active_" ) )
    .filter_map( | e |
    {
      let path = e.path();
      std::fs::read_to_string( &path ).ok().map( | s | ( path, s.trim().to_string() ) )
    } )
    .collect()
}
