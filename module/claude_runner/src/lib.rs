//! `claude_runner` crate / `clr` binary — Claude Code CLI + command schema constants.
//!
//! This crate has two roles:
//!
//! 1. **Library** — exports [`COMMANDS_YAML`], the path to the `.claude` command schema,
//!    for use by YAML consumers at compile time or runtime.
//!
//! 2. **Binary** (`clr`) — Standalone CLI that mirrors Claude Code's native
//!    `--flag value` syntax and executes via `claude_runner_core`.
//!    Session continuation (`-c`) is applied by default; use `--new-session` to start fresh.
//!
//! ## Two roles, two consumers
//!
//! ```text
//! clr binary (standalone CLI)
//!   invoked directly: clr "Fix bug" --dir /path --model sonnet
//!     → parse_args() → ClaudeCommand builder → claude subprocess (with -c by default)
//!   message given → print mode (default); bare clr → interactive REPL
//!
//! YAML consumers (e.g. consumer workspace's CLI, build.rs)
//!   aggregate: claude_runner::COMMANDS_YAML → registers .claude + .claude.help in PHF map
//! ```
//!
//! This lib has **zero consumer workspace dependencies**. Without `enabled`, it is a pure constants +
//! types crate. With `enabled`, it also exposes [`register_commands`] (API consistency shim).
//!
//! ## Registering commands in other binaries
//!
//! **Build-time (PHF):** Point `build.rs` at [`COMMANDS_YAML`].
//!
//! **Runtime:** Use `MultiYamlAggregator` with [`COMMANDS_YAML`].

pub mod verbosity;
pub use verbosity::VerbosityLevel;

/// Absolute path to this crate's command definitions YAML.
///
/// Use in `build.rs` for compile-time aggregation or at runtime for dynamic registration.
pub const COMMANDS_YAML : &str = concat!( env!( "CARGO_MANIFEST_DIR" ), "/claude.commands.yaml" );

#[ cfg( feature = "enabled" ) ]
/// Register `claude_runner` commands into an existing registry.
///
/// `claude_runner` commands are defined in [`COMMANDS_YAML`] for compile-time aggregation
/// (used by `assistant/build.rs`). This function is provided for API consistency with
/// other Layer 2 crates; the body is intentionally empty because runtime registration of
/// `.claude` commands is handled by the build-time YAML aggregation path in `assistant`.
#[ inline ]
pub fn register_commands( _registry : &mut unilang::registry::CommandRegistry ) {}

#[ cfg( feature = "enabled" ) ]
mod cli
{
  use super::VerbosityLevel;
  use claude_runner_core::{ ClaudeCommand, EffortLevel, RunnerError, run_isolated };
  use error_tools::{ Error, Result };

  /// Parsed CLI arguments.
  #[ allow( clippy::struct_excessive_bools ) ]
  #[ derive( Default ) ]
  pub( super ) struct CliArgs
  {
    pub( super ) message              : Option< String >,
    pub( super ) print_mode           : bool,
    pub( super ) interactive          : bool,
    pub( super ) new_session          : bool,
    pub( super ) model                : Option< String >,
    pub( super ) verbose              : bool,
    pub( super ) no_skip_permissions  : bool,
    pub( super ) max_tokens           : Option< u32 >,
    pub( super ) session_dir          : Option< String >,
    pub( super ) dir                  : Option< String >,
    pub( super ) dry_run              : bool,
    pub( super ) trace                : bool,
    pub( super ) verbosity            : VerbosityLevel,
    pub( super ) help                 : bool,
    pub( super ) system_prompt        : Option< String >,
    pub( super ) append_system_prompt : Option< String >,
    pub( super ) no_ultrathink        : bool,
    pub( super ) effort               : Option< EffortLevel >,
    pub( super ) no_effort_max        : bool,
    pub( super ) no_chrome            : bool,
    pub( super ) no_persist           : bool,
    pub( super ) json_schema          : Option< String >,
    pub( super ) mcp_config           : Vec< String >,
  }

  /// Parsed arguments for the `isolated` subcommand.
  #[ derive( Default ) ]
  pub( super ) struct IsolatedArgs
  {
    pub( super ) creds_path       : String,
    pub( super ) timeout_secs     : u64,
    pub( super ) message          : Option< String >,
    pub( super ) passthrough_args : Vec< String >,
  }

  pub( super ) fn print_help()
  {
    println!( "clr — Execute Claude Code with configurable parameters" );
    println!();
    println!( "USAGE:" );
    println!( "  clr [OPTIONS] [MESSAGE]" );
    println!( "  clr isolated --creds <FILE> [--timeout <SECS>] [MESSAGE]" );
    println!();
    println!( "COMMANDS:" );
    println!( "  isolated                           Run Claude with credential-isolated temp HOME" );
    println!();
    println!( "ARGUMENTS:" );
    println!( "  [MESSAGE]                          Prompt message for Claude" );
    println!();
    println!( "OPTIONS:" );
    println!( "  -p, --print                        Non-interactive mode (capture and print output)" );
    println!( "  --interactive                      Force interactive mode even when a message is given" );
    println!( "  --new-session                      Start a new session (default: continues previous)" );
    println!( "  --model <MODEL>                    Model to use" );
    println!( "  --verbose                          Enable verbose output" );
    println!( "  --no-skip-permissions              Disable automatic permission bypass (on by default)" );
    println!( "  --max-tokens <N>                   Max output tokens (default: 200000)" );
    println!( "  --session-dir <PATH>               Session storage directory" );
    println!( "  --dir <PATH>                       Working directory" );
    println!( "  --dry-run                          Print command without executing" );
    println!( "  --trace                            Print command to stderr then execute (like set -x)" );
    println!( "  --system-prompt <TEXT>             Set system prompt (replaces the default)" );
    println!( "  --append-system-prompt <TEXT>      Append text to the default system prompt" );
    println!( "  --no-ultrathink                    Disable automatic \"\\n\\nultrathink\" message suffix" );
    println!( "  --effort <LEVEL>                   Reasoning effort: low, medium, high, max (default: max)" );
    println!( "  --no-effort-max                    Suppress default --effort max injection" );
    println!( "  --no-chrome                        Suppress default --chrome injection" );
    println!( "  --no-persist                       Disable session persistence (--no-session-persistence)" );
    println!( "  --json-schema <SCHEMA>             JSON schema for structured output" );
    println!( "  --mcp-config <PATH>                MCP server config file (repeatable)" );
    println!( "  --verbosity <0-5>                  Runner output verbosity level (default: 3)" );
    println!( "  -h, --help                         Show this help" );
    println!();
    println!( "ISOLATED OPTIONS:" );
    println!( "  --creds <FILE>                     Credentials JSON file (required for isolated)" );
    println!( "  --timeout <SECS>                   Max seconds to wait for subprocess (default: 30)" );
  }

  /// Consume the next argv element as a flag's value.
  fn next_value<'a>( tokens : &'a [ String ], idx : usize, flag : &str ) -> Result< &'a str >
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

  /// Parse a raw string as a `u64` timeout in seconds.
  ///
  /// Rejects negative numbers (which start with `-` and fail `u64` parsing)
  /// and non-numeric strings with a clear error message.
  fn parse_timeout( raw : &str ) -> Result< u64 >
  {
    raw.parse::< u64 >().map_err( | _ |
      Error::msg( format!(
        "invalid --timeout value: {raw}\n\
         Expected non-negative integer"
      ) )
    )
  }

  /// Print help for the `isolated` subcommand and exit 0.
  ///
  /// Called when `parse_isolated_args` encounters `-h` or `--help`.
  /// Terminates the process via `std::process::exit(0)` so the caller
  /// never needs to handle a return value.
  fn print_isolated_help() -> !
  {
    println!( "clr isolated — Run Claude Code with credential-isolated temp HOME" );
    println!();
    println!( "USAGE:" );
    println!( "  clr isolated --creds <FILE> [--timeout <SECS>] [MESSAGE] [-- PASSTHROUGH...]" );
    println!();
    println!( "ARGUMENTS:" );
    println!( "  [MESSAGE]                          Prompt message for Claude" );
    println!();
    println!( "ISOLATED OPTIONS:" );
    println!( "  --creds <FILE>                     Credentials JSON file (required)" );
    println!( "  --timeout <SECS>                   Max seconds to wait for subprocess (default: 30)" );
    println!( "  -h, --help                         Show this help" );
    println!();
    println!( "EXIT CODES:" );
    println!( "  0    Success" );
    println!( "  1    Error (bad arguments, subprocess failure)" );
    std::process::exit( 0 );
  }

  /// Parse `tokens` as arguments to the `isolated` subcommand.
  ///
  /// Recognises `--creds <FILE>`, `--timeout <SECS>`, a positional `[MESSAGE]`,
  /// and `-- <PASSTHROUGH...>`. Everything after `--` is collected verbatim.
  /// Unknown flags (before `--`) are rejected with an error.
  pub( super ) fn parse_isolated_args( tokens : &[ String ] ) -> Result< IsolatedArgs >
  {
    let mut creds_path       : Option< String > = None;
    let mut timeout_secs     : u64              = 30;
    let mut message_parts    : Vec< String >    = Vec::new();
    let mut passthrough_args : Vec< String >    = Vec::new();
    let mut i = 0;
    while i < tokens.len()
    {
      let token = tokens[ i ].as_str();
      match token
      {
        "--" =>
        {
          passthrough_args.extend( tokens[ i + 1 .. ].iter().cloned() );
          break;
        }
        "--creds" =>
        {
          creds_path = Some( next_value( tokens, i + 1, "--creds" )?.to_string() );
          i += 1;
        }
        "--timeout" =>
        {
          let raw      = next_value( tokens, i + 1, "--timeout" )?;
          timeout_secs = parse_timeout( raw )?;
          i += 1;
        }
        // Fix(issue-isolated-help): parse_isolated_args fell through --help to the
        // starts_with('-') catch-all, returning Err("unknown option: --help") → exit 1.
        // Root cause: no explicit --help arm in parse_isolated_args; global parse_args has
        // one but parse_isolated_args was written without it.
        // Pitfall: any catch-all for unknown flags silently swallows --help and -h;
        // always add an explicit --help arm before the catch-all in every subcommand parser.
        "-h" | "--help" => { print_isolated_help(); }
        s if s.starts_with( '-' ) =>
        {
          return Err( Error::msg( format!(
            "unknown option: {s}\nRun with --help for usage."
          ) ) );
        }
        _ =>
        {
          if !tokens[ i ].is_empty() { message_parts.push( tokens[ i ].clone() ); }
        }
      }
      i += 1;
    }
    // Note: creds_path validation is deferred to after apply_isolated_env_vars() is called
    // in run_cli() so that CLR_CREDS env var can supply the value before the check.
    let creds_path   = creds_path.unwrap_or_default();
    let message      = if message_parts.is_empty() { None } else { Some( message_parts.join( " " ) ) };
    Ok( IsolatedArgs { creds_path, timeout_secs, message, passthrough_args } )
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
      "--verbosity" =>
      {
        let raw = next_value( tokens, next, "--verbosity" )?;
        parsed.verbosity = raw.parse::< VerbosityLevel >().map_err( Error::msg )?;
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
  pub( super ) fn parse_args( tokens : &[ String ] ) -> Result< CliArgs >
  {
    // --help/-h always wins — return early before any other token is parsed.
    // This ensures help is shown even when unknown flags or other errors are present.
    // Fix(issue-help-loses-to-unknown): parse_args returned Err on the first unknown flag,
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
        "--" =>
        {
          // Everything after `--` is positional.
          // Fix(issue-empty-msg-double-dash): filter empty tokens here too — `clr -- ""`
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
          // Fix(issue-empty-msg-ultrathink): skip empty tokens so `clr ""` behaves like
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
  fn env_bool( var : &str ) -> bool
  {
    std::env::var( var ).ok()
      .is_some_and( | v | matches!( v.to_lowercase().as_str(), "1" | "true" ) )
  }

  /// Returns `Some(value)` if `var` is set to a non-empty string; `None` otherwise.
  fn env_str( var : &str ) -> Option< String >
  {
    std::env::var( var ).ok().filter( | v | !v.is_empty() )
  }

  /// Apply `CLR_*` environment variable fallbacks for the 22 run parameters.
  ///
  /// Each field is updated only when it is still at its zero/default value — the CLI
  /// flag always wins when both are present (CLI-wins field-default check).
  pub( super ) fn apply_env_vars( parsed : &mut CliArgs )
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
    if parsed.verbosity == VerbosityLevel::default()
    {
      if let Some( v ) = env_str( "CLR_VERBOSITY" )
      {
        if let Ok( level ) = v.parse::< VerbosityLevel >() { parsed.verbosity = level; }
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
  }

  /// Apply `CLR_CREDS` and `CLR_TIMEOUT` env var fallbacks for the `isolated` subcommand.
  ///
  /// `CLR_CREDS` applies when `creds_path` is empty (no `--creds` on CLI).
  /// `CLR_TIMEOUT` applies when `timeout_secs == 30` (the default); explicit `--timeout 30`
  /// is indistinguishable from the default and is an accepted limitation.
  pub( super ) fn apply_isolated_env_vars( parsed : &mut IsolatedArgs )
  {
    if parsed.creds_path.is_empty()
    {
      parsed.creds_path = env_str( "CLR_CREDS" ).unwrap_or_default();
    }
    if parsed.timeout_secs == 30
    {
      if let Some( v ) = env_str( "CLR_TIMEOUT" )
      {
        if let Ok( secs ) = v.parse::< u64 >() { parsed.timeout_secs = secs; }
      }
    }
  }

  /// Translate parsed CLI args into a `ClaudeCommand` builder.
  ///
  /// Session continuation (`-c`) is applied by default unless `--new-session` is set.
  pub( super ) fn build_claude_command( cli : &CliArgs ) -> ClaudeCommand
  {
    let mut builder = ClaudeCommand::new();

    if let Some( ref dir ) = cli.dir
    {
      builder = builder.with_working_directory( dir.clone() );
    }
    if let Some( n ) = cli.max_tokens
    {
      builder = builder.with_max_output_tokens( n );
    }
    if !cli.new_session
    {
      builder = builder.with_continue_conversation( true );
    }
    if !cli.no_skip_permissions
    {
      builder = builder.with_skip_permissions( true );
    }
    if !cli.no_effort_max
    {
      builder = builder.with_effort(
        cli.effort.unwrap_or( EffortLevel::Max )
      );
    }
    if cli.no_chrome
    {
      builder = builder.with_chrome( None );
    }
    if cli.no_persist
    {
      builder = builder.with_no_session_persistence( true );
    }
    if let Some( ref schema ) = cli.json_schema
    {
      builder = builder.with_json_schema( schema.as_str() );
    }
    if !cli.mcp_config.is_empty()
    {
      builder = builder.with_mcp_config( cli.mcp_config.iter().map( String::as_str ) );
    }
    if cli.verbose
    {
      builder = builder.with_verbose( true );
    }
    if let Some( ref model ) = cli.model
    {
      builder = builder.with_model( model.clone() );
    }
    if let Some( ref sd ) = cli.session_dir
    {
      builder = builder.with_session_dir( sd.clone() );
    }
    if let Some( ref sp ) = cli.system_prompt
    {
      builder = builder.with_system_prompt( sp.clone() );
    }
    if let Some( ref asp ) = cli.append_system_prompt
    {
      builder = builder.with_append_system_prompt( asp.clone() );
    }
    // Auto-add --print when a message is given and interactive mode is not explicitly requested.
    // Fix(issue-default-print): message without -p was silently using TTY passthrough,
    // producing raw TUI escape codes instead of clean text output in scripted contexts.
    // Root cause: print mode was only enabled by explicit -p/--print; no auto-detection.
    // Pitfall: `--interactive` must suppress this auto-addition to allow prompted REPL sessions.
    let use_print = cli.print_mode || ( cli.message.is_some() && !cli.interactive );
    if use_print
    {
      builder = builder.with_arg( "--print" );
    }
    if let Some( ref msg ) = cli.message
    {
      // Fix(issue-ultrathink-suffix): inject as suffix not prefix so the user task
      //   comes first in Claude's context window — earlier tokens carry more weight.
      // Root cause: original format!("ultrathink {msg}") buried the task description
      //   under the directive; suffix form preserves natural "state task, then direct thinking"
      //   order that matches Claude's conversational expectations.
      // Pitfall: idempotent guard must use trim_end().ends_with not starts_with —
      //   suffix anchors at the end; starts_with would miss re-injection on existing suffixes.
      let effective_msg = if cli.no_ultrathink || msg.trim_end().ends_with( "ultrathink" )
      {
        msg.clone()
      }
      else
      {
        format!( "{msg}\n\nultrathink" )
      };
      builder = builder.with_message( effective_msg );
    }

    builder
  }

  /// Handle dry-run mode: print command preview and exit.
  ///
  /// Always emits output regardless of verbosity level. Verbosity controls runner
  /// diagnostics only; `--dry-run` output is core functionality the user explicitly requested.
  // Fix(issue-dry-run-verbosity-gate): always emit; verbosity must not suppress --dry-run output
  // Root cause: prior version gated on shows_progress() (≥3); --verbosity 0–2 produced silent exit
  // Pitfall: Verbosity gates runner diagnostics only, never core feature output like --dry-run
  pub( super ) fn handle_dry_run( builder : &ClaudeCommand )
  {
    let env = builder.describe_env();
    let command = builder.describe();
    if !env.is_empty() { println!( "{env}" ); }
    println!( "{command}" );
  }

  /// Execute in non-interactive print mode (captures output).
  ///
  /// Both `--print` (passed to claude) and `execute()` (captures stdout) are required:
  /// `--print` tells claude to run single-shot with clean text output (no TUI);
  /// `execute()` captures that output into memory for programmatic use.
  /// Without `--print`, captured output would be TUI escape codes.
  /// Without `execute()`, clean output would go straight to terminal uncaptured.
  pub( super ) fn run_print_mode( builder : &ClaudeCommand, verbosity : VerbosityLevel )
  {
    let output = match builder.execute()
    {
      Ok( o )  => o,
      Err( e ) =>
      {
        if verbosity.shows_errors()
        {
          eprintln!( "Error: {e}" );
        }
        std::process::exit( 1 );
      }
    };

    if !output.stderr.is_empty() { eprint!( "{}", output.stderr ); }

    if output.exit_code != 0
    {
      if verbosity.shows_errors()
      {
        eprintln!( "Error: Claude exited with code {}", output.exit_code );
      }
      std::process::exit( 1 );
    }

    print!( "{}", output.stdout );
  }

  /// Execute in interactive mode (TTY passthrough).
  pub( super ) fn run_interactive( builder : &ClaudeCommand, verbosity : VerbosityLevel )
  {
    let status = match builder.execute_interactive()
    {
      Ok( s )  => s,
      Err( e ) =>
      {
        if verbosity.shows_errors()
        {
          eprintln!( "Error: {e}" );
        }
        std::process::exit( 1 );
      }
    };

    if !status.success()
    {
      std::process::exit( status.code().unwrap_or( 1 ) );
    }
  }

  /// Execute the `isolated` subcommand.
  ///
  /// Reads the credentials file at `creds_path`, builds the argument list for
  /// `run_isolated`, then handles the result:
  ///
  /// - **Success (`exit_code >= 0`):** propagates the subprocess exit code.
  /// - **Success (`exit_code == -1`, creds refreshed at startup before timeout):**
  ///   writes back updated credentials and exits 0.
  /// - **`Err(Timeout)`:** subprocess exceeded the deadline without refreshing
  ///   credentials — exits 2.
  /// - **Other errors:** exits 1 with an error message.
  ///
  /// This function never returns; it always calls `std::process::exit`.
  pub( super ) fn run_isolated_command
  (
    creds_path       : &str,
    timeout_secs     : u64,
    message          : Option< &str >,
    passthrough_args : &[ String ],
  ) -> !
  {
    let creds_json = match std::fs::read_to_string( creds_path )
    {
      Ok( s )  => s,
      Err( e ) =>
      {
        eprintln!( "Error: cannot read credentials file '{creds_path}': {e}" );
        std::process::exit( 1 );
      }
    };
    // Build the args to forward: --print + message when a message is given,
    // then any passthrough args supplied after `--`.
    let mut args : Vec< String > = message
      .map( | m | vec![ "--print".to_string(), m.to_string() ] )
      .unwrap_or_default();
    args.extend_from_slice( passthrough_args );
    match run_isolated( &creds_json, args, timeout_secs )
    {
      Ok( result ) =>
      {
        // Write back refreshed credentials if Claude updated them before
        // the subprocess finished (or before the timeout killed it).
        if let Some( ref new_creds ) = result.credentials
        {
          if let Err( e ) = std::fs::write( creds_path, new_creds )
          {
            eprintln!( "Warning: could not write back refreshed credentials to '{creds_path}': {e}" );
          }
        }
        if !result.stderr.is_empty() { eprint!( "{}", result.stderr ); }
        if !result.stdout.is_empty() { print!( "{}", result.stdout ); }
        // exit_code == -1: subprocess was killed by timeout BUT credentials were
        // refreshed before the kill — per spec, exit 0 and write-back already done.
        let exit_code = if result.exit_code == -1 { 0 } else { result.exit_code };
        std::process::exit( exit_code );
      }
      Err( RunnerError::Timeout { secs } ) =>
      {
        eprintln!( "Error: isolated subprocess timed out after {secs} seconds" );
        std::process::exit( 2 );
      }
      Err( e ) =>
      {
        eprintln!( "Error: {e}" );
        std::process::exit( 1 );
      }
    }
  }
}

#[ cfg( feature = "enabled" ) ]
/// Run the `clr`/`claude_runner` CLI.
///
/// Entry point shared by the `clr` and `claude_runner` binary targets.
#[ inline ]
pub fn run_cli()
{
  use cli::{
    parse_args, parse_isolated_args, build_claude_command, handle_dry_run,
    print_help, run_print_mode, run_interactive, run_isolated_command,
    apply_env_vars, apply_isolated_env_vars,
  };

  let tokens : Vec< String > = std::env::args().skip( 1 ).collect();

  // Dispatch `isolated` subcommand before the standard flag parser.
  if tokens.first().map( String::as_str ) == Some( "isolated" )
  {
    let mut isolated_cli = match parse_isolated_args( &tokens[ 1 .. ] )
    {
      Ok( c )  => c,
      Err( e ) =>
      {
        eprintln!( "Error: {e}" );
        std::process::exit( 1 );
      }
    };
    apply_isolated_env_vars( &mut isolated_cli );
    if isolated_cli.creds_path.is_empty()
    {
      eprintln!( "Error: missing required argument: --creds\nRun with --help for usage." );
      std::process::exit( 1 );
    }
    run_isolated_command(
      &isolated_cli.creds_path,
      isolated_cli.timeout_secs,
      isolated_cli.message.as_deref(),
      &isolated_cli.passthrough_args,
    );
  }

  let mut cli = match parse_args( &tokens )
  {
    Ok( c )  => c,
    Err( e ) =>
    {
      eprintln!( "Error: {e}" );
      std::process::exit( 1 );
    }
  };
  apply_env_vars( &mut cli );

  if cli.help
  {
    print_help();
    return;
  }

  if cli.print_mode && cli.message.is_none()
  {
    eprintln!( "Error: --print requires a message argument" );
    eprintln!( "Run with --help for usage." );
    std::process::exit( 1 );
  }

  let builder = build_claude_command( &cli );

  if cli.dry_run
  {
    handle_dry_run( &builder );
    return;
  }

  // Trace / verbose detail preview: show command to stderr before executing.
  if cli.trace || cli.verbosity.shows_verbose_detail()
  {
    let env = builder.describe_env();
    let command = builder.describe();
    let mut preview = String::new();
    if !env.is_empty() { preview.push_str( &env ); preview.push( '\n' ); }
    preview.push_str( &command );
    eprintln!( "{preview}" );
  }

  // Dispatch to print mode when message given (default) or -p/--print explicit.
  // --interactive overrides the message-default back to TTY passthrough.
  if cli.print_mode || ( cli.message.is_some() && !cli.interactive )
  {
    run_print_mode( &builder, cli.verbosity );
  }
  else
  {
    run_interactive( &builder, cli.verbosity );
  }
}
