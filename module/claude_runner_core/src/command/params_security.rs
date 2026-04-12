//! Tier 2 essential parameters for `ClaudeCommand` builder.
//!
//! Contains `with_*` methods for security-sensitive parameters that have
//! standard defaults and require explicit opt-in, plus tool permission methods.

use super::ClaudeCommand;

impl ClaudeCommand {
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
}
