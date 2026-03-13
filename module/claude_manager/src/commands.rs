//! Command handlers: one function per `claude_manager` subcommand.
//!
//! Each handler receives a `VerifiedCommand` and `ExecutionContext` and returns
//! `Result<OutputData, ErrorData>`. Handlers are registered in `build_registry()`
//! in `main.rs`.
//!
//! # Architectural Boundary
//!
//! Account management commands (`.account.*`) are **not** implemented here.
//! They live in `claude_profile` (Layer 2 peer crate). `claude_manager` has
//! zero dependency on `claude_profile_core` — account CLI code moved there
//! in plan 005 to fix a layering violation where `claude_profile_core`
//! (Layer 1 pure domain) had pulled in CLI dependencies.
//!
//! # Note on `needless_pass_by_value`
//!
//! Every handler takes `VerifiedCommand` by value because the `CommandRoutine`
//! type alias requires ownership.

use core::fmt::Write as _;
use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;

use crate::output::{ OutputFormat, OutputOptions, json_escape };
use claude_runner_core::process::{ find_claude_processes, send_sigkill, send_sigterm };
use claude_manager_core::settings_io::{ StoredAs, get_setting, infer_type, read_all_settings, set_setting };
use claude_manager_core::version::{
  VERSION_ALIASES,
  extract_semver, get_claude_version_raw, get_installed_version,
  perform_install, read_preferred_version,
  resolve_version_spec, store_preferred_version,
  validate_version_spec,
};

// ── Private helpers ───────────────────────────────────────────────────────────

/// Require a non-empty string argument from the command's argument map.
fn require_nonempty_string_arg( cmd : &VerifiedCommand, name : &str ) -> Result< String, ErrorData >
{
  let val = match cmd.arguments.get( name )
  {
    Some( Value::String( s ) ) => s.clone(),
    _ => return Err( ErrorData::new( ErrorCode::ArgumentMissing, format!( "{name}:: is required" ) ) ),
  };
  if val.is_empty()
  {
    return Err( ErrorData::new( ErrorCode::ArgumentMissing, format!( "{name}:: value cannot be empty" ) ) );
  }
  Ok( val )
}


/// Return `true` when the command has `dry::1`.
#[ inline ]
fn is_dry( cmd : &VerifiedCommand ) -> bool
{
  matches!( cmd.arguments.get( "dry" ), Some( Value::Boolean( true ) ) )
}

/// Return `true` when the command has `force::1`.
#[ inline ]
fn is_force( cmd : &VerifiedCommand ) -> bool
{
  matches!( cmd.arguments.get( "force" ), Some( Value::Boolean( true ) ) )
}

/// Validate HOME is non-empty and return a `ClaudePaths`.
fn require_claude_paths() -> Result< claude_common::ClaudePaths, ErrorData >
{
  match std::env::var( "HOME" )
  {
    Ok( home ) if !home.is_empty() =>
    {
      claude_common::ClaudePaths::new().ok_or_else( || ErrorData::new(
        ErrorCode::InternalError,
        "HOME environment variable not set".to_string(),
      ) )
    }
    _ => Err( ErrorData::new( ErrorCode::InternalError, "HOME environment variable not set".to_string() ) ),
  }
}


/// Read the active account name from `~/.claude/accounts/_active`.
///
/// Still needed by `status_routine`; not removed with the account routines in Task 038.
fn get_active_account() -> Option< String >
{
  let paths = claude_common::ClaudePaths::new()?;
  let marker = paths.accounts_dir().join( "_active" );
  std::fs::read_to_string( marker )
  .ok()
  .map( | s | s.trim().to_string() )
  .filter( | s | !s.is_empty() )
}

// ── Constants ─────────────────────────────────────────────────────────────────

const RELEASES_API_URL  : &str = "https://api.github.com/repos/anthropics/claude-code/releases?per_page=100";
const CACHE_TTL_SECS    : u64  = 3600;

// ── Command handlers ──────────────────────────────────────────────────────────

/// `.status` — show installation state, process count, and active account.
///
/// # Errors
///
/// Returns `Err` only when `format::` has an unrecognised value.
#[ allow( clippy::needless_pass_by_value, clippy::missing_inline_in_public_items ) ]
pub fn status_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts     = OutputOptions::from_cmd( &cmd )?;
  let version  = get_installed_version()
    .or_else( || get_claude_version_raw().map( | r | extract_semver( &r ).to_string() ) )
    .unwrap_or_else( || "not found".to_string() );
  let processes = find_claude_processes().len();
  let account  = get_active_account().unwrap_or_else( || "unknown".to_string() );
  let pref     = read_preferred_version();

  let content = match ( opts.format, opts.verbosity )
  {
    ( OutputFormat::Json, _ ) =>
    {
      let v = json_escape( &version );
      let a = json_escape( &account );
      match &pref
      {
        Some( ( spec, resolved ) ) =>
        {
          let ps = json_escape( spec );
          let pr = resolved.as_deref().map_or( "null".to_string(), | r | format!( "\"{}\"", json_escape( r ) ) );
          format!( "{{\"version\":\"{v}\",\"processes\":{processes},\"account\":\"{a}\",\"preferred\":{{\"spec\":\"{ps}\",\"resolved\":{pr}}}}}\n" )
        }
        None => format!( "{{\"version\":\"{v}\",\"processes\":{processes},\"account\":\"{a}\"}}\n" ),
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
      let base = format!( "Version:  {version}\nProcesses: {processes}\nAccount:  {account}" );
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
            format!( "{base}\nPreferred: {pref_str}  (settings.json \u{2192} preferredVersionSpec)\n" )
          }
          else
          {
            format!( "{base}\nPreferred: {pref_str}\n" )
          }
        }
        None => format!( "{base}\n" ),
      }
    }
  };

  Ok( OutputData::new( content, "text" ) )
}

/// `.version.show` — print the currently installed Claude Code version.
///
/// # Errors
///
/// Returns `Err(InternalError)` if `claude` is not found in PATH.
#[ allow( clippy::needless_pass_by_value, clippy::missing_inline_in_public_items ) ]
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
/// Returns `Err(ArgumentTypeMismatch)` when the version spec is invalid.
/// Returns `Err(InternalError)` when `curl` is not found or the install fails.
#[ allow( clippy::needless_pass_by_value, clippy::missing_inline_in_public_items ) ]
pub fn version_install_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
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

  // Dry-run: preview what would happen, no side effects.
  if is_dry( &cmd )
  {
    let out = if is_latest
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
    };
    return Ok( OutputData::new( out, "text" ) );
  }

  // Idempotency guard: skip install if already at target version.
  // Fix(issue-358): store preference even on idempotent skip
  // Root cause: early return bypassed store_preferred_version()
  // Pitfall: every exit path that confirms a version must persist the preference
  if !is_force( &cmd ) && !is_latest
  {
    if let Some( current ) = get_installed_version()
    {
      if current == resolved
      {
        let _ = store_preferred_version( &version_spec, resolved, is_latest );
        return Ok( OutputData::new( format!( "already at {label}\n" ), "text" ) );
      }
    }
  }

  perform_install( resolved, is_latest )
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, e.to_string() ) )?;
  store_preferred_version( &version_spec, resolved, is_latest )
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, e.to_string() ) )?;

  let pref_label = if is_latest { version_spec.clone() } else { format!( "{version_spec} (v{resolved})" ) };
  Ok( OutputData::new(
    format!( "installed {label}\nautoUpdates = {auto_label}\npreferred = {pref_label}\n" ),
    "text",
  ) )
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
#[ allow( clippy::needless_pass_by_value, clippy::missing_inline_in_public_items ) ]
pub fn version_guard_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let dry   = is_dry( &cmd );
  let force = is_force( &cmd );
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
    return guard_once( dry, force, version_override.as_deref() );
  }

  // Watch mode: loop until interrupted (Ctrl+C).
  let mut iterations : u64 = 0;
  loop
  {
    iterations += 1;
    let now    = chrono_timestamp();
    let result = guard_once( dry, force, version_override.as_deref() );

    match &result
    {
      Ok( out ) =>
      {
        let status = out.content.trim_end();
        eprintln!( "[{now}] #{iterations} {status}" );
      }
      Err( e ) =>
      {
        // Fix(issue-415): watch loop terminated on any install error in watch mode.
        // Root cause: prior code had `return result` here, which exited the daemon
        //   on the first failure; ETXTBSY ("Text file busy") from a running claude
        //   binary silently killed the guard after one drift-restore attempt.
        // Pitfall: one-shot mode (interval==0) returns before this loop and still
        //   propagates errors normally — do NOT add a continue/return here.
        eprintln!( "[{now}] #{iterations} error: {e}" );
      }
    }
    std::thread::sleep( core::time::Duration::from_secs( interval_secs ) );
  }
}

/// Single iteration of the version guard check.
/// Defaults to `stable` when no preference is stored.
/// When `version_override` is `Some`, it replaces the stored preference for this invocation
/// without writing to `settings.json`.
fn guard_once( dry : bool, force : bool, version_override : Option< &str > ) -> Result< OutputData, ErrorData >
{
  // If HOME is unset or empty, installation would target "/.claude" (root)
  // which requires root permission.  Degrade gracefully rather than crashing.
  let home_valid = std::env::var( "HOME" ).map( | h | !h.is_empty() ).unwrap_or( false );
  if !home_valid
  {
    return Ok( OutputData::new(
      "no HOME directory; defaulting to stable (nothing to guard)\n".to_string(),
      "text",
    ) );
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
    return Ok( guard_once_latest( dry ) );
  }
  guard_once_pinned( dry, force, &spec, resolved.as_deref().unwrap_or( &spec ) )
}

/// Guard path for `latest` preference: verify auto-update config, fix if wrong.
///
/// Returns `OutputData` (not `Result`) because every code path succeeds: settings
/// read and write errors are silently tolerated so the guard degrades gracefully
/// rather than failing when HOME exists but settings are unreadable.
fn guard_once_latest( dry : bool ) -> OutputData
{
  if dry
  {
    return OutputData::new( "preferred = latest (no version pin to guard)\n".to_string(), "text" );
  }
  if let Some( paths ) = claude_common::ClaudePaths::new()
  {
    let settings_file = paths.settings_file();
    let auto_val = get_setting( &settings_file, "autoUpdates" )
      .ok()
      .flatten()
      .unwrap_or_default();
    if auto_val != "true"
    {
      let _ = set_setting( &settings_file, "autoUpdates", "true" );
      return OutputData::new( "fixed autoUpdates = true for latest preference\n".to_string(), "text" );
    }
  }
  OutputData::new( "preferred = latest (auto-update enabled)\n".to_string(), "text" )
}

/// Guard path for pinned versions: compare installed vs preferred and restore on drift.
fn guard_once_pinned( dry : bool, force : bool, spec : &str, resolved : &str ) -> Result< OutputData, ErrorData >
{
  // Re-resolve alias through current table so stale settings don't trigger false drift.
  let resolved_now = resolve_version_spec( spec );
  let target = if resolved_now == spec { resolved } else { resolved_now };
  let pref_label = if spec == target { format!( "v{target}" ) } else { format!( "{spec} (v{target})" ) };

  if !force
  {
    if let Some( current ) = get_installed_version()
    {
      if current == target
      {
        return Ok( OutputData::new(
          format!( "version {current} matches preferred {pref_label}\n" ),
          "text",
        ) );
      }
      if dry
      {
        return Ok( OutputData::new(
          format!( "[dry-run] drift detected: installed {current}, preferred {pref_label}\n\
                    [dry-run] would reinstall {pref_label}\n" ),
          "text",
        ) );
      }
      eprintln!( "drift detected: installed {current}, preferred {pref_label} \u{2014} restoring" );
      perform_install( target, false )
        .map_err( | e | ErrorData::new( ErrorCode::InternalError, e.to_string() ) )?;
      return Ok( OutputData::new( format!( "restored {pref_label}\n" ), "text" ) );
    }
  }
  if dry
  {
    return Ok( OutputData::new(
      format!( "[dry-run] would install preferred {pref_label}\n" ),
      "text",
    ) );
  }
  perform_install( target, false )
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, e.to_string() ) )?;
  Ok( OutputData::new( format!( "installed preferred {pref_label}\n" ), "text" ) )
}

/// Simple UTC timestamp string (no chrono crate — uses system time).
fn chrono_timestamp() -> String
{
  let dur = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .unwrap_or_default();
  let secs = dur.as_secs();
  let h = ( secs / 3600 ) % 24;
  let m = ( secs / 60 ) % 60;
  let s = secs % 60;
  format!( "{h:02}:{m:02}:{s:02}" )
}

/// `.version.list` — list all named version aliases.
///
/// # Errors
///
/// Returns `Err` if `format::` has an unrecognised value.
#[ allow( clippy::needless_pass_by_value, clippy::missing_inline_in_public_items ) ]
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

/// `.processes` — list all running Claude Code processes.
///
/// # Errors
///
/// Returns `Err` if `format::` has an unrecognised value.
#[ allow( clippy::needless_pass_by_value, clippy::missing_inline_in_public_items ) ]
pub fn processes_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts  = OutputOptions::from_cmd( &cmd )?;
  let procs = find_claude_processes();

  let content = match opts.format
  {
    OutputFormat::Json =>
    {
      if procs.is_empty()
      {
        "{\"processes\":[]}\n".to_string()
      }
      else
      {
        let entries : Vec< String > = procs.iter().map( | p |
        {
          let cwd = json_escape( &p.cwd.to_string_lossy() );
          format!( "  {{\"pid\":{},\"cwd\":\"{cwd}\"}}", p.pid )
        } ).collect();
        format!( "{{\"processes\":[\n{}\n]}}\n", entries.join( ",\n" ) )
      }
    }
    OutputFormat::Text =>
    {
      if procs.is_empty()
      {
        String::new()
      }
      else
      {
        let lines : Vec< String > = procs.iter().map( | p |
          match opts.verbosity
          {
            0 => format!( "{} {}", p.pid, p.cwd.display() ),
            _ => format!( "PID: {}  CWD: {}", p.pid, p.cwd.display() ),
          }
        ).collect();
        format!( "{}\n", lines.join( "\n" ) )
      }
    }
  };

  Ok( OutputData::new( content, "text" ) )
}

/// `.processes.kill` — terminate all Claude Code processes.
///
/// # Errors
///
/// Returns `Err(InternalError)` if signal delivery fails for all processes.
#[ allow( clippy::needless_pass_by_value, clippy::missing_inline_in_public_items ) ]
pub fn processes_kill_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let procs = find_claude_processes();

  if procs.is_empty()
  {
    return Ok( OutputData::new( "no active processes\n".to_string(), "text" ) );
  }

  if is_dry( &cmd )
  {
    let signal = if is_force( &cmd ) { "SIGKILL" } else { "SIGTERM" };
    let lines : Vec< String > = procs.iter()
    .map( | p | format!( "[dry-run] would send {signal} to PID {}", p.pid ) )
    .collect();
    return Ok( OutputData::new( format!( "{}\n", lines.join( "\n" ) ), "text" ) );
  }

  let count = procs.len();

  if is_force( &cmd )
  {
    for p in &procs
    {
      let _ = send_sigkill( p.pid );
    }
  }
  else
  {
    for p in &procs
    {
      let _ = send_sigterm( p.pid );
    }
    std::thread::sleep( core::time::Duration::from_secs( 2 ) );
    let survivors = find_claude_processes();
    for p in &survivors
    {
      let _ = send_sigkill( p.pid );
    }
  }

  std::thread::sleep( core::time::Duration::from_millis( 500 ) );
  let remaining = find_claude_processes().len();
  if remaining > 0
  {
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "killed {count} process(es) but {remaining} could not be terminated" ),
    ) );
  }

  Ok( OutputData::new( format!( "killed {count} process(es)\n" ), "text" ) )
}

/// `.settings.show` — print all entries from `~/.claude/settings.json`.
///
/// # Errors
///
/// Returns `Err(InternalError)` when HOME is missing or settings unreadable.
#[ allow( clippy::needless_pass_by_value, clippy::missing_inline_in_public_items ) ]
pub fn settings_show_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts  = OutputOptions::from_cmd( &cmd )?;
  let paths = require_claude_paths()?;
  let pairs = read_all_settings( &paths.settings_file() ).map_err( | e |
    ErrorData::new( ErrorCode::InternalError, format!( "failed to read settings: {e}" ) )
  )?;

  let content = match opts.format
  {
    OutputFormat::Json =>
    {
      let body : String = pairs.iter()
      .map( | ( k, v ) |
      {
        let json_val = match infer_type( v )
        {
          StoredAs::Bool | StoredAs::Number | StoredAs::Raw => v.clone(),
          StoredAs::Str => format!( "\"{}\"", json_escape( v ) ),
        };
        format!( "  \"{}\":{json_val}", json_escape( k ) )
      } )
      .collect::< Vec< _ > >()
      .join( ",\n" );
      if body.is_empty()
      {
        "{}\n".to_string()
      }
      else
      {
        format!( "{{\n{body}\n}}\n" )
      }
    }
    OutputFormat::Text =>
    {
      let lines : Vec< String > = pairs.iter().map( | ( k, v ) |
        match opts.verbosity
        {
          0 => format!( "{k}={v}" ),
          _ => format!( "{k}: {v}" ),
        }
      ).collect();
      if lines.is_empty() { String::new() } else { format!( "{}\n", lines.join( "\n" ) ) }
    }
  };

  Ok( OutputData::new( content, "text" ) )
}

/// `.settings.get` — read and print the value of a single setting by key.
///
/// # Errors
///
/// Returns `Err(ArgumentMissing)` when `key::` is missing.
/// Returns `Err(InternalError)` when HOME is missing or key not found.
#[ allow( clippy::needless_pass_by_value, clippy::missing_inline_in_public_items ) ]
pub fn settings_get_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let key   = require_nonempty_string_arg( &cmd, "key" )?;
  let opts  = OutputOptions::from_cmd( &cmd )?;
  let paths = require_claude_paths()?;

  let value = get_setting( &paths.settings_file(), &key )
  .map_err( | e |
    ErrorData::new( ErrorCode::InternalError, format!( "failed to read settings: {e}" ) )
  )?
  .ok_or_else( || ErrorData::new(
    ErrorCode::InternalError,
    format!( "key '{key}' not found in settings" ),
  ) )?;

  let content = match ( opts.format, opts.verbosity )
  {
    ( OutputFormat::Json, _ ) =>
    {
      let k = json_escape( &key );
      let v = json_escape( &value );
      format!( "{{\"key\":\"{k}\",\"value\":\"{v}\"}}\n" )
    }
    ( OutputFormat::Text, 0 ) => format!( "{value}\n" ),
    ( OutputFormat::Text, _ ) => format!( "{key}: {value}\n" ),
  };

  Ok( OutputData::new( content, "text" ) )
}

/// `.settings.set` — write a new value for a single setting atomically.
///
/// # Errors
///
/// Returns `Err(ArgumentMissing)` when `key::` or `value::` is missing or empty.
/// Returns `Err(InternalError)` when HOME is missing or write fails.
///
/// Fix(issue-settings-set-empty-value): `value::` (empty) was accepted and stored `""` in JSON.
/// Root cause: used `require_string_arg` (allows empty) instead of `require_nonempty_string_arg`
///   for the `value::` parameter, silently bypassing the FR-04 empty-value rejection.
/// Pitfall: `cm .settings.set key::k value::` appeared to succeed but wrote a meaningless
///   empty-string entry — indistinguishable from "key not set" when read back via `.settings.get`.
#[ allow( clippy::needless_pass_by_value, clippy::missing_inline_in_public_items ) ]
pub fn settings_set_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let key   = require_nonempty_string_arg( &cmd, "key" )?;
  let value = require_nonempty_string_arg( &cmd, "value" )?;

  let stored_as = infer_type( &value );

  if is_dry( &cmd )
  {
    let type_label = match stored_as
    {
      StoredAs::Bool   => "bool",
      StoredAs::Number => "number",
      StoredAs::Str    => "string",
      StoredAs::Raw    => "object",
    };
    return Ok( OutputData::new(
      format!( "[dry-run] would set {key} = {value}  ({type_label})\n" ),
      "text",
    ) );
  }

  let paths = require_claude_paths()?;
  let settings_file = paths.settings_file();

  if let Some( parent ) = settings_file.parent()
  {
    std::fs::create_dir_all( parent ).map_err( | e |
      ErrorData::new( ErrorCode::InternalError, format!( "failed to create settings directory: {e}" ) )
    )?;
  }

  set_setting( &settings_file, &key, &value ).map_err( | e |
    ErrorData::new( ErrorCode::InternalError, format!( "failed to write settings: {e}" ) )
  )?;

  Ok( OutputData::new( format!( "set {key} = {value}\n" ), "text" ) )
}

// ── Release history ───────────────────────────────────────────────────────────

/// A parsed release entry from the GitHub Releases API.
struct ReleaseInfo
{
  version : String,
  date    : String,
  summary : String,
  body    : String,
}

/// Extract the string value for a given JSON key from raw JSON text.
///
/// # Pitfall
///
/// Never iterate `json.as_bytes()` by index and cast each byte to `char`.
/// `bytes[i] as char` interprets each byte as a Unicode scalar of the same
/// value, silently corrupting every multi-byte UTF-8 sequence (any codepoint
/// above U+007F). Use `str::chars()` instead — it respects character
/// boundaries natively.
fn parse_json_string_value( json : &str, key : &str ) -> Option< String >
{
  let colon_pat = format!( "\"{key}\":" );
  let colon_pos = json.find( &colon_pat )? + colon_pat.len();
  let rest  = &json[ colon_pos.. ];
  let quote = rest.find( '"' )?;
  // Byte offset of the character after the opening quote.
  let value_start = colon_pos + quote + 1;
  let content = &json[ value_start.. ];

  let mut out     = String::new();
  let mut chars   = content.chars();
  let mut escaped = false;

  while let Some( ch ) = chars.next()
  {
    if escaped
    {
      match ch
      {
        'n'  => out.push( '\n' ),
        'r'  => out.push( '\r' ),
        't'  => out.push( '\t' ),
        '"'  => out.push( '"'  ),
        '\\' => out.push( '\\' ),
        'u'  =>
        {
          // Consume exactly 4 hex digits that follow \u.
          let hex : String = chars.by_ref().take( 4 ).collect();
          if hex.len() == 4
          {
            if let Ok( cp ) = u32::from_str_radix( &hex, 16 )
            {
              if let Some( c ) = char::from_u32( cp )
              {
                out.push( c );
              }
            }
          }
        }
        other => out.push( other ),
      }
      escaped = false;
    }
    else if ch == '\\'
    {
      escaped = true;
    }
    else if ch == '"'
    {
      return Some( out );
    }
    else
    {
      out.push( ch );
    }
  }

  None
}

/// Parse the full GitHub Releases API JSON response into a `Vec<ReleaseInfo>`.
fn extract_releases( json : &str ) -> Vec< ReleaseInfo >
{
  let marker = "\"tag_name\": \"v";
  let chunks : Vec< &str > = json.split( marker ).collect();
  let mut releases = Vec::new();

  for chunk in chunks.iter().skip( 1 )
  {
    let restored = format!( "{marker}{chunk}" );

    let version = parse_json_string_value( &restored, "tag_name" )
    .map( | v | v.strip_prefix( 'v' ).unwrap_or( &v ).to_string() )
    .unwrap_or_default();

    let date = parse_json_string_value( &restored, "published_at" )
    .map( | d | d.chars().take( 10 ).collect() )
    .unwrap_or_default();

    let body_raw = parse_json_string_value( &restored, "body" )
    .unwrap_or_default();

    let summary = body_raw
    .lines()
    .find( | l | l.starts_with( "- " ) )
    .map_or_else( || "(no changelog)".to_string(), | l | l[ 2.. ].trim().to_string() );

    releases.push( ReleaseInfo { version, date, summary, body : body_raw } );
  }

  releases
}

/// Check whether the cache file's mtime is less than 1 hour old.
fn cache_is_fresh( path : &std::path::Path ) -> bool
{
  std::fs::metadata( path )
  .and_then( | m | m.modified() )
  .ok()
  .and_then( | mtime | std::time::SystemTime::now().duration_since( mtime ).ok() )
  .is_some_and( | elapsed | elapsed.as_secs() < CACHE_TTL_SECS )
}

/// Fetch releases JSON, using a 1-hour file cache in `~/.claude/.transient/`.
fn fetch_releases_json( base : &std::path::Path ) -> Result< String, ErrorData >
{
  let cache_dir  = base.join( ".transient" );
  let cache_path = cache_dir.join( "version_history_cache.json" );

  if cache_is_fresh( &cache_path )
  {
    if let Ok( cached ) = std::fs::read_to_string( &cache_path )
    {
      if !cached.is_empty()
      {
        return Ok( cached );
      }
    }
  }

  let output = std::process::Command::new( "bash" )
  .args( [ "-c", &format!( "curl -fsSL '{RELEASES_API_URL}'" ) ] )
  .output()
  .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "failed to fetch release history: {e}" ) ) )?;

  if !output.status.success()
  {
    return Err( ErrorData::new( ErrorCode::InternalError, "failed to fetch release history".to_string() ) );
  }

  let response = String::from_utf8_lossy( &output.stdout ).to_string();
  if response.trim().is_empty()
  {
    return Err( ErrorData::new( ErrorCode::InternalError, "empty response from GitHub API".to_string() ) );
  }

  let _ = std::fs::create_dir_all( &cache_dir );
  let _ = std::fs::write( &cache_path, &response );

  Ok( response )
}

/// `.version.history` — show release history with changelogs from GitHub.
///
/// # Errors
///
/// Returns `Err(InternalError)` when HOME is missing or the network request fails.
/// Returns `Err(ArgumentTypeMismatch)` when `format::` has an invalid value.
#[ allow( clippy::needless_pass_by_value, clippy::missing_inline_in_public_items ) ]
pub fn version_history_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts  = OutputOptions::from_cmd( &cmd )?;
  let count = match cmd.arguments.get( "count" )
  {
    Some( Value::Integer( n ) ) => usize::try_from( *n ).unwrap_or( 10 ),
    _                           => 10,
  };

  let paths = require_claude_paths()?;
  let json  = fetch_releases_json( paths.base() )?;
  let mut releases = extract_releases( &json );
  releases.truncate( count );

  let content = match ( opts.format, opts.verbosity )
  {
    ( OutputFormat::Json, _ ) =>
    {
      if releases.is_empty()
      {
        "[]\n".to_string()
      }
      else
      {
        let entries : Vec< String > = releases.iter().map( | r |
        {
          let v = json_escape( &r.version );
          let d = json_escape( &r.date );
          let s = json_escape( &r.summary );
          format!( "  {{\"version\":\"{v}\",\"date\":\"{d}\",\"summary\":\"{s}\"}}" )
        } ).collect();
        format!( "[\n{}\n]\n", entries.join( ",\n" ) )
      }
    }
    ( OutputFormat::Text, 0 ) =>
    {
      if releases.is_empty()
      {
        String::new()
      }
      else
      {
        let lines : Vec< String > = releases.iter()
        .map( | r | format!( "{}  {}", r.version, r.date ) )
        .collect();
        format!( "{}\n", lines.join( "\n" ) )
      }
    }
    ( OutputFormat::Text, 1 ) =>
    {
      if releases.is_empty()
      {
        String::new()
      }
      else
      {
        let lines : Vec< String > = releases.iter()
        .map( | r | format!( "{}  {}  {}", r.version, r.date, r.summary ) )
        .collect();
        format!( "{}\n", lines.join( "\n" ) )
      }
    }
    ( OutputFormat::Text, _ ) =>
    {
      if releases.is_empty()
      {
        String::new()
      }
      else
      {
        let blocks : Vec< String > = releases.iter()
        .map( | r |
        {
          let header = format!( "## {} ({})", r.version, r.date );
          if r.body.is_empty()
          {
            header
          }
          else
          {
            format!( "{header}\n\n{}", r.body )
          }
        } )
        .collect();
        format!( "{}\n", blocks.join( "\n\n" ) )
      }
    }
  };

  Ok( OutputData::new( content, "text" ) )
}


