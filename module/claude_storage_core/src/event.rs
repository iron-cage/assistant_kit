//! Session events — every JSONL line, not only the conversation ones.
//!
//! [`Entry`] models the two lines a *conversation* is made of (`user` and
//! `assistant`) and [`Session::entries`] deliberately drops everything else — see
//! its "Graceful Degradation Design" note. That policy is correct for the readers
//! it serves (export, search, statistics), and this module does not change it.
//!
//! What it leaves unread is a second stream of lines describing how the session's
//! *context* was assembled: which tools were deferred, which agents and skills
//! were offered, how much of the token budget the harness reported remaining. A
//! consumer that wants those has to parse the same file with a wider schema,
//! which is what [`SessionEvent`] is.
//!
//! Two rules hold this apart from [`Entry`]:
//!
//! - **A `user`/`assistant` line is recognized, never re-parsed.** Its variant
//!   carries no message payload; [`Entry::from_json_line`] remains the single
//!   parser for conversation content. Duplicating it here would mean two schemas
//!   for one line, drifting independently.
//! - **An unrecognized line is data, not an error.** Claude Code's format grows
//!   between releases — agent sessions moved directory layout in v2.x — so an
//!   unknown `type` becomes [`EventKind::Other`] rather than a parse failure.
//!
//! [`Entry`]: crate::Entry
//! [`Entry::from_json_line`]: crate::Entry::from_json_line
//! [`Session::entries`]: crate::Session::entries

use std::path::PathBuf;

use crate::{ json::{ JsonValue, parse_json }, Error, Result };

/// One line of a session JSONL file, whatever kind of line it is.
///
/// The fields on this struct are the envelope — the metadata Claude Code repeats
/// on most lines. They are all optional because the short envelope kinds
/// (`mode`, `ai-title`, `last-prompt`) carry only a `sessionId` and their own
/// payload.
#[ derive( Debug, Clone, PartialEq ) ]
#[ non_exhaustive ]
pub struct SessionEvent
{
  /// Conversation id this line belongs to.
  pub session_id : String,

  /// Unique id of this line. Absent on the short envelope kinds.
  pub uuid : Option< String >,

  /// Parent line's uuid, for threading. Absent on the short envelope kinds.
  pub parent_uuid : Option< String >,

  /// ISO 8601 timestamp. Absent on the short envelope kinds.
  pub timestamp : Option< String >,

  /// Working directory the session runs in.
  pub cwd : Option< PathBuf >,

  /// Claude Code version that wrote this line.
  pub version : Option< String >,

  /// Whether this line belongs to a sidechain (subagent) conversation.
  pub is_sidechain : bool,

  /// What kind of line this is, and its payload.
  pub kind : EventKind,
}

/// The kind of a session line, and whatever that kind carries.
#[ derive( Debug, Clone, PartialEq ) ]
#[ non_exhaustive ]
pub enum EventKind
{
  /// A `user` conversation line.
  ///
  /// Carries no payload by design — parse the same line with
  /// [`Entry::from_json_line`] for its content.
  ///
  /// [`Entry::from_json_line`]: crate::Entry::from_json_line
  User,

  /// An `assistant` conversation line.
  ///
  /// Carries no payload, for the same reason as [`EventKind::User`].
  Assistant,

  /// A `system` telemetry line.
  System
  {
    /// Which system event this is: `turn_duration`, `compact_boundary`, …
    subtype : String,
    /// How long the turn took, on a `turn_duration` line.
    duration_ms : Option< u64 >,
    /// Messages in context, on a `turn_duration` line.
    message_count : Option< u64 >,
  },

  /// An `attachment` line — the harness injecting context into the conversation.
  Attachment( Attachment ),

  /// A `mode` line: the session's operating mode changed.
  Mode
  {
    /// The new mode, e.g. `normal` or `plan`.
    mode : String,
  },

  /// A `permission-mode` line: the session's permission mode changed.
  PermissionMode
  {
    /// The new permission mode, e.g. `acceptEdits` or `bypassPermissions`.
    permission_mode : String,
  },

  /// A `last-prompt` line: a pointer to the most recent prompt's line.
  LastPrompt
  {
    /// Uuid of the line holding that prompt.
    leaf_uuid : String,
  },

  /// An `ai-title` line: the session's generated display title.
  AiTitle
  {
    /// The title.
    title : String,
  },

  /// A `queue-operation` line: a command queued while a turn was in flight.
  QueueOperation
  {
    /// Which operation was performed.
    operation : String,
  },

  /// A line whose `type` this version does not model.
  ///
  /// Retained rather than rejected so a newer Claude Code's added line kind
  /// never fails a fold — it is counted and skipped, not lost silently.
  Other
  {
    /// The line's declared `type` field.
    kind : String,
  },
}

/// The payload of an `attachment` line.
///
/// These are what the harness injects into a session's context: the tool and
/// agent rosters, the skill catalogue, the remaining token budget. Each variant
/// keeps only the structured fields a fold needs — the human-readable `content`
/// and `addedLines` blobs that accompany several of them are not retained, since
/// the structured field beside them says the same thing without the parsing.
#[ derive( Debug, Clone, PartialEq ) ]
#[ non_exhaustive ]
pub enum Attachment
{
  /// The remaining token budget, as the harness reported it.
  TotalTokensReminder
  {
    /// Tokens left, parsed out of the reminder text.
    ///
    /// `None` when the text carried no parseable number — the reminder's
    /// wording is not a stable contract, so a changed phrasing degrades to
    /// "unknown" rather than to a wrong number.
    remaining : Option< u64 >,
  },

  /// The deferred-tool roster changed.
  DeferredToolsDelta
  {
    /// Tools newly deferred.
    added : Vec< String >,
    /// Tools no longer deferred.
    removed : Vec< String >,
    /// Tools deferred again after having been removed.
    readded : Vec< String >,
    /// MCP servers whose tools are not yet resolved.
    pending_mcp_servers : Vec< String >,
  },

  /// The subagent-type roster changed.
  AgentListingDelta
  {
    /// Agent types newly offered.
    added : Vec< String >,
    /// Agent types withdrawn.
    removed : Vec< String >,
    /// Whether this is the session's first listing rather than a delta.
    is_initial : bool,
  },

  /// The MCP instruction blocks changed.
  McpInstructionsDelta
  {
    /// Servers whose instructions were added.
    added : Vec< String >,
    /// Servers whose instructions were withdrawn.
    removed : Vec< String >,
  },

  /// The available-skill catalogue was published.
  SkillListing
  {
    /// Skill names offered.
    names : Vec< String >,
    /// Count the harness itself reported.
    ///
    /// Kept alongside `names` rather than derived from it: a disagreement
    /// between the two is a signal the listing was truncated, which a derived
    /// length would erase.
    skill_count : usize,
    /// Whether this is the session's first listing rather than a replacement.
    is_initial : bool,
  },

  /// One or more skills were invoked this turn.
  InvokedSkills
  {
    /// The skills invoked.
    skills : Vec< InvokedSkill >,
  },

  /// The pending-task list was restated.
  TaskReminder
  {
    /// How many items the list held.
    item_count : usize,
  },

  /// A background task changed state.
  TaskStatus
  {
    /// The task's id.
    task_id : String,
    /// What kind of task it is.
    task_type : String,
    /// Its new status.
    status : String,
    /// Human-readable description, when present.
    description : Option< String >,
    /// Where the task's output was written, when present.
    output_file_path : Option< String >,
  },

  /// The allowed-tool set for command execution was declared.
  CommandPermissions
  {
    /// Tools permitted.
    allowed_tools : Vec< String >,
  },

  /// A command was queued while a turn was in flight.
  QueuedCommand
  {
    /// The queued prompt text.
    prompt : Option< String >,
    /// The mode it was queued in.
    command_mode : Option< String >,
  },

  /// A file was edited outside the conversation and re-shown to the model.
  EditedTextFile
  {
    /// The file's path.
    filename : String,
  },

  /// A file carried across a `/compact` boundary by reference.
  CompactFileReference
  {
    /// The file's path.
    filename : String,
    /// The path as displayed to the user, usually relative.
    display_path : Option< String >,
  },

  /// A file's contents were attached to the conversation.
  File
  {
    /// The file's path.
    filename : String,
    /// The path as displayed to the user, usually relative.
    display_path : Option< String >,
  },

  /// The wall-clock date advanced during the session.
  DateChange
  {
    /// The new date, `YYYY-MM-DD`.
    new_date : String,
  },

  /// An attachment whose `type` this version does not model.
  Other
  {
    /// The attachment's declared `type` field.
    kind : String,
  },
}

/// One skill invoked during a turn.
#[ derive( Debug, Clone, PartialEq ) ]
#[ non_exhaustive ]
pub struct InvokedSkill
{
  /// The skill's name, as invoked.
  pub name : String,
  /// Where the skill was resolved from, e.g. `userSettings:dev`.
  pub path : Option< String >,
}

impl SessionEvent
{
  /// Parse one JSONL line into an event.
  ///
  /// Unlike [`Entry::from_json_line`], this accepts every line kind Claude Code
  /// writes. A line whose `type` is unmodelled parses successfully into
  /// [`EventKind::Other`].
  ///
  /// # Errors
  ///
  /// Returns [`Error::Parse`] if the line is not valid JSON, is not a JSON
  /// object, or carries no `type` field. Those three are structural: a line
  /// failing any of them is not a session line at all.
  ///
  /// [`Entry::from_json_line`]: crate::Entry::from_json_line
  #[ inline ]
  pub fn from_json_line( line : &str ) -> Result< Self >
  {
    let json = parse_json( line )
      .map_err( | e | Error::parse( 0, line, &format!( "JSON parse error: {e}" ) ) )?;

    if json.as_object().is_none()
    {
      return Err( Error::parse( 0, line, "expected JSON object" ) );
    }

    let type_str = json.get_str( "type" )
      .ok_or_else( || Error::parse( 0, line, "missing 'type' field" ) )?;

    Ok( Self
    {
      session_id : json.get_str( "sessionId" ).unwrap_or_default().to_string(),
      uuid : owned( &json, "uuid" ),
      parent_uuid : owned( &json, "parentUuid" ),
      timestamp : owned( &json, "timestamp" ),
      cwd : json.get_str( "cwd" ).map( PathBuf::from ),
      version : owned( &json, "version" ),
      is_sidechain : json.get_bool( "isSidechain" ).unwrap_or( false ),
      kind : Self::parse_kind( type_str, &json ),
    })
  }

  /// Dispatch on the envelope's `type` field.
  fn parse_kind( type_str : &str, json : &JsonValue ) -> EventKind
  {
    match type_str
    {
      "user" => EventKind::User,
      "assistant" => EventKind::Assistant,
      "system" => EventKind::System
      {
        subtype : json.get_str( "subtype" ).unwrap_or_default().to_string(),
        duration_ms : unsigned( json, "durationMs" ),
        message_count : unsigned( json, "messageCount" ),
      },
      "attachment" => json.get( "attachment" )
        .map_or_else
        (
          || EventKind::Other { kind : "attachment".to_string() },
          | a | EventKind::Attachment( Attachment::from_json( a ) ),
        ),
      "mode" => EventKind::Mode
      {
        mode : json.get_str( "mode" ).unwrap_or_default().to_string(),
      },
      "permission-mode" => EventKind::PermissionMode
      {
        permission_mode : json.get_str( "permissionMode" ).unwrap_or_default().to_string(),
      },
      "last-prompt" => EventKind::LastPrompt
      {
        leaf_uuid : json.get_str( "leafUuid" ).unwrap_or_default().to_string(),
      },
      "ai-title" => EventKind::AiTitle
      {
        title : json.get_str( "aiTitle" ).unwrap_or_default().to_string(),
      },
      "queue-operation" => EventKind::QueueOperation
      {
        operation : json.get_str( "operation" ).unwrap_or_default().to_string(),
      },
      other => EventKind::Other { kind : other.to_string() },
    }
  }

  /// The attachment payload, when this event is an attachment.
  #[ must_use ]
  #[ inline ]
  pub const fn attachment( &self ) -> Option< &Attachment >
  {
    match &self.kind
    {
      EventKind::Attachment( a ) => Some( a ),
      _ => None,
    }
  }

  /// Whether this event is a conversation line.
  ///
  /// True for exactly the two kinds [`crate::Entry`] models, so a caller
  /// folding events can hand those lines to the conversation parser and keep
  /// the two schemas from overlapping.
  #[ must_use ]
  #[ inline ]
  pub const fn is_conversation( &self ) -> bool
  {
    matches!( self.kind, EventKind::User | EventKind::Assistant )
  }
}

impl Attachment
{
  /// Parse an `attachment` object into its payload.
  ///
  /// An unmodelled `type`, or a missing one, becomes [`Attachment::Other`].
  #[ must_use ]
  #[ inline ]
  pub fn from_json( json : &JsonValue ) -> Self
  {
    let Some( kind ) = json.get_str( "type" )
    else
    {
      return Self::Other { kind : String::new() };
    };

    match kind
    {
      "total_tokens_reminder" => Self::TotalTokensReminder
      {
        remaining : json.get_str( "text" ).and_then( parse_leading_number ),
      },
      "deferred_tools_delta" => Self::DeferredToolsDelta
      {
        added : strings( json, "addedNames" ),
        removed : strings( json, "removedNames" ),
        readded : strings( json, "readdedNames" ),
        pending_mcp_servers : strings( json, "pendingMcpServers" ),
      },
      "agent_listing_delta" => Self::AgentListingDelta
      {
        added : strings( json, "addedTypes" ),
        removed : strings( json, "removedTypes" ),
        is_initial : json.get_bool( "isInitial" ).unwrap_or( false ),
      },
      "mcp_instructions_delta" => Self::McpInstructionsDelta
      {
        added : strings( json, "addedNames" ),
        removed : strings( json, "removedNames" ),
      },
      "skill_listing" => Self::SkillListing
      {
        names : strings( json, "names" ),
        skill_count : count( json, "skillCount" ),
        is_initial : json.get_bool( "isInitial" ).unwrap_or( false ),
      },
      "invoked_skills" => Self::InvokedSkills
      {
        skills : Self::invoked_skills( json ),
      },
      "task_reminder" => Self::TaskReminder
      {
        item_count : count( json, "itemCount" ),
      },
      "task_status" => Self::TaskStatus
      {
        task_id : json.get_str( "taskId" ).unwrap_or_default().to_string(),
        task_type : json.get_str( "taskType" ).unwrap_or_default().to_string(),
        status : json.get_str( "status" ).unwrap_or_default().to_string(),
        description : owned( json, "description" ),
        output_file_path : owned( json, "outputFilePath" ),
      },
      "command_permissions" => Self::CommandPermissions
      {
        allowed_tools : strings( json, "allowedTools" ),
      },
      "queued_command" => Self::QueuedCommand
      {
        prompt : owned( json, "prompt" ),
        command_mode : owned( json, "commandMode" ),
      },
      "edited_text_file" => Self::EditedTextFile
      {
        filename : json.get_str( "filename" ).unwrap_or_default().to_string(),
      },
      "compact_file_reference" => Self::CompactFileReference
      {
        filename : json.get_str( "filename" ).unwrap_or_default().to_string(),
        display_path : owned( json, "displayPath" ),
      },
      "file" => Self::File
      {
        filename : json.get_str( "filename" ).unwrap_or_default().to_string(),
        display_path : owned( json, "displayPath" ),
      },
      "date_change" => Self::DateChange
      {
        new_date : json.get_str( "newDate" ).unwrap_or_default().to_string(),
      },
      other => Self::Other { kind : other.to_string() },
    }
  }

  /// Extract the `skills` array of an `invoked_skills` attachment.
  ///
  /// Each entry's `content` field — the skill's full text, which the harness
  /// also injects into the prompt — is deliberately not retained: it is large,
  /// it is already on disk at `path`, and a fold needs only to know the skill
  /// ran.
  fn invoked_skills( json : &JsonValue ) -> Vec< InvokedSkill >
  {
    let Some( items ) = json.get_array( "skills" ) else { return Vec::new() };

    items.iter()
      .filter_map( | item | Some( InvokedSkill
      {
        name : item.get_str( "name" )?.to_string(),
        path : owned( item, "path" ),
      }))
      .collect()
  }
}

/// Read an optional owned string field.
fn owned( json : &JsonValue, key : &str ) -> Option< String >
{
  json.get_str( key ).map( ToString::to_string )
}

/// Read a string array, treating a missing or wrongly-typed field as empty.
///
/// Non-string elements are dropped rather than failing the whole array: one
/// unexpected element must not erase the names beside it.
fn strings( json : &JsonValue, key : &str ) -> Vec< String >
{
  json.get_array( key )
    .map( | items | items.iter().filter_map( | v | v.as_str().map( ToString::to_string ) ).collect() )
    .unwrap_or_default()
}

/// Read a non-negative count field.
fn count( json : &JsonValue, key : &str ) -> usize
{
  unsigned( json, key ).unwrap_or( 0 ).try_into().unwrap_or( usize::MAX )
}

/// Read a non-negative integer field.
///
/// JSON numbers arrive as `f64`; a negative or non-finite value is rejected
/// rather than wrapped, since every field read this way is a count or a
/// duration.
fn unsigned( json : &JsonValue, key : &str ) -> Option< u64 >
{
  let n = json.get_number( key )?;

  if n.is_finite() && n >= 0.0
  {
    // Truncation is intended: these fields are integers that JSON widened.
    #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
    Some( n as u64 )
  }
  else
  {
    None
  }
}

/// Pull the first run of ASCII digits out of `text` as a number.
///
/// Used for the token reminder, whose payload is prose — `<total_tokens>N tokens
/// left</total_tokens>` — rather than a numeric field. Wording that no longer
/// contains a number yields `None`, which the caller renders as "unknown".
fn parse_leading_number( text : &str ) -> Option< u64 >
{
  let digits : String = text
    .chars()
    .skip_while( | c | !c.is_ascii_digit() )
    .take_while( char::is_ascii_digit )
    .collect();

  digits.parse().ok()
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  #[ test ]
  fn parse_leading_number_reads_the_reminder_wording()
  {
    assert_eq!( parse_leading_number( "<total_tokens>14869351 tokens left</total_tokens>" ), Some( 14_869_351 ) );
  }

  #[ test ]
  fn parse_leading_number_without_digits_is_none()
  {
    assert_eq!( parse_leading_number( "budget unavailable" ), None );
  }

  #[ test ]
  fn unsigned_rejects_negative()
  {
    let json = parse_json( r#"{"a":-1,"b":7}"# ).unwrap();
    assert_eq!( unsigned( &json, "a" ), None );
    assert_eq!( unsigned( &json, "b" ), Some( 7 ) );
  }

  #[ test ]
  fn strings_drops_non_string_elements()
  {
    let json = parse_json( r#"{"names":["a",3,"b"]}"# ).unwrap();
    assert_eq!( strings( &json, "names" ), vec![ "a".to_string(), "b".to_string() ] );
  }
}
