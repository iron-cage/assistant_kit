//! Shared types for the `.usage` command module.
//!
//! All enums, structs, and their `impl` blocks live here so other submodules
//! can import them without circular dependencies.
// Items are pub for test_bridge re-export (testing feature); missing_docs and
// missing_debug_implementations are suppressed here — these are internal types
// exposed only via the test_bridge gated module.
#![ allow( missing_docs, missing_debug_implementations, clippy::missing_inline_in_public_items, clippy::must_use_candidate, clippy::missing_errors_doc ) ]

use claude_quota::OauthUsageData;

// ── Sort and prefer strategies ─────────────────────────────────────────────────

#[ derive( Copy, Clone, PartialEq, Eq, Debug ) ]
pub enum SortStrategy { Name, Renew, Renews }

impl SortStrategy
{
  pub fn parse( s : &str ) -> Result< Self, String >
  {
    match s
    {
      "name"   => Ok( Self::Name ),
      "renew"  => Ok( Self::Renew ),
      "renews" => Ok( Self::Renews ),
      _        => Err( format!(
        "invalid sort:: value {s:?}: valid values are `name`, `renew`, `renews`",
      ) ),
    }
  }

  /// Context-sensitive default `desc` direction for each strategy.
  ///
  /// All strategies default to ascending (`false`).
  pub fn default_desc( self ) -> bool
  {
    match self { Self::Name | Self::Renew | Self::Renews => false }
  }
}

#[ derive( Copy, Clone, PartialEq, Eq, Debug ) ]
pub enum PreferStrategy { Any, Opus, Sonnet }

impl PreferStrategy
{
  pub fn parse( s : &str ) -> Result< Self, String >
  {
    match s
    {
      "any"    => Ok( Self::Any ),
      "opus"   => Ok( Self::Opus ),
      "sonnet" => Ok( Self::Sonnet ),
      _        => Err( format!(
        "invalid prefer:: value {s:?}: valid values are `any`, `opus`, `sonnet`",
      ) ),
    }
  }
}

/// Column visibility state for the `.usage` quota table.
///
/// `flag` (first col) and `account` (name) are structural and always visible.
/// All other columns follow the default set; `cols::` modifiers toggle each one.
#[ derive( Debug ) ]
#[ allow( clippy::struct_excessive_bools ) ]
pub struct ColsVisibility
{
  /// `●` composite status emoji column (default ON).
  pub status       : bool,
  /// `Expires` token TTL column (default ON).
  pub expires      : bool,
  /// `Sub` subscription label column (default OFF).
  pub sub          : bool,
  /// `~Renews` next billing date column (default ON).
  pub renews       : bool,
  /// `5h Left` session quota remaining (default ON).
  pub h5_left      : bool,
  /// `5h Reset` session reset countdown (default ON).
  pub h5_reset     : bool,
  /// `7d Left` weekly quota remaining (default ON).
  pub d7_left      : bool,
  /// `7d(Son)` Sonnet-only weekly quota remaining (default ON).
  pub d7_son       : bool,
  /// `7d Reset` weekly reset countdown (default ON).
  pub d7_reset     : bool,
  /// `7d Son Reset` Sonnet weekly reset countdown (default OFF).
  pub d7_son_reset : bool,
  /// `Host` machine label column (default OFF).
  pub host         : bool,
  /// `Role` user-defined role tag column (default OFF).
  pub role         : bool,
  /// `Owner` account owner identity column (default ON).
  pub owner        : bool,
  /// `→ Next` soonest upcoming event column (default ON).
  pub next         : bool,
}

impl ColsVisibility
{
  pub fn default_set() -> Self
  {
    Self
    {
      status       : true,
      expires      : true,
      sub          : false,
      renews       : true,
      h5_left      : true,
      h5_reset     : true,
      d7_left      : true,
      d7_son       : true,
      d7_reset     : true,
      d7_son_reset : false,
      host         : false,
      role         : false,
      owner        : true,
      next         : true,
    }
  }

  pub fn apply_modifier( &mut self, modifier : &str ) -> Result< (), String >
  {
    let ( show, id ) = if let Some( rest ) = modifier.strip_prefix( '+' )
    {
      ( true, rest )
    }
    else if let Some( rest ) = modifier.strip_prefix( '-' )
    {
      ( false, rest )
    }
    else
    {
      return Err( format!( "cols:: modifier {modifier:?} must start with `+` or `-`" ) );
    };
    match id
    {
      "status"       => self.status       = show,
      "expires"      => self.expires      = show,
      "sub"          => self.sub          = show,
      "renews"       => self.renews       = show,
      "5h_left"      => self.h5_left      = show,
      "5h_reset"     => self.h5_reset     = show,
      "7d_left"      => self.d7_left      = show,
      "7d_son"       => self.d7_son       = show,
      "7d_reset"     => self.d7_reset     = show,
      "7d_son_reset" => self.d7_son_reset = show,
      "host"         => self.host         = show,
      "role"         => self.role         = show,
      "owner"        => self.owner        = show,
      "next"         => self.next         = show,
      _              => return Err( format!(
        "cols:: unknown column {id:?}: valid IDs are `status`, `expires`, `sub`, `renews`, `5h_left`, `5h_reset`, `7d_left`, `7d_son`, `7d_reset`, `7d_son_reset`, `host`, `role`, `owner`, `next`",
      ) ),
    }
    Ok( () )
  }

  pub fn parse( s : &str ) -> Result< Self, String >
  {
    let mut vis = Self::default_set();
    for modifier in s.split( ',' ).map( str::trim ).filter( |m| !m.is_empty() )
    {
      vis.apply_modifier( modifier )?;
    }
    Ok( vis )
  }
}

// ── Per-account quota result ───────────────────────────────────────────────────

/// Per-account quota fetch result, bundling identity, state flags, and the raw usage data.
#[ allow( clippy::struct_excessive_bools ) ]
pub struct AccountQuota
{
  pub name                  : String,
  /// Live-token match: `accessToken` in `~/.claude/.credentials.json` equals this account's stored token.
  pub is_current            : bool,
  /// Active-marker match: per-machine active marker file in the credential store names this account.
  pub is_active             : bool,
  /// Another machine's `_active_*` file names this account.
  pub is_occupied_elsewhere : bool,
  pub expires_at_ms         : u64,
  /// `Ok` = live quota fetched; `Err` = reason string (expired, network, etc.).
  pub result                : Result< OauthUsageData, String >,
  /// Billing state from `GET /api/oauth/account`; `None` if the fetch failed.
  pub account               : Option< claude_quota::OauthAccountData >,
  /// Machine label from `{name}.json`; empty when absent.
  pub host                  : String,
  /// User-defined role tag from `{name}.json`; empty when absent.
  pub role                  : String,
  /// Override billing renewal date from `{name}.json`; `None` when not set.
  pub renewal_at            : Option< String >,
  /// `true` when result was loaded from cache (fetch failed, fallback used).
  pub cached                : bool,
  /// Seconds since last successful fetch; present only when `cached == true`.
  pub cache_age_secs        : Option< u64 >,
  /// `true` when `owner` in `{name}.json` is empty or matches `current_identity()`.
  /// `false` for accounts owned by a different machine — G1–G7 enforcement gates apply.
  pub is_owned              : bool,
  /// Raw owner identity string from `{name}.json`; empty when unset.
  pub owner                 : String,
}

// ── Command handler ────────────────────────────────────────────────────────────

/// Parsed `.usage` parameters extracted from a `VerifiedCommand`.
#[ derive( Debug ) ]
#[ allow( clippy::struct_excessive_bools ) ]
pub struct UsageParams
{
  /// 1 = auto-refresh expired tokens (default); 0 = show errors as-is.
  pub refresh           : i64,
  /// 1 = continuous live-monitor loop; 0 = single fetch (default).
  pub live              : i64,
  /// Seconds between live-loop cycles (default 30; only validated when live=1).
  pub interval          : u64,
  /// Max random seconds added to each cycle (default 0; only validated when live=1).
  pub jitter            : u64,
  /// true = emit timestamped diagnostic lines to stderr (`YYYY-MM-DD · HH:MM:SS · …`).
  pub trace             : bool,
  /// Row ordering strategy for the text table.
  pub sort              : SortStrategy,
  /// Sort direction override; `None` = use strategy's context-sensitive default.
  pub desc              : Option< bool >,
  /// Weekly quota column selector for strategies that reference weekly availability.
  pub prefer            : PreferStrategy,
  /// Column visibility modifiers applied to the text table.
  pub cols              : ColsVisibility,
  /// 1 = activate idle 5h session windows via subprocess (default); 0 = off.
  pub touch             : i64,
  /// Subprocess model selection (default: `auto`).
  pub imodel            : SubprocessModel,
  /// Subprocess effort level (default: `auto`).
  pub effort            : SubprocessEffort,
  // ── Row filtering (TSK-223) ────────────────────────────────────────────────
  /// Max rows to display; 0 = show all.
  pub count             : u64,
  /// Skip first N rows from the filtered result before display.
  pub offset            : u64,
  /// When true, show only the per-machine active account row.
  pub only_active       : bool,
  /// When true, show only the row selected as the recommended next account.
  pub only_next         : bool,
  /// Minimum 5h quota percentage (0–100); rows below threshold are hidden.
  pub min_5h            : u8,
  /// Minimum 7d quota percentage (0–100); rows below threshold are hidden.
  pub min_7d            : u8,
  /// When true, hide 🔴 rows (invalid/expired token).
  pub only_valid        : bool,
  /// When true, hide 🟡 and 🔴 rows; show only 🟢 rows.
  pub exclude_exhausted : bool,
  // ── Format / extraction (TSK-224) ─────────────────────────────────────────
  /// Output format for the result set.
  pub format    : UsageOutputFormat,
  /// When `Some`, extract this field's value from the first row as bare string.
  pub get       : Option< GetField >,
  /// When true, replace percentage columns with absolute token counts (no-op when API data absent).
  pub abs       : bool,
  /// When true, strip emoji and ANSI sequences from the output.
  pub no_color  : bool,
  /// When `Some`, write this value to `set_session_model` instead of running `apply_model_override`.
  /// String is the raw user-provided value (e.g., `"opus"`, `"default"`); resolve at use site.
  pub set_model : Option< String >,
  // ── Rotation (Feature 038) ─────────────────────────────────────────────────
  /// When true, switch to the `→` winner after rendering the quota table.
  pub rotate    : bool,
  /// When true, bypass the G5 ownership gate on the rotate path (and G8 on unclaim).
  pub force     : bool,
  // ── Sessions table (Plan 022) ──────────────────────────────────────────────
  /// Controls sessions table visibility: `None` = auto (shown when >1 `_active_*` marker),
  /// `Some(true)` = force on, `Some(false)` = suppress.
  pub who       : Option< bool >,
  // ── Token conservation (TSK-314) ──────────────────────────────────────────
  /// When true, restrict all fetch/refresh/touch operations to the current+owned account.
  /// All other accounts use approximated historical data from the quota cache.
  pub solo      : bool,
}

// ── Output format ─────────────────────────────────────────────────────────────

/// Output format for the `.usage` command.
#[ derive( Copy, Clone, PartialEq, Eq, Debug ) ]
pub enum UsageOutputFormat
{
  /// Human-readable table (default).
  Text,
  /// Machine-readable JSON array.
  Json,
  /// Tab-separated values, plain-text status labels (`ok`/`warn`/`err`).
  Tsv,
  /// Same layout as `Text` with no emoji or ANSI sequences.
  Plain,
  /// Bare value extraction; outputs one field for the first row only.
  Value,
}

// ── GetField ──────────────────────────────────────────────────────────────────

/// Field selector for `get::` single-value extraction.
#[ derive( Copy, Clone, PartialEq, Eq, Debug ) ]
pub enum GetField
{
  FiveHourLeft,
  FiveHourReset,
  SevenDayLeft,
  SevenDaySon,
  SevenDayReset,
  Expires,
  Renews,
  Sub,
  Status,
  Account,
  Host,
  Role,
  NextEventType,
  NextEventSecs,
}

impl GetField
{
  pub fn parse( s : &str ) -> Result< Self, String >
  {
    match s
    {
      "5h_left"         => Ok( Self::FiveHourLeft ),
      "5h_reset"        => Ok( Self::FiveHourReset ),
      "7d_left"         => Ok( Self::SevenDayLeft ),
      "7d_son"          => Ok( Self::SevenDaySon ),
      "7d_reset"        => Ok( Self::SevenDayReset ),
      "expires"         => Ok( Self::Expires ),
      "renews"          => Ok( Self::Renews ),
      "sub"             => Ok( Self::Sub ),
      "status"          => Ok( Self::Status ),
      "account"         => Ok( Self::Account ),
      "host"            => Ok( Self::Host ),
      "role"            => Ok( Self::Role ),
      "next_event_type" => Ok( Self::NextEventType ),
      "next_event_secs" => Ok( Self::NextEventSecs ),
      _                 => Err( format!(
        "invalid get:: field {s:?}: valid IDs are \
`5h_left`, `5h_reset`, `7d_left`, `7d_son`, `7d_reset`, `expires`, `renews`, \
`sub`, `status`, `account`, `host`, `role`, `next_event_type`, `next_event_secs`",
      ) ),
    }
  }
}

// ── Subprocess model / effort enums ───────────────────────────────────────────

/// `imodel::` parameter value — determines how the subprocess model is selected.
#[ derive( Copy, Clone, PartialEq, Eq, Debug ) ]
pub enum SubprocessModel { Auto, Sonnet, Opus, Keep, Haiku }

impl SubprocessModel
{
  pub fn parse( s : &str ) -> Result< Self, String >
  {
    match s
    {
      "auto"   => Ok( Self::Auto ),
      "sonnet" => Ok( Self::Sonnet ),
      "opus"   => Ok( Self::Opus ),
      "keep"   => Ok( Self::Keep ),
      "haiku"  => Ok( Self::Haiku ),
      _ => Err( format!( "imodel:: must be one of: auto, sonnet, opus, keep, haiku; got {s:?}" ) ),
    }
  }
}

/// `effort::` parameter value — determines the `--effort` flag injected into subprocesses.
#[ derive( Copy, Clone, PartialEq, Eq, Debug ) ]
pub enum SubprocessEffort { Auto, High, Max, Low, Normal }

impl SubprocessEffort
{
  pub fn parse( s : &str ) -> Result< Self, String >
  {
    match s
    {
      "auto"   => Ok( Self::Auto ),
      "high"   => Ok( Self::High ),
      "max"    => Ok( Self::Max ),
      "low"    => Ok( Self::Low ),
      "normal" => Ok( Self::Normal ),
      _ => Err( format!( "effort:: must be one of: auto, high, max, low, normal; got {s:?}" ) ),
    }
  }
}

// ── Quota status thresholds ────────────────────────────────────────────────────

/// Sonnet 7d model-override boundary.
///
/// `recommended_model()` and `apply_model_override()` select Opus when
/// `Sonnet 7d Left < 10%` (i.e., `utilization > 90.0`). All model-override call
/// sites must reference this constant; never duplicate the literal `10.0`.
///
/// 5-hour exhaustion uses the separate `H_EXHAUSTED_THRESHOLD = 15.0`.
pub const OPUS_OVERRIDE_THRESHOLD : f64 = 10.0;

/// 5-hour session quota exhaustion boundary.
///
/// An account is classified **h-exhausted** when `5h Left ≤ 15%`. All 5h-exhaustion
/// call sites (`pct_emoji`, `status_emoji`, `status_group_of`) must reference this
/// constant; never duplicate the literal `15.0`.
///
/// Sonnet 7d model-override uses the separate `OPUS_OVERRIDE_THRESHOLD = 10.0`.
pub const H_EXHAUSTED_THRESHOLD : f64 = 15.0;

/// 7-day weekly quota exhaustion boundary.
///
/// An account is classified **weekly-exhausted** when `7d Left ≤ 5%`. All comparison
/// sites must reference this constant; never duplicate the literal `5.0`.
pub const WEEKLY_EXHAUSTION_THRESHOLD : f64 = 5.0;

/// Map a model shorthand to its full model ID.
///
/// Returns `Some(Some(model_id))` for `opus`, `sonnet`, `haiku`;
/// `Some(None)` for `default` (removes the `model` key from `settings.json`);
/// `None` for unknown values.
///
/// Shared by `validate_set_model` (`.account.use` / `.usage` `set_model::` parameter)
/// and the `.model` command handler. The model-ID table lives here exactly once.
// `Option<Option<T>>` is intentional: tri-state (known model / remove key / unknown input).
#[ allow( clippy::option_option ) ]
pub fn map_model_shorthand( s : &str ) -> Option< Option< &'static str > >
{
  match s
  {
    "opus"    => Some( Some( "claude-opus-4-8" ) ),
    "sonnet"  => Some( Some( "claude-sonnet-5" ) ),
    "haiku"   => Some( Some( "claude-haiku-4-5-20251001" ) ),
    "default" => Some( None ),
    _         => None,
  }
}

/// Validate a `set_model::` string and resolve to the model ID to write.
///
/// Returns `Ok(Some(model_id))` for `opus`, `sonnet`, `haiku`;
/// `Ok(None)` for `default` (removes the `model` key from `settings.json`);
/// `Err(message)` for unknown values.
pub fn validate_set_model( s : &str ) -> Result< Option< &'static str >, String >
{
  map_model_shorthand( s )
    .ok_or_else( || format!( "set_model:: must be one of: opus, sonnet, haiku, default; got {s:?}" ) )
}
