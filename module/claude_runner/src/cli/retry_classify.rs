//! Error classification and 3-tier retry-parameter resolution for the print-mode retry loop.
//!
//! Split out of `execution.rs` (which was over the line-count guideline) — this cluster of
//! types/functions has no dependents outside `execution.rs`.

use claude_runner_core::{ ErrorKind, ExecutionOutput };
use super::parse::CliArgs;

/// Semantic class for caller-facing retry decisions.
///
/// Maps `ErrorKind` (subprocess classification) and CLR-layer ad-hoc exits
/// to a uniform 6-class taxonomy for the retry loop.  Validation and Runner
/// classes are handled outside the main retry loop.
#[ derive( Clone, Copy ) ]
pub( super ) enum ErrorClass
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
  pub( super ) fn label( self ) -> &'static str
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

/// Per-class retry attempt counters, one field per [`ErrorClass`] variant.
///
/// A named-field struct (rather than a `[usize; 6]` array indexed by `class as usize`) so
/// that adding an `ErrorClass` variant forces a compile error at `get_mut()`'s non-exhaustive
/// match instead of a silent out-of-bounds panic at runtime — the two can no longer drift.
#[ derive( Default ) ]
pub( super ) struct ClassAttempts
{
  transient : usize,
  account   : usize,
  auth      : usize,
  service   : usize,
  process   : usize,
  unknown   : usize,
}

impl ClassAttempts
{
  pub( super ) fn get_mut( &mut self, class : ErrorClass ) -> &mut usize
  {
    match class
    {
      ErrorClass::Transient => &mut self.transient,
      ErrorClass::Account   => &mut self.account,
      ErrorClass::Auth      => &mut self.auth,
      ErrorClass::Service   => &mut self.service,
      ErrorClass::Process   => &mut self.process,
      ErrorClass::Unknown   => &mut self.unknown,
    }
  }
}

/// Map an `ErrorKind` (or CLR-layer exit 4) to an `ErrorClass`.
pub( super ) fn classify_to_class( kind : Option< &ErrorKind >, exit_code : i32 ) -> ErrorClass
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
pub( super ) fn resolve_count(
  over      : Option< u8 >,
  class_cli : Option< u8 >,
  fallback  : Option< u8 >,
) -> u8
{
  over.or( class_cli ).or( fallback ).unwrap_or( 2 )
}

/// 3-tier resolution for retry delay: override ?? class-specific ?? fallback (30).
pub( super ) fn resolve_delay( over : Option< u32 >, class : Option< u32 >, fallback : Option< u32 > ) -> u32
{
  over.or( class ).or( fallback ).unwrap_or( 30 )
}

/// Return the class-specific (count, delay) fields from `CliArgs` for the given class.
pub( super ) fn class_fields( cli : &CliArgs, class : ErrorClass ) -> ( Option< u8 >, Option< u32 > )
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
pub( super ) fn first_message( output : &ExecutionOutput, class : ErrorClass, use_summary : bool ) -> String
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
pub( super ) fn delay_suffix( delay : u32 ) -> String
{
  if delay > 0 { format!( " in {delay}s" ) } else { String::new() }
}
