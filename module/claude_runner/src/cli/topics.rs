//! `clr topics` — list topics, or resolve one topic name to its absolute path.
//!
//! Read-only counterpart to `clr topic`: `topic` creates and enters a topic session,
//! `topics` reports on them without running anything. Both compute paths through the
//! same `claude_topic_core`/`claude_storage_core` helpers, so a path printed here is exactly
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
//! (`claude_topic_core::registry`), since their `UUIDv5` identity is one-way and the
//! name cannot be recovered from the session file alone. The same name can appear once
//! per mode.

use super::help::print_topics_help;
use claude_topic_core::{ topic_base, topic_dir };

/// Parsed `topics` flags: base override, resolver selections, and global switch.
struct TopicsArgs
{
  dir : Option< String >,
  path : Option< String >,
  file : Option< String >,
  global : bool,
}

/// Parse the `topics` token stream; prints help or an error and exits on
/// `help`/`--help`, unknown options, missing values, and invalid names.
fn parse_topics_args( tokens : &[ String ] ) -> TopicsArgs
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

  TopicsArgs { dir : dir_arg, path : path_arg, file : file_arg, global }
}

/// Parse, validate, and execute the `topics` subcommand. Never returns.
///
/// Three forms: `--path NAME` resolves one name to its dir-mode directory, `--file NAME`
/// to its fork-mode session file — each printed and exited; otherwise every topic under
/// the resolved base is listed.
pub( crate ) fn dispatch_topics( tokens : &[ String ] ) -> !
{
  let args = parse_topics_args( tokens );

  if args.path.is_some() && args.file.is_some()
  {
    eprintln!( "Error: --path and --file are mutually exclusive (dir-mode directory vs fork-mode session file)\nRun with --help for usage." );
    std::process::exit( 1 );
  }

  let base = topic_base( args.dir.as_deref(), args.global );
  let path_arg = args.path;
  let file_arg = args.file;

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
  let topics = claude_topic_core::enumerate( &base );
  if topics.is_empty()
  {
    eprintln!( "no topics in {}", base.display() );
    std::process::exit( 0 );
  }

  let name_width = topics.iter().map( | t | t.name.len() ).max().unwrap_or( 4 ).max( 4 );
  println!( "{:<name_width$}  MODE  {:>8}  PATH", "NAME", "SESSIONS" );
  for t in &topics
  {
    println!
    (
      "{:<name_width$}  {:<4}  {:>8}  {}",
      t.name, t.mode.as_str(), t.sessions, t.path.display()
    );
  }
  std::process::exit( 0 );
}
