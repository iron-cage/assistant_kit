//! Account domain types — [`AccountBackend`] discriminator and the [`Account`] metadata record.

/// Which API surface an account routes traffic through.
///
/// No serde derive — (de)serialized manually via `parse_string_field()` at the
/// same call sites that already parse other `Account` string fields, matching
/// this module's existing no-serde-derive convention throughout.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum AccountBackend
{
  /// Anthropic OAuth — the default for every account with no `backend` key.
  Anthropic,
  /// Static-API-key redirect to a foreign, Anthropic-API-compatible endpoint.
  Redirect,
}

impl AccountBackend
{
  /// Parse a `backend` field value. Unrecognized or absent → `Anthropic`
  /// (never an error) — this is what makes AC-05's "neither errors nor
  /// misclassifies" hold at the type level.
  #[ must_use ]
  #[ inline ]
  pub fn parse( s : &str ) -> Self
  {
    match s
    {
      "redirect" => Self::Redirect,
      _          => Self::Anthropic,
    }
  }

  /// Return the canonical string form written to `{name}.json`'s `backend` key.
  #[ must_use ]
  #[ inline ]
  pub fn as_str( &self ) -> &'static str
  {
    match self
    {
      Self::Anthropic => "anthropic",
      Self::Redirect  => "redirect",
    }
  }
}

/// Metadata for a saved Claude Code account credential snapshot.
#[ derive( Debug, Clone ) ]
#[ allow( clippy::struct_excessive_bools ) ]
pub struct Account
{
  /// Account name — the email address used as the credential filename stem.
  pub name : String,
  /// Claude subscription type (e.g., `"max"`, `"pro"`).
  pub subscription_type : String,
  /// Rate limit tier identifier.
  pub rate_limit_tier : String,
  /// OAuth token expiry as Unix epoch milliseconds.
  pub expires_at_ms : u64,
  /// Whether this account's credentials are currently active.
  pub is_active : bool,
  /// Email address from saved `{name}.json` `emailAddress`.
  /// Empty string when snapshot absent or field missing.
  pub email : String,
  /// Display name from saved `{name}.json` `oauthAccount.displayName`.
  /// Empty string when snapshot absent or field missing.
  pub display_name : String,
  /// Billing type from saved `{name}.json` `oauthAccount.billingType`.
  /// Empty string when snapshot absent or field missing.
  pub billing : String,
  /// Active model from saved `{name}.json` `model` field.
  /// Empty string when snapshot absent or field missing.
  pub model : String,
  /// Stable user identifier from saved `{name}.json` `oauthAccount.taggedId`.
  /// Empty string when snapshot absent or field missing.
  pub tagged_id : String,
  /// UUID form of user identifier from saved `{name}.json` `oauthAccount.uuid`.
  /// Empty string when snapshot absent or field missing.
  pub uuid : String,
  /// Enabled product capabilities from saved `{name}.json` `oauthAccount.capabilities`.
  /// Empty vec when snapshot absent or field missing.
  pub capabilities : Vec< String >,
  /// Organisation UUID from saved `{name}.json` `organization_uuid`.
  /// Empty string when snapshot absent or field missing.
  pub organization_uuid : String,
  /// Organisation display name from saved `{name}.json` `organization_name`.
  /// Empty string when snapshot absent or field missing.
  pub organization_name : String,
  /// User's role in the organisation from saved `{name}.json` `organization_role` (Roles API path).
  /// Empty string when snapshot absent or field missing.
  pub org_role : String,
  /// Workspace UUID from saved `{name}.json` `workspace_uuid`.
  /// Empty string when snapshot absent or field missing (personal accounts have `null`).
  pub workspace_uuid : String,
  /// Workspace display name from saved `{name}.json` `workspace_name`.
  /// Empty string when snapshot absent or field missing (personal accounts have `null`).
  pub workspace_name : String,
  /// Machine host label from saved `{name}.json` `host`.
  /// Empty string when file absent or field missing.
  pub host : String,
  /// User-defined role label from saved `{name}.json` `role`.
  /// Empty string when file absent or field missing.
  pub role : String,
  /// Account owner from saved `{name}.json` `owner`; empty when unset — see Feature 036.
  pub owner : String,
  /// `true` when `owner` is empty (unowned) or matches `current_identity()` (owned by this machine).
  pub is_owned : bool,
  /// Claim-lock from saved `{name}.json` `claim_lock`; `false` when unset — see Feature 070.
  /// `true` excludes this account from unattended rotation (Gate 9 / G9).
  pub claim_lock : bool,
  /// Reserve marker from saved `{name}.json` `reserve`; `false` when unset — see Feature 070.
  /// `true` deprioritizes (does not exclude) this account in sort-based selection.
  pub reserve : bool,
  /// Renewal override from saved `{name}.json` `_renewal_at`; `None` when unset — see Feature 030.
  pub renewal_at : Option< String >,
  /// Which API surface this account routes through; `Anthropic` when the `backend`
  /// key is absent or unrecognized — see Feature 071.
  pub backend : AccountBackend,
  /// Foreign API base URL from saved `{name}.json` `base_url`; `None` for Anthropic accounts
  /// or when unset — see Feature 071.
  pub base_url : Option< String >,
  /// Foreign model identifier from saved `{name}.json` `redirect_model`; `None` for
  /// Anthropic accounts or when unset — see Feature 071.
  pub redirect_model : Option< String >,
  /// Selected inference provider from saved `{name}.json` `inference_provider`; empty string
  /// when unset — see Feature 072. Written only via `.provider.select` (no auto-detection).
  pub inference_provider : String,
}
