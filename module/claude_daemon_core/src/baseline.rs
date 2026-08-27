//! Measuring what a conversation costs before a word of it has been said.
//!
//! [`crate::context`] reports how full a session is — `tokens.context`, the
//! newest call's whole billed prompt. What that figure will not say is how much
//! of it was already spent before the conversation started: the system prompt,
//! the tool definitions, the skill and agent rosters. A session occupying 30k
//! tokens is in a very different position depending on whether 4k or 24k of that
//! is fixed overhead, and the transcript cannot tell the two apart.
//!
//! Nothing in the transcript can, because the overhead is charged identically on
//! every call and so never varies within a session. Separating it needs a
//! conversation with *no* conversation in it, which is what a probe is: one
//! `--print` call carrying a two-letter prompt, whose billed prompt is therefore
//! almost entirely the floor.
//!
//! # Why this is not automatic, and not a daemon request
//!
//! A probe spends real tokens against the user's account. So nothing here runs
//! on its own — a measurement happens when a caller asks for one, and until then
//! the summary reports the overhead as `null` rather than guessing at it. The
//! floor moves only when Claude Code's version or the model changes, so a
//! measurement is cached against both and re-taken rarely.
//!
//! There is deliberately no `measure` request in the wire protocol either. The
//! daemon is single-threaded and serves one request at a time, so a probe run
//! inside it would freeze every hosted session for a full API round trip. It is
//! also unnecessary: a probe needs a `claude` binary and nothing the daemon
//! holds, so whoever knows where that binary is can call [`measure`] and
//! [`store`] directly. The daemon's half is the cheap one — [`load`], on a
//! request that is already reading files.
//!
//! ```rust,ignore
//! let measured = baseline::measure( &claude_path, &version, None )?;
//! baseline::store( paths.runtime_dir(), &measured )?;
//! ```
//!
//! # Why the probe sums three fields
//!
//! `usage.input_tokens` counts *uncached* input only. The static prompt is
//! identical on every call and so is exactly what prompt caching captures —
//! most of it arrives as `cache_read_input_tokens` or
//! `cache_creation_input_tokens` instead. Reading `input_tokens` alone would put
//! the floor at a few hundred tokens and would swing by an order of magnitude
//! between a cold and a warm cache. The sum is stable across both.

use std::collections::BTreeMap;
use std::path::{ Path, PathBuf };
use std::process::Command;

use serde::{ Deserialize, Serialize };

use crate::{ Error, Result };

/// Name of the cache file inside the daemon's runtime directory.
pub const BASELINE_FILE_NAME : &str = "baseline.json";

/// The prompt a probe sends.
///
/// Short enough that its own cost is negligible against the floor being
/// measured, and inert enough that no tool call or file read can be provoked by
/// it — a probe that did work would measure that work too.
pub const PROBE_PROMPT : &str = "hi";

/// What one conversation costs before anything has been said in it.
#[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
#[ non_exhaustive ]
pub struct StaticBaseline
{
  /// Claude Code version this measurement was taken against.
  pub version : String,
  /// Model id this measurement was taken against, as the response reported it.
  pub model : String,
  /// The floor: `input + cache_read + cache_creation` for the probe's one call.
  ///
  /// Comparable directly against `tokens.context` in a context summary, which is
  /// computed the same way.
  pub prompt_tokens : u64,
  /// The uncached part of [`Self::prompt_tokens`].
  pub input_tokens : u64,
  /// The part of [`Self::prompt_tokens`] served from an existing cache entry.
  pub cache_read_tokens : u64,
  /// The part of [`Self::prompt_tokens`] written into a new cache entry.
  pub cache_creation_tokens : u64,
}

impl StaticBaseline
{
  /// How much of `context` is the conversation rather than the fixed floor.
  ///
  /// Saturating: a `context` below the floor is not a negative conversation, it
  /// is a stale baseline — the floor moved and this measurement predates it.
  #[ inline ]
  #[ must_use ]
  pub fn conversation_tokens( &self, context : u64 ) -> u64
  {
    context.saturating_sub( self.prompt_tokens )
  }
}

/// The argument list a probe runs.
///
/// Split out from [`measure`] so the flags are assertable without spending a
/// call to see them. Every one of them is load-bearing:
///
/// - `--print` — answer and exit, rather than opening an interactive session.
/// - `--output-format json` — the usage object is the entire point.
/// - `--no-session-persistence` — a probe is not a conversation anyone will
///   resume, and leaving one in the session registry would put a session in
///   `list_sessions` that no client asked for.
#[ inline ]
#[ must_use ]
pub fn probe_args( model : Option< &str > ) -> Vec< String >
{
  let mut args : Vec< String > = [ "--print", "--output-format", "json", "--no-session-persistence" ]
    .iter()
    .map( | one | ( *one ).to_string() )
    .collect();

  if let Some( model ) = model
  {
    args.push( "--model".to_string() );
    args.push( model.to_string() );
  }

  args.push( PROBE_PROMPT.to_string() );
  args
}

/// Read a probe's response into a baseline.
///
/// `version` is supplied rather than read from the response, which does not
/// carry one — it is what the measurement is keyed against, so a caller that
/// cannot name the version it ran has nothing to cache.
///
/// # Errors
///
/// Returns [`Error::Probe`] when the response is not JSON, carries no `usage`,
/// or names no model. A probe that ran but answered something unrecognisable is
/// a failure to measure, not a measurement of zero.
#[ inline ]
pub fn parse_probe( version : &str, stdout : &str ) -> Result< StaticBaseline >
{
  let response : serde_json::Value = serde_json::from_str( stdout )
    .map_err( | error | Error::Probe { reason : format!( "response was not JSON: {error}" ) } )?;

  let usage = response.get( "usage" ).ok_or_else( || Error::Probe
  {
    reason : "response carried no `usage` object".to_string(),
  } )?;

  let model = response.get( "model" ).and_then( serde_json::Value::as_str ).ok_or_else( || Error::Probe
  {
    reason : "response named no model, so the measurement could not be keyed".to_string(),
  } )?;

  // Absent cache fields mean zero — they are omitted entirely when caching did
  // not apply, which is a real measurement rather than a missing one. An absent
  // `input_tokens` is different, and is what the check below rejects.
  let field = | name : &str | usage.get( name ).and_then( serde_json::Value::as_u64 );

  let input_tokens = field( "input_tokens" ).ok_or_else( || Error::Probe
  {
    reason : "`usage` carried no `input_tokens`".to_string(),
  } )?;
  let cache_read_tokens = field( "cache_read_input_tokens" ).unwrap_or( 0 );
  let cache_creation_tokens = field( "cache_creation_input_tokens" ).unwrap_or( 0 );

  Ok( StaticBaseline
  {
    version : version.to_string(),
    model : model.to_string(),
    // Saturating for the same reason the two checks above exist: this parses
    // whatever `claude` printed, and a debug build would panic on a response
    // carrying absurd counts rather than reporting an unusable measurement.
    prompt_tokens : input_tokens
      .saturating_add( cache_read_tokens )
      .saturating_add( cache_creation_tokens ),
    input_tokens,
    cache_read_tokens,
    cache_creation_tokens,
  } )
}

/// Run one probe and read the floor off it.
///
/// Spends a real API call against the caller's account — see the module note on
/// why nothing calls this on its own.
///
/// # Errors
///
/// Returns [`Error::Io`] when `claude` cannot be run at all, and
/// [`Error::Probe`] when it runs but fails, or answers something this cannot
/// read. A non-zero exit carries its stderr, since that is where `claude`
/// explains an authentication or quota failure.
#[ inline ]
pub fn measure( claude : &Path, version : &str, model : Option< &str > ) -> Result< StaticBaseline >
{
  let output = Command::new( claude ).args( probe_args( model ) ).output()?;

  if !output.status.success()
  {
    return Err( Error::Probe
    {
      reason : format!
      (
        "`claude` exited {}: {}",
        output.status.code().map_or_else( || "on a signal".to_string(), | code | code.to_string() ),
        String::from_utf8_lossy( &output.stderr ).trim(),
      ),
    } );
  }

  parse_probe( version, &String::from_utf8_lossy( &output.stdout ) )
}

/// Path of the cache file inside `runtime_dir`.
#[ inline ]
#[ must_use ]
pub fn cache_path( runtime_dir : &Path ) -> PathBuf
{
  runtime_dir.join( BASELINE_FILE_NAME )
}

/// Key a measurement is stored under.
///
/// Both halves matter: the floor moves when Claude Code changes what it puts in
/// the system prompt, and it differs between models whose tool encodings differ.
fn key( version : &str, model : &str ) -> String
{
  format!( "{version}/{model}" )
}

/// Every measurement currently cached, keyed by version and model.
///
/// A cache that cannot be read or parsed yields an empty map rather than an
/// error: the file is a memo, and a corrupt one should cost a re-measurement,
/// never a failed request.
#[ inline ]
#[ must_use ]
pub fn load_all( runtime_dir : &Path ) -> BTreeMap< String, StaticBaseline >
{
  std::fs::read_to_string( cache_path( runtime_dir ) )
    .ok()
    .and_then( | body | serde_json::from_str( &body ).ok() )
    .unwrap_or_default()
}

/// The cached measurement for `version` and `model`, if one has been taken.
#[ inline ]
#[ must_use ]
pub fn load( runtime_dir : &Path, version : &str, model : &str ) -> Option< StaticBaseline >
{
  load_all( runtime_dir ).remove( &key( version, model ) )
}

/// Cache `baseline`, replacing any earlier measurement of the same version and model.
///
/// # Errors
///
/// Returns [`Error::Io`] when the runtime directory cannot be created or the
/// file cannot be written.
#[ inline ]
pub fn store( runtime_dir : &Path, baseline : &StaticBaseline ) -> Result< () >
{
  let mut all = load_all( runtime_dir );
  all.insert( key( &baseline.version, &baseline.model ), baseline.clone() );

  std::fs::create_dir_all( runtime_dir )?;

  let body = serde_json::to_string_pretty( &all )
    .map_err( | error | Error::Probe { reason : format!( "cache would not serialize: {error}" ) } )?;

  std::fs::write( cache_path( runtime_dir ), body )?;
  Ok( () )
}
