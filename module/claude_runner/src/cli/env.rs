use crate::VerbosityLevel;
use claude_runner_core::EffortLevel;
use error_tools::{ Error, Result };
use super::parse::{ CliArgs, ExpectStrategy, parse_u8_bounded };

/// Returns `true` if `var` is set to `"1"` or `"true"` (case-insensitive).
///
/// Any other value — including `"yes"`, `"0"`, `"false"`, empty, or absent — returns `false`.
pub( super ) fn env_bool( var : &str ) -> bool
{
  std::env::var( var ).ok()
    .is_some_and( | v | matches!( v.to_lowercase().as_str(), "1" | "true" ) )
}

/// Returns `Some(value)` if `var` is set to a non-empty string; `None` otherwise.
pub( super ) fn env_str( var : &str ) -> Option< String >
{
  std::env::var( var ).ok().filter( | v | !v.is_empty() )
}

/// Apply `CLR_*` environment variable fallbacks for the 36 run parameters.
///
/// Each field is updated only when it is still at its zero/default value — the CLI
/// flag always wins when both are present (CLI-wins field-default check).
///
/// Returns `Err` for env vars with values that fail validation: `CLR_EXPECT_STRATEGY`
/// (invalid strategy name) and `CLR_EXPECT_RETRIES` (exceeds u8 range).  All other
/// env var parse failures are silently ignored so operators can set global env vars
/// safely without breaking unconfigured invocations.
#[ allow( clippy::too_many_lines ) ] // env-var mapping is inherently wide — one branch per var.
pub( crate ) fn apply_env_vars( parsed : &mut CliArgs ) -> Result< () >
{
  if parsed.message.is_none()              { parsed.message              = env_str( "CLR_MESSAGE" ); }
  if !parsed.print_mode                    { parsed.print_mode           = env_bool( "CLR_PRINT" ); }
  if parsed.model.is_none()               { parsed.model                = env_str( "CLR_MODEL" ); }
  if !parsed.verbose                       { parsed.verbose              = env_bool( "CLR_VERBOSE" ); }
  if !parsed.no_skip_permissions           { parsed.no_skip_permissions  = env_bool( "CLR_NO_SKIP_PERMISSIONS" ); }
  if !parsed.interactive                   { parsed.interactive          = env_bool( "CLR_INTERACTIVE" ); }
  if !parsed.new_session                   { parsed.new_session          = env_bool( "CLR_NEW_SESSION" ); }
  if parsed.dir.is_none()                 { parsed.dir                  = env_str( "CLR_DIR" ); }
  if parsed.max_tokens.is_none()
  {
    if let Some( v ) = env_str( "CLR_MAX_TOKENS" ) { parsed.max_tokens = v.parse::< u32 >().ok(); }
  }
  if parsed.session_dir.is_none()         { parsed.session_dir          = env_str( "CLR_SESSION_DIR" ); }
  if !parsed.dry_run                       { parsed.dry_run              = env_bool( "CLR_DRY_RUN" ); }
  // Fix(BUG-213): `CLR_VERBOSITY` overwrote an explicit `--verbosity N` CLI flag when N equalled
  //   the default (3) — indistinguishable from an unset field because `verbosity` was non-optional.
  // Root cause: non-optional field whose default is non-zero/non-false cannot act as a "set" sentinel;
  //   `parsed.verbosity == VerbosityLevel::default()` misfires when the user explicitly passes that value.
  // Pitfall: use `Option<T>` (never `T == default()`) for any env-var-fallback field whose default
  //   is non-false/non-zero; equality-with-default is always ambiguous as a set-sentinel.
  if parsed.verbosity.is_none()
  {
    if let Some( v ) = env_str( "CLR_VERBOSITY" )
    {
      if let Ok( level ) = v.parse::< VerbosityLevel >() { parsed.verbosity = Some( level ); }
    }
  }
  if !parsed.trace                         { parsed.trace                = env_bool( "CLR_TRACE" ); }
  if !parsed.no_ultrathink                 { parsed.no_ultrathink        = env_bool( "CLR_NO_ULTRATHINK" ); }
  if parsed.system_prompt.is_none()       { parsed.system_prompt        = env_str( "CLR_SYSTEM_PROMPT" ); }
  if parsed.append_system_prompt.is_none(){ parsed.append_system_prompt = env_str( "CLR_APPEND_SYSTEM_PROMPT" ); }
  if parsed.effort.is_none()
  {
    if let Some( v ) = env_str( "CLR_EFFORT" ) { parsed.effort = v.parse::< EffortLevel >().ok(); }
  }
  if !parsed.no_effort_max                 { parsed.no_effort_max        = env_bool( "CLR_NO_EFFORT_MAX" ); }
  if !parsed.no_chrome                     { parsed.no_chrome            = env_bool( "CLR_NO_CHROME" ); }
  if !parsed.no_persist                    { parsed.no_persist           = env_bool( "CLR_NO_PERSIST" ); }
  if parsed.json_schema.is_none()         { parsed.json_schema          = env_str( "CLR_JSON_SCHEMA" ); }
  if parsed.mcp_config.is_empty()
  {
    if let Some( v ) = env_str( "CLR_MCP_CONFIG" ) { parsed.mcp_config.push( v ); }
  }
  if parsed.file.is_none()             { parsed.file             = env_str( "CLR_FILE" ); }
  if !parsed.strip_fences              { parsed.strip_fences     = env_bool( "CLR_STRIP_FENCES" ); }
  if !parsed.keep_claudecode           { parsed.keep_claudecode  = env_bool( "CLR_KEEP_CLAUDECODE" ); }
  // Fix(BUG-233): validate CLR_SUBDIR same as --subdir — reject `/` in the value.
  // Root cause: CLR_SUBDIR env var was accepted without the slash-rejection guard applied to --subdir.
  // Pitfall: env-var fallbacks for validated flags must replicate the same validation as the flag parser.
  // Matches apply_env_vars convention: silently ignore invalid env values.
  if parsed.subdir.is_none()
  {
    if let Some( v ) = env_str( "CLR_SUBDIR" )
    {
      if !v.contains( '/' ) { parsed.subdir = Some( v ); }
    }
  }
  if parsed.output_file.is_none()  { parsed.output_file  = env_str( "CLR_OUTPUT_FILE" ); }
  if parsed.expect.is_none()       { parsed.expect        = env_str( "CLR_EXPECT" ); }
  if parsed.expect_strategy.is_none()
  {
    if let Some( v ) = env_str( "CLR_EXPECT_STRATEGY" )
    {
      parsed.expect_strategy = Some(
        v.parse::< ExpectStrategy >().map_err( | e |
          Error::msg( format!( "CLR_EXPECT_STRATEGY: {e}" ) )
        )?
      );
    }
  }
  if parsed.expect_retries.is_none()
  {
    if let Some( v ) = env_str( "CLR_EXPECT_RETRIES" )
    {
      parsed.expect_retries = Some(
        parse_u8_bounded( &v, "--expect-retries" ).map_err( | e |
          Error::msg( format!( "CLR_EXPECT_RETRIES: {e}" ) )
        )?
      );
    }
  }
  if parsed.max_sessions.is_none()
  {
    if let Some( v ) = env_str( "CLR_MAX_SESSIONS" )
    {
      parsed.max_sessions = v.parse::< u32 >().ok();
    }
  }
  if parsed.retry_on_rate_limit.is_none()
  {
    if let Some( v ) = env_str( "CLR_RETRY_ON_RATE_LIMIT" )
    {
      parsed.retry_on_rate_limit = v.parse::< u8 >().ok();
    }
  }
  if parsed.retry_delay.is_none()
  {
    if let Some( v ) = env_str( "CLR_RETRY_DELAY" )
    {
      parsed.retry_delay = v.parse::< u32 >().ok();
    }
  }
  if parsed.timeout.is_none()
  {
    if let Some( v ) = env_str( "CLR_TIMEOUT" )
    {
      parsed.timeout = v.parse::< u32 >().ok();
    }
  }
  if parsed.retry_on_api_error.is_none()
  {
    if let Some( v ) = env_str( "CLR_RETRY_ON_API_ERROR" )
    {
      parsed.retry_on_api_error = v.parse::< u8 >().ok();
    }
  }
  if parsed.api_error_delay.is_none()
  {
    if let Some( v ) = env_str( "CLR_API_ERROR_DELAY" )
    {
      parsed.api_error_delay = v.parse::< u32 >().ok();
    }
  }
  if parsed.retry_on_unknown_error.is_none()
  {
    if let Some( v ) = env_str( "CLR_RETRY_ON_UNKNOWN_ERROR" )
    {
      parsed.retry_on_unknown_error = v.parse::< u8 >().ok();
    }
  }
  Ok( () )
}
