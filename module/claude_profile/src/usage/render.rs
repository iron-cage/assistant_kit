//! Table and JSON renderers for quota results.
//!
//! `render_text` produces the human-readable `data_fmt` table; `render_json`
//! produces a JSON array. Both are consumed by `api.rs::usage_routine`.

use data_fmt::{ RowBuilder, TableFormatter, Format };
use crate::output::{ format_duration_secs, json_escape };
use super::types::{ AccountQuota, SortStrategy, PreferStrategy, NextStrategy, ColsVisibility };
use super::format::{
  compute_expires_cell, sub_label, next_billing_label, shorten_error,
  quota_text_cells, status_emoji,
};
use super::sort::{ sort_indices, find_next_for_strategy, strategy_metric };

// ── Text renderer ─────────────────────────────────────────────────────────────

/// Render quota results as a plain-text table using `data_fmt`.
///
/// Empty store renders `(no accounts configured)` without a table.
/// Column visibility is controlled by `cols` (structural `flag` and `account`
/// columns are always shown). Footer (TSK-184): always-visible 2-strategy block
/// when ≥2 accounts have valid quota — shows `endurance` and `drain` lines.
/// The `→` marker in the table body points to the active-strategy winner.
/// Footer is omitted when < 2 accounts have valid quota data.
#[ allow( clippy::too_many_lines ) ]
pub( crate ) fn render_text(
  accounts : &[ AccountQuota ],
  sort     : SortStrategy,
  desc     : Option< bool >,
  prefer   : PreferStrategy,
  next     : NextStrategy,
  cols     : &ColsVisibility,
) -> String
{
  use std::time::{ SystemTime, UNIX_EPOCH };

  if accounts.is_empty()
  {
    return "Quota\n\n  (no accounts configured)\n".to_string();
  }

  let now_secs = SystemTime::now()
    .duration_since( UNIX_EPOCH )
    .unwrap_or_default()
    .as_secs();

  // Compute the winner for the active strategy; placed as → marker in the table body.
  let best_idx       = find_next_for_strategy( accounts, next, prefer, now_secs );
  let sorted_indices = sort_indices( accounts, sort, desc, prefer, now_secs );

  // Three-tier grouping: sort order preserved within each tier (🟢 → 🟡 → 🔴).
  // Applied after the sort strategy so each tier's internal order reflects the chosen sort.
  // AC-26: within 🟡, session-exhausted (5h Left ≤ 15%) precedes weekly-exhausted.
  // Accounts where both 5h Left ≤ 15% AND 7d Left ≤ 5% fall in the session-exhausted sub-group.
  let ( mut green_indices, mut red_indices ) = ( Vec::new(), Vec::new() );
  let ( mut session_yellow, mut weekly_yellow ) = ( Vec::new(), Vec::new() );
  for idx in sorted_indices
  {
    match status_emoji( &accounts[ idx ].result )
    {
      "🟢" => green_indices.push( idx ),
      "🟡" =>
      {
        let h5_left = if let Ok( data ) = &accounts[ idx ].result
        {
          100.0 - data.five_hour.as_ref().map_or( 0.0, |p| p.utilization )
        }
        else { 100.0 };
        if h5_left <= 15.0 { session_yellow.push( idx ); }
        else               { weekly_yellow.push( idx ); }
      }
      _    => red_indices.push( idx ),
    }
  }
  let sorted_indices : Vec< usize > = green_indices.into_iter()
    .chain( session_yellow )
    .chain( weekly_yellow )
    .chain( red_indices )
    .collect();

  // Build headers conditionally — structural cols always first and always visible.
  let mut headers = vec![ String::new() ]; // flag col
  if cols.status       { headers.push( "●".to_string() ); }
  headers.push( "Account".to_string() ); // account name — structural
  if cols.h5_left      { headers.push( "5h Left".to_string() ); }
  if cols.h5_reset     { headers.push( "5h Reset".to_string() ); }
  if cols.d7_left      { headers.push( "7d Left".to_string() ); }
  if cols.d7_son       { headers.push( "7d(Son)".to_string() ); }
  if cols.d7_reset     { headers.push( "7d Reset".to_string() ); }
  if cols.d7_son_reset { headers.push( "7d Son Reset".to_string() ); }
  if cols.expires      { headers.push( "Expires".to_string() ); }
  if cols.sub          { headers.push( "Sub".to_string() ); }
  if cols.renews       { headers.push( "~Renews".to_string() ); }

  let mut builder = RowBuilder::new( headers );
  for orig_idx in sorted_indices.iter().copied()
  {
    let aq = &accounts[ orig_idx ];
    // Four-level priority: ✓ (is_current) > * (is_active, not current) > → (active-strategy winner) > blank.
    let flag_cell = if aq.is_current
    {
      "✓".to_string()
    }
    else if aq.is_active
    {
      "*".to_string()
    }
    else if best_idx == Some( orig_idx )
    {
      "→".to_string()
    }
    else
    {
      String::new()
    };

    let expires_cell = compute_expires_cell( aq.expires_at_ms, now_secs );
    let sub_str      = sub_label( aq.account.as_ref() ).to_string();
    let renews_str   = aq.account.as_ref()
      .map_or_else( || "?".to_string(), |a| next_billing_label( &a.org_created_at, now_secs ) );

    match &aq.result
    {
      Ok( data ) =>
      {
        let cells        = quota_text_cells( data, now_secs );
        let son_reset    = data.seven_day_sonnet.as_ref()
          .and_then( |p| p.resets_at.as_deref() )
          .and_then( claude_quota::iso_to_unix_secs )
          .map_or_else(
            || "\u{2014}".to_string(),
            |t| format!( "in {}", format_duration_secs( t.saturating_sub( now_secs ) ) ),
          );

        let mut row : Vec< String > = vec![ flag_cell ];
        if cols.status       { row.push( status_emoji( &aq.result ).to_string() ); }
        row.push( aq.name.clone() );
        if cols.h5_left      { row.push( cells[ 0 ].clone() ); }
        if cols.h5_reset     { row.push( cells[ 1 ].clone() ); }
        if cols.d7_left      { row.push( cells[ 2 ].clone() ); }
        if cols.d7_son       { row.push( cells[ 3 ].clone() ); }
        if cols.d7_reset     { row.push( cells[ 4 ].clone() ); }
        if cols.d7_son_reset { row.push( son_reset ); }
        if cols.expires      { row.push( expires_cell ); }
        if cols.sub          { row.push( sub_str ); }
        if cols.renews       { row.push( renews_str ); }
        builder = builder.add_row( row.into_iter().map( Into::into ).collect() );
      }
      Err( reason ) =>
      {
        let dash      = "\u{2014}".to_string();
        let error_str = format!( "({})", shorten_error( reason ) );

        let mut row : Vec< String > = vec![ flag_cell ];
        if cols.status       { row.push( status_emoji( &aq.result ).to_string() ); }
        row.push( aq.name.clone() );
        let structural_len = row.len();
        if cols.h5_left      { row.push( dash.clone() ); }
        if cols.h5_reset     { row.push( dash.clone() ); }
        if cols.d7_left      { row.push( dash.clone() ); }
        if cols.d7_son       { row.push( dash.clone() ); }
        if cols.d7_reset     { row.push( dash.clone() ); }
        if cols.d7_son_reset { row.push( dash.clone() ); }
        if cols.expires      { row.push( expires_cell ); }
        if cols.sub          { row.push( sub_str ); }
        if cols.renews       { row.push( renews_str ); }
        // Error reason replaces the last visible non-structural column (009 AC-03).
        if row.len() > structural_len
        {
          *row.last_mut().unwrap() = error_str;
        }
        builder = builder.add_row( row.into_iter().map( Into::into ).collect() );
      }
    }
  }

  let view  = builder.build_view();
  let table = Format::format( &TableFormatter::new(), &view ).unwrap_or_default();
  let body  = format!( "Quota\n\n{table}\n" );

  // Footer: shown only when ≥2 valid accounts (AC-09 from 023_next_account_strategies.md).
  let valid_count = accounts.iter().filter( |aq| aq.result.is_ok() ).count();
  if valid_count < 2 { return body; }

  // Responsibility(TSK-184-footer): unconditional 2-strategy footer (Endurance, Drain).
  // Both lines shown when valid_count >= 2; NOT gated on next:: value.
  // The → marker in the table body is already placed on the active-strategy winner.
  {
    use core::fmt::Write as _;
    let strategies = [ NextStrategy::Endurance, NextStrategy::Drain ];
    let names      = [ "endurance", "drain" ];
    let mut lines  = String::new();
    for ( strategy, name ) in strategies.iter().zip( names.iter() )
    {
      if let Some( idx ) = find_next_for_strategy( accounts, *strategy, prefer, now_secs )
      {
        let rec      = &accounts[ idx ];
        let metric   = strategy_metric( rec, *strategy, prefer, now_secs );
        let rec_name = &rec.name;
        let _ = writeln!( lines, "  {name:<10}{rec_name}   {metric}" );
      }
    }
    if lines.is_empty() { return body; }
    let total = accounts.len();
    format!( "{body}Valid: {valid_count} / {total}   ->  Next by strategy:\n{lines}" )
  }
}

// ── JSON renderer ─────────────────────────────────────────────────────────────

/// Render quota results as a JSON array (one object per account).
///
/// Every row includes `expires_in_secs`. Successful accounts include quota
/// fields using `_left_pct` naming (remaining, not consumed); failed accounts
/// include `error`.
pub( crate ) fn render_json( accounts : &[ AccountQuota ] ) -> String
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
    let is_current_str   = if aq.is_current { "true" } else { "false" };
    let is_active_str    = if aq.is_active  { "true" } else { "false" };
    let expires_in_secs  = ( aq.expires_at_ms / 1000 ).saturating_sub( now_secs );
    let billing_type_str = aq.account.as_ref()
      .map_or_else( || "null".to_string(), |a| format!( "\"{}\"", json_escape( &a.billing_type ) ) );
    let has_max_str      = aq.account.as_ref()
      .map_or( "null", |a| if a.has_max { "true" } else { "false" } );
    let next_renewal_str = aq.account.as_ref()
      .map_or_else( || "null".to_string(), |a| format!( "\"{}\"", json_escape( &next_billing_label( &a.org_created_at, now_secs ) ) ) );
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
        format!(
          "{{\"account\":\"{name_esc}\",\"is_current\":{is_current_str},\"is_active\":{is_active_str},\
\"expires_in_secs\":{expires_in_secs},\
\"billing_type\":{billing_type_str},\"has_max\":{has_max_str},\"next_renewal_est\":{next_renewal_str},\
\"session_5h_left_pct\":{session_pct},\"session_5h_resets_in_secs\":{session_reset},\
\"weekly_7d_left_pct\":{weekly_pct},\"weekly_7d_sonnet_left_pct\":{sonnet_pct},\
\"weekly_7d_resets_in_secs\":{weekly_reset}}}",
        )
      }
      Err( reason ) =>
      {
        format!(
          "{{\"account\":\"{name_esc}\",\"is_current\":{is_current_str},\"is_active\":{is_active_str},\
\"expires_in_secs\":{expires_in_secs},\
\"billing_type\":{billing_type_str},\"has_max\":{has_max_str},\"next_renewal_est\":{next_renewal_str},\
\"error\":\"{}\"}}",
          json_escape( reason ),
        )
      }
    };
    parts.push( entry );
  }

  format!( "[\n  {}\n]\n", parts.join( ",\n  " ) )
}
