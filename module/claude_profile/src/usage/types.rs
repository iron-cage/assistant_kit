//! Shared types for the `.usage` command module.
//!
//! All enums, structs, and their `impl` blocks live here so other submodules
//! can import them without circular dependencies.

use claude_quota::OauthUsageData;

// ── Sort and prefer strategies ─────────────────────────────────────────────────

#[ derive( Copy, Clone, PartialEq, Eq, Debug ) ]
pub( crate ) enum SortStrategy { Name, Endurance, Drain, Renew, Next }

impl SortStrategy
{
  pub( crate ) fn parse( s : &str ) -> Result< Self, String >
  {
    match s
    {
      "name"      => Ok( Self::Name ),
      "endurance" => Ok( Self::Endurance ),
      "drain"     => Ok( Self::Drain ),
      "renew"     => Ok( Self::Renew ),
      "next"      => Ok( Self::Next ),
      _           => Err( format!(
        "invalid sort:: value {s:?}: valid values are `name`, `endurance`, `drain`, `renew`, `next`",
      ) ),
    }
  }

  /// Context-sensitive default `desc` direction for each strategy.
  ///
  /// `Endurance` defaults to `true` (best on top). All others default to `false`.
  /// `Next` is always resolved to a concrete strategy before `default_desc` is called
  /// (see `parse_usage_params`), so this arm is unreachable in practice.
  pub( crate ) fn default_desc( self ) -> bool
  {
    matches!( self, SortStrategy::Endurance )
  }
}

#[ derive( Copy, Clone, PartialEq, Eq, Debug ) ]
pub( crate ) enum PreferStrategy { Any, Opus, Sonnet }

impl PreferStrategy
{
  pub( crate ) fn parse( s : &str ) -> Result< Self, String >
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

#[ derive( Copy, Clone, PartialEq, Eq, Debug ) ]
pub( crate ) enum NextStrategy { Renew, Endurance, Drain }

impl NextStrategy
{
  pub( crate ) fn parse( s : &str ) -> Result< Self, String >
  {
    match s
    {
      "renew"     => Ok( Self::Renew ),
      "endurance" => Ok( Self::Endurance ),
      "drain"     => Ok( Self::Drain ),
      _           => Err( format!(
        "invalid next:: value {s:?}: valid values are `renew`, `endurance`, `drain`",
      ) ),
    }
  }
}

/// Column visibility state for the `.usage` quota table.
///
/// `flag` (first col) and `account` (name) are structural and always visible.
/// All other columns follow the default set; `cols::` modifiers toggle each one.
#[ allow( clippy::struct_excessive_bools ) ]
pub( crate ) struct ColsVisibility
{
  /// `●` composite status emoji column (default ON).
  pub( crate ) status       : bool,
  /// `Expires` token TTL column (default ON).
  pub( crate ) expires      : bool,
  /// `Sub` subscription label column (default OFF).
  pub( crate ) sub          : bool,
  /// `~Renews` next billing date column (default ON).
  pub( crate ) renews       : bool,
  /// `5h Left` session quota remaining (default ON).
  pub( crate ) h5_left      : bool,
  /// `5h Reset` session reset countdown (default ON).
  pub( crate ) h5_reset     : bool,
  /// `7d Left` weekly quota remaining (default ON).
  pub( crate ) d7_left      : bool,
  /// `7d(Son)` Sonnet-only weekly quota remaining (default ON).
  pub( crate ) d7_son       : bool,
  /// `7d Reset` weekly reset countdown (default ON).
  pub( crate ) d7_reset     : bool,
  /// `7d Son Reset` Sonnet weekly reset countdown (default OFF).
  pub( crate ) d7_son_reset : bool,
  /// `Host` machine label column (default OFF).
  pub( crate ) host         : bool,
  /// `Role` user-defined role tag column (default OFF).
  pub( crate ) role         : bool,
}

impl ColsVisibility
{
  pub( crate ) fn default_set() -> Self
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
    }
  }

  pub( crate ) fn apply_modifier( &mut self, modifier : &str ) -> Result< (), String >
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
      _              => return Err( format!(
        "cols:: unknown column {id:?}: valid IDs are `status`, `expires`, `sub`, `renews`, `5h_left`, `5h_reset`, `7d_left`, `7d_son`, `7d_reset`, `7d_son_reset`, `host`, `role`",
      ) ),
    }
    Ok( () )
  }

  pub( crate ) fn parse( s : &str ) -> Result< Self, String >
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
pub( crate ) struct AccountQuota
{
  pub( crate ) name          : String,
  /// Live-token match: `accessToken` in `~/.claude/.credentials.json` equals this account's stored token.
  pub( crate ) is_current    : bool,
  /// Active-marker match: per-machine active marker file in the credential store names this account.
  pub( crate ) is_active     : bool,
  pub( crate ) expires_at_ms : u64,
  /// `Ok` = live quota fetched; `Err` = reason string (expired, network, etc.).
  pub( crate ) result        : Result< OauthUsageData, String >,
  /// Billing state from `GET /api/oauth/account`; `None` if the fetch failed.
  pub( crate ) account       : Option< claude_quota::OauthAccountData >,
  /// Machine label from `{name}.profile.json`; empty when absent.
  pub( crate ) host          : String,
  /// User-defined role tag from `{name}.profile.json`; empty when absent.
  pub( crate ) role          : String,
}

// ── Command handler ────────────────────────────────────────────────────────────

/// Parsed `.usage` parameters extracted from a `VerifiedCommand`.
#[ allow( clippy::struct_excessive_bools ) ]
pub( crate ) struct UsageParams
{
  /// 1 = auto-refresh expired tokens (default); 0 = show errors as-is.
  pub( crate ) refresh           : i64,
  /// 1 = continuous live-monitor loop; 0 = single fetch (default).
  pub( crate ) live              : i64,
  /// Seconds between live-loop cycles (default 30; only validated when live=1).
  pub( crate ) interval          : u64,
  /// Max random seconds added to each cycle (default 0; only validated when live=1).
  pub( crate ) jitter            : u64,
  /// true = emit `[trace]` diagnostic lines to stderr.
  pub( crate ) trace             : bool,
  /// Row ordering strategy for the text table.
  pub( crate ) sort              : SortStrategy,
  /// Sort direction override; `None` = use strategy's context-sensitive default.
  pub( crate ) desc              : Option< bool >,
  /// Weekly quota column selector for strategies that reference weekly availability.
  pub( crate ) prefer            : PreferStrategy,
  /// Recommendation strategy controlling `→` marker and footer format.
  pub( crate ) next              : NextStrategy,
  /// Column visibility modifiers applied to the text table.
  pub( crate ) cols              : ColsVisibility,
  /// 1 = activate idle 5h session windows via subprocess (default); 0 = off.
  pub( crate ) touch             : i64,
  /// Subprocess model selection (default: `auto`).
  pub( crate ) imodel            : SubprocessModel,
  /// Subprocess effort level (default: `auto`).
  pub( crate ) effort            : SubprocessEffort,
  // ── Row filtering (TSK-223) ────────────────────────────────────────────────
  /// Max rows to display; 0 = show all.
  pub( crate ) count             : u64,
  /// Skip first N rows from the filtered result before display.
  pub( crate ) offset            : u64,
  /// When true, show only the per-machine active account row.
  pub( crate ) only_active       : bool,
  /// When true, show only the row receiving the `→` recommendation marker.
  pub( crate ) only_next         : bool,
  /// Minimum 5h quota percentage (0–100); rows below threshold are hidden.
  pub( crate ) min_5h            : u8,
  /// Minimum 7d quota percentage (0–100); rows below threshold are hidden.
  pub( crate ) min_7d            : u8,
  /// When true, hide 🔴 rows (invalid/expired token).
  pub( crate ) only_valid        : bool,
  /// When true, hide 🟡 and 🔴 rows; show only 🟢 rows.
  pub( crate ) exclude_exhausted : bool,
  // ── Format / extraction (TSK-224) ─────────────────────────────────────────
  /// Output format for the result set.
  pub( crate ) format    : UsageOutputFormat,
  /// When `Some`, extract this field's value from the first row as bare string.
  pub( crate ) get       : Option< GetField >,
  /// When true, replace percentage columns with absolute token counts (no-op when API data absent).
  pub( crate ) abs       : bool,
  /// When true, strip emoji and ANSI sequences from the output.
  pub( crate ) no_color  : bool,
}

// ── Output format ─────────────────────────────────────────────────────────────

/// Output format for the `.usage` command.
#[ derive( Copy, Clone, PartialEq, Eq, Debug ) ]
pub( crate ) enum UsageOutputFormat
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
pub( crate ) enum GetField
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
  pub( crate ) fn parse( s : &str ) -> Result< Self, String >
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
pub( crate ) enum SubprocessModel { Auto, Sonnet, Opus, Keep, Haiku }

impl SubprocessModel
{
  pub( crate ) fn parse( s : &str ) -> Result< Self, String >
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
pub( crate ) enum SubprocessEffort { Auto, High, Max, Low, Normal }

impl SubprocessEffort
{
  pub( crate ) fn parse( s : &str ) -> Result< Self, String >
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
