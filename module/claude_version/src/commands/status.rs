//! `.status` — installation state, process count, active account, and lock-state visibility.

use unilang::data::{ ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;

use crate::output::{ OutputFormat, OutputOptions, json_escape };
use claude_runner_core::process::find_claude_processes;
use claude_version_core::config_catalog::catalog;
use claude_version_core::config_resolve::resolve;
use claude_version_core::settings_io::get_setting;
use claude_version_core::version::{
  extract_semver, get_claude_version_raw, get_installed_version, read_preferred_version,
  read_versions_dir_lock_mode,
};

/// Read the active account name from the credential store `_active` marker.
///
/// Checks `$HOME/.persistent/claude/credential/_active`.
fn get_active_account() -> Option< String >
{
  let root = std::env::var_os( "HOME" ).filter( | v | !v.is_empty() )?;
  let marker = std::path::Path::new( &root )
    .join( ".persistent" ).join( "claude" ).join( "credential" ).join( "_active" );
  std::fs::read_to_string( marker )
  .ok()
  .map( | s | s.trim().to_string() )
  .filter( | s | !s.is_empty() )
}

/// One row of the `Lock:` compliance report — a single lock-mechanism key's
/// actual value compared against what the current pin state implies.
struct LockRow
{
  key       : &'static str,
  actual    : Option< String >,
  expected  : Option< String >,
  compliant : bool,
}

/// Compute the 6 lock-mechanism rows (5 settings keys + `chmod`) for the
/// current pin state.
///
/// Degrades to `None`/absent actual values if `HOME` cannot be resolved —
/// `.status` never fails (FR-01).
fn compute_lock_rows( is_pinned : bool, resolved_version : Option< &str > ) -> Vec< LockRow >
{
  let settings_file = super::require_claude_paths().ok().map( | p | p.settings_file() );
  let resolve_ctx    = super::config_resolve_context().ok();

  let auto_updates_actual = settings_file.as_deref()
    .and_then( | f | get_setting( f, "autoUpdates" ).ok().flatten() );
  let auto_updates_channel_actual = settings_file.as_deref()
    .and_then( | f | get_setting( f, "autoUpdatesChannel" ).ok().flatten() );
  let minimum_version_actual = settings_file.as_deref()
    .and_then( | f | get_setting( f, "minimumVersion" ).ok().flatten() );
  let disable_autoupdater_actual = resolve_ctx.as_ref()
    .and_then( | ( home, cwd ) | resolve( "env.DISABLE_AUTOUPDATER", home, cwd, catalog() ).value );
  let disable_updates_actual = resolve_ctx.as_ref()
    .and_then( | ( home, cwd ) | resolve( "env.DISABLE_UPDATES", home, cwd, catalog() ).value );
  let chmod_actual = Some( read_versions_dir_lock_mode().to_string() );

  // `autoUpdates` is the one key `lock_version()` always sets explicitly (never
  // removes it) — but a never-pinned install that predates any `.version.install`
  // call also has no `autoUpdates` key at all. `absent` and `"true"` both mean
  // "not locked" when unpinned, so both are accepted as compliant in that case.
  let auto_updates_expected  = Some( if is_pinned { "false" } else { "true" }.to_string() );
  let auto_updates_compliant = if is_pinned
  {
    auto_updates_actual.as_deref() == Some( "false" )
  }
  else
  {
    auto_updates_actual.is_none() || auto_updates_actual.as_deref() == Some( "true" )
  };

  let auto_updates_channel_expected = if is_pinned { Some( "stable".to_string() ) } else { None };
  let minimum_version_expected      = if is_pinned { resolved_version.map( str::to_string ) } else { None };
  let disable_autoupdater_expected  = if is_pinned { Some( "1".to_string() ) } else { None };
  let disable_updates_expected      = if is_pinned { Some( "1".to_string() ) } else { None };
  let chmod_expected                = Some( if is_pinned { "555" } else { "755" }.to_string() );

  let auto_updates_channel_compliant = auto_updates_channel_actual == auto_updates_channel_expected;
  let minimum_version_compliant      = minimum_version_actual == minimum_version_expected;
  let disable_autoupdater_compliant  = disable_autoupdater_actual == disable_autoupdater_expected;
  let disable_updates_compliant      = disable_updates_actual == disable_updates_expected;
  let chmod_compliant                = chmod_actual == chmod_expected;

  vec!
  [
    LockRow { key : "autoUpdates", actual : auto_updates_actual,
      expected : auto_updates_expected, compliant : auto_updates_compliant },
    LockRow { key : "autoUpdatesChannel", actual : auto_updates_channel_actual,
      expected : auto_updates_channel_expected, compliant : auto_updates_channel_compliant },
    LockRow { key : "minimumVersion", actual : minimum_version_actual,
      expected : minimum_version_expected, compliant : minimum_version_compliant },
    LockRow { key : "env.DISABLE_AUTOUPDATER", actual : disable_autoupdater_actual,
      expected : disable_autoupdater_expected, compliant : disable_autoupdater_compliant },
    LockRow { key : "env.DISABLE_UPDATES", actual : disable_updates_actual,
      expected : disable_updates_expected, compliant : disable_updates_compliant },
    LockRow { key : "chmod", actual : chmod_actual, expected : chmod_expected, compliant : chmod_compliant },
  ]
}

/// Render the `Lock:` section for text-mode output at `v >= 2`.
fn render_lock_text( rows : &[ LockRow ] ) -> String
{
  let mut out = String::from( "Lock:\n" );
  for row in rows
  {
    let actual   = row.actual.as_deref().unwrap_or( "absent" );
    let expected = row.expected.as_deref().unwrap_or( "absent" );
    let marker   = if row.compliant { "" } else { " MISMATCH" };
    out.push_str( &format!( "  {}: {actual} (expected: {expected}){marker}\n", row.key ) );
  }
  out
}

/// Render the `"lock"` JSON object.
fn render_lock_json( rows : &[ LockRow ] ) -> String
{
  let entries : Vec< String > = rows.iter().map( | row |
  {
    let actual = row.actual.as_deref()
      .map_or( "null".to_string(), | v | format!( "\"{}\"", json_escape( v ) ) );
    let expected = row.expected.as_deref()
      .map_or( "null".to_string(), | v | format!( "\"{}\"", json_escape( v ) ) );
    format!(
      "\"{}\":{{\"actual\":{actual},\"expected\":{expected},\"compliant\":{}}}",
      json_escape( row.key ), row.compliant
    )
  } ).collect();
  format!( "{{{}}}", entries.join( "," ) )
}

/// `.status` — show installation state, process count, active account, and
/// (at `v::2`+ or in JSON) lock-state compliance for the current pin.
///
/// # Errors
///
/// Returns `Err` only when `format::` has an unrecognised value.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn status_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts     = OutputOptions::from_cmd( &cmd )?;
  let version  = get_installed_version()
    .or_else( || get_claude_version_raw().map( | r | extract_semver( &r ).to_string() ) )
    .unwrap_or_else( || "not found".to_string() );
  let processes = find_claude_processes().len();
  let account  = get_active_account().unwrap_or_else( || "unknown".to_string() );
  let pref     = read_preferred_version();
  let is_pinned = pref.as_ref().is_some_and( | ( _, resolved ) | resolved.is_some() );
  let resolved_version = pref.as_ref().and_then( | ( _, resolved ) | resolved.as_deref() );

  let content = match ( opts.format, opts.verbosity )
  {
    ( OutputFormat::Json, _ ) =>
    {
      let v = json_escape( &version );
      let a = json_escape( &account );
      let lock_json = render_lock_json( &compute_lock_rows( is_pinned, resolved_version ) );
      match &pref
      {
        Some( ( spec, resolved ) ) =>
        {
          let ps = json_escape( spec );
          let pr = resolved.as_deref().map_or( "null".to_string(), | r | format!( "\"{}\"", json_escape( r ) ) );
          format!( "{{\"version\":\"{v}\",\"processes\":{processes},\"account\":\"{a}\",\"preferred\":{{\"spec\":\"{ps}\",\"resolved\":{pr}}},\"lock\":{lock_json}}}\n" )
        }
        None => format!( "{{\"version\":\"{v}\",\"processes\":{processes},\"account\":\"{a}\",\"lock\":{lock_json}}}\n" ),
      }
    }
    ( OutputFormat::Text, 0 ) =>
    {
      match &pref
      {
        Some( ( spec, _ ) ) => format!( "{version}\n{processes}\n{account}\n{spec}\n" ),
        None => format!( "{version}\n{processes}\n{account}\n" ),
      }
    }
    ( OutputFormat::Text, v ) =>
    {
      // "Processes:" is 10 chars; pad shorter labels to align values at column 12.
      let base = format!( "Version:   {version}\nProcesses: {processes}\nAccount:   {account}" );
      match &pref
      {
        Some( ( spec, resolved ) ) =>
        {
          let pref_str = match resolved
          {
            Some( r ) => format!( "{spec} (v{r})" ),
            None => spec.clone(),
          };
          if v >= 2
          {
            let lock_text = render_lock_text( &compute_lock_rows( is_pinned, resolved_version ) );
            format!( "{base}\nPreferred: {pref_str}  (settings.json \u{2192} preferredVersionSpec)\n{lock_text}" )
          }
          else
          {
            format!( "{base}\nPreferred: {pref_str}\n" )
          }
        }
        None =>
        {
          if v >= 2
          {
            let lock_text = render_lock_text( &compute_lock_rows( is_pinned, resolved_version ) );
            format!( "{base}\n{lock_text}" )
          }
          else
          {
            format!( "{base}\n" )
          }
        }
      }
    }
  };

  Ok( OutputData::new( content, "text" ) )
}
