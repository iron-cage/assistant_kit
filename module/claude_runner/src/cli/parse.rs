use claude_runner_core::EffortLevel;
use error_tools::{ Error, Result };

/// Strategy for `--expect` output validation.
///
/// Determines how `run_print_mode` behaves when the captured output does not
/// match any value listed in `--expect`.
pub( crate ) enum ExpectStrategy
{
  /// Exit 3 immediately on first mismatch (default).
  Fail,
  /// Re-invoke the subprocess up to `--retry-on-validation` more times; exit 3 if exhausted.
  Retry,
  /// Print the fallback value and exit 0 regardless of subprocess output.
  Default( String ),
}

impl core::str::FromStr for ExpectStrategy
{
  type Err = String;
  fn from_str( s : &str ) -> core::result::Result< Self, Self::Err >
  {
    match s
    {
      "fail"  => Ok( ExpectStrategy::Fail ),
      "retry" => Ok( ExpectStrategy::Retry ),
      _ if s.starts_with( "default:" ) =>
      {
        let val = s[ "default:".len() .. ].to_string();
        Ok( ExpectStrategy::Default( val ) )
      }
      _ => Err( format!(
        "invalid --expect-strategy value: {s}\nExpected: fail, retry, or default:<VALUE>"
      ) ),
    }
  }
}

/// Parsed CLI arguments.
#[ allow( clippy::struct_excessive_bools ) ]
#[ derive( Default ) ]
pub( crate ) struct CliArgs
{
  pub( crate ) message              : Option< String >,
  pub( crate ) print_mode           : bool,
  pub( crate ) interactive          : bool,
  pub( crate ) new_session          : bool,
  pub( crate ) model                : Option< String >,
  pub( crate ) verbose              : bool,
  pub( crate ) no_skip_permissions  : bool,
  pub( crate ) max_tokens           : Option< u32 >,
  pub( crate ) session_dir          : Option< String >,
  pub( crate ) dir                  : Option< String >,
  pub( crate ) dry_run              : bool,
  pub( crate ) trace                : bool,
  pub( crate ) quiet                : bool,
  pub( crate ) help                 : bool,
  pub( crate ) system_prompt        : Option< String >,
  pub( crate ) append_system_prompt : Option< String >,
  pub( crate ) no_ultrathink        : bool,
  pub( crate ) effort               : Option< EffortLevel >,
  pub( crate ) no_effort_max        : bool,
  pub( crate ) no_chrome            : bool,
  pub( crate ) no_persist           : bool,
  pub( crate ) json_schema          : Option< String >,
  pub( crate ) mcp_config           : Vec< String >,
  pub( crate ) file                 : Option< String >,
  pub( crate ) strip_fences         : bool,
  pub( crate ) keep_claudecode      : bool,
  pub( crate ) subdir               : Option< String >,
  pub( crate ) output_file          : Option< String >,
  pub( crate ) expect               : Option< String >,
  pub( crate ) expect_strategy      : Option< ExpectStrategy >,
  pub( crate ) max_sessions            : Option< u32 >,
  pub( crate ) retry_on_transient      : Option< u8 >,
  pub( crate ) transient_delay         : Option< u32 >,
  pub( crate ) timeout                 : Option< u32 >,
  pub( crate ) retry_on_account        : Option< u8 >,
  pub( crate ) account_delay           : Option< u32 >,
  pub( crate ) retry_on_auth           : Option< u8 >,
  pub( crate ) auth_delay              : Option< u32 >,
  pub( crate ) retry_on_service        : Option< u8 >,
  pub( crate ) service_delay           : Option< u32 >,
  pub( crate ) retry_on_process        : Option< u8 >,
  pub( crate ) process_delay           : Option< u32 >,
  pub( crate ) retry_on_validation     : Option< u8 >,
  pub( crate ) validation_delay        : Option< u32 >,
  pub( crate ) retry_on_runner         : Option< u8 >,
  pub( crate ) runner_delay            : Option< u32 >,
  pub( crate ) retry_on_unknown        : Option< u8 >,
  pub( crate ) unknown_delay           : Option< u32 >,
  pub( crate ) retry_override          : Option< u8 >,
  pub( crate ) retry_override_delay    : Option< u32 >,
  pub( crate ) retry_default           : Option< u8 >,
  pub( crate ) retry_default_delay     : Option< u32 >,
  pub( crate ) output_format           : Option< String >,
  pub( crate ) max_turns               : Option< String >,
  pub( crate ) allowed_tools           : Option< String >,
  pub( crate ) disallowed_tools        : Option< String >,
  pub( crate ) max_budget_usd          : Option< String >,
  pub( crate ) add_dir                 : Option< String >,
  pub( crate ) fallback_model          : Option< String >,
  pub( crate ) output_style            : Option< String >,
  pub( crate ) summary_fields         : Option< String >,
  pub( crate ) journal                : Option< String >,
  pub( crate ) journal_dir            : Option< String >,
  pub( crate ) args_file              : Option< String >,
  pub( crate ) no_compact_window      : bool,
}

/// Consume the next argv element as a flag's value.
pub( super ) fn next_value<'a>( tokens : &'a [ String ], idx : usize, flag : &str ) -> Result< &'a str >
{
  tokens.get( idx ).map( String::as_str ).ok_or_else( ||
    Error::msg( format!( "{flag} requires a value" ) )
  )
}

/// Parse a raw string as a u32 token limit with a clear error message.
///
/// Called from `parse_value_flag()`. Isolates multi-line parse logic so each
/// value-consuming arm in `parse_value_flag` stays single-expression.
fn parse_token_limit( raw : &str ) -> Result< u32 >
{
  raw.parse::< u32 >().map_err( | _ |
    Error::msg( format!(
      "invalid --max-tokens value: {raw}\n\
       Expected unsigned integer 0–4294967295"
    ) )
  )
}

/// Parse a raw string as an `EffortLevel` with a clear error message.
///
/// Called from `parse_value_flag()`. Delegates to `EffortLevel::from_str`.
fn parse_effort_level( raw : &str ) -> Result< EffortLevel >
{
  raw.parse::< EffortLevel >().map_err( Error::msg )
}

/// Parse a raw string as an `ExpectStrategy` with a clear error message.
///
/// Called from `parse_value_flag()`. Delegates to `ExpectStrategy::from_str`.
fn parse_expect_strategy( raw : &str ) -> Result< ExpectStrategy >
{
  raw.parse::< ExpectStrategy >().map_err( Error::msg )
}

/// Parse a raw string as a bounded u8 (0–255) with a labeled error message.
///
/// Used for retry count flags like `--retry-on-transient` and `--retry-on-service`.
pub( crate ) fn parse_u8_bounded( raw : &str, flag_name : &str ) -> Result< u8 >
{
  raw.parse::< u32 >()
    .ok()
    .and_then( | v | u8::try_from( v ).ok() )
    .ok_or_else( || Error::msg( format!(
      "invalid {flag_name} value: {raw}\nExpected integer 0–255"
    ) ) )
}

/// Parse a raw string as a u32 flag value with a labeled error message and hint.
///
/// Used for flags like `--max-sessions`, `--transient-delay`, and `--timeout`.
fn parse_u32_flag( raw : &str, flag_name : &str, hint : &str ) -> Result< u32 >
{
  raw.parse::< u32 >().map_err( | _ |
    Error::msg( format!(
      "invalid {flag_name} value: {raw}\nExpected unsigned integer{hint}"
    ) )
  )
}

/// Parse a value-consuming flag (`--flag value` pair) into `parsed`.
///
/// Handles flags that are forwarded to the Claude command line or modify
/// the subprocess environment. Falls through to `parse_runner_value_flag`
/// for runner-behavior flags (output capture, validation, concurrency, timeouts).
///
/// Returns `true` when `token` is a recognised value-consuming flag and its
/// following value was consumed into `parsed`. Returns `false` when `token`
/// is not a known value-consuming flag (caller decides whether to treat it
/// as unknown). `next` is the index of the token immediately after `token`.
fn parse_value_flag(
  token  : &str,
  tokens : &[ String ],
  next   : usize,
  parsed : &mut CliArgs,
) -> Result< bool >
{
  match token
  {
    "--effort" =>
    {
      parsed.effort = Some(
        parse_effort_level( next_value( tokens, next, "--effort" )? )?
      );
    }
    "--message" =>
    {
      parsed.message = Some( next_value( tokens, next, "--message" )?.to_string() );
    }
    "--system-prompt" =>
    {
      parsed.system_prompt = Some( next_value( tokens, next, "--system-prompt" )?.to_string() );
    }
    "--append-system-prompt" =>
    {
      parsed.append_system_prompt = Some( next_value( tokens, next, "--append-system-prompt" )?.to_string() );
    }
    "--model" =>
    {
      parsed.model = Some( next_value( tokens, next, "--model" )?.to_string() );
    }
    "--max-tokens" =>
    {
      parsed.max_tokens = Some( parse_token_limit( next_value( tokens, next, "--max-tokens" )? )? );
    }
    "--session-dir" =>
    {
      parsed.session_dir = Some( next_value( tokens, next, "--session-dir" )?.to_string() );
    }
    "--dir" =>
    {
      parsed.dir = Some( next_value( tokens, next, "--dir" )?.to_string() );
    }
    "--json-schema" =>
    {
      parsed.json_schema = Some( next_value( tokens, next, "--json-schema" )?.to_string() );
    }
    "--mcp-config" =>
    {
      parsed.mcp_config.push( next_value( tokens, next, "--mcp-config" )?.to_string() );
    }
    "--file" =>
    {
      parsed.file = Some( next_value( tokens, next, "--file" )?.to_string() );
    }
    // Fix(BUG-230): reject subdir names containing `/` — spec requires single name component
    // Root cause: no validation; `create_dir_all` silently created nested dirs for `a/b`
    // Pitfall: must reject `/` in the value, not just leading `/` — any separator violates
    // the "directory name component" type constraint in 028_subdir.md
    "--subdir" =>
    {
      let val = next_value( tokens, next, "--subdir" )?;
      if val.contains( '/' )
      {
        return Err( Error::msg(
          "--subdir must be a single directory name component (no '/' separators)"
        ) );
      }
      parsed.subdir = Some( val.to_string() );
    }
    "--output-format" =>
    {
      parsed.output_format = Some( next_value( tokens, next, "--output-format" )?.to_string() );
    }
    "--max-turns" =>
    {
      parsed.max_turns = Some( next_value( tokens, next, "--max-turns" )?.to_string() );
    }
    "--allowed-tools" =>
    {
      parsed.allowed_tools = Some( next_value( tokens, next, "--allowed-tools" )?.to_string() );
    }
    "--disallowed-tools" =>
    {
      parsed.disallowed_tools = Some( next_value( tokens, next, "--disallowed-tools" )?.to_string() );
    }
    "--max-budget-usd" =>
    {
      parsed.max_budget_usd = Some( next_value( tokens, next, "--max-budget-usd" )?.to_string() );
    }
    "--add-dir" =>
    {
      parsed.add_dir = Some( next_value( tokens, next, "--add-dir" )?.to_string() );
    }
    "--fallback-model" =>
    {
      parsed.fallback_model = Some( next_value( tokens, next, "--fallback-model" )?.to_string() );
    }
    "--args-file" =>
    {
      parsed.args_file = Some( next_value( tokens, next, "--args-file" )?.to_string() );
    }
    _ => return parse_runner_value_flag( token, tokens, next, parsed ),
  }
  Ok( true )
}

/// Parse runner-behavior value flags into `parsed`.
///
/// Handles flags that control output capture, expect validation, session concurrency,
/// retry logic, and subprocess timeouts — none of which are forwarded to the claude
/// command line directly.
#[ allow( clippy::too_many_lines ) ] // mechanical dispatch — one arm per retry param.
fn parse_runner_value_flag(
  token  : &str,
  tokens : &[ String ],
  next   : usize,
  parsed : &mut CliArgs,
) -> Result< bool >
{
  match token
  {
    "--output-file" =>
    {
      parsed.output_file = Some( next_value( tokens, next, "--output-file" )?.to_string() );
    }
    "--expect" =>
    {
      parsed.expect = Some( next_value( tokens, next, "--expect" )?.to_string() );
    }
    "--expect-strategy" =>
    {
      parsed.expect_strategy = Some(
        parse_expect_strategy( next_value( tokens, next, "--expect-strategy" )? )?
      );
    }
    "--max-sessions" =>
    {
      parsed.max_sessions = Some(
        parse_u32_flag( next_value( tokens, next, "--max-sessions" )?, "--max-sessions", " (0 = unlimited)" )?
      );
    }
    "--retry-on-transient" =>
    {
      parsed.retry_on_transient = Some(
        parse_u8_bounded( next_value( tokens, next, "--retry-on-transient" )?, "--retry-on-transient" )?
      );
    }
    "--transient-delay" =>
    {
      parsed.transient_delay = Some(
        parse_u32_flag( next_value( tokens, next, "--transient-delay" )?, "--transient-delay", " (seconds)" )?
      );
    }
    "--timeout" =>
    {
      parsed.timeout = Some(
        parse_u32_flag( next_value( tokens, next, "--timeout" )?, "--timeout", " (seconds; 0 = unlimited)" )?
      );
    }
    "--retry-on-account" =>
    {
      parsed.retry_on_account = Some(
        parse_u8_bounded( next_value( tokens, next, "--retry-on-account" )?, "--retry-on-account" )?
      );
    }
    "--account-delay" =>
    {
      parsed.account_delay = Some(
        parse_u32_flag( next_value( tokens, next, "--account-delay" )?, "--account-delay", " (seconds)" )?
      );
    }
    "--retry-on-auth" =>
    {
      parsed.retry_on_auth = Some(
        parse_u8_bounded( next_value( tokens, next, "--retry-on-auth" )?, "--retry-on-auth" )?
      );
    }
    "--auth-delay" =>
    {
      parsed.auth_delay = Some(
        parse_u32_flag( next_value( tokens, next, "--auth-delay" )?, "--auth-delay", " (seconds)" )?
      );
    }
    "--retry-on-service" =>
    {
      parsed.retry_on_service = Some(
        parse_u8_bounded( next_value( tokens, next, "--retry-on-service" )?, "--retry-on-service" )?
      );
    }
    "--service-delay" =>
    {
      parsed.service_delay = Some(
        parse_u32_flag( next_value( tokens, next, "--service-delay" )?, "--service-delay", " (seconds)" )?
      );
    }
    "--retry-on-process" =>
    {
      parsed.retry_on_process = Some(
        parse_u8_bounded( next_value( tokens, next, "--retry-on-process" )?, "--retry-on-process" )?
      );
    }
    "--process-delay" =>
    {
      parsed.process_delay = Some(
        parse_u32_flag( next_value( tokens, next, "--process-delay" )?, "--process-delay", " (seconds)" )?
      );
    }
    "--retry-on-validation" =>
    {
      parsed.retry_on_validation = Some(
        parse_u8_bounded( next_value( tokens, next, "--retry-on-validation" )?, "--retry-on-validation" )?
      );
    }
    "--validation-delay" =>
    {
      parsed.validation_delay = Some(
        parse_u32_flag( next_value( tokens, next, "--validation-delay" )?, "--validation-delay", " (seconds)" )?
      );
    }
    "--retry-on-runner" =>
    {
      parsed.retry_on_runner = Some(
        parse_u8_bounded( next_value( tokens, next, "--retry-on-runner" )?, "--retry-on-runner" )?
      );
    }
    "--runner-delay" =>
    {
      parsed.runner_delay = Some(
        parse_u32_flag( next_value( tokens, next, "--runner-delay" )?, "--runner-delay", " (seconds)" )?
      );
    }
    "--retry-on-unknown" =>
    {
      parsed.retry_on_unknown = Some(
        parse_u8_bounded( next_value( tokens, next, "--retry-on-unknown" )?, "--retry-on-unknown" )?
      );
    }
    "--unknown-delay" =>
    {
      parsed.unknown_delay = Some(
        parse_u32_flag( next_value( tokens, next, "--unknown-delay" )?, "--unknown-delay", " (seconds)" )?
      );
    }
    "--retry-override" =>
    {
      parsed.retry_override = Some(
        parse_u8_bounded( next_value( tokens, next, "--retry-override" )?, "--retry-override" )?
      );
    }
    "--retry-override-delay" =>
    {
      parsed.retry_override_delay = Some(
        parse_u32_flag( next_value( tokens, next, "--retry-override-delay" )?, "--retry-override-delay", " (seconds)" )?
      );
    }
    "--retry-default" =>
    {
      parsed.retry_default = Some(
        parse_u8_bounded( next_value( tokens, next, "--retry-default" )?, "--retry-default" )?
      );
    }
    "--retry-default-delay" =>
    {
      parsed.retry_default_delay = Some(
        parse_u32_flag( next_value( tokens, next, "--retry-default-delay" )?, "--retry-default-delay", " (seconds)" )?
      );
    }
    "--output-style" =>
    {
      let v = next_value( tokens, next, "--output-style" )?;
      if !matches!( v, "summary" | "raw" )
      {
        return Err( Error::msg( format!(
          "invalid output-style '{v}' — expected: summary, raw"
        ) ) );
      }
      parsed.output_style = Some( v.to_string() );
    }
    "--summary-fields" =>
    {
      let v = next_value( tokens, next, "--summary-fields" )?;
      if let Err( bad ) = super::summary::resolve_fields( v )
      {
        if v.contains( ',' )
        {
          return Err( Error::msg( format!(
            "invalid summary-fields: unknown field '{bad}'"
          ) ) );
        }
        return Err( Error::msg( format!(
          "invalid summary-fields '{v}'"
        ) ) );
      }
      parsed.summary_fields = Some( v.to_string() );
    }
    "--journal" =>
    {
      let v = next_value( tokens, next, "--journal" )?;
      if !matches!( v, "full" | "meta" | "off" )
      {
        return Err( Error::msg( format!(
          "invalid --journal value '{v}' — expected: full, meta, off"
        ) ) );
      }
      parsed.journal = Some( v.to_string() );
    }
    "--journal-dir" =>
    {
      parsed.journal_dir = Some( next_value( tokens, next, "--journal-dir" )?.to_string() );
    }
    _ => return Ok( false ),
  }
  Ok( true )
}

/// Parse argv into structured CLI arguments.
///
/// Mirrors Claude Code's native `--flag value` syntax.
/// Positional (non-flag) arguments are joined with space to form the message.
///
/// `--help`/`-h` wins regardless of other flags or unknown tokens: if either appears
/// anywhere in `tokens`, parsing short-circuits and returns `CliArgs { help: true, .. }`.
#[ allow( clippy::too_many_lines ) ]
pub( crate ) fn parse_args( tokens : &[ String ] ) -> Result< CliArgs >
{
  // --help/-h always wins — return early before any other token is parsed.
  // This ensures help is shown even when unknown flags or other errors are present.
  // Fix(BUG-221): parse_args returned Err on the first unknown flag,
  // so main() never reached the cli.help check even when --help was present in argv.
  // Root cause: early Err return on unknown flags prevented the help check from firing.
  // Pitfall: checking cli.help after parse_args completes is insufficient — the Err path
  // in main() exits before any field of CliArgs is consulted.
  if tokens.iter().any( | t | t == "--help" || t == "-h" )
  {
    return Ok( CliArgs
    {
      help                 : true,
      message              : None,
      print_mode           : false,
      interactive          : false,
      new_session          : false,
      model                : None,
      verbose              : false,
      no_skip_permissions  : false,
      max_tokens           : None,
      session_dir          : None,
      dir                  : None,
      dry_run              : false,
      trace                : false,
      quiet                : false,
      system_prompt        : None,
      append_system_prompt : None,
      no_ultrathink        : false,
      effort               : None,
      no_effort_max        : false,
      no_chrome            : false,
      no_persist           : false,
      json_schema          : None,
      mcp_config           : Vec::new(),
      file                 : None,
      strip_fences         : false,
      keep_claudecode      : false,
      subdir               : None,
      output_file          : None,
      expect               : None,
      expect_strategy      : None,
      max_sessions            : None,
      retry_on_transient      : None,
      transient_delay         : None,
      timeout                 : None,
      retry_on_account        : None,
      account_delay           : None,
      retry_on_auth           : None,
      auth_delay              : None,
      retry_on_service        : None,
      service_delay           : None,
      retry_on_process        : None,
      process_delay           : None,
      retry_on_validation     : None,
      validation_delay        : None,
      retry_on_runner         : None,
      runner_delay            : None,
      retry_on_unknown        : None,
      unknown_delay           : None,
      retry_override          : None,
      retry_override_delay    : None,
      retry_default           : None,
      retry_default_delay     : None,
      output_format           : None,
      max_turns               : None,
      allowed_tools           : None,
      disallowed_tools        : None,
      max_budget_usd          : None,
      add_dir                 : None,
      fallback_model          : None,
      output_style            : None,
      summary_fields          : None,
      journal                 : None,
      journal_dir             : None,
      args_file               : None,
      no_compact_window       : false,
    } );
  }

  let mut parsed = CliArgs::default();
  let mut positional : Vec< String > = Vec::new();
  let mut i = 0;

  while i < tokens.len()
  {
    let token = tokens[ i ].as_str();
    match token
    {
      "-h" | "--help" =>
      {
        parsed.help = true;
      }
      "-p" | "--print" =>
      {
        parsed.print_mode = true;
      }
      "--interactive" =>
      {
        parsed.interactive = true;
      }
      "--new-session" =>
      {
        parsed.new_session = true;
      }
      "--verbose" =>
      {
        parsed.verbose = true;
      }
      "--no-skip-permissions" =>
      {
        parsed.no_skip_permissions = true;
      }
      "--dry-run" =>
      {
        parsed.dry_run = true;
      }
      "--trace" =>
      {
        parsed.trace = true;
      }
      "--no-ultrathink" =>
      {
        parsed.no_ultrathink = true;
      }
      "--no-effort-max" =>
      {
        parsed.no_effort_max = true;
      }
      "--no-chrome" =>
      {
        parsed.no_chrome = true;
      }
      "--no-persist" =>
      {
        parsed.no_persist = true;
      }
      "--strip-fences" =>
      {
        parsed.strip_fences = true;
      }
      "--keep-claudecode" =>
      {
        parsed.keep_claudecode = true;
      }
      "--no-compact-window" =>
      {
        parsed.no_compact_window = true;
      }
      "--quiet" =>
      {
        parsed.quiet = true;
      }
      "--" =>
      {
        // Everything after `--` is positional.
        // Fix(BUG-220): filter empty tokens here too — `clr -- ""`
        // must behave like bare `clr`, not forward a degenerate "\n\nultrathink" message.
        // Root cause: positional.extend() copies all tokens verbatim; the empty-token
        // guard in the `_` arm does not apply to the `--` code path.
        // Pitfall: filter at the individual-token level (not the joined string) so that
        // whitespace-only strings like " " are still valid messages and pass through.
        positional.extend( tokens[ i + 1 .. ].iter().filter( | t | !t.is_empty() ).cloned() );
        break;
      }
      s if s.starts_with( '-' ) =>
      {
        if parse_value_flag( s, tokens, i + 1, &mut parsed )?
        {
          i += 1; // advance past the consumed value token
        }
        else
        {
          return Err( Error::msg( format!( "unknown option: {s}\nRun with --help for usage." ) ) );
        }
      }
      _ =>
      {
        // Fix(BUG-219): skip empty tokens so `clr ""` behaves like
        // bare `clr` (no message, no --print, no degenerate "\n\nultrathink" forwarded).
        // Root cause: empty string was pushed to positional, joined to message=Some(""),
        // then the ultrathink suffix produced "\n\nultrathink" for an empty payload.
        // Pitfall: filter individual empty tokens, not the joined string — whitespace-only
        // strings like " " are valid non-empty messages and must not be filtered out.
        if !tokens[ i ].is_empty()
        {
          positional.push( tokens[ i ].clone() );
        }
      }
    }
    i += 1;
  }

  // Positional args form the message only when --message was not given explicitly.
  if !positional.is_empty() && parsed.message.is_none()
  {
    parsed.message = Some( positional.join( " " ) );
  }

  Ok( parsed )
}

