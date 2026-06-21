use claude_runner_core::{ ClaudeCommand, ErrorKind, ExecutionOutput, signal_exit_code };
use super::parse::{ CliArgs, ExpectStrategy };
use super::fence::strip_fences;

// Return a user-facing error message for a spawn `io::Error`.
//
// Distinguishes the common "not found" case (claude not installed) from other OS errors
// so callers can surface an actionable install hint without duplicating the check.
// Fix(BUG-298): [Runner] prefix missing from all branches — see task/claude_runner/bug/298_spawn_error_missing_runner_class_prefix.md
fn spawn_error_msg( e : &std::io::Error ) -> String
{
  if e.kind() == std::io::ErrorKind::NotFound
  {
    "[Runner] claude binary not found in PATH — install with: npm i -g @anthropic-ai/claude-code".to_string()
  }
  else
  {
    format!( "[Runner] Failed to execute Claude Code: {e}" )
  }
}

// Check if the deadline has been reached; if so, kill the child and return `true`.
// Otherwise sleep 50ms and return `false`.
//
// The caller decides what to do on timeout: print-mode returns a synthetic
// `ExecutionOutput` (exit 4) to the retry loop; interactive mode exits directly.
fn check_timeout( child : &mut std::process::Child, deadline : std::time::Instant ) -> bool
{
  if std::time::Instant::now() >= deadline
  {
    let _ = child.kill();
    let _ = child.wait();
    return true;
  }
  std::thread::sleep( core::time::Duration::from_millis( 50 ) );
  false
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

// -------------------------------------------------------------------
// Error class taxonomy and 3-tier resolution
// -------------------------------------------------------------------

/// Semantic class for caller-facing retry decisions.
///
/// Maps `ErrorKind` (subprocess classification) and CLR-layer ad-hoc exits
/// to a uniform 6-class taxonomy for the retry loop.  Validation and Runner
/// classes are handled outside the main retry loop.
#[ derive( Clone, Copy ) ]
enum ErrorClass
{
  Transient,
  Account,
  Auth,
  Service,
  Process,
  Unknown,
}

impl ErrorClass
{
  fn label( self ) -> &'static str
  {
    match self
    {
      ErrorClass::Transient => "Transient",
      ErrorClass::Account   => "Account",
      ErrorClass::Auth      => "Auth",
      ErrorClass::Service   => "Service",
      ErrorClass::Process   => "Process",
      ErrorClass::Unknown   => "Unknown",
    }
  }
  fn fallback_message( self ) -> &'static str
  {
    match self
    {
      ErrorClass::Transient => "rate limit",
      ErrorClass::Account   => "quota exhausted",
      ErrorClass::Auth      => "auth error",
      ErrorClass::Service   => "API error",
      ErrorClass::Process   => "terminated by signal",
      ErrorClass::Unknown   => "unknown error",
    }
  }
}

/// Map an `ErrorKind` (or CLR-layer exit 4) to an `ErrorClass`.
fn classify_to_class( kind : Option< &ErrorKind >, exit_code : i32 ) -> ErrorClass
{
  if exit_code == 4 { return ErrorClass::Process; }
  match kind
  {
    Some( ErrorKind::RateLimit )      => ErrorClass::Transient,
    Some( ErrorKind::QuotaExhausted ) => ErrorClass::Account,
    Some( ErrorKind::AuthError )      => ErrorClass::Auth,
    Some( ErrorKind::ApiError )       => ErrorClass::Service,
    Some( ErrorKind::Signal )         => ErrorClass::Process,
    Some( ErrorKind::Unknown ) | None => ErrorClass::Unknown,
  }
}

/// 3-tier resolution for retry count: override ?? class-specific ?? fallback (2).
fn resolve_count(
  over      : Option< u8 >,
  class_cli : Option< u8 >,
  fallback  : Option< u8 >,
) -> u8
{
  over.or( class_cli ).or( fallback ).unwrap_or( 2 )
}

/// 3-tier resolution for retry delay: override ?? class-specific ?? fallback (30).
fn resolve_delay( over : Option< u32 >, class : Option< u32 >, fallback : Option< u32 > ) -> u32
{
  over.or( class ).or( fallback ).unwrap_or( 30 )
}

/// Return the class-specific (count, delay) fields from `CliArgs` for the given class.
fn class_fields( cli : &CliArgs, class : ErrorClass ) -> ( Option< u8 >, Option< u32 > )
{
  match class
  {
    ErrorClass::Transient => ( cli.retry_on_transient, cli.transient_delay ),
    ErrorClass::Account   => ( cli.retry_on_account,   cli.account_delay ),
    ErrorClass::Auth      => ( cli.retry_on_auth,       cli.auth_delay ),
    ErrorClass::Service   => ( cli.retry_on_service,    cli.service_delay ),
    ErrorClass::Process   => ( cli.retry_on_process,    cli.process_delay ),
    ErrorClass::Unknown   => ( cli.retry_on_unknown,    cli.unknown_delay ),
  }
}

/// Extract the first non-empty line from stdout or stderr as the original message.
/// Falls back to the class-specific default when both are empty.
fn first_message( output : &ExecutionOutput, class : ErrorClass ) -> String
{
  for s in [ &output.stdout, &output.stderr ]
  {
    for line in s.lines()
    {
      let t = line.trim();
      if !t.is_empty() { return t.to_string(); }
    }
  }
  class.fallback_message().to_string()
}

/// Format the retry delay suffix: " in Xs" when delay > 0, empty when immediate.
fn delay_suffix( delay : u32 ) -> String
{
  if delay > 0 { format!( " in {delay}s" ) } else { String::new() }
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
      let retries = resolve_count(
        cli.retry_override,
        cli.retry_on_validation,
        cli.retry_default,
      ) as usize;
      let delay = resolve_delay(
        cli.retry_override_delay,
        cli.validation_delay,
        cli.retry_default_delay,
      );
      let msg = format!( "expected \"{pattern}\", got \"{}\"", out.trim() );
      for attempt in 1 ..= retries
      {
        let suf = delay_suffix( delay );
        eprintln!(
          "[Validation] {msg} — retrying{suf} (attempt {attempt}/{})…",
          retries + 1
        );
        if delay > 0
        {
          std::thread::sleep( core::time::Duration::from_secs( u64::from( delay ) ) );
        }
        let retry_output = match builder.execute()
        {
          Ok( o )  => o,
          Err( e ) => { eprintln!( "Error: [Runner] {e}" ); std::process::exit( 1 ); }
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
      eprintln!(
        "Error: [Validation] expected \"{pattern}\", got \"{}\" — retries exhausted (exit 3)",
        out.trim()
      );
      std::process::exit( 3 );
    }
    Some( ExpectStrategy::Default( fallback ) ) =>
    {
      let fallback = fallback.clone();
      write_output_file( cli.output_file.as_deref(), &fallback );
      print!( "{fallback}" );
      std::process::exit( 0 );
    }
    Some( ExpectStrategy::Fail ) | None =>
    {
      eprintln!(
        "Error: [Validation] expected \"{pattern}\", got \"{}\" (exit 3)",
        out.trim()
      );
      std::process::exit( 3 );
    }
  }
}

/// Execute one print-mode subprocess attempt with an optional timeout watchdog.
///
/// Returns `Ok(ExecutionOutput)` on success or timeout (exit 4 for timeout).
/// Returns `Err(io::Error)` on spawn failure so `run_print_mode()` can apply
/// Runner-class retry logic via `apply_runner_retry()`.
///
/// When `timeout_secs == 0`, `builder.execute()` is used (blocking, no polling overhead).
/// When `timeout_secs > 0`, `spawn_piped()` + `try_wait()` polling is used, mirroring the
/// established pattern in `claude_runner_core::isolated`.
fn execute_print_attempt( builder : &ClaudeCommand, timeout_secs : u32 )
  -> Result< ExecutionOutput, std::io::Error >
{
  if timeout_secs == 0
  {
    // Fix(BUG-240): always emit fatal spawn errors regardless of verbosity.
    // Root cause: Err(e) branch was inside `if verbosity.shows_errors()`; verbosity 0 swallowed fatal errors.
    // Pitfall: verbosity gates runner diagnostics only — never fatal errors.
    // Fix(BUG-299): return Err instead of exit so run_print_mode() can apply runner retry.
    return match builder.execute()
    {
      Ok( o )  => Ok( o ),
      Err( e ) => Err( std::io::Error::other( e.to_string() ) ),
    };
  }

  // Timeout path: spawn and poll with try_wait(), mirroring isolated.rs BUG-243 fix.
  // Pitfall: keep the Child in scope so child.kill() + child.wait() can recover output
  //   and prevent the subprocess from becoming an orphan.
  // Fix(BUG-299): return Err with descriptive message so run_print_mode() can apply runner retry.
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
      return Err( std::io::Error::other( msg ) );
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
        return Ok( ExecutionOutput { stdout, stderr, exit_code } );
      }
      Ok( None ) =>
      {
        if check_timeout( &mut child, deadline )
        {
          return Ok( ExecutionOutput
          {
            stdout   : String::new(),
            stderr   : format!( "timeout after {timeout_secs}s" ),
            exit_code : 4,
          } );
        }
      }
      Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
    }
  }
}

/// Apply Runner-class retry logic for spawn failures.
///
/// On non-final attempt: emits a `[Runner]` retry message, sleeps the configured
/// delay, and returns `()` so the caller can re-attempt the spawn.
/// On exhaustion: emits a `[Runner]` error message and calls `std::process::exit(1)`.
/// Visibility `pub(super)` so gate.rs (sibling under `cli/`) can call it for gate timeouts.
///
/// # Fix(BUG-299)
/// Root cause: `--retry-on-runner`/`--runner-delay` params were parsed but never wired to
///   any retry call site — all spawn-error arms called `exit(1)` directly, bypassing the system.
/// Pitfall: Runner retry is caller-driven (Option B): `run_print_mode()` owns the outer loop
///   and calls `execute_print_attempt()` again after `apply_runner_retry()` returns `()`.
///   Unlike `apply_expect_validation()` (owns its loop), this function only decides+waits.
pub( super ) fn apply_runner_retry( cli : &CliArgs, err : &std::io::Error, attempt : &mut u32 )
{
  let count = resolve_count( cli.retry_override, cli.retry_on_runner, cli.retry_default );
  let delay = resolve_delay( cli.retry_override_delay, cli.runner_delay, cli.retry_default_delay );
  let msg   = err.to_string();

  if *attempt < u32::from( count )
  {
    *attempt += 1;
    let suf = delay_suffix( delay );
    eprintln!(
      "[Runner] {msg} — retrying{suf} (attempt {}/{})…",
      *attempt,
      u32::from( count ) + 1
    );
    if delay > 0
    {
      std::thread::sleep( core::time::Duration::from_secs( u64::from( delay ) ) );
    }
    return;
  }

  eprintln!( "Error: [Runner] {msg} — retries exhausted (exit 1)" );
  std::process::exit( 1 );
}

/// Default watchdog for print-mode when `--timeout` is absent and `CLR_TIMEOUT` is unset.
/// Interactive mode retains `unwrap_or( 0 )` (unlimited) — `run_interactive()` must NOT adopt this constant.
const DEFAULT_PRINT_TIMEOUT_SECS : u32 = 3600;

/// Resolve the default print-mode timeout.
///
/// In production: always returns `DEFAULT_PRINT_TIMEOUT_SECS` (3600).
/// In tests: `_CLR_DEFAULT_TIMEOUT` env var overrides the constant so integration
///   tests can verify the kill path without waiting 3600 seconds.
/// The `_` prefix signals internal/test-only use — not exposed in `--help`.
fn default_print_timeout() -> u32
{
  std::env::var( "_CLR_DEFAULT_TIMEOUT" )
    .ok()
    .and_then( | s | s.parse().ok() )
    .unwrap_or( DEFAULT_PRINT_TIMEOUT_SECS )
}

/// Execute in non-interactive print mode (captures output).
///
/// Both `--print` (passed to claude) and output capture are required:
/// `--print` tells claude to run single-shot with clean text output (no TUI);
/// output capture makes the content available for programmatic use.
/// Without `--print`, captured output would be TUI escape codes.
///
/// Uses a 4-tier retry hierarchy: override → class-specific → class-default → fallback.
/// Every error class is retried when its effective count > 0.
/// Console output uses `[Class] <message>` format on stderr.
/// Supports subprocess timeout via `--timeout` (0 = unlimited; absent = `DEFAULT_PRINT_TIMEOUT_SECS`).
#[ allow( clippy::too_many_lines ) ]
pub( super ) fn run_print_mode( builder : &ClaudeCommand, cli : &CliArgs )
{
  let verbosity    = cli.verbosity.unwrap_or_default();
  // Fix(BUG-305): print-mode sessions had no default watchdog, leaving unattended sessions unbounded.
  // Root cause: unwrap_or( 0 ) treated absent --timeout as unlimited; print-mode should default to 1h.
  // Pitfall: DEFAULT_PRINT_TIMEOUT_SECS applies ONLY here in run_print_mode(); run_interactive() must retain unwrap_or( 0 ) — it is user-attended.
  let timeout_secs = cli.timeout.unwrap_or( default_print_timeout() );
  // Validate stdin file before the retry loop — a missing file is a user error,
  // not a transient spawn failure; it must not trigger runner retry.
  if let Some( ref file_path ) = cli.file
  {
    if let Err( e ) = std::fs::File::open( file_path )
    {
      eprintln!( "Error: cannot open stdin file '{file_path}': {e}" );
      std::process::exit( 1 );
    }
  }
  // Per-class attempt counters: [Transient, Account, Auth, Service, Process, Unknown]
  let mut attempts = [ 0usize; 6 ];
  // Fix(BUG-299): Runner retry counter — tracks spawn failure attempts separately from class retries.
  let mut runner_attempt = 0u32;

  loop
  {
    // Fix(BUG-240): spawn errors always emitted regardless of verbosity (inside execute_print_attempt).
    // Root cause: Err(e) branch was guarded by verbosity check; verbosity 0 swallowed fatal spawn errors.
    // Pitfall: verbosity gates diagnostics only — fatal errors must surface regardless of verbosity level.
    // Fix(BUG-299): handle Err(spawn_err) from execute_print_attempt() via apply_runner_retry().
    let output = match execute_print_attempt( builder, timeout_secs )
    {
      Ok( o )  => o,
      Err( e ) => { apply_runner_retry( cli, &e, &mut runner_attempt ); continue; }
    };

    if !output.stderr.is_empty() { eprint!( "{}", output.stderr ); }

    if output.exit_code != 0
    {
      // Fix(BUG-037): classify non-zero exit for labeled diagnostic.
      // Root cause: no error classification existed; all non-zero exits produced identical log output.
      // Pitfall: classify_error() scans stdout AND stderr — rate-limit reason may be in stdout.
      let kind = output.classify_error();
      let class = classify_to_class( kind.as_ref(), output.exit_code );
      let class_idx = class as usize;
      let label = class.label();
      let ( count_field, delay_field ) = class_fields( cli, class );
      let limit = resolve_count( cli.retry_override, count_field, cli.retry_default ) as usize;
      let delay = resolve_delay( cli.retry_override_delay, delay_field, cli.retry_default_delay );
      let msg = first_message( &output, class );

      if attempts[ class_idx ] < limit
      {
        attempts[ class_idx ] += 1;
        if verbosity.shows_warnings()
        {
          let suf = delay_suffix( delay );
          eprintln!(
            "[{label}] {msg} — retrying{suf} (attempt {}/{})…",
            attempts[ class_idx ],
            limit + 1
          );
        }
        if delay > 0
        {
          std::thread::sleep( core::time::Duration::from_secs( u64::from( delay ) ) );
        }
        continue;
      }

      // Non-retriable error or retries exhausted.
      if verbosity.shows_errors()
      {
        if attempts[ class_idx ] > 0
        {
          eprintln!( "Error: [{label}] {msg} — retries exhausted (exit {})", output.exit_code );
        }
        else
        {
          eprintln!( "Error: [{label}] {msg} (exit {})", output.exit_code );
        }
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
    // summary format: intercept JSON output, render YAML header + text body.
    // Falls back to raw output when JSON cannot be parsed.
    let out = if cli.output_style.as_deref().unwrap_or( "summary" ) == "summary"
    {
      super::summary::render_summary( &out ).unwrap_or( out )
    }
    else
    {
      out
    };
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
pub( super ) fn run_interactive( builder : &ClaudeCommand, cli : &CliArgs )
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
      Err( e ) => { eprintln!( "Error: [Runner] {e}" ); std::process::exit( 1 ); }
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
    Err( e ) => { eprintln!( "Error: {}", spawn_error_msg( &e ) ); std::process::exit( 1 ); }
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
        if check_timeout( &mut child, deadline )
        {
          eprintln!( "Error: timeout after {timeout_secs}s" );
          std::process::exit( 4 );
        }
      }
      Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
    }
  }
}
