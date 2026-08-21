//! `clr topics` — list topics, or resolve one topic name to its absolute path.
//!
//! Read-only counterpart to `clr topic`: `topic` creates and enters a topic session,
//! `topics` reports on them without running anything. Both compute paths through the
//! same `topic_path`/`claude_storage_core` helpers, so a path printed here is exactly
//! what `topic` would use for that name.
//!
//! Two topic mechanisms, two resolve flags:
//! - `--path NAME` — the legacy DIR-mode topic directory, `<base>/-<name>`. Pure
//!   computation that never touches the disk — it answers "where would this topic's
//!   directory live?" identically whether or not it exists, which is what makes a
//!   global topic recoverable from its name alone in a later shell.
//! - `--file NAME` — the FORK-mode topic session file,
//!   `<storage of base>/<UUIDv5( canonical base, name )>.jsonl`. Equally pure (the
//!   file need not exist) and byte-identical to
//!   `claude_storage .session.path path::<base> topic::NAME` — both delegate to
//!   `claude_storage_core::topic_session_file`.
//!
//! The listing merges both mechanisms: directory topics are discovered by scanning
//! the base for `-<name>` directories; fork topics are read from the topics registry
//! (`topic_registry`), since their `UUIDv5` identity is one-way and the name cannot be
//! recovered from the session file alone. The same name can appear once per mode.

use super::help::print_topics_help;
use super::topic_path::{ topic_base, topic_dir, topic_name_of };

/// One topic found under the base — a `-<name>` directory (mode `dir`) or a
/// registry-recorded fork session (mode `fork`) — with its session count.
struct TopicEntry
{
  name     : String,
  mode     : &'static str,
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

/// Collect every DIR-mode topic directory directly under `base`, unsorted (the
/// caller merges and sorts across modes).
///
/// A non-existent or unreadable base yields an empty list rather than an error: the
/// global topic home legitimately does not exist until the first global topic is created.
fn collect_topics( base : &std::path::Path ) -> Vec< TopicEntry >
{
  let Ok( entries ) = std::fs::read_dir( base ) else { return Vec::new(); };
  entries
    .filter_map( Result::ok )
    .filter( | e | e.path().is_dir() )
    .filter_map( | e |
    {
      let file_name = e.file_name();
      let raw = file_name.to_str()?;
      let name = topic_name_of( raw )?.to_string();
      let path = e.path();
      let sessions = session_count( &path );
      Some( TopicEntry { name, mode : "dir", path, sessions } )
    } )
    .collect()
}

/// Collect every FORK-mode topic recorded for `base` in the topics registry,
/// unsorted (the caller merges and sorts across modes).
///
/// Path and existence are resolved through the shared `UUIDv5` rule
/// (`claude_storage_core::topic_session_file`); the registry contributes only the
/// names. Sessions is 1 when the session file exists non-empty, 0 otherwise — a
/// registry entry whose file was deleted (topic restarted by hand) stays listed
/// with 0, since its name is still reserved for auto-naming purposes.
fn collect_fork_topics( base : &std::path::Path ) -> Vec< TopicEntry >
{
  let canonical_base = claude_storage_core::physical_abs( base );
  super::topic_registry::list( &canonical_base )
    .into_iter()
    .filter_map( | name |
    {
      let file = claude_storage_core::topic_session_file( &canonical_base, &name )?;
      let sessions = usize::from(
        std::fs::metadata( &file ).is_ok_and( | meta | meta.len() > 0 ) );
      Some( TopicEntry { name, mode : "fork", path : file, sessions } )
    } )
    .collect()
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
  let mut file_arg : Option< String > = None;
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
      "--file" =>
      {
        let Some( val ) = tokens.get( i + 1 ) else
        {
          eprintln!( "Error: --file requires a value\nRun with --help for usage." );
          std::process::exit( 1 );
        };
        // Same single-component guard as --path: a topic name is a name, never a path.
        if val.is_empty() || val.contains( '/' )
        {
          eprintln!( "Error: --file must be a single topic name (non-empty, no '/' separators)" );
          std::process::exit( 1 );
        }
        file_arg = Some( val.clone() );
        i += 2;
      }
      other =>
      {
        eprintln!( "Error: unknown option '{other}'\nRun with --help for usage." );
        std::process::exit( 1 );
      }
    }
  }

  if path_arg.is_some() && file_arg.is_some()
  {
    eprintln!( "Error: --path and --file are mutually exclusive (dir-mode directory vs fork-mode session file)\nRun with --help for usage." );
    std::process::exit( 1 );
  }

  let base = topic_base( dir_arg.as_deref(), global );

  // Resolve forms: pure computations, always exit 0 on success.
  // --path: the dir-mode topic directory — no filesystem access at all.
  if let Some( name ) = path_arg
  {
    println!( "{}", topic_dir( &base, &name ).display() );
    std::process::exit( 0 );
  }
  // --file: the fork-mode topic session file — the shared UUIDv5 rule keyed on the
  // canonical physical base, so the printed path is byte-identical to
  // `claude_storage .session.path path::<base> topic::NAME`. The file need not exist.
  if let Some( name ) = file_arg
  {
    let canonical_base = claude_storage_core::physical_abs( &base );
    let Some( file ) = claude_storage_core::topic_session_file( &canonical_base, &name ) else
    {
      eprintln!( "Error: cannot resolve session storage for topic '{name}' (is HOME set?)" );
      std::process::exit( 1 );
    };
    println!( "{}", file.display() );
    std::process::exit( 0 );
  }

  // List form: merge dir-mode (scanned) and fork-mode (registry) topics. The same
  // name can legitimately exist once per mode — both rows are shown.
  let mut topics = collect_topics( &base );
  topics.extend( collect_fork_topics( &base ) );
  topics.sort_by( | a, b | a.name.cmp( &b.name ).then( a.mode.cmp( b.mode ) ) );
  if topics.is_empty()
  {
    eprintln!( "no topics in {}", base.display() );
    std::process::exit( 0 );
  }

  let name_width = topics.iter().map( | t | t.name.len() ).max().unwrap_or( 4 ).max( 4 );
  println!( "{:<name_width$}  MODE  {:>8}  PATH", "NAME", "SESSIONS" );
  for t in &topics
  {
    println!( "{:<name_width$}  {:<4}  {:>8}  {}", t.name, t.mode, t.sessions, t.path.display() );
  }
  std::process::exit( 0 );
}
