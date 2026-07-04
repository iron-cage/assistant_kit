//! Refresh predicate for quota errors.
//!
//! `should_refresh` decides whether a quota error warrants a credential refresh.
//! Declared `pub(super)` so `refresh.rs` can re-import without leaking outside
//! the `usage` module.
//!
//! ## Architectural Constraint: No Proactive (Approaching-Expiry) Arm
//!
//! It is permanently forbidden to add an arm that triggers `should_refresh` when a token is
//! valid but approaching expiry (`expires_secs > now_secs && expires_secs <= now_secs + MARGIN`).
//!
//! **Why:** `refresh_account_token()` calls `run_isolated(["--print", "."])`. When the access
//! token is still valid, Claude Code uses it as-is and exits without performing an OAuth refresh,
//! returning `credentials=None`. An approaching-expiry arm would invoke `refresh_account_token()`
//! and get `credentials=None` back — a silent no-op that wastes 35 seconds per account per poll.
//!
//! **Spec reference:** `docs/feature/017_token_refresh.md` line 8 explicitly marks "proactive
//! expiry detection before any API call" as **Out of Scope**.
//!
//! **Mechanism reference:** `docs/invariant/008_single_token_refresh_entry.md` — the `expiresAt=1`
//! trick only works because it forces Claude Code to treat the AT as expired before calling the
//! OAuth server. A genuinely valid AT cannot be force-refreshed this way via `run_isolated`.
//!
//! SR-11 (`sr11_approaching_expiry_must_not_trigger_refresh`) enforces this constraint in tests.
//! If this constraint is ever proposed for removal, first resolve the subprocess limitation —
//! until `run_isolated` supports proactive token rotation, the arm cannot be made functional.
//! See BUG-323 for the full investigation history.

use super::types::AccountQuota;

// ── Refresh predicate ─────────────────────────────────────────────────────────

/// Return `true` when `apply_refresh` should attempt a token refresh for `aq`.
///
/// Triggers on:
/// - 401 or 403 — authentication failure; token rejected by the server.
/// - "token expired (local)" — `fetch_all_quota` skipped the API call because
///   `expiresAt` is in the past; the OAuth **refresh** token is still valid and
///   Claude Code will renew the access token automatically via `run_isolated()`.
/// - 429 AND locally expired (`expires_at_ms / 1000 ≤ now_secs`) — the per-account
///   credentials file may be stale (Claude Code updated the live session file but not
///   the saved per-account copy). Refreshing updates both the token and `expiresAt`.
/// - `cached=true` AND locally expired — the cache fallback in `fetch_all_quota`
///   converts `Err` to `Ok(cached_data)`; all Err guards are bypassed. An expired
///   cached entry needs a fresh token just as much as an Err entry does (BUG-255).
///
/// Returns `false` for 429 with a non-expired local token: the token is valid;
/// refreshing would add a 30-second subprocess wait with no benefit.
/// Returns `false` for cached accounts with a valid (non-expired) token: the cache
/// hit is legitimate; there is nothing to refresh.
#[ must_use ]
#[ inline ]
pub fn should_refresh( aq : &AccountQuota, now_secs : u64 ) -> bool
{
  // G2: Non-owned accounts must never be refreshed — credential mutation forbidden.
  // Fix(BUG-303): add is_occupied_elsewhere guard — credential mutation for owned-but-occupied
  //   accounts invalidates the live session on the other machine.
  // Root cause: G2 was written when is_occupied_elsewhere was not yet available (Feature 036).
  // Pitfall: is_owned and is_occupied_elsewhere are independent flags; both can be true simultaneously.
  if !aq.is_owned || aq.is_occupied_elsewhere { return false; }

  if matches!( aq.result, Err( ref e ) if e.contains( "401" ) || e.contains( "403" ) )
  {
    return true;
  }
  // Fix(BUG-235): refresh when token is locally expired — OAuth refresh token still valid.
  // Root cause: `fetch_all_quota` stores Err("token expired (local)") and skips the API call
  //   (correct — avoids a guaranteed 401). But `should_refresh` only checked for "401"/"403"/"429";
  //   "token expired (local)" matched none of them, so the account was silently skipped.
  // Pitfall: the access token and refresh token have independent lifetimes; a past `expiresAt`
  //   does NOT mean the refresh token is expired. Always attempt renewal for locally-expired tokens.
  if matches!( aq.result, Err( ref e ) if e.contains( "token expired (local)" ) )
  {
    return true;
  }
  // Fix(BUG-156): also refresh when rate-limited AND locally expired.
  // Root cause: 429+expired accounts were unconditionally excluded; the guard
  //   assumed "429 = valid token" but a past `expiresAt` indicates the per-account
  //   file may be stale — the token may need refreshing despite the 429 response.
  // Pitfall: don't refresh ALL 429 accounts (as task 142 did) — that adds a
  //   pointless 30-second wait for valid-but-rate-limited accounts.
  if matches!( aq.result, Err( ref e ) if e.contains( "429" ) )
    && ( aq.expires_at_ms / 1000 ) <= now_secs
  {
    return true;
  }
  // Fix(BUG-255): refresh cached+expired account — cache fallback converts Err to Ok.
  // Root cause: when fetch fails, `fetch_all_quota` returns Ok(cached_data) with cached=true.
  //   All existing guards match only Err variants; a cached account with Ok result and an
  //   expired local token is never refreshed, leaving stale credentials permanently.
  // Pitfall: don't refresh ALL cached accounts — only those with an expired token; a cached
  //   account with a valid (non-expired) token needs no credential refresh.
  aq.cached && ( aq.expires_at_ms / 1000 ) <= now_secs
}


// Tests live in tests/usage/refresh_predicate_tests.rs (integration tests via test_bridge).
