//! Settings catalog: known Claude Code configuration keys with their env var
//! mappings and catalog default values.
//!
//! The catalog is the authoritative registry for 4-layer config resolution.
//! See `docs/algorithm/002_config_resolution.md` § Catalog for the canonical
//! entry list and per-key semantics.

/// A known Claude Code setting definition.
///
/// Used by the 4-layer resolution engine (`config_resolve`) to locate env var
/// overrides and to supply catalog defaults when no other layer has a value.
#[ derive( Debug ) ]
pub struct SettingDef
{
  /// The settings key name (e.g., `"model"`, `"theme"`).
  pub key     : &'static str,
  /// Optional environment variable that overrides this key (e.g., `"ANTHROPIC_MODEL"`).
  pub env_var : Option< &'static str >,
  /// Optional catalog default value string (raw, as produced by `read_all_settings`).
  pub default : Option< &'static str >,
}

/// Return a static slice of all known Claude Code setting definitions.
///
/// Catalog entries per `docs/algorithm/002_config_resolution.md` § Catalog.
#[ inline ]
#[ must_use ]
pub fn catalog() -> &'static [ SettingDef ]
{
  static ENTRIES : &[ SettingDef ] = &[
    // Fix(BUG-579): `model`'s env var is `ANTHROPIC_MODEL`, the one Claude Code reads.
    // Root cause: the entry named `CLAUDE_MODEL`, which the Claude Code binary never
    // reads, so `.config` showed a dead override as `(env)` and ignored the live one.
    // Pitfall: an env name on another program's surface must be checked against that
    // program (binary strings, a captured request), not against tests that set it.
    SettingDef { key : "model",                   env_var : Some( "ANTHROPIC_MODEL" ), default : Some( "claude-sonnet-5" ) },
    SettingDef { key : "preferredVersionSpec",     env_var : None,                     default : None                       },
    SettingDef { key : "preferredVersionResolved", env_var : None,                     default : None                       },
    SettingDef { key : "autoUpdates",              env_var : None,                     default : Some( "true" )             },
    SettingDef { key : "theme",                    env_var : None,                     default : Some( "dark" )             },
    SettingDef { key : "hasCompletedOnboarding",   env_var : None,                     default : Some( "false" )            },
    SettingDef { key : "env.DISABLE_AUTOUPDATER",  env_var : None,                     default : None                       },
    SettingDef { key : "autoUpdatesChannel",       env_var : None,                     default : Some( "latest" )           },
    SettingDef { key : "minimumVersion",           env_var : None,                     default : None                       },
    SettingDef { key : "env.DISABLE_UPDATES",      env_var : Some( "DISABLE_UPDATES" ), default : None                       },
  ];
  ENTRIES
}
