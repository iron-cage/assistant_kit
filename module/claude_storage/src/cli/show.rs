//! `.show` command — display session or project details.

use core::fmt::Write as FmtWrite;
use unilang::{ VerifiedCommand, ExecutionContext, OutputData, ErrorData, ErrorCode };
use super::storage::{ create_storage, parse_project_parameter, find_session_mut };
use super::scope::{ validate_scope, resolve_scoped_projects, decode_project_display };
use super::format::format_entry_content;
use super::field_selector::FieldSelector;
use super::color;

/// Display control flags for session output.
#[ allow( clippy::struct_excessive_bools ) ]
struct SessionDisplayOptions
{
  show_entries  : bool,
  metadata_only : bool,
  show_tokens   : bool,
  fields        : Option< FieldSelector >,
  index         : Option< usize >,
}

/// Show session or project details (location-aware)
///
/// Smart behavior based on parameters:
/// - No parameters → Show current directory project (all sessions)
/// - `session_id` only → Show that session in current project
/// - project only → Show that project (all sessions)
/// - Both parameters → Show that session in that project
///
/// # Errors
///
/// Returns error if parameter combinations are invalid, storage creation
/// fails, or project/session loading fails.
///
/// # Panics
///
/// Does not panic — the `tail_count` conversion below is only reached after the
/// negative-value branch already returned, so the value is always non-negative.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn show_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let session_id_raw = cmd.get_string( "session_id" );
  // Fix(issue-030): Reject whitespace-only session_id values.
  //
  // Root cause: cli_main.rs quotes argv values containing spaces before joining into the
  // REPL command line, so `session_id::   ` (spaces only) becomes `session_id::"   "`.
  // The REPL parser preserves the whitespace-only string as a non-empty value, bypassing
  // the prior implicit empty-check that relied on the REPL splitting spaces away.
  //
  // Pitfall: Always trim-validate string parameters with a "must be non-empty" constraint.
  let session_id : Option< &str > = match session_id_raw
  {
    Some( s ) if s.trim().is_empty() =>
    {
      return Err
      (
        ErrorData::new( ErrorCode::InternalError, "session_id must be non-empty".to_string() )
      );
    }
    Some( s ) => Some( s.trim() ),
    None => None,
  };

  let project_param = cmd.get_string( "project" );
  let metadata_only = cmd.get_boolean( "show_metadata" ).unwrap_or( false );

  // `fields::`/`index::` are validated upfront (shape only — index range depends
  // on entry count, known only once storage is loaded at each call site below),
  // mirroring `tail`'s own upfront negative-value check just below.
  let field_selector = match cmd.get_string( "fields" )
  {
    Some( s ) => Some( FieldSelector::parse( s ).map_err( | e | ErrorData::new( ErrorCode::InternalError, e ) )? ),
    None => None,
  };
  let index : Option< usize > = match cmd.get_integer( "index" )
  {
    Some( v ) if v <= 0 => return Err( ErrorData::new( ErrorCode::InternalError, format!( "index must be a positive integer (1-based), got {v}" ) ) ),
    Some( v ) => Some( usize::try_from( v ).expect( "index <= 0 rejected above" ) ),
    None => None,
  };

  let opts = SessionDisplayOptions
  {
    show_entries  : cmd.get_boolean( "show_entries" ).unwrap_or( false ),
    metadata_only,
    show_tokens   : cmd.get_boolean( "show_tokens" ).unwrap_or( false ),
    fields        : field_selector,
    index,
  };

  // Note: the show_entries-requires-session_id constraint (former Fix issue-001)
  // is superseded by task 526's project-overview redesign — show_entries now
  // also controls raw-list rendering of the tail window in Cases 1/3 (see
  // docs/cli/command/03_show.md).

  // Fix(issue-022): Accept entries::1 in content mode as a no-op
  //
  // Root cause: A prior "fix" (issue-008) added an error when entries::1 was used
  // in content mode (!metadata_only), intending to prevent a
  // "garbage parameter" scenario. However, the YAML spec explicitly lists
  // `.show session_id::abc123 entries::1` as a valid example (example 6), and
  // content mode already shows all entries by default — entries::1 is a valid
  // no-op in this context.
  //
  // Pitfall: Don't add errors for parameters whose spec examples show them working
  // standalone. A no-op is preferable to an error when the parameter has no
  // additional effect in the current mode. Errors should be reserved for truly
  // incompatible combinations, not for parameters that are simply redundant.

  // scope::/path:: only affect Case 2 below — no scope is used once project::
  // is given (Cases 3/4), and Case 1 has no session to search for.
  let scope_raw = cmd.get_string( "scope" );
  let path_raw = cmd.get_string( "path" );

  // Validate `tail` before any storage access, mirroring `.tail`'s own
  // validation shape (see tail.rs) independently — see Design Decision D3.
  let tail_count_raw = cmd.get_integer( "tail" ).unwrap_or( 10 );
  if tail_count_raw < 0
  {
    return Err( ErrorData::new( ErrorCode::InternalError, "tail must be non-negative".to_string() ) );
  }
  let tail_count = usize::try_from( tail_count_raw ).expect( "tail < 0 rejected above" );

  // `detail::` is parsed locally rather than via a shared DetailLevel type:
  // task 525 never introduced one (its own `detail::` validation in projects.rs
  // is a private per-command function, not a shared type) — see task 526 checklist item C8.
  let detail_raw = cmd.get_string( "detail" ).unwrap_or( "projects" ).to_lowercase();
  let show_sessions_detail = match detail_raw.as_str()
  {
    "projects" => false,
    "sessions" => true,
    other => return Err( ErrorData::new( ErrorCode::InternalError, format!( "detail must be projects|sessions, got {other}" ) ) ),
  };

  // Smart parameter detection (4 cases)
  match ( session_id, project_param )
  {
    // Case 1: No parameters → Show current directory project
    ( None, None ) =>
    {
      show_project_for_cwd_impl( tail_count, show_sessions_detail, opts.show_entries, opts.fields.as_ref(), opts.index )
    }

    // Case 2: session_id only → Show session in current project
    ( Some( sid ), None ) =>
    {
      show_session_in_cwd_impl( sid, opts, scope_raw, path_raw )
    }

    // Case 3: project only → Show that project
    ( None, Some( proj ) ) =>
    {
      show_project_impl( proj, tail_count, show_sessions_detail, opts.show_entries, opts.fields.as_ref(), opts.index )
    }

    // Case 4: Both parameters → Show session in that project
    ( Some( sid ), Some( proj ) ) =>
    {
      show_session_in_project_impl( sid, proj, opts )
    }
  }
}

/// Helper: Show session in scope-resolved projects (default scope: `local`,
/// i.e. the current directory project — identical to the pre-scope behavior).
fn show_session_in_cwd_impl(
  session_id : &str,
  opts : SessionDisplayOptions,
  scope_raw : Option< &str >,
  path_raw : Option< &str >,
) -> core::result::Result< OutputData, ErrorData >
{
  let storage = create_storage()?;

  let scope = validate_scope( scope_raw, "local" )?;
  let scoped_projects = resolve_scoped_projects( &storage, &scope, path_raw )?;

  for project in &scoped_projects
  {
    if let Ok( output ) = format_session_output( project, session_id, &opts )
    {
      return Ok( output );
    }
  }

  Err( ErrorData::new( ErrorCode::InternalError, format!( "Session '{session_id}' not found in current directory projects" ) ) )
}

/// Helper: Show session in specific project
fn show_session_in_project_impl(
  session_id : &str,
  project_param : &str,
  opts : SessionDisplayOptions,
) -> core::result::Result< OutputData, ErrorData >
{
  let storage = create_storage()?;

  // Parse project parameter
  let proj_id = parse_project_parameter( project_param )
    .map_err( | e | ErrorData::new
    (
      ErrorCode::InternalError,
      format!( "Invalid project parameter: {e}" )
    ))?;

  let project = storage.load_project( &proj_id )
    .map_err( | e | ErrorData::new
    (
      ErrorCode::InternalError,
      format!( "Failed to load project {}: {e}", format_project_id( &proj_id ) )
    ))?;

  format_session_output( &project, session_id, &opts )
}

/// Helper: Show project for current directory
fn show_project_for_cwd_impl(
  tail_count : usize,
  show_sessions_detail : bool,
  show_entries : bool,
  fields : Option< &FieldSelector >,
  index : Option< usize >,
) -> core::result::Result< OutputData, ErrorData >
{
  let storage = create_storage()?;

  let project = storage.load_project_for_cwd()
    .map_err( | e | ErrorData::new
    (
      ErrorCode::InternalError,
      format!( "Failed to load project from current directory: {e}" )
    ))?;

  format_project_output( &project, tail_count, show_sessions_detail, show_entries, fields, index )
}

/// Helper: Show specific project
fn show_project_impl(
  project_param : &str,
  tail_count : usize,
  show_sessions_detail : bool,
  show_entries : bool,
  fields : Option< &FieldSelector >,
  index : Option< usize >,
) -> core::result::Result< OutputData, ErrorData >
{
  let storage = create_storage()?;

  // Parse project parameter
  let proj_id = parse_project_parameter( project_param )
    .map_err( | e | ErrorData::new
    (
      ErrorCode::InternalError,
      format!( "Invalid project parameter: {e}" )
    ))?;

  let project = storage.load_project( &proj_id )
    .map_err( | e | ErrorData::new
    (
      ErrorCode::InternalError,
      format!( "Failed to load project {}: {e}", format_project_id( &proj_id ) )
    ))?;

  format_project_output( &project, tail_count, show_sessions_detail, show_entries, fields, index )
}

/// Helper: Render a `ProjectId` as the plain identifier text a user typed or
/// would recognize — never `{:?}`, which leaks the enum variant name (e.g.
/// `Uuid("does-not-exist-project")` instead of `does-not-exist-project`).
fn format_project_id( id : &claude_storage_core::ProjectId ) -> String
{
  match id
  {
    claude_storage_core::ProjectId::Uuid( s ) => s.clone(),
    claude_storage_core::ProjectId::Path( p ) => p.display().to_string(),
  }
}

/// Helper: Format session output (extracted logic)
///
/// REQ-011: Content-First Display
///
/// Default shows conversation content in readable chat-log format, preceded
/// by the same key:val attribute block used by `show_metadata::1`.
/// Use `show_metadata::1` for metadata-only behavior (suppresses content).
/// `show_stat::1` is accepted but has no effect (see `show_stat` param docs).
/// Use `show_tokens::1` to add token usage counts.
fn format_session_output(
  project : &claude_storage_core::Project,
  session_id : &str,
  opts : &SessionDisplayOptions,
) -> core::result::Result< OutputData, ErrorData >
{
  // Find session
  let mut sessions = project.all_sessions()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list sessions: {e}" ) ) )?;

  // Fix(issue-011): Support partial UUID matching (first 8 chars)
  //
  // Root cause: Session lookup only did exact string matching without checking
  // if provided ID is a prefix of existing session IDs. Users expect Git-style
  // prefix matching for UUIDs (e.g., "79f86582" matches "79f86582-1435-442c-935a-13f8d874918a").
  //
  // Pitfall: ID lookups should always support prefix matching for UUIDs. Test with
  // both exact and partial IDs to ensure both work. Use production-format test data
  // (actual UUIDs) not test-friendly strings like "test-session-123".
  let session = find_session_mut( &mut sessions, session_id )?;

  // Get session stats
  let stats = session.stats()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to get session stats: {e}" ) ) )?;

  // Format output
  let mut output = String::new();

  // REQ-011: Content-first paradigm
  // - show_metadata::1 → Metadata only (suppresses conversation content)
  // - default → key:val attribute block, then conversation content in chat-log format
  // - show_stat::1 → no-op (block above already shows the equivalent fields)
  // - show_tokens::1 → Adds token usage counts section

  // Always show basic session header
  // Fix(issue-028): derive "entry"/"entries" from count; same pattern as issue-025/027.
  // Root cause: hardcoded plural "entries" produced "Session: abc (1 entries)".
  // Pitfall: "entry" is irregular — singular differs from plural root.
  let entry_noun = if stats.total_entries == 1 { "entry" } else { "entries" };
  writeln!( output, "Session: {} · {} {entry_noun}", session_id, stats.total_entries ).unwrap();

  // Metadata-only mode (show_metadata::1)
  if opts.metadata_only
  {
    write_session_metadata_block( &mut output, session, &stats );

    if opts.show_tokens
    {
      output.push_str( "\nToken Usage:\n" );
      writeln!( output, "- Input: {}", stats.total_input_tokens ).unwrap();
      writeln!( output, "- Output: {}", stats.total_output_tokens ).unwrap();
      writeln!( output, "- Cache Read: {}", stats.total_cache_read_tokens ).unwrap();
      writeln!( output, "- Cache Creation: {}", stats.total_cache_creation_tokens ).unwrap();
    }

    // Old entries::1 behavior (UUID list) for backward compat. `index::`
    // narrows this list to one line (T18/EC-8); `fields::` swaps each line
    // for the same field-projection block used elsewhere — per
    // docs/cli/command/03_show.md, it projects "any per-entry rendering"
    // show_metadata::1 includes, which covers this show_entries::1 list.
    if opts.show_entries
    {
      let entries = session.entries()
        .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to load entries: {e}" ) ) )?;

      output.push_str( "\nEntries:\n" );

      let narrowed = narrow_by_index( entries, opts.index )?;
      for ( idx, entry ) in narrowed
      {
        if let Some( selector ) = &opts.fields
        {
          output.push_str( &format_entry_fields( idx + 1, entry, selector ) );
          output.push( '\n' );
        }
        else
        {
          let role_label = color::role( &format!( "{:?}", entry.entry_type ) );
          writeln!
          (
            output,
            "{}. {role_label} · {} · {}",
            idx + 1,
            entry.uuid,
            entry.timestamp
          ).unwrap();
        }
      }
    }
  }
  // Content-first mode (default)
  else
  {
    // Key:val attribute block (same fields/helper as show_metadata::1)
    write_session_metadata_block( &mut output, session, &stats );
    output.push( '\n' );

    let entries = session.entries()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to load entries: {e}" ) ) )?;

    // `index::` narrows the entry set to one message (original position
    // preserved for display numbering); `fields::` swaps chat-log content
    // for an explicit field-by-field block.
    let narrowed = narrow_by_index( entries, opts.index )?;
    for ( pos, entry ) in narrowed
    {
      let formatted = match &opts.fields
      {
        Some( selector ) => format_entry_fields( pos + 1, entry, selector ),
        None => format_entry_content( entry, None ),
      };
      output.push_str( &formatted );
      output.push_str( "\n\n" );
    }

    // Token usage section (show_tokens::1)
    if opts.show_tokens
    {
      output.push_str( "\nToken Usage:\n" );
      writeln!( output, "- Input: {}", stats.total_input_tokens ).unwrap();
      writeln!( output, "- Output: {}", stats.total_output_tokens ).unwrap();
      writeln!( output, "- Cache Read: {}", stats.total_cache_read_tokens ).unwrap();
      writeln!( output, "- Cache Creation: {}", stats.total_cache_creation_tokens ).unwrap();
    }
  }

  Ok( OutputData::new( output, "text" ) )
}

/// Helper: Write the shared key:val attribute block (Path, Agent Session,
/// Total/User/Assistant Entries, First/Last Entry) — used by both
/// `show_metadata::1` mode and the default content-first mode.
fn write_session_metadata_block(
  output  : &mut String,
  session : &claude_storage_core::Session,
  stats   : &claude_storage_core::SessionStats,
)
{
  writeln!( output, "Path: {}", session.storage_path().display() ).unwrap();
  writeln!( output, "Agent Session: {}", stats.is_agent_session ).unwrap();
  writeln!( output, "Total Entries: {}", stats.total_entries ).unwrap();
  writeln!( output, "User Entries: {}", stats.user_entries ).unwrap();
  writeln!( output, "Assistant Entries: {}", stats.assistant_entries ).unwrap();

  if let Some( first ) = &stats.first_timestamp
  {
    writeln!( output, "First Entry: {first}" ).unwrap();
  }

  if let Some( last ) = &stats.last_timestamp
  {
    writeln!( output, "Last Entry: {last}" ).unwrap();
  }
}

/// Helper: Format one raw entry list line (index, type, uuid, timestamp).
///
/// Used by `format_project_output`'s `show_entries::1` tail-window rendering.
/// Deliberately not shared with `format_session_output`'s own equivalent
/// inline block — that function is Out of Scope for task 526 and must stay
/// byte-for-byte untouched (see task 526's Verification Checklist item C9).
fn format_entry_raw( idx : usize, entry : &claude_storage_core::Entry ) -> String
{
  let role_label = color::role( &format!( "{:?}", entry.entry_type ) );
  format!( "{}. {role_label} · {} · {}\n", idx + 1, entry.uuid, entry.timestamp )
}

/// Helper: Narrow an entry slice to the message at 1-based position `index`,
/// or leave it unchanged when `index` is `None`. Each surviving pair keeps
/// the entry's ORIGINAL 0-based position within `entries` — so display
/// numbering (`pos + 1`) still reflects where the message sits in the
/// caller's in-scope set, not "1" after narrowing away everything else.
///
/// # Errors
///
/// Returns the canonical out-of-range error (`docs/cli/param/33_index.md`)
/// when `index` exceeds `entries.len()`.
fn narrow_by_index(
  entries : &[ claude_storage_core::Entry ],
  index   : Option< usize >,
) -> core::result::Result< Vec< ( usize, &claude_storage_core::Entry ) >, ErrorData >
{
  let Some( i ) = index else { return Ok( entries.iter().enumerate().collect() ); };

  entries.get( i - 1 )
    .map( | entry | vec![ ( i - 1, entry ) ] )
    .ok_or_else( || ErrorData::new( ErrorCode::InternalError, format!( "index out of range: {i} ({} entries)", entries.len() ) ) )
}

/// Helper: Render one entry as an explicit field-by-field block — one
/// `field · value` line per requested attribute, in request order — instead
/// of `format_entry_content`'s chat-log format. `pos` is the entry's 1-based
/// display position within the caller's in-scope set (survives `index::`
/// narrowing — see `narrow_by_index`). See `docs/cli/type/15_field_selector.md`.
fn format_entry_fields( pos : usize, entry : &claude_storage_core::Entry, selector : &FieldSelector ) -> String
{
  let role_label = color::role( &format!( "{:?}", entry.entry_type ) );
  let mut out = format!( "{pos}. {role_label}" );
  for field in selector.fields()
  {
    if field == "content"
    {
      push_content_field( &mut out, entry );
    }
    else
    {
      write!( out, "\n{} · {}", color::field_name( field ), field_value( entry, field ) ).unwrap();
    }
  }
  out
}

/// Helper: Resolve one canonical field name (other than `content`, handled
/// separately by `push_content_field`) to its display value for `entry`.
/// Cross-role gaps (an assistant-only field on a `user` entry, or vice
/// versa) render as `—` rather than erroring — see EC-11/EC-13.
fn field_value( entry : &claude_storage_core::Entry, field : &str ) -> String
{
  match field
  {
    "uuid" => entry.uuid.clone(),
    "parent_uuid" => entry.parent_uuid.clone().unwrap_or_else( || "—".to_string() ),
    "timestamp" => entry.timestamp.clone(),
    "entry_type" => format!( "{:?}", entry.entry_type ),
    "cwd" => entry.cwd.display().to_string(),
    "session_id" => entry.session_id.clone(),
    "version" => entry.version.clone(),
    "git_branch" => entry.git_branch.clone().unwrap_or_else( || "—".to_string() ),
    "user_type" => entry.user_type.clone(),
    "is_sidechain" => entry.is_sidechain.to_string(),
    "thinking_level" | "thinking_disabled" => user_only_field_value( entry, field ),
    "model" | "message_id" | "stop_reason" | "stop_sequence" | "request_id" => assistant_only_field_value( entry, field ),
    _ => unreachable!( "content is handled by push_content_field; FieldSelector only yields canonical names otherwise" ),
  }
}

/// Helper: Resolve `thinking_level`/`thinking_disabled` — present only on a
/// `user` entry with `thinking_metadata` set; `—` otherwise (an `assistant`
/// entry, or a `user` entry with no `thinking_metadata`). See EC-13.
fn user_only_field_value( entry : &claude_storage_core::Entry, field : &str ) -> String
{
  let claude_storage_core::MessageContent::User( user_msg ) = &entry.message else { return "—".to_string(); };
  let Some( tm ) = &user_msg.thinking_metadata else { return "—".to_string(); };
  match field
  {
    "thinking_level" => tm.level.clone(),
    _ => tm.disabled.to_string(),
  }
}

/// Helper: Resolve `model`/`message_id`/`stop_reason`/`stop_sequence`/
/// `request_id` — present only on an `assistant` entry; `—` on a `user`
/// entry. See EC-11.
fn assistant_only_field_value( entry : &claude_storage_core::Entry, field : &str ) -> String
{
  let claude_storage_core::MessageContent::Assistant( msg ) = &entry.message else { return "—".to_string(); };
  match field
  {
    "model" => msg.model.clone(),
    "message_id" => msg.message_id.clone(),
    "stop_reason" => msg.stop_reason.clone().unwrap_or_else( || "—".to_string() ),
    "stop_sequence" => msg.stop_sequence.clone().unwrap_or_else( || "—".to_string() ),
    _ => msg.request_id.clone(),
  }
}

/// Helper: Expand the `content` field into one line per content-block
/// sub-field on an `assistant` entry (text/thinking/tool-use/tool-result —
/// see `docs/cli/type/15_field_selector.md`'s `content` entry), or a single
/// `content · {text}` line on a `user` entry (plain message text).
fn push_content_field( out : &mut String, entry : &claude_storage_core::Entry )
{
  use claude_storage_core::{ MessageContent, ContentBlock };

  match &entry.message
  {
    MessageContent::User( msg ) =>
    {
      write!( out, "\n{} · {}", color::field_name( "content" ), msg.content ).unwrap();
    }
    MessageContent::Assistant( msg ) =>
    {
      for block in &msg.content
      {
        match block
        {
          ContentBlock::Text { text } => write!( out, "\n{} · {text}", color::field_name( "content.text" ) ).unwrap(),
          ContentBlock::Thinking { thinking, signature } =>
          {
            write!( out, "\n{} · {thinking}", color::field_name( "content.thinking" ) ).unwrap();
            write!( out, "\n{} · {signature}", color::field_name( "content.thinking.signature" ) ).unwrap();
          }
          ContentBlock::ToolUse { id, name, input } =>
          {
            write!( out, "\n{} · {id}", color::field_name( "content.tool_use.id" ) ).unwrap();
            write!( out, "\n{} · {name}", color::field_name( "content.tool_use.name" ) ).unwrap();
            write!( out, "\n{} · {}", color::field_name( "content.tool_use.input" ), format_json_value( input ) ).unwrap();
          }
          ContentBlock::ToolResult { tool_use_id, content, is_error } =>
          {
            write!( out, "\n{} · {tool_use_id}", color::field_name( "content.tool_result.tool_use_id" ) ).unwrap();
            write!( out, "\n{} · {content}", color::field_name( "content.tool_result.content" ) ).unwrap();
            write!( out, "\n{} · {is_error}", color::field_name( "content.tool_result.is_error" ) ).unwrap();
          }
        }
      }
    }
  }
}

/// Helper: Render a parsed `JsonValue` back to compact JSON text — object
/// keys sorted for deterministic output, since `JsonValue::Object` is a
/// `HashMap` with no stable iteration order of its own. Used in place of
/// `{:?}` so `content.tool_use.input` prints valid JSON instead of Rust's
/// internal `Object({"path": String("...")})` debug representation. Written
/// locally rather than depending on `serde_json` — this crate deliberately
/// has no `serde_json` dependency (see `src/cli/projects.rs`).
fn format_json_value( value : &claude_storage_core::JsonValue ) -> String
{
  use claude_storage_core::JsonValue;

  match value
  {
    JsonValue::Null => "null".to_string(),
    JsonValue::Bool( b ) => b.to_string(),
    JsonValue::Number( n ) => n.to_string(),
    JsonValue::String( s ) => format_json_string( s ),
    JsonValue::Array( items ) =>
    {
      let rendered : Vec< String > = items.iter().map( format_json_value ).collect();
      format!( "[{}]", rendered.join( "," ) )
    }
    JsonValue::Object( obj ) =>
    {
      let mut keys : Vec< &String > = obj.keys().collect();
      keys.sort();
      let rendered : Vec< String > = keys.iter()
        .map( | k | format!( "{}:{}", format_json_string( k ), format_json_value( &obj[ *k ] ) ) )
        .collect();
      format!( "{{{}}}", rendered.join( "," ) )
    }
  }
}

/// Helper: JSON-escape and quote a string literal for `format_json_value`.
fn format_json_string( s : &str ) -> String
{
  let mut out = String::with_capacity( s.len() + 2 );
  out.push( '"' );
  for c in s.chars()
  {
    match c
    {
      '"' => out.push_str( "\\\"" ),
      '\\' => out.push_str( "\\\\" ),
      '\n' => out.push_str( "\\n" ),
      '\r' => out.push_str( "\\r" ),
      '\t' => out.push_str( "\\t" ),
      c if ( c as u32 ) < 0x20 => write!( out, "\\u{:04x}", c as u32 ).unwrap(),
      c => out.push( c ),
    }
  }
  out.push( '"' );
  out
}

/// Result of `scan_sessions`: earliest `first_timestamp`, latest
/// `last_timestamp`, and the index of the most-recently-active session.
type SessionScanResult = core::result::Result< ( Option< String >, Option< String >, Option< usize > ), ErrorData >;

/// Helper: Single pass computing the project's aggregate first/last entry
/// timestamps and the index of the most-recently-active session — keyed on
/// each session's `SessionStats.last_timestamp`, never filesystem mtime.
fn scan_sessions( sessions : &mut [ claude_storage_core::Session ] ) -> SessionScanResult
{
  let mut first : Option< String > = None;
  let mut last : Option< String > = None;
  let mut best_idx : Option< usize > = None;
  for ( idx, session ) in sessions.iter_mut().enumerate()
  {
    let s = session.stats()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to get session stats: {e}" ) ) )?;
    if let Some( ft ) = &s.first_timestamp
    {
      if first.as_deref().map_or( true, | f | ft.as_str() < f ) { first = Some( ft.clone() ); }
    }
    if let Some( lt ) = &s.last_timestamp
    {
      if last.as_deref().map_or( true, | l | lt.as_str() > l ) { last = Some( lt.clone() ); best_idx = Some( idx ); }
    }
  }
  Ok( ( first, last, best_idx ) )
}

/// Helper: Write the project summary block (Path, Storage, session/entry
/// counts, First/Last Entry) — shared by both `detail::` modes.
fn write_project_summary_block(
  output  : &mut String,
  project : &claude_storage_core::Project,
  stats   : &claude_storage_core::ProjectStats,
  first   : Option< &str >,
  last    : Option< &str >,
)
{
  let dir_name = project.storage_dir()
    .file_name()
    .and_then( | n | n.to_str() )
    .unwrap_or( "" )
    .to_string();
  writeln!( output, "Path: {}", decode_project_display( &dir_name ) ).unwrap();
  writeln!( output, "Storage: {}", project.storage_dir().display() ).unwrap();
  output.push( '\n' );

  writeln!( output, "Sessions: {} · Main: {} · Agent: {}",
    stats.session_count,
    stats.main_session_count,
    stats.agent_session_count
  ).unwrap();
  writeln!( output, "Total Entries: {}", stats.total_entries ).unwrap();

  let first = first.unwrap_or( "unknown" );
  let last = last.unwrap_or( "unknown" );
  writeln!( output, "First Entry: {first}" ).unwrap();
  writeln!( output, "Last Entry: {last}" ).unwrap();
}

/// Helper: Render the most-recently-active session's tail window (last
/// `tail_count` entries; `0` = uncapped) — formatted chat-log content by
/// default, a field-projection block when `fields` is set, or a raw
/// uuid/type/timestamp list when `show_entries` is set. `index` further
/// narrows to exactly one message, counted within this tail-windowed slice
/// (not the full session) — see `docs/cli/param/33_index.md`.
/// Mirrors `tail.rs`'s own slicing idiom independently (no shared helper).
fn write_tail_window(
  output       : &mut String,
  session      : &mut claude_storage_core::Session,
  tail_count   : usize,
  show_entries : bool,
  fields       : Option< &FieldSelector >,
  index        : Option< usize >,
) -> core::result::Result< (), ErrorData >
{
  let entries = session.entries()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to load entries: {e}" ) ) )?;

  let sliced = if tail_count == 0 || tail_count >= entries.len()
  {
    &entries[ .. ]
  }
  else
  {
    &entries[ entries.len() - tail_count.. ]
  };

  let narrowed = narrow_by_index( sliced, index )?;

  for ( idx, entry ) in narrowed
  {
    if let Some( selector ) = fields
    {
      output.push_str( &format_entry_fields( idx + 1, entry, selector ) );
      output.push_str( "\n\n" );
    }
    else if show_entries
    {
      output.push_str( &format_entry_raw( idx, entry ) );
    }
    else
    {
      output.push_str( &format_entry_content( entry, None ) );
      output.push_str( "\n\n" );
    }
  }
  Ok( () )
}

/// Helper: Append the full per-session list (`detail::sessions`) — ID, entry
/// count, last timestamp. Rendering logic unchanged from the pre-task
/// implementation; only its call site is now conditionally gated.
fn write_per_session_list(
  output   : &mut String,
  sessions : &mut [ claude_storage_core::Session ],
) -> core::result::Result< (), ErrorData >
{
  output.push_str( "Sessions:\n" );

  for session in sessions.iter_mut()
  {
    let session_stats = session.stats()
      .map_err( | e | ErrorData::new
      (
        ErrorCode::InternalError,
        format!( "Failed to get session stats: {e}" )
      ))?;

    // Standard: ID + entry count + last timestamp
    let last = session_stats.last_timestamp
      .unwrap_or_else( || "unknown".to_string() );

    // Fix(issue-028): derive "entry"/"entries" from count — sibling of session_count fix.
    // Root cause: hardcoded "entries" produced "(1 entries, last: ...)".
    // Pitfall: "entry" is irregular — singular differs from plural root.
    let e_noun = if session_stats.total_entries == 1 { "entry" } else { "entries" };
    writeln!( output, "  - {} · {} {e_noun} · last: {}",
      session.id(),
      session_stats.total_entries,
      last
    ).unwrap();
  }
  Ok( () )
}

/// Helper: Format project output (extracted logic)
fn format_project_output(
  project : &claude_storage_core::Project,
  tail_count : usize,
  show_sessions_detail : bool,
  show_entries : bool,
  fields : Option< &FieldSelector >,
  index : Option< usize >,
) -> core::result::Result< OutputData, ErrorData >
{
  let stats = project.project_stats()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to get project stats: {e}" ) ) )?;

  let mut sessions = project.sessions()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list sessions: {e}" ) ) )?;

  let mut output = String::new();

  // Zero-sessions edge case: summary only, skip scan/tail/list entirely.
  // index:: against an empty project still must reject — narrow_by_index
  // on an empty slice yields the canonical out-of-range error (T21).
  if sessions.is_empty()
  {
    if let Some( i ) = index
    {
      narrow_by_index( &[], Some( i ) )?;
    }
    write_project_summary_block( &mut output, project, &stats, None, None );
    return Ok( OutputData::new( output.trim_end().to_string(), "text" ) );
  }

  let ( first, last, best_idx ) = scan_sessions( &mut sessions )?;
  write_project_summary_block( &mut output, project, &stats, first.as_deref(), last.as_deref() );
  output.push( '\n' );

  if let Some( idx ) = best_idx
  {
    write_tail_window( &mut output, &mut sessions[ idx ], tail_count, show_entries, fields, index )?;
  }

  // cli_main.rs's println! contract requires content with no trailing '\n' —
  // trim here so the two `detail::` modes share an identical checkpoint,
  // regardless of how many blank lines the tail window itself ends with.
  let mut output = output.trim_end().to_string();

  if show_sessions_detail
  {
    output.push( '\n' );
    write_per_session_list( &mut output, &mut sessions )?;
    output = output.trim_end().to_string();
  }

  Ok( OutputData::new( output, "text" ) )
}
