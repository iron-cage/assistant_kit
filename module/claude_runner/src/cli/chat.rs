//! `clr chat` — one prompt, one answer, and the session stays alive.
//!
//! This is the command the rest of the daemon exists to make possible. From the
//! outside it looks exactly like print mode: type a prompt, get an answer, get
//! the shell prompt back. What is different is what happens afterwards — the
//! session it talked to is still there, still holding the conversation, ready
//! for the next `clr chat` to continue it rather than start over.
//!
//! # Knowing when the answer is finished
//!
//! Harder than it sounds, and the reason this file is not four lines. A hosted
//! session is a terminal application: output arrives continuously, including
//! while it is thinking, and nothing in the stream says "done".
//!
//! Two independent signals are required to agree, because each one alone is
//! wrong in a way the other is not:
//!
//! - **The session reports itself idle.** Claude Code writes its own status into
//!   its registry, and the daemon reads it back through a turn watcher. On its
//!   own this is too eager: the status is written by another process, so for a
//!   moment after `send` the session is still recorded as idle from *before* the
//!   prompt arrived, and a client that trusted it would return having printed
//!   nothing.
//! - **Output has stopped arriving.** On its own this is too eager as well, in
//!   the opposite direction: a session waiting on a slow tool call is silent for
//!   seconds at a time without being finished.
//!
//! Idle *and* quiet is neither. The registry lag is covered by the quiet
//! requirement, because output is streaming during it; the mid-turn pause is
//! covered by the idle requirement, because a session waiting on a tool is
//! recorded busy.
//!
//! # Knowing *when* is not knowing *what*
//!
//! The two signals above settle when the turn ended. They say nothing about what
//! the answer was, and the terminal — which is where the words physically
//! arrived — is a bad place to ask, because what arrived is a picture of Claude
//! Code's interface rather than a message: input box, status bar, spinner
//! frames, box rules, and the answer somewhere among them.
//!
//! So the answer is read from the session's own transcript instead, as
//! structured data, keyed by the conversation id the daemon already holds. See
//! [`super::chat_answer`]. The terminal stays the fallback, and `--raw` still
//! prints it verbatim for anyone who wants the bytes.
//!
//! # Why not just wait for the session to be idle
//!
//! Because `idle` is only trustworthy when the session was started with
//! background-task reporting on — otherwise a session parked on an outstanding
//! background task is indistinguishable from one that has finished. The daemon
//! does start them that way, which is what makes the first signal usable at all;
//! see [`crate::cli::daemon`]. Requiring quiet as well means this command does
//! not silently degrade if that ever stops being true.

use core::time::Duration;
use std::path::PathBuf;
use std::time::Instant;

use claude_daemon_core::{ client, to_plain_text, OutputSlice, Request, SessionSummary };

use super::chat_answer;
use super::daemon::{ daemon_paths, ensure_running };

/// How long to wait for an answer before giving up on it.
const DEFAULT_TIMEOUT : Duration = Duration::from_secs( 300 );

/// Gap between polls while an answer is arriving.
///
/// Short enough that the settle window below is a fraction of a second's
/// resolution, long enough that a five-minute turn is a few thousand reads over
/// a local socket rather than a busy loop.
const POLL : Duration = Duration::from_millis( 100 );

/// Consecutive empty reads that count as the output having stopped.
///
/// Eight polls is roughly eight tenths of a second. Under it, the pause between
/// two lines of a streaming answer starts to look like the end of one.
const QUIET_POLLS : usize = 8;

/// How long a freshly spawned session gets to draw its banner and settle before
/// a prompt is sent to it.
///
/// A session is registered — and therefore addressable — before its interface
/// has finished drawing. Text delivered into a terminal that is still painting
/// its startup screen is not reliably read as input.
const BANNER_SETTLE : Duration = Duration::from_secs( 3 );

/// How long to wait for the transcript to catch up with the finished turn.
///
/// The turn is over when the session is idle and quiet; the transcript is
/// complete when Claude Code has finished flushing it. The second follows the
/// first closely but not atomically, and this covers the difference. Generous,
/// because the cost of being wrong is printing a terminal dump instead of an
/// answer, and it is only ever paid when the transcript really is unavailable.
const TRANSCRIPT_GRACE : Duration = Duration::from_secs( 5 );

/// Parsed `clr chat` invocation.
struct ChatArgs
{
  message : String,
  dir : PathBuf,
  session : Option< String >,
  timeout : Duration,
  raw : bool,
}

/// `clr chat "<MESSAGE>" [--dir <PATH>] [--session <ID>] [--timeout <SECS>] [--raw]`.
pub( crate ) fn dispatch_chat( tokens : &[ String ] ) -> !
{
  let args = parse_args( &tokens[ 1.. ] );

  let paths = daemon_paths();
  let socket = paths.socket_file();

  if let Err( reason ) = ensure_running( &paths )
  {
    eprintln!( "Error: {reason}" );
    eprintln!( "The daemon's log is at {}", paths.log_file().display() );
    std::process::exit( 1 )
  }

  let ( session_id, cwd ) = resolve_session( &socket, &args );

  // Both marks are taken before the write, and for the same reason: everything
  // past them is this turn. One marks the terminal, the other the transcript.
  let transcript = chat_answer::transcript_path( &cwd, &session_id );
  let entries_before = transcript.as_deref().map_or( 0, chat_answer::mark );

  let cursor = match client::call( &socket, &Request::Send
  {
    session_id : session_id.clone(),
    text : args.message.clone(),
  } )
  {
    // The cursor from immediately before the write — so what is read back starts
    // at this prompt and not at whatever the session was showing beforehand.
    Ok( result ) => result[ "cursor" ].as_u64().unwrap_or( 0 ),
    Err( error ) =>
    {
      eprintln!( "Error: the session would not take the message: {error}" );
      std::process::exit( 1 )
    },
  };

  let answer = collect_answer( &socket, &session_id, cursor, args.timeout );

  // Not consulted under `--raw`: that flag asks for the terminal's bytes, and
  // spending five seconds looking for a nicer answer nobody asked for would only
  // delay them.
  let written = if args.raw
  {
    None
  }
  else
  {
    transcript
      .as_deref()
      .and_then( | path | chat_answer::answer_since( path, entries_before, TRANSCRIPT_GRACE ) )
  };

  print_answer( &answer, written.as_deref(), args.raw );

  std::process::exit( 0 )
}

/// Pick out the flags, and reject anything that is not one.
fn parse_args( tokens : &[ String ] ) -> ChatArgs
{
  let mut message : Option< String > = None;
  let mut dir : Option< PathBuf > = None;
  let mut session : Option< String > = None;
  let mut timeout = DEFAULT_TIMEOUT;
  let mut raw = false;

  let mut rest = tokens.iter();
  while let Some( token ) = rest.next()
  {
    match token.as_str()
    {
      "help" | "-h" | "--help" => print_chat_help(),
      "--raw" => raw = true,
      "--dir" => dir = Some( PathBuf::from( expect_value( rest.next(), "--dir" ) ) ),
      "--session" => session = Some( expect_value( rest.next(), "--session" ).clone() ),
      "--timeout" =>
      {
        let value = expect_value( rest.next(), "--timeout" );
        let Ok( seconds ) = value.parse::< u64 >() else
        {
          eprintln!( "Error: --timeout wants a whole number of seconds, got {value:?}" );
          std::process::exit( 1 )
        };
        timeout = Duration::from_secs( seconds );
      },
      other if other.starts_with( '-' ) =>
      {
        eprintln!( "Error: unknown option {other:?} for 'clr chat'" );
        eprintln!( "Run `clr chat help` for usage." );
        std::process::exit( 1 )
      },
      // Everything else is the message. More than one means the quotes were
      // forgotten, which is worth saying rather than silently chatting about
      // only the first word.
      other if message.is_none() => message = Some( other.to_string() ),
      other =>
      {
        eprintln!( "Error: unexpected extra argument {other:?}" );
        eprintln!( "The message is one argument — quote it: clr chat \"your message\"" );
        std::process::exit( 1 )
      },
    }
  }

  let Some( message ) = message else
  {
    eprintln!( "Error: 'clr chat' needs a message" );
    eprintln!( "Run `clr chat help` for usage." );
    std::process::exit( 1 )
  };

  ChatArgs
  {
    message,
    dir : dir.unwrap_or_else( || std::env::current_dir().unwrap_or_else( | _ | PathBuf::from( "." ) ) ),
    session,
    timeout,
    raw,
  }
}

/// The value after a flag, or a complaint that there was not one.
fn expect_value< 'token >( value : Option< &'token String >, flag : &str ) -> &'token String
{
  value.unwrap_or_else( ||
  {
    eprintln!( "Error: {flag} needs a value" );
    std::process::exit( 1 )
  } )
}

/// Find the session to talk to, starting one if there is nothing suitable.
///
/// Order matters. An explicit `--session` is honoured even if it is busy — the
/// caller named it. Otherwise the working directory decides, because that is
/// what makes `clr chat` twice in a row continue one conversation instead of
/// accumulating sessions.
///
/// Returns the session's own working directory alongside its id, because that
/// directory is half of the transcript's address and `--session` can name a
/// session somewhere else entirely.
fn resolve_session( socket : &std::path::Path, args : &ChatArgs ) -> ( String, PathBuf )
{
  let sessions = list_sessions( socket );

  if let Some( id ) = &args.session
  {
    // `args.dir` is the fallback rather than an error: an id the daemon does not
    // list will fail at `send` in a moment with a better message than anything
    // that could be said here, and until then the caller's own directory is the
    // best guess available.
    let cwd = sessions
      .iter()
      .find( | session | &session.session_id == id )
      .map_or_else( || args.dir.clone(), | session | session.cwd.clone() );
    return ( id.clone(), cwd );
  }

  let here = args.dir.canonicalize().unwrap_or_else( | _ | args.dir.clone() );
  let existing = sessions.iter().find( | session |
  {
    session.cwd.canonicalize().unwrap_or_else( | _ | session.cwd.clone() ) == here
  } );

  if let Some( session ) = existing
  {
    return ( session.session_id.clone(), session.cwd.clone() );
  }

  let session_id = spawn_session( socket, &here );
  ( session_id, here )
}

/// Ask the daemon what it is hosting.
fn list_sessions( socket : &std::path::Path ) -> Vec< SessionSummary >
{
  let Ok( listed ) = client::call( socket, &Request::ListSessions ) else { return Vec::new() };
  serde_json::from_value( listed ).unwrap_or_default()
}

/// Start a session in `cwd` and let it finish drawing itself.
fn spawn_session( socket : &std::path::Path, cwd : &std::path::Path ) -> String
{
  eprintln!( "Starting a session in {} …", cwd.display() );

  let spawned = client::call( socket, &Request::Spawn
  {
    cwd : cwd.to_path_buf(),
    // Sent separately afterwards, not here. The daemon delivers an inline prompt
    // the instant registration completes, which is earlier than the interface is
    // ready to be typed into.
    prompt : None,
  } );

  let session_id = match spawned
  {
    Ok( result ) => result[ "session_id" ].as_str().unwrap_or_default().to_string(),
    Err( error ) =>
    {
      let message = error.to_string();
      eprintln!( "Error: the session would not start: {message}" );

      // The daemon can only report that no conversation id arrived; it cannot see
      // *why*, because the reason is on a terminal it does not read. In practice
      // the common cause is a `claude` that came up in a first-run prompt — a
      // theme picker, a trust prompt — and is sitting there waiting to be
      // answered, having never got as far as opening a conversation. That is
      // invisible from here and unfixable from here, but it is fixable in one
      // step, so it is worth naming.
      if message.contains( "never registered a conversation id" )
      {
        eprintln!( "Hint: run `claude` once in this environment and answer any" );
        eprintln!( "      first-run prompts — a session parked on one never gets" );
        eprintln!( "      far enough to be hosted. `clr daemon log` shows the rest." );
      }

      std::process::exit( 1 )
    },
  };

  if session_id.is_empty()
  {
    eprintln!( "Error: the daemon started a session but did not name it" );
    std::process::exit( 1 )
  }

  // Nothing is read here — the banner takes care of itself, because `send`
  // reports the cursor as it stands at that moment and everything printed before
  // it is behind that mark. This wait is only so the prompt lands in a terminal
  // that has finished drawing.
  std::thread::sleep( BANNER_SETTLE );

  session_id
}

/// What one answer amounted to.
struct Answer
{
  text : String,
  missed : bool,
  ended : bool,
  timed_out : bool,
}

/// Poll until the answer has finished arriving, and return everything it was.
///
/// See the module docs for why two signals are required rather than one.
fn collect_answer
(
  socket : &std::path::Path,
  session_id : &str,
  from : u64,
  timeout : Duration,
) -> Answer
{
  let mut text = String::new();
  let mut cursor = from;
  let mut missed = false;
  let mut quiet = 0_usize;
  let deadline = Instant::now() + timeout;

  loop
  {
    if Instant::now() >= deadline
    {
      return Answer { text, missed, ended : false, timed_out : true };
    }

    // A read that failed is not an answer that ended. Retrying costs one poll
    // interval, and the deadline above is what stops it going on forever.
    let Some( slice ) = read_slice( socket, session_id, cursor ) else
    {
      std::thread::sleep( POLL );
      continue;
    };

    missed |= slice.missed > 0;
    cursor = slice.cursor;

    if slice.text.is_empty()
    {
      quiet += 1;
    }
    else
    {
      text.push_str( &slice.text );
      quiet = 0;
    }

    // Nothing more will ever arrive — no point waiting for it to go quiet.
    if slice.ended
    {
      return Answer { text, missed, ended : true, timed_out : false };
    }

    if quiet >= QUIET_POLLS && !is_busy( socket, session_id )
    {
      return Answer { text, missed, ended : false, timed_out : false };
    }

    std::thread::sleep( POLL );
  }
}

/// One `read`, or `None` if the daemon did not answer it.
fn read_slice( socket : &std::path::Path, session_id : &str, cursor : u64 ) -> Option< OutputSlice >
{
  let result = client::call( socket, &Request::Read
  {
    session_id : session_id.to_string(),
    cursor,
  } )
  .ok()?;
  serde_json::from_value( result ).ok()
}

/// Whether the daemon still believes a turn is in flight.
///
/// A daemon that cannot say is treated as not busy, so an unanswerable question
/// cannot hold the command open past a genuinely finished answer — the quiet
/// requirement has already been met by the time this is consulted.
fn is_busy( socket : &std::path::Path, session_id : &str ) -> bool
{
  list_sessions( socket )
    .iter()
    .find( | session | session.session_id == session_id )
    .is_some_and( | session | session.busy )
}

/// Print what came back, and say so if it is not the whole story.
///
/// `written` is the answer as the transcript recorded it, when there was one.
/// It is preferred over the terminal because it is the message rather than a
/// picture of one — see [`super::chat_answer`].
fn print_answer( answer : &Answer, written : Option< &str >, raw : bool )
{
  match ( raw, written )
  {
    ( true, _ ) => print!( "{}", answer.text ),
    ( false, Some( text ) ) => println!( "{text}" ),
    ( false, None ) =>
    {
      let rendered = to_plain_text( &answer.text );
      if !rendered.is_empty()
      {
        println!( "{rendered}" );
      }
      // Worth saying, because the difference is visible: this output is a
      // terminal, chrome and all, where the caller was promised an answer.
      eprintln!( "Note: the answer could not be read from the session transcript — showing the terminal instead." );
    },
  }

  // All of these go to stderr: they are about the answer, not part of it, and a
  // caller redirecting stdout wants the answer alone.
  //
  // `missed` describes a gap in the terminal's ring buffer, so it only describes
  // a gap in the *answer* when the answer came from the terminal.
  if answer.missed && ( raw || written.is_none() )
  {
    eprintln!( "Warning: some output was dropped before it could be read — the answer above has a gap." );
  }
  if answer.ended
  {
    eprintln!( "Note: the session ended while answering." );
  }
  if answer.timed_out
  {
    eprintln!( "Warning: gave up waiting — the answer above may be incomplete." );
    eprintln!( "The session is still running; `clr chat` again to see the rest." );
  }
}

/// Usage text for `clr chat`.
fn print_chat_help() -> !
{
  println!( "clr chat — send a prompt to a hosted session and print the answer" );
  println!();
  println!( "USAGE" );
  println!( "  clr chat \"<MESSAGE>\" [OPTIONS]" );
  println!();
  println!( "OPTIONS" );
  println!( "  --dir <PATH>       Working directory of the session (default: here)" );
  println!( "  --session <ID>     Talk to this session, whatever directory it is in" );
  println!( "  --timeout <SECS>   Give up waiting after this long (default: 300)" );
  println!( "  --raw              Print the session's terminal bytes — interface and all —" );
  println!( "                     instead of the answer its transcript recorded" );
  println!();
  println!( "NOTES" );
  println!( "  Reuses the session already running in the working directory, so two" );
  println!( "  chats in a row continue one conversation. `clr sessions` lists them." );
  println!();
  println!( "  Starts the daemon if it is not running — no separate setup step." );
  println!();
  println!( "  The session stays alive and idle afterwards. `clr daemon stop` ends" );
  println!( "  every one of them." );
  std::process::exit( 0 )
}
