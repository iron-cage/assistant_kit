use claude_runner_core::{ ClaudeCommand, ErrorKind, ExecutionOutput, signal_exit_code };
use super::parse::{ CliArgs, ExpectStrategy };
use super::fence::strip_fences;
use claude_journal::{ EventRecord, EventType, JournalWriter };
use claude_storage_core::SessionId;

// -------------------------------------------------------------------
// Journal helpers
// -------------------------------------------------------------------

// Emit a journal event; silently ignores write errors.
//
// Journaling is best-effort — a write failure must never abort the runner.
fn emit( writer : Option< &JournalWriter >, ev : &EventRecord )
{
  if let Some( w ) = writer { let _ = w.append( ev ); }
}

// Return true if the journal level is "full" (stdout/stderr included in events).
fn is_full_level( cli : &CliArgs ) -> bool
{
  cli.journal.as_deref().unwrap_or( "full" ) == "full"
}

// Emit an Execution event (success or error path).
//
// `fallback_message` overrides `cli.message` in the journal when BUG-327's one-shot
// fallback substitution fired for this attempt — Some(FALLBACK_MESSAGE) once set, else None.
fn emit_execution(
  writer           : Option< &JournalWriter >,
  cli              : &CliArgs,
  stdout           : &str,
  stderr           : &str,
  exit_code        : i32,
  fallback_message : Option< &str >,
)
{
  let mut ev               = EventRecord::new( EventType::Execution );
  ev.fields.exit_code      = Some( exit_code );
  ev.fields.message         = fallback_message.map( ToOwned::to_owned ).or_else( || cli.message.clone() );
  ev.fields.dir.clone_from( &cli.dir );
  ev.fields.model.clone_from( &cli.model );
  ev.fields.timeout_secs   = cli.timeout;
  ev.fields.output_style.clone_from( &cli.output_style );
  ev.fields.output_format.clone_from( &cli.output_format );
  if is_full_level( cli )
  {
    const TRUNCATE : usize = 1_048_576;
    const MARKER   : &str  = "\n[truncated at 1MB]";
    if !stdout.is_empty()
    {
      let mut s : String = stdout.chars().take( TRUNCATE ).collect();
      if stdout.chars().count() > TRUNCATE { s.push_str( MARKER ); }
      ev.fields.stdout = Some( s );
    }
    if !stderr.is_empty()
    {
      let mut s : String = stderr.chars().take( TRUNCATE ).collect();
      if stderr.chars().count() > TRUNCATE { s.push_str( MARKER ); }
      ev.fields.stderr = Some( s );
    }
  }
  emit( writer, &ev );
}

// Emit a Timeout event.
fn emit_timeout(
  writer       : Option< &JournalWriter >,
  cli          : &CliArgs,
  timeout_secs : u32,
)
{
  let mut ev             = EventRecord::new( EventType::Timeout );
  ev.fields.exit_code    = Some( 4 );
  ev.fields.timeout_secs = Some( timeout_secs );
  ev.fields.message.clone_from( &cli.message );
  emit( writer, &ev );
}

// Emit a Retry event.
fn emit_retry(
  writer        : Option< &JournalWriter >,
  error_class   : &str,
  attempt       : u32,
  limit         : u32,
  delay_secs    : u32,
  error_message : &str,
)
{
  let mut ev              = EventRecord::new( EventType::Retry );
  ev.fields.error_class   = Some( error_class.to_string() );
  ev.fields.attempt       = Some( attempt );
  ev.fields.limit         = Some( limit );
  ev.fields.delay_secs    = Some( delay_secs );
  ev.fields.error_message = Some( error_message.to_string() );
  emit( writer, &ev );
}

// Emit a ValidationRetry event.
fn emit_validation_retry(
  writer     : Option< &JournalWriter >,
  attempt    : u32,
  limit      : u32,
  delay_secs : u32,
  msg        : &str,
)
{
  let mut ev              = EventRecord::new( EventType::ValidationRetry );
  ev.fields.attempt       = Some( attempt );
  ev.fields.limit         = Some( limit );
  ev.fields.delay_secs    = Some( delay_secs );
  ev.fields.error_message = Some( msg.to_string() );
  emit( writer, &ev );
}

// Emit an Interactive event.
fn emit_interactive(
  writer    : Option< &JournalWriter >,
  cli       : &CliArgs,
  exit_code : i32,
)
{
  let mut ev           = EventRecord::new( EventType::Interactive );
  ev.fields.exit_code  = Some( exit_code );
  ev.fields.message.clone_from( &cli.message );
  ev.fields.dir.clone_from( &cli.dir );
  ev.fields.model.clone_from( &cli.model );
  emit( writer, &ev );
}

// -------------------------------------------------------------------

// Return a user-facing error message for a spawn `io::Error`.
//
// Distinguishes the common "not found" case (claude not installed) from other OS errors
// so callers can surface an actionable install hint without duplicating the check.
// Fix(BUG-298): prepend `[Runner]` to both branches of `spawn_error_msg`.
// Root cause: both branches returned bare strings without the class prefix, so the error
//   classification display showed no class label — the `[Runner]` marker was present in
//   `classify_error()` logic but never inserted into the human-facing message.
// Pitfall: spawn errors bypass the normal `ExecutionOutput`-based classification path;
//   the `[Runner]` label must be injected here, at message construction time.
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
///
/// When `use_summary` is true and stdout looks like a JSON envelope, extracts the
/// `"result"` field first so retry diagnostics show human-readable text rather than
/// the raw JSON blob.
fn first_message( output : &ExecutionOutput, class : ErrorClass, use_summary : bool ) -> String
{
  if use_summary && output.stdout.trim_start().starts_with( '{' )
  {
    if let Some( text ) = super::summary::extract_result_text( &output.stdout )
    {
      for line in text.lines()
      {
        let t = line.trim();
        if !t.is_empty() { return t.to_string(); }
      }
    }
  }
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
fn apply_expect_validation(
  cli     : &CliArgs,
  builder : &ClaudeCommand,
  out     : String,
  journal : Option< &JournalWriter >,
) -> String
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
        emit_validation_retry(
          journal,
          u32::try_from( attempt ).unwrap_or( u32::MAX ),
          u32::try_from( retries + 1 ).unwrap_or( u32::MAX ),
          delay,
          &msg,
        );
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
/// When `timeout_secs > 0`, `spawn_piped()` + `try_wait()` polling is used with background
/// reader threads draining stdout/stderr to prevent pipe-buffer deadlock when the subprocess
/// produces more than the kernel pipe buffer (64 KiB on Linux).
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

  // Timeout path: spawn with piped I/O, drain pipes in background threads, and poll
  // for exit with try_wait().
  //
  // Pitfall (pipe deadlock): without background draining, a subprocess that writes more
  //   than the kernel pipe buffer (64 KiB) blocks on write.  try_wait() then never
  //   returns Some(_), and the test hangs until timeout fires.  Background threads prevent
  //   this by continuously consuming the pipe, allowing the subprocess to run to completion.
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

  // Take pipe handles before entering the poll loop so background threads own them.
  let stdout_pipe = child.stdout.take().expect( "stdout piped by spawn_piped" );
  let stderr_pipe = child.stderr.take().expect( "stderr piped by spawn_piped" );

  let stdout_t = std::thread::spawn( move || -> Vec< u8 >
  {
    use std::io::Read as _;
    let mut buf = Vec::new();
    let _ = { let mut r = stdout_pipe; r.read_to_end( &mut buf ) };
    buf
  } );
  let stderr_t = std::thread::spawn( move || -> Vec< u8 >
  {
    use std::io::Read as _;
    let mut buf = Vec::new();
    let _ = { let mut r = stderr_pipe; r.read_to_end( &mut buf ) };
    buf
  } );

  let deadline = std::time::Instant::now()
    + core::time::Duration::from_secs( u64::from( timeout_secs ) );

  loop
  {
    match child.try_wait()
    {
      Ok( Some( status ) ) =>
      {
        // Subprocess exited: join reader threads to collect remaining buffered data.
        let stdout_bytes = stdout_t.join().unwrap_or_default();
        let stderr_bytes = stderr_t.join().unwrap_or_default();
        let exit_code = signal_exit_code( &status );
        let stdout = String::from_utf8_lossy( &stdout_bytes ).to_string();
        let stderr = String::from_utf8_lossy( &stderr_bytes ).to_string();
        return Ok( ExecutionOutput { stdout, stderr, exit_code } );
      }
      Ok( None ) =>
      {
        if check_timeout( &mut child, deadline )
        {
          // Drop reader threads — do NOT join.  Shell child processes (e.g. `sleep`)
          // inherit the pipe write end; joining would block until they exit too.
          // Dropped JoinHandles are detached; threads complete when the pipe closes.
          drop( stdout_t );
          drop( stderr_t );
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
pub( super ) fn apply_runner_retry(
  cli     : &CliArgs,
  err     : &std::io::Error,
  attempt : &mut u32,
  journal : Option< &JournalWriter >,
)
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
    // Emit RunnerRetry event.
    {
      let mut ev              = EventRecord::new( EventType::RunnerRetry );
      ev.fields.attempt       = Some( *attempt );
      ev.fields.limit         = Some( u32::from( count ) + 1 );
      ev.fields.delay_secs    = Some( delay );
      ev.fields.error_message = Some( msg.clone() );
      emit( journal, &ev );
    }
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

/// Diagnostic Claude Code prints when a resumed session's deferred-tool marker is stale
/// (tool already ran) or falls outside the tail-scan window (BUG-327).
/// Resending the original message reproduces the same stale-marker state, so this class
/// cannot recover via the standard same-message retry path — see `run_print_mode()`.
const DEFERRED_TOOL_MARKER : &str = "No deferred tool marker found in the resumed session";

/// One-shot substitute message sent when `DEFERRED_TOOL_MARKER` fires (BUG-327).
const FALLBACK_MESSAGE : &str = "Continue.";

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
#[ allow( clippy::too_many_lines ) ] // retry orchestration — per-class attempt counters, delay logic, and exit handling in one coherent loop
pub( super ) fn run_print_mode(
  builder             : &ClaudeCommand,
  cli                 : &CliArgs,
  journal             : Option< &JournalWriter >,
  expected_session_id : Option< &SessionId >,
)
{
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
  // Fix(BUG-327): one-shot fallback builder — Some(..) once the deferred-tool marker has
  // fired, substituting FALLBACK_MESSAGE for the original message on the next attempt.
  let mut fallback_builder : Option< ClaudeCommand > = None;

  loop
  {
    let active = fallback_builder.as_ref().unwrap_or( builder );
    // Fix(BUG-327): journaled message must reflect the fallback substitution, not the
    // original cli.message, once fallback_builder is set — see emit_execution().
    let fallback_note = fallback_builder.is_some().then_some( FALLBACK_MESSAGE );
    // Fix(BUG-240): spawn errors always emitted regardless of verbosity (inside execute_print_attempt).
    // Root cause: Err(e) branch was guarded by verbosity check; verbosity 0 swallowed fatal spawn errors.
    // Pitfall: verbosity gates diagnostics only — fatal errors must surface regardless of verbosity level.
    // Fix(BUG-299): handle Err(spawn_err) from execute_print_attempt() via apply_runner_retry().
    let output = match execute_print_attempt( active, timeout_secs )
    {
      Ok( o )  => o,
      Err( e ) =>
      {
        apply_runner_retry( cli, &e, &mut runner_attempt, journal );
        continue;
      }
    };

    // Fix(BUG-317): suppress double-emission of CLR-synthesized timeout label.
    // Root cause: execute_print_attempt() stores the timeout label in output.stderr (exit 4);
    //   unconditional forward fires before the retry formatter, which also surfaces the same
    //   string via first_message(), concatenating bare "timeout after Ns" before "[Process]"
    //   with no newline separator — garbling every retry and exhaustion line on timeout.
    // Pitfall: storing CLR-synthesized strings in output.stderr violates the field's implicit
    //   contract (subprocess-originated only); gate on exit_code != 4 distinguishes them.
    // Save stderr before the eprint! so we can include it in journal events.
    let raw_stderr = output.stderr.clone();
    if !output.stderr.is_empty() && output.exit_code != 4
    {
      eprint!( "{}", output.stderr );
    }

    if output.exit_code == 4
    {
      emit_timeout( journal, cli, timeout_secs );
    }

    if output.exit_code != 0
    {
      // Fix(BUG-327): resumed session with a stale/out-of-window deferred-tool marker.
      // Root cause: classify_error() has no pattern for this diagnostic, so it fell through
      //   to the generic Unknown class and retried with the *same* message — which reproduces
      //   the same stale-marker state forever. One-shot substitution of FALLBACK_MESSAGE breaks the loop.
      // Pitfall: checked before classify_error() and gated on fallback_builder.is_none() so it
      //   can only fire once per run — a subprocess that keeps emitting the marker still falls
      //   through to the bounded classify_error()/retry-count path on the next iteration.
      if fallback_builder.is_none()
        && ( output.stdout.contains( DEFERRED_TOOL_MARKER ) || output.stderr.contains( DEFERRED_TOOL_MARKER ) )
      {
        if !cli.quiet
        {
          eprintln!( "[Runner] deferred tool marker not found in resumed session — retrying with fallback prompt…" );
        }
        emit_retry( journal, "FallbackPrompt", 1, 2, 0, "deferred tool marker not found in resumed session" );
        let fallback = active.clone().with_message( FALLBACK_MESSAGE.to_string() );
        fallback_builder = Some( fallback );
        continue;
      }

      // Fix(BUG-037): classify non-zero exit for labeled diagnostic.
      // Root cause: no error classification existed; all non-zero exits produced identical log output.
      // Pitfall: classify_error() scans stdout AND stderr — rate-limit reason may be in stdout.
      let kind = output.classify_error();
      let class = classify_to_class( kind.as_ref(), output.exit_code );
      // Fix(BUG-325): Auth uses the same 3-tier retry resolution as all other classes.
      // Root cause: BUG-315 introduced `!is_auth_error` guard that blocked retry-block
      //   entry unconditionally, making `--retry-on-auth` a dead parameter.
      // Pitfall: never use `break` inside this retry block — break exits the loop and
      //   falls through to the function's implicit return (), bypassing the
      //   process::exit(exit_code) call below. Guard the block ENTRY instead.
      let class_idx = class as usize;
      let label = class.label();
      let ( count_field, delay_field ) = class_fields( cli, class );
      let limit = resolve_count( cli.retry_override, count_field, cli.retry_default ) as usize;
      let delay = resolve_delay( cli.retry_override_delay, delay_field, cli.retry_default_delay );
      let use_summary = cli.output_style.as_deref().unwrap_or( "summary" ) == "summary";
      let msg = first_message( &output, class, use_summary );

      if attempts[ class_idx ] < limit
      {
        attempts[ class_idx ] += 1;
        if !cli.quiet
        {
          let suf = delay_suffix( delay );
          eprintln!(
            "[{label}] {msg} — retrying{suf} (attempt {}/{})…",
            attempts[ class_idx ],
            limit + 1
          );
        }
        emit_retry( journal, label, u32::try_from( attempts[ class_idx ] ).unwrap_or( u32::MAX ), u32::try_from( limit + 1 ).unwrap_or( u32::MAX ), delay, &msg );
        if delay > 0
        {
          std::thread::sleep( core::time::Duration::from_secs( u64::from( delay ) ) );
        }
        continue;
      }

      // Non-retriable error or retries exhausted.
      if !cli.quiet
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
      if !output.stdout.is_empty()
      {
        let rendered = if use_summary
        {
          super::summary::render_summary( &output.stdout, cli.summary_fields.as_deref() )
            .unwrap_or_else( || output.stdout.clone() )
        }
        else
        {
          output.stdout.clone()
        };
        if !rendered.is_empty() { eprint!( "{rendered}" ); }
      }
      emit_execution( journal, cli, &output.stdout, &raw_stderr, output.exit_code, fallback_note );
      std::process::exit( output.exit_code );
    }

    // Success path — expect validation, file write, stdout.
    let raw_stdout = output.stdout.clone();
    let out = if cli.strip_fences { strip_fences( &output.stdout ) } else { output.stdout };
    // summary format: intercept JSON output, render key:val header + text body.
    // Falls back to raw output when JSON cannot be parsed.
    let out = if cli.output_style.as_deref().unwrap_or( "summary" ) == "summary"
    {
      super::summary::render_summary( &out, cli.summary_fields.as_deref() ).unwrap_or( out )
    }
    else if cli.json_schema.is_some()
    {
      // Fix(BUG-318): raw mode + --json-schema produced empty stdout because claude returns
      //   an empty "result" field for structured responses; the value lives in "structured_output".
      // Root cause: the raw else-branch passed through stdout unchanged, returning the empty
      //   "result" text instead of extracting "structured_output" from the JSON envelope.
      // Pitfall: only activate when json_schema is present; unconditional extraction would
      //   silently drop output for non-structured responses that have no structured_output field.
      super::summary::extract_structured_output( &out ).unwrap_or( out )
    }
    else
    {
      out
    };
    let out = apply_expect_validation( cli, active, out, journal );
    emit_execution( journal, cli, &raw_stdout, &raw_stderr, 0, fallback_note );
    // Fix(BUG-320): detect session mismatch — warn when claude resumed a different session.
    // Root cause: without UUID comparison, silent session drift goes unnoticed; callers may
    //   believe they are continuing a specific conversation but get a different one instead.
    // Pitfall: non-fatal — emit warning to stderr but exit 0 so callers are not disrupted.
    //   Only fires in print mode (interactive TTY output cannot be reliably parsed for UUID).
    if let Some( expected ) = expected_session_id
    {
      if let Some( actual ) = super::summary::extract_session_id( &raw_stdout )
      {
        if actual != expected.as_str()
        {
          eprintln!(
            "[Runner] warning: session mismatch — expected {expected}, got {actual} (BUG-320 detected)"
          );
        }
      }
    }
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
pub( super ) fn run_interactive(
  builder : &ClaudeCommand,
  cli     : &CliArgs,
  journal : Option< &JournalWriter >,
)
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
    let code = signal_exit_code( &status );
    emit_interactive( journal, cli, code );
    if !status.success()
    {
      std::process::exit( code );
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
        let code = signal_exit_code( &status );
        emit_interactive( journal, cli, code );
        if !status.success()
        {
          std::process::exit( code );
        }
        return;
      }
      Ok( None ) =>
      {
        if check_timeout( &mut child, deadline )
        {
          emit_timeout( journal, cli, timeout_secs );
          eprintln!( "Error: timeout after {timeout_secs}s" );
          std::process::exit( 4 );
        }
      }
      Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
    }
  }
}
