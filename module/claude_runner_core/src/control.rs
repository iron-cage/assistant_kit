//! Agent SDK `stream-json` control protocol — bidirectional session control.
//!
//! [`ControlSession`] wraps a `claude` subprocess spawned with a live, held-open stdin pipe
//! (via [`crate::ClaudeCommand::spawn_control_session`]) and implements full parameter parity
//! with the Agent SDK's `Query` interface: all 25 control methods (task 415).
//!
//! Wire shapes here are evidenced against a captured real-SDK session, not inferred solely
//! from the SDK's TypeScript type definitions — see `tests/fixtures/sdk_control_capture/`
//! (task 415 Phase 0) and that directory's own `readme.md` for the full capture methodology
//! and key findings this module's design directly reflects:
//! - `initialization_result`, `supported_commands`, `supported_models`, `supported_agents`,
//!   `account_info` are cached accessors — they issue no wire `control_request` of their own.
//! - `stream_input` has no control-envelope wire shape at all — it is a plain `user`-typed
//!   message line, with no matching `control_response` to await.
//! - `reconnect_mcp_server`/`toggle_mcp_server` return a real subprocess-side error when
//!   targeting an in-process MCP server ("SDK servers should be handled in print.ts").

use error_tools::{ Result, Error };
use std::collections::HashMap;
use std::io::{ BufRead, Write };
use std::sync::{ Arc, Mutex };
use core::sync::atomic::{ AtomicBool, AtomicU64, Ordering };
use std::sync::mpsc;
use core::time::Duration;

use crate::types::
{
  AccountInfo, ContextUsageResult, InitializeResult, McpPermissionOverrideMode,
  McpServerStatusEntry, ReadFileEncoding, ReadFileResult, ReloadPluginsResult,
  ReloadSkillsResult, RewindFilesResult, SetMcpPermissionModeOverrideResult, SetMcpServersResult,
  ThinkingDisplay,
};

/// Default per-request timeout: how long a control method waits for its matching
/// `control_response` before surfacing a timeout error instead of hanging indefinitely
/// (Test Matrix IT-8).
const DEFAULT_REQUEST_TIMEOUT : Duration = Duration::from_secs( 30 );

/// Outcome of a single dispatched `control_request`, routed to its waiter by `request_id`.
enum WireOutcome
{
  Success( serde_json::Value ),
  Error( String ),
}

/// A live, bidirectionally-controllable `claude` subprocess session.
///
/// Returned by [`crate::ClaudeCommand::spawn_control_session`]. Owns the subprocess's stdin
/// (for control requests and streamed user turns) and a background thread demultiplexing
/// stdout: `control_response` lines are routed to their matching in-flight request by
/// `request_id`; every other line (system/assistant/result/etc.) is forwarded unparsed via
/// [`ControlSession::recv_message`]/[`ControlSession::try_recv_message`] — full `SDKMessage`
/// typing is out of this task's scope (see `contract/sdk/docs/api/005_sdk_message_stream.md`).
pub struct ControlSession
{
  child : std::process::Child,
  stdin : Mutex< Option< std::process::ChildStdin > >,
  pending : Arc< Mutex< HashMap< String, mpsc::Sender< WireOutcome > > > >,
  /// Last-received `initialize`-shaped payload (from session startup or `reinitialize()`),
  /// backing the 5 cached-accessor methods.
  cache : Arc< Mutex< Option< serde_json::Value > > >,
  messages : Mutex< mpsc::Receiver< serde_json::Value > >,
  broken : Arc< Mutex< Option< String > > >,
  reader : Option< std::thread::JoinHandle< () > >,
  next_id : AtomicU64,
  closed : AtomicBool,
  timeout : Duration,
}

impl core::fmt::Debug for ControlSession
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
    f.debug_struct( "ControlSession" )
      .field( "pid", &self.child.id() )
      .field( "closed", &self.closed.load( Ordering::Relaxed ) )
      .finish_non_exhaustive()
  }
}

impl ControlSession
{
  /// Wrap an already-spawned `Child` (stdin/stdout must be `Stdio::piped()`) into a
  /// [`ControlSession`], starting the background stdout-demultiplexing thread.
  ///
  /// Used exclusively by [`crate::ClaudeCommand::spawn_control_session`] — not part of the
  /// Single Execution Point (`Command::new("claude")` remains solely in `build_command()`);
  /// this only takes ownership of a `Child` that method already spawned.
  pub( crate ) fn from_child( mut child : std::process::Child ) -> Result< Self >
  {
    let stdin = child.stdin.take()
      .ok_or_else( || Error::msg( "spawned control session child has no stdin pipe" ) )?;
    let stdout = child.stdout.take()
      .ok_or_else( || Error::msg( "spawned control session child has no stdout pipe" ) )?;

    let pending : Arc< Mutex< HashMap< String, mpsc::Sender< WireOutcome > > > > =
      Arc::new( Mutex::new( HashMap::new() ) );
    let cache : Arc< Mutex< Option< serde_json::Value > > > = Arc::new( Mutex::new( None ) );
    let broken : Arc< Mutex< Option< String > > > = Arc::new( Mutex::new( None ) );

    let ( message_tx, message_rx ) = mpsc::channel();

    let reader_pending = Arc::clone( &pending );
    let reader_cache = Arc::clone( &cache );
    let reader_broken = Arc::clone( &broken );

    let reader = std::thread::spawn( move ||
    {
      read_loop( stdout, &reader_pending, &reader_cache, &message_tx );
      // Stdout closed (subprocess exited or pipe broken) — fail every still-pending waiter
      // immediately rather than letting each one wait out its own timeout (Test Matrix IT-9).
      *reader_broken.lock().unwrap() = Some( "subprocess stdout closed (process likely exited)".to_string() );
      let mut pending_map = reader_pending.lock().unwrap();
      for ( _, sender ) in pending_map.drain()
      {
        let _ = sender.send( WireOutcome::Error( "subprocess exited before responding".to_string() ) );
      }
    } );

    Ok( Self
    {
      child,
      stdin : Mutex::new( Some( stdin ) ),
      pending,
      cache,
      messages : Mutex::new( message_rx ),
      broken,
      reader : Some( reader ),
      next_id : AtomicU64::new( 0 ),
      closed : AtomicBool::new( false ),
      timeout : DEFAULT_REQUEST_TIMEOUT,
    } )
  }

  /// Override the per-request timeout (default: 30s). Primarily for tests that need a
  /// shorter bound (e.g. confirming a malformed-response scenario surfaces quickly).
  #[ inline ]
  pub fn set_request_timeout( &mut self, timeout : Duration )
  {
    self.timeout = timeout;
  }

  /// Receive the next raw, unparsed non-control message (system/assistant/result/etc.),
  /// blocking up to `timeout`. Returns `None` on timeout or if the session ended.
  ///
  /// Raw passthrough for test/caller synchronization (e.g. waiting for `system:init` before
  /// issuing control methods) — not typed `SDKMessage` parsing, which is out of scope here.
  ///
  /// # Panics
  ///
  /// Panics if the internal message-channel mutex is poisoned (a prior thread panicked
  /// while holding the lock).
  #[ inline ]
  #[ must_use ]
  pub fn recv_message( &self, timeout : Duration ) -> Option< serde_json::Value >
  {
    self.messages.lock().unwrap().recv_timeout( timeout ).ok()
  }

  /// Non-blocking variant of [`recv_message`](Self::recv_message).
  ///
  /// # Panics
  ///
  /// Panics if the internal message-channel mutex is poisoned (a prior thread panicked
  /// while holding the lock).
  #[ inline ]
  #[ must_use ]
  pub fn try_recv_message( &self ) -> Option< serde_json::Value >
  {
    self.messages.lock().unwrap().try_recv().ok()
  }

  fn next_request_id( &self ) -> String
  {
    format!( "clr-req-{}", self.next_id.fetch_add( 1, Ordering::Relaxed ) )
  }

  /// Core send/await primitive: wrap `params` (must serialize to a JSON object) with
  /// `subtype` and a fresh `request_id` into a `control_request` envelope, write it to
  /// stdin, and block until the matching `control_response` arrives (or times out, or the
  /// session breaks). Returns the raw `response.response` payload (or `Value::Null` if the
  /// wire response omitted it, e.g. `seed_read_state`).
  fn send_request< Req : serde::Serialize > ( &self, subtype : &str, params : &Req ) -> Result< serde_json::Value >
  {
    if let Some( reason ) = self.broken.lock().unwrap().clone()
    {
      return Err( Error::msg( format!( "control session broken: {reason}" ) ) );
    }

    let mut request_obj = serde_json::to_value( params )
      .map_err( | e | Error::msg( format!( "failed to serialize '{subtype}' request params: {e}" ) ) )?;
    if let serde_json::Value::Object( ref mut map ) = request_obj
    {
      map.insert( "subtype".to_string(), serde_json::Value::String( subtype.to_string() ) );
    }

    let request_id = self.next_request_id();
    let envelope = serde_json::json!(
    {
      "request_id" : request_id,
      "type" : "control_request",
      "request" : request_obj,
    } );
    let mut line = serde_json::to_string( &envelope )
      .map_err( | e | Error::msg( format!( "failed to serialize '{subtype}' control request: {e}" ) ) )?;
    line.push( '\n' );

    let ( tx, rx ) = mpsc::channel();
    self.pending.lock().unwrap().insert( request_id.clone(), tx );

    {
      let mut guard = self.stdin.lock().unwrap();
      let stdin = guard.as_mut()
        .ok_or_else( || Error::msg( "control session already closed" ) )?;
      stdin.write_all( line.as_bytes() )
        .map_err( | e | Error::msg( format!( "failed to write '{subtype}' request to stdin: {e}" ) ) )?;
      stdin.flush()
        .map_err( | e | Error::msg( format!( "failed to flush '{subtype}' request to stdin: {e}" ) ) )?;
    }

    match rx.recv_timeout( self.timeout )
    {
      Ok( WireOutcome::Success( value ) ) => Ok( value ),
      Ok( WireOutcome::Error( message ) ) => Err( Error::msg( format!( "control request '{subtype}' failed: {message}" ) ) ),
      Err( _ ) =>
      {
        self.pending.lock().unwrap().remove( &request_id );
        Err( Error::msg( format!( "control request '{subtype}' timed out after {:?}", self.timeout ) ) )
      }
    }
  }

  /// Deserialize a `send_request` payload into a concrete response type, with a clear typed
  /// error (not a panic) on shape mismatch (Test Matrix IT-8).
  fn parse_response< T : serde::de::DeserializeOwned > ( subtype : &str, value : serde_json::Value ) -> Result< T >
  {
    serde_json::from_value( value )
      .map_err( | e | Error::msg( format!( "malformed '{subtype}' control response: {e}" ) ) )
  }

  /// Refresh the `initialize`-shaped cache backing the 5 cached-accessor methods.
  fn set_cache( &self, value : serde_json::Value )
  {
    *self.cache.lock().unwrap() = Some( value );
  }

  fn cached_field( &self, field : &str ) -> Result< serde_json::Value >
  {
    let guard = self.cache.lock().unwrap();
    let cached = guard.as_ref()
      .ok_or_else( || Error::msg( "no cached initialize response yet — call reinitialize() or wait for session startup" ) )?;
    cached.get( field )
      .cloned()
      .ok_or_else( || Error::msg( format!( "cached initialize response has no '{field}' field" ) ) )
  }

  // ==========================================================================
  // The 25 `Query` control methods
  // ==========================================================================

  /// `interrupt()` — cancel the current turn mid-generation. Wire subtype `interrupt`,
  /// no request fields, no meaningful response payload (confirmed empty in captured trace).
  ///
  /// # Errors
  ///
  /// Returns `Err` if the session is broken, the request cannot be written to stdin, or
  /// the subprocess does not respond before the timeout elapses.
  #[ inline ]
  pub fn interrupt( &self ) -> Result< () >
  {
    self.send_request( "interrupt", &serde_json::json!( {} ) )?;
    Ok( () )
  }

  /// `rewindFiles(userMessageId, { dryRun })` — rewind file edits back to a given user
  /// message's state. Wire subtype `rewind_files`, fields `user_message_id`/`dry_run`.
  ///
  /// # Errors
  ///
  /// Returns `Err` if the session is broken, the request cannot be written to stdin, the
  /// subprocess does not respond before the timeout elapses, or the response payload
  /// doesn't match [`RewindFilesResult`]'s shape.
  #[ inline ]
  pub fn rewind_files( &self, user_message_id : &str, dry_run : bool ) -> Result< RewindFilesResult >
  {
    let params = serde_json::json!( { "user_message_id" : user_message_id, "dry_run" : dry_run } );
    let value = self.send_request( "rewind_files", &params )?;
    Self::parse_response( "rewind_files", value )
  }

  /// `setPermissionMode(mode)` — change the live permission mode. Wire subtype
  /// `set_permission_mode`, field `mode`.
  ///
  /// # Errors
  ///
  /// Returns `Err` if the session is broken, the request cannot be written to stdin, or
  /// the subprocess does not respond before the timeout elapses.
  #[ inline ]
  pub fn set_permission_mode( &self, mode : crate::types::PermissionMode ) -> Result< () >
  {
    let params = serde_json::json!( { "mode" : mode.as_str() } );
    self.send_request( "set_permission_mode", &params )?;
    Ok( () )
  }

  /// `setMcpPermissionModeOverride(serverName, mode)` — pin or clear a per-MCP-server
  /// permission override. Wire subtype `set_mcp_permission_mode_override`, fields
  /// `serverName`/`mode` (`mode: null` clears the override).
  ///
  /// # Errors
  ///
  /// Returns `Err` if the session is broken, the request cannot be written to stdin, the
  /// subprocess does not respond before the timeout elapses, or a non-null response
  /// payload doesn't match [`SetMcpPermissionModeOverrideResult`]'s shape.
  #[ inline ]
  pub fn set_mcp_permission_mode_override(
    &self, server_name : &str, mode : Option< McpPermissionOverrideMode >
  ) -> Result< SetMcpPermissionModeOverrideResult >
  {
    let mode_value = mode.map_or( serde_json::Value::Null, | m | serde_json::Value::String( m.as_str().to_string() ) );
    let params = serde_json::json!( { "serverName" : server_name, "mode" : mode_value } );
    let value = self.send_request( "set_mcp_permission_mode_override", &params )?;
    if value.is_null()
    {
      return Ok( SetMcpPermissionModeOverrideResult::default() );
    }
    Self::parse_response( "set_mcp_permission_mode_override", value )
  }

  /// `setModel(model?)` — change the live model. Wire subtype `set_model`, optional field
  /// `model` (omitted entirely when `None`, confirmed in captured trace).
  ///
  /// # Errors
  ///
  /// Returns `Err` if the session is broken, the request cannot be written to stdin, or
  /// the subprocess does not respond before the timeout elapses.
  #[ inline ]
  pub fn set_model( &self, model : Option< &str > ) -> Result< () >
  {
    let params = match model
    {
      Some( m ) => serde_json::json!( { "model" : m } ),
      None => serde_json::json!( {} ),
    };
    self.send_request( "set_model", &params )?;
    Ok( () )
  }

  /// `setMaxThinkingTokens(n, thinkingDisplay?)` — change the live thinking-token budget.
  /// `@deprecated` on the SDK (steers callers to `query()`'s `thinking` option) but still
  /// functional. Wire subtype `set_max_thinking_tokens`, fields `max_thinking_tokens`/
  /// `thinking_display`.
  ///
  /// # Errors
  ///
  /// Returns `Err` if the session is broken, the request cannot be written to stdin, or
  /// the subprocess does not respond before the timeout elapses.
  #[ inline ]
  pub fn set_max_thinking_tokens(
    &self, max_thinking_tokens : Option< u64 >, thinking_display : Option< ThinkingDisplay >
  ) -> Result< () >
  {
    let params = serde_json::json!(
    {
      "max_thinking_tokens" : max_thinking_tokens,
      "thinking_display" : thinking_display.map( ThinkingDisplay::as_str ),
    } );
    self.send_request( "set_max_thinking_tokens", &params )?;
    Ok( () )
  }

  /// `applyFlagSettings(settings)` — apply named settings live, without a session restart.
  /// Wire subtype `apply_flag_settings`, field `settings`. `settings`'s shape mirrors the
  /// CLI's own settings.json schema (out of this task's scope to model) — accepted as
  /// `serde_json::Value` by design.
  ///
  /// # Errors
  ///
  /// Returns `Err` if the session is broken, the request cannot be written to stdin, or
  /// the subprocess does not respond before the timeout elapses.
  #[ inline ]
  pub fn apply_flag_settings( &self, settings : serde_json::Value ) -> Result< () >
  {
    let params = serde_json::json!( { "settings" : settings } );
    self.send_request( "apply_flag_settings", &params )?;
    Ok( () )
  }

  /// `initializationResult()` — return the session's initialization payload.
  ///
  /// **Cached accessor, not a wire round-trip** (confirmed: no dedicated wire subtype
  /// exists) — returns the last-received `initialize`-shaped response, populated during
  /// session startup or refreshed by [`reinitialize`](Self::reinitialize).
  ///
  /// # Errors
  ///
  /// Returns `Err` if no cached `initialize` response is available yet (call
  /// [`reinitialize`](Self::reinitialize) or wait for session startup first), or if the
  /// cached payload doesn't match [`InitializeResult`]'s shape.
  ///
  /// # Panics
  ///
  /// Panics if the internal cache mutex is poisoned (a prior thread panicked while
  /// holding the lock).
  #[ inline ]
  pub fn initialization_result( &self ) -> Result< InitializeResult >
  {
    let guard = self.cache.lock().unwrap();
    let cached = guard.as_ref()
      .ok_or_else( || Error::msg( "no cached initialize response yet — call reinitialize() or wait for session startup" ) )?
      .clone();
    drop( guard );
    Self::parse_response( "initialize (cached)", cached )
  }

  /// `reinitialize()` — re-initialize the session. Unlike the 5 cached accessors, this
  /// **is** a real wire round-trip: confirmed wire subtype is `initialize` again (same
  /// subtype as session startup). Refreshes the cache backing the 5 cached accessors.
  ///
  /// A pure-Rust caller has no SDK-side in-process MCP server registry or `query()`-level
  /// system-prompt option to resupply (those are Strategy-A-only concepts, explicitly out
  /// of scope — `contract/sdk/docs/pattern/001_in_process_custom_tool.md`), so this sends
  /// `{subtype: "initialize"}` with no additional fields.
  ///
  /// # Errors
  ///
  /// Returns `Err` if the session is broken, the request cannot be written to stdin, the
  /// subprocess does not respond before the timeout elapses, or the response payload
  /// doesn't match [`InitializeResult`]'s shape.
  #[ inline ]
  pub fn reinitialize( &self ) -> Result< InitializeResult >
  {
    let value = self.send_request( "initialize", &serde_json::json!( {} ) )?;
    self.set_cache( value.clone() );
    Self::parse_response( "initialize", value )
  }

  /// `supportedCommands()` — list of supported slash commands.
  ///
  /// **Cached accessor, not a wire round-trip** — returns the `commands` field of the
  /// cached `initialize`-shaped payload (same cache as [`initialization_result`](Self::initialization_result)).
  ///
  /// # Errors
  ///
  /// Returns `Err` if no cached `initialize` response is available yet (call
  /// [`reinitialize`](Self::reinitialize) or wait for session startup first).
  #[ inline ]
  pub fn supported_commands( &self ) -> Result< serde_json::Value >
  {
    self.cached_field( "commands" )
  }

  /// `supportedModels()` — list of supported models.
  ///
  /// **Cached accessor, not a wire round-trip** — returns the `models` field of the cached
  /// `initialize`-shaped payload.
  ///
  /// # Errors
  ///
  /// Returns `Err` if no cached `initialize` response is available yet (call
  /// [`reinitialize`](Self::reinitialize) or wait for session startup first).
  #[ inline ]
  pub fn supported_models( &self ) -> Result< serde_json::Value >
  {
    self.cached_field( "models" )
  }

  /// `supportedAgents()` — list of supported subagents.
  ///
  /// **Cached accessor, not a wire round-trip** — returns the `agents` field of the cached
  /// `initialize`-shaped payload.
  ///
  /// # Errors
  ///
  /// Returns `Err` if no cached `initialize` response is available yet (call
  /// [`reinitialize`](Self::reinitialize) or wait for session startup first).
  #[ inline ]
  pub fn supported_agents( &self ) -> Result< serde_json::Value >
  {
    self.cached_field( "agents" )
  }

  /// `mcpServerStatus()` — current MCP server status list. Wire subtype `mcp_status`, no
  /// request fields; response `{mcpServers: [...]}`.
  ///
  /// # Errors
  ///
  /// Returns `Err` if the session is broken, the request cannot be written to stdin, the
  /// subprocess does not respond before the timeout elapses, or the response payload
  /// doesn't match the expected `{mcpServers: [...]}` shape.
  #[ inline ]
  pub fn mcp_server_status( &self ) -> Result< Vec< McpServerStatusEntry > >
  {
    let value = self.send_request( "mcp_status", &serde_json::json!( {} ) )?;
    let servers = value.get( "mcpServers" ).cloned()
      .ok_or_else( || Error::msg( "malformed 'mcp_status' control response: missing 'mcpServers'" ) )?;
    Self::parse_response( "mcp_status", servers )
  }

  /// `getContextUsage()` — context-window usage breakdown by category. Wire subtype
  /// `get_context_usage`, no request fields.
  ///
  /// # Errors
  ///
  /// Returns `Err` if the session is broken, the request cannot be written to stdin, or
  /// the subprocess does not respond before the timeout elapses.
  #[ inline ]
  pub fn get_context_usage( &self ) -> Result< ContextUsageResult >
  {
    self.send_request( "get_context_usage", &serde_json::json!( {} ) )
  }

  /// `readFile(path, options?)` — read a file through the session's own file-access layer.
  /// Wire subtype `read_file`, fields `path`/optional `maxBytes`/`encoding`. Returns `None`
  /// on permission denial or missing file, matching the SDK's own
  /// `Promise<SDKControlReadFileResponse | null>` contract.
  ///
  /// **Confirmed empirically (task 415 Phase 2 probing):** the real CLI reports a missing or
  /// unreadable file as a wire-level `control_response` error (e.g. `"ENOENT: no such file or
  /// directory, open '...'"` or `"read denied: ..."`), never a null/empty success payload —
  /// so this method distinguishes that specific, file-scoped error from every other failure
  /// mode (broken session, stdin write failure, timed-out request) and folds only the former
  /// into `Ok(None)`; the latter still propagate as `Err`.
  ///
  /// # Errors
  ///
  /// Returns `Err` if the session is broken, the request cannot be written to stdin, the
  /// subprocess does not respond before the timeout elapses, or a non-null response
  /// payload doesn't match [`ReadFileResult`]'s shape.
  #[ inline ]
  pub fn read_file(
    &self, path : &str, max_bytes : Option< u64 >, encoding : Option< ReadFileEncoding >
  ) -> Result< Option< ReadFileResult > >
  {
    let params = serde_json::json!(
    {
      "path" : path,
      "maxBytes" : max_bytes,
      "encoding" : encoding.map( ReadFileEncoding::as_str ),
    } );
    match self.send_request( "read_file", &params )
    {
      Ok( value ) if value.is_null() => Ok( None ),
      Ok( value ) => Self::parse_response( "read_file", value ).map( Some ),
      Err( e ) if e.to_string().starts_with( "control request 'read_file' failed:" ) => Ok( None ),
      Err( e ) => Err( e ),
    }
  }

  /// `reloadPlugins()` — reload plugins from disk. Wire subtype `reload_plugins`, no
  /// request fields; refreshes the cache's `commands`/`agents` fields too (response
  /// includes both) but is not itself the cache-refresh path `reinitialize()` is.
  ///
  /// # Errors
  ///
  /// Returns `Err` if the session is broken, the request cannot be written to stdin, the
  /// subprocess does not respond before the timeout elapses, or the response payload
  /// doesn't match [`ReloadPluginsResult`]'s shape.
  #[ inline ]
  pub fn reload_plugins( &self ) -> Result< ReloadPluginsResult >
  {
    let value = self.send_request( "reload_plugins", &serde_json::json!( {} ) )?;
    Self::parse_response( "reload_plugins", value )
  }

  /// `reloadSkills()` — reload skills from disk. Wire subtype `reload_skills`, no request
  /// fields.
  ///
  /// # Errors
  ///
  /// Returns `Err` if the session is broken, the request cannot be written to stdin, the
  /// subprocess does not respond before the timeout elapses, or the response payload
  /// doesn't match [`ReloadSkillsResult`]'s shape.
  #[ inline ]
  pub fn reload_skills( &self ) -> Result< ReloadSkillsResult >
  {
    let value = self.send_request( "reload_skills", &serde_json::json!( {} ) )?;
    Self::parse_response( "reload_skills", value )
  }

  /// `accountInfo()` — account information.
  ///
  /// **Cached accessor, not a wire round-trip** — returns the `account` field of the cached
  /// `initialize`-shaped payload.
  ///
  /// # Errors
  ///
  /// Returns `Err` if no cached `initialize` response is available yet (call
  /// [`reinitialize`](Self::reinitialize) or wait for session startup first), or if the
  /// cached `account` field doesn't match [`AccountInfo`]'s shape.
  #[ inline ]
  pub fn account_info( &self ) -> Result< AccountInfo >
  {
    let value = self.cached_field( "account" )?;
    Self::parse_response( "initialize.account (cached)", value )
  }

  /// `seedReadState(path, mtime)` — seed the session's read-tracking cache so a subsequent
  /// Edit doesn't fail "file not read yet". Wire subtype `seed_read_state`, fields
  /// `path`/`mtime`; wire response omits `response` entirely (confirmed in captured trace).
  ///
  /// # Errors
  ///
  /// Returns `Err` if the session is broken, the request cannot be written to stdin, or
  /// the subprocess does not respond before the timeout elapses.
  #[ inline ]
  pub fn seed_read_state( &self, path : &str, mtime : u64 ) -> Result< () >
  {
    let params = serde_json::json!( { "path" : path, "mtime" : mtime } );
    self.send_request( "seed_read_state", &params )?;
    Ok( () )
  }

  /// `reconnectMcpServer(serverName)` — reconnect a named MCP server. Wire subtype
  /// `mcp_reconnect`, field `serverName`.
  ///
  /// **Confirmed caveat:** against an in-process (SDK-registered) server, the real
  /// subprocess returns error `"SDK servers should be handled in print.ts"` — surfaced as
  /// `Err`, not a hang. Use a process-based (stdio/http) MCP server for happy-path
  /// reconnect semantics.
  ///
  /// # Errors
  ///
  /// Returns `Err` if the session is broken, the request cannot be written to stdin, the
  /// subprocess does not respond before the timeout elapses, or (per the confirmed
  /// caveat above) the target is an in-process MCP server.
  #[ inline ]
  pub fn reconnect_mcp_server( &self, server_name : &str ) -> Result< () >
  {
    let params = serde_json::json!( { "serverName" : server_name } );
    self.send_request( "mcp_reconnect", &params )?;
    Ok( () )
  }

  /// `toggleMcpServer(serverName, enabled)` — enable/disable a named MCP server. Wire
  /// subtype `mcp_toggle`, fields `serverName`/`enabled`. Same in-process-server caveat as
  /// [`reconnect_mcp_server`](Self::reconnect_mcp_server).
  ///
  /// # Errors
  ///
  /// Returns `Err` if the session is broken, the request cannot be written to stdin, the
  /// subprocess does not respond before the timeout elapses, or (per the confirmed
  /// caveat above) the target is an in-process MCP server.
  #[ inline ]
  pub fn toggle_mcp_server( &self, server_name : &str, enabled : bool ) -> Result< () >
  {
    let params = serde_json::json!( { "serverName" : server_name, "enabled" : enabled } );
    self.send_request( "mcp_toggle", &params )?;
    Ok( () )
  }

  /// `setMcpServers(servers)` — replace/update the MCP server set. Wire subtype
  /// `mcp_set_servers`, field `servers` (server name → config object); response
  /// `{added, removed, errors}`.
  ///
  /// # Errors
  ///
  /// Returns `Err` if the session is broken, the request cannot be written to stdin, the
  /// subprocess does not respond before the timeout elapses, or the response payload
  /// doesn't match [`SetMcpServersResult`]'s shape.
  #[ inline ]
  pub fn set_mcp_servers( &self, servers : serde_json::Value ) -> Result< SetMcpServersResult >
  {
    let params = serde_json::json!( { "servers" : servers } );
    let value = self.send_request( "mcp_set_servers", &params )?;
    Self::parse_response( "mcp_set_servers", value )
  }

  /// `streamInput(stream)` — inject a further user turn into the already-running session.
  ///
  /// **Confirmed: not a `control_request` at all** — the wire evidence is a plain
  /// `{type: "user", message: {...}, parent_tool_use_id: null}` line, identical in shape to
  /// an ordinary prompt turn, with no matching `control_response` to await. This method
  /// resolves once the write succeeds — it does not block for a wire acknowledgment that
  /// doesn't exist (captured trace: this call did not resolve within a 15s client-side
  /// timeout under a different, contending sequencing — see
  /// `tests/fixtures/sdk_control_capture/readme.md`).
  ///
  /// # Errors
  ///
  /// Returns `Err` if the session is already closed/broken, or the message cannot be
  /// written to stdin.
  ///
  /// # Panics
  ///
  /// Panics if an internal mutex (`broken` or `stdin`) is poisoned (a prior thread
  /// panicked while holding the lock).
  #[ inline ]
  pub fn stream_input( &self, content : &str ) -> Result< () >
  {
    if let Some( reason ) = self.broken.lock().unwrap().clone()
    {
      return Err( Error::msg( format!( "control session broken: {reason}" ) ) );
    }

    let message = serde_json::json!(
    {
      "type" : "user",
      "message" : { "role" : "user", "content" : content },
      "parent_tool_use_id" : serde_json::Value::Null,
    } );
    let mut line = serde_json::to_string( &message )
      .map_err( | e | Error::msg( format!( "failed to serialize user message: {e}" ) ) )?;
    line.push( '\n' );

    let mut guard = self.stdin.lock().unwrap();
    let stdin = guard.as_mut()
      .ok_or_else( || Error::msg( "control session already closed" ) )?;
    stdin.write_all( line.as_bytes() )
      .map_err( | e | Error::msg( format!( "failed to write user message to stdin: {e}" ) ) )?;
    stdin.flush()
      .map_err( | e | Error::msg( format!( "failed to flush user message to stdin: {e}" ) ) )?;
    Ok( () )
  }

  /// `stopTask(taskId)` — stop a named background task. Wire subtype `stop_task`, field
  /// `task_id`.
  ///
  /// # Errors
  ///
  /// Returns `Err` if the session is broken, the request cannot be written to stdin, or
  /// the subprocess does not respond before the timeout elapses.
  #[ inline ]
  pub fn stop_task( &self, task_id : &str ) -> Result< () >
  {
    let params = serde_json::json!( { "task_id" : task_id } );
    self.send_request( "stop_task", &params )?;
    Ok( () )
  }

  /// `backgroundTasks(toolUseId?)` — background the current (or a named) foreground task.
  /// Wire subtype `background_tasks`, optional field `tool_use_id`.
  ///
  /// **Confirmed empirically (task 415 Phase 0 + Phase 2 probing):** the real wire response
  /// on success is an empty object `{}`, in both an idle session and one with a genuine
  /// in-flight foreground tool call — there is no observable true/false signal on the wire.
  /// Any successful ack is therefore treated as `true`, matching the SDK's own documented
  /// `Promise<boolean>` contract; a literal wire boolean (should a future CLI version ever
  /// send one) is still honored directly rather than assumed away.
  ///
  /// # Errors
  ///
  /// Returns `Err` if the session is broken, the request cannot be written to stdin, or the
  /// subprocess does not respond before the timeout elapses.
  #[ inline ]
  pub fn background_tasks( &self, tool_use_id : Option< &str > ) -> Result< bool >
  {
    let params = match tool_use_id
    {
      Some( id ) => serde_json::json!( { "tool_use_id" : id } ),
      None => serde_json::json!( {} ),
    };
    let value = self.send_request( "background_tasks", &params )?;
    Ok( value.as_bool().unwrap_or( true ) )
  }

  // ==========================================================================
  // Teardown
  // ==========================================================================

  /// Close the session: close stdin (signaling EOF), wait briefly for a clean exit, then
  /// force-kill if needed, and join the background reader thread. Idempotent — safe to call
  /// more than once, and safe to skip (the `Drop` impl calls this as a fallback).
  ///
  /// # Errors
  ///
  /// Never returns `Err` in practice (subprocess wait/kill failures are swallowed as
  /// best-effort teardown) — `Result` is kept for forward compatibility and API symmetry
  /// with the other control methods.
  ///
  /// # Panics
  ///
  /// Panics if the internal stdin mutex is poisoned (a prior thread panicked while
  /// holding the lock).
  #[ inline ]
  pub fn close( &mut self ) -> Result< () >
  {
    if self.closed.swap( true, Ordering::SeqCst )
    {
      return Ok( () );
    }

    // Dropping stdin closes the pipe, signaling EOF to the subprocess.
    drop( self.stdin.lock().unwrap().take() );

    if matches!( self.child.try_wait(), Ok( None ) | Err( _ ) )
    {
      std::thread::sleep( Duration::from_millis( 200 ) );
      if matches!( self.child.try_wait(), Ok( None ) | Err( _ ) )
      {
        let _ = self.child.kill();
      }
    }
    let _ = self.child.wait();

    if let Some( handle ) = self.reader.take()
    {
      let _ = handle.join();
    }

    Ok( () )
  }
}

impl Drop for ControlSession
{
  #[ inline ]
  fn drop( &mut self )
  {
    let _ = self.close();
  }
}

/// Background stdout-demultiplexing loop: reads NDJSON lines, routes `control_response`
/// lines to their waiter by `request_id`, refreshes the `initialize`-shaped cache on a
/// successful `initialize` response, and forwards every other line to `message_tx` unparsed.
fn read_loop(
  stdout : std::process::ChildStdout,
  pending : &Arc< Mutex< HashMap< String, mpsc::Sender< WireOutcome > > > >,
  cache : &Arc< Mutex< Option< serde_json::Value > > >,
  message_tx : &mpsc::Sender< serde_json::Value >,
)
{
  let reader = std::io::BufReader::new( stdout );
  for line in reader.lines()
  {
    let Ok( line ) = line else { break };
    if line.trim().is_empty()
    {
      continue;
    }
    let Ok( value ) = serde_json::from_str::< serde_json::Value >( &line ) else { continue };

    if value.get( "type" ).and_then( serde_json::Value::as_str ) != Some( "control_response" )
    {
      let _ = message_tx.send( value );
      continue;
    }

    let Some( response ) = value.get( "response" ) else { continue };
    let Some( request_id ) = response.get( "request_id" ).and_then( serde_json::Value::as_str ) else { continue };

    let outcome = match response.get( "subtype" ).and_then( serde_json::Value::as_str )
    {
      Some( "success" ) =>
      {
        let payload = response.get( "response" ).cloned().unwrap_or( serde_json::Value::Null );
        if is_initialize_shaped( &payload )
        {
          *cache.lock().unwrap() = Some( payload.clone() );
        }
        WireOutcome::Success( payload )
      }
      Some( "error" ) =>
      {
        let message = response.get( "error" ).and_then( serde_json::Value::as_str )
          .unwrap_or( "unknown control error" ).to_string();
        WireOutcome::Error( message )
      }
      _ => WireOutcome::Error( format!( "unrecognized control_response subtype in: {response}" ) ),
    };

    if let Some( sender ) = pending.lock().unwrap().remove( request_id )
    {
      let _ = sender.send( outcome );
    }
  }
}

/// Whether a successful control-response payload has the `initialize` response's shape
/// (used to identify which successful responses should refresh the cached-accessor cache).
fn is_initialize_shaped( value : &serde_json::Value ) -> bool
{
  value.get( "commands" ).is_some() && value.get( "models" ).is_some() && value.get( "account" ).is_some()
}
