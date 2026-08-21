use std::io::IsTerminal;
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

/// Return the path to the JSON config source: `--args-file` CLI value or `CLR_ARGS_FILE` env var.
///
/// CLI value wins over env var — mirrors the 4-tier precedence (CLI > JSON > CLR_* > default)
/// at the "which file?" level: if the user named a file on the command line, it is used;
/// otherwise `CLR_ARGS_FILE` provides the path.  Returns `None` when neither is set.
pub( super ) fn resolve_args_file_path( cli_path : Option< &str > ) -> Option< String >
{
  cli_path.map( ToString::to_string ).or_else( || env_str( "CLR_ARGS_FILE" ) )
}

/// The result of reading piped stdin: either a JSON config object candidate, or arbitrary
/// content to forward verbatim to the subprocess's own stdin.
pub( super ) enum StdinPayload
{
  /// Content whose first non-whitespace byte is `{` — a JSON object candidate.
  Json( String ),
  /// Content that is not a JSON object — forwarded byte-for-byte, never UTF-8-validated.
  Raw( Vec< u8 > ),
}

/// Detect and read stdin piped to the CLI, distinguishing a JSON config object from
/// arbitrary content to forward.
///
/// Returns `Some(StdinPayload::Json(_))` or `Some(StdinPayload::Raw(_))` whenever stdin is
/// actually read — i.e. when all of these hold:
/// - `--no-stdin` is absent from `tokens` and `CLR_NO_STDIN` is unset (explicit opt-out —
///   stdin is left entirely untouched, even when piped)
/// - `--file` is absent from `tokens` (raw scan; `--file` gates out stdin detection
///   for `run`/`ask` because `--file` already reserves stdin/file content for the message)
/// - stdin is not attached to a TTY (i.e. it is a pipe or redirect)
///
/// Which variant depends on the first non-whitespace byte: `{` selects `Json`, anything
/// else selects `Raw`. Returns `None` only when a gate blocks the read entirely — stdin is
/// left untouched in that case. Consumes stdin — must be called before any other operation
/// that reads from stdin.
// Fix(BUG-424): forward non-JSON piped stdin instead of silently discarding it.
// Root cause: read_to_string() discarded `src` on the non-JSON branch, and separately could
//   never succeed at all for non-UTF-8 (binary) stdin content, losing both cases outright.
// Pitfall: read_to_end()/Vec<u8> is required, not just a return-type change — a lossy
//   String round-trip (e.g. from_utf8_lossy) would corrupt binary content before the sniff
//   check ever ran, reintroducing a narrower version of the same bug.
pub( super ) fn detect_stdin_json( tokens : &[ String ] ) -> Option< StdinPayload >
{
  // Gate 0: --no-stdin / CLR_NO_STDIN opt out of stdin handling entirely.
  // Fix(BUG-492): non-TTY stdin was read unconditionally with a blocking read_to_end;
  //   a held-open pipe (`tail -f |`, a FIFO with a live writer, a supervisor-inherited
  //   fd) hung clr forever, before argument parsing could even reject anything.
  // Root cause: no opt-out existed ahead of the blocking read — TTY detection alone
  //   cannot distinguish "pipe with data" from "pipe that never closes".
  // Pitfall: this must stay a raw token/env scan BEFORE the read — a parsed-flag check
  //   would run after stdin was already consumed (parsing receives stdin content as input).
  if tokens.iter().any( | t | t == "--no-stdin" ) || env_bool( "CLR_NO_STDIN" ) { return None; }
  // Gate 1: --file bypasses stdin detection for run/ask.
  if tokens.iter().any( | t | t == "--file" ) { return None; }
  // Gate 2: TTY stdin is interactive — not a pipe.
  if std::io::stdin().is_terminal() { return None; }
  // Read raw stdin bytes — byte-safe, no UTF-8 validation, so binary content survives intact.
  let mut src = Vec::new();
  std::io::Read::read_to_end( &mut std::io::stdin().lock(), &mut src ).ok();
  // Gate 3: JSON object detection — must open with `{` (leading ASCII whitespace skipped,
  // mirroring the previous String::trim_start() check, now at the byte level).
  let first_non_ws = src.iter().find( | & & b | !b.is_ascii_whitespace() );
  if first_non_ws == Some( &b'{' )
  {
    match String::from_utf8( src )
    {
      Ok( text ) => Some( StdinPayload::Json( text ) ),
      Err( e )   => Some( StdinPayload::Raw( e.into_bytes() ) ),
    }
  }
  else
  {
    Some( StdinPayload::Raw( src ) )
  }
}

/// Detect and read a JSON config object piped to stdin, without any token gating.
///
/// Used by `isolated` and `refresh` dispatchers — unlike `detect_stdin_json`, there is no
/// `--file` guard because these subcommands do not use `--file` to pipe message content.
/// Returns `Some(json_string)` when stdin is not a TTY and the content starts with `{`.
pub( super ) fn detect_stdin_json_unconstrained() -> Option< String >
{
  if std::io::stdin().is_terminal() { return None; }
  let mut src = String::new();
  std::io::Read::read_to_string( &mut std::io::stdin().lock(), &mut src ).ok();
  if src.trim_start().starts_with( '{' ) { Some( src ) } else { None }
}

/// Apply `CLR_*` environment variable fallbacks for the 61 run parameters.
///
/// Each field is updated only when it is still at its zero/default value — the CLI
/// flag always wins when both are present (CLI-wins field-default check).
///
/// Returns `Err` for env vars with values that fail validation: `CLR_EXPECT_STRATEGY`
/// (invalid strategy name) and `CLR_RETRY_ON_VALIDATION` (exceeds u8 range).  All other
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
  if parsed.from.is_none()                 { parsed.from                 = env_str( "CLR_FROM" ); }
  if !parsed.dry_run                       { parsed.dry_run              = env_bool( "CLR_DRY_RUN" ); }
  if !parsed.quiet                         { parsed.quiet                = env_bool( "CLR_QUIET" ); }
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
  if !parsed.no_stdin                      { parsed.no_stdin             = env_bool( "CLR_NO_STDIN" ); }
  if parsed.json_schema.is_none()         { parsed.json_schema          = env_str( "CLR_JSON_SCHEMA" ); }
  if parsed.mcp_config.is_empty()
  {
    if let Some( v ) = env_str( "CLR_MCP_CONFIG" ) { parsed.mcp_config.push( v ); }
  }
  if parsed.file.is_none()             { parsed.file             = env_str( "CLR_FILE" ); }
  if !parsed.strip_fences              { parsed.strip_fences     = env_bool( "CLR_STRIP_FENCES" ); }
  if !parsed.keep_claudecode           { parsed.keep_claudecode  = env_bool( "CLR_KEEP_CLAUDECODE" ); }
  if !parsed.keep_clone                { parsed.keep_clone       = env_bool( "CLR_KEEP_CLONE" ); }
  // Fix(BUG-233): validate CLR_TOPIC same as --topic — reject `/` in the value.
  // Root cause: CLR_TOPIC env var was accepted without the slash-rejection guard applied to --topic.
  // Pitfall: env-var fallbacks for validated flags must replicate the same validation as the flag parser.
  // Matches apply_env_vars convention: silently ignore invalid env values.
  if parsed.topic.is_none()
  {
    if let Some( v ) = env_str( "CLR_TOPIC" )
    {
      if !v.contains( '/' ) { parsed.topic = Some( v ); }
    }
  }
  // Matches apply_env_vars convention: silently ignore invalid env values (same as CLR_TOPIC).
  if parsed.topic_mode.is_none()
  {
    if let Some( v ) = env_str( "CLR_TOPIC_MODE" )
    {
      parsed.topic_mode = v.parse::< super::topic_path::TopicMode >().ok();
    }
  }
  if !parsed.global                { parsed.global       = env_bool( "CLR_GLOBAL" ); }
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
  if parsed.max_sessions.is_none()
  {
    if let Some( v ) = env_str( "CLR_MAX_SESSIONS" )
    {
      parsed.max_sessions = v.parse::< u32 >().ok();
    }
  }
  // Fix: give CLR_GATE_POLL_SECS/CLR_GATE_MAX_ATTEMPTS/CLR_GATE_STALE_SECS the same
  // CLI-flag + config.toml tier parity every other numeric knob already has (see
  // gate_limits.rs's gate_poll_secs_from()/gate_max_attempts_from()/gate_stale_secs_from()
  // for the pure parse-or-default siblings this mirrors). An invalid value here
  // leaves the field None (not the hardcoded default) so config.toml still gets a
  // chance to contribute before the final unwrap_or() at the call site applies —
  // same silently-ignore-invalid convention as CLR_MAX_SESSIONS above.
  if parsed.gate_poll_secs.is_none()
  {
    if let Some( v ) = env_str( "CLR_GATE_POLL_SECS" )
    {
      parsed.gate_poll_secs = v.parse::< u64 >().ok();
    }
  }
  if parsed.gate_max_attempts.is_none()
  {
    if let Some( v ) = env_str( "CLR_GATE_MAX_ATTEMPTS" )
    {
      parsed.gate_max_attempts = v.parse::< u32 >().ok();
    }
  }
  if parsed.gate_stale_secs.is_none()
  {
    if let Some( v ) = env_str( "CLR_GATE_STALE_SECS" )
    {
      parsed.gate_stale_secs = v.parse::< u64 >().ok();
    }
  }
  if parsed.retry_on_transient.is_none()
  {
    if let Some( v ) = env_str( "CLR_RETRY_ON_TRANSIENT" )
    {
      parsed.retry_on_transient = v.parse::< u8 >().ok();
    }
  }
  if parsed.transient_delay.is_none()
  {
    if let Some( v ) = env_str( "CLR_TRANSIENT_DELAY" )
    {
      parsed.transient_delay = v.parse::< u32 >().ok();
    }
  }
  if parsed.timeout.is_none()
  {
    if let Some( v ) = env_str( "CLR_TIMEOUT" )
    {
      parsed.timeout = v.parse::< u32 >().ok();
    }
  }
  if parsed.retry_on_account.is_none()
  {
    if let Some( v ) = env_str( "CLR_RETRY_ON_ACCOUNT" )
    {
      parsed.retry_on_account = v.parse::< u8 >().ok();
    }
  }
  if parsed.account_delay.is_none()
  {
    if let Some( v ) = env_str( "CLR_ACCOUNT_DELAY" )
    {
      parsed.account_delay = v.parse::< u32 >().ok();
    }
  }
  if parsed.retry_on_auth.is_none()
  {
    if let Some( v ) = env_str( "CLR_RETRY_ON_AUTH" )
    {
      parsed.retry_on_auth = v.parse::< u8 >().ok();
    }
  }
  if parsed.auth_delay.is_none()
  {
    if let Some( v ) = env_str( "CLR_AUTH_DELAY" )
    {
      parsed.auth_delay = v.parse::< u32 >().ok();
    }
  }
  if parsed.retry_on_service.is_none()
  {
    if let Some( v ) = env_str( "CLR_RETRY_ON_SERVICE" )
    {
      parsed.retry_on_service = v.parse::< u8 >().ok();
    }
  }
  if parsed.service_delay.is_none()
  {
    if let Some( v ) = env_str( "CLR_SERVICE_DELAY" )
    {
      parsed.service_delay = v.parse::< u32 >().ok();
    }
  }
  if parsed.retry_on_process.is_none()
  {
    if let Some( v ) = env_str( "CLR_RETRY_ON_PROCESS" )
    {
      parsed.retry_on_process = v.parse::< u8 >().ok();
    }
  }
  if parsed.process_delay.is_none()
  {
    if let Some( v ) = env_str( "CLR_PROCESS_DELAY" )
    {
      parsed.process_delay = v.parse::< u32 >().ok();
    }
  }
  if parsed.retry_on_validation.is_none()
  {
    if let Some( v ) = env_str( "CLR_RETRY_ON_VALIDATION" )
    {
      parsed.retry_on_validation = Some(
        parse_u8_bounded( &v, "--retry-on-validation" ).map_err( | e |
          Error::msg( format!( "CLR_RETRY_ON_VALIDATION: {e}" ) )
        )?
      );
    }
  }
  if parsed.validation_delay.is_none()
  {
    if let Some( v ) = env_str( "CLR_VALIDATION_DELAY" )
    {
      parsed.validation_delay = v.parse::< u32 >().ok();
    }
  }
  if parsed.retry_on_runner.is_none()
  {
    if let Some( v ) = env_str( "CLR_RETRY_ON_RUNNER" )
    {
      parsed.retry_on_runner = v.parse::< u8 >().ok();
    }
  }
  if parsed.runner_delay.is_none()
  {
    if let Some( v ) = env_str( "CLR_RUNNER_DELAY" )
    {
      parsed.runner_delay = v.parse::< u32 >().ok();
    }
  }
  if parsed.retry_on_unknown.is_none()
  {
    if let Some( v ) = env_str( "CLR_RETRY_ON_UNKNOWN" )
    {
      parsed.retry_on_unknown = v.parse::< u8 >().ok();
    }
  }
  if parsed.unknown_delay.is_none()
  {
    if let Some( v ) = env_str( "CLR_UNKNOWN_DELAY" )
    {
      parsed.unknown_delay = v.parse::< u32 >().ok();
    }
  }
  if parsed.retry_override.is_none()
  {
    if let Some( v ) = env_str( "CLR_RETRY_OVERRIDE" )
    {
      parsed.retry_override = v.parse::< u8 >().ok();
    }
  }
  if parsed.retry_override_delay.is_none()
  {
    if let Some( v ) = env_str( "CLR_RETRY_OVERRIDE_DELAY" )
    {
      parsed.retry_override_delay = v.parse::< u32 >().ok();
    }
  }
  if parsed.retry_default.is_none()
  {
    if let Some( v ) = env_str( "CLR_RETRY_DEFAULT" )
    {
      parsed.retry_default = v.parse::< u8 >().ok();
    }
  }
  if parsed.retry_default_delay.is_none()
  {
    if let Some( v ) = env_str( "CLR_RETRY_DEFAULT_DELAY" )
    {
      parsed.retry_default_delay = v.parse::< u32 >().ok();
    }
  }
  if parsed.output_format.is_none()    { parsed.output_format    = env_str( "CLR_OUTPUT_FORMAT" ); }
  if parsed.max_turns.is_none()        { parsed.max_turns        = env_str( "CLR_MAX_TURNS" ); }
  if parsed.allowed_tools.is_none()    { parsed.allowed_tools    = env_str( "CLR_ALLOWED_TOOLS" ); }
  if parsed.disallowed_tools.is_none() { parsed.disallowed_tools = env_str( "CLR_DISALLOWED_TOOLS" ); }
  if parsed.max_budget_usd.is_none()   { parsed.max_budget_usd   = env_str( "CLR_MAX_BUDGET_USD" ); }
  if parsed.add_dir.is_none()          { parsed.add_dir          = env_str( "CLR_ADD_DIR" ); }
  if parsed.fallback_model.is_none()   { parsed.fallback_model   = env_str( "CLR_FALLBACK_MODEL" ); }
  if parsed.output_style.is_none()
  {
    if let Some( v ) = env_str( "CLR_OUTPUT_STYLE" )
    {
      if !matches!( v.as_str(), "summary" | "raw" )
      {
        return Err( Error::msg( format!(
          "CLR_OUTPUT_STYLE: invalid value '{v}' — expected: summary, raw"
        ) ) );
      }
      parsed.output_style = Some( v );
    }
  }
  if parsed.summary_fields.is_none()
  {
    if let Some( v ) = env_str( "CLR_SUMMARY_FIELDS" )
    {
      if super::summary::resolve_fields( &v ).is_err()
      {
        return Err( Error::msg( format!(
          "CLR_SUMMARY_FIELDS: invalid value '{v}'"
        ) ) );
      }
      parsed.summary_fields = Some( v );
    }
  }
  if parsed.journal.is_none()
  {
    if let Some( v ) = env_str( "CLR_JOURNAL" )
    {
      if !matches!( v.as_str(), "full" | "meta" | "off" )
      {
        return Err( Error::msg( format!(
          "CLR_JOURNAL: invalid value '{v}' — expected: full, meta, off"
        ) ) );
      }
      parsed.journal = Some( v );
    }
  }
  if parsed.journal_dir.is_none() { parsed.journal_dir = env_str( "CLR_JOURNAL_DIR" ); }
  if !parsed.no_compact_window { parsed.no_compact_window = env_bool( "CLR_NO_COMPACT_WINDOW" ); }
  Ok( () )
}

/// Read `CLR_PS_MODE`, `CLR_PS_COLUMNS`, `CLR_PS_PID`, `CLR_PS_ANCIENT_SECS`, and
/// `CLR_PS_HIGH_RAM_MB` env-var defaults for `clr ps`.
///
/// Returns `(mode, columns, pids, ancient_secs, high_ram_mb)` — `mode` and `columns` are
/// `None` when absent or empty; `pids` is an empty `Vec` when `CLR_PS_PID` is absent or
/// contains no parseable PIDs. Non-numeric entries in `CLR_PS_PID` are silently ignored.
/// `ancient_secs` defaults to 28800 (8 h); `high_ram_mb` defaults to 400. Invalid values
/// for either threshold are silently ignored and the default is used instead.
/// The caller applies these as defaults before parsing CLI tokens; CLI values
/// always overwrite env-var values (CLI-wins).
pub( super ) fn apply_ps_env_vars()
  -> ( Option< String >, Option< String >, Vec< u32 >, u64, u64 )
{
  let pids = env_str( "CLR_PS_PID" )
    .map( | csv |
    {
      csv.split( ',' )
        .filter_map( | s | s.trim().parse::< u32 >().ok() )
        .collect()
    } )
    .unwrap_or_default();
  let ancient_secs = env_str( "CLR_PS_ANCIENT_SECS" )
    .and_then( | v | v.parse::< u64 >().ok() )
    .unwrap_or( 28_800 );
  let high_ram_mb = env_str( "CLR_PS_HIGH_RAM_MB" )
    .and_then( | v | v.parse::< u64 >().ok() )
    .unwrap_or( 400 );
  ( env_str( "CLR_PS_MODE" ), env_str( "CLR_PS_COLUMNS" ), pids, ancient_secs, high_ram_mb )
}
