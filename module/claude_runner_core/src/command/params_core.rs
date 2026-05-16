//! Tier 1 critical parameters for `ClaudeCommand` builder.
//!
//! Contains `with_*` methods for parameters that have different defaults
//! from the standard Claude Code CLI (automation-blocker fixes).

use super::ClaudeCommand;
use std::path::PathBuf;

impl ClaudeCommand {
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

  /// Override the `HOME` environment variable for process isolation
  ///
  /// Sets `HOME=<path>` on the spawned process, directing it to use a different
  /// home directory. Used by `run_isolated()` to prevent credential contamination.
  ///
  /// # Example
  ///
  /// ```
  /// use claude_runner_core::ClaudeCommand;
  ///
  /// let cmd = ClaudeCommand::new()
  ///   .with_home( std::path::Path::new( "/tmp/isolated_home" ) );
  /// ```
  #[ inline ]
  #[ must_use ]
  pub fn with_home( mut self, path : &std::path::Path ) -> Self
  {
    self.home_override = Some( path.to_owned() );
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
}
