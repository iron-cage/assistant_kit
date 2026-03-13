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
