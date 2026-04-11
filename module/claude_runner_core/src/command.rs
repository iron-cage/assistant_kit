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
  working_directory: Option<PathBuf>,
  max_output_tokens: Option<u32>,
  continue_conversation: bool,
  message: Option<String>,
  args: Vec<String>,

  // Tier 1: Critical parameters with different defaults (fix automation blockers)
  bash_default_timeout_ms: Option<u32>,
  bash_max_timeout_ms: Option<u32>,
  auto_continue: Option<bool>,
  telemetry: Option<bool>,

  // Tier 2: Essential parameters with standard defaults (security-sensitive)
  auto_approve_tools: Option<bool>,
  action_mode: Option<crate::types::ActionMode>,
  log_level: Option<crate::types::LogLevel>,
  temperature: Option<f64>,

  // Safety override
  skip_permissions: bool,

  // Terminal & IDE flags with non-standard builder defaults
  chrome: Option<bool>,

  // Tier 3: Optional parameters with standard defaults
  sandbox_mode: Option<bool>,
  session_dir: Option<PathBuf>,
  top_p: Option<f64>,
  top_k: Option<u32>,

  // Execution control
  dry_run: bool,
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

  /// Set working directory for Claude Code execution
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_working_directory("/home/user/project");
  /// ```
  #[inline]
  #[must_use]
  pub fn with_working_directory<P: Into<PathBuf>>( mut self, dir: P ) -> Self {
    self.working_directory = Some( dir.into() );
    self
  }

  /// Set maximum output tokens (default: 200,000)
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_max_output_tokens(200_000);
  /// ```
  #[inline]
  #[must_use]
  pub fn with_max_output_tokens( mut self, tokens: u32 ) -> Self {
    self.max_output_tokens = Some( tokens );
    self
  }

  /// Enable conversation continuation
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_continue_conversation(true);
  /// ```
  #[inline]
  #[must_use]
  pub fn with_continue_conversation( mut self, continue_: bool ) -> Self {
    self.continue_conversation = continue_;
    self
  }

  /// Set message to send to Claude
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_message("Explain this code");
  /// ```
  #[inline]
  #[must_use]
  pub fn with_message<S: Into<String>>( mut self, message: S ) -> Self {
    self.message = Some( message.into() );
    self
  }

  /// Add a single argument to the command
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_arg("--dangerously-skip-permissions");
  /// ```
  #[inline]
  #[must_use]
  pub fn with_arg<S: Into<String>>( mut self, arg: S ) -> Self {
    self.args.push( arg.into() );
    self
  }

  /// Add multiple arguments to the command
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_args(vec!["--dangerously-skip-permissions", "-c"]);
  /// ```
  #[inline]
  #[must_use]
  pub fn with_args<I, S>( mut self, args: I ) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    self.args.extend( args.into_iter().map( Into::into ) );
    self
  }

  /// Set Claude model
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_model("claude-opus-4-5");
  /// ```
  #[inline]
  #[must_use]
  pub fn with_model<S: Into<String>>( mut self, model: S ) -> Self {
    self.args.push( "--model".to_string() );
    self.args.push( model.into() );
    self
  }

  /// Set API key via environment variable
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_api_key("sk-ant-...");
  /// ```
  #[inline]
  #[must_use]
  pub fn with_api_key<S: Into<String>>( mut self, key: S ) -> Self {
    self.args.push( "--api-key".to_string() );
    self.args.push( key.into() );
    self
  }

  /// Enable verbose output
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_verbose(true);
  /// ```
  #[inline]
  #[must_use]
  pub fn with_verbose( mut self, verbose: bool ) -> Self {
    if verbose {
      self.args.push( "--verbose".to_string() );
    }
    self
  }

  /// Set system prompt
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_system_prompt("You are a helpful coding assistant");
  /// ```
  #[inline]
  #[must_use]
  pub fn with_system_prompt<S: Into<String>>( mut self, prompt: S ) -> Self {
    self.args.push( "--system-prompt".to_string() );
    self.args.push( prompt.into() );
    self
  }

  /// Set default bash command timeout in milliseconds
  ///
  /// Default: 3,600,000 ms (1 hour). Standard default: 120,000 ms (2 minutes).
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_bash_timeout_ms(3_600_000);  // 1 hour
  /// ```
  #[inline]
  #[must_use]
  pub fn with_bash_timeout_ms( mut self, timeout_ms: u32 ) -> Self {
    self.bash_default_timeout_ms = Some( timeout_ms );
    self
  }

  /// Set maximum bash command timeout in milliseconds
  ///
  /// Default: 7,200,000 ms (2 hours). Standard default: 600,000 ms (10 minutes).
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_bash_max_timeout_ms(7_200_000);  // 2 hours
  /// ```
  #[inline]
  #[must_use]
  pub fn with_bash_max_timeout_ms( mut self, timeout_ms: u32 ) -> Self {
    self.bash_max_timeout_ms = Some( timeout_ms );
    self
  }

  /// Enable or disable auto-continue mode
  ///
  /// Default: true. Standard default: false.
  /// When true, enables programmatic automation without manual prompts.
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_auto_continue(true);
  /// ```
  #[inline]
  #[must_use]
  pub fn with_auto_continue( mut self, auto_continue: bool ) -> Self {
    self.auto_continue = Some( auto_continue );
    self
  }

  /// Enable or disable telemetry
  ///
  /// Default: false. Standard default: true.
  /// Disables usage data collection in automation contexts.
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_telemetry(false);
  /// ```
  #[inline]
  #[must_use]
  pub fn with_telemetry( mut self, telemetry: bool ) -> Self {
    self.telemetry = Some( telemetry );
    self
  }

  /// Enable or disable auto-approval of tool executions
  ///
  /// Default: false (inherits standard). Security-sensitive: requires explicit opt-in.
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_auto_approve_tools(false);  // Explicit denial
  /// ```
  #[inline]
  #[must_use]
  pub fn with_auto_approve_tools( mut self, approve: bool ) -> Self {
    self.auto_approve_tools = Some( approve );
    self
  }

  /// Set action mode for tool execution
  ///
  /// Default: `ActionMode::Ask` (inherits standard).
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::{ ClaudeCommand, ActionMode };
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_action_mode(ActionMode::Ask);
  /// ```
  #[inline]
  #[must_use]
  pub fn with_action_mode( mut self, mode: crate::types::ActionMode ) -> Self {
    self.action_mode = Some( mode );
    self
  }

  /// Set logging verbosity level
  ///
  /// Default: `LogLevel::Info` (inherits standard).
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::{ ClaudeCommand, LogLevel };
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_log_level(LogLevel::Debug);
  /// ```
  #[inline]
  #[must_use]
  pub fn with_log_level( mut self, level: crate::types::LogLevel ) -> Self {
    self.log_level = Some( level );
    self
  }

  /// Set model temperature
  ///
  /// Default: 1.0 (inherits standard). Range: 0.0 to 1.0.
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_temperature(0.7);
  /// ```
  #[inline]
  #[must_use]
  pub fn with_temperature( mut self, temperature: f64 ) -> Self {
    self.temperature = Some( temperature );
    self
  }

  /// Enable or disable sandbox mode
  ///
  /// Default: true (inherits standard).
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_sandbox_mode(true);
  /// ```
  #[inline]
  #[must_use]
  pub fn with_sandbox_mode( mut self, sandbox: bool ) -> Self {
    self.sandbox_mode = Some( sandbox );
    self
  }

  /// Set explicit session directory
  ///
  /// Default: None (auto-detect). Overrides default session storage location.
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_session_dir("/tmp/sessions");
  /// ```
  #[inline]
  #[must_use]
  pub fn with_session_dir<P: Into<PathBuf>>( mut self, dir: P ) -> Self {
    self.session_dir = Some( dir.into() );
    self
  }

  /// Set top-p sampling parameter
  ///
  /// Default: None (inherits standard). Range: 0.0 to 1.0.
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_top_p(0.9);
  /// ```
  #[inline]
  #[must_use]
  pub fn with_top_p( mut self, top_p: f64 ) -> Self {
    self.top_p = Some( top_p );
    self
  }

  /// Set top-k sampling parameter
  ///
  /// Default: None (inherits standard).
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_top_k(40);
  /// ```
  #[inline]
  #[must_use]
  pub fn with_top_k( mut self, top_k: u32 ) -> Self {
    self.top_k = Some( top_k );
    self
  }

  // ── I/O Parameters (TSK-072) ─────────────────────────────────────────────

  /// Force non-interactive print mode (`-p`)
  ///
  /// When true, adds `-p` which forces non-interactive output and is required
  /// for reliable programmatic execution when no TTY is attached.
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_print( true );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_print( mut self, print: bool ) -> Self {
    if print {
      self.args.push( "-p".to_string() );
    }
    self
  }

  /// Set output format
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::{ ClaudeCommand, OutputFormat };
  ///
  /// let cmd = ClaudeCommand::new().with_output_format( OutputFormat::Json );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_output_format( mut self, format: crate::types::OutputFormat ) -> Self {
    self.args.push( "--output-format".to_string() );
    self.args.push( format.as_str().to_string() );
    self
  }

  /// Set input format
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::{ ClaudeCommand, InputFormat };
  ///
  /// let cmd = ClaudeCommand::new().with_input_format( InputFormat::StreamJson );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_input_format( mut self, format: crate::types::InputFormat ) -> Self {
    self.args.push( "--input-format".to_string() );
    self.args.push( format.as_str().to_string() );
    self
  }

  /// Enable or disable partial message streaming
  ///
  /// When true, adds `--include-partial-messages` for token-by-token streaming
  /// with `stream-json` output format.
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_include_partial_messages( true );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_include_partial_messages( mut self, include: bool ) -> Self {
    if include {
      self.args.push( "--include-partial-messages".to_string() );
    }
    self
  }

  /// Enable or disable replaying user messages on stdout
  ///
  /// When true, adds `--replay-user-messages`.
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_replay_user_messages( true );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_replay_user_messages( mut self, replay: bool ) -> Self {
    if replay {
      self.args.push( "--replay-user-messages".to_string() );
    }
    self
  }

  /// Constrain output to a JSON Schema
  ///
  /// Adds `--json-schema <schema>` where schema is passed verbatim.
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_json_schema( r#"{"type":"object"}"# );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_json_schema<S: Into<String>>( mut self, schema: S ) -> Self {
    self.args.push( "--json-schema".to_string() );
    self.args.push( schema.into() );
    self
  }

  // ── Tool Directory Parameters (TSK-073) ──────────────────────────────────

  /// Add a directory to Claude's accessible paths (Pattern F: repeated-flag)
  ///
  /// Each call adds a `--add-dir <path>` pair. Call multiple times to add
  /// multiple directories.
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_add_dir( "/src" )
  ///   .with_add_dir( "/tests" );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_add_dir<S: Into<String>>( mut self, path: S ) -> Self {
    self.args.push( "--add-dir".to_string() );
    self.args.push( path.into() );
    self
  }

  /// Set allowed tools list (Pattern E: one flag + N space-separated values)
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_allowed_tools( [ "bash", "read" ] );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_allowed_tools<I, S>( mut self, tools: I ) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    // Fix(issue-pattern-e-empty-iterator): Collect first; skip flag entirely if empty
    // Root cause: Pushing flag header before loop left "--allowedTools" with no values
    // when called with an empty iterator, causing CLI to misparse next positional arg
    // Pitfall: Pattern E must collect before pushing flag; Pattern F is immune (flag inside loop)
    let collected : Vec<String> = tools.into_iter().map( Into::into ).collect();
    if !collected.is_empty() {
      self.args.push( "--allowedTools".to_string() );
      self.args.extend( collected );
    }
    self
  }

  /// Set disallowed tools list (Pattern E: one flag + N space-separated values)
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_disallowed_tools( [ "bash" ] );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_disallowed_tools<I, S>( mut self, tools: I ) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    // Fix(issue-pattern-e-empty-iterator): Collect first; skip flag entirely if empty
    // Root cause: Same as with_allowed_tools — flag header before loop, empty iter = orphan
    // Pitfall: Always collect Pattern E iterators before pushing the flag header
    let collected : Vec<String> = tools.into_iter().map( Into::into ).collect();
    if !collected.is_empty() {
      self.args.push( "--disallowedTools".to_string() );
      self.args.extend( collected );
    }
    self
  }

  /// Set tools list (Pattern E: one flag + N space-separated values)
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_tools( [ "bash" ] );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_tools<I, S>( mut self, tools: I ) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    // Fix(issue-pattern-e-empty-iterator): Collect first; skip flag entirely if empty
    // Root cause: Same as with_allowed_tools — flag header before loop, empty iter = orphan
    // Pitfall: Always collect Pattern E iterators before pushing the flag header
    let collected : Vec<String> = tools.into_iter().map( Into::into ).collect();
    if !collected.is_empty() {
      self.args.push( "--tools".to_string() );
      self.args.extend( collected );
    }
    self
  }

  // ── Model and Budget Parameters (TSK-076) ───────────────────────────────

  /// Set reasoning effort level
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::{ ClaudeCommand, EffortLevel };
  ///
  /// let cmd = ClaudeCommand::new().with_effort( EffortLevel::High );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_effort( mut self, level: crate::types::EffortLevel ) -> Self {
    self.args.push( "--effort".to_string() );
    self.args.push( level.as_str().to_string() );
    self
  }

  /// Set fallback model when the primary is unavailable
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_fallback_model( "claude-haiku-4-5" );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_fallback_model<S: Into<String>>( mut self, model: S ) -> Self {
    self.args.push( "--fallback-model".to_string() );
    self.args.push( model.into() );
    self
  }

  /// Cap the API spend in USD for this session
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_max_budget_usd( 0.50 );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_max_budget_usd( mut self, amount: f64 ) -> Self {
    self.args.push( "--max-budget-usd".to_string() );
    // Fix(issue-max-budget-float): Use clean float serialization via Display
    // Root cause: f64 Display in Rust produces clean minimal strings ("0.5" not "0.50000")
    // Pitfall: Do not use {:?} (Debug) or manual truncation — Display is correct here
    self.args.push( format!( "{amount}" ) );
    self
  }

  // ── MCP and Extension Parameters (TSK-077) ──────────────────────────────

  /// Load MCP servers from JSON config files (Pattern F: repeated-flag per value)
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_mcp_config( [ "/path/mcp.json" ] );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_mcp_config<I, S>( mut self, configs: I ) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    for config in configs {
      self.args.push( "--mcp-config".to_string() );
      self.args.push( config.into() );
    }
    self
  }

  /// Disable all non-`--mcp-config` MCP servers
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_strict_mcp_config( true );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_strict_mcp_config( mut self, strict: bool ) -> Self {
    if strict {
      self.args.push( "--strict-mcp-config".to_string() );
    }
    self
  }

  /// Load a settings file or inline JSON string
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_settings( "/path/settings.json" );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_settings<S: Into<String>>( mut self, settings: S ) -> Self {
    self.args.push( "--settings".to_string() );
    self.args.push( settings.into() );
    self
  }

  /// Filter which setting sources load
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_setting_sources( "local" );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_setting_sources<S: Into<String>>( mut self, sources: S ) -> Self {
    self.args.push( "--setting-sources".to_string() );
    self.args.push( sources.into() );
    self
  }

  /// Override the agent for this session
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_agent( "reviewer" );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_agent<S: Into<String>>( mut self, agent: S ) -> Self {
    self.args.push( "--agent".to_string() );
    self.args.push( agent.into() );
    self
  }

  /// Define custom agents as a single JSON string
  ///
  /// **CRITICAL**: Takes a single JSON string — NOT an iterator. The entire agents
  /// definition is one `--agents <json>` pair.
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_agents( r#"[{"name":"bot","model":"claude-opus-4-6"}]"# );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_agents<S: Into<String>>( mut self, json: S ) -> Self {
    self.args.push( "--agents".to_string() );
    self.args.push( json.into() );
    self
  }

  /// Load plugins from directories (Pattern F: repeated-flag per value)
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_plugin_dir( [ "/plugins" ] );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_plugin_dir<I, S>( mut self, dirs: I ) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    for dir in dirs {
      self.args.push( "--plugin-dir".to_string() );
      self.args.push( dir.into() );
    }
    self
  }

  // ── Session Parameters (TSK-074) ─────────────────────────────────────────

  /// Resume the most recent conversation, or a specific session by ID
  ///
  /// - `with_resume(None)` adds `-r` (resume most recent)
  /// - `with_resume(Some("uuid"))` adds `-r uuid` (resume specific session)
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_resume( None );                   // most recent
  /// let cmd = ClaudeCommand::new().with_resume( Some( "abc-123" ) );     // specific
  /// ```
  #[inline]
  #[must_use]
  pub fn with_resume( mut self, id: Option<&str> ) -> Self {
    self.args.push( "-r".to_string() );
    if let Some( session_id ) = id {
      self.args.push( session_id.to_string() );
    }
    self
  }

  /// Pin the session UUID for this invocation
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_session_id( "dead-beef-0000" );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_session_id<S: Into<String>>( mut self, id: S ) -> Self {
    self.args.push( "--session-id".to_string() );
    self.args.push( id.into() );
    self
  }

  /// Create a new session ID on resume (fork)
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_fork_session( true );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_fork_session( mut self, fork: bool ) -> Self {
    if fork {
      self.args.push( "--fork-session".to_string() );
    }
    self
  }

  /// Disable session persistence (do not save to disk)
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_no_session_persistence( true );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_no_session_persistence( mut self, no_persist: bool ) -> Self {
    if no_persist {
      self.args.push( "--no-session-persistence".to_string() );
    }
    self
  }

  /// Resume a session linked to a PR
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_from_pr( "42" );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_from_pr<S: Into<String>>( mut self, value: S ) -> Self {
    self.args.push( "--from-pr".to_string() );
    self.args.push( value.into() );
    self
  }

  // ── Prompt and Permission Parameters (TSK-075) ───────────────────────────

  /// Append text to the default system prompt (without replacing it)
  ///
  /// Complementary to `with_system_prompt()` which replaces the system prompt.
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_append_system_prompt( "Always respond in JSON" );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_append_system_prompt<S: Into<String>>( mut self, prompt: S ) -> Self {
    self.args.push( "--append-system-prompt".to_string() );
    self.args.push( prompt.into() );
    self
  }

  /// Set fine-grained permission mode
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::{ ClaudeCommand, PermissionMode };
  ///
  /// let cmd = ClaudeCommand::new().with_permission_mode( PermissionMode::AcceptEdits );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_permission_mode( mut self, mode: crate::types::PermissionMode ) -> Self {
    self.args.push( "--permission-mode".to_string() );
    self.args.push( mode.as_str().to_string() );
    self
  }

  /// Allow the `--dangerously-skip-permissions` flag without activating it
  ///
  /// Distinct from `with_skip_permissions()` which unconditionally activates it.
  /// This method enables the option without triggering it.
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_allow_dangerously_skip_permissions( true );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_allow_dangerously_skip_permissions( mut self, allow: bool ) -> Self {
    if allow {
      self.args.push( "--allow-dangerously-skip-permissions".to_string() );
    }
    self
  }

  // ── Debug and Advanced Parameters (TSK-078) ─────────────────────────────

  /// Enable debug output, with an optional category filter
  ///
  /// - `with_debug(None)` adds `-d` (all debug categories)
  /// - `with_debug(Some("mcp"))` adds `-d mcp` (filtered)
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_debug( None );
  /// let cmd = ClaudeCommand::new().with_debug( Some( "mcp" ) );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_debug( mut self, filter: Option<&str> ) -> Self {
    self.args.push( "-d".to_string() );
    if let Some( f ) = filter {
      self.args.push( f.to_string() );
    }
    self
  }

  /// Redirect debug logs to a file
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_debug_file( "/tmp/debug.log" );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_debug_file<S: Into<String>>( mut self, path: S ) -> Self {
    self.args.push( "--debug-file".to_string() );
    self.args.push( path.into() );
    self
  }

  /// Enable beta API headers (Pattern F: repeated-flag per beta)
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_betas( [ "computer-use-2024-10-22" ] );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_betas<I, S>( mut self, betas: I ) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    for beta in betas {
      self.args.push( "--betas".to_string() );
      self.args.push( beta.into() );
    }
    self
  }

  /// Enable the `SendUserMessage` tool for sub-agents
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_brief( true );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_brief( mut self, brief: bool ) -> Self {
    if brief {
      self.args.push( "--brief".to_string() );
    }
    self
  }

  /// Disable all slash command skills
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_disable_slash_commands( true );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_disable_slash_commands( mut self, disable: bool ) -> Self {
    if disable {
      self.args.push( "--disable-slash-commands".to_string() );
    }
    self
  }

  /// Download file resources at startup (Pattern F: repeated-flag per spec)
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_file( [ "https://example.com/data.json" ] );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_file<I, S>( mut self, specs: I ) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    for spec in specs {
      self.args.push( "--file".to_string() );
      self.args.push( spec.into() );
    }
    self
  }

  // ── Terminal and IDE Parameters (TSK-079) ────────────────────────────────

  /// Create a git worktree for the session (optional name)
  ///
  /// - `with_worktree(None)` adds `-w` (auto-name)
  /// - `with_worktree(Some("feature"))` adds `-w feature`
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_worktree( None );
  /// let cmd = ClaudeCommand::new().with_worktree( Some( "feature" ) );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_worktree( mut self, name: Option<&str> ) -> Self {
    self.args.push( "-w".to_string() );
    if let Some( n ) = name {
      self.args.push( n.to_string() );
    }
    self
  }

  /// Create a tmux session for the worktree
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_tmux( true );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_tmux( mut self, tmux: bool ) -> Self {
    if tmux {
      self.args.push( "--tmux".to_string() );
    }
    self
  }

  /// Auto-connect to IDE on startup
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new().with_ide( true );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_ide( mut self, ide: bool ) -> Self {
    if ide {
      self.args.push( "--ide".to_string() );
    }
    self
  }

  /// Toggle Claude-in-Chrome integration (Pattern G: tri-state)
  ///
  /// - `Some(true)` → `--chrome`
  /// - `Some(false)` → `--no-chrome`
  /// - `None` → omit flag entirely (overrides `Some(true)` builder default; defers to Claude's own config)
  ///
  /// Builder default is `Some(true)` — `--chrome` is emitted unless explicitly overridden.
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// // Default: chrome already on — no call needed
  /// let cmd = ClaudeCommand::new();
  ///
  /// // Explicitly disable chrome for this invocation
  /// let cmd = ClaudeCommand::new().with_chrome( Some( false ) );  // --no-chrome
  ///
  /// // Defer to Claude's own config (omit flag entirely, overrides default)
  /// let cmd = ClaudeCommand::new().with_chrome( None );           // omit
  /// ```
  #[inline]
  #[must_use]
  pub fn with_chrome( mut self, enabled: Option<bool> ) -> Self {
    // Fix(issue-chrome-tristate): tri-state — Some(true)=--chrome, Some(false)=--no-chrome, None=omit
    // Root cause: chrome has both positive and negative flags; None must produce no output at all
    // Pitfall: Never collapse Some(false) to None — the negative flag IS meaningful to the CLI
    self.chrome = enabled;
    self
  }

  /// Enable `--dangerously-skip-permissions` flag
  ///
  /// When true, adds the `--dangerously-skip-permissions` flag to bypass
  /// tool permission prompts. Use with caution in automated pipelines only.
  ///
  /// Default: false.
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_skip_permissions( true );
  /// ```
  #[inline]
  #[must_use]
  pub fn with_skip_permissions( mut self, skip: bool ) -> Self {
    self.skip_permissions = skip;
    self
  }

  /// Enable or disable dry-run mode
  ///
  /// When true, `execute()` and `execute_interactive()` short-circuit without
  /// spawning a real process. `execute()` returns `describe_compact()` as stdout
  /// with exit code 0. Useful for inspecting what would be executed.
  ///
  /// Default: false.
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let output = ClaudeCommand::new()
  ///   .with_message( "hello" )
  ///   .with_dry_run( true )
  ///   .execute()?;
  /// assert!( output.stdout.starts_with( "claude" ) );
  /// # Ok::<(), Box<dyn std::error::Error>>(())
  /// ```
  #[inline]
  #[must_use]
  pub fn with_dry_run( mut self, dry_run: bool ) -> Self {
    self.dry_run = dry_run;
    self
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
