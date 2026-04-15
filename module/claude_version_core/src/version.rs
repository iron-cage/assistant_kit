//! Version management helpers for Claude Code.
//!
//! Provides version detection, alias resolution, installation, and preference persistence.
//! These are pure domain operations with no CLI framework dependencies.

use claude_core::ClaudePaths;
use claude_core::process::find_claude_processes;
use crate::settings_io::{ get_setting, set_setting, set_env_var, remove_env_var };
use crate::CoreError;

// ── Constants ─────────────────────────────────────────────────────────────────

const INSTALL_URL : &str = "https://claude.ai/install.sh";

// ── Version alias table ───────────────────────────────────────────────────────
//
// Maintenance: when bumping `month` (or any pinned alias value), update ALL
// six locations atomically — a partial update silently breaks test assertions:
//   1. module/claude_version_core/src/version.rs — this table (the canonical source)
//   2. spec.md                  — FR table (§ Version Aliases)
//   3. tests/integration/mutation_commands_test.rs — TC-309 + TC-410 assertions
//   4. docs/cli/types.md        — alias resolution table
//   5. docs/cli/workflows.md    — monthly baseline workflow examples (7 refs)
//   6. docs/cli/testing/command/version_guard.md — TC-410 spec (3 refs)

/// A named version alias that resolves to a specific semver or the literal `"latest"`.
#[ derive( Debug ) ]
pub struct VersionAlias
{
  /// Short alias name used on the CLI (e.g. `"stable"`, `"month"`, `"latest"`).
  pub name        : &'static str,
  /// Resolved semver string, or empty string for the `latest` alias.
  pub value       : &'static str,
  /// Human-readable description shown in `.version.list` output.
  pub description : &'static str,
}

/// All known version aliases in display order.
pub const VERSION_ALIASES : &[ VersionAlias ] = &[
  VersionAlias { name : "latest", value : "",       description : "Most recent published release" },
  VersionAlias { name : "stable", value : "2.1.78", description : "Pinned stable release (recommended)" },
  VersionAlias { name : "month",  value : "2.1.74", description : "~1 month old release for stability" },
];

// ── Version detection ─────────────────────────────────────────────────────────

/// Extract the semver token (digits and dots) from a raw version string.
///
/// Strips an optional leading `v` or `V` prefix. Returns `raw` unchanged if
/// no semver-shaped token is found.
#[ inline ]
#[ must_use ]
pub fn extract_semver( raw : &str ) -> &str
{
  raw.split_whitespace()
  .find_map( | t |
  {
    let candidate = t.strip_prefix( 'v' )
    .or_else( || t.strip_prefix( 'V' ) )
    .unwrap_or( t );
    if !candidate.is_empty() && candidate.chars().all( | c | c.is_ascii_digit() || c == '.' )
    {
      Some( candidate )
    }
    else
    {
      None
    }
  } )
  .unwrap_or( raw )
}

/// Read the installed version from the `~/.local/bin/claude` symlink target.
///
/// Returns `None` if `HOME` is not set or the symlink does not exist.
#[ inline ]
#[ must_use ]
pub fn get_version_from_symlink() -> Option< String >
{
  let home = std::env::var( "HOME" ).ok().filter( | h | !h.is_empty() )?;
  let link = format!( "{home}/.local/bin/claude" );
  let target = std::fs::read_link( &link ).ok()?;
  let name = target.file_name()?.to_str()?;
  if !name.is_empty() && name.chars().all( | c | c.is_ascii_digit() || c == '.' )
  {
    Some( name.to_string() )
  }
  else
  {
    None
  }
}

/// Run `claude --version` and return its trimmed stdout.
///
/// Returns `None` if `claude` is not in PATH or the command fails.
#[ inline ]
#[ must_use ]
pub fn get_claude_version_raw() -> Option< String >
{
  let output = std::process::Command::new( "bash" )
  .args( [ "-c", "claude --version" ] )
  .env( "DISABLE_AUTOUPDATER", "1" )
  .output()
  .ok()?;
  let s = String::from_utf8_lossy( &output.stdout ).trim().to_string();
  if s.is_empty() { None } else { Some( s ) }
}

/// Get the installed Claude Code version (symlink-based detection preferred).
///
/// Returns `None` if no installed version can be detected.
#[ inline ]
#[ must_use ]
pub fn get_installed_version() -> Option< String >
{
  get_version_from_symlink()
  .or_else( ||
  {
    get_claude_version_raw().map( | raw | extract_semver( &raw ).to_string() )
  } )
}

// ── Alias resolution ──────────────────────────────────────────────────────────

/// Resolve a version spec to the value passed to the official installer.
///
/// Aliases map to their pinned semver or `"latest"`. Unknown specs are returned
/// unchanged (e.g. a raw `"1.2.3"` passes through as-is).
#[ inline ]
#[ must_use ]
pub fn resolve_version_spec( spec : &str ) -> &str
{
  VERSION_ALIASES.iter()
  .find( | a | a.name == spec )
  .map_or( spec, | a | if a.value.is_empty() { a.name } else { a.value } )
}

/// Validate a version spec: must be a known alias or a 3-part semver.
///
/// # Errors
///
/// Returns [`CoreError::ParseError`] for empty or unrecognised specs.
#[ inline ]
pub fn validate_version_spec( spec : &str ) -> Result< (), CoreError >
{
  if spec.is_empty()
  {
    return Err( CoreError::ParseError( "version:: value cannot be empty".to_string() ) );
  }

  if VERSION_ALIASES.iter().any( | a | a.name == spec )
  {
    return Ok( () );
  }

  // Semver: exactly 3 dot-separated numeric parts, no leading zeros.
  let parts : Vec< &str > = spec.split( '.' ).collect();
  if parts.len() == 3
  && parts.iter().all( | p |
  {
    !p.is_empty()
    && p.chars().all( | c | c.is_ascii_digit() )
    && ( p.len() == 1 || !p.starts_with( '0' ) )
  } )
  {
    return Ok( () );
  }

  Err( CoreError::ParseError( format!(
    "unknown version '{spec}': expected 'stable', 'latest', 'month', or semver like '1.2.3'"
  ) ) )
}

// ── Installation helpers ──────────────────────────────────────────────────────

/// Remove the existing `claude` binary so a new install replaces it cleanly.
#[ inline ]
pub fn hot_swap_binary()
{
  let claude_path = std::process::Command::new( "which" )
  .arg( "claude" )
  .output()
  .ok()
  .filter( | o | o.status.success() )
  .map_or_else(
    ||
    {
      let home = std::env::var( "HOME" ).unwrap_or_default();
      format!( "{home}/.local/bin/claude" )
    },
    | o | String::from_utf8_lossy( &o.stdout ).trim().to_string(),
  );

  if std::path::Path::new( &claude_path ).exists()
  {
    let _ = std::fs::remove_file( &claude_path );
  }
}

/// Return the path to the versions directory where Claude Code binaries live.
#[ inline ]
#[ must_use ]
pub fn versions_dir_path() -> String
{
  let home = std::env::var( "HOME" ).unwrap_or_default();
  format!( "{home}/.local/share/claude/versions" )
}

/// Purge all cached binaries from `versions_dir` except `keep`.
///
/// Best-effort: silently ignores all errors (consistent with `lock_version()`
/// and `unlock_versions_dir()`). Only deletes entries whose names consist
/// entirely of ASCII digits and dots — the version-string pattern (e.g. `2.1.78`).
/// This guard prevents accidental deletion of future lock/metadata files that
/// Claude's updater might add to the same directory.
///
/// Called from `perform_install()` before `lock_version()` for pinned installs.
/// The `versions_dir` parameter is explicit (not read from `HOME`) to allow
/// test isolation without `std::env::set_var`, which is not thread-safe.
#[ inline ]
pub fn purge_stale_versions( versions_dir : &str, keep : &str )
{
  let Ok( entries ) = std::fs::read_dir( versions_dir ) else { return; };
  for entry in entries.flatten()
  {
    let name      = entry.file_name();
    let name_str  = name.to_string_lossy();
    if name_str == keep { continue; }
    if !name_str.chars().all( | c | c.is_ascii_digit() || c == '.' ) { continue; }
    let _ = std::fs::remove_file( entry.path() );
  }
}

/// Unlock the versions directory so the installer can write new binaries.
#[ inline ]
pub fn unlock_versions_dir()
{
  let dir = versions_dir_path();
  if std::path::Path::new( &dir ).exists()
  {
    let _ = std::process::Command::new( "chmod" )
    .args( [ "755", &dir ] )
    .status();
  }
}

/// Apply version lock (pinned) or unlock (latest) after a successful install.
///
/// Sets or removes `env.DISABLE_AUTOUPDATER` and updates `autoUpdates` in
/// `~/.claude/settings.json`. For pinned versions, also `chmod 555` the
/// versions directory to prevent silent auto-updates.
#[ inline ]
pub fn lock_version( is_latest : bool )
{
  if let Some( paths ) = ClaudePaths::new()
  {
    let settings_file = paths.settings_file();
    if let Some( parent ) = settings_file.parent()
    {
      let _ = std::fs::create_dir_all( parent );
    }

    let auto_val = if is_latest { "true" } else { "false" };
    let _ = set_setting( &settings_file, "autoUpdates", auto_val );

    if is_latest
    {
      let _ = remove_env_var( &settings_file, "DISABLE_AUTOUPDATER" );
    }
    else
    {
      let _ = set_env_var( &settings_file, "DISABLE_AUTOUPDATER", "1" );
    }
  }

  let dir = versions_dir_path();
  if std::path::Path::new( &dir ).exists()
  {
    let mode = if is_latest { "755" } else { "555" };
    let _ = std::process::Command::new( "chmod" )
    .args( [ mode, &dir ] )
    .status();
  }
}

/// Execute the install sequence: hot-swap → unlock → curl → purge → lock.
///
/// For pinned versions (`!is_latest`), `purge_stale_versions` runs after the
/// curl install and BEFORE `lock_version` (which applies chmod 555). Purging
/// after chmod 555 would silently fail. Purge is skipped for `latest` so the
/// cached version history remains available for rollback.
///
/// `resolved` is the semver string or `"latest"`. `is_latest` controls
/// whether auto-updates are enabled and the versions dir is left unlocked.
///
/// # Errors
///
/// Returns [`CoreError::ProcessError`] if the installer script fails.
#[ inline ]
pub fn perform_install( resolved : &str, is_latest : bool ) -> Result< (), CoreError >
{
  if !find_claude_processes().is_empty()
  {
    hot_swap_binary();
  }

  unlock_versions_dir();

  let shell_cmd = if is_latest
  {
    format!( "curl -fsSL {INSTALL_URL} | bash" )
  }
  else
  {
    format!( "curl -fsSL {INSTALL_URL} | bash -s -- {resolved}" )
  };

  let status = std::process::Command::new( "bash" )
  .args( [ "-c", &shell_cmd ] )
  .env( "DISABLE_AUTOUPDATER", "1" )
  .status()
  .map_err( | e | CoreError::ProcessError( format!( "failed to run installer: {e}" ) ) )?;

  if !status.success()
  {
    return Err( CoreError::ProcessError( "install failed".to_string() ) );
  }

  if !is_latest
  {
    purge_stale_versions( &versions_dir_path(), resolved );
  }
  lock_version( is_latest );
  Ok( () )
}

// ── Preference persistence ─────────────────────────────────────────────────────

/// Read the user's preferred version from `~/.claude/settings.json`.
///
/// Returns `None` if `HOME` is unset, the settings file is absent, or no
/// preference has been stored yet.
#[ inline ]
#[ must_use ]
pub fn read_preferred_version() -> Option< ( String, Option< String > ) >
{
  let paths = ClaudePaths::new()?;
  let settings_file = paths.settings_file();
  let spec = get_setting( &settings_file, "preferredVersionSpec" )
    .ok()?
    .filter( | s | !s.is_empty() )?;
  let resolved = get_setting( &settings_file, "preferredVersionResolved" )
    .ok()
    .flatten()
    .filter( | v | v != "null" && !v.is_empty() );
  Some( ( spec, resolved ) )
}

/// Persist the user's preferred version in `~/.claude/settings.json`.
///
/// Both `preferredVersionSpec` and `preferredVersionResolved` are written.
/// For the `latest` alias, `resolved` is stored as `"null"`.
///
/// # Errors
///
/// Returns [`CoreError`] if `HOME` is unset or the settings file cannot be written.
#[ inline ]
pub fn store_preferred_version( spec : &str, resolved : &str, is_latest : bool ) -> Result< (), CoreError >
{
  let paths = ClaudePaths::new().ok_or_else( ||
    CoreError::ProcessError( "HOME environment variable not set".to_string() )
  )?;
  let settings_file = paths.settings_file();
  if let Some( parent ) = settings_file.parent()
  {
    let _ = std::fs::create_dir_all( parent );
  }
  set_setting( &settings_file, "preferredVersionSpec", spec )
    .map_err( CoreError::IoError )?;
  let resolved_val = if is_latest { "null" } else { resolved };
  set_setting( &settings_file, "preferredVersionResolved", resolved_val )
    .map_err( CoreError::IoError )?;
  Ok( () )
}
