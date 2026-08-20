//! `clr topic` — an auto-named `run`/`ask` alias.
//!
//! When `--topic` is not explicitly given, a directory-name slug is derived from the
//! message and injected as `--topic` (disambiguated with a `-2`, `-3`, ... counter
//! suffix against what already exists on disk), then delegation proceeds through
//! `dispatch_run()` exactly like `dispatch_ask()` does. An explicit `--topic` bypasses
//! slug generation entirely — `clr topic --topic NAME "msg"` is then byte-identical to
//! `clr ask --topic NAME "msg"` (see IT-3, `tests/docs/cli/command/11_topic.md`).
//!
//! No new session-management logic lives here: the existing `--topic`+`--from`
//! clone/continue mechanism in `builder.rs` already handles first-call-clones and
//! repeat-call-continues generically for any `--topic` value, whether the user
//! supplied it directly or `dispatch_topic` injected it (task 521 Out of Scope).

use super::dispatch_run;
use super::help::print_topic_help;
use super::parse::parse_args;
use super::topic_path::{ topic_base, topic_dir };

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
/// message, or one made entirely of punctuation) — the caller then leaves `--topic`
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

/// True when `name` is not safe to auto-assign: its working directory already exists
/// on disk, OR its session storage already holds a qualifying session.
///
/// Uses `topic_path::topic_dir` for the join, so the check is by construction the
/// same path `resolve_effective_dir()` will later compute — previously the only site
/// spelled the `<base>/-<sub>` formula out separately and was kept in sync by hand.
///
/// Fix(BUG-542): a name is "taken" when EITHER signal fires, not directory existence
/// alone — a topic's working directory can be deleted (e.g. `rm -rf`) while its
/// session storage survives untouched, since storage lives under
/// `~/.claude/projects/`, entirely independent of the working directory's own
/// filesystem lifetime. Auto-naming previously judged such a slug "fresh" from the
/// directory check alone, and `builder.rs`'s Fix(BUG-541) prefer-target's-own-storage
/// rule — deliberately authoritative once a target has ANY qualifying session — would
/// then guarantee the recreated directory silently resumed that orphaned, unrelated
/// history instead of starting fresh.
/// Root cause: freshness was judged by directory existence only; session storage was
/// never consulted despite persisting independently of the directory.
/// Pitfall: only auto-naming (this function) needs the extra probe — an explicit
/// `--topic NAME` bypasses `disambiguate_slug` entirely (`dispatch_topic`'s early
/// return) and correctly continues existing storage by name; that is intended
/// behavior, not this defect.
fn name_is_taken( base : &std::path::Path, name : &str ) -> bool
{
  let dir = topic_dir( base, name );
  if dir.exists()
  {
    return true;
  }
  let storage = claude_storage_core::scope_for( &super::builder::physical_abs( &dir ) ).claude_session_dir;
  super::builder::session_exists( &storage ).is_some()
}

/// Disambiguate `slug` against `base`: return it unchanged when `base/-{slug}` is free
/// per [`name_is_taken`]; otherwise append `-2`, `-3`, ... until a free name is found.
fn disambiguate_slug( base : &std::path::Path, slug : &str ) -> String
{
  if !name_is_taken( base, slug )
  {
    return slug.to_string();
  }
  let mut n : u32 = 2;
  loop
  {
    let candidate = format!( "{slug}-{n}" );
    if !name_is_taken( base, &candidate )
    {
      return candidate;
    }
    n += 1;
  }
}

/// Parse, validate, and execute the `topic` subcommand. Never returns.
///
/// `topic` behaves exactly like `run`/`ask`, with one addition: when `--topic` is
/// not explicitly given, a slug generated from the message is injected as `--topic`
/// before delegating. An explicit `--topic` disables slug generation entirely.
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

  // Explicit --topic: byte-identical to `ask` (IT-3) — no slug injection.
  if cli.topic.is_some()
  {
    dispatch_run( &tokens[ 1 .. ] );
  }

  // No message to derive a slug from: fall back to plain ask/run behavior.
  let Some( slug ) = cli.message.as_deref().and_then( slug_from_message ) else
  {
    dispatch_run( &tokens[ 1 .. ] );
  };

  // Same base resolution `resolve_effective_dir` will apply to the injected --topic,
  // so an auto-named global topic probes the global home rather than the cwd.
  let base = topic_base( cli.dir.as_deref(), cli.global );
  let final_slug = disambiguate_slug( &base, &slug );

  let mut injected : Vec< String > = tokens[ 1 .. ].to_vec();
  injected.push( "--topic".to_string() );
  injected.push( final_slug );
  dispatch_run( &injected );
}
