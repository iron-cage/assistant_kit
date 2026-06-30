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
      name       : "add_dir",
      cli_flag   : Some( "--add-dir" ),
      env_var    : None,
      config_key : None,
      default    : None,
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
      name       : "auto_updates",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "autoUpdates" ),
      default    : Some( "true" ),
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
      name       : "disable_autoupdater",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "env.DISABLE_AUTOUPDATER" ),
      default    : None,
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
      name       : "disallowed_tools",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "disallowedTools" ),
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
      name       : "ide_hint",
      cli_flag   : None,
      env_var    : Some( "CLAUDE_CODE_IDE_HINT" ),
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
      name       : "mcp_server",
      cli_flag   : Some( "--mcp-server" ),
      env_var    : None,
      config_key : None,
      default    : None,
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
      name       : "model",
      cli_flag   : Some( "--model" ),
      env_var    : Some( "CLAUDE_MODEL" ),
      config_key : Some( "model" ),
      default    : Some( "claude-sonnet-4-6" ),
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
      name       : "output_format",
      cli_flag   : Some( "--output-format" ),
      env_var    : None,
      config_key : None,
      default    : None,
    },
    ParamDef
    {
      name       : "permission_mode",
      cli_flag   : Some( "--permission-mode" ),
      env_var    : None,
      config_key : None,
      default    : None,
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
      name       : "quiet",
      cli_flag   : Some( "--quiet" ),
      env_var    : None,
      config_key : None,
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
      name       : "skip_permissions",
      cli_flag   : Some( "--dangerously-skip-permissions" ),
      env_var    : None,
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
      name       : "theme",
      cli_flag   : None,
      env_var    : None,
      config_key : Some( "theme" ),
      default    : Some( "system" ),
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
  ];
  ENTRIES
}
