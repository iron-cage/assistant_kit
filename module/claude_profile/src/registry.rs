//! Command registration: argument definitions and routines for the `claude_profile` CLI.

use unilang::data::Kind;
use crate::commands::
{
  credentials_status_routine,
  accounts_routine,
  account_limits_routine,
  account_save_routine,
  account_use_routine,
  account_delete_routine,
  account_relogin_routine,
  account_renewal_routine,
  account_rotate_routine,
  account_inspect_routine,
  account_assign_routine,
  model_routine,
  token_status_routine,
  paths_routine,
  usage_routine,
};

/// Register all `claude_profile` commands into an existing registry.
///
/// Registers 15 commands (credentials status, account management including limits, relogin, rotate, renewal, inspect, and assign, model get/set, token status, paths, usage).
/// The `.` (dot) hidden command and `.help` are binary-specific — they are NOT
/// included here.
///
/// # Panics
///
/// Panics if a command fails to register (duplicate name = programming error).
#[ allow( clippy::too_many_lines ) ]
#[ inline ]
pub fn register_commands( registry : &mut unilang::registry::CommandRegistry )
{
  // Fix(BUG-203): convenience closures must chain `.with_description()` so that
  // per-command `.help` output shows meaningful param descriptions.
  // Root cause: bare `reg_arg_opt()` emits a blank description line.
  // Pitfall: `.with_description()` is not enforced by the type system — only tests catch the omission.
  let fmt = || reg_arg_opt( "format",    Kind::String  ).with_description( "Output format: text (default) or json" );
  let dry = || reg_arg_opt( "dry",       Kind::Boolean ).with_description( "Dry run mode (0 = off, default; 1 = on)" );
  let nam = || reg_arg_opt( "name",      Kind::String  ).with_description( "Account name to operate on" );
  let thr = || reg_arg_opt( "threshold", Kind::Integer ).with_description( "Token expiry warning threshold in seconds (default 3600)" );
  let bfd = | nm : &'static str, desc : &'static str |
    reg_arg_opt( nm, Kind::Boolean ).with_description( desc );
  // Strict opt-in flags: only "0" or "1" accepted (not "yes"/"no"/"true").
  let bfs = | nm : &'static str, desc : &'static str |
    reg_arg_opt( nm, Kind::String ).with_description( desc );
  let trc = || reg_arg_opt( "trace", Kind::Integer ).with_description( "Print [trace] lines to stderr for each file read and write step (0 = off, default; 1 = on)" );

  reg_cmd( registry, ".credentials.status", "Show live credential metadata without account store dependency",
    vec![
      reg_arg_opt( "format", Kind::String ).with_description( "Output format: `text` (default) or `json`" ),
      bfd( "account", "Show account name from per-machine active marker (default on)"   ),
      bfd( "sub",     "Show subscription type from credentials (default on)"    ),
      bfd( "tier",    "Show rate-limit tier from credentials (default on)"      ),
      bfd( "token",   "Show OAuth token validity state (default on)"            ),
      bfd( "expires", "Show token expiry time (default on)"                     ),
      bfd( "email",   "Show email address from `~/.claude.json` (default on)"    ),
      bfd( "file",         "Show path to `.credentials.json` file (opt-in)"                         ),
      bfd( "saved",        "Show count of saved accounts in credential store (opt-in)"               ),
      bfd( "display_name", "Show display name from `~/.claude.json` oauthAccount (opt-in)"           ),
      bfd( "role",         "Show organisation role from `~/.claude.json` oauthAccount (opt-in)"      ),
      bfd( "billing",      "Show billing type from `~/.claude.json` oauthAccount (opt-in)"           ),
      bfd( "model",        "Show active model from `~/.claude/settings.json` (opt-in)"               ),
      bfs( "uuid",         "Show stable user identifier (`taggedId`) from `~/.claude.json` (opt-in)"          ),
      bfs( "capabilities", "Show enabled capabilities list from `~/.claude.json` (opt-in)"                    ),
      bfs( "org_uuid",     "Show organisation UUID from active account's `{name}.json` snapshot (opt-in)"       ),
      bfs( "org_name",     "Show organisation display name from active account's `{name}.json` snapshot (opt-in)" ),
      reg_arg_opt( "get", Kind::String ).with_description( "Extract bare field value for scripting: `subscription`, `tier`, `token`, `expires_in_secs`, `email`, `account`, `file`" ),
      trc(),
    ],
    Box::new( credentials_status_routine ) );
  reg_cmd( registry, ".accounts",       "List all saved accounts with field-presence control",
    vec![
      nam(),
      bfd( "active",       "Show active/inactive status per account (default on)"                                   ),
      bfd( "current",      "Show current (live) session match per account (default on)"                             ),
      bfd( "sub",          "Show subscription type per account (default on)"                                        ),
      bfd( "tier",         "Show rate-limit tier per account (default on)"                                          ),
      bfd( "expires",      "Show token expiry duration per account (default on)"                                    ),
      bfd( "email",        "Show email address per account (default on)"                                            ),
      bfd( "display_name", "Show display name from saved `{name}.json` snapshot (opt-in)"                              ),
      bfs( "host",         "Show machine host label from saved `{name}.json` snapshot (opt-in)"                          ),
      bfd( "role",         "Show user-defined role label from saved `{name}.json` snapshot (opt-in)"                     ),
      bfd( "billing",      "Show billing type from saved `{name}.json` snapshot (opt-in)"                                ),
      bfd( "model",        "Show active model from saved `{name}.json` snapshot (opt-in)"                                ),
      bfs( "uuid",         "Show stable user identifier (`taggedId`) from saved `{name}.json` snapshot (opt-in)"         ),
      bfs( "capabilities", "Show enabled capabilities list from saved `{name}.json` snapshot (opt-in)"                   ),
      bfs( "org_uuid",     "Show organisation UUID from saved `{name}.json` snapshot (opt-in)"                           ),
      bfs( "org_name",     "Show organisation display name from saved `{name}.json` snapshot (opt-in)"                   ),
      trc(),
      fmt(),
    ],
    Box::new( accounts_routine ) );
  reg_cmd( registry, ".account.limits", "Show rate-limit utilization for the selected account (FR-18)", vec![ nam(), fmt(), trc() ], Box::new( account_limits_routine ) );
  reg_cmd( registry, ".account.save", "Save current credentials as a named account profile",
    vec![
      nam(),
      dry(),
      trc(),
      reg_arg_opt( "host", Kind::String ).with_description( "Machine label for this account (default: auto-capture `$USER@$HOSTNAME`); written to `{name}.json`" ),
      reg_arg_opt( "role", Kind::String ).with_description( "User-defined role tag (e.g. `work`, `personal`); written to `{name}.json`" ),
    ],
    Box::new( account_save_routine    ) );
  // Registered inline (not via reg_cmd) to add per-command examples — required by feature 015
  // AC-10 (help shows positional shortcut syntax).
  {
    let def = unilang::data::CommandDefinition::former()
    .name( ".account.use" )
    .description( "Switch active account by name with atomic credential rotation" )
    .arguments( vec!
    [
      reg_arg_req( "name", Kind::String ).with_description( "Account name (positional: alice@acme.com; or keyword: name::alice@acme.com)" ),
      dry(),
      reg_arg_opt( "touch",   Kind::String ).with_description( "Activate idle 5h session window via subprocess after switch (0/false = off; 1/true = on, default)" ),
      reg_arg_opt( "refresh", Kind::String ).with_description( "Attempt OAuth token refresh when stored credentials are locally expired (1 = enabled, default; 0 = disabled)" ),
      reg_arg_opt( "imodel",    Kind::String ).with_description( "Subprocess model: `auto` (default, haiku — sufficient for keep-alive), `sonnet`, `opus`, `haiku` (claude-haiku-4-5-20251001), `keep`" ),
      reg_arg_opt( "effort",    Kind::String ).with_description( "Subprocess effort level: `auto` (default, low for any model), `low`, `normal`, `high`, `max`" ),
      reg_arg_opt( "set_model", Kind::String ).with_description( "Set Claude Code session model: `opus` (claude-opus-4-6), `sonnet` (claude-sonnet-4-6), `haiku` (claude-haiku-4-5-20251001), `default` (removes override)" ),
      reg_arg_opt( "trace",     Kind::String ).with_description( "Print [trace] lines to stderr for each internal operation (0 = off, default; 1 = on)" ),
    ] )
    .examples( vec![ "clp .account.use alice@acme.com".to_string() ] )
    .end();
    registry
    .register_with_routine( &def, Box::new( account_use_routine ) )
    .expect( "internal error: failed to register .account.use" );
  }
  reg_cmd( registry, ".account.delete", "Delete a saved account from the account store",                                   vec![ reg_arg_req( "name", Kind::String ).with_description( "Account name to operate on" ), dry(), trc() ], Box::new( account_delete_routine  ) );
  reg_cmd( registry, ".account.relogin", "Force browser re-authentication for a named account with dead refreshToken",     vec![ nam(), dry(), trc() ], Box::new( account_relogin_routine ) );
  reg_cmd( registry, ".account.renewal", "Set or clear a billing renewal timestamp override for one or more accounts",
    vec![
      reg_arg_req( "name",     Kind::String ).with_description( "Account name, `all`, or comma-separated list of accounts" ),
      reg_arg_opt( "at",       Kind::String ).with_description( "Set exact renewal timestamp (ISO-8601 UTC, e.g. 2026-06-29T21:00:00Z); mutually exclusive with from_now:: and clear::" ),
      reg_arg_opt( "from_now", Kind::String ).with_description( "Set renewal relative to now (e.g. +1h30m, -30m, +0m); mutually exclusive with at:: and clear::" ),
      bfs( "clear", "Remove the renewal override (restores ~-prefixed estimate in .usage); mutually exclusive with at:: and from_now::" ),
      dry(),
      trc(),
    ],
    Box::new( account_renewal_routine ) );
  reg_cmd( registry, ".account.rotate", "Auto-rotate to the best inactive account (highest remaining token expiry)",       vec![ dry(), trc() ], Box::new( account_rotate_routine ) );
  reg_cmd( registry, ".account.inspect", "Show identity, subscription, and org fields for one account via live endpoints",
    vec![
      nam(),
      bfs( "refresh", "Attempt OAuth token refresh when stored credentials are locally expired (1 = enabled, default; 0 = disabled)" ),
      trc(),
      fmt(),
    ],
    Box::new( account_inspect_routine ) );
  reg_cmd( registry, ".account.assign", "Write the per-machine active-account marker for any host+user pair without credential rotation",
    vec![
      nam(),
      reg_arg_opt( "for", Kind::String ).with_description( "Target identity as USER@MACHINE (default: current $USER@hostname); split on first '@'; both parts sanitized (alphanumeric, '-', '.' kept)" ),
      dry(),
      trc(),
    ],
    Box::new( account_assign_routine ) );
  reg_cmd( registry, ".model", "Get or set the Claude Code session model in ~/.claude/settings.json",
    vec![
      reg_arg_opt( "set", Kind::String ).with_description( "Set model: `opus` (claude-opus-4-6), `sonnet` (claude-sonnet-4-6), `haiku` (claude-haiku-4-5-20251001), `default` (removes override)" ),
      fmt(),
    ],
    Box::new( model_routine ) );
  reg_cmd( registry, ".token.status",   "Show active OAuth token expiry classification",                  vec![ fmt(), thr(), trc() ], Box::new( token_status_routine   ) );
  reg_cmd( registry, ".paths",          "Show all resolved ~/.claude/ canonical file paths",
    vec![
      fmt(),
      reg_arg_opt( "field", Kind::String ).with_description( "Output a single named path value; format:: is ignored when set. Valid: base, credentials, credential_store, projects, stats, settings, session_env, sessions" ),
      trc(),
    ],
    Box::new( paths_routine ) );
  reg_cmd( registry, ".usage",          "Show live rate-limit quota for all saved accounts",
    vec![
      reg_arg_opt( "format", Kind::String ).with_description( "Output format: `text` (default), `json`, `tsv` (tab-separated, plain status), `plain` (no emoji), `value` (bare, use with `get::`)" ),
      reg_arg_opt( "refresh",   Kind::Integer ).with_description( "Retry once on 401/403 (auth errors) or 429 when token is locally expired, via isolated subprocess (1 = enabled, default; 0 = disabled)" ),
      reg_arg_opt( "live",      Kind::Integer ).with_description( "Continuous monitor mode (0 = off, default; 1 = on)" ),
      reg_arg_opt( "interval",  Kind::Integer ).with_description( "Seconds between refreshes (minimum 30, default 30)" ),
      reg_arg_opt( "jitter",    Kind::Integer ).with_description( "Max random seconds added to interval (0 = none, default)" ),
      reg_arg_opt( "trace",     Kind::Integer ).with_description( "Print [trace] lines to stderr showing each credential read, API call, and refresh step (0 = off; 1 = on)" ),
      reg_arg_opt( "sort",      Kind::String  ).with_description( "Row ordering strategy: `renew` (default), `name`, `endurance`, `drain`, `next` (mirrors active next:: strategy)" ),
      reg_arg_opt( "desc",      Kind::Integer ).with_description( "Sort direction: 0 = ascending (strategy default for name/drain/renew), 1 = descending (strategy default for endurance)" ),
      reg_arg_opt( "prefer",    Kind::String  ).with_description( "Weekly quota column for strategies: `any` (default, min of both), `opus` (7d Left), `sonnet` (7d(Son))" ),
      reg_arg_opt( "next",      Kind::String  ).with_description( "Recommendation strategy: `renew` (default, soonest reset timer), `endurance` (most quota left), `drain` (least quota left)" ),
      reg_arg_opt( "cols",      Kind::String  ).with_description( "Column visibility modifiers (comma-separated `+col_id`/`-col_id`); default shows all except `sub` and `7d_son_reset`" ),
      reg_arg_opt( "touch",             Kind::String  ).with_description( "Extend active 5h session windows via isolated subprocess for accounts with an active reset countdown (0/false = off; 1/true = on, default)" ),
      reg_arg_opt( "imodel",            Kind::String  ).with_description( "Subprocess model for touch/refresh: `auto` (default, haiku — sufficient for keep-alive), `sonnet` (claude-sonnet-4-6), `opus` (claude-opus-4-6), `haiku` (claude-haiku-4-5-20251001), `keep` (no --model flag)" ),
      reg_arg_opt( "effort",            Kind::String  ).with_description( "Subprocess effort level: `auto` (default, low for any model), `low` (always --effort low), `normal` (always --effort normal), `high` (always --effort high), `max` (always --effort max)" ),
      // Row filtering parameters (TSK-223)
      reg_arg_opt( "count",             Kind::Integer ).with_description( "Max rows to display; 0 = show all (default)" ),
      reg_arg_opt( "offset",            Kind::Integer ).with_description( "Skip first N rows from the filtered result before display (default 0)" ),
      reg_arg_opt( "only_active",       Kind::String  ).with_description( "Show only the per-machine active account row (0 = off, default; 1 = on)" ),
      reg_arg_opt( "only_next",         Kind::String  ).with_description( "Show only the row receiving the `→` recommendation marker (0 = off, default; 1 = on)" ),
      reg_arg_opt( "min_5h",            Kind::Integer ).with_description( "Hide rows where 5h Left is below this percentage 0–100 (default 0 = no filter); rows with no quota also hidden" ),
      reg_arg_opt( "min_7d",            Kind::Integer ).with_description( "Hide rows where 7d Left is below this percentage 0–100 (default 0 = no filter); rows with no quota also hidden" ),
      reg_arg_opt( "only_valid",        Kind::String  ).with_description( "Hide 🔴 rows (invalid/expired token) (0 = off, default; 1 = on)" ),
      reg_arg_opt( "exclude_exhausted", Kind::String  ).with_description( "Hide 🟡 and 🔴 rows; show only 🟢 rows (0 = off, default; 1 = on)" ),
      // Extraction and display (TSK-224)
      reg_arg_opt( "get",       Kind::String  ).with_description( "Extract bare field value for first row: `5h_left`, `5h_reset`, `7d_left`, `7d_son`, `7d_reset`, `expires`, `renews`, `sub`, `status`, `account`, `host`, `role`, `next_event_type`, `next_event_secs`" ),
      reg_arg_opt( "abs",       Kind::String  ).with_description( "Replace percentage columns with absolute token counts where available (0 = off, default; 1 = on)" ),
      reg_arg_opt( "no_color",  Kind::String  ).with_description( "Strip emoji and ANSI sequences; status shows `ok`/`warn`/`err` (0 = off, default; 1 = on)" ),
      reg_arg_opt( "set_model", Kind::String  ).with_description( "Set Claude Code session model: `opus` (claude-opus-4-6), `sonnet` (claude-sonnet-4-6), `haiku` (claude-haiku-4-5-20251001), `default` (removes override)" ),
    ],
    Box::new( usage_routine          ) );
}

fn reg_arg_opt( name : &str, kind : unilang::data::Kind ) -> unilang::data::ArgumentDefinition
{
  unilang::data::ArgumentDefinition::new( name, kind ).with_optional( None::< String > )
}

// Fix(BUG-204): required-parameter registration helper.
// Root cause: `reg_arg_opt` unconditionally sets `optional: true`; commands like `.account.use`
// enforce `name` as required at runtime but the help system showed `optional`.
// Pitfall: `ArgumentDefinition::new()` defaults to `optional: false` — do NOT chain `.with_optional()`.
fn reg_arg_req( name : &str, kind : unilang::data::Kind ) -> unilang::data::ArgumentDefinition
{
  unilang::data::ArgumentDefinition::new( name, kind )
}

fn reg_cmd(
  registry : &mut unilang::registry::CommandRegistry,
  name     : &str,
  desc     : &str,
  args     : Vec< unilang::data::ArgumentDefinition >,
  routine  : unilang::registry::CommandRoutine,
)
{
  let def = unilang::data::CommandDefinition::former()
  .name( name )
  .description( desc )
  .arguments( args )
  .end();
  registry
  .register_with_routine( &def, routine )
  .expect( "internal error: failed to register command" );
}
