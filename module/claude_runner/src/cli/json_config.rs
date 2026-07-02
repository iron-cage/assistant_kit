use error_tools::{ Error, Result };
use serde_json::{ Map, Value };
use super::parse::{ CliArgs, ExpectStrategy };
use claude_runner_core::EffortLevel;
use std::fs;

/// Load JSON config file content from disk.
///
/// Returns `Err` with a human-readable message on any I/O failure, including
/// file-not-found. The caller is responsible for exiting with an appropriate
/// error code on `Err`.
pub( super ) fn load_json_source( path : &str ) -> Result< String >
{
  fs::read_to_string( path ).map_err( | e |
    Error::msg( format!( "args-file not found: {path}: {e}" ) )
  )
}

/// Parse a string as a JSON object and return the inner field map.
///
/// Returns `Err` when `src` is not valid JSON or when the JSON root is not
/// an object (array, string, number, bool, or null at the root level).
pub( super ) fn parse_json_object( src : &str ) -> Result< Map< String, Value > >
{
  let v : Value = serde_json::from_str( src ).map_err( | e |
    Error::msg( format!( "args-file: invalid JSON: {e}" ) )
  )?;
  match v
  {
    Value::Object( map ) => Ok( map ),
    other =>
    {
      let type_name = match &other
      {
        Value::Null      => "null",
        Value::Bool( _ ) => "bool",
        Value::Number( _ ) => "number",
        Value::String( _ ) => "string",
        Value::Array( _ )  => "array",
        Value::Object( _ ) => "object",
      };
      Err( Error::msg( format!( "args-file: JSON root must be an object, got {type_name}" ) ) )
    }
  }
}

/// Apply a JSON config map to `parsed`, filling only fields still at their default.
///
/// CLI-set fields (non-default values) are never overwritten — the default-check per
/// field type is: `Option<T>` → `is_none()`, `bool` → `!field`, `Vec<T>` → `is_empty()`.
/// `bool` fields set to JSON `false` are a no-op (no unset semantics).
/// Unknown JSON keys are silently skipped.
#[ allow( clippy::too_many_lines ) ]    // mechanical dispatch — one arm per configurable parameter
#[ allow( clippy::collapsible_match ) ] // each arm is one condition + one pattern check
pub( super ) fn apply_json_config( parsed : &mut CliArgs, map : &Map< String, Value > )
{
  for ( key, v ) in map
  {
    match key.as_str()
    {
      "message" =>
      {
        if parsed.message.is_none()
        {
          if let Value::String( s ) = v { parsed.message = Some( s.clone() ); }
        }
      }
      "print" =>
      {
        if !parsed.print_mode
        {
          if let Value::Bool( b ) = v { if *b { parsed.print_mode = true; } }
        }
      }
      "interactive" =>
      {
        if !parsed.interactive
        {
          if let Value::Bool( b ) = v { if *b { parsed.interactive = true; } }
        }
      }
      "new-session" =>
      {
        if !parsed.new_session
        {
          if let Value::Bool( b ) = v { if *b { parsed.new_session = true; } }
        }
      }
      "model" =>
      {
        if parsed.model.is_none()
        {
          if let Value::String( s ) = v { parsed.model = Some( s.clone() ); }
        }
      }
      "verbose" =>
      {
        if !parsed.verbose
        {
          if let Value::Bool( b ) = v { if *b { parsed.verbose = true; } }
        }
      }
      "no-skip-permissions" =>
      {
        if !parsed.no_skip_permissions
        {
          if let Value::Bool( b ) = v { if *b { parsed.no_skip_permissions = true; } }
        }
      }
      "max-tokens" =>
      {
        if parsed.max_tokens.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u32::try_from( x ).ok() )
            {
              parsed.max_tokens = Some( u );
            }
          }
        }
      }
      "session-dir" =>
      {
        if parsed.session_dir.is_none()
        {
          if let Value::String( s ) = v { parsed.session_dir = Some( s.clone() ); }
        }
      }
      "session-from" =>
      {
        if parsed.session_from.is_none()
        {
          if let Value::String( s ) = v { parsed.session_from = Some( s.clone() ); }
        }
      }
      "dir" =>
      {
        if parsed.dir.is_none()
        {
          if let Value::String( s ) = v { parsed.dir = Some( s.clone() ); }
        }
      }
      "dry-run" =>
      {
        if !parsed.dry_run
        {
          if let Value::Bool( b ) = v { if *b { parsed.dry_run = true; } }
        }
      }
      "trace" =>
      {
        if !parsed.trace
        {
          if let Value::Bool( b ) = v { if *b { parsed.trace = true; } }
        }
      }
      "quiet" =>
      {
        if !parsed.quiet
        {
          if let Value::Bool( b ) = v { if *b { parsed.quiet = true; } }
        }
      }
      "system-prompt" =>
      {
        if parsed.system_prompt.is_none()
        {
          if let Value::String( s ) = v { parsed.system_prompt = Some( s.clone() ); }
        }
      }
      "append-system-prompt" =>
      {
        if parsed.append_system_prompt.is_none()
        {
          if let Value::String( s ) = v { parsed.append_system_prompt = Some( s.clone() ); }
        }
      }
      "no-ultrathink" =>
      {
        if !parsed.no_ultrathink
        {
          if let Value::Bool( b ) = v { if *b { parsed.no_ultrathink = true; } }
        }
      }
      "effort" =>
      {
        if parsed.effort.is_none()
        {
          if let Value::String( s ) = v
          {
            if let Ok( e ) = s.parse::< EffortLevel >() { parsed.effort = Some( e ); }
          }
        }
      }
      "no-effort-max" =>
      {
        if !parsed.no_effort_max
        {
          if let Value::Bool( b ) = v { if *b { parsed.no_effort_max = true; } }
        }
      }
      "no-chrome" =>
      {
        if !parsed.no_chrome
        {
          if let Value::Bool( b ) = v { if *b { parsed.no_chrome = true; } }
        }
      }
      "no-persist" =>
      {
        if !parsed.no_persist
        {
          if let Value::Bool( b ) = v { if *b { parsed.no_persist = true; } }
        }
      }
      "json-schema" =>
      {
        if parsed.json_schema.is_none()
        {
          if let Value::String( s ) = v { parsed.json_schema = Some( s.clone() ); }
        }
      }
      "mcp-config" =>
      {
        if parsed.mcp_config.is_empty()
        {
          match v
          {
            Value::String( s ) => parsed.mcp_config.push( s.clone() ),
            Value::Array( arr ) =>
            {
              for item in arr
              {
                if let Value::String( s ) = item { parsed.mcp_config.push( s.clone() ); }
              }
            }
            _ => {}
          }
        }
      }
      "file" =>
      {
        if parsed.file.is_none()
        {
          if let Value::String( s ) = v { parsed.file = Some( s.clone() ); }
        }
      }
      "strip-fences" =>
      {
        if !parsed.strip_fences
        {
          if let Value::Bool( b ) = v { if *b { parsed.strip_fences = true; } }
        }
      }
      "keep-claudecode" =>
      {
        if !parsed.keep_claudecode
        {
          if let Value::Bool( b ) = v { if *b { parsed.keep_claudecode = true; } }
        }
      }
      "subdir" =>
      {
        if parsed.subdir.is_none()
        {
          if let Value::String( s ) = v
          {
            // Mirror CLI constraint: single name component, no '/' separators.
            if !s.contains( '/' ) { parsed.subdir = Some( s.clone() ); }
          }
        }
      }
      "output-file" =>
      {
        if parsed.output_file.is_none()
        {
          if let Value::String( s ) = v { parsed.output_file = Some( s.clone() ); }
        }
      }
      "expect" =>
      {
        if parsed.expect.is_none()
        {
          if let Value::String( s ) = v { parsed.expect = Some( s.clone() ); }
        }
      }
      "expect-strategy" =>
      {
        if parsed.expect_strategy.is_none()
        {
          if let Value::String( s ) = v
          {
            if let Ok( e ) = s.parse::< ExpectStrategy >() { parsed.expect_strategy = Some( e ); }
          }
        }
      }
      "max-sessions" =>
      {
        if parsed.max_sessions.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u32::try_from( x ).ok() )
            {
              parsed.max_sessions = Some( u );
            }
          }
        }
      }
      "retry-on-transient" =>
      {
        if parsed.retry_on_transient.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u8::try_from( x ).ok() )
            {
              parsed.retry_on_transient = Some( u );
            }
          }
        }
      }
      "transient-delay" =>
      {
        if parsed.transient_delay.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u32::try_from( x ).ok() )
            {
              parsed.transient_delay = Some( u );
            }
          }
        }
      }
      "timeout" =>
      {
        if parsed.timeout.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u32::try_from( x ).ok() )
            {
              parsed.timeout = Some( u );
            }
          }
        }
      }
      "retry-on-account" =>
      {
        if parsed.retry_on_account.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u8::try_from( x ).ok() )
            {
              parsed.retry_on_account = Some( u );
            }
          }
        }
      }
      "account-delay" =>
      {
        if parsed.account_delay.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u32::try_from( x ).ok() )
            {
              parsed.account_delay = Some( u );
            }
          }
        }
      }
      "retry-on-auth" =>
      {
        if parsed.retry_on_auth.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u8::try_from( x ).ok() )
            {
              parsed.retry_on_auth = Some( u );
            }
          }
        }
      }
      "auth-delay" =>
      {
        if parsed.auth_delay.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u32::try_from( x ).ok() )
            {
              parsed.auth_delay = Some( u );
            }
          }
        }
      }
      "retry-on-service" =>
      {
        if parsed.retry_on_service.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u8::try_from( x ).ok() )
            {
              parsed.retry_on_service = Some( u );
            }
          }
        }
      }
      "service-delay" =>
      {
        if parsed.service_delay.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u32::try_from( x ).ok() )
            {
              parsed.service_delay = Some( u );
            }
          }
        }
      }
      "retry-on-process" =>
      {
        if parsed.retry_on_process.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u8::try_from( x ).ok() )
            {
              parsed.retry_on_process = Some( u );
            }
          }
        }
      }
      "process-delay" =>
      {
        if parsed.process_delay.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u32::try_from( x ).ok() )
            {
              parsed.process_delay = Some( u );
            }
          }
        }
      }
      "retry-on-validation" =>
      {
        if parsed.retry_on_validation.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u8::try_from( x ).ok() )
            {
              parsed.retry_on_validation = Some( u );
            }
          }
        }
      }
      "validation-delay" =>
      {
        if parsed.validation_delay.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u32::try_from( x ).ok() )
            {
              parsed.validation_delay = Some( u );
            }
          }
        }
      }
      "retry-on-runner" =>
      {
        if parsed.retry_on_runner.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u8::try_from( x ).ok() )
            {
              parsed.retry_on_runner = Some( u );
            }
          }
        }
      }
      "runner-delay" =>
      {
        if parsed.runner_delay.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u32::try_from( x ).ok() )
            {
              parsed.runner_delay = Some( u );
            }
          }
        }
      }
      "retry-on-unknown" =>
      {
        if parsed.retry_on_unknown.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u8::try_from( x ).ok() )
            {
              parsed.retry_on_unknown = Some( u );
            }
          }
        }
      }
      "unknown-delay" =>
      {
        if parsed.unknown_delay.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u32::try_from( x ).ok() )
            {
              parsed.unknown_delay = Some( u );
            }
          }
        }
      }
      "retry-override" =>
      {
        if parsed.retry_override.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u8::try_from( x ).ok() )
            {
              parsed.retry_override = Some( u );
            }
          }
        }
      }
      "retry-override-delay" =>
      {
        if parsed.retry_override_delay.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u32::try_from( x ).ok() )
            {
              parsed.retry_override_delay = Some( u );
            }
          }
        }
      }
      "retry-default" =>
      {
        if parsed.retry_default.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u8::try_from( x ).ok() )
            {
              parsed.retry_default = Some( u );
            }
          }
        }
      }
      "retry-default-delay" =>
      {
        if parsed.retry_default_delay.is_none()
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64().and_then( | x | u32::try_from( x ).ok() )
            {
              parsed.retry_default_delay = Some( u );
            }
          }
        }
      }
      "output-format" =>
      {
        if parsed.output_format.is_none()
        {
          if let Value::String( s ) = v { parsed.output_format = Some( s.clone() ); }
        }
      }
      "max-turns" =>
      {
        if parsed.max_turns.is_none()
        {
          if let Value::String( s ) = v { parsed.max_turns = Some( s.clone() ); }
        }
      }
      "allowed-tools" =>
      {
        if parsed.allowed_tools.is_none()
        {
          if let Value::String( s ) = v { parsed.allowed_tools = Some( s.clone() ); }
        }
      }
      "disallowed-tools" =>
      {
        if parsed.disallowed_tools.is_none()
        {
          if let Value::String( s ) = v { parsed.disallowed_tools = Some( s.clone() ); }
        }
      }
      "max-budget-usd" =>
      {
        if parsed.max_budget_usd.is_none()
        {
          if let Value::String( s ) = v { parsed.max_budget_usd = Some( s.clone() ); }
        }
      }
      "add-dir" =>
      {
        if parsed.add_dir.is_none()
        {
          if let Value::String( s ) = v { parsed.add_dir = Some( s.clone() ); }
        }
      }
      "fallback-model" =>
      {
        if parsed.fallback_model.is_none()
        {
          if let Value::String( s ) = v { parsed.fallback_model = Some( s.clone() ); }
        }
      }
      "output-style" =>
      {
        if parsed.output_style.is_none()
        {
          if let Value::String( s ) = v
          {
            // Mirror CLI constraint: only "summary" or "raw" are valid values.
            if matches!( s.as_str(), "summary" | "raw" ) { parsed.output_style = Some( s.clone() ); }
          }
        }
      }
      "summary-fields" =>
      {
        if parsed.summary_fields.is_none()
        {
          if let Value::String( s ) = v { parsed.summary_fields = Some( s.clone() ); }
        }
      }
      "journal" =>
      {
        if parsed.journal.is_none()
        {
          if let Value::String( s ) = v
          {
            // Mirror CLI constraint: only "full", "meta", or "off" are valid values.
            if matches!( s.as_str(), "full" | "meta" | "off" ) { parsed.journal = Some( s.clone() ); }
          }
        }
      }
      "journal-dir" =>
      {
        if parsed.journal_dir.is_none()
        {
          if let Value::String( s ) = v { parsed.journal_dir = Some( s.clone() ); }
        }
      }
      // "args-file" is self-referential — silently skip to avoid re-entrant loading.
      // All other unknown keys are silently ignored per AC-009.
      _ => {}
    }
  }
}

/// Load a JSON config file and apply it to `parsed`.
///
/// Convenience combinator: `load_json_source` → `parse_json_object` → `apply_json_config`.
/// Returns `Err` on any I/O or parse failure; the caller prints the error and exits.
pub( super ) fn load_and_apply( path : &str, parsed : &mut CliArgs ) -> Result< () >
{
  let src = load_json_source( path )?;
  let map = parse_json_object( &src )?;
  apply_json_config( parsed, &map );
  Ok( () )
}

/// Apply a JSON config map to `parsed` for the `isolated` subcommand.
///
/// Covers the subset of `IsolatedArgs` fields that JSON config can supply.
/// Same default-check semantics as `apply_json_config`: only fills fields still at default.
#[ allow( clippy::too_many_lines ) ]    // mechanical dispatch — grows linearly with IsolatedArgs parameter set (see rulebook).
#[ allow( clippy::collapsible_match ) ] // mechanical dispatch — each arm is one condition + one pattern check
#[ allow( clippy::assigning_clones ) ]  // field = s.clone() is clearer than clone_from in this dispatch context
pub( super ) fn apply_json_config_isolated(
  parsed : &mut super::cred_parse::IsolatedArgs,
  map    : &Map< String, Value >,
)
{
  // Default sentinel for timeout_secs: 30 (set by parse_isolated_args).
  // Accepted limitation: --timeout 30 (explicit) is indistinguishable from the default.
  const ISOLATED_TIMEOUT_SENTINEL : u64 = 30;

  for ( key, v ) in map
  {
    match key.as_str()
    {
      "message" =>
      {
        if parsed.message.is_none()
        {
          if let Value::String( s ) = v { parsed.message = Some( s.clone() ); }
        }
      }
      "trace" =>
      {
        if !parsed.trace
        {
          if let Value::Bool( b ) = v { if *b { parsed.trace = true; } }
        }
      }
      "dry-run" =>
      {
        if !parsed.dry_run
        {
          if let Value::Bool( b ) = v { if *b { parsed.dry_run = true; } }
        }
      }
      "dir" =>
      {
        if parsed.dir.is_none()
        {
          if let Value::String( s ) = v { parsed.dir = Some( s.clone() ); }
        }
      }
      "add-dir" =>
      {
        if parsed.add_dirs.is_empty()
        {
          match v
          {
            Value::String( s ) => parsed.add_dirs.push( s.clone() ),
            Value::Array( arr ) =>
            {
              for item in arr
              {
                if let Value::String( s ) = item { parsed.add_dirs.push( s.clone() ); }
              }
            }
            _ => {}
          }
        }
      }
      "file" =>
      {
        if parsed.file.is_none()
        {
          if let Value::String( s ) = v { parsed.file = Some( s.clone() ); }
        }
      }
      "expect" =>
      {
        if parsed.expect.is_none()
        {
          if let Value::String( s ) = v { parsed.expect = Some( s.clone() ); }
        }
      }
      "expect-strategy" =>
      {
        if parsed.expect_strategy.is_none()
        {
          // IsolatedArgs stores expect_strategy as String (no enum conversion here).
          if let Value::String( s ) = v { parsed.expect_strategy = Some( s.clone() ); }
        }
      }
      "journal" =>
      {
        if parsed.journal.is_none()
        {
          if let Value::String( s ) = v
          {
            if matches!( s.as_str(), "full" | "meta" | "off" ) { parsed.journal = Some( s.clone() ); }
          }
        }
      }
      "journal-dir" =>
      {
        if parsed.journal_dir.is_none()
        {
          if let Value::String( s ) = v { parsed.journal_dir = Some( s.clone() ); }
        }
      }
      "output-file" =>
      {
        if parsed.output_file.is_none()
        {
          if let Value::String( s ) = v { parsed.output_file = Some( s.clone() ); }
        }
      }
      "strip-fences" =>
      {
        if !parsed.strip_fences
        {
          if let Value::Bool( b ) = v { if *b { parsed.strip_fences = true; } }
        }
      }
      "output-style" =>
      {
        if parsed.output_style.is_none()
        {
          if let Value::String( s ) = v
          {
            if matches!( s.as_str(), "summary" | "raw" ) { parsed.output_style = Some( s.clone() ); }
          }
        }
      }
      "summary-fields" =>
      {
        if parsed.summary_fields.is_none()
        {
          if let Value::String( s ) = v { parsed.summary_fields = Some( s.clone() ); }
        }
      }
      "timeout" =>
      {
        if parsed.timeout_secs == ISOLATED_TIMEOUT_SENTINEL
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64() { parsed.timeout_secs = u; }
          }
        }
      }
      "creds" =>
      {
        if parsed.creds_path.is_empty()
        {
          if let Value::String( s ) = v { parsed.creds_path = s.clone(); }
        }
      }
      // "args-file" is self-referential — silently skip.
      // Unknown keys are silently ignored.
      _ => {}
    }
  }
}

/// Apply a JSON config map to `parsed` for the `refresh` subcommand.
///
/// Covers the small set of `RefreshArgs` fields that JSON config can supply.
#[ allow( clippy::too_many_lines ) ]    // mechanical dispatch — grows linearly with RefreshArgs parameter set (see rulebook).
#[ allow( clippy::collapsible_match ) ] // mechanical dispatch — each arm is one condition + one pattern check
#[ allow( clippy::assigning_clones ) ]  // field = s.clone() is clearer than clone_from in this dispatch context
pub( super ) fn apply_json_config_refresh(
  parsed : &mut super::cred_parse::RefreshArgs,
  map    : &Map< String, Value >,
)
{
  // Default sentinel for timeout_secs: 45 (set by parse_refresh_args).
  // Accepted limitation: --timeout 45 (explicit) is indistinguishable from the default.
  const REFRESH_TIMEOUT_SENTINEL : u64 = 45;

  for ( key, v ) in map
  {
    match key.as_str()
    {
      "trace" =>
      {
        if !parsed.trace
        {
          if let Value::Bool( b ) = v { if *b { parsed.trace = true; } }
        }
      }
      "journal" =>
      {
        if parsed.journal.is_none()
        {
          if let Value::String( s ) = v
          {
            if matches!( s.as_str(), "full" | "meta" | "off" ) { parsed.journal = Some( s.clone() ); }
          }
        }
      }
      "journal-dir" =>
      {
        if parsed.journal_dir.is_none()
        {
          if let Value::String( s ) = v { parsed.journal_dir = Some( s.clone() ); }
        }
      }
      "timeout" =>
      {
        if parsed.timeout_secs == REFRESH_TIMEOUT_SENTINEL
        {
          if let Value::Number( n ) = v
          {
            if let Some( u ) = n.as_u64() { parsed.timeout_secs = u; }
          }
        }
      }
      "creds" =>
      {
        if parsed.creds_path.is_empty()
        {
          if let Value::String( s ) = v { parsed.creds_path = s.clone(); }
        }
      }
      // "args-file" is self-referential — silently skip.
      // Unknown keys are silently ignored.
      _ => {}
    }
  }
}

/// Load a JSON config file and apply it to `parsed` for the `isolated` subcommand.
pub( super ) fn load_and_apply_isolated(
  path   : &str,
  parsed : &mut super::cred_parse::IsolatedArgs,
) -> Result< () >
{
  let src = load_json_source( path )?;
  let map = parse_json_object( &src )?;
  apply_json_config_isolated( parsed, &map );
  Ok( () )
}

/// Load a JSON config file and apply it to `parsed` for the `refresh` subcommand.
pub( super ) fn load_and_apply_refresh(
  path   : &str,
  parsed : &mut super::cred_parse::RefreshArgs,
) -> Result< () >
{
  let src = load_json_source( path )?;
  let map = parse_json_object( &src )?;
  apply_json_config_refresh( parsed, &map );
  Ok( () )
}
