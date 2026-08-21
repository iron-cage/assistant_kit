//! CLI invocation telemetry — appends one redacted `Command` event per invocation.
//!
//! Failure isolation is the governing constraint throughout this module: a journal
//! write is observability, never load-bearing, so every fallible step degrades to a
//! best-effort default or a silently-dropped write rather than propagating an error
//! into the underlying command's own exit code.

use claude_journal::{ EventRecord, EventType, JournalWriter };
use json_redact::RedactionPolicy;

/// Resolve the journal directory: `CLR_JOURNAL_DIR` env var if set and non-empty,
/// else `~/.clr/journal`.
///
/// Mirrors the env/default tiers of `claude_journal_viewer::output::resolve_journal_dir`
/// (there is no `dir::` CLI param here for `clp` to mirror that function's third tier).
fn journal_dir() -> std::path::PathBuf
{
  if let Ok( d ) = std::env::var( "CLR_JOURNAL_DIR" )
  {
    if !d.is_empty()
    {
      return std::path::PathBuf::from( d );
    }
  }
  let home = std::env::var( "HOME" ).unwrap_or_else( | _ | "/tmp".to_owned() );
  std::path::PathBuf::from( home ).join( ".clr" ).join( "journal" )
}

/// Current OS user name, or `"unknown"` if undetectable.
///
/// Reads `$USER` only — no subprocess spawn (forbidden by this crate's own
/// architectural boundary test) and no `unsafe` FFI (denied by workspace lints).
fn current_user() -> String
{
  std::env::var( "USER" ).unwrap_or_else( | _ | "unknown".to_owned() )
}

/// Current hostname, or `"unknown"` if undetectable.
///
/// Prefers `$HOSTNAME`; falls back to reading `/proc/sys/kernel/hostname` (a plain
/// file read — no subprocess, no `unsafe` FFI). Linux-only, matching this toolkit's
/// target platform.
fn current_host() -> String
{
  std::env::var( "HOSTNAME" )
  .ok()
  .filter( | h | !h.is_empty() )
  .or_else( || std::fs::read_to_string( "/proc/sys/kernel/hostname" ).ok().map( | s | s.trim().to_owned() ) )
  .unwrap_or_else( || "unknown".to_owned() )
}

/// Redact sensitive values from `argv`, preserving individual-argument structure.
///
/// Joins `argv` with spaces (matching `json_redact::redact_str`'s whitespace-token
/// contract), redacts, then splits back on spaces.
fn redact_args( argv : &[ String ] ) -> Vec< String >
{
  if argv.is_empty()
  {
    return Vec::new();
  }
  let joined   = argv.join( " " );
  let redacted = json_redact::redact_str( &joined, &RedactionPolicy::default() );
  redacted.split( ' ' ).map( ToOwned::to_owned ).collect()
}

/// The active account name from clp's own credential store, or `None` when no
/// store root or marker resolves.
///
/// Same store the CLI commands operate on: root from `$PRO` (if a directory)
/// else `$HOME`, suffix owned by `credential_store_for_root`. The marker holds
/// only the account name — never token material (task 547).
fn active_account_name() -> Option< String >
{
  let store = claude_profile_core::account::default_credential_store()?;
  claude_profile_core::account::active_account( &store )
}

/// Append one redacted `Command` event to the journal for this invocation.
///
/// Failure isolation: any error resolving the directory or writing the event is
/// swallowed — telemetry must never change the underlying command's exit code or
/// abort it (Delivery Requirement).
///
/// Attribution (task 547): every event carries `dir` (invocation cwd),
/// `agent_id` (`{user}@{host}{abs_dir}/` via `claude_journal::compose_agent_id`,
/// absent only when the cwd is unresolvable), and `account` (the active account
/// name, absent when none resolves) — completing the attribution set so clp and
/// clr events are uniformly attributable. `agent_id` reuses the same
/// user/host/dir values stamped on the event, so all fields describe one identity.
pub( crate ) fn record( argv : &[ String ], exit_code : i32, duration_ms : u64 )
{
  let user = current_user();
  let host = current_host();
  let dir  = std::env::current_dir().ok().map( | p | p.display().to_string() );

  let mut event = EventRecord::new( EventType::Command );
  event.fields.agent_id    = dir.as_deref().map( | d | claude_journal::compose_agent_id( &user, &host, d ) );
  event.fields.dir         = dir;
  event.fields.account     = active_account_name();
  event.fields.user        = Some( user );
  event.fields.host        = Some( host );
  event.fields.args        = Some( redact_args( argv ) );
  event.fields.exit_code   = Some( exit_code );
  event.fields.duration_ms = Some( duration_ms );

  let writer = JournalWriter::new( journal_dir() );
  let _ = writer.append( &event );
}
