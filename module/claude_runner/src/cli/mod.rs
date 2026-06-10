mod parse;
mod cred_parse;
mod builder;
mod fence;
mod credential;
mod help;
mod gate;

use claude_runner_core::{ ClaudeCommand, EffortLevel, ErrorKind, ExecutionOutput, IsolatedModel, signal_exit_code };
use parse::{ CliArgs, ExpectStrategy };
use cred_parse::{
  parse_isolated_args, parse_refresh_args,
  apply_isolated_env_vars, apply_refresh_env_vars,
};
pub use fence::strip_fences;
use credential::{ run_isolated_command, run_refresh_command };
use help::print_ask_help;
use gate::wait_for_session_slot;

pub( super ) use parse::{ parse_args, apply_env_vars };
pub( super ) use builder::build_claude_command;
pub( super ) use help::print_help;

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
        // Fix(BUG-250): extend guard to catch one-character insertion/substitution typos.
        // Root cause: prefix/superstring checks only caught truncations and extensions;
        //   mid-word insertions (e.g. "assk" for "ask") bypassed the guard and fell through
        //   to dispatch_run, treating the typo silently as the message argument to Claude.
        // Pitfall: is_close_typo requires matching first char to avoid false positives for
        //   common English words that happen to be within edit distance 1 (e.g. "task" → "ask").
        if first != sub
          && ( sub.starts_with( first.as_str() ) || first.starts_with( sub ) || is_close_typo( first, sub ) )
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

/// Returns `true` when `first` is likely a one-character typo of `sub`.
///
/// Two conditions must both hold:
/// 1. The first character matches — typos virtually always preserve the initial letter;
///    a different first character means a different word entirely, not a typo.
/// 2. Levenshtein distance exactly 1 — one substitution, insertion, or deletion.
///
/// The first-character constraint prevents false positives for common English words that
/// happen to be within edit distance 1 of a known short subcommand name (e.g. `"task"`
/// has edit distance 1 from `"ask"`, but `'t' ≠ 'a'` so it is correctly excluded).
///
/// Used by [`guard_unknown_subcommand`] for mid-word insertion/substitution typos that
/// are not caught by either `starts_with` direction (e.g. `"assk"` vs `"ask"`).
fn is_close_typo( first : &str, sub : &str ) -> bool
{
  // First-character guard: real typos start with the correct letter.
  if first.chars().next() != sub.chars().next() { return false; }
  let a = first.as_bytes();
  let b = sub.as_bytes();
  let la = a.len();
  let lb = b.len();
  if la.abs_diff( lb ) > 1 { return false; }
  if la == lb
  {
    // Same length: exactly one character substitution.
    return a.iter().zip( b.iter() ).filter( |( x, y )| x != y ).count() == 1;
  }
  // Lengths differ by 1: exactly one insertion or deletion.
  let ( longer, shorter ) = if la > lb { ( a, b ) } else { ( b, a ) };
  let mut i = 0;
  let mut j = 0;
  let mut skipped = false;
  while i < longer.len() && j < shorter.len()
  {
    if longer[ i ] == shorter[ j ] { i += 1; j += 1; }
    else if skipped               { return false; }
    else                          { skipped = true; i += 1; }
  }
  true
}

pub( super ) fn run_built_command( builder : &ClaudeCommand, cli : &CliArgs )
{
  let verbosity = cli.verbosity.unwrap_or_default();

  // Concurrency gate: block before subprocess launch when max active claude sessions is reached.
  // Default limit is 15; 0 = unlimited.  dry-run is bypassed by caller (never reaches here).
  let max_sessions = cli.max_sessions.unwrap_or( 15 );
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
    run_interactive( builder, cli );
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

/// Execute one print-mode subprocess attempt with an optional timeout watchdog.
///
/// Returns the completed `ExecutionOutput`. On spawn failure or timeout, exits the
/// process directly (timeout → exit 2; spawn error → exit 1).  The caller is
/// responsible for retry logic and success/failure dispatch.
///
/// When `timeout_secs == 0`, `builder.execute()` is used (blocking, no polling overhead).
/// When `timeout_secs > 0`, `spawn_piped()` + `try_wait()` polling is used, mirroring the
/// established pattern in `claude_runner_core::isolated`.
fn execute_print_attempt( builder : &ClaudeCommand, timeout_secs : u32 ) -> ExecutionOutput
{
  if timeout_secs == 0
  {
    // Fix(BUG-240): always emit fatal spawn errors regardless of verbosity.
    // Root cause: Err(e) branch was inside `if verbosity.shows_errors()`; verbosity 0 swallowed fatal errors.
    // Pitfall: verbosity gates runner diagnostics only — never fatal errors.
    return match builder.execute()
    {
      Ok( o )  => o,
      Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
    };
  }

  // Timeout path: spawn and poll with try_wait(), mirroring isolated.rs BUG-243 fix.
  // Pitfall: keep the Child in scope so child.kill() + child.wait() can recover output
  //   and prevent the subprocess from becoming an orphan.
  let mut child = match builder.spawn_piped()
  {
    Ok( c )  => c,
    Err( e ) =>
    {
      let msg = if e.kind() == std::io::ErrorKind::NotFound
      {
        "claude binary not found in PATH — install with: npm i -g @anthropic-ai/claude-code".to_string()
      }
      else
      {
        format!( "Failed to execute Claude Code: {e}" )
      };
      eprintln!( "Error: {msg}" );
      std::process::exit( 1 );
    }
  };

  let deadline = std::time::Instant::now()
    + core::time::Duration::from_secs( u64::from( timeout_secs ) );

  loop
  {
    match child.try_wait()
    {
      Ok( Some( _ ) ) =>
      {
        let raw = match child.wait_with_output()
        {
          Ok( o )  => o,
          Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
        };
        let exit_code = signal_exit_code( &raw.status );
        let stdout = String::from_utf8_lossy( &raw.stdout ).to_string();
        let stderr = String::from_utf8_lossy( &raw.stderr ).to_string();
        return ExecutionOutput { stdout, stderr, exit_code };
      }
      Ok( None ) =>
      {
        if std::time::Instant::now() >= deadline
        {
          let _ = child.kill();
          let _ = child.wait();
          eprintln!( "Error: timeout after {timeout_secs}s" );
          std::process::exit( 2 );
        }
        std::thread::sleep( core::time::Duration::from_millis( 50 ) );
      }
      Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
    }
  }
}

/// Execute in non-interactive print mode (captures output).
///
/// Both `--print` (passed to claude) and output capture are required:
/// `--print` tells claude to run single-shot with clean text output (no TUI);
/// output capture makes the content available for programmatic use.
/// Without `--print`, captured output would be TUI escape codes.
///
/// Supports automatic retry on transient `RateLimit` errors (exit code 2 with no
/// `QuotaExhausted` pattern) when `--retry-on-rate-limit` is set to a non-zero value.
/// Supports subprocess timeout via `--timeout` (0 = unlimited).
fn run_print_mode( builder : &ClaudeCommand, cli : &CliArgs )
{
  let verbosity    = cli.verbosity.unwrap_or_default();
  let retry_limit  = cli.retry_on_rate_limit.unwrap_or( 0 ) as usize;
  let retry_delay  = cli.retry_delay.unwrap_or( 60 );
  let timeout_secs = cli.timeout.unwrap_or( 0 );
  let mut attempts = 0usize;

  loop
  {
    // Fix(BUG-240): spawn errors always emitted regardless of verbosity (inside execute_print_attempt).
    // Root cause: Err(e) branch was guarded by verbosity check; verbosity 0 swallowed fatal spawn errors.
    // Pitfall: verbosity gates diagnostics only — fatal errors must surface regardless of verbosity level.
    let output = execute_print_attempt( builder, timeout_secs );

    if !output.stderr.is_empty() { eprint!( "{}", output.stderr ); }

    if output.exit_code != 0
    {
      // Fix(BUG-037): classify non-zero exit for labeled diagnostic.
      // Root cause: no error classification existed; all non-zero exits produced identical log output.
      // Pitfall: classify_error() scans stdout AND stderr — rate-limit reason may be in stdout.
      let kind = output.classify_error();

      // Retry on transient RateLimit if retries remain.
      // QuotaExhausted, AuthError, ApiError, Signal, Unknown: never retry.
      if let Some( ErrorKind::RateLimit ) = &kind
      {
        if attempts < retry_limit
        {
          attempts += 1;
          if verbosity.shows_warnings()
          {
            eprintln!(
              "Rate limit (attempt {attempts}/{}); retrying in {retry_delay}s…",
              retry_limit + 1
            );
          }
          if retry_delay > 0
          {
            std::thread::sleep( core::time::Duration::from_secs( u64::from( retry_delay ) ) );
          }
          continue;
        }
      }

      // Non-retriable error or retries exhausted.
      if verbosity.shows_errors()
      {
        let label = match &kind
        {
          Some( ErrorKind::RateLimit ) if attempts > 0 => "rate limit retries exhausted",
          Some( ErrorKind::RateLimit )                 => "rate limit",
          Some( ErrorKind::QuotaExhausted )            => "quota exhausted",
          Some( ErrorKind::ApiError )                  => "API error",
          Some( ErrorKind::AuthError )                 => "auth error",
          Some( ErrorKind::Signal )                    => "terminated by signal",
          Some( ErrorKind::Unknown ) | None            => "unknown error",
        };
        eprintln!( "Error: {label} (exit {})", output.exit_code );
      }

      // Fix(BUG-239): propagate exact subprocess exit code.
      // Root cause: std::process::exit(1) was hardcoded; subprocess exit code was discarded.
      // Pitfall: any hardcoded exit(1) after a subprocess wait silently discards the real code.
      // Fix(BUG-247): forward captured stdout to stderr on failure before exiting.
      // Root cause: on non-zero exit, captured stdout was never forwarded; diagnostic output was lost.
      // Pitfall: in print mode stdout is captured — on failure it must be re-emitted to stderr.
      if !output.stdout.is_empty() { eprint!( "{}", output.stdout ); }
      std::process::exit( output.exit_code );
    }

    // Success path — expect validation, file write, stdout.
    let out = if cli.strip_fences { strip_fences( &output.stdout ) } else { output.stdout };
    let out = apply_expect_validation( cli, builder, out );
    write_output_file( cli.output_file.as_deref(), &out );
    print!( "{out}" );
    return;
  }
}

/// Execute in interactive mode (TTY passthrough) with optional timeout.
///
/// When `timeout_secs == 0`, uses the blocking `execute_interactive()` path.
/// When `timeout_secs > 0`, uses `spawn_tty()` + `try_wait()` polling so the
/// subprocess can be killed after the deadline while still using the TTY.
fn run_interactive( builder : &ClaudeCommand, cli : &CliArgs )
{
  let timeout_secs = cli.timeout.unwrap_or( 0 );

  if timeout_secs == 0
  {
    // Fix(BUG-240): always emit fatal spawn errors regardless of verbosity.
    // Root cause: Err(e) branch was inside `if verbosity.shows_errors()`; verbosity 0 swallowed errors.
    // Pitfall: verbosity gates diagnostics only — never fatal errors.
    // Fix(BUG-242): use signal_exit_code() for interactive signal propagation.
    // Root cause: status.code().unwrap_or(1) collapsed SIGTERM (143) and SIGKILL (137) to 1.
    // Pitfall: on Unix code() returns None for signal-killed processes; always use signal_exit_code().
    let status = match builder.execute_interactive()
    {
      Ok( s )  => s,
      Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
    };
    if !status.success()
    {
      std::process::exit( signal_exit_code( &status ) );
    }
    return;
  }

  // Timeout path: spawn with inherited TTY stdio and poll, mirroring execute_print_attempt.
  let mut child = match builder.spawn_tty()
  {
    Ok( c )  => c,
    Err( e ) =>
    {
      let msg = if e.kind() == std::io::ErrorKind::NotFound
      {
        "claude binary not found in PATH — install with: npm i -g @anthropic-ai/claude-code".to_string()
      }
      else
      {
        format!( "Failed to execute Claude Code: {e}" )
      };
      eprintln!( "Error: {msg}" );
      std::process::exit( 1 );
    }
  };

  let deadline = std::time::Instant::now()
    + core::time::Duration::from_secs( u64::from( timeout_secs ) );

  loop
  {
    match child.try_wait()
    {
      Ok( Some( status ) ) =>
      {
        if !status.success()
        {
          std::process::exit( signal_exit_code( &status ) );
        }
        return;
      }
      Ok( None ) =>
      {
        if std::time::Instant::now() >= deadline
        {
          let _ = child.kill();
          let _ = child.wait();
          eprintln!( "Error: timeout after {timeout_secs}s" );
          std::process::exit( 2 );
        }
        std::thread::sleep( core::time::Duration::from_millis( 50 ) );
      }
      Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
    }
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
  run_isolated_command(
    "isolated",
    &cli.creds_path,
    cli.timeout_secs,
    cli.trace,
    IsolatedModel::Default,
    EffortLevel::Max,
    cli.message.as_deref(),
    &cli.passthrough_args,
    cli.message.is_some(), // skip-perms when a real task message is present
    false,                 // chrome stays on for isolated tasks (may use browser tools)
  )
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
