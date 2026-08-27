//! `clr sessions` — what the daemon is currently hosting.
//!
//! # Not `clr ps`
//!
//! `clr ps` scans `/proc` and lists every Claude Code process on the machine,
//! however it was started and whoever started it. This lists only the sessions
//! *this daemon owns* — the ones `clr chat` can address by conversation id, and
//! the ones `clr daemon stop` would take down.
//!
//! The two answers are genuinely different and both are worth having: `ps` is
//! "what is running", this is "what is hosted". A session that appears in `ps`
//! and not here is one the daemon cannot talk to.
//!
//! # Why this does not start a daemon
//!
//! `clr chat` starts one, because a client asking to talk to a session wants a
//! session, and the daemon is how it gets one. Asking what is hosted is a
//! question, and a question that starts a process to answer itself has changed
//! the thing it was asking about. With no daemon there is nothing hosted, which
//! is a complete and correct answer — reported on stderr so it cannot be mistaken
//! for a row of output, with exit 0 because nothing failed.

use claude_daemon_core::{ client, Request, SessionSummary };
use claude_runner_core::ps_table::{ render_headed_table, shorten_path };
use data_fmt::{ Heading, RowBuilder };

use super::daemon::{ daemon_paths, probe };

/// `clr sessions [--json]`.
pub( crate ) fn dispatch_sessions( tokens : &[ String ] ) -> !
{
  let mut as_json = false;

  for token in tokens.iter().skip( 1 )
  {
    match token.as_str()
    {
      "--json" => as_json = true,
      "help" | "-h" | "--help" => print_sessions_help(),
      other =>
      {
        eprintln!( "Error: unknown option {other:?} for 'clr sessions'" );
        eprintln!( "Run `clr sessions help` for usage." );
        std::process::exit( 1 )
      },
    }
  }

  let paths = daemon_paths();
  let socket = paths.socket_file();

  if probe( &socket ).is_none()
  {
    // stderr, so `clr sessions | wc -l` counts sessions and not this.
    eprintln!( "No session daemon is running — nothing is hosted." );
    eprintln!( "Start one with `clr daemon start`, or just run `clr chat`." );
    if as_json
    {
      println!( "[]" );
    }
    std::process::exit( 0 )
  }

  let listed = match client::call( &socket, &Request::ListSessions )
  {
    Ok( listed ) => listed,
    Err( error ) =>
    {
      eprintln!( "Error: the daemon would not list its sessions: {error}" );
      std::process::exit( 1 )
    },
  };

  if as_json
  {
    println!( "{}", serde_json::to_string_pretty( &listed ).unwrap_or_else( | _ | listed.to_string() ) );
    std::process::exit( 0 )
  }

  let sessions : Vec< SessionSummary > = serde_json::from_value( listed )
    .unwrap_or_else( | error |
    {
      eprintln!( "Error: the daemon's session list did not parse: {error}" );
      std::process::exit( 1 )
    } );

  if sessions.is_empty()
  {
    println!( "No hosted sessions." );
    std::process::exit( 0 )
  }

  print!( "{}", render_sessions( &sessions ) );
  std::process::exit( 0 )
}

/// Lay the sessions out the way `clr ps` lays out its own tables.
fn render_sessions( sessions : &[ SessionSummary ] ) -> String
{
  let headers = vec!
  [
    "#".to_string(),
    "SESSION".to_string(),
    "PID".to_string(),
    "STATE".to_string(),
    "CWD".to_string(),
  ];

  let mut builder = RowBuilder::new( headers );
  for ( index, session ) in sessions.iter().enumerate()
  {
    let row = vec!
    [
      ( index + 1 ).to_string(),
      // Never abbreviated: this is the handle `clr chat --session` takes, and a
      // handle you have to retype from memory is not a handle.
      session.session_id.clone(),
      session.pid.to_string(),
      if session.busy { "busy".to_string() } else { "idle".to_string() },
      shorten_path( &session.cwd.display().to_string() ),
    ];
    builder = builder.add_row( row.into_iter().map( Into::into ).collect() );
  }

  let busy = sessions.iter().filter( | session | session.busy ).count();
  let heading = Heading::new( "Hosted Sessions" )
    .with_field( format!( "{} total", sessions.len() ) )
    .with_field( format!( "{busy} busy" ) );

  render_headed_table( builder, heading )
}

/// Usage text for `clr sessions`.
fn print_sessions_help() -> !
{
  println!( "clr sessions — list the sessions the daemon is hosting" );
  println!();
  println!( "USAGE" );
  println!( "  clr sessions            One row per hosted session" );
  println!( "  clr sessions --json     The daemon's own list, unformatted" );
  println!( "  clr sessions help       Show this help" );
  println!();
  println!( "NOTES" );
  println!( "  Lists what this daemon owns, not every Claude Code process on the" );
  println!( "  machine — `clr ps` is the one that answers that." );
  println!();
  println!( "  Does not start a daemon. With none running there is nothing hosted," );
  println!( "  which is said on stderr and is not an error." );
  println!();
  println!( "  The SESSION column is the conversation id: pass it to" );
  println!( "  `clr chat --session <ID>` to continue that conversation." );
  std::process::exit( 0 )
}
