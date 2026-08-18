//! `clr topic` — an auto-named `run`/`ask` alias.
//!
//! When `--subdir` is not explicitly given, a directory-name slug is derived from the
//! message and injected as `--subdir` (disambiguated with a `-2`, `-3`, ... counter
//! suffix against what already exists on disk), then delegation proceeds through
//! `dispatch_run()` exactly like `dispatch_ask()` does. An explicit `--subdir` bypasses
//! slug generation entirely — `clr topic --subdir NAME "msg"` is then byte-identical to
//! `clr ask --subdir NAME "msg"` (see IT-3, `tests/docs/cli/command/11_topic.md`).
//!
//! No new session-management logic lives here: the existing `--subdir`+`--from`
//! clone/continue mechanism in `builder.rs` already handles first-call-clones and
//! repeat-call-continues generically for any `--subdir` value, whether the user
//! supplied it directly or `dispatch_topic` injected it (task 521 Out of Scope).

use super::dispatch_run;
use super::help::print_topic_help;
use super::parse::parse_args;

/// Longest slug `slug_from_message` will produce before cutting back to a whole-word
/// boundary. Chosen as a reasonable directory-name length — recorded here as the
/// single source of truth for task 521's M1 measurement (concrete value: 40).
const MAX_SLUG_LEN : usize = 40;

/// Derive a directory-name slug from a message: lowercase ASCII alphanumerics kept,
/// every run of other characters collapsed to a single hyphen, leading/trailing
/// hyphens trimmed. When the raw slug exceeds `MAX_SLUG_LEN`, it is cut back to the
/// nearest whole-word boundary at or under the limit (never a mid-word truncation).
///
/// Returns `None` when the message yields no usable characters (e.g. an empty
/// message, or one made entirely of punctuation) — the caller then leaves `--subdir`
/// unset, falling back to plain `ask`/`run` behavior for that message.
fn slug_from_message( msg : &str ) -> Option< String >
{
  let mut slug = String::new();
  let mut last_was_sep = true; // suppress a leading hyphen
  for ch in msg.chars()
  {
    if ch.is_ascii_alphanumeric()
    {
      slug.push( ch.to_ascii_lowercase() );
      last_was_sep = false;
    }
    else if !last_was_sep
    {
      slug.push( '-' );
      last_was_sep = true;
    }
  }
  while slug.ends_with( '-' ) { slug.pop(); }

  if slug.len() > MAX_SLUG_LEN
  {
    slug.truncate( MAX_SLUG_LEN );
    if let Some( last_hyphen ) = slug.rfind( '-' )
    {
      slug.truncate( last_hyphen );
    }
    while slug.ends_with( '-' ) { slug.pop(); }
  }

  if slug.is_empty() { None } else { Some( slug ) }
}

/// Disambiguate `slug` against `base`: return it unchanged when `base/-{slug}` does
/// not yet exist on disk; otherwise append `-2`, `-3`, ... until a free name is found.
///
/// Mirrors `resolve_effective_dir()`'s own `<base>/-<sub>` join formula
/// (`src/cli/builder.rs`) so the existence check matches exactly what the eventual
/// effective working directory will be.
fn disambiguate_slug( base : &std::path::Path, slug : &str ) -> String
{
  if !base.join( format!( "-{slug}" ) ).exists()
  {
    return slug.to_string();
  }
  let mut n : u32 = 2;
  loop
  {
    let candidate = format!( "{slug}-{n}" );
    if !base.join( format!( "-{candidate}" ) ).exists()
    {
      return candidate;
    }
    n += 1;
  }
}

/// Parse, validate, and execute the `topic` subcommand. Never returns.
///
/// `topic` behaves exactly like `run`/`ask`, with one addition: when `--subdir` is
/// not explicitly given, a slug generated from the message is injected as `--subdir`
/// before delegating. An explicit `--subdir` disables slug generation entirely.
pub( crate ) fn dispatch_topic( tokens : &[ String ] ) -> !
{
  // tokens[0] == "topic"
  // Fix(BUG-249 pattern): 'clr topic help' must show topic help, not treat "help" as
  // a message — every subcommand dispatcher that delegates to dispatch_run must
  // repeat this positional-help intercept independently (see dispatch_ask).
  if tokens.get( 1 ).map( String::as_str ) == Some( "help" )
  {
    print_topic_help();
  }

  let cli = match parse_args( &tokens[ 1 .. ] )
  {
    Ok( c )  => c,
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
  };

  if cli.help
  {
    print_topic_help();
  }

  // Explicit --subdir: byte-identical to `ask` (IT-3) — no slug injection.
  if cli.subdir.is_some()
  {
    dispatch_run( &tokens[ 1 .. ] );
  }

  // No message to derive a slug from: fall back to plain ask/run behavior.
  let Some( slug ) = cli.message.as_deref().and_then( slug_from_message ) else
  {
    dispatch_run( &tokens[ 1 .. ] );
  };

  let base = match cli.dir.as_deref()
  {
    Some( d ) => std::path::PathBuf::from( d ),
    None      => std::env::current_dir().unwrap_or_else( | _ | std::path::PathBuf::from( "." ) ),
  };
  let final_slug = disambiguate_slug( &base, &slug );

  let mut injected : Vec< String > = tokens[ 1 .. ].to_vec();
  injected.push( "--subdir".to_string() );
  injected.push( final_slug );
  dispatch_run( &injected );
}
