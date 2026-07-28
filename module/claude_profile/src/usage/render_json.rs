// Items are pub for test_bridge re-export; lints suppressed — internal API.
#![ allow( clippy::missing_inline_in_public_items, clippy::must_use_candidate ) ]
//! JSON renderer for quota results.

use crate::output::json_escape;
use super::types::AccountQuota;
use super::format::{ renewal_secs, next_event_raw, shorten_error };

/// Produce the `"cached":bool,"cache_age_secs":N|null` JSON fragment.
fn cache_json_fields( cached : bool, age : Option< u64 > ) -> String
{
  let age_str = age.map_or_else( || "null".to_string(), |a| a.to_string() );
  format!( "\"cached\":{cached},\"cache_age_secs\":{age_str}" )
}

/// Render quota results as a JSON array (one object per account).
///
/// Every row includes `expires_in_secs`. Successful accounts include quota
/// fields using `_left_pct` naming (remaining, not consumed); failed accounts
/// include `error`.
#[ allow( clippy::too_many_lines ) ]
pub fn render_json( accounts : &[ AccountQuota ] ) -> String
{
  use std::time::{ SystemTime, UNIX_EPOCH };

  if accounts.is_empty()
  {
    return "[]\n".to_string();
  }

  let now_secs = SystemTime::now()
    .duration_since( UNIX_EPOCH )
    .unwrap_or_default()
    .as_secs();

  let mut parts = Vec::with_capacity( accounts.len() );
  for aq in accounts
  {
    let name_esc         = json_escape( &aq.name );
    let is_current_str            = if aq.is_current            { "true" } else { "false" };
    let is_active_str             = if aq.is_active             { "true" } else { "false" };
    let is_occupied_elsewhere_str = if aq.is_occupied_elsewhere { "true" } else { "false" };
    let is_owned_str              = if aq.is_owned              { "true" } else { "false" };
    let owner_esc                 = json_escape( &aq.owner );
    let expires_in_secs  = ( aq.expires_at_ms / 1000 ).saturating_sub( now_secs );
    let billing_type_str = aq.account.as_ref()
      .map_or_else( || "null".to_string(), |a| format!( "\"{}\"", json_escape( &a.billing_type ) ) );
    let has_max_str      = aq.account.as_ref()
      .map_or( "null", |a| if a.has_max { "true" } else { "false" } );
    let ren_pair                                       = renewal_secs(
      aq.renewal_at.as_deref(),
      aq.org_created_at.as_deref(),
      now_secs,
    );
    let ( renewal_secs_str, renewal_is_estimate_str ) = match ren_pair
    {
      Some( ( s, est ) ) => ( s.to_string(), if est { "true".to_string() } else { "false".to_string() } ),
      None               => ( "null".to_string(), "null".to_string() ),
    };
    let entry = match &aq.result
    {
      Ok( data ) =>
      {
        // Helpers: Option<f64> utilization → "{:.0}" percent or "null";
        //          Option<&str> ISO reset  → seconds-until-reset or "null".
        let pct_str   = |util : Option< f64 >| -> String
        {
          util.map_or_else( || "null".to_string(), |u| format!( "{:.0}", 100.0 - u ) )
        };
        let reset_str = |iso : Option< &str >| -> String
        {
          iso.and_then( claude_quota::iso_to_unix_secs )
            .map_or_else( || "null".to_string(), |t| t.saturating_sub( now_secs ).to_string() )
        };
        let session_pct   = pct_str( data.five_hour.as_ref().map( |p| p.utilization ) );
        let session_reset = reset_str( data.five_hour.as_ref().and_then( |p| p.resets_at.as_deref() ) );
        let weekly_pct    = pct_str( data.seven_day.as_ref().map( |p| p.utilization ) );
        let sonnet_pct    = pct_str( data.seven_day_sonnet.as_ref().map( |p| p.utilization ) );
        let weekly_reset  = reset_str( data.seven_day.as_ref().and_then( |p| p.resets_at.as_deref() ) );
        let seven_reset_secs = data.seven_day.as_ref().and_then( |p| p.resets_at.as_deref() )
          .and_then( claude_quota::iso_to_unix_secs )
          .map( |t| t.saturating_sub( now_secs ) );
        let ( next_type_str, next_secs_str ) = match next_event_raw(
          seven_reset_secs,
          ren_pair.map( |( s, _ )| s ),
          ren_pair.is_some_and( |( _, est )| est ),
        )
        {
          None                        => ( "null".to_string(), "null".to_string() ),
          Some( ( secs, prefix, _ ) ) =>
            ( format!( "\"{}\"", prefix.trim_start_matches( '+' ).trim_start_matches( '$' ) ),
              secs.to_string() ),
        };
        // Fix(BUG-335): Ok-branch JSON never surfaced the cache-fallback failure reason.
        //   Root cause: AccountQuota had no field to carry the reason forward from fetch.rs's
        //   Err→Ok cache-fallback conversion; render_json had nothing to emit.
        //   Pitfall: use shorten_error() here (matching AC-14), unlike the Err-branch "error"
        //   field below which intentionally emits the raw reason via json_escape() alone.
        let fallback_reason_json = aq.fallback_reason.as_deref()
          .map_or_else( || "null".to_string(), |r| format!( "\"{}\"", json_escape( shorten_error( r ) ) ) );
        format!(
          "{{\"account\":\"{name_esc}\",\"is_current\":{is_current_str},\"is_active\":{is_active_str},\
\"is_occupied_elsewhere\":{is_occupied_elsewhere_str},\"is_owned\":{is_owned_str},\
\"owner\":\"{owner_esc}\",\"expires_in_secs\":{expires_in_secs},\
\"billing_type\":{billing_type_str},\"has_max\":{has_max_str},\
\"renewal_secs\":{renewal_secs_str},\"renewal_is_estimate\":{renewal_is_estimate_str},\
\"next_event_type\":{next_type_str},\"next_event_secs\":{next_secs_str},\
\"session_5h_left_pct\":{session_pct},\"session_5h_resets_in_secs\":{session_reset},\
\"weekly_7d_left_pct\":{weekly_pct},\"weekly_7d_sonnet_left_pct\":{sonnet_pct},\
\"weekly_7d_resets_in_secs\":{weekly_reset},\"fallback_reason\":{fallback_reason_json},{cached_json}}}",
          cached_json = cache_json_fields( aq.cached, aq.cache_age_secs ),
        )
      }
      Err( reason ) =>
      {
        // Err accounts lack quota data but still have optional renewal;
        // compute next_event from that source so JSON callers get useful data.
        let ( next_type_str, next_secs_str ) = match next_event_raw(
          None,
          ren_pair.map( |( s, _ )| s ),
          ren_pair.is_some_and( |( _, est )| est ),
        )
        {
          None                         => ( "null".to_string(), "null".to_string() ),
          Some( ( secs, prefix, _ ) ) =>
            ( format!( "\"{}\"", prefix.trim_start_matches( '+' ).trim_start_matches( '$' ) ),
              secs.to_string() ),
        };
        format!(
          "{{\"account\":\"{name_esc}\",\"is_current\":{is_current_str},\"is_active\":{is_active_str},\
\"is_occupied_elsewhere\":{is_occupied_elsewhere_str},\"is_owned\":{is_owned_str},\
\"owner\":\"{owner_esc}\",\"expires_in_secs\":{expires_in_secs},\
\"billing_type\":{billing_type_str},\"has_max\":{has_max_str},\
\"renewal_secs\":{renewal_secs_str},\"renewal_is_estimate\":{renewal_is_estimate_str},\
\"next_event_type\":{next_type_str},\"next_event_secs\":{next_secs_str},\
\"error\":\"{}\",{cached_json}}}",
          json_escape( reason ),
          cached_json = cache_json_fields( aq.cached, aq.cache_age_secs ),
        )
      }
    };
    parts.push( entry );
  }

  format!( "[\n  {}\n]\n", parts.join( ",\n  " ) )
}
