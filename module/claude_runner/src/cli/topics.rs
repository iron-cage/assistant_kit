//! `clr topics` — list topic directories, or resolve one topic name to its absolute path.
//!
//! Read-only counterpart to `clr topic`: `topic` creates and enters a topic session,
//! `topics` reports on them without running anything. Both compute paths through the
//! same `topic_path` helpers, so a path printed here is exactly the directory `topic`
//! would use for that name.
//!
//! `--path NAME` is a pure computation (`<base>/-<name>`) that never touches the disk —
//! it answers "where would this topic live?" identically whether or not it exists, which
//! is what makes a global topic recoverable from its name alone in a later shell.

use super::help::print_topics_help;
use super::topic_path::{ topic_base, topic_dir, topic_name_of };

/// One topic directory found under the base, with the session count read from its own
/// Claude Code storage.
struct TopicEntry
{
  name     : String,
  path     : std::path::PathBuf,
  sessions : usize,
}

/// Count `*.jsonl` session files in `dir`'s own Claude Code session storage.
///
/// Returns 0 for a topic that exists but has never been entered — the session directory
/// is created by Claude Code on first run, not by `resolve_effective_dir`'s `create_dir_all`.
fn session_count( dir : &std::path::Path ) -> usize
{
  let scope = claude_storage_core::scope_for( dir );
  let Ok( entries ) = std::fs::read_dir( &scope.claude_session_dir ) else { return 0; };
  entries
    .filter_map( Result::ok )
    .filter( | e | e.path().extension().is_some_and( | x | x == "jsonl" ) )
    .count()
}

/// Collect every topic directory directly under `base`, sorted by name.
///
/// A non-existent or unreadable base yields an empty list rather than an error: the
/// global topic home legitimately does not exist until the first global topic is created.
fn collect_topics( base : &std::path::Path ) -> Vec< TopicEntry >
{
  let Ok( entries ) = std::fs::read_dir( base ) else { return Vec::new(); };
  let mut found : Vec< TopicEntry > = entries
    .filter_map( Result::ok )
    .filter( | e | e.path().is_dir() )
    .filter_map( | e |
    {
      let file_name = e.file_name();
      let raw = file_name.to_str()?;
      let name = topic_name_of( raw )?.to_string();
      let path = e.path();
      let sessions = session_count( &path );
      Some( TopicEntry { name, path, sessions } )
    } )
    .collect();
  found.sort_by( | a, b | a.name.cmp( &b.name ) );
  found
}

/// Parse, validate, and execute the `topics` subcommand. Never returns.
///
/// Two forms: `--path NAME` resolves one name to its absolute path and exits; otherwise
/// every topic under the resolved base is listed.
pub( crate ) fn dispatch_topics( tokens : &[ String ] ) -> !
{
  // tokens[0] == "topics"
  // Fix(BUG-249 pattern): bare positional `help` must print help, not be parsed as a value —
  // every subcommand dispatcher repeats this intercept independently (see dispatch_topic).
  if tokens.get( 1 ).map( String::as_str ) == Some( "help" )
  {
    print_topics_help();
  }

  let mut dir_arg  : Option< String > = None;
  let mut path_arg : Option< String > = None;
  let mut global   = false;
  let mut i = 1_usize;
  while i < tokens.len()
  {
    match tokens[ i ].as_str()
    {
      "--help" | "-h" => print_topics_help(),
      "--global" | "-g" =>
      {
        global = true;
        i += 1;
      }
      "--dir" | "--to" =>
      {
        let Some( val ) = tokens.get( i + 1 ) else
        {
          eprintln!( "Error: {} requires a value\nRun with --help for usage.", tokens[ i ] );
          std::process::exit( 1 );
        };
        dir_arg = Some( val.clone() );
        i += 2;
      }
      "--path" =>
      {
        let Some( val ) = tokens.get( i + 1 ) else
        {
          eprintln!( "Error: --path requires a value\nRun with --help for usage." );
          std::process::exit( 1 );
        };
        // Mirrors --topic's own single-component guard (BUG-230): a topic name is a
        // directory name component, never a path, so `/` can never appear in one.
        if val.contains( '/' )
        {
          eprintln!( "Error: --path must be a single topic name (no '/' separators)" );
          std::process::exit( 1 );
        }
        path_arg = Some( val.clone() );
        i += 2;
      }
      other =>
      {
        eprintln!( "Error: unknown option '{other}'\nRun with --help for usage." );
        std::process::exit( 1 );
      }
    }
  }

  let base = topic_base( dir_arg.as_deref(), global );

  // Resolve form: pure computation, no filesystem access, always exits 0.
  if let Some( name ) = path_arg
  {
    println!( "{}", topic_dir( &base, &name ).display() );
    std::process::exit( 0 );
  }

  // List form.
  let topics = collect_topics( &base );
  if topics.is_empty()
  {
    eprintln!( "no topics in {}", base.display() );
    std::process::exit( 0 );
  }

  let name_width = topics.iter().map( | t | t.name.len() ).max().unwrap_or( 4 ).max( 4 );
  println!( "{:<name_width$}  {:>8}  PATH", "NAME", "SESSIONS" );
  for t in &topics
  {
    println!( "{:<name_width$}  {:>8}  {}", t.name, t.sessions, t.path.display() );
  }
  std::process::exit( 0 );
}
