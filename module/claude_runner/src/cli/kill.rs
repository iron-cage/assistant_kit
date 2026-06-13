//! `clr kill <pid>` — terminate a running Claude Code session by PID.
//!
//! Validates that the target PID belongs to a currently running `claude`
//! process before sending SIGTERM.  This prevents accidental termination of
//! unrelated processes that happen to share the same numeric PID.

use claude_core::process::{ find_claude_processes, send_sigterm };

/// Print help for the `kill` subcommand and exit 0.
///
/// Called when `-h` or `--help` is detected in `dispatch_kill`.
pub( super ) fn print_kill_help() -> !
{
  println!( "clr kill — Terminate a running Claude Code session by PID" );
  println!();
  println!( "USAGE:" );
  println!( "  clr kill <PID>" );
  println!();
  println!( "ARGUMENTS:" );
  println!( "  <PID>                              Process ID of the Claude Code session to terminate" );
  println!();
  println!( "Sends SIGTERM to the target process.  The PID must belong to a currently" );
  println!( "running Claude Code process.  Use `clr ps` to list active sessions and PIDs." );
  println!();
  println!( "EXIT CODES:" );
  println!( "  0    SIGTERM delivered successfully" );
  println!( "  1    Error (missing PID, invalid PID, not a Claude session, signal failed)" );
  println!();
  println!( "  -h, --help                         Show this help" );
  std::process::exit( 0 );
}

/// Dispatch `clr kill <pid>`: send SIGTERM to the given Claude Code process.
///
/// Validates:
/// - Exactly one positional argument (the PID) is present.
/// - The argument is a valid positive integer.
/// - The PID belongs to a currently running `claude` process.
///
/// Exits 0 on success; exits 1 on any error.
pub( crate ) fn dispatch_kill( tokens : &[ String ] ) -> !
{
  // Handle help before positional-argument checks.
  if tokens.iter().skip( 1 ).any( | t | t == "--help" || t == "-h" )
  {
    print_kill_help();
  }

  let pid_str = match tokens.get( 1 )
  {
    None =>
    {
      eprintln!( "Error: missing PID argument.\nUsage: clr kill <PID>\nRun 'clr kill --help' for usage." );
      std::process::exit( 1 );
    }
    Some( tok ) if tok.starts_with( '-' ) =>
    {
      eprintln!( "Error: unknown option: {tok}\nRun 'clr kill --help' for usage." );
      std::process::exit( 1 );
    }
    Some( tok ) => tok,
  };

  if let Some( extra ) = tokens.get( 2 )
  {
    eprintln!( "Error: unexpected argument: {extra}\nRun 'clr kill --help' for usage." );
    std::process::exit( 1 );
  }

  let pid : u32 = if let Ok( p ) = pid_str.parse()
  {
    p
  }
  else
  {
    eprintln!( "Error: invalid PID '{pid_str}': must be a positive integer" );
    std::process::exit( 1 );
  };

  // Validate the PID belongs to an active claude process before sending the signal.
  // This prevents accidentally killing unrelated processes that share a PID.
  let procs = find_claude_processes();
  if !procs.iter().any( | p | p.pid == pid )
  {
    eprintln!(
      "Error: PID {pid} is not a running Claude Code session.\n\
       Use 'clr ps' to list active sessions."
    );
    std::process::exit( 1 );
  }

  if let Err( e ) = send_sigterm( pid )
  {
    eprintln!( "Error: failed to send SIGTERM to PID {pid}: {e}" );
    std::process::exit( 1 );
  }
  println!( "Sent SIGTERM to Claude Code session {pid}." );
  std::process::exit( 0 );
}
