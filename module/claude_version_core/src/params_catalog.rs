//! Claude Code parameter catalog: all observable params with their CLI, env,
//! and config forms plus catalog default values.
//!
//! The catalog is the source of truth for the `.params` command. Each entry
//! describes one logical Claude Code parameter that may be observable via at
//! least one of: CLI flag, environment variable, or settings.json config key.
//!
//! See `docs/cli/command/params.md` and `docs/feature/007_params_command.md`
//! for the specification.

/// A Claude Code parameter definition.
///
/// One entry per logical parameter. A parameter may be observable through any
/// combination of CLI flag, env var, and config key. Parameters with only a
/// CLI flag and no env var or config key are "CLI-only" and unobservable
/// outside of a running process.
#[ derive( Debug ) ]
pub struct ParamDef
{
  /// Canonical short name used as the lookup key (e.g., `"model"`).
  pub name       : &'static str,
  /// CLI flag form (e.g., `"--model"` or `"-p / --print"`). `None` = no CLI flag.
  pub cli_flag   : Option< &'static str >,
  /// Environment variable name (e.g., `"CLAUDE_MODEL"`). `None` = not env-settable.
  pub env_var    : Option< &'static str >,
  /// settings.json key (e.g., `"model"`). `None` = not config-settable.
  pub config_key : Option< &'static str >,
  /// Catalog default value. `None` = no known default.
  pub default    : Option< &'static str >,
}

impl ParamDef
{
  /// Returns `true` if this parameter is only settable via CLI flag.
  ///
  /// CLI-only params cannot be persisted — they are unobservable outside
  /// an active process invocation.
  #[ inline ]
  #[ must_use ]
  pub fn is_cli_only( &self ) -> bool
  {
    self.cli_flag.is_some() && self.env_var.is_none() && self.config_key.is_none()
  }

  /// Returns `true` if this parameter has a config-key form.
  #[ inline ]
  #[ must_use ]
  pub fn has_config( &self ) -> bool
  {
    self.config_key.is_some()
  }

  /// Returns `true` if this parameter has an env-var form.
  #[ inline ]
  #[ must_use ]
  pub fn has_env( &self ) -> bool
  {
    self.env_var.is_some()
  }
}

/// Lookup a single parameter by canonical name.
///
/// Returns `None` if the name is not in the catalog.
#[ inline ]
#[ must_use ]
pub fn lookup( name : &str ) -> Option< &'static ParamDef >
{
  params_catalog().iter().find( |p| p.name == name )
}

/// Return a static slice of all known Claude Code parameter definitions,
/// sorted alphabetically by `name`.
///
/// Catalog entries per `docs/cli/command/params.md` § Catalog.
// Static data declaration — line count is inherent to the catalog size, not complexity.
#[ allow( clippy::too_many_lines ) ]
#[ inline ]
#[ must_use ]
pub fn params_catalog() -> &'static [ ParamDef ]
{
  static ENTRIES : &[ ParamDef ] = &[
    ParamDef
    {
      name       : "action_mode",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_ACTION_MODE" ),
      config_key : None,
      default    : Some( "Ask" ),
    },
    ParamDef
    {
      name       : "add_dir",
      cli_flag   : Some( "--add-dir" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "agent",
      cli_flag   : Some( "--agent" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "agents",
      cli_flag   : Some( "--agents" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "allow_dangerously_skip_permissions",
      cli_flag   : Some( "--allow-dangerously-skip-permissions" ),
      env_var    : None,
      config_key : None,
      default    : Some( "off" ),
    },
    ParamDef
    {
      name       : "allowed_tools",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "allowedTools" ),
      default    : None,
    },
    ParamDef
    {
      name       : "api_force_idle_timeout",
      cli_flag   : None,
      env_var    : Some( "API_FORCE_IDLE_TIMEOUT" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "api_key",
      cli_flag   : None,
      env_var    : Some( "ANTHROPIC_API_KEY" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "api_url",
      cli_flag   : None,
      env_var    : Some( "ANTHROPIC_API_URL" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "append_system",
      cli_flag   : Some( "--append-system-prompt" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "auto_approve_tools",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_AUTO_APPROVE_TOOLS" ),
      config_key : None,
      default    : Some( "false" ),
    },
    ParamDef
    {
      name       : "auto_compact_window",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ),
      config_key : None,
      default    : Some( "200000" ),
    },
    ParamDef
    {
      name       : "auto_continue",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_AUTO_CONTINUE" ),
      config_key : None,
      default    : Some( "false" ),
    },
    ParamDef
    {
      name       : "auto_updates",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "autoUpdates" ),
      default    : Some( "true" ),
    },
    ParamDef
    {
      name       : "auto_updates_channel",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "autoUpdatesChannel" ),
      default    : Some( "latest" ),
    },
    ParamDef
    {
      name       : "autocompact_pct_override",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_AUTOCOMPACT_PCT_OVERRIDE" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "bash_default_timeout_ms",
      cli_flag   : None,
      env_var    : Some( "BASH_DEFAULT_TIMEOUT_MS" ),
      config_key : None,
      default    : Some( "120000" ),
    },
    ParamDef
    {
      name       : "bash_max_output_length",
      cli_flag   : None,
      env_var    : Some( "BASH_MAX_OUTPUT_LENGTH" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "bash_max_timeout",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_BASH_MAX_TIMEOUT" ),
      config_key : None,
      default    : Some( "600000" ),
    },
    ParamDef
    {
      name       : "bash_max_timeout_ms",
      cli_flag   : None,
      env_var    : Some( "BASH_MAX_TIMEOUT_MS" ),
      config_key : None,
      default    : Some( "600000" ),
    },
    ParamDef
    {
      name       : "bash_timeout",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_BASH_TIMEOUT" ),
      config_key : None,
      default    : Some( "120000" ),
    },
    ParamDef
    {
      name       : "betas",
      cli_flag   : Some( "--betas" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "brief",
      cli_flag   : Some( "--brief" ),
      env_var    : None,
      config_key : None,
      default    : Some( "off" ),
    },
    ParamDef
    {
      name       : "chrome",
      cli_flag   : Some( "--chrome / --no-chrome" ),
      env_var    : None,
      config_key : None,
      default    : Some( "off" ),
    },
    ParamDef
    {
      name       : "client_presence_file",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CLIENT_PRESENCE_FILE" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "config_file",
      cli_flag   : Some( "--config" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "continue_session",
      cli_flag   : Some( "--continue" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "custom_instructions",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "customInstructions" ),
      default    : None,
    },
    ParamDef
    {
      name       : "cwd",
      cli_flag   : Some( "--cwd" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "debug",
      cli_flag   : Some( "--debug" ),
      env_var    : Some( "ANTHROPIC_DEBUG" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "debug_file",
      cli_flag   : Some( "--debug-file" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "default_sonnet_model",
      cli_flag   : None,
      env_var    : Some( "ANTHROPIC_DEFAULT_SONNET_MODEL" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "disable_auto_compact",
      cli_flag   : None,
      env_var    : Some( "DISABLE_AUTO_COMPACT" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "disable_autoupdater",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "env.DISABLE_AUTOUPDATER" ),
      default    : None,
    },
    ParamDef
    {
      name       : "disable_bundled_skills",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_DISABLE_BUNDLED_SKILLS" ),
      config_key : Some( "disableBundledSkills" ),
      default    : Some( "false" ),
    },
    ParamDef
    {
      name       : "disable_compact",
      cli_flag   : None,
      env_var    : Some( "DISABLE_COMPACT" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "disable_cost_warnings",
      cli_flag   : None,
      env_var    : Some( "DISABLE_COST_WARNINGS" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "disable_doctor_command",
      cli_flag   : None,
      env_var    : Some( "DISABLE_DOCTOR_COMMAND" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "disable_error_reporting",
      cli_flag   : None,
      env_var    : Some( "DISABLE_ERROR_REPORTING" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "disable_extra_usage_command",
      cli_flag   : None,
      env_var    : Some( "DISABLE_EXTRA_USAGE_COMMAND" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "disable_feedback_command",
      cli_flag   : None,
      env_var    : Some( "DISABLE_FEEDBACK_COMMAND" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "disable_growthbook",
      cli_flag   : None,
      env_var    : Some( "DISABLE_GROWTHBOOK" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "disable_install_github_app_command",
      cli_flag   : None,
      env_var    : Some( "DISABLE_INSTALL_GITHUB_APP_COMMAND" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "disable_installation_checks",
      cli_flag   : None,
      env_var    : Some( "DISABLE_INSTALLATION_CHECKS" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "disable_interleaved_thinking",
      cli_flag   : None,
      env_var    : Some( "DISABLE_INTERLEAVED_THINKING" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "disable_login_command",
      cli_flag   : None,
      env_var    : Some( "DISABLE_LOGIN_COMMAND" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "disable_logout_command",
      cli_flag   : None,
      env_var    : Some( "DISABLE_LOGOUT_COMMAND" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "disable_nonessential_traffic",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "disable_prompt_caching",
      cli_flag   : None,
      env_var    : Some( "DISABLE_PROMPT_CACHING" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "disable_prompt_caching_fable",
      cli_flag   : None,
      env_var    : Some( "DISABLE_PROMPT_CACHING_FABLE" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "disable_prompt_caching_haiku",
      cli_flag   : None,
      env_var    : Some( "DISABLE_PROMPT_CACHING_HAIKU" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "disable_prompt_caching_opus",
      cli_flag   : None,
      env_var    : Some( "DISABLE_PROMPT_CACHING_OPUS" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "disable_prompt_caching_sonnet",
      cli_flag   : None,
      env_var    : Some( "DISABLE_PROMPT_CACHING_SONNET" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "disable_slash_commands",
      cli_flag   : Some( "--disable-slash-commands" ),
      env_var    : None,
      config_key : None,
      default    : Some( "off" ),
    },
    ParamDef
    {
      name       : "disable_telemetry",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_DISABLE_TELEMETRY" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "disable_updates",
      cli_flag   : None,
      env_var    : Some( "DISABLE_UPDATES" ),
      config_key : Some( "env.DISABLE_UPDATES" ),
      default    : None,
    },
    ParamDef
    {
      name       : "disable_upgrade_command",
      cli_flag   : None,
      env_var    : Some( "DISABLE_UPGRADE_COMMAND" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "disallowed_tools",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "disallowedTools" ),
      default    : None,
    },
    ParamDef
    {
      name       : "effort",
      cli_flag   : Some( "--effort" ),
      env_var    : None,
      config_key : Some( "effortLevel" ),
      default    : Some( "medium" ),
    },
    ParamDef
    {
      name       : "enable_auto_mode",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_ENABLE_AUTO_MODE" ),
      config_key : None,
      default    : Some( "false" ),
    },
    ParamDef
    {
      name       : "enabled_plugins",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "enabledPlugins" ),
      default    : Some( "{}" ),
    },
    ParamDef
    {
      name       : "env_overrides",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "env" ),
      default    : Some( "{}" ),
    },
    ParamDef
    {
      name       : "experimental_agent_teams",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS" ),
      config_key : None,
      default    : Some( "false" ),
    },
    ParamDef
    {
      name       : "fallback_model",
      cli_flag   : Some( "--fallback-model" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "file",
      cli_flag   : Some( "--file" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "file_checkpointing_enabled",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "fileCheckpointingEnabled" ),
      default    : Some( "false" ),
    },
    ParamDef
    {
      name       : "fork_session",
      cli_flag   : Some( "--fork-session" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "from_pr",
      cli_flag   : Some( "--from-pr" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "has_completed_onboarding",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "hasCompletedOnboarding" ),
      default    : Some( "false" ),
    },
    ParamDef
    {
      name       : "hooks",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "hooks" ),
      default    : Some( "{}" ),
    },
    ParamDef
    {
      name       : "ide",
      cli_flag   : Some( "--ide" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "ide_hint",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_IDE_HINT" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "include_partial_messages",
      cli_flag   : Some( "--include-partial-messages" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "input_format",
      cli_flag   : Some( "--input-format" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "json_schema",
      cli_flag   : Some( "--json-schema" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "language",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "language" ),
      default    : None,
    },
    ParamDef
    {
      name       : "log_level",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_LOG_LEVEL" ),
      config_key : None,
      default    : Some( "Info" ),
    },
    ParamDef
    {
      name       : "max_budget_usd",
      cli_flag   : Some( "--max-budget-usd" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "max_output_tokens",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_MAX_OUTPUT_TOKENS" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "max_tokens",
      cli_flag   : Some( "--max-tokens" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "max_turns",
      cli_flag   : Some( "--max-turns" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "mcp_config",
      cli_flag   : Some( "--mcp-config" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "mcp_debug",
      cli_flag   : Some( "--mcp-debug" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "mcp_server",
      cli_flag   : Some( "--mcp-server" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "mcp_servers",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "mcpServers" ),
      default    : Some( "{}" ),
    },
    ParamDef
    {
      name       : "mcp_timeout",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_MCP_TIMEOUT" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "mcp_tool_timeout",
      cli_flag   : None,
      env_var    : Some( "MCP_TOOL_TIMEOUT" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "minimum_version",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "minimumVersion" ),
      default    : None,
    },
    ParamDef
    {
      name       : "model",
      cli_flag   : Some( "--model" ),
      env_var    : Some( "CLAUDE_MODEL" ),
      config_key : Some( "model" ),
      default    : Some( "claude-sonnet-5" ),
    },
    ParamDef
    {
      name       : "no_color",
      cli_flag   : Some( "--no-color" ),
      env_var    : Some( "NO_COLOR" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "no_session_persistence",
      cli_flag   : Some( "--no-session-persistence" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "output_format",
      cli_flag   : Some( "--output-format" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "output_style",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "outputStyle" ),
      default    : Some( "default" ),
    },
    ParamDef
    {
      name       : "package_manager_auto_update",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_PACKAGE_MANAGER_AUTO_UPDATE" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "permission_mode",
      cli_flag   : Some( "--permission-mode" ),
      env_var    : None,
      config_key : Some( "permissionMode" ),
      default    : None,
    },
    ParamDef
    {
      name       : "permissions",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "permissions" ),
      default    : Some( "{}" ),
    },
    ParamDef
    {
      name       : "plugin_dir",
      cli_flag   : Some( "--plugin-dir" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "plugin_prefer_https",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_PLUGIN_PREFER_HTTPS" ),
      config_key : None,
      default    : Some( "false" ),
    },
    ParamDef
    {
      name       : "preferred_version_resolved",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "preferredVersionResolved" ),
      default    : None,
    },
    ParamDef
    {
      name       : "preferred_version_spec",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "preferredVersionSpec" ),
      default    : Some( "stable" ),
    },
    ParamDef
    {
      name       : "prefill",
      cli_flag   : Some( "--prefill" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "print",
      cli_flag   : Some( "-p / --print" ),
      env_var    : None,
      config_key : None,
      default    : Some( "off" ),
    },
    ParamDef
    {
      name       : "prompt",
      cli_flag   : Some( "<message> (positional)" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "ps_execution_policy",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_POWERSHELL_RESPECT_EXECUTION_POLICY" ),
      config_key : None,
      default    : Some( "false" ),
    },
    ParamDef
    {
      name       : "quiet",
      cli_flag   : Some( "--quiet" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "remote_control_at_startup",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "remoteControlAtStartup" ),
      default    : Some( "false" ),
    },
    ParamDef
    {
      name       : "replay_user_messages",
      cli_flag   : Some( "--replay-user-messages" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "required_maximum_version",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "requiredMaximumVersion" ),
      default    : None,
    },
    ParamDef
    {
      name       : "required_minimum_version",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "requiredMinimumVersion" ),
      default    : None,
    },
    ParamDef
    {
      name       : "resume",
      cli_flag   : Some( "--resume" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "safe_mode",
      cli_flag   : Some( "--safe-mode" ),
      env_var    : Some( "CLAUDE_CODE_SAFE_MODE" ),
      config_key : None,
      default    : Some( "false" ),
    },
    ParamDef
    {
      name       : "sandbox_allow_apple_events",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "sandbox.allowAppleEvents" ),
      default    : Some( "false" ),
    },
    ParamDef
    {
      name       : "sandbox_mode",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_SANDBOX_MODE" ),
      config_key : None,
      default    : Some( "true" ),
    },
    ParamDef
    {
      name       : "session_dir",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_SESSION_DIR" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "session_id",
      cli_flag   : Some( "--session-id" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "setting_sources",
      cli_flag   : Some( "--setting-sources" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "settings",
      cli_flag   : Some( "--settings" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "skip_dangerous_mode_permission_prompt",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "skipDangerousModePermissionPrompt" ),
      default    : Some( "false" ),
    },
    ParamDef
    {
      name       : "skip_permissions",
      cli_flag   : Some( "--dangerously-skip-permissions" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "stop_hook_block_cap",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_STOP_HOOK_BLOCK_CAP" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "strict_mcp_config",
      cli_flag   : Some( "--strict-mcp-config" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "subagent_model",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_SUBAGENT_MODEL" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "system_prompt",
      cli_flag   : Some( "--system-prompt" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "telemetry",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_TELEMETRY" ),
      config_key : None,
      default    : Some( "true" ),
    },
    ParamDef
    {
      name       : "temperature",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_TEMPERATURE" ),
      config_key : None,
      default    : Some( "1.0" ),
    },
    ParamDef
    {
      name       : "theme",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "theme" ),
      default    : Some( "system" ),
    },
    ParamDef
    {
      name       : "thinking_disabled_display",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "thinking.disabled.display" ),
      default    : None,
    },
    ParamDef
    {
      name       : "tmpdir",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_TMPDIR" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "tmux",
      cli_flag   : Some( "--tmux" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "tools",
      cli_flag   : Some( "--tools" ),
      env_var    : None,
      config_key : None,
      default    : Some( "default" ),
    },
    ParamDef
    {
      name       : "top_k",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_TOP_K" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "top_p",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_TOP_P" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "verbose",
      cli_flag   : Some( "--verbose" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "voice_enabled",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "voiceEnabled" ),
      default    : Some( "false" ),
    },
    ParamDef
    {
      name       : "wheel_scroll_accel",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "wheelScrollAccelerationEnabled" ),
      default    : Some( "false" ),
    },
    ParamDef
    {
      name       : "workspace_id",
      cli_flag   : None,
      env_var    : Some( "ANTHROPIC_WORKSPACE_ID" ),
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "worktree",
      cli_flag   : Some( "-w / --worktree" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "worktree_bg_isolation",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "worktree.bgIsolation" ),
      default    : Some( "false" ),
    },
  ];
  ENTRIES
}
