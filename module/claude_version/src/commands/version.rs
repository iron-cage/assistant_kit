//! `.version.*` — version show, install, guard, and list.

use core::fmt::Write as _;

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;

use crate::output::{ OutputFormat, OutputOptions, json_escape };
use claude_version_core::settings_io::{ get_setting, set_setting };
use claude_version_core::version::{
  VERSION_ALIASES,
  get_installed_version,
  perform_install, read_preferred_version,
  resolve_version_spec, store_preferred_version,
  validate_version_spec,
};

/// `.version.show` — print the currently installed Claude Code version.
///
/// # Errors
///
/// Returns `Err(InternalError)` if `claude` is not found in PATH.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn version_show_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts    = OutputOptions::from_cmd( &cmd )?;
  let version = get_installed_version().ok_or_else( || ErrorData::new(
    ErrorCode::InternalError,
    "claude binary not found in PATH".to_string(),
  ) )?;
  let pref = read_preferred_version();

  let content = match ( opts.format, opts.verbosity )
  {
    ( OutputFormat::Json, _ ) =>
    {
      let v = json_escape( &version );
      format!( "{{\"version\":\"{v}\"}}\n" )
    }
    ( OutputFormat::Text, 0 ) => format!( "{version}\n" ),
    ( OutputFormat::Text, _ ) =>
    {
      let mut out = format!( "Version: {version}\n" );
      if let Some( ( spec, resolved ) ) = &pref
      {
        let pref_str = match resolved
        {
          Some( r ) => format!( "{spec} (v{r})" ),
          None => spec.clone(),
        };
        let match_status = match resolved
        {
          Some( r ) if r == &version => "match",
          Some( _ ) => "MISMATCH",
          None => "latest",
        };
        let _ = writeln!( out, "Preferred: {pref_str} -- {match_status}" );
      }
      out
    }
  };

  Ok( OutputData::new( content, "text" ) )
}

/// `.version.install` — download and install a Claude Code version.
///
/// # Errors
///
/// Returns `Err(ArgumentTypeMismatch)` when the version spec or format is invalid.
/// Returns `Err(InternalError)` when `curl` is not found or the install fails.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn version_install_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts = OutputOptions::from_cmd( &cmd )?;
  let version_spec = match cmd.arguments.get( "version" )
  {
    Some( Value::String( s ) ) => s.clone(),
    _                          => "stable".to_string(),
  };
  validate_version_spec( &version_spec )
    .map_err( | e | ErrorData::new( ErrorCode::ArgumentTypeMismatch, e.to_string() ) )?;

  let resolved   = resolve_version_spec( &version_spec );
  let is_latest  = resolved == "latest";
  let is_alias   = version_spec != resolved;
  let label      = if is_alias { format!( "{version_spec} (v{resolved})" ) }
                   else if is_latest { "latest".to_string() }
                   else { format!( "v{resolved}" ) };
  let auto_label = if is_latest { "true" } else { "false" };

  if super::is_dry( &cmd )
  {
    let content = install_dry_content( &opts, &label, auto_label, is_latest, &version_spec, resolved );
    return Ok( OutputData::new( content, "text" ) );
  }

  // Idempotency guard: skip install if already at target version.
  // Fix(BUG-004): store preference even on idempotent skip
  // Root cause: early return bypassed store_preferred_version()
  // Pitfall: every exit path that confirms a version must persist the preference
  if !super::is_force( &cmd ) && !is_latest
  {
    if let Some( current ) = get_installed_version()
    {
      if current == resolved
      {
        let _ = store_preferred_version( &version_spec, resolved, is_latest );
        let content = match opts.format
        {
          OutputFormat::Json =>
          {
            let l = json_escape( &label );
            format!( "{{\"installed\":false,\"label\":\"{l}\"}}\n" )
          }
          OutputFormat::Text =>
          {
            // v::0 = bare label only; v::1+ = labeled confirmation.
            if opts.verbosity == 0
            {
              format!( "{label}\n" )
            }
            else
            {
              format!( "already at {label}\n" )
            }
          }
        };
        return Ok( OutputData::new( content, "text" ) );
      }
    }
  }

  perform_install( resolved, is_latest )
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, e.to_string() ) )?;
  store_preferred_version( &version_spec, resolved, is_latest )
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, e.to_string() ) )?;

  let pref_label = if is_latest { version_spec.clone() } else { format!( "{version_spec} (v{resolved})" ) };
  let content = match opts.format
  {
    OutputFormat::Json =>
    {
      let l = json_escape( &label );
      let p = json_escape( &pref_label );
      format!( "{{\"installed\":true,\"label\":\"{l}\",\"auto_updates\":{auto_label},\"preferred\":\"{p}\"}}\n" )
    }
    OutputFormat::Text =>
    {
      // v::0 = bare label only; v::1+ = full labeled output.
      if opts.verbosity == 0
      {
        format!( "{label}\n" )
      }
      else
      {
        format!( "installed {label}\nautoUpdates = {auto_label}\npreferred = {pref_label}\n" )
      }
    }
  };
  Ok( OutputData::new( content, "text" ) )
}

/// Build dry-run output for `version_install_routine`.
fn install_dry_content(
  opts         : &OutputOptions,
  label        : &str,
  auto_label   : &str,
  is_latest    : bool,
  version_spec : &str,
  resolved     : &str,
) -> String
{
  match opts.format
  {
    OutputFormat::Json =>
    {
      let l = json_escape( label );
      format!( "{{\"dry_run\":true,\"version\":\"{l}\",\"auto_updates\":{auto_label}}}\n" )
    }
    OutputFormat::Text =>
    {
      if is_latest
      {
        format!(
          "[dry-run] would install {label}\n\
           [dry-run] would set autoUpdates = {auto_label}\n\
           [dry-run] would remove env.DISABLE_AUTOUPDATER\n\
           [dry-run] would leave versions dir unlocked\n\
           [dry-run] would store preferred version = {version_spec}\n"
        )
      }
      else
      {
        format!(
          "[dry-run] would install {label}\n\
           [dry-run] would set autoUpdates = {auto_label}\n\
           [dry-run] would set env.DISABLE_AUTOUPDATER = 1\n\
           [dry-run] would chmod 555 versions dir (hard lock)\n\
           [dry-run] would purge stale cached binaries (keep v{resolved})\n\
           [dry-run] would store preferred version = {version_spec} (v{resolved})\n"
        )
      }
    }
  }
}

/// `.version.guard` — check for version drift and restore preferred version.
///
/// When no preference is stored, defaults to `stable`. Optional `version::SPEC`
/// overrides the stored preference for this single invocation without writing to
/// `settings.json` — see FR-21.
///
/// # Errors
///
/// Returns `Err(ArgumentMissing)` when `version::` is present but empty.
/// Returns `Err(InternalError)` when HOME is unset or the install fails.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn version_guard_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts  = OutputOptions::from_cmd( &cmd )?;
  let dry   = super::is_dry( &cmd );
  let force = super::is_force( &cmd );
  let version_override = match cmd.arguments.get( "version" )
  {
    Some( Value::String( s ) ) =>
    {
      // Reuse the same validator as .version.install so both commands accept/reject
      // identical specs (aliases, semver format, empty value).
      validate_version_spec( s )
        .map_err( | e | ErrorData::new( ErrorCode::ArgumentTypeMismatch, e.to_string() ) )?;
      Some( s.clone() )
    }
    _ => None,
  };
  let interval_secs = match cmd.arguments.get( "interval" )
  {
    Some( Value::Integer( n ) ) => u64::try_from( *n ).unwrap_or( 0 ),
    _                           => 0,
  };

  if interval_secs == 0
  {
    return guard_once( dry, force, version_override.as_deref(), opts.verbosity, opts.format );
  }

  // Watch mode: loop until interrupted (Ctrl+C).
  // Exception: dry mode runs one iteration and exits — preview mode must not run a daemon.
  let mut iterations : u64 = 0;
  loop
  {
    iterations += 1;
    let now    = current_timestamp();
    let result = guard_once( dry, force, version_override.as_deref(), opts.verbosity, opts.format );

    match result
    {
      Ok( out ) =>
      {
        let status = out.content.trim_end();
        eprintln!( "[{now}] #{iterations} {status}" );
        if dry { return Ok( out ); }
      }
      Err( e ) =>
      {
        // Fix(BUG-005): watch loop terminated on any install error in watch mode.
        // Root cause: prior code had `return result` here, which exited the daemon
        //   on the first failure; ETXTBSY ("Text file busy") from a running claude
        //   binary silently killed the guard after one drift-restore attempt.
        // Pitfall: one-shot mode (interval==0) returns before this loop and still
        //   propagates errors normally — do NOT add a continue/return here.
        eprintln!( "[{now}] #{iterations} error: {e}" );
        if dry { return Err( e ); }
      }
    }
    std::thread::sleep( core::time::Duration::from_secs( interval_secs ) );
  }
}

/// Single iteration of the version guard check.
/// Defaults to `stable` when no preference is stored.
/// When `version_override` is `Some`, it replaces the stored preference for this invocation
/// without writing to `settings.json`.
fn guard_once( dry : bool, force : bool, version_override : Option< &str >, verbosity : u8, format : OutputFormat ) -> Result< OutputData, ErrorData >
{
  // If HOME is unset or empty, installation would target "/.claude" (root)
  // which requires root permission.  Degrade gracefully rather than crashing.
  let home_valid = std::env::var( "HOME" ).is_ok_and( | h | !h.is_empty() );
  if !home_valid
  {
    let msg = match format
    {
      OutputFormat::Json => "{\"status\":\"no-home\"}\n".to_string(),
      OutputFormat::Text =>
      {
        if verbosity == 0
        {
          "no-home\n".to_string()
        }
        else
        {
          "no HOME directory; defaulting to stable (nothing to guard)\n".to_string()
        }
      }
    };
    return Ok( OutputData::new( msg, "text" ) );
  }

  let ( spec, resolved ) = if let Some( ver ) = version_override
  {
    // Override: resolve alias immediately; do NOT read or write settings.json.
    let resolved_ver = resolve_version_spec( ver );
    let resolved_opt = if resolved_ver == ver { None } else { Some( resolved_ver.to_string() ) };
    ( ver.to_string(), resolved_opt.or_else( || Some( ver.to_string() ) ) )
  }
  else
  {
    read_preferred_version()
      .unwrap_or_else( || ( "stable".to_string(), Some( resolve_version_spec( "stable" ).to_string() ) ) )
  };

  if spec == "latest" || resolved.is_none()
  {
    return guard_once_latest( dry, verbosity, format );
  }
  guard_once_pinned( dry, force, &spec, resolved.as_deref().unwrap_or( &spec ), verbosity, format )
}

/// Guard path for `latest` preference: verify auto-update config, fix if wrong.
///
/// # Errors
///
/// Returns `Err(InternalError)` when the `autoUpdates` setting must be written
/// but the write fails (e.g. read-only filesystem, permissions error).
fn guard_once_latest( dry : bool, verbosity : u8, format : OutputFormat ) -> Result< OutputData, ErrorData >
{
  if dry
  {
    let msg = match format
    {
      OutputFormat::Json => "{\"status\":\"dry\",\"spec\":\"latest\"}\n".to_string(),
      OutputFormat::Text =>
      {
        if verbosity == 0 { "latest\n" } else { "preferred = latest (no version pin to guard)\n" }.to_string()
      }
    };
    return Ok( OutputData::new( msg, "text" ) );
  }
  if let Some( paths ) = claude_core::ClaudePaths::new()
  {
    let settings_file = paths.settings_file();
    let auto_val = get_setting( &settings_file, "autoUpdates" )
      .ok()
      .flatten()
      .unwrap_or_default();
    if auto_val != "true"
    {
      set_setting( &settings_file, "autoUpdates", "true" )
        .map_err( | e | ErrorData::new(
          ErrorCode::InternalError,
          format!( "failed to set autoUpdates: {e}" ),
        ) )?;
      let msg = match format
      {
        OutputFormat::Json => "{\"status\":\"fixed\",\"action\":\"autoUpdates_enabled\"}\n".to_string(),
        OutputFormat::Text =>
        {
          if verbosity == 0 { "fixed\n" } else { "fixed autoUpdates = true for latest preference\n" }.to_string()
        }
      };
      return Ok( OutputData::new( msg, "text" ) );
    }
  }
  let msg = match format
  {
    OutputFormat::Json => "{\"status\":\"ok\",\"spec\":\"latest\"}\n".to_string(),
    OutputFormat::Text =>
    {
      if verbosity == 0 { "latest\n" } else { "preferred = latest (auto-update enabled)\n" }.to_string()
    }
  };
  Ok( OutputData::new( msg, "text" ) )
}

/// Check installed version and handle drift for the guard command.
///
/// Returns `Ok(Some(output))` when the installed version yields an early response,
/// `Ok(None)` if no version is installed (caller proceeds to fresh install),
/// or `Err` if a reinstall was attempted and failed.
fn check_installed_guard(
  target     : &str,
  pref_label : &str,
  dry        : bool,
  verbosity  : u8,
  format     : OutputFormat,
) -> Result< Option< OutputData >, ErrorData >
{
  let Some( current ) = get_installed_version() else { return Ok( None ); };
  if current == target
  {
    let pl  = json_escape( pref_label );
    let msg = match format
    {
      OutputFormat::Json =>
      {
        format!( "{{\"status\":\"ok\",\"installed\":\"{current}\",\"preferred\":\"{pl}\"}}\n" )
      }
      OutputFormat::Text =>
      {
        if verbosity == 0
        {
          "ok\n".to_string()
        }
        else
        {
          format!( "version {current} matches preferred {pref_label}\n" )
        }
      }
    };
    return Ok( Some( OutputData::new( msg, "text" ) ) );
  }
  if dry
  {
    let pl  = json_escape( pref_label );
    let msg = match format
    {
      OutputFormat::Json =>
      {
        format!( "{{\"status\":\"dry\",\"drift\":true,\"installed\":\"{current}\",\"preferred\":\"{pl}\"}}\n" )
      }
      OutputFormat::Text =>
      {
        if verbosity == 0
        {
          format!( "[dry-run] {current}\u{2192}{target}\n" )
        }
        else
        {
          format!( "[dry-run] drift detected: installed {current}, preferred {pref_label}\n\
                    [dry-run] would reinstall {pref_label}\n" )
        }
      }
    };
    return Ok( Some( OutputData::new( msg, "text" ) ) );
  }
  eprintln!( "drift detected: installed {current}, preferred {pref_label} \u{2014} restoring" );
  perform_install( target, false )
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, e.to_string() ) )?;
  let pl  = json_escape( pref_label );
  let msg = match format
  {
    OutputFormat::Json =>
    {
      format!( "{{\"status\":\"restored\",\"preferred\":\"{pl}\"}}\n" )
    }
    OutputFormat::Text =>
    {
      if verbosity == 0 { format!( "restored {target}\n" ) } else { format!( "restored {pref_label}\n" ) }
    }
  };
  Ok( Some( OutputData::new( msg, "text" ) ) )
}

/// Guard path for pinned versions: compare installed vs preferred and restore on drift.
///
/// `resolved` is the stored `preferredVersionResolved` value from settings.
/// For alias specs (e.g. "stable", "month") it is advisory only — this function
/// re-resolves `spec` through [`resolve_version_spec()`] at call time and uses the
/// fresh `resolved_now` as the install target. `resolved` is authoritative only
/// when `spec` is a concrete semver string (where `resolve_version_spec` returns
/// `spec` unchanged).
fn guard_once_pinned( dry : bool, force : bool, spec : &str, resolved : &str, verbosity : u8, format : OutputFormat ) -> Result< OutputData, ErrorData >
{
  // Re-resolve alias through current table so stale settings don't trigger false drift.
  let resolved_now = resolve_version_spec( spec );
  let target = if resolved_now == spec { resolved } else { resolved_now };
  let pref_label = if spec == target { format!( "v{target}" ) } else { format!( "{spec} (v{target})" ) };

  if !force
  {
    if let Some( output ) = check_installed_guard( target, &pref_label, dry, verbosity, format )?
    {
      return Ok( output );
    }
  }
  if dry
  {
    let pl = json_escape( &pref_label );
    let msg = match format
    {
      OutputFormat::Json =>
      {
        format!( "{{\"status\":\"dry\",\"drift\":false,\"preferred\":\"{pl}\"}}\n" )
      }
      OutputFormat::Text =>
      {
        if verbosity == 0
        {
          format!( "[dry-run] {target}\n" )
        }
        else
        {
          format!( "[dry-run] would install preferred {pref_label}\n" )
        }
      }
    };
    return Ok( OutputData::new( msg, "text" ) );
  }
  perform_install( target, false )
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, e.to_string() ) )?;
  let pl = json_escape( &pref_label );
  let msg = match format
  {
    OutputFormat::Json =>
    {
      format!( "{{\"status\":\"installed\",\"preferred\":\"{pl}\"}}\n" )
    }
    OutputFormat::Text =>
    {
      if verbosity == 0 { format!( "installed {target}\n" ) } else { format!( "installed preferred {pref_label}\n" ) }
    }
  };
  Ok( OutputData::new( msg, "text" ) )
}

/// Convert a count of days since the Unix epoch into a (year, month, day) tuple (UTC).
///
/// Uses Gregorian calendar arithmetic with 400-year cycle constants.
/// No leap-second adjustment: this is for human-readable log timestamps only.
fn days_to_ymd( mut days : u64 ) -> ( u64, u8, u8 )
{
  let y400 = days / 146_097;    days %= 146_097;
  let y100 = ( days / 36_524 ).min( 3 );  days -= y100 * 36_524;
  let y4   = days / 1_461;                days %= 1_461;
  let y1   = ( days / 365 ).min( 3 );     days -= y1 * 365;
  let year = 1970 + y400 * 400 + y100 * 100 + y4 * 4 + y1;
  let leap  = ( year % 4 == 0 && year % 100 != 0 ) || year % 400 == 0;
  let mdays : &[ u64 ] = if leap
  {
    &[ 31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 ]
  }
  else
  {
    &[ 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 ]
  };
  let mut month = 1u8;
  for &md in mdays
  {
    if days < md { break; }
    days  -= md;
    month += 1;
  }
  ( year, month, u8::try_from( days ).expect( "day of month always 0-30" ) + 1 )
}

/// UTC timestamp in ISO 8601 format: `YYYY-MM-DDTHH:MM:SSZ` (no chrono crate).
fn current_timestamp() -> String
{
  let secs = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .unwrap_or_default()
    .as_secs();
  let s = secs % 60;
  let m = ( secs / 60 ) % 60;
  let h = ( secs / 3600 ) % 24;
  let ( year, month, day ) = days_to_ymd( secs / 86_400 );
  format!( "{year:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z" )
}

/// `.version.list` — list all named version aliases.
///
/// # Errors
///
/// Returns `Err` if `format::` has an unrecognised value.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn version_list_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts = OutputOptions::from_cmd( &cmd )?;

  let content = match ( opts.format, opts.verbosity )
  {
    ( OutputFormat::Json, _ ) =>
    {
      let entries : Vec< String > = VERSION_ALIASES.iter().map( | a |
      {
        if a.value.is_empty()
        {
          format!( "  {{\"name\":\"{}\",\"description\":\"{}\"}}", a.name, a.description )
        }
        else
        {
          format!( "  {{\"name\":\"{}\",\"value\":\"{}\",\"description\":\"{}\"}}", a.name, a.value, a.description )
        }
      } ).collect();
      format!( "[\n{}\n]\n", entries.join( ",\n" ) )
    }
    ( OutputFormat::Text, 0 ) =>
    {
      let names : Vec< &str > = VERSION_ALIASES.iter().map( | a | a.name ).collect();
      format!( "{}\n", names.join( "\n" ) )
    }
    ( OutputFormat::Text, _ ) =>
    {
      let lines : Vec< String > = VERSION_ALIASES.iter()
      .map( | a |
      {
        if a.value.is_empty()
        {
          format!( "{} \u{2014} {}", a.name, a.description )
        }
        else
        {
          format!( "{} \u{2014} {} (v{})", a.name, a.description, a.value )
        }
      } )
      .collect();
      format!( "{}\n", lines.join( "\n" ) )
    }
  };

  Ok( OutputData::new( content, "text" ) )
}
