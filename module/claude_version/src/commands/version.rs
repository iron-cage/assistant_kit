//! `.version.*` — version show, install, guard, and list.

use core::fmt::Write as _;

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;

use crate::output::{ OutputFormat, OutputOptions, json_escape };
use claude_core::settings_io::{ get_setting, set_setting };
use claude_version_core::version::{
  VERSION_ALIASES,
  CustomMarker,
  get_installed_version,
  load_custom_markers,
  perform_install, read_preferred_version,
  remove_custom_marker,
  resolve_version_spec, save_custom_marker, store_preferred_version,
  validate_marker_name, validate_version_spec,
};

/// `.version.show` — print the currently installed Claude Code version.
///
/// # Errors
///
/// Returns `Err(InternalError)` if `claude` is not found in PATH.
// Registered as a boxed unilang CommandRoutine (Box< dyn Fn >) — every call goes through
// dynamic dispatch, so #[ inline ] could never apply at the call site.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn version_show_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts    = OutputOptions::from_cmd( &cmd )?;
  let version = get_installed_version().ok_or_else( || ErrorData::new(
    ErrorCode::InternalError,
    "claude binary not found in PATH".to_string(),
  ) )?;

  struct Label
  {
    name        : String,
    kind        : &'static str,
    description : Option< String >,
  }

  // Skip label resolution entirely at v::0 text — per spec.
  let labels : Vec< Label > = if opts.format == OutputFormat::Text && opts.verbosity == 0
  {
    Vec::new()
  }
  else
  {
    let mut buf = Vec::new();
    for marker in load_custom_markers()
    {
      if marker.value == version
      {
        let description = if marker.description.is_empty() { None } else { Some( marker.description ) };
        buf.push( Label { name : marker.name, kind : "custom", description } );
      }
    }
    if let Some( ( spec, Some( resolved ) ) ) = read_preferred_version()
    {
      if resolved == version && VERSION_ALIASES.iter().any( | a | a.name == spec.as_str() )
      {
        buf.push( Label { name : spec, kind : "builtin", description : None } );
      }
    }
    buf
  };

  let content = match ( opts.format, opts.verbosity )
  {
    ( OutputFormat::Json, _ ) =>
    {
      let v = json_escape( &version );
      let labels_json : Vec< String > = labels.iter().map( | l |
      {
        let n = json_escape( &l.name );
        if let Some( ref d ) = l.description
        {
          format!( "{{\"name\":\"{n}\",\"kind\":\"{}\",\"description\":\"{}\"}}", l.kind, json_escape( d ) )
        }
        else
        {
          format!( "{{\"name\":\"{n}\",\"kind\":\"{}\"}}", l.kind )
        }
      } ).collect();
      format!( "{{\"version\":\"{v}\",\"labels\":[{}]}}\n", labels_json.join( "," ) )
    }
    ( OutputFormat::Text, 0 ) => format!( "{version}\n" ),
    ( OutputFormat::Text, 1 ) =>
    {
      if labels.is_empty()
      {
        format!( "Version: {version}\n" )
      }
      else
      {
        let names : Vec< &str > = labels.iter().map( | l | l.name.as_str() ).collect();
        format!( "Version: {version}  [{}]\n", names.join( ", " ) )
      }
    }
    ( OutputFormat::Text, _ ) =>
    {
      let mut out = format!( "version: {version}\n" );
      if !labels.is_empty()
      {
        let parts : Vec< String > = labels.iter().map( | l |
          match &l.description
          {
            Some( d ) => format!( "{} ({}, \"{}\")", l.name, l.kind, d ),
            None      => format!( "{} ({})", l.name, l.kind ),
          }
        ).collect();
        let _ = writeln!( out, "labels:  {}", parts.join( ", " ) );
      }
      out
    }
  };

  Ok( OutputData::new( content, "text" ) )
}

/// `.version.install` — download and install a Claude Code version.
///
/// `record_only::1` persists the resolved preference to `settings.json` without
/// invoking `perform_install()` — lets a caller re-point `.version.show`/`.version.guard`
/// at a new target without downloading/reinstalling `claude`.
///
/// # Errors
///
/// Returns `Err(ArgumentTypeMismatch)` when the version spec or format is invalid.
/// Returns `Err(ArgumentMissing)` when `record_only::1` and `dry::1` are both set.
/// Returns `Err(InternalError)` when `curl` is not found, the install fails, or
/// (under `record_only::1`) the preference write fails.
// Registered as a boxed unilang CommandRoutine (Box< dyn Fn >) — every call goes through
// dynamic dispatch, so #[ inline ] could never apply at the call site.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn version_install_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts = OutputOptions::from_cmd( &cmd )?;

  if super::is_record_only( &cmd ) && super::is_dry( &cmd )
  {
    return Err( ErrorData::new( ErrorCode::ArgumentMissing,
      "record_only:: and dry:: are mutually exclusive".to_string() ) );
  }

  let version_spec = match cmd.arguments.get( "version" )
  {
    Some( Value::String( s ) ) => s.clone(),
    _                          => "stable".to_string(),
  };
  let custom = load_custom_markers();
  validate_version_spec( &version_spec, &custom )
    .map_err( | e | ErrorData::new( ErrorCode::ArgumentTypeMismatch, e.to_string() ) )?;

  let resolved   = resolve_version_spec( &version_spec, &custom );
  let is_latest  = resolved == "latest";
  let is_alias   = version_spec != resolved;
  let label      = if is_alias { format!( "{version_spec} (v{resolved})" ) }
                   else if is_latest { "latest".to_string() }
                   else { format!( "v{resolved}" ) };
  let auto_label = if is_latest { "true" } else { "false" };

  if super::is_dry( &cmd )
  {
    let content = install_dry_content( &opts, &label, auto_label, is_latest, &version_spec, &resolved );
    return Ok( OutputData::new( content, "text" ) );
  }

  // record_only::1 — persist the preference only; never call perform_install().
  // Unlike the idempotency guard below, this branch is unconditional: it fires
  // regardless of whether `resolved` matches the currently-installed version,
  // because the whole point of record_only is "just record it" — not "record it
  // when convenient". force:: has no install to bypass here, so it's a silent
  // no-op under record_only rather than an error (mirrors force:: being inert
  // under dry::).
  if super::is_record_only( &cmd )
  {
    store_preferred_version( &version_spec, &resolved, is_latest )
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, e.to_string() ) )?;
    let pref_label = if is_latest { version_spec.clone() } else { format!( "{version_spec} (v{resolved})" ) };
    let content = install_recorded_content( &opts, &label, &pref_label );
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
        let _ = store_preferred_version( &version_spec, &resolved, is_latest );
        let content = install_skip_content( &opts, &label );
        return Ok( OutputData::new( content, "text" ) );
      }
    }
  }

  // Fix(MAAV-found, Task 314 Round 4 Fresh Challenger): preference must be
  // recorded BEFORE the lock mechanism is applied, not after.
  // Root cause: `perform_install()` (which ends by calling `lock_version()` —
  // setting autoUpdates/chmod/etc.) previously ran before `store_preferred_version()`.
  // A crash/kill in the window between the two left the mechanism genuinely
  // locked but `preferredVersionSpec` unset, so `is_pinned` read `false` and
  // `.status` reported a false MISMATCH on all 6 rows despite the install
  // having actually succeeded.
  // Pitfall: with this order, a crash during `perform_install()` itself now
  // leaves the preference recorded but the mechanism not yet (fully) applied —
  // `.status` will report a MISMATCH in that case too, but it is a TRUE one
  // (the user's recorded intent genuinely isn't enforced yet), not a false
  // positive — this is the correct signal, not a regression.
  store_preferred_version( &version_spec, &resolved, is_latest )
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, e.to_string() ) )?;
  perform_install( &resolved, is_latest )
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, e.to_string() ) )?;

  let pref_label = if is_latest { version_spec.clone() } else { format!( "{version_spec} (v{resolved})" ) };
  let content = install_installed_content( &opts, &label, auto_label, &pref_label );
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
      // Fix(BUG-016): preview the settings-unlock and outcome-verification
      // steps `perform_install()` now performs, per the output-parity
      // requirement in docs/feature/004_dry_run.md.
      // Root cause: the real install gained steps the preview did not mention.
      // Pitfall: the `latest` branch must never contain the word "purge"
      // (TC-360 asserts its absence — Layer 4 is pinned-only).
      if is_latest
      {
        format!(
          "[dry-run] would install {label}\n\
           [dry-run] would lift settings update-locks before install\n\
           [dry-run] would verify install outcome before applying changes\n\
           [dry-run] would set autoUpdates = {auto_label}\n\
           [dry-run] would remove env.DISABLE_AUTOUPDATER\n\
           [dry-run] would remove env.DISABLE_UPDATES\n\
           [dry-run] would remove autoUpdatesChannel\n\
           [dry-run] would remove minimumVersion\n\
           [dry-run] would leave versions dir unlocked\n\
           [dry-run] would store preferred version = {version_spec}\n"
        )
      }
      else
      {
        format!(
          "[dry-run] would install {label}\n\
           [dry-run] would lift settings update-locks before install\n\
           [dry-run] would verify installed version = {resolved} before locking\n\
           [dry-run] would set autoUpdates = {auto_label}\n\
           [dry-run] would set env.DISABLE_AUTOUPDATER = 1\n\
           [dry-run] would set env.DISABLE_UPDATES = 1\n\
           [dry-run] would set autoUpdatesChannel = stable\n\
           [dry-run] would set minimumVersion = {resolved}\n\
           [dry-run] would chmod 555 versions dir (hard lock)\n\
           [dry-run] would purge stale cached binaries (keep v{resolved})\n\
           [dry-run] would store preferred version = {version_spec} (v{resolved})\n"
        )
      }
    }
  }
}

/// Build success output for `version_install_routine`'s `record_only::1` branch.
fn install_recorded_content(
  opts       : &OutputOptions,
  label      : &str,
  pref_label : &str,
) -> String
{
  match opts.format
  {
    OutputFormat::Json =>
    {
      let l = json_escape( label );
      let p = json_escape( pref_label );
      format!( "{{\"installed\":false,\"recorded\":true,\"label\":\"{l}\",\"preferred\":\"{p}\"}}\n" )
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
        format!( "recorded {label}\npreferred = {pref_label}\n" )
      }
    }
  }
}

/// Build output for `version_install_routine`'s idempotency-skip branch (already at target version).
fn install_skip_content(
  opts  : &OutputOptions,
  label : &str,
) -> String
{
  match opts.format
  {
    OutputFormat::Json =>
    {
      let l = json_escape( label );
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
  }
}

/// Build success output for `version_install_routine` after a real install completes.
fn install_installed_content(
  opts       : &OutputOptions,
  label      : &str,
  auto_label : &str,
  pref_label : &str,
) -> String
{
  match opts.format
  {
    OutputFormat::Json =>
    {
      let l = json_escape( label );
      let p = json_escape( pref_label );
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
// Registered as a boxed unilang CommandRoutine (Box< dyn Fn >) — every call goes through
// dynamic dispatch, so #[ inline ] could never apply at the call site.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn version_guard_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts   = OutputOptions::from_cmd( &cmd )?;
  let dry    = super::is_dry( &cmd );
  let force  = super::is_force( &cmd );
  let custom = load_custom_markers();
  let version_override = match cmd.arguments.get( "version" )
  {
    Some( Value::String( s ) ) =>
    {
      // Reuse the same validator as .version.install so both commands accept/reject
      // identical specs (aliases, semver format, empty value).
      validate_version_spec( s, &custom )
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
    return guard_once( dry, force, version_override.as_deref(), opts.verbosity, opts.format, &custom );
  }

  // Watch mode: loop until interrupted (Ctrl+C).
  // Exception: dry mode runs one iteration and exits — preview mode must not run a daemon.
  let next_check = format_interval( interval_secs );
  loop
  {
    let ( date, time ) = current_date_time_parts();
    let result = guard_once( dry, force, version_override.as_deref(), opts.verbosity, opts.format, &custom );

    match result
    {
      Ok( out ) =>
      {
        match opts.format
        {
          // JSON consumers get the check result verbatim, one line per iteration —
          // the compact dot-separated wrapper is a text-format convention only.
          OutputFormat::Json => eprintln!( "{}", out.content.trim_end() ),
          OutputFormat::Text =>
          {
            let status = out.content.trim_end();
            if status == "ok"
            {
              eprintln!( "{date} · {time} · ok · next check in {next_check}" );
            }
            else
            {
              eprintln!( "{date} · {time} · ok · {status} · next check in {next_check}" );
            }
          }
        }
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
        eprintln!( "{date} · {time} · error · {e} · next check in {next_check}" );
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
fn guard_once( dry : bool, force : bool, version_override : Option< &str >, verbosity : u8, format : OutputFormat, custom : &[ CustomMarker ] ) -> Result< OutputData, ErrorData >
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
    let resolved_ver = resolve_version_spec( ver, custom );
    let resolved_opt = if resolved_ver == ver { None } else { Some( resolved_ver ) };
    ( ver.to_string(), resolved_opt.or_else( || Some( ver.to_string() ) ) )
  }
  else
  {
    read_preferred_version()
      .unwrap_or_else( || ( "stable".to_string(), Some( resolve_version_spec( "stable", custom ) ) ) )
  };

  if spec == "latest" || resolved.is_none()
  {
    return guard_once_latest( dry, verbosity, format );
  }
  guard_once_pinned( dry, force, &spec, resolved.as_deref().unwrap_or( &spec ), verbosity, format, custom )
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
/// For alias specs (e.g. "stable", "latest") it is advisory only — this function
/// re-resolves `spec` through [`resolve_version_spec()`] at call time and uses the
/// fresh `resolved_now` as the install target. `resolved` is authoritative only
/// when `spec` is a concrete semver string (where `resolve_version_spec` returns
/// `spec` unchanged).
fn guard_once_pinned( dry : bool, force : bool, spec : &str, resolved : &str, verbosity : u8, format : OutputFormat, custom : &[ CustomMarker ] ) -> Result< OutputData, ErrorData >
{
  // Re-resolve alias through current table so stale settings don't trigger false drift.
  let resolved_now = resolve_version_spec( spec, custom );
  let target = if resolved_now == spec { resolved } else { &resolved_now };
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

/// Current UTC date and time as separate `YYYY-MM-DD` / `HH:MM:SS` parts (no chrono crate).
fn current_date_time_parts() -> ( String, String )
{
  let secs = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .unwrap_or_default()
    .as_secs();
  let s = secs % 60;
  let m = ( secs / 60 ) % 60;
  let h = ( secs / 3600 ) % 24;
  let ( year, month, day ) = days_to_ymd( secs / 86_400 );
  ( format!( "{year:04}-{month:02}-{day:02}" ), format!( "{h:02}:{m:02}:{s:02}" ) )
}

/// Format a duration in seconds as a human-readable interval: whole minutes as `Nm`, else `Ns`.
fn format_interval( secs : u64 ) -> String
{
  if secs >= 60 && secs % 60 == 0
  {
    format!( "{}m", secs / 60 )
  }
  else
  {
    format!( "{secs}s" )
  }
}

/// The 2 list modes `.version.list` can render.
///
/// Not part of the crate's public API — reachable only within `commands::version`,
/// since the declaring `mod version;` in `commands/mod.rs` is private.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
enum ListMode
{
  /// Named version aliases (`stable`, `latest`, plus any custom markers) — default.
  Aliases,
  /// Full release history from the GitHub Releases API (or compiled-in fallback).
  History,
}

impl ListMode
{
  /// Parse a `mode::` value; case-sensitive exact match against the 2 labels.
  fn parse( s : &str ) -> Result< Self, String >
  {
    match s
    {
      "aliases" => Ok( Self::Aliases ),
      "history" => Ok( Self::History ),
      other => Err( format!( "unknown mode '{other}': expected aliases or history" ) ),
    }
  }
}

/// `.version.list` — list version aliases (`mode::aliases`, default) or release
/// history from GitHub (`mode::history`).
///
/// # Errors
///
/// Returns `Err(ArgumentTypeMismatch)` for an unrecognised or empty `mode::` value (exit 1),
/// or if `format::` has an unrecognised value.
/// Returns `Err(InternalError)` under `mode::history` when `HOME` is unset (exit 2).
// Registered as a boxed unilang CommandRoutine (Box< dyn Fn >) — every call goes through
// dynamic dispatch, so #[ inline ] could never apply at the call site.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn version_list_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let mode = match cmd.arguments.get( "mode" )
  {
    Some( Value::String( s ) ) if !s.is_empty() =>
      ListMode::parse( s ).map_err( | e | ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?,
    Some( Value::String( _ ) ) =>
      return Err( ErrorData::new( ErrorCode::ArgumentTypeMismatch, "mode:: value cannot be empty".to_string() ) ),
    _ => ListMode::Aliases,
  };
  let opts = OutputOptions::from_cmd( &cmd )?;

  match mode
  {
    ListMode::Aliases => Ok( render_aliases_mode( &opts ) ),
    ListMode::History =>
    {
      let count = match cmd.arguments.get( "count" )
      {
        Some( Value::Integer( n ) ) => usize::try_from( *n ).unwrap_or( 10 ),
        _                           => 10,
      };
      super::history::render_history_mode( count, &opts )
    }
  }
}

/// Render the `mode::aliases` output — built-in aliases plus custom markers.
fn render_aliases_mode( opts : &OutputOptions ) -> OutputData
{
  let custom  = load_custom_markers();
  let content = match ( opts.format, opts.verbosity )
  {
    ( OutputFormat::Json, _ ) =>
    {
      let mut entries : Vec< String > = VERSION_ALIASES.iter().map( | a |
      {
        if a.value.is_empty()
        {
          format!( "  {{\"name\":\"{}\",\"kind\":\"builtin\",\"description\":\"{}\"}}", a.name, a.description )
        }
        else
        {
          format!(
            "  {{\"name\":\"{}\",\"kind\":\"builtin\",\"value\":\"{}\",\"description\":\"{}\"}}",
            a.name, a.value, a.description
          )
        }
      } ).collect();
      for m in &custom
      {
        entries.push( format!(
          "  {{\"name\":\"{}\",\"kind\":\"custom\",\"value\":\"{}\",\"description\":\"{}\"}}",
          json_escape( &m.name ), json_escape( &m.value ), json_escape( &m.description )
        ) );
      }
      format!( "[\n{}\n]\n", entries.join( ",\n" ) )
    }
    ( OutputFormat::Text, 0 ) =>
    {
      let mut names : Vec< String > = VERSION_ALIASES.iter().map( | a | a.name.to_string() ).collect();
      for m in &custom { names.push( m.name.clone() ); }
      format!( "{}\n", names.join( "\n" ) )
    }
    ( OutputFormat::Text, _ ) =>
    {
      let mut lines : Vec< String > = VERSION_ALIASES.iter()
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
      for m in &custom
      {
        if m.description.is_empty()
        {
          lines.push( format!( "{} \u{2014} {} (custom)", m.name, m.value ) );
        }
        else
        {
          lines.push( format!( "{} \u{2014} {} — {} (custom)", m.name, m.value, m.description ) );
        }
      }
      format!( "{}\n", lines.join( "\n" ) )
    }
  };

  OutputData::new( content, "text" )
}

/// `.version.mark` — create, update, or remove a custom version marker.
///
/// # Errors
///
/// Returns `Err` on invalid name, invalid version spec, or I/O failures.
// Registered as a boxed unilang CommandRoutine (Box< dyn Fn >) — every call goes through
// dynamic dispatch, so #[ inline ] could never apply at the call site.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn version_mark_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  use super::{ is_dry, require_nonempty_string_arg };

  let opts = OutputOptions::from_cmd( &cmd )?;
  let name = require_nonempty_string_arg( &cmd, "name" )?;
  let dry  = is_dry( &cmd );

  // ── unset path ────────────────────────────────────────────────────────────
  if matches!( cmd.arguments.get( "unset" ), Some( Value::Boolean( true ) ) )
  {
    if dry
    {
      let content = match opts.format
      {
        OutputFormat::Json => format!(
          "{{\"action\":\"would-remove\",\"name\":\"{}\"}}\n",
          json_escape( &name ),
        ),
        OutputFormat::Text => format!( "[dry] would remove marker '{name}'\n" ),
      };
      return Ok( OutputData::new( content, "text" ) );
    }
    let removed = remove_custom_marker( &name ).map_err( | e |
      ErrorData::new( ErrorCode::InternalError, e.to_string() )
    )?;
    let content = match ( opts.format, removed )
    {
      ( OutputFormat::Json, true  ) =>
        format!( "{{\"action\":\"removed\",\"name\":\"{}\"}}\n", json_escape( &name ) ),
      ( OutputFormat::Json, false ) =>
        format!( "{{\"action\":\"not-found\",\"name\":\"{}\"}}\n", json_escape( &name ) ),
      ( OutputFormat::Text, true  ) => format!( "marker '{name}' removed\n" ),
      ( OutputFormat::Text, false ) => format!( "marker '{name}' not found\n" ),
    };
    return Ok( OutputData::new( content, "text" ) );
  }

  // ── set path ──────────────────────────────────────────────────────────────
  let version = require_nonempty_string_arg( &cmd, "version" )?;
  let desc    = match cmd.arguments.get( "description" )
  {
    Some( Value::String( s ) ) => s.clone(),
    _                          => String::new(),
  };

  validate_marker_name( &name ).map_err( | e |
    ErrorData::new( ErrorCode::ValidationRuleFailed, e.to_string() )
  )?;

  let custom = load_custom_markers();
  validate_version_spec( &version, &custom ).map_err( | e |
    ErrorData::new( ErrorCode::ValidationRuleFailed, e.to_string() )
  )?;

  if dry
  {
    let content = match opts.format
    {
      OutputFormat::Json => format!(
        "{{\"action\":\"would-set\",\"name\":\"{}\",\"version\":\"{}\"}}\n",
        json_escape( &name ), json_escape( &version ),
      ),
      OutputFormat::Text => format!( "[dry] would set marker '{name}' → '{version}'\n" ),
    };
    return Ok( OutputData::new( content, "text" ) );
  }

  save_custom_marker( &name, &version, &desc ).map_err( | e |
    ErrorData::new( ErrorCode::InternalError, e.to_string() )
  )?;

  let content = match opts.format
  {
    OutputFormat::Json => format!(
      "{{\"action\":\"set\",\"name\":\"{}\",\"version\":\"{}\"}}\n",
      json_escape( &name ), json_escape( &version ),
    ),
    OutputFormat::Text => format!( "marker '{name}' set to '{version}'\n" ),
  };
  Ok( OutputData::new( content, "text" ) )
}
