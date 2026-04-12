//! Claude Code Command Builder
//!
//! Provides fluent API for constructing and executing Claude Code CLI commands.
//!
//! ## Execution Modes
//!
//! This module supports two execution modes:
//!
//! - **Non-interactive mode** ([`execute`](ClaudeCommand::execute)): Captures stdout/stderr, suitable for programmatic usage
//! - **Interactive mode** ([`execute_interactive`](ClaudeCommand::execute_interactive)): Allows Claude Code to take over terminal (TTY attached)
//!
//! The distinction is critical: `.output()` captures process output which prevents Claude Code from
//! accessing the terminal for interactive sessions. Interactive mode uses `.status()` to preserve TTY access.

use std::path::PathBuf;
use error_tools::{ Result, Error };

mod params_core;
mod params_security;
mod params_extended;

/// Builder for Claude Code CLI commands
///
/// # Example
///
/// ```no_run
/// use claude_runner_core::ClaudeCommand;
///
/// let result = ClaudeCommand::new()
///   .with_working_directory( "/home/user/project" )
///   .with_max_output_tokens( 200_000 )
///   .execute()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive( Debug )]
pub struct ClaudeCommand {
  pub(super) working_directory: Option<PathBuf>,
  pub(super) max_output_tokens: Option<u32>,
  pub(super) continue_conversation: bool,
  pub(super) message: Option<String>,
  pub(super) args: Vec<String>,

  // Tier 1: Critical parameters with different defaults (fix automation blockers)
  pub(super) bash_default_timeout_ms: Option<u32>,
  pub(super) bash_max_timeout_ms: Option<u32>,
  pub(super) auto_continue: Option<bool>,
  pub(super) telemetry: Option<bool>,

  // Tier 2: Essential parameters with standard defaults (security-sensitive)
  pub(super) auto_approve_tools: Option<bool>,
  pub(super) action_mode: Option<crate::types::ActionMode>,
  pub(super) log_level: Option<crate::types::LogLevel>,
  pub(super) temperature: Option<f64>,

  // Safety override
  pub(super) skip_permissions: bool,

  // Terminal & IDE flags with non-standard builder defaults
  pub(super) chrome: Option<bool>,

  // Tier 3: Optional parameters with standard defaults
  pub(super) sandbox_mode: Option<bool>,
  pub(super) session_dir: Option<PathBuf>,
  pub(super) top_p: Option<f64>,
  pub(super) top_k: Option<u32>,

  // Execution control
  pub(super) dry_run: bool,
}

impl ClaudeCommand {
  /// Create a new Claude Code command builder
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new();
  /// ```
  #[inline]
  #[must_use]
  pub fn new() -> Self {
    // Fix(issue-token-limit-default): Default token limit changed from 32K to 200K
    // Root cause: Migration from factory pattern didnt preserve correct default value
    // Pitfall: Always verify defaults match specification when refactoring APIs

    // Fix(issue-bash-timeout-default): Bash timeouts increased from 2min/10min to 1hr/2hr
    // Root cause: Standard 2min default causes premature timeout in real automation workflows
    // Pitfall: Always set explicit timeouts matching actual operation duration needs

    // Fix(issue-auto-continue-default): Auto-continue enabled by default (true vs false)
    // Root cause: Standard false blocks all automation with manual prompts
    // Pitfall: Programmatic usage requires automation-friendly defaults

    // Fix(issue-telemetry-default): Telemetry disabled by default (false vs true)
    // Root cause: Automation contexts shouldnt send usage data without explicit consent
    // Pitfall: Respect user privacy in programmatic execution

    // Fix(issue-chrome-default): Chrome enabled by default (Some(true) vs None/omit)
    // Root cause: Browser context is essential for web-aware automation; omitting --chrome
    //             relies on the user's global claude config being set to on, which is not guaranteed
    // Pitfall: Store as field, not push to args in new() — must remain overridable by with_chrome()

    Self {
      working_directory: None,
      max_output_tokens: Some( 200_000 ),
      continue_conversation: false,
      message: None,
      args: Vec::new(),

      // Tier 1: Different defaults (fix automation blockers)
      bash_default_timeout_ms: Some( 3_600_000 ),  // 1 hour (vs 2 min standard)
      bash_max_timeout_ms: Some( 7_200_000 ),      // 2 hours (vs 10 min standard)
      auto_continue: Some( true ),                 // Enable automation (vs false standard)
      telemetry: Some( false ),                    // Disable telemetry (vs true standard)

      skip_permissions: false,
      chrome: Some( true ),  // Enable browser context by default (vs off in raw claude binary)

      // Tier 2 & 3: Standard defaults (security-sensitive, opt-in only)
      auto_approve_tools: None,  // Inherits standard: false
      action_mode: None,         // Inherits standard: Ask
      log_level: None,           // Inherits standard: Info
      temperature: None,         // Inherits standard: 1.0
      sandbox_mode: None,        // Inherits standard: true
      session_dir: None,         // Inherits standard: auto-detect
      top_p: None,               // Inherits standard: None
      top_k: None,               // Inherits standard: None

      dry_run: false,
    }
  }

  /// Describe only the `claude ...` invocation line (no `cd` prefix)
  ///
  /// Unlike `describe()`, this always returns a single line containing only
  /// the `claude` invocation. When a working directory is set, `describe()`
  /// returns two lines (`cd /dir\nclaude ...`); this method returns only the last.
  ///
  /// # Critical: Implementation Must Use `describe().lines().last()`
  ///
  /// Do NOT reconstruct the command from parts — that would diverge from the
  /// actual command built by `build_command()`. The only correct implementation
  /// is to delegate to `describe()` and extract the last line.
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let compact = ClaudeCommand::new()
  ///   .with_working_directory( "/tmp" )
  ///   .with_skip_permissions( true )
  ///   .describe_compact();
  ///
  /// assert!( compact.starts_with( "claude" ) );
  /// assert!( !compact.contains( "cd " ) );
  /// ```
  #[inline]
  #[must_use]
  pub fn describe_compact( &self ) -> String {
    // Fix(issue-describe-compact-double-cd): Always extract last line from describe()
    // Root cause: describe() emits "cd /dir\nclaude ..." when working_directory is set
    // Pitfall: Rebuilding from parts diverges from build_command(); always delegate to describe()
    self.describe()
      .lines()
      .last()
      .unwrap_or( "claude" )
      .to_string()
  }

  /// Describe the command line that would be executed
  ///
  /// Returns a human-readable representation of the command. If a working
  /// directory is set, the first line is `cd /path/to/dir`. The last line
  /// is the `claude` invocation with all flags and arguments.
  ///
  /// # Output Flag Order
  ///
  /// The command-line flag order in the output is fixed by the implementation,
  /// **not** by the order in which `with_*` builder methods are called. The order is:
  ///
  /// 1. `--dangerously-skip-permissions` (if `skip_permissions` is true)
  /// 2. `--chrome` or `--no-chrome` (from `chrome` field; default `Some(true)` = `--chrome`)
  /// 3. custom args (in insertion order via `with_arg`)
  /// 4. `-c` (if `continue_conversation` is true)
  /// 5. `"<message>"` (if message is set)
  ///
  /// This matters when writing tests that assert the exact output string (e.g. `assert_eq!`).
  /// Use `contains` assertions for individual flags when order is not the subject of the test.
  ///
  /// # Critical: Must Mirror `build_command()`
  ///
  /// `describe()` reconstructs the command string independently of `build_command()`. Every CLI
  /// flag that `build_command()` emits from a **typed field** (not from `self.args`) MUST also
  /// appear in `describe()` at the corresponding position.
  ///
  /// Typed-field flags (currently `skip_permissions`, `chrome`, `continue_conversation`) are
  /// emitted directly in `build_command()` — NOT via `self.args`. Updating `build_command()`
  /// without updating `describe()` causes dry-run output to diverge from the actual command.
  ///
  /// Pitfall: always update both methods when adding a new typed-field CLI parameter.
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let desc = ClaudeCommand::new()
  ///   .with_working_directory( "/tmp" )
  ///   .with_skip_permissions( true )
  ///   .with_message( "hello" )
  ///   .describe();
  ///
  /// assert!( desc.contains( "cd /tmp" ) );
  /// assert!( desc.contains( "--dangerously-skip-permissions" ) );
  /// ```
  #[inline]
  #[must_use]
  pub fn describe( &self ) -> String {
    let mut lines = Vec::new();

    if let Some( ref dir ) = self.working_directory {
      lines.push( format!( "cd {}", dir.display() ) );
    }

    let mut parts = vec![ "claude".to_string() ];

    if self.skip_permissions {
      parts.push( "--dangerously-skip-permissions".to_string() );
    }

    // Emit chrome flag from typed field (default: Some(true) → --chrome)
    match self.chrome {
      Some( true )  => parts.push( "--chrome".to_string() ),
      Some( false ) => parts.push( "--no-chrome".to_string() ),
      None          => {}
    }

    for arg in &self.args {
      parts.push( arg.clone() );
    }

    if self.continue_conversation {
      parts.push( "-c".to_string() );
    }

    if let Some( ref msg ) = self.message {
      // Fix(issue-describe-backslash-escape): Escape `\` before `"` to prevent malformed shell output
      // Root cause: Only `"` was escaped, not `\`. Messages containing `\"` produced `\\"` in output
      // which shell parses as a closing double-quote, breaking the command representation.
      // Pitfall: Always escape `\` first, then `"`, when quoting for double-quoted shell strings.
      let escaped = msg.replace( '\\', "\\\\" ).replace( '"', "\\\"" );
      parts.push( format!( "\"{escaped}\"" ) );
    }

    lines.push( parts.join( " " ) );
    lines.join( "\n" )
  }

  /// Describe environment variables that would be set
  ///
  /// Returns one `NAME=VALUE` line per configured environment variable.
  /// Only includes variables that have been explicitly set (via defaults
  /// or builder methods). Omits `None` values.
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let env = ClaudeCommand::new().describe_env();
  ///
  /// assert!( env.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000" ) );
  /// assert!( env.contains( "CLAUDE_CODE_BASH_TIMEOUT=3600000" ) );
  /// ```
  #[inline]
  #[must_use]
  pub fn describe_env( &self ) -> String {
    let mut lines = Vec::new();

    if let Some( tokens ) = self.max_output_tokens {
      lines.push( format!( "CLAUDE_CODE_MAX_OUTPUT_TOKENS={tokens}" ) );
    }
    if let Some( timeout ) = self.bash_default_timeout_ms {
      lines.push( format!( "CLAUDE_CODE_BASH_TIMEOUT={timeout}" ) );
    }
    if let Some( max_timeout ) = self.bash_max_timeout_ms {
      lines.push( format!( "CLAUDE_CODE_BASH_MAX_TIMEOUT={max_timeout}" ) );
    }
    if let Some( auto_continue ) = self.auto_continue {
      lines.push( format!( "CLAUDE_CODE_AUTO_CONTINUE={auto_continue}" ) );
    }
    if let Some( telemetry ) = self.telemetry {
      lines.push( format!( "CLAUDE_CODE_TELEMETRY={telemetry}" ) );
    }
    if let Some( approve ) = self.auto_approve_tools {
      lines.push( format!( "CLAUDE_CODE_AUTO_APPROVE_TOOLS={approve}" ) );
    }
    if let Some( mode ) = self.action_mode {
      lines.push( format!( "CLAUDE_CODE_ACTION_MODE={}", mode.as_str() ) );
    }
    if let Some( level ) = self.log_level {
      lines.push( format!( "CLAUDE_CODE_LOG_LEVEL={}", level.as_str() ) );
    }
    if let Some( temp ) = self.temperature {
      lines.push( format!( "CLAUDE_CODE_TEMPERATURE={temp}" ) );
    }
    if let Some( sandbox ) = self.sandbox_mode {
      lines.push( format!( "CLAUDE_CODE_SANDBOX_MODE={sandbox}" ) );
    }
    if let Some( ref dir ) = self.session_dir {
      lines.push( format!( "CLAUDE_CODE_SESSION_DIR={}", dir.display() ) );
    }
    if let Some( top_p ) = self.top_p {
      lines.push( format!( "CLAUDE_CODE_TOP_P={top_p}" ) );
    }
    if let Some( top_k ) = self.top_k {
      lines.push( format!( "CLAUDE_CODE_TOP_K={top_k}" ) );
    }

    lines.join( "\n" )
  }

  /// Execute the Claude Code command and capture output (non-interactive mode)
  ///
  /// This is the SINGLE execution point for non-interactive Claude Code process invocations.
  /// For interactive sessions, use [`execute_interactive`](Self::execute_interactive).
  ///
  /// Returns [`ExecutionOutput`](crate::ExecutionOutput) with stdout, stderr, and exit code.
  ///
  /// # Errors
  ///
  /// Returns error if Claude Code binary not found in PATH or process fails to spawn.
  ///
  /// # Example
  ///
  /// ```no_run
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let result = ClaudeCommand::new()
  ///   .with_max_output_tokens( 200_000 )
  ///   .execute()?;
  /// println!( "{}", result.stdout );
  /// # Ok::<(), Box<dyn std::error::Error>>(())
  /// ```
  #[inline]
  pub fn execute( &self ) -> Result< crate::types::ExecutionOutput > {
    if self.dry_run {
      return Ok( crate::types::ExecutionOutput {
        stdout: self.describe_compact(),
        stderr: String::new(),
        exit_code: 0,
      } );
    }

    let mut cmd = self.build_command();

    let output = cmd.output()
      .map_err( |e| Error::msg( format!( "Failed to execute Claude Code: {e}" ) ) )?;

    let stdout = String::from_utf8_lossy( &output.stdout ).to_string();
    let stderr = String::from_utf8_lossy( &output.stderr ).to_string();
    let exit_code = output.status.code().unwrap_or( -1 );

    Ok( crate::types::ExecutionOutput { stdout, stderr, exit_code } )
  }

  /// Execute the Claude Code command in interactive mode (TTY attached)
  ///
  /// This method allows Claude Code to take over the terminal for interactive sessions.
  /// Unlike [`execute`](Self::execute), this doesnt capture output and instead lets
  /// Claude Code directly interact with the user's terminal.
  ///
  /// # Errors
  ///
  /// Returns error if Claude Code binary not found in PATH or process fails to spawn.
  ///
  /// # Example
  ///
  /// ```no_run
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let exit_status = ClaudeCommand::new()
  ///   .with_max_output_tokens( 200_000 )
  ///   .execute_interactive()?;
  /// # Ok::<(), Box<dyn std::error::Error>>(())
  /// ```
  #[inline]
  pub fn execute_interactive( &self ) -> Result< std::process::ExitStatus > {
    if self.dry_run {
      #[cfg( unix )]
      {
        use std::os::unix::process::ExitStatusExt;
        return Ok( std::process::ExitStatus::from_raw( 0 ) );
      }
      #[cfg( not( unix ) )]
      {
        // On non-Unix, run a no-op command to obtain a success ExitStatus
        let status = std::process::Command::new( "cmd" )
          .args( [ "/C", "exit", "0" ] )
          .status()
          .map_err( |e| Error::msg( format!( "Failed to create dry-run status: {e}" ) ) )?;
        return Ok( status );
      }
    }

    let mut cmd = self.build_command();

    let status = cmd.status()
      .map_err( |e| Error::msg( format!( "Failed to execute Claude Code: {e}" ) ) )?;

    Ok( status )
  }

  /// Build the Command instance with all configured parameters
  ///
  /// SINGLE EXECUTION POINT: This is the ONLY location where `Command::new("claude")` appears
  ///
  /// Pitfall: any typed-field CLI flag added here MUST also be added to `describe()` at the
  /// same relative position — otherwise dry-run output diverges from actual execution.
  /// Typed-field flags: `skip_permissions`, `chrome`, `continue_conversation` (see `describe()`).
  /// Flags pushed via `self.args` are automatically mirrored; only typed fields need manual sync.
  #[inline]
  fn build_command( &self ) -> std::process::Command {
    use std::process::Command;

    // SINGLE EXECUTION POINT: This is the ONLY location where `Command::new("claude")` appears
    let mut cmd = Command::new( "claude" );

    // Set working directory if provided
    if let Some( ref dir ) = self.working_directory {
      cmd.current_dir( dir );
    }

    // Set max output tokens (fixes token limit bug: 32K → 200K)
    if let Some( tokens ) = self.max_output_tokens {
      cmd.env( "CLAUDE_CODE_MAX_OUTPUT_TOKENS", tokens.to_string() );
    }

    // Tier 1: Critical parameters with different defaults
    if let Some( timeout ) = self.bash_default_timeout_ms {
      cmd.env( "CLAUDE_CODE_BASH_TIMEOUT", timeout.to_string() );
    }

    if let Some( max_timeout ) = self.bash_max_timeout_ms {
      cmd.env( "CLAUDE_CODE_BASH_MAX_TIMEOUT", max_timeout.to_string() );
    }

    if let Some( auto_continue ) = self.auto_continue {
      cmd.env( "CLAUDE_CODE_AUTO_CONTINUE", auto_continue.to_string() );
    }

    if let Some( telemetry ) = self.telemetry {
      cmd.env( "CLAUDE_CODE_TELEMETRY", telemetry.to_string() );
    }

    // Tier 2: Essential parameters (security-sensitive)
    if let Some( approve ) = self.auto_approve_tools {
      cmd.env( "CLAUDE_CODE_AUTO_APPROVE_TOOLS", approve.to_string() );
    }

    if let Some( mode ) = self.action_mode {
      cmd.env( "CLAUDE_CODE_ACTION_MODE", mode.as_str() );
    }

    if let Some( level ) = self.log_level {
      cmd.env( "CLAUDE_CODE_LOG_LEVEL", level.as_str() );
    }

    if let Some( temp ) = self.temperature {
      cmd.env( "CLAUDE_CODE_TEMPERATURE", temp.to_string() );
    }

    // Tier 3: Optional parameters
    if let Some( sandbox ) = self.sandbox_mode {
      cmd.env( "CLAUDE_CODE_SANDBOX_MODE", sandbox.to_string() );
    }

    if let Some( ref dir ) = self.session_dir {
      cmd.env( "CLAUDE_CODE_SESSION_DIR", dir.to_string_lossy().as_ref() );
    }

    if let Some( top_p ) = self.top_p {
      cmd.env( "CLAUDE_CODE_TOP_P", top_p.to_string() );
    }

    if let Some( top_k ) = self.top_k {
      cmd.env( "CLAUDE_CODE_TOP_K", top_k.to_string() );
    }

    // Add skip-permissions flag before custom args
    if self.skip_permissions {
      cmd.arg( "--dangerously-skip-permissions" );
    }

    // Emit chrome flag from typed field (default: Some(true) → --chrome)
    match self.chrome {
      Some( true )  => { cmd.arg( "--chrome" ); }
      Some( false ) => { cmd.arg( "--no-chrome" ); }
      None          => {}
    }

    // Add custom arguments
    for arg in &self.args {
      cmd.arg( arg );
    }

    // Add continuation flag if requested
    if self.continue_conversation {
      cmd.arg( "-c" );
    }

    // Add message last if provided
    if let Some( ref msg ) = self.message {
      cmd.arg( msg );
    }

    cmd
  }
}

/// Query installed Claude Code version.
///
/// Runs `claude --version` and returns trimmed stdout.
/// Returns `None` if binary not found or produces no output.
///
/// # Examples
///
/// ```no_run
/// if let Some( version ) = claude_runner_core::claude_version()
/// {
///   println!( "Claude Code version: {version}" );
/// }
/// ```
#[ inline ]
#[ must_use ]
pub fn claude_version() -> Option< String >
{
  let output = std::process::Command::new( "claude" )
    .arg( "--version" )
    .output()
    .ok()?;
  let s = String::from_utf8_lossy( &output.stdout ).trim().to_string();
  if s.is_empty() { None } else { Some( s ) }
}

impl Default for ClaudeCommand {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

// ============================================================================
// Testing Support
// ============================================================================
//
// Note: Uses #[doc(hidden)] instead of #[cfg(test)] because integration tests
// in tests/ directory need access to this method. Integration tests compile
// against the public API and cannot see #[cfg(test)] items from the library.

impl ClaudeCommand {
  /// Test helper: Expose built Command for inspection
  ///
  /// **FOR TESTING ONLY** - This method allows integration tests to inspect
  /// the constructed Command without executing it.
  ///
  /// # Why Public?
  ///
  /// Integration tests (in `tests/` directory) need this to verify command
  /// construction. Cannot use `#[cfg(test)]` because integration tests compile
  /// against the public API.
  ///
  /// # Do Not Use in Production
  ///
  /// This method is marked `#[doc(hidden)]` to prevent it from appearing in
  /// public documentation. It should only be used by tests in this crate.
  #[ doc( hidden ) ]
  #[ inline ]
  #[ must_use ]
  pub fn build_command_for_test( &self ) -> std::process::Command {
    self.build_command()
  }
}
