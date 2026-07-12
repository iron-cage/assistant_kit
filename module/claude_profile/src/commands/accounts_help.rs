//! Grouped, `::`-aligned help rendering for `.accounts`.

use cli_fmt::help::{ CliHelpData, CliHelpStyle, CliHelpTemplate, OptionEntry, OptionGroup };

/// One `.accounts` parameter's group membership, value placeholder, and description.
struct ParamSpec
{
  name  : &'static str,
  group : &'static str,
  value : &'static str,
  desc  : &'static str,
}

/// The 6 documented presentation groups, in display order.
const GROUP_ORDER : [ &str ; 6 ] =
[
  "Core",
  "Account Ownership",
  "Sort Control",
  "Row Filtering & Pagination",
  "Display Rendering",
  "Refresh & Subprocess Control",
];

/// All 30 active `.accounts` parameters, per
/// `docs/cli/command/001_account.md § Help Rendering Scheme`.
const PARAMS : [ ParamSpec ; 30 ] =
[
  ParamSpec { name : "name",              group : "Core",                          value : "EMAIL",    desc : "Account identifier or prefix (optional)" },
  ParamSpec { name : "format",            group : "Core",                          value : "FORMAT",   desc : "Output serialization format" },
  ParamSpec { name : "dry",               group : "Core",                          value : "0",        desc : "Preview mutation without writing" },

  ParamSpec { name : "owner",             group : "Account Ownership",             value : "OWNER",    desc : "Set or release account ownership" },
  ParamSpec { name : "assignee",          group : "Account Ownership",             value : "ASSIGNEE", desc : "Write per-machine active-account marker" },
  ParamSpec { name : "force",             group : "Account Ownership",             value : "0",        desc : "Bypass G8 ownership gate" },

  ParamSpec { name : "sort",              group : "Sort Control",                  value : "SORT",     desc : "Row ordering strategy" },
  ParamSpec { name : "desc",              group : "Sort Control",                  value : "0",        desc : "Reverse sort direction" },
  ParamSpec { name : "prefer",            group : "Sort Control",                  value : "PREFER",   desc : "Tiebreaker sort strategy" },

  ParamSpec { name : "cols",              group : "Row Filtering & Pagination",    value : "COLS",     desc : "Column visibility modifiers" },
  ParamSpec { name : "count",             group : "Row Filtering & Pagination",    value : "N",        desc : "Limit row count after filtering" },
  ParamSpec { name : "offset",            group : "Row Filtering & Pagination",    value : "N",        desc : "Skip first N rows" },
  ParamSpec { name : "only_active",       group : "Row Filtering & Pagination",    value : "0",        desc : "Keep only active account row" },
  ParamSpec { name : "only_next",         group : "Row Filtering & Pagination",    value : "0",        desc : "Keep only recommended next account row" },
  ParamSpec { name : "only_valid",        group : "Row Filtering & Pagination",    value : "0",        desc : "Keep non-exhausted non-expired rows" },
  ParamSpec { name : "exclude_exhausted", group : "Row Filtering & Pagination",    value : "0",        desc : "Remove exhausted rows" },
  ParamSpec { name : "min_5h",            group : "Row Filtering & Pagination",    value : "N",        desc : "Keep rows with 5h quota >= N%" },
  ParamSpec { name : "min_7d",            group : "Row Filtering & Pagination",    value : "N",        desc : "Keep rows with 7d quota >= N%" },

  ParamSpec { name : "abs",               group : "Display Rendering",             value : "0",        desc : "Show absolute token counts" },
  ParamSpec { name : "no_color",          group : "Display Rendering",             value : "0",        desc : "Strip emoji and ANSI sequences" },
  ParamSpec { name : "get",               group : "Display Rendering",             value : "FIELD",    desc : "Extract bare field value from first row" },

  ParamSpec { name : "trace",             group : "Refresh & Subprocess Control",  value : "0",        desc : "Diagnostic trace output" },
  ParamSpec { name : "refresh",           group : "Refresh & Subprocess Control",  value : "0",        desc : "Force token refresh" },
  ParamSpec { name : "touch",             group : "Refresh & Subprocess Control",  value : "0",        desc : "Activate idle 5h session window" },
  ParamSpec { name : "imodel",            group : "Refresh & Subprocess Control",  value : "MODEL",    desc : "Model for post-switch subprocess" },
  ParamSpec { name : "effort",            group : "Refresh & Subprocess Control",  value : "EFFORT",   desc : "Effort for post-switch subprocess" },
  ParamSpec { name : "set_model",         group : "Refresh & Subprocess Control",  value : "MODEL",    desc : "Write session model after operation" },
  ParamSpec { name : "live",              group : "Refresh & Subprocess Control",  value : "0",        desc : "Continuous monitor mode" },
  ParamSpec { name : "interval",          group : "Refresh & Subprocess Control",  value : "N",        desc : "Seconds between live refresh cycles" },
  ParamSpec { name : "jitter",            group : "Refresh & Subprocess Control",  value : "N",        desc : "Random jitter added to interval" },
];

/// Print the grouped, `::`-aligned `.accounts.help` text to stdout.
///
/// Bypasses unilang's automatic per-command help — builds `cli_fmt::CliHelpData`
/// directly from `PARAMS` so the `::` delimiter aligns at the same column across
/// all 30 rows, spanning group boundaries (per
/// `docs/cli/command/001_account.md § Help Rendering Scheme`).
#[ inline ]
pub fn print_accounts_help( binary : &str )
{
  let width = PARAMS.iter().map( | p | p.name.len() ).max().unwrap_or( 0 );

  let mut data = CliHelpData::default();
  data.binary  = binary.to_string();
  data.tagline = "List all saved accounts with identity column control. \
    Boolean parameters are shown bare and accept 0 (off, default) or 1 (on).".to_string();
  data.option_groups = GROUP_ORDER.iter().map( | &group_name |
  {
    let entries : Vec< OptionEntry > = PARAMS.iter()
    .filter( | p | p.group == group_name )
    .map( | p | OptionEntry
    {
      name : format!( "{:<width$}::{}", p.name, p.value ),
      desc : p.desc.to_string(),
    } )
    .collect();
    OptionGroup { name : group_name.to_string(), entries }
  } ).collect();

  print!( "{}", CliHelpTemplate::new( CliHelpStyle::default(), data ).render() );
}
