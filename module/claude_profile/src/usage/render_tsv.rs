// Items are pub for test_bridge re-export; lints suppressed — internal API.
#![ allow( clippy::missing_inline_in_public_items, clippy::must_use_candidate, clippy::missing_panics_doc ) ]
//! TSV renderer for quota results.

use crate::output::format_duration_secs;
use super::types::{ AccountQuota, SortStrategy, PreferStrategy, ColsVisibility };
use super::format::{
  compute_expires_cell, sub_label, shorten_error,
  quota_text_cells, status_emoji, renews_label, next_event_label, renewal_secs,
};
use super::sort::sort_indices;

/// Render quota results as tab-separated values.
///
/// Status column uses plain-text labels (`ok`/`warn`/`err`). Percentage cells in
/// `5h Left` and `7d Left` are rendered without the emoji prefix. No tier grouping
/// or footer; rows are in sort strategy order. First row is a header.
#[ allow( clippy::too_many_lines ) ]
pub fn render_tsv(
  accounts : &[ AccountQuota ],
  sort     : SortStrategy,
  desc     : Option< bool >,
  prefer   : PreferStrategy,
  cols     : &ColsVisibility,
) -> String
{
  use std::time::{ SystemTime, UNIX_EPOCH };
  let now_secs = SystemTime::now()
    .duration_since( UNIX_EPOCH )
    .unwrap_or_default()
    .as_secs();

  // Build header.
  let mut headers = vec![ "flag".to_string() ];
  if cols.status       { headers.push( "status".to_string() ); }
  headers.push( "account".to_string() );
  if cols.h5_left      { headers.push( "5h_left".to_string() ); }
  if cols.h5_reset     { headers.push( "5h_reset".to_string() ); }
  if cols.d7_left      { headers.push( "7d_left".to_string() ); }
  if cols.d7_son       { headers.push( "7d_son".to_string() ); }
  if cols.d7_reset     { headers.push( "7d_reset".to_string() ); }
  if cols.d7_son_reset { headers.push( "7d_son_reset".to_string() ); }
  if cols.expires      { headers.push( "expires".to_string() ); }
  if cols.sub          { headers.push( "sub".to_string() ); }
  if cols.renews       { headers.push( "renews".to_string() ); }
  if cols.host         { headers.push( "host".to_string() ); }
  if cols.role         { headers.push( "role".to_string() ); }
  if cols.owner        { headers.push( "owner".to_string() ); }
  if cols.next         { headers.push( "next".to_string() ); }
  let mut out = headers.join( "\t" );
  out.push( '\n' );

  if accounts.is_empty() { return out; }

  let sorted_indices = sort_indices( accounts, sort, desc, prefer, now_secs );
  for idx in sorted_indices
  {
    let aq         = &accounts[ idx ];
    let flag_cell  = if aq.is_current { "\u{2713}" } else if aq.is_active { "*" } else if aq.is_occupied_elsewhere { "@" } else { "" };
    let status_str = match status_emoji( aq )
    {
      "🟢" => "ok",
      "🟡" => "warn",
      _    => "err",
    };
    let expires_str = compute_expires_cell( aq.expires_at_ms, now_secs );
    let sub_str     = sub_label( aq.account.as_ref() ).to_string();
    // Fix(BUG-232): billing_type=="none" → no active subscription → no renewal date to show.
    // Root cause: renews_label uses org_created_at unconditionally; has no billing_type param.
    // Pitfall: org_created_at may be present even when subscription is cancelled; must check
    //   billing_type BEFORE passing org_created_at to renews_label.
    let renews_str = if aq.is_no_subscription()
    {
      "\u{2014}".to_string()
    }
    else
    {
      renews_label(
        aq.renewal_at.as_deref(),
        aq.account.as_ref().map( |a| a.org_created_at.as_str() ),
        now_secs,
      )
    };

    let mut row = vec![ flag_cell.to_string() ];
    if cols.status { row.push( status_str.to_string() ); }
    // Fix(BUG-335): NAME cell discarded the cache-fallback failure reason entirely.
    //   Root cause: AccountQuota had no field to carry the reason forward from fetch.rs's
    //   Err→Ok cache-fallback conversion; TSV render had nothing to append.
    //   Pitfall: unlike render.rs's text table, this format has no pre-existing age-suffix
    //   mechanism to append alongside (AC-03 does not apply here) — the shortened reason is
    //   the cell's only staleness indicator; do not invent a new age label to pair it with.
    let name_cell = match &aq.fallback_reason
    {
      Some( reason ) => format!( "{} ({})", aq.name, shorten_error( reason ) ),
      None           => aq.name.clone(),
    };
    row.push( name_cell );

    match &aq.result
    {
      Ok( data ) =>
      {
        // Plain percentage cells (no emoji prefix).
        let dash     = "\u{2014}".to_string();
        let pct_bare = |util : Option< f64 >| -> String
        {
          util.map_or_else( || dash.clone(), |u| format!( "{:.0}%", 100.0 - u ) )
        };
        let cells = quota_text_cells( data, now_secs );
        // cells[0] = "🟢 88%" — strip emoji; use bare pct_bare instead.
        let h5_left_bare  = pct_bare( data.five_hour.as_ref().map( |p| p.utilization ) );
        let d7_left_bare  = pct_bare( data.seven_day.as_ref().map( |p| p.utilization ) );
        let d7_son_reset  = data.seven_day_sonnet.as_ref()
          .and_then( |p| p.resets_at.as_deref() )
          .and_then( claude_quota::iso_to_unix_secs )
          .map_or_else( || dash.clone(), |t| format!( "in {}", format_duration_secs( t.saturating_sub( now_secs ) ) ) );

        let ( ren_secs, ren_est ) = renewal_secs(
          aq.renewal_at.as_deref(),
          aq.account.as_ref().map( |a| a.org_created_at.as_str() ),
          now_secs,
        ).unzip();
        let next_str = next_event_label(
          data.seven_day.as_ref().and_then( |p| p.resets_at.as_deref() )
            .and_then( claude_quota::iso_to_unix_secs )
            .map( |t| t.saturating_sub( now_secs ) ),
          ren_secs,
          ren_est.unwrap_or( false ),
        );
        if cols.h5_left      { row.push( h5_left_bare ); }
        if cols.h5_reset     { row.push( cells[ 1 ].clone() ); }
        if cols.d7_left      { row.push( d7_left_bare ); }
        if cols.d7_son       { row.push( cells[ 3 ].clone() ); }
        if cols.d7_reset     { row.push( cells[ 4 ].clone() ); }
        if cols.d7_son_reset { row.push( d7_son_reset ); }
        if cols.expires      { row.push( expires_str ); }
        if cols.sub          { row.push( sub_str ); }
        if cols.renews       { row.push( renews_str ); }
        if cols.host         { row.push( aq.host.clone() ); }
        if cols.role         { row.push( aq.role.clone() ); }
        if cols.owner        { row.push( aq.owner.clone() ); }
        if cols.next         { row.push( next_str ); }
      }
      Err( reason ) =>
      {
        let dash      = "\u{2014}".to_string();
        let error_str = format!( "({})", shorten_error( reason ) );
        let col_count = [ cols.h5_left, cols.h5_reset, cols.d7_left, cols.d7_son,
                          cols.d7_reset, cols.d7_son_reset ].iter().filter( |&&b| b ).count();
        for _ in 0..col_count { row.push( dash.clone() ); }
        // Fix(BUG-220): replace last quota-dash with error_str (last visible quota column carries
        //   the error reason); renews cell must push renews_str, not error_str.
        // Root cause: explicit error_str push for renews cell — same incorrect scope as Site 1.
        // Pitfall: only replace when col_count > 0 (at least one quota col visible).
        if col_count > 0 { *row.last_mut().unwrap() = error_str; }
        if cols.expires { row.push( expires_str ); }
        if cols.sub     { row.push( sub_str ); }
        if cols.renews  { row.push( renews_str ); }  // Fix: was error_str
        if cols.host    { row.push( aq.host.clone() ); }
        if cols.role    { row.push( aq.role.clone() ); }
        if cols.owner   { row.push( aq.owner.clone() ); }
        if cols.next    { row.push( "\u{2014}".to_string() ); }
      }
    }

    out.push_str( &row.join( "\t" ) );
    out.push( '\n' );
  }

  out
}
