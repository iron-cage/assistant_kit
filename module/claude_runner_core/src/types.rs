//! Type definitions for Claude Code configuration
//!
//! Provides type-safe enums for configuration options that map to Claude Code
//! environment variables.

/// Tool approval behavior mode
///
/// Controls how Claude Code handles tool execution requests.
///
/// # Environment Variable
///
/// Maps to `CLAUDE_CODE_ACTION_MODE` environment variable.
///
/// # Examples
///
/// ```
/// use claude_runner_core::ActionMode;
///
/// let mode = ActionMode::Ask;  // Default: prompt user for each tool
/// let mode = ActionMode::Allow;  // Auto-approve all tools (use with caution)
/// let mode = ActionMode::Deny;  // Reject all tool executions
/// ```
#[derive( Debug, Clone, Copy, PartialEq, Eq )]
pub enum ActionMode {
  /// Prompt user before each tool execution (default, safest)
  Ask,
  /// Automatically approve all tool executions (requires explicit opt-in)
  Allow,
  /// Deny all tool execution requests
  Deny,
}

impl ActionMode {
  /// Convert to environment variable string value
  ///
  /// # Examples
  ///
  /// ```
  /// use claude_runner_core::ActionMode;
  ///
  /// assert_eq!( ActionMode::Ask.as_str(), "ask" );
  /// assert_eq!( ActionMode::Allow.as_str(), "allow" );
  /// assert_eq!( ActionMode::Deny.as_str(), "deny" );
  /// ```
  #[inline]
  #[must_use]
  pub fn as_str( self ) -> &'static str {
    match self {
      Self::Ask => "ask",
      Self::Allow => "allow",
      Self::Deny => "deny",
    }
  }
}

impl Default for ActionMode {
  #[inline]
  fn default() -> Self {
    // Fix(issue-action-mode-default): Default is Ask for security
    // Root cause: Allow would auto-approve all tools without user consent
    // Pitfall: Never default to Allow - requires explicit opt-in
    Self::Ask
  }
}

/// Logging verbosity level
///
/// Controls the verbosity of Claude Code logging output.
///
/// # Environment Variable
///
/// Maps to `CLAUDE_CODE_LOG_LEVEL` environment variable.
///
/// # Examples
///
/// ```
/// use claude_runner_core::LogLevel;
///
/// let level = LogLevel::Info;   // Default: standard information
/// let level = LogLevel::Debug;  // Verbose debugging output
/// let level = LogLevel::Error;  // Only errors
/// ```
#[derive( Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord )]
pub enum LogLevel {
  /// Only critical errors
  Error,
  /// Warnings and errors
  Warn,
  /// Standard information (default)
  Info,
  /// Detailed debugging information
  Debug,
  /// All possible logging output
  Trace,
}

impl LogLevel {
  /// Convert to environment variable string value
  ///
  /// # Examples
  ///
  /// ```
  /// use claude_runner_core::LogLevel;
  ///
  /// assert_eq!( LogLevel::Error.as_str(), "error" );
  /// assert_eq!( LogLevel::Warn.as_str(), "warn" );
  /// assert_eq!( LogLevel::Info.as_str(), "info" );
  /// assert_eq!( LogLevel::Debug.as_str(), "debug" );
  /// assert_eq!( LogLevel::Trace.as_str(), "trace" );
  /// ```
  #[inline]
  #[must_use]
  pub fn as_str( self ) -> &'static str {
    match self {
      Self::Error => "error",
      Self::Warn => "warn",
      Self::Info => "info",
      Self::Debug => "debug",
      Self::Trace => "trace",
    }
  }
}

impl Default for LogLevel {
  #[inline]
  fn default() -> Self {
    Self::Info
  }
}

/// Output format for Claude Code execution
///
/// Controls the `--output-format` CLI flag.
///
/// # Critical
///
/// `StreamJson` serializes to `"stream-json"` with a **hyphen**, not an underscore.
///
/// # Examples
///
/// ```
/// use claude_runner_core::OutputFormat;
///
/// assert_eq!( OutputFormat::Text.as_str(), "text" );
/// assert_eq!( OutputFormat::Json.as_str(), "json" );
/// assert_eq!( OutputFormat::StreamJson.as_str(), "stream-json" );  // hyphen!
/// ```
#[derive( Debug, Clone, Copy, PartialEq, Eq )]
pub enum OutputFormat
{
  /// Plain text output (default)
  Text,
  /// JSON object output
  Json,
  /// Streaming JSON output (newline-delimited JSON events)
  StreamJson,
}

impl OutputFormat
{
  /// Convert to CLI string value
  ///
  /// # Examples
  ///
  /// ```
  /// use claude_runner_core::OutputFormat;
  ///
  /// // CRITICAL: StreamJson uses a hyphen, not underscore
  /// assert_eq!( OutputFormat::StreamJson.as_str(), "stream-json" );
  /// ```
  #[inline]
  #[must_use]
  pub fn as_str( self ) -> &'static str
  {
    match self
    {
      // Fix(issue-output-format-stream-json-hyphen): StreamJson maps to "stream-json" with a hyphen
      // Root cause: Claude CLI uses "stream-json" (hyphen) not "stream_json" (underscore)
      // Pitfall: Do not use underscore — "stream_json" is not a valid claude CLI value
      Self::Text => "text",
      Self::Json => "json",
      Self::StreamJson => "stream-json",
    }
  }
}

impl Default for OutputFormat
{
  #[inline]
  fn default() -> Self { Self::Text }
}

/// Input format for Claude Code stdin
///
/// Controls the `--input-format` CLI flag.
///
/// # Critical
///
/// `StreamJson` serializes to `"stream-json"` with a **hyphen**, not an underscore.
///
/// # Examples
///
/// ```
/// use claude_runner_core::InputFormat;
///
/// assert_eq!( InputFormat::Text.as_str(), "text" );
/// assert_eq!( InputFormat::StreamJson.as_str(), "stream-json" );  // hyphen!
/// ```
#[derive( Debug, Clone, Copy, PartialEq, Eq )]
pub enum InputFormat
{
  /// Plain text input (default)
  Text,
  /// Streaming JSON input (newline-delimited JSON events)
  StreamJson,
}

impl InputFormat
{
  /// Convert to CLI string value
  ///
  /// # Examples
  ///
  /// ```
  /// use claude_runner_core::InputFormat;
  ///
  /// assert_eq!( InputFormat::StreamJson.as_str(), "stream-json" );
  /// ```
  #[inline]
  #[must_use]
  pub fn as_str( self ) -> &'static str
  {
    match self
    {
      Self::Text => "text",
      Self::StreamJson => "stream-json",
    }
  }
}

impl Default for InputFormat
{
  #[inline]
  fn default() -> Self { Self::Text }
}

/// Permission mode for tool execution
///
/// Controls the `--permission-mode` CLI flag.
///
/// # Critical: camelCase strings
///
/// `AcceptEdits` and `BypassPermissions` serialize to camelCase strings — NOT lowercase.
/// These exact strings are required by the Claude CLI.
///
/// # Examples
///
/// ```
/// use claude_runner_core::PermissionMode;
///
/// assert_eq!( PermissionMode::Default.as_str(), "default" );
/// assert_eq!( PermissionMode::AcceptEdits.as_str(), "acceptEdits" );
/// assert_eq!( PermissionMode::BypassPermissions.as_str(), "bypassPermissions" );
/// ```
#[derive( Debug, Clone, Copy, PartialEq, Eq )]
pub enum PermissionMode
{
  /// Default permission prompting behavior
  Default,
  /// Auto-accept edit operations
  AcceptEdits,
  /// Bypass all permission checks (use with caution)
  BypassPermissions,
}

impl PermissionMode
{
  /// Convert to CLI string value
  ///
  /// # Examples
  ///
  /// ```
  /// use claude_runner_core::PermissionMode;
  ///
  /// // WARNING: AcceptEdits and BypassPermissions use camelCase — not lowercase
  /// assert_eq!( PermissionMode::AcceptEdits.as_str(), "acceptEdits" );
  /// assert_eq!( PermissionMode::BypassPermissions.as_str(), "bypassPermissions" );
  /// ```
  #[inline]
  #[must_use]
  pub fn as_str( self ) -> &'static str
  {
    match self
    {
      // Fix(issue-permission-mode-camelcase): AcceptEdits/BypassPermissions use camelCase
      // Root cause: Claude CLI uses camelCase for multi-word permission mode strings
      // Pitfall: Do not use lowercase — "acceptedits" and "bypasspermissions" are not valid
      Self::Default => "default",
      Self::AcceptEdits => "acceptEdits",
      Self::BypassPermissions => "bypassPermissions",
    }
  }
}

impl Default for PermissionMode
{
  #[inline]
  fn default() -> Self { Self::Default }
}

/// Effort level for model reasoning
///
/// Controls the `--effort` CLI flag.
///
/// # Examples
///
/// ```
/// use claude_runner_core::EffortLevel;
///
/// assert_eq!( EffortLevel::Low.as_str(), "low" );
/// assert_eq!( EffortLevel::Medium.as_str(), "medium" );
/// assert_eq!( EffortLevel::High.as_str(), "high" );
/// assert_eq!( EffortLevel::Max.as_str(), "max" );  // NOT "maximum"
/// ```
#[derive( Debug, Clone, Copy, PartialEq, Eq )]
pub enum EffortLevel
{
  /// Minimal reasoning effort
  Low,
  /// Standard effort (default)
  Medium,
  /// High reasoning effort
  High,
  /// Maximum reasoning effort
  Max,
}

impl EffortLevel
{
  /// Convert to CLI string value
  ///
  /// # Examples
  ///
  /// ```
  /// use claude_runner_core::EffortLevel;
  ///
  /// // NOTE: Max maps to "max", not "maximum"
  /// assert_eq!( EffortLevel::Max.as_str(), "max" );
  /// ```
  #[inline]
  #[must_use]
  pub fn as_str( self ) -> &'static str
  {
    match self
    {
      Self::Low => "low",
      Self::Medium => "medium",
      Self::High => "high",
      Self::Max => "max",
    }
  }
}

impl Default for EffortLevel
{
  #[inline]
  fn default() -> Self { Self::Medium }
}

impl core::str::FromStr for EffortLevel
{
  type Err = String;

  /// Parse a CLI string into an `EffortLevel`.
  ///
  /// Accepts exactly the strings returned by `as_str()`:
  /// `"low"`, `"medium"`, `"high"`, `"max"`.
  ///
  /// # Examples
  ///
  /// ```
  /// use claude_runner_core::EffortLevel;
  ///
  /// assert_eq!( "max".parse::< EffortLevel >().unwrap(), EffortLevel::Max );
  /// assert_eq!( "low".parse::< EffortLevel >().unwrap(), EffortLevel::Low );
  /// assert!( "invalid".parse::< EffortLevel >().is_err() );
  /// let err = "bad".parse::< EffortLevel >().unwrap_err();
  /// assert!( err.contains( "valid values" ), "error must list valid values: {err}" );
  /// ```
  #[inline]
  fn from_str( s : &str ) -> Result< Self, Self::Err >
  {
    match s
    {
      "low"    => Ok( Self::Low ),
      "medium" => Ok( Self::Medium ),
      "high"   => Ok( Self::High ),
      "max"    => Ok( Self::Max ),
      _ => Err( format!(
        "unknown effort level: '{s}' — valid values: low, medium, high, max"
      ) ),
    }
  }
}

/// Output from a non-interactive Claude Code execution
///
/// Contains captured stdout, stderr, and exit code from the process.
///
/// # Examples
///
/// ```no_run
/// use claude_runner_core::ClaudeCommand;
///
/// let output = ClaudeCommand::new()
///   .with_message( "hello" )
///   .execute()?;
///
/// println!( "stdout: {}", output.stdout );
/// if !output.stderr.is_empty()
/// {
///   eprintln!( "stderr: {}", output.stderr );
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive( Debug, Clone, PartialEq, Eq )]
pub struct ExecutionOutput
{
  /// Captured stdout from Claude Code process.
  pub stdout : String,
  /// Captured stderr from Claude Code process.
  pub stderr : String,
  /// Process exit code (0 = success).
  pub exit_code : i32,
}

impl core::fmt::Display for ExecutionOutput
{
  #[inline]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
    write!( f, "{}", self.stdout )
  }
}

/// Classification of CLR subprocess failure modes.
///
/// Returned by [`ExecutionOutput::classify_error`] when stderr/stdout content
/// or exit code indicates a specific failure category.
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub enum ErrorKind
{
  /// Transient rate limit reached (exit code 2 — HTTP 429, retry in seconds).
  RateLimit,
  /// Period quota exhausted ("You've hit your limit" pattern — wait for reset or switch account).
  QuotaExhausted,
  /// API-level error (HTTP 5xx/4xx — "API Error: " pattern).
  ApiError,
  /// Authentication/authorization failure ("Your organization does not have access" pattern).
  AuthError,
  /// Process killed by signal (exit code > 128).
  Signal,
  /// Non-zero exit with no recognized pattern and no signal code.
  Unknown,
}

// Priority-ordered stderr/stdout patterns → ErrorKind.
// Both AuthError patterns appear before ApiError so all 401 responses hit AuthError:
//   "Your organization..." — org-level access denial
//   "authentication_error" — Claude CLI 401 form (Fix BUG-314: contains "API Error: "
//     as a substring; without this pattern the catch-all fired first → misclassified
//     as ApiError → ErrorClass::Service instead of ErrorClass::Auth)
//
// NOTE: E4 (Request Timed Out) retry progress uses "API Error (Request timed out.)"
// which does NOT match "API Error: " (parenthesis, not colon-space). However, E4 hangs
// after 10 retries without exiting — classify_error() is never called in that scenario.
// When --timeout kills the subprocess it gets SIGTERM → Signal, not Unknown.
//
// NOTE: E3 (Context Limit) API-overflow form begins with "API Error: 400 ..." and
// matches ApiError. The in-session UI text ("Context limit reached") is interactive-only
// and never appears in print-mode captured stdout/stderr.
const ERROR_PATTERNS : &[ ( &str, ErrorKind ) ] =
&[
  ( "You've hit your limit",                            ErrorKind::QuotaExhausted ),
  ( "Your organization does not have access to Claude", ErrorKind::AuthError ),
  // Fix(BUG-314): "authentication_error" precedes the "API Error: " catch-all.
  // Root cause: the Claude CLI 401 form "Failed to authenticate. API Error: 401
  //   {\"type\":\"authentication_error\",...}" contains "API Error: " as a substring;
  //   without this entry the catch-all fired first → ApiError → ErrorClass::Service.
  // Pitfall: priority-ordered pattern lists silently misclassify errors that also contain
  //   a catch-all substring — every non-catch-all class needs a pattern placed before it.
  ( "authentication_error",                             ErrorKind::AuthError ),
  ( "API Error: ",                                      ErrorKind::ApiError ),
];

impl ExecutionOutput
{
  /// Classify the subprocess failure type from stderr/stdout patterns and exit code.
  ///
  /// Returns `None` when `exit_code == 0` (success). For non-zero exits:
  /// 1. Scans both stderr and stdout for known patterns (priority order,
  ///    including `QuotaExhausted` for period limits).
  /// 2. Falls back to exit code 2 → `RateLimit` (transient rate-limit sentinel).
  /// 3. Falls back to exit code > 128 → `Signal`.
  /// 4. Returns `Unknown` for all other non-zero exits.
  #[ inline ]
  #[ must_use ]
  pub fn classify_error( &self ) -> Option< ErrorKind >
  {
    if self.exit_code == 0
    {
      return None;
    }
    for ( pattern, kind ) in ERROR_PATTERNS
    {
      if self.stderr.contains( pattern ) || self.stdout.contains( pattern )
      {
        return Some( kind.clone() );
      }
    }
    if self.exit_code == 2
    {
      return Some( ErrorKind::RateLimit );
    }
    if self.exit_code > 128
    {
      return Some( ErrorKind::Signal );
    }
    Some( ErrorKind::Unknown )
  }
}

// ============================================================================
// Control Protocol Types (Agent SDK `Query` interface parity, task 415)
// ============================================================================
//
// Request-parameter and response-payload types for the 25 `Query` control
// methods `crate::control::ControlSession` implements. Field names, casing,
// and shapes are evidenced directly against a captured real-SDK wire trace —
// see `tests/fixtures/sdk_control_capture/` (task 415 Phase 0) — not inferred
// solely from the SDK's TypeScript type definitions. Genuinely large/dynamic
// payloads (slash-command lists, model lists, context-usage breakdowns) are
// left as `serde_json::Value` rather than fully modeled: this task's scope is
// the 25-method control surface, not a full mirror of the SDK's own internal
// type system.

/// Per-MCP-server permission override, for [`crate::control::ControlSession::set_mcp_permission_mode_override`].
///
/// Wire values confirmed against `tests/fixtures/sdk_control_capture/wire_stdin.ndjson`
/// (`set_mcp_permission_mode_override` request, `mode` field).
#[derive( Debug, Clone, Copy, PartialEq, Eq, serde::Serialize )]
#[ serde( rename_all = "lowercase" ) ]
pub enum McpPermissionOverrideMode
{
  /// Restore default per-tool permission prompting for this MCP server.
  Default,
  /// Auto-approve all tool calls from this MCP server.
  Auto,
}

impl McpPermissionOverrideMode
{
  /// Convert to wire string value.
  #[ inline ]
  #[ must_use ]
  pub fn as_str( self ) -> &'static str
  {
    match self
    {
      Self::Default => "default",
      Self::Auto => "auto",
    }
  }
}

/// File-read encoding, for [`crate::control::ControlSession::read_file`].
///
/// Wire value confirmed against `sdk.d.ts`'s `readFile(path, options?: { encoding?: 'utf-8' | 'base64' })`.
#[derive( Debug, Clone, Copy, PartialEq, Eq, serde::Serialize )]
pub enum ReadFileEncoding
{
  /// UTF-8 text encoding (default when omitted).
  #[ serde( rename = "utf-8" ) ]
  Utf8,
  /// Base64-encoded binary content.
  #[ serde( rename = "base64" ) ]
  Base64,
}

impl ReadFileEncoding
{
  /// Convert to wire string value.
  #[ inline ]
  #[ must_use ]
  pub fn as_str( self ) -> &'static str
  {
    match self
    {
      Self::Utf8 => "utf-8",
      Self::Base64 => "base64",
    }
  }
}

/// Thinking-token display mode, for [`crate::control::ControlSession::set_max_thinking_tokens`].
///
/// Wire value confirmed against `sdk.d.ts`'s deprecated `setMaxThinkingTokens(n, thinkingDisplay?)`.
#[derive( Debug, Clone, Copy, PartialEq, Eq, serde::Serialize )]
#[ serde( rename_all = "lowercase" ) ]
pub enum ThinkingDisplay
{
  /// Show a summarized form of the model's thinking.
  Summarized,
  /// Omit thinking display entirely.
  Omitted,
}

impl ThinkingDisplay
{
  /// Convert to wire string value.
  #[ inline ]
  #[ must_use ]
  pub fn as_str( self ) -> &'static str
  {
    match self
    {
      Self::Summarized => "summarized",
      Self::Omitted => "omitted",
    }
  }
}

/// Result of [`crate::control::ControlSession::rewind_files`].
///
/// Shape confirmed against a captured `rewind_files` response:
/// `{"canRewind":false,"error":"No file checkpoint found for this message."}`
/// (`tests/fixtures/sdk_control_capture/wire_stdout.ndjson`).
#[derive( Debug, Clone, PartialEq, Eq, serde::Deserialize ) ]
pub struct RewindFilesResult
{
  /// Whether the rewind can be (or was, for a non-dry-run) applied.
  #[ serde( rename = "canRewind" ) ]
  pub can_rewind : bool,
  /// Present when `can_rewind` is `false`, explaining why.
  pub error : Option< String >,
}

/// Result of [`crate::control::ControlSession::set_mcp_permission_mode_override`].
///
/// Wire response omits the `response` object entirely on a clean override (no `warning`
/// key at all) — confirmed against the captured trace's `set_mcp_permission_mode_override`
/// response. `#[serde(default)]` treats a missing `response` object as `warning: None`.
#[derive( Debug, Clone, Default, PartialEq, Eq, serde::Deserialize ) ]
pub struct SetMcpPermissionModeOverrideResult
{
  /// Set when `serverName` matched no currently-known MCP server.
  #[ serde( default ) ]
  pub warning : Option< String >,
}

/// Result of [`crate::control::ControlSession::initialization_result`] and
/// [`crate::control::ControlSession::reinitialize`].
///
/// Top-level keys confirmed against the captured `initialize` `control_response`; nested
/// shapes (slash commands, model list, account info, etc.) are intentionally left as
/// `serde_json::Value` — modeling the SDK's full internal type system is out of this
/// task's scope (see `types.rs` § Control Protocol Types module doc above).
#[derive( Debug, Clone, PartialEq, serde::Deserialize ) ]
pub struct InitializeResult
{
  /// Available slash commands.
  pub commands : serde_json::Value,
  /// Available subagents.
  pub agents : serde_json::Value,
  /// Currently active output style.
  pub output_style : serde_json::Value,
  /// All output styles the session supports.
  pub available_output_styles : serde_json::Value,
  /// Available models.
  pub models : serde_json::Value,
  /// Account information (see also [`AccountInfo`], the typed accessor's own view of this field).
  pub account : serde_json::Value,
  /// Claude Code subprocess PID.
  pub pid : serde_json::Value,
  /// Feedback survey configuration.
  pub feedback_survey_config : serde_json::Value,
}

/// One entry in [`crate::control::ControlSession::mcp_server_status`]'s returned list.
///
/// Shape confirmed against the captured `mcp_status` response:
/// `{"mcpServers":[{"name":"phase0probe","status":"connected","scope":"dynamic","tools":[]}]}`.
#[derive( Debug, Clone, PartialEq, serde::Deserialize ) ]
pub struct McpServerStatusEntry
{
  /// Server name as registered with the session.
  pub name : String,
  /// Connection status (e.g. `"connected"`).
  pub status : String,
  /// Registration scope (e.g. `"dynamic"`).
  pub scope : String,
  /// Tools this server currently exposes.
  pub tools : Vec< serde_json::Value >,
}

/// Result of [`crate::control::ControlSession::read_file`].
///
/// Shape confirmed against the captured `read_file` response:
/// `{"contents":"line one\nline two\n","absPath":"/tmp/-phase0-scratch/sample.txt"}`.
#[derive( Debug, Clone, PartialEq, Eq, serde::Deserialize ) ]
pub struct ReadFileResult
{
  /// File contents (UTF-8 text, or base64 when `ReadFileEncoding::Base64` was requested).
  pub contents : String,
  /// Absolute resolved path of the file that was read.
  #[ serde( rename = "absPath" ) ]
  pub abs_path : String,
}

/// Result of [`crate::control::ControlSession::reload_plugins`].
///
/// Shape confirmed against the captured `reload_plugins` response (`commands`, `agents`,
/// `plugins`, `mcpServers`, `error_count` keys); nested contents left as `serde_json::Value`.
#[derive( Debug, Clone, PartialEq, serde::Deserialize ) ]
pub struct ReloadPluginsResult
{
  /// Refreshed slash-command list.
  pub commands : serde_json::Value,
  /// Refreshed subagent list.
  pub agents : serde_json::Value,
  /// Refreshed plugin list.
  pub plugins : serde_json::Value,
  /// Refreshed MCP server list.
  #[ serde( rename = "mcpServers" ) ]
  pub mcp_servers : serde_json::Value,
  /// Number of errors encountered while reloading.
  pub error_count : u32,
}

/// Result of [`crate::control::ControlSession::reload_skills`].
///
/// Shape confirmed against the captured `reload_skills` response (`skills` key only).
#[derive( Debug, Clone, PartialEq, serde::Deserialize ) ]
pub struct ReloadSkillsResult
{
  /// Refreshed skill list.
  pub skills : serde_json::Value,
}

/// Result of [`crate::control::ControlSession::account_info`].
///
/// Shape confirmed against the captured (PII-redacted) `initialize` response's `account`
/// field: `{"email":"...","organization":"...","subscriptionType":"...","apiProvider":"..."}`.
#[derive( Debug, Clone, PartialEq, Eq, serde::Deserialize ) ]
pub struct AccountInfo
{
  /// Account email address.
  pub email : String,
  /// Organization display name.
  pub organization : String,
  /// Subscription tier (e.g. `"Claude Max"`).
  #[ serde( rename = "subscriptionType" ) ]
  pub subscription_type : String,
  /// API provider (e.g. `"firstParty"`).
  #[ serde( rename = "apiProvider" ) ]
  pub api_provider : String,
}

/// Result of [`crate::control::ControlSession::set_mcp_servers`].
///
/// Shape confirmed against the captured `mcp_set_servers` response:
/// `{"added":[],"removed":[],"errors":{}}`.
#[derive( Debug, Clone, Default, PartialEq, Eq, serde::Deserialize ) ]
pub struct SetMcpServersResult
{
  /// Server names successfully added.
  pub added : Vec< String >,
  /// Server names successfully removed.
  pub removed : Vec< String >,
  /// Server name → error message, for servers that failed to apply.
  pub errors : std::collections::HashMap< String, String >,
}

/// Result of [`crate::control::ControlSession::get_context_usage`].
///
/// Left as `serde_json::Value`: the captured `get_context_usage` response has 16 top-level
/// keys (`categories`, `totalTokens`, `maxTokens`, `rawMaxTokens`, `autocompactSource`,
/// `percentage`, `gridRows`, `model`, `memoryFiles`, `mcpTools`, `agents`, `slashCommands`,
/// `skills`, `autoCompactThreshold`, `isAutoCompactEnabled`, `messageBreakdown`, `apiUsage`),
/// several deeply nested — fully modeling this is out of this task's scope.
pub type ContextUsageResult = serde_json::Value;
