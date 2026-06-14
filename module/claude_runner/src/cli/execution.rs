use claude_runner_core::{ ClaudeCommand, ErrorKind, ExecutionOutput, signal_exit_code };
use super::parse::{ CliArgs, ExpectStrategy };
use super::fence::strip_fences;

// Return a user-facing error message for a spawn `io::Error`.
//
// Distinguishes the common "not found" case (claude not installed) from other OS errors
// so callers can surface an actionable install hint without duplicating the check.
fn spawn_error_msg( e : &std::io::Error ) -> String
{
  if e.kind() == std::io::ErrorKind::NotFound
  {
    "claude binary not found in PATH — install with: npm i -g @anthropic-ai/claude-code".to_string()
  }
  else
  {
    format!( "Failed to execute Claude Code: {e}" )
  }
}

// Poll once in the `Ok(None)` arm of a `try_wait()` loop: check deadline and sleep.
//
// When the deadline is reached, kills the child, waits for it, prints the timeout
// error, and exits 4.  Never returns on timeout.  The caller's loop continues on
// the next iteration when the child is still running.
fn poll_timeout( child : &mut std::process::Child, deadline : std::time::Instant, timeout_secs : u32 )
{
  if std::time::Instant::now() >= deadline
  {
    let _ = child.kill();
    let _ = child.wait();
    eprintln!( "Error: timeout after {timeout_secs}s" );
    std::process::exit( 4 );
  }
  std::thread::sleep( core::time::Duration::from_millis( 50 ) );
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
    Some( ExpectStrategy::Fail ) | None => std::process::exit( 3 ),
  }
}

/// Execute one print-mode subprocess attempt with an optional timeout watchdog.
///
/// Returns the completed `ExecutionOutput`. On spawn failure or timeout, exits the
/// process directly (timeout → exit 4; spawn error → exit 1).  The caller is
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
    Err( e ) => { eprintln!( "Error: {}", spawn_error_msg( &e ) ); std::process::exit( 1 ); }
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
      Ok( None ) => poll_timeout( &mut child, deadline, timeout_secs ),
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
pub( super ) fn run_print_mode( builder : &ClaudeCommand, cli : &CliArgs )
{
  let verbosity          = cli.verbosity.unwrap_or_default();
  let retry_limit        = cli.retry_on_rate_limit.unwrap_or( 1 ) as usize;
  let retry_delay        = cli.retry_delay.unwrap_or( 30 );
  let timeout_secs       = cli.timeout.unwrap_or( 0 );
  let api_retry_limit    = cli.retry_on_api_error.unwrap_or( 0 ) as usize;
  let api_error_delay    = cli.api_error_delay.unwrap_or( 30 );
  let unknown_retry_limit = cli.retry_on_unknown_error.unwrap_or( 0 ) as usize;
  let mut attempts         = 0usize;
  let mut api_attempts     = 0usize;
  let mut unknown_attempts = 0usize;

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
      // QuotaExhausted, AuthError, Signal: never retry.
      // ApiError: retry if --retry-on-api-error N set.
      // Unknown: retry if --retry-on-unknown-error N set.
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

      // Retry on ApiError if retries remain.
      if let Some( ErrorKind::ApiError ) = &kind
      {
        if api_attempts < api_retry_limit
        {
          api_attempts += 1;
          if verbosity.shows_warnings()
          {
            eprintln!(
              "API error (attempt {api_attempts}/{}); retrying in {api_error_delay}s…",
              api_retry_limit + 1
            );
          }
          if api_error_delay > 0
          {
            std::thread::sleep( core::time::Duration::from_secs( u64::from( api_error_delay ) ) );
          }
          continue;
        }
      }

      // Retry on Unknown if retries remain.
      if let Some( ErrorKind::Unknown ) = &kind
      {
        if unknown_attempts < unknown_retry_limit
        {
          unknown_attempts += 1;
          if verbosity.shows_warnings()
          {
            eprintln!(
              "Unknown error (attempt {unknown_attempts}/{}); retrying in {retry_delay}s…",
              unknown_retry_limit + 1
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
          Some( ErrorKind::RateLimit ) if attempts > 0       => "rate limit retries exhausted",
          Some( ErrorKind::RateLimit )                       => "rate limit",
          Some( ErrorKind::QuotaExhausted )                  => "quota exhausted",
          Some( ErrorKind::ApiError ) if api_attempts > 0    => "API error retries exhausted",
          Some( ErrorKind::ApiError )                        => "API error",
          Some( ErrorKind::AuthError )                       => "auth error",
          Some( ErrorKind::Signal )                          => "terminated by signal",
          Some( ErrorKind::Unknown ) if unknown_attempts > 0 => "unknown error retries exhausted",
          Some( ErrorKind::Unknown ) | None                  => "unknown error",
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
      Ok( None ) => poll_timeout( &mut child, deadline, timeout_secs ),
      Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
    }
  }
}
