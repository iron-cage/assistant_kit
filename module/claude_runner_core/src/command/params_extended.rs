//! Tier 3+ extended parameters for `ClaudeCommand` builder.
//!
//! Contains `with_*` methods for optional parameters with standard defaults:
//! I/O, model/budget, MCP/extensions, session management, prompts/permissions,
//! debug/advanced, and terminal/IDE integration.

use super::ClaudeCommand;
use std::path::PathBuf;

impl ClaudeCommand {
  // ── Core optional parameters ─────────────────────────────────────────────

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
}
