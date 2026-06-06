mod parse;
mod cred_parse;
mod builder;
mod fence;
mod credential;

use super::VerbosityLevel;
use claude_runner_core::{ ClaudeCommand, ErrorKind, IsolatedModel, signal_exit_code };
use parse::{ CliArgs, ExpectStrategy };
use cred_parse::{
  parse_isolated_args, parse_refresh_args,
  apply_isolated_env_vars, apply_refresh_env_vars,
};
pub use fence::strip_fences;
use credential::{ run_isolated_command, run_refresh_command };

pub( super ) use parse::{ parse_args, apply_env_vars };
pub( super ) use builder::build_claude_command;

pub( super ) fn print_help()
{
  println!( "clr — Execute Claude Code with configurable parameters" );
  println!();
  println!( "USAGE:" );
  println!( "  clr [OPTIONS] [MESSAGE]" );
  println!( "  clr run      [OPTIONS] [MESSAGE]" );
  println!( "  clr ask      [OPTIONS] [QUESTION]" );
  println!( "  clr isolated --creds <FILE> [--timeout <SECS>] [--trace] [MESSAGE]" );
  println!( "  clr refresh  --creds <FILE> [--timeout <SECS>] [--trace]" );
  println!( "  clr help" );
  println!();
  println!( "COMMANDS:" );
  // Fix(BUG-212): `run` was absent from COMMANDS despite being a valid explicit subcommand.
  // Root cause: print_help() only listed ask/isolated/refresh/help; discoverability AC violated.
  // Pitfall: `clr run` must strip the leading token before reaching the parser — see lib.rs.
  println!( "  run                                Execute Claude Code with configurable parameters (default mode)" );
  println!( "  ask                                Semantic alias for `run` (identical behavior)" );
  println!( "  isolated                           Run Claude with credential-isolated temp HOME" );
  println!( "  refresh                            Refresh OAuth credentials without running a task" );
  println!( "  help                               Print usage information and exit" );
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
  println!( "  --subdir <NAME>                    Named subdirectory appended to --dir as /-NAME; . = identity" );
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
  println!( "  --file <PATH>                      Pipe file content to subprocess stdin" );
  println!( "  --strip-fences                     Strip outermost markdown code fences from stdout" );
  println!( "  --keep-claudecode                  Preserve CLAUDECODE env var in subprocess (default: removed)" );
  println!( "  --verbosity <0-5>                  Runner output verbosity level (default: 3)" );
  println!( "  --output-file <PATH>               Write captured output to file (tee: stdout + file)" );
  println!( "  --expect <VALS>                    Pipe-separated expected values; mismatch → exit 3 (case-insensitive, trimmed)" );
  println!( "  --expect-strategy <STRAT>          Mismatch handling: fail (default), retry, default:<VAL>" );
  println!( "  --expect-retries <N>               Retry attempts when --expect-strategy retry (0–255, default: 0)" );
  println!( "  --max-sessions <N>                 Max concurrent claude sessions before blocking (0=unlimited, default: 10)" );
  println!( "  -h, --help                         Show this help" );
  println!();
  println!( "CREDENTIAL OPTIONS (isolated, refresh):" );
  println!( "  --creds <FILE>                     Credentials JSON file (required)" );
  println!( "  --timeout <SECS>                   Max seconds to wait (default: 30 isolated, 45 refresh)" );
  println!( "  --trace                            Print creds path, timeout, and claude invocation to stderr" );
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
  println!( "CREDENTIAL OPTIONS:" );
  println!( "  --creds <FILE>                     Credentials JSON file (required)" );
  println!( "  --timeout <SECS>                   Max seconds to wait for subprocess (default: 30)" );
  println!( "  --trace                            Print underlying call details to stderr" );
  println!( "  -h, --help                         Show this help" );
  println!();
  println!( "EXIT CODES:" );
  println!( "  0    Success" );
  println!( "  1    Error (bad arguments, subprocess failure)" );
  println!( "  2    Timeout — subprocess did not finish within --timeout seconds" );
  std::process::exit( 0 );
}

/// Print help for the `refresh` subcommand and exit 0.
fn print_refresh_help() -> !
{
  println!( "clr refresh — Refresh OAuth credentials without running a task" );
  println!();
  println!( "USAGE:" );
  println!( "  clr refresh --creds <FILE> [--timeout <SECS>] [--trace]" );
  println!();
  println!( "CREDENTIAL OPTIONS:" );
  println!( "  --creds <FILE>                     Credentials JSON file (required)" );
  println!( "  --timeout <SECS>                   Max seconds to wait for refresh (default: 45)" );
  println!( "  --trace                            Print underlying call details to stderr" );
  println!( "  -h, --help                         Show this help" );
  println!();
  println!( "EXIT CODES:" );
  println!( "  0    Credentials were refreshed and written back" );
  println!( "  1    Error (bad arguments, no refresh occurred, subprocess failure)" );
  println!( "  2    Timeout — subprocess did not finish within --timeout seconds" );
  std::process::exit( 0 );
}

/// Print help for the `ask` subcommand and exit 0.
///
/// Called when `--help` or `-h` is detected in `dispatch_ask` before delegating to `dispatch_run`.
fn print_ask_help() -> !
{
  println!( "clr ask — Semantic alias for `clr run`" );
  println!();
  println!( "USAGE:" );
  println!( "  clr ask [OPTIONS] [QUESTION]" );
  println!();
  println!( "ARGUMENTS:" );
  println!( "  [QUESTION]                         Question to ask Claude" );
  println!();
  println!( "`clr ask` is a pure semantic alias for `clr run` — all options are identical." );
  println!( "See `clr --help` or `clr run --help` for the full option list." );
  println!();
  println!( "OPTIONS:" );
  println!( "  -p, --print                        Non-interactive mode (capture and print output)" );
  println!( "  --effort <LEVEL>                   Reasoning effort: low, medium, high, max (default: max)" );
  println!( "  --max-tokens <N>                   Max output tokens (default: 200000)" );
  println!( "  --model <MODEL>                    Model to use" );
  println!( "  --dry-run                          Print command without executing" );
  println!( "  --trace                            Print command to stderr then execute" );
  println!( "  --system-prompt <TEXT>             Set system prompt" );
  println!( "  --append-system-prompt <TEXT>      Append to default system prompt" );
  println!( "  --dir <PATH>                       Working directory" );
  println!( "  --subdir <NAME>                    Named subdirectory appended to --dir as /-NAME; . = identity" );
  println!( "  --session-dir <PATH>               Session storage directory" );
  println!( "  --verbosity <0-5>                  Runner output verbosity level" );
  println!( "  --json-schema <SCHEMA>             JSON schema for structured output" );
  println!( "  --mcp-config <PATH>                MCP server config file (repeatable)" );
  println!( "  --file <PATH>                      Pipe file content to subprocess stdin" );
  println!( "  --strip-fences                     Strip outermost markdown code fences" );
  println!( "  --output-file <PATH>               Write captured output to file (tee: stdout + file)" );
  println!( "  --expect <VALS>                    Pipe-separated expected values; mismatch → exit 3 (case-insensitive, trimmed)" );
  println!( "  --expect-strategy <STRAT>          Mismatch handling: fail (default), retry, default:<VAL>" );
  println!( "  --expect-retries <N>               Retry attempts when --expect-strategy retry (0–255, default: 0)" );
  println!( "  -h, --help                         Show this help" );
  std::process::exit( 0 );
}

/// Handle dry-run mode: print command preview and exit.
///
/// Always emits output regardless of verbosity level. Verbosity controls runner
/// diagnostics only; `--dry-run` output is core functionality the user explicitly requested.
// Fix(BUG-228): always emit; verbosity must not suppress --dry-run output
// Root cause: prior version gated on shows_progress() (≥3); --verbosity 0–2 produced silent exit
// Pitfall: Verbosity gates runner diagnostics only, never core feature output like --dry-run
pub( super ) fn handle_dry_run( builder : &ClaudeCommand )
{
  let env = builder.describe_env();
  let command = builder.describe();
  if !env.is_empty() { println!( "{env}" ); }
  println!( "{command}" );
}

// Fix(BUG-225): Guard against typos/truncations of known subcommand names.
// Root cause: `run_cli()` dispatched subcommands by exact string match only — any
//   non-matching first token silently fell through to `parse_args()`.
// Pitfall: Bare string comparison only guards exact matches; typos pass silently
//   unless a prefix-match guard is also placed before the main argument parser.
pub( super ) fn guard_unknown_subcommand( tokens : &[ String ] )
{
  // Fix(BUG-212): `run` was absent from KNOWN; typing `clr running` produced no helpful error.
  // Root cause: KNOWN list was never updated when `run` became an explicit subcommand.
  // Pitfall: `clr run` (len=3) bypasses is_identifier guard (requires len>=4), so it reaches
  //   the run_cli dispatch before guard is called — that is correct and expected.
  const KNOWN : &[ &str ] = &[ "run", "ask", "isolated", "refresh", "help" ];
  if let Some( first ) = tokens.first()
  {
    let is_identifier = !first.starts_with( '-' )
      && first.len() >= 4
      && first.chars().all( | c | c.is_alphanumeric() || c == '_' || c == '-' );
    if is_identifier
    {
      for &sub in KNOWN
      {
        if first != sub
          && ( sub.starts_with( first.as_str() ) || first.starts_with( sub ) )
        {
          eprintln!(
            "Error: unknown subcommand: {first}. Did you mean '{sub}'?\nRun with --help for usage."
          );
          std::process::exit( 1 );
        }
      }
    }
  }
}

/// Emit trace preview then dispatch to print mode or interactive mode.
///
/// Called after a command has been built and the dry-run branch has been handled.
/// Dispatches to `run_print_mode` when a message is given (or `-p` is set),
/// and to `run_interactive` otherwise.
/// Count running `claude` binary processes by scanning `/proc/*/cmdline`.
///
/// Reads the NUL-delimited argv from each numeric `/proc/<pid>/cmdline` entry and
/// checks whether the first argument's file-name component equals `"claude"`.
/// Returns 0 on any I/O failure so the gate degrades gracefully.
fn count_claude_sessions() -> usize
{
  let Ok( entries ) = std::fs::read_dir( "/proc" ) else { return 0; };
  entries
    .flatten()
    .filter( | e | e.file_name().to_string_lossy().chars().all( | c | c.is_ascii_digit() ) )
    .filter( | e |
    {
      let cmdline_path = e.path().join( "cmdline" );
      std::fs::read( cmdline_path )
        .is_ok_and( | bytes |
        {
          // cmdline is NUL-separated argv; first arg is the binary path.
          let first     = bytes.split( | &c | c == 0 ).next().unwrap_or_default();
          let first_str = String::from_utf8_lossy( first );
          std::path::Path::new( first_str.as_ref() )
            .file_name()
            .is_some_and( | n | n == "claude" )
        } )
    } )
    .count()
}

/// Block until fewer than `max` `claude` sessions are running, or until the 15-minute
/// timeout elapses.  `max == 0` means unlimited — returns immediately without checking.
fn wait_for_session_slot( max : u32, verbosity : VerbosityLevel )
{
  if max == 0 { return; }
  let timeout = core::time::Duration::from_secs( 15 * 60 );
  let poll    = core::time::Duration::from_secs( 30 );
  let start   = std::time::Instant::now();
  loop
  {
    let count = count_claude_sessions();
    if u32::try_from( count ).unwrap_or( u32::MAX ) < max { return; }
    if start.elapsed() >= timeout
    {
      eprintln!(
        "Error: --max-sessions limit ({max}) reached; timed out after 15 minutes waiting for a slot."
      );
      std::process::exit( 1 );
    }
    if verbosity.shows_warnings()
    {
      eprintln!( "Info: {count} claude session(s) running (limit {max}); waiting 30s for a free slot..." );
    }
    std::thread::sleep( poll );
  }
}

pub( super ) fn run_built_command( builder : &ClaudeCommand, cli : &CliArgs )
{
  let verbosity = cli.verbosity.unwrap_or_default();

  // Concurrency gate: block before subprocess launch when max active claude sessions is reached.
  // Default limit is 10; 0 = unlimited.  dry-run is bypassed by caller (never reaches here).
  let max_sessions = cli.max_sessions.unwrap_or( 10 );
  wait_for_session_slot( max_sessions, verbosity );

  if cli.trace || verbosity.shows_verbose_detail()
  {
    let env     = builder.describe_env();
    let command = builder.describe();
    let mut preview = String::new();
    if !env.is_empty() { preview.push_str( &env ); preview.push( '\n' ); }
    preview.push_str( &command );
    eprintln!( "{preview}" );
  }

  if cli.print_mode || ( cli.message.is_some() && !cli.interactive )
  {
    run_print_mode( builder, cli );
  }
  else
  {
    run_interactive( builder, verbosity );
  }
}

/// Write `content` to the output file at `path` if present; exit 1 on error.
fn write_output_file( path : Option< &str >, content : &str )
{
  if let Some( p ) = path
  {
    if let Err( e ) = std::fs::write( p, content.as_bytes() )
    {
      eprintln!( "Error: failed to write output file '{p}': {e}" );
      std::process::exit( 1 );
    }
  }
}

/// Validate `out` against `--expect`; apply retry/default/fail strategy on mismatch.
///
/// Returns `out` when validation passes (or when `--expect` is not set).
/// Exits the process when a mismatch is not resolved:
/// - Retry exhausted → exit 3; Fail strategy → exit 3.
/// - Retry succeeds or Default strategy → prints result and exits 0.
fn apply_expect_validation( cli : &CliArgs, builder : &ClaudeCommand, out : String ) -> String
{
  let Some( ref pattern ) = cli.expect else { return out; };
  let allowed : Vec< String > = pattern.split( '|' )
    .map( | s | s.trim().to_lowercase() )
    .collect();
  let trimmed = out.trim().to_lowercase();

  if allowed.iter().any( | v | v.as_str() == trimmed ) { return out; }

  match &cli.expect_strategy
  {
    Some( ExpectStrategy::Retry ) =>
    {
      let retries = cli.expect_retries.unwrap_or( 0 ) as usize;
      for _ in 0 .. retries
      {
        let retry_output = match builder.execute()
        {
          Ok( o )  => o,
          Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
        };
        if !retry_output.stderr.is_empty() { eprint!( "{}", retry_output.stderr ); }
        if retry_output.exit_code != 0 { std::process::exit( retry_output.exit_code ); }
        let retry_out = if cli.strip_fences
        {
          strip_fences( &retry_output.stdout )
        }
        else
        {
          retry_output.stdout
        };
        if allowed.iter().any( | v | v.as_str() == retry_out.trim().to_lowercase() )
        {
          write_output_file( cli.output_file.as_deref(), &retry_out );
          print!( "{retry_out}" );
          std::process::exit( 0 );
        }
      }
      std::process::exit( 3 );
    }
    Some( ExpectStrategy::Default( fallback ) ) =>
    {
      let fallback = fallback.clone();
      write_output_file( cli.output_file.as_deref(), &fallback );
      print!( "{fallback}" );
      std::process::exit( 0 );
    }
    _ => std::process::exit( 3 ), // Fail strategy (default or explicit ExpectStrategy::Fail)
  }
}

/// Execute in non-interactive print mode (captures output).
///
/// Both `--print` (passed to claude) and `execute()` (captures stdout) are required:
/// `--print` tells claude to run single-shot with clean text output (no TUI);
/// `execute()` captures that output into memory for programmatic use.
/// Without `--print`, captured output would be TUI escape codes.
/// Without `execute()`, clean output would go straight to terminal uncaptured.
fn run_print_mode( builder : &ClaudeCommand, cli : &CliArgs )
{
  let verbosity = cli.verbosity.unwrap_or_default();

  // Fix(BUG-240): always emit fatal spawn errors regardless of verbosity.
  // Root cause: the Err branch was guarded by verbosity.shows_errors(); at CLR_VERBOSITY=0
  //   spawn failures (binary not found, permission denied) produced zero stderr output while
  //   still exiting 1 — a perfectly silent failure with no diagnostic.
  // Pitfall: verbosity controls runner diagnostics (progress, trace); it must never gate
  //   fatal errors that signal a broken environment — those are always user-visible.
  let output = match builder.execute()
  {
    Ok( o )  => o,
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
  };

  if !output.stderr.is_empty() { eprint!( "{}", output.stderr ); }

  // Fix(BUG-037): classify non-zero exit and emit labeled per-type diagnostic.
  // Root cause: prior code emitted a generic "possible rate limit" message for ALL silent
  //   failures, hiding the actual failure mode (rate limit vs quota vs auth vs API error vs signal).
  // Pitfall: classify_error() scans stdout AND stderr — `claude --print` on rate-limit
  //   writes the reason only to its JSONL session file, so stderr is often empty; the
  //   stdout scan ensures auth/rate-limit patterns are still caught.
  if let Some( kind ) = output.classify_error()
  {
    if verbosity.shows_errors()
    {
      let label = match kind
      {
        ErrorKind::RateLimit      => "rate limit",
        ErrorKind::QuotaExhausted => "quota exhausted",
        ErrorKind::ApiError       => "API error",
        ErrorKind::AuthError      => "auth error",
        ErrorKind::Signal         => "terminated by signal",
        ErrorKind::Unknown        => "unknown error",
      };
      eprintln!( "Error: {label} (exit {})", output.exit_code );
    }
  }

  // Fix(BUG-239): propagate the exact subprocess exit code instead of collapsing to 1.
  // Root cause: `std::process::exit(1)` was hardcoded; callers that branch on exit code
  //   (e.g. 2 = rate-limit, 130 = SIGINT cancel) received 1 regardless of actual cause.
  // Pitfall: never substitute a generic exit code where the subprocess's code is available;
  //   use output.exit_code directly so all domain-specific codes are preserved.
  // Fix(BUG-247): forward captured stdout to stderr on subprocess failure so callers see it.
  // Root cause: std::process::exit() was called before stdout was printed; on the failure
  //   path the captured output was silently discarded.
  // Pitfall: stdout must be emitted BEFORE the exit call; order of these two blocks is load-bearing.
  if output.exit_code != 0 && !output.stdout.is_empty() { eprint!( "{}", output.stdout ); }
  if output.exit_code != 0 { std::process::exit( output.exit_code ); }

  // Expect validation: compare trimmed, lowercased output against allowed values.
  // Only active in print mode (run_print_mode is never called from run_interactive).
  let out = if cli.strip_fences { strip_fences( &output.stdout ) } else { output.stdout };
  let out = apply_expect_validation( cli, builder, out );

  write_output_file( cli.output_file.as_deref(), &out );
  print!( "{out}" );
}

/// Execute in interactive mode (TTY passthrough).
fn run_interactive( builder : &ClaudeCommand, _verbosity : VerbosityLevel )
{
  // Fix(BUG-240): always emit fatal spawn errors regardless of verbosity.
  // Root cause: same as run_print_mode — the Err branch was gated on shows_errors();
  //   at verbosity 0 a missing binary or permission error produced no stderr output.
  // Pitfall: mirrors the fix in run_print_mode; both execution paths must be updated
  //   together — missing one leaves interactive mode silently broken at low verbosity.
  let status = match builder.execute_interactive()
  {
    Ok( s )  => s,
    Err( e ) =>
    {
      eprintln!( "Error: {e}" );
      std::process::exit( 1 );
    }
  };

  if !status.success()
  {
    // Fix(BUG-242): use signal_exit_code() so SIGTERM (→143) and SIGKILL (→137) are
    //   propagated correctly in interactive mode.
    // Root cause: unwrap_or(1) collapsed all signal kills to exit code 1 in interactive
    //   mode; callers using Claude interactively could not distinguish Ctrl+C (SIGINT=130)
    //   from other kills.
    // Pitfall: mirrors the fix in execute() in claude_runner_core; both execution paths
    //   (print mode and interactive) must use signal_exit_code() for consistency.
    std::process::exit( signal_exit_code( &status ) );
  }
}

/// Parse, validate, and execute the `run` subcommand (default mode).  Never returns.
///
/// Shared implementation for both `clr run` and `clr ask` — called from both
/// `run_cli()` (after subcommand dispatch) and `dispatch_ask()`.
pub( super ) fn dispatch_run( tokens : &[ String ] ) -> !
{
  let mut cli = match parse_args( tokens )
  {
    Ok( c )  => c,
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
  };
  if let Err( e ) = apply_env_vars( &mut cli )
  {
    eprintln!( "Error: {e}" );
    std::process::exit( 1 );
  }

  if cli.help
  {
    print_help();
    std::process::exit( 0 );
  }

  if cli.print_mode && cli.message.is_none()
  {
    eprintln!( "Error: --print requires a message argument" );
    eprintln!( "Run with --help for usage." );
    std::process::exit( 1 );
  }

  let builder = build_claude_command( &cli );

  // Fix(BUG-248): warn when --keep-claudecode is set while CLAUDECODE is present in
  //   the parent environment — the child will run in nested-agent mode unintentionally.
  // Root cause: no diagnostic existed when the user explicitly disabled CLAUDECODE removal;
  //   the consequence (nested-agent context injection) is non-obvious without a warning.
  // Pitfall: gate on shows_warnings() (level ≥ 2) so operators who suppress output at
  //   --verbosity 0/1 still get silence; the warning is informational, not fatal.
  //   Placed before the dry-run check so it fires in all execution modes including --dry-run.
  {
    let verbosity_for_warning = cli.verbosity.unwrap_or_default();
    if cli.keep_claudecode
      && verbosity_for_warning.shows_warnings()
      && std::env::var( "CLAUDECODE" ).is_ok()
    {
      eprintln!(
        "Warning: --keep-claudecode is set and CLAUDECODE is present in environment; \
         child claude will run in nested-agent mode"
      );
    }
  }

  if cli.dry_run
  {
    handle_dry_run( &builder );
    std::process::exit( 0 );
  }

  run_built_command( &builder, &cli );
  std::process::exit( 0 );
}

/// Parse, validate, and execute the `ask` subcommand.  Never returns.
///
/// `ask` is a pure semantic alias for `run` — delegates directly to `dispatch_run()`.
/// The only difference from `clr run` is that `clr ask --help` shows the ask-specific
/// help text rather than the generic `clr` help.
pub( super ) fn dispatch_ask( tokens : &[ String ] ) -> !
{
  if tokens.iter().skip( 1 ).any( | t | t == "--help" || t == "-h" )
  {
    print_ask_help();
  }
  // Fix(BUG-249): 'clr ask help' must show ask help, not treat "help" as a message.
  // Root cause: only --help/-h were intercepted; positional "help" flowed into
  //   dispatch_run as a message and hit the session gate when limit was reached.
  // Pitfall: mirrors BUG-215 fix in run_cli() for 'clr run help'; both subcommands
  //   need the positional check; future subcommands that delegate to dispatch_run
  //   must include it too.
  if tokens.get( 1 ).map( String::as_str ) == Some( "help" )
  {
    print_ask_help();
  }
  dispatch_run( &tokens[ 1 .. ] );
}

/// Parse, validate, and execute the `isolated` subcommand.  Never returns.
pub( super ) fn dispatch_isolated( tokens : &[ String ] ) -> !
{
  let mut cli = match parse_isolated_args( &tokens[ 1 .. ] )
  {
    Ok( c )  => c,
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
  };
  apply_isolated_env_vars( &mut cli );
  if cli.creds_path.is_empty()
  {
    eprintln!( "Error: cannot resolve credentials path: HOME is not set; provide --creds or set CLR_CREDS\nRun with --help for usage." );
    std::process::exit( 1 );
  }
  run_isolated_command( &cli.creds_path, cli.timeout_secs, cli.trace, IsolatedModel::Default, cli.message.as_deref(), &cli.passthrough_args )
}

/// Parse, validate, and execute the `refresh` subcommand.  Never returns.
pub( super ) fn dispatch_refresh( tokens : &[ String ] ) -> !
{
  let mut cli = match parse_refresh_args( &tokens[ 1 .. ] )
  {
    Ok( c )  => c,
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
  };
  apply_refresh_env_vars( &mut cli );
  if cli.creds_path.is_empty()
  {
    eprintln!( "Error: cannot resolve credentials path: HOME is not set; provide --creds or set CLR_CREDS\nRun with --help for usage." );
    std::process::exit( 1 );
  }
  run_refresh_command( &cli.creds_path, cli.timeout_secs, cli.trace )
}
