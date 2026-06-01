//! Table and JSON renderers for quota results.
//!
//! `render_text` produces the human-readable `data_fmt` table; `render_json`
//! produces a JSON array; `render_tsv` produces tab-separated output; `render_plain`
//! produces a no-emoji version of the text table; `extract_get_field` extracts one
//! column value as a bare string. All consumed by `api.rs::usage_routine`.

use data_fmt::{ RowBuilder, TableFormatter, TableConfig, Format };
use crate::output::{ format_duration_secs, json_escape };
use super::types::{ AccountQuota, SortStrategy, PreferStrategy, NextStrategy, ColsVisibility, GetField };
use super::format::{
  compute_expires_cell, sub_label, shorten_error,
  quota_text_cells, status_emoji, renews_label, next_event_label, next_event_raw, renewal_secs,
};
use super::sort::{ sort_indices, find_next_for_strategy, strategy_metric };

// ── Text renderer ─────────────────────────────────────────────────────────────

/// Render quota results as a plain-text table using `data_fmt`.
///
/// Empty store renders `(no accounts configured)` without a table.
/// Column visibility is controlled by `cols` (structural `flag` and `account`
/// columns are always shown). Footer (TSK-184): unconditional 3-strategy block
/// when ≥2 accounts have valid quota — shows `renew`, `endurance`, and `drain` lines.
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
  if cols.host         { headers.push( "Host".to_string() ); }
  if cols.role         { headers.push( "Role".to_string() ); }
  if cols.next         { headers.push( "\u{2192} Next".to_string() ); }

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
    // Fix(BUG-227): pass token expiry as a candidate for → Next; None when expiry unknown (== 0).
    // Root cause: TSK-228 removed token expiry from next_event_raw(), causing the column to report
    //   a quota event days away even when the token expires in minutes.
    // Pitfall: expires_at_ms==0 means unknown (fetch failed); pass None, not Some(0).
    let tok_secs     = if aq.expires_at_ms == 0 { None }
                       else { Some( ( aq.expires_at_ms / 1000 ).saturating_sub( now_secs ) ) };
    let sub_str      = sub_label( aq.account.as_ref() ).to_string();
    let renews_str   = renews_label(
      aq.renewal_at.as_deref(),
      aq.account.as_ref().map( |a| a.org_created_at.as_str() ),
      now_secs,
    );

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
        let ( ren_secs, ren_est ) = renewal_secs(
          aq.renewal_at.as_deref(),
          aq.account.as_ref().map( |a| a.org_created_at.as_str() ),
          now_secs,
        ).unzip();
        let next_cell    = next_event_label(
          data.seven_day.as_ref().and_then( |p| p.resets_at.as_deref() )
            .and_then( claude_quota::iso_to_unix_secs )
            .map( |t| t.saturating_sub( now_secs ) ),
          ren_secs,
          ren_est.unwrap_or( false ),
          tok_secs,
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
        if cols.host         { row.push( aq.host.clone() ); }
        if cols.role         { row.push( aq.role.clone() ); }
        if cols.next         { row.push( next_cell ); }
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
        let quota_end_len = row.len();
        if cols.expires      { row.push( expires_cell ); }
        if cols.sub          { row.push( sub_str ); }
        if cols.renews       { row.push( renews_str ); }
        if cols.host         { row.push( aq.host.clone() ); }
        if cols.role         { row.push( aq.role.clone() ); }
        if cols.next         { row.push( "\u{2014}".to_string() ); }
        // Fix(BUG-220): only the last visible quota-data column carries error_str — non-quota
        //   metadata columns (expires, sub, renews) are sourced independently and must be preserved.
        // Root cause: positional last_mut() targeted ~Renews after BUG-180 moved it to trail quota
        //   columns; AC-03 "last visible column" intent was "last quota column", not "last of all".
        // Pitfall: quota_end_len == structural_len when all quota cols are hidden — skip override.
        if quota_end_len > structural_len
        {
          row[ quota_end_len - 1 ] = error_str;
        }
        builder = builder.add_row( row.into_iter().map( Into::into ).collect() );
      }
    }
  }

  let view  = builder.build_view();
  let table = Format::format( &TableFormatter::with_config( TableConfig::default().auto_wrap( false ) ), &view ).unwrap_or_default();
  let body  = format!( "Quota\n\n{table}\n" );

  // Footer: shown only when ≥2 valid accounts (AC-09 from 023_next_account_strategies.md).
  let valid_count = accounts.iter().filter( |aq| aq.result.is_ok() ).count();
  if valid_count < 2 { return body; }

  // Responsibility(TSK-184-footer): unconditional 3-strategy footer (Renew, Endurance, Drain).
  // All three lines shown when valid_count >= 2; NOT gated on next:: value.
  // The → marker in the table body is already placed on the active-strategy winner.
  {
    use core::fmt::Write as _;
    let strategies = [ NextStrategy::Renew, NextStrategy::Endurance, NextStrategy::Drain ];
    let names      = [ "renew", "endurance", "drain" ];
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
#[ allow( clippy::too_many_lines ) ]
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
    let tok_secs_json    = if aq.expires_at_ms == 0 { None } else { Some( expires_in_secs ) };
    let billing_type_str = aq.account.as_ref()
      .map_or_else( || "null".to_string(), |a| format!( "\"{}\"", json_escape( &a.billing_type ) ) );
    let has_max_str      = aq.account.as_ref()
      .map_or( "null", |a| if a.has_max { "true" } else { "false" } );
    let ren_pair                                       = renewal_secs(
      aq.renewal_at.as_deref(),
      aq.account.as_ref().map( |a| a.org_created_at.as_str() ),
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
          tok_secs_json,
        )
        {
          None                        => ( "null".to_string(), "null".to_string() ),
          Some( ( secs, prefix, _ ) ) =>
            ( format!( "\"{}\"", prefix.trim_start_matches( '+' ).trim_start_matches( '!' ).trim_start_matches( '$' ) ),
              secs.to_string() ),
        };
        format!(
          "{{\"account\":\"{name_esc}\",\"is_current\":{is_current_str},\"is_active\":{is_active_str},\
\"expires_in_secs\":{expires_in_secs},\
\"billing_type\":{billing_type_str},\"has_max\":{has_max_str},\
\"renewal_secs\":{renewal_secs_str},\"renewal_is_estimate\":{renewal_is_estimate_str},\
\"next_event_type\":{next_type_str},\"next_event_secs\":{next_secs_str},\
\"session_5h_left_pct\":{session_pct},\"session_5h_resets_in_secs\":{session_reset},\
\"weekly_7d_left_pct\":{weekly_pct},\"weekly_7d_sonnet_left_pct\":{sonnet_pct},\
\"weekly_7d_resets_in_secs\":{weekly_reset}}}",
        )
      }
      Err( reason ) =>
      {
        // Err accounts lack quota data but still have token-expiry and optional renewal;
        // compute next_event from those two sources so JSON callers get useful data.
        let ( next_type_str, next_secs_str ) = match next_event_raw(
          None,
          ren_pair.map( |( s, _ )| s ),
          ren_pair.is_some_and( |( _, est )| est ),
          tok_secs_json,
        )
        {
          None                         => ( "null".to_string(), "null".to_string() ),
          Some( ( secs, prefix, _ ) ) =>
            ( format!( "\"{}\"", prefix.trim_start_matches( '+' ).trim_start_matches( '!' ).trim_start_matches( '$' ) ),
              secs.to_string() ),
        };
        format!(
          "{{\"account\":\"{name_esc}\",\"is_current\":{is_current_str},\"is_active\":{is_active_str},\
\"expires_in_secs\":{expires_in_secs},\
\"billing_type\":{billing_type_str},\"has_max\":{has_max_str},\
\"renewal_secs\":{renewal_secs_str},\"renewal_is_estimate\":{renewal_is_estimate_str},\
\"next_event_type\":{next_type_str},\"next_event_secs\":{next_secs_str},\
\"error\":\"{}\"}}",
          json_escape( reason ),
        )
      }
    };
    parts.push( entry );
  }

  format!( "[\n  {}\n]\n", parts.join( ",\n  " ) )
}

// ── TSV renderer ───────────────────────────────────────────────────────────────

/// Render quota results as tab-separated values.
///
/// Status column uses plain-text labels (`ok`/`warn`/`err`). Percentage cells in
/// `5h Left` and `7d Left` are rendered without the emoji prefix. No tier grouping
/// or footer; rows are in sort strategy order. First row is a header.
#[ allow( clippy::too_many_lines ) ]
pub( crate ) fn render_tsv(
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
  if cols.next         { headers.push( "next".to_string() ); }
  let mut out = headers.join( "\t" );
  out.push( '\n' );

  if accounts.is_empty() { return out; }

  let sorted_indices = sort_indices( accounts, sort, desc, prefer, now_secs );
  for idx in sorted_indices
  {
    let aq         = &accounts[ idx ];
    let flag_cell  = if aq.is_current { "\u{2713}" } else if aq.is_active { "*" } else { "" };
    let status_str = match status_emoji( &aq.result )
    {
      "🟢" => "ok",
      "🟡" => "warn",
      _    => "err",
    };
    let expires_str = compute_expires_cell( aq.expires_at_ms, now_secs );
    let tok_secs    = if aq.expires_at_ms == 0 { None }
                      else { Some( ( aq.expires_at_ms / 1000 ).saturating_sub( now_secs ) ) };
    let sub_str     = sub_label( aq.account.as_ref() ).to_string();
    let renews_str  = renews_label(
      aq.renewal_at.as_deref(),
      aq.account.as_ref().map( |a| a.org_created_at.as_str() ),
      now_secs,
    );

    let mut row = vec![ flag_cell.to_string() ];
    if cols.status { row.push( status_str.to_string() ); }
    row.push( aq.name.clone() );

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
          tok_secs,
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
        if cols.next    { row.push( "\u{2014}".to_string() ); }
      }
    }

    out.push_str( &row.join( "\t" ) );
    out.push( '\n' );
  }

  out
}

// ── Plain-text renderer ────────────────────────────────────────────────────────

/// Render quota results as plain text (same as `render_text` with emoji replaced).
///
/// `🟢`→`ok`, `🟡`→`warn`, `🔴`→`err`, `→`→`->`, `✓`→`*`.
pub( crate ) fn render_plain(
  accounts : &[ AccountQuota ],
  sort     : SortStrategy,
  desc     : Option< bool >,
  prefer   : PreferStrategy,
  next     : NextStrategy,
  cols     : &ColsVisibility,
) -> String
{
  let raw = render_text( accounts, sort, desc, prefer, next, cols );
  raw
    .replace( "🟢", "ok" )
    .replace( "🟡", "warn" )
    .replace( "🔴", "err" )
    .replace( '→', "->" )
    .replace( '✓', "*" )
}

// ── Field extractor ────────────────────────────────────────────────────────────

/// Extract the value of one column for `aq` as a bare string with no table chrome.
///
/// The returned string is the same value that would appear in the corresponding
/// cell of the text table — but without trailing whitespace or ANSI sequences.
/// `host` and `role` return the values from `{name}.profile.json`, empty when absent.
pub( crate ) fn extract_get_field( aq : &AccountQuota, field : GetField, now_secs : u64 ) -> String
{
  let dash = "\u{2014}".to_string();
  match field
  {
    GetField::Status  => status_emoji( &aq.result ).to_string(),
    GetField::Account => aq.name.clone(),
    GetField::Expires => compute_expires_cell( aq.expires_at_ms, now_secs ),
    GetField::Sub    => sub_label( aq.account.as_ref() ).to_string(),
    GetField::Renews => renews_label(
      aq.renewal_at.as_deref(),
      aq.account.as_ref().map( |a| a.org_created_at.as_str() ),
      now_secs,
    ),
    GetField::Host         => aq.host.clone(),
    GetField::Role         => aq.role.clone(),
    GetField::NextEventType | GetField::NextEventSecs =>
    {
      if let Ok( data ) = &aq.result
      {
        let seven_reset = data.seven_day.as_ref().and_then( |p| p.resets_at.as_deref() )
          .and_then( claude_quota::iso_to_unix_secs ).map( |t| t.saturating_sub( now_secs ) );
        let ren_pair = renewal_secs(
          aq.renewal_at.as_deref(),
          aq.account.as_ref().map( |a| a.org_created_at.as_str() ),
          now_secs,
        );
        // Fix(BUG-227): pass token expiry as a candidate for next event
        let tok_secs = if aq.expires_at_ms == 0 { None }
                       else { Some( ( aq.expires_at_ms / 1000 ).saturating_sub( now_secs ) ) };
        match next_event_raw(
          seven_reset,
          ren_pair.map( |( s, _ )| s ),
          ren_pair.is_some_and( |( _, est )| est ),
          tok_secs,
        )
        {
          None => dash,
          Some( ( secs, prefix, _ ) ) => match field
          {
            GetField::NextEventType => prefix.to_string(),
            _                       => secs.to_string(),
          },
        }
      }
      else
      {
        dash
      }
    }
    _ =>
    {
      let Ok( data ) = &aq.result else { return dash; };
      let pct_bare = |util : Option< f64 >| -> String
      {
        util.map_or_else( || dash.clone(), |u| format!( "{:.0}%", 100.0 - u ) )
      };
      let reset_cell = |iso : Option< &str >| -> String
      {
        iso.and_then( claude_quota::iso_to_unix_secs )
          .map_or_else( || dash.clone(), |t|
            format!( "in {}", format_duration_secs( t.saturating_sub( now_secs ) ) )
          )
      };
      match field
      {
        GetField::FiveHourLeft  => pct_bare( data.five_hour.as_ref().map( |p| p.utilization ) ),
        GetField::FiveHourReset => reset_cell( data.five_hour.as_ref().and_then( |p| p.resets_at.as_deref() ) ),
        GetField::SevenDayLeft  => pct_bare( data.seven_day.as_ref().map( |p| p.utilization ) ),
        GetField::SevenDaySon   => pct_bare( data.seven_day_sonnet.as_ref().map( |p| p.utilization ) ),
        GetField::SevenDayReset => reset_cell( data.seven_day.as_ref().and_then( |p| p.resets_at.as_deref() ) ),
        _ => dash,
      }
    }
  }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[ cfg( test ) ]
mod tests
{
  use super::{ render_text, render_tsv };
  use crate::usage::types::{ AccountQuota, SortStrategy, PreferStrategy, NextStrategy, ColsVisibility };
  use crate::usage::test_support::FAR_FUTURE_MS;

  /// FT-20/009 — `~Renews` must retain billing date (not error reason) for 429-errored accounts.
  ///
  /// # Root Cause
  /// `render_text()` Err arm used `*row.last_mut().unwrap() = error_str` (positional blind
  /// overwrite). Under default layout (`host`/`role` OFF), `~Renews` was the last pushed
  /// column, so the billing date was discarded and replaced with the quota API error reason.
  /// `render_tsv()` Err arm explicitly pushed `error_str` for the renews cell, same effect.
  ///
  /// # Why Not Caught
  /// All prior Err-arm tests used `account: None` (→ `renews_str = "?"`), so the overwrite
  /// was invisible — both the buggy value and the intended value were "a non-date string".
  /// No test combined `account: Some(OauthAccountData { ... })` with `result: Err(...)`.
  ///
  /// # Fix Applied
  /// `render_text()`: replaced `last_mut()` with `row[ quota_end_len - 1 ] = error_str`
  /// (targets only the last visible quota-data column; `~Renews` is outside that range).
  /// `render_tsv()`: push `col_count - 1` dashes then push `error_str` directly as the last
  /// quota entry; renews cell changed from `error_str` to `renews_str`.
  ///
  /// # Prevention
  /// Any Err-arm render test covering 429/401/403 accounts must supply `account: Some(...)` and
  /// assert that the renews cell retains a billing date, not the error reason.
  ///
  /// # Pitfall
  /// `mk_aq_err()` sets `account: None` → `renews_str = "?"` → the assertion
  /// `!= "?"` would pass even with the bug present. Always construct the struct literal
  /// directly when testing Err-arm behavior with `OauthAccountData` present.
  ///
  /// Spec: [`tests/docs/feature/009_token_usage.md` FT-20]
  #[ doc = "bug_reproducer(BUG-220)" ]
  #[ test ]
  fn mre_bug_220_renews_preserved_for_429_accounts()
  {
    let aq = AccountQuota
    {
      name          : "i11@test.com".to_string(),
      is_current    : false,
      is_active     : false,
      expires_at_ms : FAR_FUTURE_MS,
      result        : Err( "rate limited (429)".to_string() ),
      account       : Some( claude_quota::OauthAccountData
      {
        billing_type   : "stripe_subscription".to_string(),
        has_max        : true,
        org_created_at : "1970-01-15T00:00:00Z".to_string(),
      }),
      host          : String::new(),
      role          : String::new(),
      renewal_at    : None,
    };
    let accounts = vec![ aq ];
    let cols     = ColsVisibility::default_set();

    // TSV: `renews` column must hold the billing date — NOT the error reason.
    let tsv        = render_tsv( &accounts, SortStrategy::Name, None, PreferStrategy::Any, &cols );
    let mut lines  = tsv.lines();
    let header     = lines.next().expect( "TSV must have a header row" );
    let data_row   = lines.next().expect( "TSV must have a data row" );
    let headers    : Vec< &str > = header.split( '\t' ).collect();
    let fields     : Vec< &str > = data_row.split( '\t' ).collect();
    let renews_idx = headers.iter().position( |h| *h == "renews" )
      .expect( "renews column must be present in TSV header" );
    let renews_val = fields.get( renews_idx ).copied().unwrap_or( "" );

    assert_ne!(
      renews_val,
      "(rate limited (429))",
      "BUG-220: TSV ~Renews must not contain error_str for 429 accounts with valid \
       OauthAccountData; got {renews_val:?}",
    );
    assert_ne!(
      renews_val,
      "?",
      "BUG-220: TSV ~Renews must show billing date when OauthAccountData is available; \
       got {renews_val:?}",
    );

    // Text renderer: the error reason must appear somewhere in output (in a quota column).
    let text = render_text(
      &accounts, SortStrategy::Name, None, PreferStrategy::Any, NextStrategy::Endurance, &cols,
    );
    assert!(
      text.contains( "(rate limited (429))" ),
      "BUG-220: error reason must appear in render_text output (in a quota column)",
    );
  }
}
