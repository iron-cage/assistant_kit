//! `clr scope` — print all 6 `CLAUDE_*` path variables for a directory.

use super::help::print_scope_help;

/// Parse, validate, and execute the `scope` subcommand.  Never returns.
///
/// Accepts an optional `--dir <PATH>` flag (alias `--to`) to set the target
/// directory; defaults to the current working directory.
///
/// Exits 0 after printing 6 `CLAUDE_*=value` lines to stdout in eval-safe format.
/// Exits 1 with a stderr error when the directory does not exist or `--dir` is
/// given without a value.
pub( crate ) fn dispatch_scope( tokens : &[ String ] ) -> !
{
  // tokens[0] == "scope"
  if tokens.iter().skip( 1 ).any( | t | t == "--help" || t == "-h" )
  {
    print_scope_help();
  }

  let mut dir_arg : Option< String > = None;
  let mut i = 1_usize;
  while i < tokens.len()
  {
    match tokens[ i ].as_str()
    {
      "--dir" | "--to" =>
      {
        if let Some( val ) = tokens.get( i + 1 )
        {
          dir_arg = Some( val.clone() );
          i += 2;
        }
        else
        {
          eprintln!( "Error: {} requires a value\nRun with --help for usage.", tokens[ i ] );
          std::process::exit( 1 );
        }
      }
      _ => { i += 1; }
    }
  }

  let dir = match dir_arg
  {
    Some( d ) => std::path::PathBuf::from( d ),
    None      => std::env::current_dir().unwrap_or_else( | _ | std::path::PathBuf::from( "." ) ),
  };

  if !dir.exists()
  {
    eprintln!( "Error: directory does not exist: {}", dir.display() );
    std::process::exit( 1 );
  }

  let scope = claude_storage_core::scope_for( &dir );

  println!( "CLAUDE_HOME={}", scope.claude_home.display() );
  println!( "CLAUDE_PROJECTS_DIR={}", scope.claude_projects_dir.display() );
  println!( "CLAUDE_SESSION_DIR={}", scope.claude_session_dir.display() );
  println!( "CLAUDE_MEMORY_DIR={}", scope.claude_memory_dir.display() );
  println!( "CLAUDE_MEMORY_FILE={}", scope.claude_memory_file.display() );
  match scope.claude_session_file
  {
    Some( f ) => println!( "CLAUDE_SESSION_FILE={}", f.display() ),
    None      => println!( "CLAUDE_SESSION_FILE=" ),
  }

  std::process::exit( 0 );
}
