//! Command registration: argument definitions and routines for the `claude_profile` CLI.

use unilang::data::Kind;
use crate::commands::
{
  credentials_status_routine,
  accounts_view_routine,
  account_limits_routine,
  account_save_routine,
  account_use_routine,
  account_delete_routine,
  account_relogin_routine,
  account_renewal_routine,
  account_inspect_routine,
  account_tag_routine,
  tags_routine,
  identities_routine,
  identity_filter_routine,
  model_routine,
  models_routine,
  model_select_routine,
  provider_select_routine,
  paths_routine,
};

/// Register all `claude_profile` commands into an existing registry.
///
/// Registers 18 commands (credentials status, account management including limits, relogin, renewal, inspect, and tag, model get/set/select, models discovery, provider select, paths, usage, tags/identities listings, identity filter).
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
  let trc = || reg_arg_opt( "trace", Kind::Integer ).with_description( "Print timestamped diagnostic lines to stderr for each file read and write step (0 = off, default; 1 = on)" );

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
      thr(),
      trc(),
    ],
    Box::new( credentials_status_routine ) );
  reg_cmd( registry, ".accounts",       "List all saved accounts with identity column control",
    vec![
      nam(),
      dry(),
      trc(),
      fmt(),
      // Mutation params
      bfd( "assign",  "REMOVED — use assignee::USER@MACHINE name::X (or assignee::0 name::X for current machine)" ),
      bfd( "unclaim", "REMOVED — use owner::0 name::X instead (or owner::0 alone to batch-clear)" ),
      reg_arg_opt( "owner", Kind::String ).with_description( "Set or clear account ownership: USER@MACHINE identity to set; sentinel value \"0\" clears ownership (owner::0)" ),
      bfs( "force",   "Bypass G8 ownership gate on owner:: (default 0)" ),
      bfs( "lock",    "Set (\"1\") or clear (\"0\") claim-lock: excludes from unattended rotation and explicit-switch targets; ungated (Feature 070)" ),
      bfs( "reserve", "Set (\"1\") or clear (\"0\") reserve marker: deprioritizes (does not exclude) in sort-based selection; ungated (Feature 070)" ),
      reg_arg_opt( "cols", Kind::String ).with_description( "Column visibility modifiers (comma-separated `+col_id`/`-col_id`); default set: account, owner, active, current, sub, tier, expires, email, inference_provider" ),
      reg_arg_opt( "tags", Kind::String ).with_description( "Subset filter: list only accounts carrying ALL listed tags (comma-separated); Feature 075" ),
      bfs( "for",     "REMOVED — use assignee::USER@MACHINE name::X (or assignee::0 name::X for current machine)" ),
      bfs( "active",  "REMOVED — use assignee::USER@MACHINE name::X (or assignee::0 name::X for current machine)" ),
      reg_arg_opt( "assignee", Kind::String ).with_description( "USER@MACHINE (or sentinel \"0\" = $USER@$HOSTNAME) assign/unassign active-account marker; Feature 065" ),
      // Unified display/query params (same set as .usage; defaults differ)
      reg_arg_opt( "refresh",           Kind::Integer ).with_description( "Attempt OAuth token refresh for expired credentials via subprocess (0 = off, default; 1 = enabled)" ),
      reg_arg_opt( "touch",             Kind::String  ).with_description( "Extend active 5h session windows via subprocess (0/false = off, default; 1/true = on)" ),
      reg_arg_opt( "imodel",            Kind::String  ).with_description( "Subprocess model for touch/refresh: `auto` (default), `sonnet`, `opus`, `haiku`, `keep`" ),
      reg_arg_opt( "effort",            Kind::String  ).with_description( "Subprocess effort level: `auto` (default), `low`, `normal`, `high`, `max`" ),
      reg_arg_opt( "sort",              Kind::String  ).with_description( "Row ordering strategy: `name` (default), `renew`, `renews`" ),
      reg_arg_opt( "desc",              Kind::Integer ).with_description( "Sort direction: 0 = ascending (default), 1 = descending" ),
      reg_arg_opt( "prefer",            Kind::String  ).with_description( "Weekly quota column preference for strategies: `any` (default), `opus`, `sonnet`" ),
      reg_arg_opt( "next",              Kind::String  ).with_description( "REMOVED — use sort:: instead; kept for migration error" ),
      reg_arg_opt( "count",             Kind::Integer ).with_description( "Max rows to display; 0 = show all (default)" ),
      reg_arg_opt( "offset",            Kind::Integer ).with_description( "Skip first N rows before display (default 0)" ),
      reg_arg_opt( "only_active",       Kind::String  ).with_description( "Show only the per-machine active account (0 = off, default; 1 = on)" ),
      reg_arg_opt( "only_next",         Kind::String  ).with_description( "Show only the recommended next account (0 = off, default; 1 = on)" ),
      reg_arg_opt( "min_5h",            Kind::Integer ).with_description( "Hide rows where 5h Left is below this percentage 0–100 (default 0 = no filter)" ),
      reg_arg_opt( "min_7d",            Kind::Integer ).with_description( "Hide rows where 7d Left is below this percentage 0–100 (default 0 = no filter)" ),
      reg_arg_opt( "only_valid",        Kind::String  ).with_description( "Hide expired/invalid token rows (0 = off, default; 1 = on)" ),
      reg_arg_opt( "exclude_exhausted", Kind::String  ).with_description( "Hide exhausted rows; show only accounts with quota (0 = off, default; 1 = on)" ),
      reg_arg_opt( "get",               Kind::String  ).with_description( "Extract bare field value for first row for scripting" ),
      reg_arg_opt( "no_color",          Kind::String  ).with_description( "Strip emoji and ANSI sequences (0 = off, default; 1 = on)" ),
      reg_arg_opt( "set_model",         Kind::String  ).with_description( "Set Claude Code session model: `opus`, `sonnet`, `haiku`, `default`" ),
      reg_arg_opt( "live",              Kind::Integer ).with_description( "Continuous monitor mode (0 = off, default; 1 = on)" ),
      reg_arg_opt( "interval",          Kind::Integer ).with_description( "Seconds between live refreshes (minimum 30, default 30)" ),
      reg_arg_opt( "jitter",            Kind::Integer ).with_description( "Max random seconds added to interval (0 = none, default)" ),
      // Shared with .usage (same command group, same handler — CV020/CV006).
      reg_arg_opt( "rotate",            Kind::Integer ).with_description( "Switch to the → winner after rendering (0 = off, default; 1 = on); .usage only" ),
      reg_arg_opt( "who",               Kind::Integer ).with_description( "Sessions table visibility: auto (default), 0 = suppress, 1 = force on; .usage only" ),
      reg_arg_opt( "solo",              Kind::Integer ).with_description( "Token conservation: restrict fetch to current+owned account only (0 = off, default; 1 = on); .usage only" ),
      reg_arg_opt( "alert",             Kind::Integer ).with_description( "Burn-rate alert horizon in minutes (default 15; 0 = off); .usage only" ),
      // Legacy field-toggle params (removed by Feature 037; kept registered so the routine
      // can emit a helpful cols:: migration message instead of a generic framework error).
      bfd( "current",      "REMOVED — use cols::-current instead"      ),
      bfd( "sub",          "REMOVED — use cols::-sub instead"          ),
      bfd( "tier",         "REMOVED — use cols::-tier instead"         ),
      bfd( "expires",      "REMOVED — use cols::-expires instead"      ),
      bfd( "email",        "REMOVED — use cols::-email instead"        ),
      bfd( "display_name", "REMOVED — use cols::+display_name instead" ),
      bfs( "host",         "REMOVED — use cols::+host instead"         ),
      bfd( "role",         "REMOVED — use cols::+role instead"         ),
      bfd( "billing",      "REMOVED — use cols::+billing instead"      ),
      bfd( "model",        "REMOVED — use cols::+model instead"        ),
      bfs( "uuid",         "REMOVED — use cols::+uuid instead"         ),
      bfs( "capabilities", "REMOVED — use cols::+capabilities instead" ),
      bfs( "org_uuid",     "REMOVED — use cols::+org_uuid instead"     ),
      bfs( "org_name",     "REMOVED — use cols::+org_name instead"     ),
    ],
    Box::new( accounts_view_routine ) );
  reg_cmd( registry, ".account.limits", "Show rate-limit utilization for the selected account (FR-18)", vec![ nam(), fmt(), trc() ], Box::new( account_limits_routine ) );
  reg_cmd( registry, ".account.save", "Save current credentials as a named account profile",
    vec![
      nam(),
      dry(),
      trc(),
      reg_arg_opt( "host",    Kind::String  ).with_description( "Machine label for this account (default: auto-capture `$USER@$HOSTNAME`); written to `{name}.json`" ),
      reg_arg_opt( "role",    Kind::String  ).with_description( "REMOVED — use tags:: instead (a role value is now just a tag, e.g. tags::work)" ),
      reg_arg_opt( "tags",    Kind::String  ).with_description( "Comma-separated tags replacing the whole stored set (e.g. tags::work,kimi_pool); each tag 1-64 chars from [a-z0-9_-]; Feature 075" ),
      reg_arg_opt( "backend",        Kind::String ).with_description( "Backend for the new account: `anthropic` (default) or `redirect` (case-insensitive); see docs/cli/param/069_backend.md" ),
      reg_arg_opt( "preset",         Kind::String ).with_description( "Named provider preset pre-filling backend::/base_url::/inference_provider:: for a known foreign provider; explicit params always override the preset default. Only `kimi` is recognized today; see docs/cli/param/074_preset.md" ),
      reg_arg_opt( "base_url",       Kind::String ).with_description( "Redirect target's API base URL; required with backend::redirect, rejected otherwise; written to `{name}.json` and env.ANTHROPIC_BASE_URL on use" ),
      reg_arg_opt( "api_key",        Kind::String ).with_description( "Static API key for a redirect-backend account; required with backend::redirect, rejected otherwise; written to `{name}.credentials.json`" ),
      reg_arg_opt( "redirect_model", Kind::String ).with_description( "Foreign provider's model identifier; required with backend::redirect, rejected otherwise; written to `{name}.json` and env.ANTHROPIC_MODEL on use" ),
      reg_arg_opt( "inference_provider", Kind::String ).with_description( "Inference provider tag for this account (e.g. `kimi`, `moonshot`); written to `{name}.json`" ),
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
      reg_arg_opt( "set_model", Kind::String ).with_description( "Set Claude Code session model: `opus` (claude-opus-4-8), `sonnet` (claude-sonnet-5), `haiku` (claude-haiku-4-5-20251001), `default` (removes override)" ),
      reg_arg_opt( "trace",     Kind::String ).with_description( "Print timestamped diagnostic lines to stderr for each internal operation (0 = off, default; 1 = on)" ),
      bfs( "force", "Bypass G5 ownership gate; allow switching to a non-owned account" ),
    ] )
    .examples( vec![ "clp .account.use alice@acme.com".to_string() ] )
    .end();
    registry
    .register_with_routine( &def, Box::new( account_use_routine ) )
    .expect( "internal error: failed to register .account.use" );
  }
  reg_cmd( registry, ".account.delete", "Delete a saved account from the account store",                                   vec![ reg_arg_req( "name", Kind::String ).with_description( "Account name to operate on" ), dry(), trc(), bfs( "force", "Bypass G6 ownership gate; allow deleting a non-owned account" ) ], Box::new( account_delete_routine  ) );
  reg_cmd( registry, ".account.relogin", "Force browser re-authentication for a named account with dead refreshToken",     vec![ nam(), dry(), trc(), bfs( "force", "Bypass G7 ownership gate; allow re-authenticating a non-owned account" ) ], Box::new( account_relogin_routine ) );
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
  reg_cmd( registry, ".account.inspect", "Show identity, subscription, and org fields for one account via live endpoints",
    vec![
      nam(),
      bfs( "refresh", "Attempt OAuth token refresh when stored credentials are locally expired (1 = enabled, default; 0 = disabled)" ),
      trc(),
      fmt(),
    ],
    Box::new( account_inspect_routine ) );
  reg_cmd( registry, ".account.tag", "Add, remove, or replace tags on one or more saved accounts",
    vec![
      reg_arg_opt( "name",   Kind::String ).with_description( "Account name or comma-separated list; every component must resolve before any write" ),
      reg_arg_opt( "add",    Kind::String ).with_description( "Comma-separated tags to union into the stored set; exactly one of add::/remove::/tags::" ),
      reg_arg_opt( "remove", Kind::String ).with_description( "Comma-separated tags to drop from the stored set (absent tags are a no-op); exactly one of add::/remove::/tags::" ),
      reg_arg_opt( "tags",   Kind::String ).with_description( "Comma-separated tags replacing the whole stored set; exactly one of add::/remove::/tags::" ),
      dry(),
      trc(),
    ],
    Box::new( account_tag_routine ) );
  reg_cmd( registry, ".model", "Get or set model + effort level for the session (~/.claude/settings.json) or subprocess (~/.clr/config.toml) scope",
    vec![
      reg_arg_opt( "scope",              Kind::String  ).with_description( "Backing store: `session` (~/.claude/settings.json, default) or `subprocess` (~/.clr/config.toml user tier)" ),
      reg_arg_opt( "model",              Kind::String  ).with_description( "Set model: `opus`/`sonnet`/`haiku`/`default` (session, shorthand) or any non-empty full model ID (subprocess)" ),
      reg_arg_opt( "effort_level",       Kind::String  ).with_description( "Set effort: `low`/`normal`/`high`/`max` (session) or `low`/`medium`/`high`/`max` (subprocess)" ),
      reg_arg_opt( "reset_model",        Kind::Integer ).with_description( "Remove the model key for the selected scope; mutually exclusive with model:: (1 = reset)" ),
      reg_arg_opt( "reset_effort_level", Kind::Integer ).with_description( "Remove the effort key for the selected scope; mutually exclusive with effort_level:: (1 = reset)" ),
      fmt(),
    ],
    Box::new( model_routine ) );
  reg_cmd( registry, ".models", "List available Claude API models from the static catalog or live endpoint",
    vec![
      reg_arg_opt( "offline", Kind::Integer ).with_description( "Use static embedded catalog without network (1 = offline, 0 = live fetch, default: 0)" ),
      reg_arg_opt( "name",    Kind::String  ).with_description( "Filter by substring match on model ID (case-insensitive)" ),
      fmt(),
    ],
    Box::new( models_routine ) );
  // `.model.select` stays registered (dispatchable, returns a migration-error stub —
  // see `model_select_routine`) but is hidden from the `.help`/`.` listing (AC-26,
  // Feature 035): `.model scope::subprocess` is the single listed entry for this
  // functionality now. Bypasses `reg_cmd()` (no hidden-flag param) — mirrors the
  // inline `CommandDefinition::former()...hidden_from_list(true)` pattern `src/cli.rs`
  // already uses to hide the bare `.` command from its own listing.
  {
    let def = unilang::data::CommandDefinition::former()
    .name( ".model.select" )
    .description( "REMOVED — use .model scope::subprocess instead" )
    .arguments( vec![
      reg_arg_opt( "id",    Kind::String  ).with_description( "REMOVED — use .model scope::subprocess model::VALUE instead" ),
      reg_arg_opt( "reset", Kind::Integer ).with_description( "REMOVED — use .model scope::subprocess reset_model::1 instead" ),
      fmt(),
    ] )
    .hidden_from_list( true )
    .end();
    registry
    .register_with_routine( &def, Box::new( model_select_routine ) )
    .expect( "internal error: failed to register .model.select" );
  }
  reg_cmd( registry, ".provider.select", "Get or pin the global inference provider selection in ~/.clr/config.toml",
    vec![
      reg_arg_opt( "id",    Kind::String  ).with_description( "Provider name to select (e.g. kimi); free-form, no allow-list" ),
      reg_arg_opt( "reset", Kind::Integer ).with_description( "Remove the provider preference and revert to the anthropic default (1 = reset)" ),
      fmt(),
    ],
    Box::new( provider_select_routine ) );
  reg_cmd( registry, ".paths",          "Show all resolved ~/.claude/ canonical file paths",
    vec![
      fmt(),
      reg_arg_opt( "field", Kind::String ).with_description( "Output a single named path value; format:: is ignored when set. Valid: base, credentials, credential_store, projects, stats, settings, session_env, sessions" ),
      trc(),
    ],
    Box::new( paths_routine ) );
  reg_cmd( registry, ".tags", "List every tag across accounts and identity filters with usage counts",
    vec![ fmt() ],
    Box::new( tags_routine ) );
  reg_cmd( registry, ".identities", "List every known Identity (USER@MACHINE) with active-marker, ownership, and filter state",
    vec![ fmt() ],
    Box::new( identities_routine ) );
  reg_cmd( registry, ".identity.filter", "Get, set, or clear an Identity's include/exclude rotation tag filter",
    vec![
      reg_arg_opt( "include",  Kind::String ).with_description( "Comma-separated tags a candidate must ALL carry to be rotation-eligible; replaces the stored include side" ),
      reg_arg_opt( "exclude",  Kind::String ).with_description( "Comma-separated tags none of which a rotation candidate may carry; replaces the stored exclude side" ),
      bfs( "clear", "Delete the filter file (restore permit-all); idempotent; mutually exclusive with include::/exclude::" ),
      reg_arg_opt( "identity", Kind::String ).with_description( "Target another Identity's filter as USER@HOST (default: current $USER@$HOSTNAME)" ),
      fmt(),
    ],
    Box::new( identity_filter_routine ) );
  reg_cmd( registry, ".usage",          "Show live rate-limit quota for all saved accounts",
    vec![
      reg_arg_opt( "format", Kind::String ).with_description( "Output format: `text` (default), `json`, `tsv` (tab-separated, plain status), `plain` (no emoji), `value` (bare, use with `get::`)" ),
      reg_arg_opt( "refresh",   Kind::Integer ).with_description( "Retry once on 401/403 (auth errors) or 429 when token is locally expired, via isolated subprocess (1 = enabled, default; 0 = disabled)" ),
      reg_arg_opt( "live",      Kind::Integer ).with_description( "Continuous monitor mode (0 = off, default; 1 = on)" ),
      reg_arg_opt( "interval",  Kind::Integer ).with_description( "Seconds between refreshes (minimum 30, default 30)" ),
      reg_arg_opt( "jitter",    Kind::Integer ).with_description( "Max random seconds added to interval (0 = none, default)" ),
      reg_arg_opt( "trace",     Kind::Integer ).with_description( "Print timestamped diagnostic lines to stderr showing each credential read, API call, and refresh step (0 = off; 1 = on)" ),
      reg_arg_opt( "sort",      Kind::String  ).with_description( "Row ordering strategy: `renew` (default), `name`, `renews`" ),
      reg_arg_opt( "desc",      Kind::Integer ).with_description( "Sort direction: 0 = ascending (default for name/renew/renews), 1 = descending" ),
      reg_arg_opt( "prefer",    Kind::String  ).with_description( "Weekly quota column for strategies: `any` (default, min of both), `opus` (7d Left), `sonnet` (7d(Son))" ),
      reg_arg_opt( "next",      Kind::String  ).with_description( "REMOVED — use sort:: instead; kept for migration error" ),
      reg_arg_opt( "cols",      Kind::String  ).with_description( "Column visibility modifiers (comma-separated `+col_id`/`-col_id`); default shows all except `sub` and `7d_son_reset`" ),
      reg_arg_opt( "touch",             Kind::String  ).with_description( "Extend active 5h session windows via isolated subprocess for accounts with an active reset countdown (0/false = off; 1/true = on, default)" ),
      reg_arg_opt( "imodel",            Kind::String  ).with_description( "Subprocess model for touch/refresh: `auto` (default, haiku — sufficient for keep-alive), `sonnet` (claude-sonnet-5), `opus` (claude-opus-4-8), `haiku` (claude-haiku-4-5-20251001), `keep` (no --model flag)" ),
      reg_arg_opt( "effort",            Kind::String  ).with_description( "Subprocess effort level: `auto` (default, low for any model), `low` (always --effort low), `normal` (always --effort normal), `high` (always --effort high), `max` (always --effort max)" ),
      // Row filtering parameters (TSK-223)
      reg_arg_opt( "count",             Kind::Integer ).with_description( "Max rows to display; 0 = show all (default)" ),
      reg_arg_opt( "offset",            Kind::Integer ).with_description( "Skip first N rows from the filtered result before display (default 0)" ),
      reg_arg_opt( "only_active",       Kind::String  ).with_description( "Show only the per-machine active account row (0 = off, default; 1 = on)" ),
      reg_arg_opt( "only_next",         Kind::String  ).with_description( "Show only the recommended next account row (0 = off, default; 1 = on)" ),
      reg_arg_opt( "min_5h",            Kind::Integer ).with_description( "Hide rows where 5h Left is below this percentage 0–100 (default 0 = no filter); rows with no quota also hidden" ),
      reg_arg_opt( "min_7d",            Kind::Integer ).with_description( "Hide rows where 7d Left is below this percentage 0–100 (default 0 = no filter); rows with no quota also hidden" ),
      reg_arg_opt( "only_valid",        Kind::String  ).with_description( "Hide 🔴 rows (invalid/expired token) (0 = off, default; 1 = on)" ),
      reg_arg_opt( "exclude_exhausted", Kind::String  ).with_description( "Hide 🟡 and 🔴 rows; show only 🟢 rows (0 = off, default; 1 = on)" ),
      // Extraction and display (TSK-224)
      reg_arg_opt( "get",       Kind::String  ).with_description( "Extract bare field value for first row: `5h_left`, `5h_reset`, `7d_left`, `7d_son`, `7d_reset`, `expires`, `renews`, `sub`, `status`, `account`, `host`, `role`, `next_event_type`, `next_event_secs`" ),
      reg_arg_opt( "no_color",  Kind::String  ).with_description( "Strip emoji and ANSI sequences; status shows `ok`/`warn`/`err` (0 = off, default; 1 = on)" ),
      reg_arg_opt( "set_model", Kind::String  ).with_description( "Set Claude Code session model: `opus` (claude-opus-4-8), `sonnet` (claude-sonnet-5), `haiku` (claude-haiku-4-5-20251001), `default` (removes override)" ),
      // Mutation params (Feature 037 — unified with .accounts)
      nam(),
      dry(),
      bfd( "assign",  "REMOVED — use assignee::USER@MACHINE name::X (or assignee::0 name::X for current machine)" ),
      bfd( "unclaim", "REMOVED — use owner::0 name::X instead (or owner::0 alone to batch-clear)" ),
      reg_arg_opt( "owner", Kind::String ).with_description( "Set or clear account ownership: USER@MACHINE identity to set; sentinel value \"0\" clears ownership (owner::0)" ),
      bfs( "force",   "Bypass G8 ownership gate on owner:: (default 0)" ),
      bfs( "lock",    "Set (\"1\") or clear (\"0\") claim-lock: excludes from unattended rotation and explicit-switch targets; ungated (Feature 070)" ),
      bfs( "reserve", "Set (\"1\") or clear (\"0\") reserve marker: deprioritizes (does not exclude) in sort-based selection; ungated (Feature 070)" ),
      bfs( "for",     "REMOVED — use assignee::USER@MACHINE name::X (or assignee::0 name::X for current machine)" ),
      bfs( "active",  "REMOVED — use assignee::USER@MACHINE name::X (or assignee::0 name::X for current machine)" ),
      reg_arg_opt( "assignee", Kind::String ).with_description( "USER@MACHINE (or sentinel \"0\" = $USER@$HOSTNAME) assign/unassign active-account marker; Feature 065" ),
      // Rotation param (Feature 038)
      reg_arg_opt( "rotate", Kind::Integer ).with_description( "Switch to the → winner after rendering the quota table (0 = off, default; 1 = on); mutually exclusive with live::1" ),
      // Sessions table visibility (Plan 022)
      reg_arg_opt( "who",  Kind::Integer ).with_description( "Sessions table: auto (default — shown when >1 active marker), 0 = suppress, 1 = force on" ),
      // Token conservation (TSK-314)
      reg_arg_opt( "solo", Kind::Integer ).with_description( "token conservation: restrict fetch to current+owned account only (0 = off, default; 1 = on)" ),
      // Stale-first fetch reduction (task 499)
      reg_arg_opt( "stalest", Kind::Integer ).with_description( "Fetch only the K accounts with the oldest cache (K >= 1); others render from cache; mutually exclusive with only_active::1; ignored with rotate::1" ),
      reg_arg_opt( "max_age", Kind::Integer ).with_description( "With stalest::, only refresh accounts whose cache is older than SECS seconds (default 0 = no threshold); requires stalest::" ),
      // Burn-rate alert (task 544)
      reg_arg_opt( "alert", Kind::Integer ).with_description( "Burn-rate alert horizon in minutes: warn under the table when a 5h window is forecast to exhaust within N minutes (default 15; 0 = off)" ),
    ],
    Box::new( accounts_view_routine   ) );
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
