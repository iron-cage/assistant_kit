use crate::VerbosityLevel;
use claude_runner_core::EffortLevel;
use error_tools::{ Error, Result };

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
  pub( crate ) verbosity            : Option< VerbosityLevel >,
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

/// Parse a value-consuming flag (`--flag value` pair) into `parsed`.
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
    "--verbosity" =>
    {
      let raw = next_value( tokens, next, "--verbosity" )?;
      parsed.verbosity = Some( raw.parse::< VerbosityLevel >().map_err( Error::msg )? );
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
    return Ok( CliArgs { help : true, ..CliArgs::default() } );
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

  if !positional.is_empty()
  {
    parsed.message = Some( positional.join( " " ) );
  }

  Ok( parsed )
}

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

/// Apply `CLR_*` environment variable fallbacks for the 28 run parameters.
///
/// Each field is updated only when it is still at its zero/default value — the CLI
/// flag always wins when both are present (CLI-wins field-default check).
pub( crate ) fn apply_env_vars( parsed : &mut CliArgs )
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
  // the default (3). Root cause: `verbosity` was `VerbosityLevel` (non-optional), so the guard
  // `parsed.verbosity == VerbosityLevel::default()` misfired for `--verbosity 3` — the value is
  // identical to the unset default, making them indistinguishable.
  // Root cause: non-optional field whose default is non-zero/non-false cannot act as a "set" sentinel.
  // Pitfall: use `Option<T>` (never `T == default()`) for any env-var-fallback field whose default
  // is a non-false/non-zero value; equality-with-default is always ambiguous as a set-sentinel.
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
}

