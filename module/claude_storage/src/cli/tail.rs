//! `.tail` command — display the last N conversation turns of a session.
// BUG-002 — real implementation replacing the hardcoded-output stub

use core::fmt::Write as FmtWrite;
use unilang::{ VerifiedCommand, ExecutionContext, OutputData, ErrorData, ErrorCode };
use claude_storage_core::{ ContentBlock, Entry, EntryType };
use super::color;
use super::storage::{ create_storage, resolve_path_parameter, find_session_mut, most_recent_session_mut };
use super::format::
{
  RULE_WIDTH, ToolResultSummary, ToolResults,
  ellipsize, format_clock, join_pieces, now_epoch_seconds, relative_time, render_blocks_annotated,
};

/// Body lines shown per turn before the remainder is folded behind a hint.
///
/// Eight lines is roughly a short paragraph — enough to recognise a turn without
/// letting one long answer push the rest of the window off screen. `full::1`
/// lifts the limit entirely.
const DEFAULT_BODY_LINES : usize = 8;

/// One conversation turn: the entries a single API response (or a single user
/// message) is spread across.
///
/// Claude Code writes one JSONL record per content chunk, so a single assistant
/// response routinely occupies several consecutive lines that all carry the same
/// `message.id`. Counting records instead of turns makes `last::4` return two
/// halves of one answer and call it four messages.
struct Turn< 'a >
{
  /// 1-based positions of this turn's entries within the session
  positions : Vec< usize >,
  /// The entries themselves, in file order
  entries : Vec< &'a Entry >,
}

impl< 'a > Turn< 'a >
{
  /// Who produced this turn
  fn entry_type( &self ) -> EntryType
  {
    self.entries[ 0 ].entry_type
  }

  /// The label a reader sees on the rule line
  fn speaker( &self ) -> &'static str
  {
    match self.entry_type()
    {
      EntryType::User => "You",
      EntryType::Assistant => "Claude",
    }
  }

  /// Timestamp of the turn's last entry — when it finished
  fn timestamp( &self ) -> &'a str
  {
    &self.entries[ self.entries.len() - 1 ].timestamp
  }

  /// 1-based session position of the turn's last entry, for `.show index::`
  fn last_position( &self ) -> usize
  {
    self.positions[ self.positions.len() - 1 ]
  }

  /// Every content block across the turn's entries
  fn blocks( &self ) -> Vec< ContentBlock >
  {
    self.entries.iter().flat_map( | e | e.content_blocks().iter().cloned() ).collect()
  }
}

/// Display-mode switches resolved from the command line.
struct RenderOptions
{
  /// Print one line per turn instead of full bodies
  compact : bool,
  /// Print every body line instead of folding after [`DEFAULT_BODY_LINES`]
  full : bool,
}

/// Display the last N turns of a session (most-recent context refresh).
///
/// Smart behavior based on parameters (see `docs/cli/command/12_tail.md`):
/// - No parameters → current directory's project, most recently modified
///   non-agent session, last 4 turns
/// - `last::N` (alias `l::N`) → last N turns (`last::0` = all turns, uncapped)
/// - `full::1` → print every body line instead of folding long turns
/// - `compact::1` → one line per turn
/// - `topic::NAME` → session `-NAME` explicitly, instead of the recency fallback
/// - `path::DIR` → resolve the project from `DIR` instead of the current directory
///
/// # Errors
///
/// Returns error (exit 1) if `last` is negative, or if the session cannot be located.
///
/// # Exit Codes
///
/// Exits directly with code 2 (bypassing the standard exit-1 error path) when no
/// project exists for the resolved directory — matches the `.status` command's
/// "not found = usage error" convention (see `status.rs`).
///
/// # Panics
///
/// Does not panic — the `last_count` conversion below is only reached after the
/// negative-value branch already returned, so the value is always non-negative.
#[ inline ]
pub fn tail_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  // Validate `last` before any storage access — rejection happens before entries
  // (or even the project) are loaded, per docs/cli/command/12_tail.md INT-8.
  //
  // `get_integer( "last" )` covers the `l::` alias too: unilang binds an alias to
  // its canonical argument name during semantic analysis, so the routine only ever
  // reads the canonical name (see unilang `semantic/argument_binding.rs`).
  let last_count = cmd.get_integer( "last" ).unwrap_or( 4 );
  if last_count < 0
  {
    return Err( ErrorData::new( ErrorCode::InternalError, "last must be non-negative".to_string() ) );
  }
  let last_count = usize::try_from( last_count ).expect( "last < 0 rejected above" );

  let options = RenderOptions
  {
    compact : cmd.get_boolean( "compact" ).unwrap_or( false ),
    full : cmd.get_boolean( "full" ).unwrap_or( false ),
  };

  let topic = cmd.get_string( "topic" );

  let storage = create_storage()?;
  let path_param = cmd.get_string( "path" );

  let project = if let Some( raw_path ) = path_param
  {
    let resolved = resolve_path_parameter( raw_path )
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to resolve path '{raw_path}': {e}" ) ) )?;
    if let Ok( project ) = storage.load_project_for_path( &resolved )
    {
      project
    }
    else
    {
      eprintln!( "No project found for path: {resolved}" );
      std::process::exit( 2 );
    }
  }
  else if let Ok( project ) = storage.load_project_for_cwd()
  {
    project
  }
  else
  {
    eprintln!( "No project found for current directory" );
    std::process::exit( 2 );
  };

  let project_label = project_label( &project );

  let mut sessions = project.all_sessions()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list sessions: {e}" ) ) )?;

  // Fix(BUG-488)
  // Root cause: an explicit topic:: always resolved to session `-{topic}` via
  //   exact/substring ID match, but with no topic:: given, the code guessed a
  //   fixed `-default_topic` ID even though real Claude Code sessions are
  //   UUID-named — the guess could never match, so the common case (bare
  //   `.tail` in a project with only native sessions) always failed.
  // Pitfall: a "default value" is not always the right fallback strategy —
  //   when the default collides with a naming scheme that never occurs in
  //   production data, prefer resolving by an orthogonal signal (recency)
  //   over guessing an ID that happens to match the parameter's own default.
  let session = if let Some( topic ) = topic
  {
    let session_id = format!( "-{topic}" );
    // Report the error against `topic` (what the user typed), not
    // `session_id` (the internally-prepended `-{topic}` form `find_session_mut`
    // matches against) — a caller-facing message must not leak an internal
    // naming convention the user never entered.
    find_session_mut( &mut sessions, &session_id )
      .map_err( | _ | ErrorData::new( ErrorCode::InternalError, format!( "Session not found for topic: {topic}" ) ) )?
  }
  else
  {
    most_recent_session_mut( &mut sessions )?
  };

  let session_id = session.id().to_string();
  let entries = session.entries()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to load entries: {e}" ) ) )?;

  let results = collect_tool_results( entries );
  let turns = group_turns( entries, &results );

  // Entries are stored oldest-first (append-only JSONL); a trailing slice therefore
  // needs no reordering — the suffix is already oldest-first.
  let window = if last_count == 0 || last_count >= turns.len()
  {
    &turns[ .. ]
  }
  else
  {
    &turns[ turns.len() - last_count.. ]
  };

  Ok( OutputData::new( render( &project_label, &session_id, &turns, window, &results, &options ), "text" ) )
}

/// Short human label for a project — the directory's own name, or its UUID.
fn project_label( project : &claude_storage_core::Project ) -> String
{
  if let Some( path ) = project.id().as_path()
  {
    if let Some( name ) = path.file_name().and_then( std::ffi::OsStr::to_str )
    {
      return name.to_string();
    }
  }

  project.id().as_uuid().unwrap_or( "unknown" ).to_string()
}

/// Index every tool result in the session by the `tool_use` id it answers.
///
/// Built across the whole session rather than the displayed window, so a tool
/// call at the very start of the window still shows what it returned even when
/// the answering record falls outside the slice.
fn collect_tool_results( entries : &[ Entry ] ) -> ToolResults
{
  let mut results = ToolResults::new();

  for entry in entries
  {
    for block in entry.content_blocks()
    {
      if let ContentBlock::ToolResult { tool_use_id, content, is_error } = block
      {
        let lines = if content.trim().is_empty() { 0 } else { content.lines().count() };
        results.insert( tool_use_id.clone(), ToolResultSummary { lines, is_error : *is_error } );
      }
    }
  }

  results
}

/// Collapse consecutive records that belong to one API response into single turns,
/// dropping turns that would render nothing.
///
/// Two records join the same turn when both are assistant records carrying the
/// same `message.id`. Every user record stands alone — user records have no
/// message id, and consecutive ones are genuinely separate events.
///
/// A turn is dropped when it renders empty: a pure tool-result turn contributes
/// nothing of its own (its content is folded onto the `⚙` line that invoked it),
/// and a turn whose only blocks are empty text carries nothing to read.
fn group_turns< 'a >( entries : &'a [ Entry ], results : &ToolResults ) -> Vec< Turn< 'a > >
{
  let mut turns : Vec< Turn< 'a > > = Vec::new();
  let mut previous_message_id : Option< &str > = None;

  for ( position, entry ) in entries.iter().enumerate()
  {
    let message_id = match &entry.message
    {
      claude_storage_core::MessageContent::Assistant( msg ) => Some( msg.message_id.as_str() ),
      claude_storage_core::MessageContent::User( _ ) => None,
    };

    let joins_previous = message_id.is_some() && message_id == previous_message_id;

    if joins_previous
    {
      let current = turns.last_mut().expect( "joins_previous implies a previous turn exists" );
      current.positions.push( position + 1 );
      current.entries.push( entry );
    }
    else
    {
      turns.push( Turn { positions : vec![ position + 1 ], entries : vec![ entry ] } );
    }

    previous_message_id = message_id;
  }

  turns.retain( | turn | !render_blocks_annotated( &turn.blocks(), results ).is_empty() );
  turns
}

/// Assemble the whole `.tail` output: session header, then one block per turn.
fn render
(
  project_label : &str,
  session_id : &str,
  turns : &[ Turn< '_ > ],
  window : &[ Turn< '_ > ],
  results : &ToolResults,
  options : &RenderOptions,
) -> String
{
  let now = now_epoch_seconds();

  // No trailing newline anywhere in here: the caller prints this through
  // `println!`, which supplies exactly one. Ending the string with `\n` too is
  // what produced the stray blank line at the bottom of every invocation.
  let mut out = session_header( project_label, session_id, turns, window, now );

  if window.is_empty()
  {
    return out;
  }

  let blocks : Vec< String > = window
    .iter()
    .enumerate()
    .map( | ( offset, turn ) |
    {
      let ordinal = turns.len() - window.len() + offset + 1;
      if options.compact
      {
        compact_line( ordinal, turn, results, now )
      }
      else
      {
        turn_block( turn, session_id, results, now, options.full )
      }
    })
    .collect();

  // One join, not a push-per-turn: a trailing separator after the last turn is
  // what left the old output ending in stray blank lines.
  out.push_str( "\n\n" );
  out.push_str( &blocks.join( if options.compact { "\n" } else { "\n\n" } ) );

  out
}

/// `claude_storage · feed0009 · turns 249-252 of 252 · last 3h ago`
fn session_header
(
  project_label : &str,
  session_id : &str,
  turns : &[ Turn< '_ > ],
  window : &[ Turn< '_ > ],
  now : i64,
) -> String
{
  let short_id = short_session_id( session_id );

  let Some( last ) = window.last() else
  {
    return color::muted( &format!( "{project_label} · {short_id} · no turns" ) );
  };

  let first_ordinal = turns.len() - window.len() + 1;
  let last_ordinal = turns.len();
  let span = if window.len() == 1
  {
    format!( "turn {last_ordinal} of {}", turns.len() )
  }
  else
  {
    format!( "turns {first_ordinal}-{last_ordinal} of {}", turns.len() )
  };

  color::muted( &format!( "{project_label} · {short_id} · {span} · last {}", relative_time( last.timestamp(), now ) ) )
}

/// Git-style short form of a session UUID, matching `.show`'s prefix lookup.
#[ inline ]
#[ must_use ]
pub fn short_session_id( session_id : &str ) -> String
{
  session_id.chars().take( 8 ).collect()
}

/// A full turn: rule-line header, then the body, folded unless `full` is set.
fn turn_block( turn : &Turn< '_ >, session_id : &str, results : &ToolResults, now : i64, full : bool ) -> String
{
  let mut out = rule_line( turn, now );
  out.push( '\n' );

  let body = join_pieces( &render_blocks_annotated( &turn.blocks(), results ) );
  let lines : Vec< &str > = body.lines().collect();

  // Fold only when the hint actually buys space — it occupies a line itself, so
  // hiding a single line behind it saves nothing and reads as censorship.
  if full || lines.len() <= DEFAULT_BODY_LINES + 1
  {
    out.push_str( &body );
    return out;
  }

  let hidden = lines.len() - DEFAULT_BODY_LINES;
  let line_noun = if hidden == 1 { "line" } else { "lines" };

  out.push_str( &lines[ ..DEFAULT_BODY_LINES ].join( "\n" ) );
  write!
  (
    out,
    "\n{}",
    color::muted( &format!
    (
      "⋯ {hidden} more {line_noun} · clg .show session_id::{} index::{}",
      short_session_id( session_id ),
      turn.last_position(),
    ))
  ).unwrap();

  out
}

/// `-- Claude ----------------------------------- 17h ago · 16:40 --`
///
/// A rule line, not a blank line, marks the turn boundary: message bodies contain
/// blank lines of their own, so whitespace alone can never be an unambiguous
/// separator. Bodies stay flush-left below it, so any line can be copied out
/// without stripping a gutter.
fn rule_line( turn : &Turn< '_ >, now : i64 ) -> String
{
  let left = format!( "── {} ", turn.speaker() );
  let right = format!( " {} · {} ──", relative_time( turn.timestamp(), now ), format_clock( turn.timestamp() ) );
  let fill = RULE_WIDTH
    .saturating_sub( left.chars().count() + right.chars().count() )
    .max( 3 );

  format!
  (
    "{}{}{}",
    color::muted( "──" ),
    color::speaker( turn.entry_type(), &format!( " {} ", turn.speaker() ) ),
    color::muted( &format!( "{}{right}", "─".repeat( fill ) ) ),
  )
}

/// `  249  17h  Claude  first line of the turn, elided to fit…`
fn compact_line( ordinal : usize, turn : &Turn< '_ >, results : &ToolResults, now : i64 ) -> String
{
  let age = relative_time( turn.timestamp(), now );
  let age = age.strip_suffix( " ago" ).unwrap_or( &age );

  let body = render_blocks_annotated( &turn.blocks(), results ).join( " / " );
  let one_line = body.split_whitespace().collect::< Vec< _ > >().join( " " );

  let prefix = format!( "{ordinal:>4}  {age:>4}  {:<6}  ", turn.speaker() );
  let room = RULE_WIDTH.saturating_sub( prefix.chars().count() ).max( 16 );

  format!( "{}{}", color::muted( &prefix ), ellipsize( &one_line, room ) )
}
