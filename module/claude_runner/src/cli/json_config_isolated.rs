//! JSON config application for the `isolated` and `refresh` subcommands.
//!
//! Split out of `json_config.rs` (which was over the line-count guideline) — mirrors that
//! file's `load_json_source` → `parse_json_object` → `apply_json_config_*` combinator shape,
//! scoped to `IsolatedArgs`/`RefreshArgs` instead of `CliArgs`.

use error_tools::Result;
use serde_json::{ Map, Value };
use claude_runner_core::EffortLevel;
use super::json_config::{ load_json_source, parse_json_object };

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
      "model" =>
      {
        if parsed.model.is_none()
        {
          if let Value::String( s ) = v { parsed.model = Some( s.clone() ); }
        }
      }
      "effort" =>
      {
        if parsed.effort.is_none()
        {
          if let Value::String( s ) = v
          {
            if let Ok( level ) = s.parse::< EffortLevel >() { parsed.effort = Some( level ); }
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
      "max-turns" =>
      {
        if parsed.max_turns.is_none()
        {
          if let Value::String( s ) = v { parsed.max_turns = Some( s.clone() ); }
        }
      }
      "no-chrome" =>
      {
        if !parsed.no_chrome
        {
          if let Value::Bool( b ) = v { if *b { parsed.no_chrome = true; } }
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
