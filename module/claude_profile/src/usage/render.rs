//! Table and JSON renderers for quota results.
//!
//! `render_text` produces the human-readable `data_fmt` table; `render_json`
//! produces a JSON array; `render_tsv` produces tab-separated output; `render_plain`
//! produces a no-emoji version of the text table; `extract_get_field` extracts one
//! column value as a bare string. All consumed by `api.rs::usage_routine`.

use data_fmt::{ RowBuilder, TableFormatter, TableConfig, Format };
use crate::output::{ format_duration_secs, json_escape };
use super::types::{ AccountQuota, SortStrategy, PreferStrategy, ColsVisibility, GetField };
use super::format::{
  recommended_model,
  compute_expires_cell, sub_label, shorten_error,
  quota_text_cells, status_emoji, renews_label, next_event_label, next_event_raw, renewal_secs,
};
use super::sort::{ sort_indices, find_next_for_strategy, strategy_metric };

// ── Text renderer ─────────────────────────────────────────────────────────────

/// Render quota results as a plain-text table using `data_fmt`.
///
/// Empty store renders `(no accounts configured)` without a table.
/// Column visibility is controlled by `cols` (structural `flag` and `account`
/// columns are always shown). Footer: single-strategy recommendation line when
/// ≥2 accounts have valid quota — shows the winner for the active `sort::`.
/// Footer is omitted when < 2 accounts have valid quota data.
#[ allow( clippy::too_many_lines, clippy::too_many_arguments ) ]
pub( crate ) fn render_text(
  accounts       : &[ AccountQuota ],
  sort           : SortStrategy,
  desc           : Option< bool >,
  prefer         : PreferStrategy,
  cols           : &ColsVisibility,
  session_model  : Option< &str >,
  session_effort : Option< &str >,
  store_path     : Option< &std::path::Path >,
  who            : Option< bool >,
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

  // `sort_indices` applies the 4-group status partition (AC-12):
  // 🟢 Green → 🟡 h-exhausted → 🟡 weekly-exhausted → 🔴 Red.
  let sorted_indices = sort_indices( accounts, sort, desc, prefer, now_secs );

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
  if cols.owner        { headers.push( "Owner".to_string() ); }
  if cols.next         { headers.push( "\u{2192} Next".to_string() ); }

  let mut builder = RowBuilder::new( headers );
  for orig_idx in sorted_indices.iter().copied()
  {
    let aq = &accounts[ orig_idx ];
    // Four-level priority: ✓ (is_current) > * (is_active, not current) > @ (occupied on another machine) > blank.
    let flag_cell = if aq.is_current
    {
      "✓".to_string()
    }
    else if aq.is_active
    {
      "*".to_string()
    }
    else if aq.is_occupied_elsewhere
    {
      "@".to_string()
    }
    else
    {
      String::new()
    };

    let expires_cell = compute_expires_cell( aq.expires_at_ms, now_secs );
    let sub_str      = sub_label( aq.account.as_ref() ).to_string();
    // Fix(BUG-232): billing_type=="none" → no active subscription → no renewal date to show.
    // Root cause: renews_label uses org_created_at unconditionally; has no billing_type param.
    // Pitfall: org_created_at may be present even when subscription is cancelled; must check
    //   billing_type BEFORE passing org_created_at to renews_label.
    let renews_str = if aq.account.as_ref().is_some_and( |a| a.billing_type == "none" )
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

    match &aq.result
    {
      Ok( data ) =>
      {
        let mut cells = quota_text_cells( data, now_secs );
        if aq.cached
        {
          prefix_tilde( &mut cells );
          // For cached rows: saturating_sub clamps negative countdowns to 0 in quota_text_cells,
          // making "in 0s" indistinguishable from a future event. Re-check timestamps here.
          let is_past = |iso : Option< &str >| -> bool
          {
            iso.and_then( claude_quota::iso_to_unix_secs ).is_some_and( |t| t < now_secs )
          };
          if is_past( data.five_hour.as_ref().and_then( |p| p.resets_at.as_deref() ) ) { cells[ 1 ] = "(stale)".to_string(); }
          if is_past( data.seven_day.as_ref().and_then( |p| p.resets_at.as_deref() ) ) { cells[ 4 ] = "(stale)".to_string(); }
        }
        let son_unix  = data.seven_day_sonnet.as_ref()
          .and_then( |p| p.resets_at.as_deref() )
          .and_then( claude_quota::iso_to_unix_secs );
        let son_reset = match son_unix
        {
          None                                    => "\u{2014}".to_string(),
          Some( t ) if aq.cached && t < now_secs => "(stale)".to_string(),
          Some( t ) =>
          {
            let label = format!( "in {}", format_duration_secs( t.saturating_sub( now_secs ) ) );
            if aq.cached { format!( "~{label}" ) } else { label }
          }
        };
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
        );

        let name_display = if aq.cached
        {
          format!( "{} {}", aq.name, cache_age_label( aq.cache_age_secs.unwrap_or( 0 ) ) )
        }
        else
        {
          aq.name.clone()
        };
        let mut row : Vec< String > = vec![ flag_cell ];
        if cols.status       { row.push( status_emoji( aq ).to_string() ); }
        row.push( name_display );
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
        if cols.owner        { row.push( aq.owner.clone() ); }
        if cols.next         { row.push( next_cell ); }
        builder = builder.add_row( row.into_iter().map( Into::into ).collect() );
      }
      Err( reason ) =>
      {
        let dash      = "\u{2014}".to_string();
        let error_str = format!( "({})", shorten_error( reason ) );

        let mut row : Vec< String > = vec![ flag_cell ];
        if cols.status       { row.push( status_emoji( aq ).to_string() ); }
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
        if cols.owner        { row.push( aq.owner.clone() ); }
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

  // Footer: shown only when ≥2 valid accounts (AC-10).
  let valid_count = accounts.iter().filter( |aq| aq.result.is_ok() ).count();
  if valid_count < 2
  {
    return append_sessions_table( body, store_path, who );
  }

  // Footer: 2-line `·`-delimited column-aligned format (AC-10).
  let strategy_name = match sort
  {
    SortStrategy::Name   => "name",
    SortStrategy::Renew  => "renew",
    SortStrategy::Renews => "renews",
  };
  let total = accounts.len();

  let Some( idx ) = find_next_for_strategy( accounts, sort, prefer, now_secs, false ) else
  {
    return append_sessions_table( body, store_path, who );
  };

  let rec    = &accounts[ idx ];
  let metric = strategy_metric( rec, sort, prefer, now_secs );
  let rec_model   = recommended_model( rec );
  let rec_display = match session_effort
  {
    Some( se ) => rec_model.to_string() + "/" + se,
    None       => rec_model.to_string(),
  };

  // Build footer lines: find current (✓) account or fall back to legacy format.
  let footer = if let Some( cur ) = accounts.iter().find( |aq| aq.is_current )
  {
    let model_effort = match ( session_model, session_effort )
    {
      ( Some( sm ), Some( se ) ) => [ sm, se ].join( "/" ),
      ( Some( sm ), None       ) => sm.to_string(),
      ( None,       Some( se ) ) => se.to_string(),
      ( None,       None       ) => String::new(),
    };
    // Column widths for `·` alignment.
    let next_label   = format!( "Next ({strategy_name})" );
    let col1_w = next_label.len().max( "Current".len() );
    let col2_w = cur.name.len().max( rec.name.len() );
    let col3_w = model_effort.len().max( rec_display.len() );
    format!(
      "{:<col1_w$} · {:<col2_w$} · {:<col3_w$} · {valid_count}/{total}\n\
       {:<col1_w$} · {:<col2_w$} · {:<col3_w$} · {metric}\n",
      "Current", cur.name, model_effort,
      next_label, rec.name, rec_display,
    )
  }
  else
  {
    // Fallback: credentials unreadable — legacy format (AC-10).
    let session_part = match ( session_model, session_effort )
    {
      ( Some( sm ), Some( se ) ) => format!( "   session: {sm}  effort: {se}" ),
      ( Some( sm ), None       ) => format!( "   session: {sm}" ),
      ( None,       Some( se ) ) => format!( "   effort: {se}" ),
      ( None,       None       ) => String::new(),
    };
    format!( "Valid: {valid_count} / {total}{session_part}\n" )
  };

  append_sessions_table( format!( "{body}{footer}" ), store_path, who )
}

// ── Sessions table ─────────────────────────────────────────────────────────────

/// Append sessions table to `body` when `store_path` is provided and visibility allows.
///
/// `who = None` → auto (show when >1 `_active_*` marker); `Some(true)` → force on;
/// `Some(false)` → suppress.
fn append_sessions_table(
  body       : String,
  store_path : Option< &std::path::Path >,
  who        : Option< bool >,
) -> String
{
  let Some( store ) = store_path else { return body; };
  let ( marker_count, sessions_text ) = build_sessions_table( store );
  let show = match who
  {
    Some( true  ) => true,
    Some( false ) => false,
    None          => marker_count > 1,
  };
  if show && !sessions_text.is_empty()
  {
    format!( "{body}\n{sessions_text}" )
  }
  else
  {
    body
  }
}

/// Read all `_active_*` markers from `store_path`, render as `{user}@{host} → account` table.
///
/// Returns `(marker_count, table_string)`. `marker_count` includes the own marker.
/// Own session receives `✓` appended to the Account column.
fn build_sessions_table( store_path : &std::path::Path ) -> ( usize, String )
{
  use claude_profile_core::account::active_marker_filename;

  let own_marker = active_marker_filename();

  // Collect all `_active_*` entries from the credential store.
  let entries : Vec< ( String, String, bool ) > =
    std::fs::read_dir( store_path )
      .ok()
      .into_iter()
      .flatten()
      .filter_map( Result::ok )
      .filter_map( | entry |
      {
        let fname = entry.file_name().to_string_lossy().into_owned();
        if !fname.starts_with( "_active_" ) { return None; }
        // Parse `{user}@{host}` from `_active_{host}_{user}`: strip prefix, split on last `_`.
        let rest = fname.strip_prefix( "_active_" )?;
        let ( host, user ) = rest.rsplit_once( '_' )?;
        let identity    = format!( "{user}@{host}" );
        let account_raw = std::fs::read_to_string( entry.path() ).unwrap_or_default();
        let account     = account_raw.trim().to_string();
        let is_own      = fname == own_marker;
        Some( ( identity, account, is_own ) )
      } )
      .collect();

  let marker_count = entries.len();
  if marker_count == 0 { return ( 0, String::new() ); }

  // Build table with `data_fmt`.
  let headers = vec![ "Session".to_string(), "Account".to_string() ];
  let mut builder = RowBuilder::new( headers );
  for ( identity, account, is_own ) in &entries
  {
    let account_cell = if *is_own
    {
      format!( "{account} ✓" )
    }
    else
    {
      account.clone()
    };
    builder = builder.add_row( vec![ identity.clone().into(), account_cell.into() ] );
  }
  let view  = builder.build_view();
  let table = Format::format(
    &TableFormatter::with_config( TableConfig::default().auto_wrap( false ) ),
    &view,
  ).unwrap_or_default();

  ( marker_count, format!( "Sessions\n\n{table}\n" ) )
}

// ── JSON renderer ─────────────────────────────────────────────────────────────

// ── Staleness display helpers ─────────────────────────────────────────────────

/// Prefix each non-dash cell with `~` to indicate cached (stale) data.
fn prefix_tilde( cells : &mut [ String; 5 ] )
{
  let dash = "\u{2014}";
  for cell in cells.iter_mut()
  {
    if *cell != dash
    {
      *cell = format!( "~{cell}" );
    }
  }
}

/// Format a cache age as a human-readable suffix: `(Nm ago)` or `(Nh ago)`.
fn cache_age_label( secs : u64 ) -> String
{
  if secs < 3600 { format!( "({}m ago)", secs / 60 ) }
  else { format!( "({}h ago)", secs / 3600 ) }
}

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
        )
        {
          None                        => ( "null".to_string(), "null".to_string() ),
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
\"session_5h_left_pct\":{session_pct},\"session_5h_resets_in_secs\":{session_reset},\
\"weekly_7d_left_pct\":{weekly_pct},\"weekly_7d_sonnet_left_pct\":{sonnet_pct},\
\"weekly_7d_resets_in_secs\":{weekly_reset},{cached_json}}}",
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
    let renews_str = if aq.account.as_ref().is_some_and( |a| a.billing_type == "none" )
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

// ── Plain-text renderer ────────────────────────────────────────────────────────

/// Render quota results as plain text (same as `render_text` with emoji replaced).
///
/// `🟢`→`ok`, `🟡`→`warn`, `🔴`→`err`, `→`→`->`, `✓`→`*`.
#[ allow( clippy::too_many_arguments ) ]
pub( crate ) fn render_plain(
  accounts       : &[ AccountQuota ],
  sort           : SortStrategy,
  desc           : Option< bool >,
  prefer         : PreferStrategy,
  cols           : &ColsVisibility,
  session_model  : Option< &str >,
  session_effort : Option< &str >,
  store_path     : Option< &std::path::Path >,
  who            : Option< bool >,
) -> String
{
  let raw = render_text( accounts, sort, desc, prefer, cols, session_model, session_effort, store_path, who );
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
/// `host` and `role` return the values from `{name}.json`, empty when absent.
pub( crate ) fn extract_get_field( aq : &AccountQuota, field : GetField, now_secs : u64 ) -> String
{
  let dash = "\u{2014}".to_string();
  match field
  {
    GetField::Status  => status_emoji( aq ).to_string(),
    GetField::Account => aq.name.clone(),
    GetField::Expires => compute_expires_cell( aq.expires_at_ms, now_secs ),
    GetField::Sub    => sub_label( aq.account.as_ref() ).to_string(),
    // Fix(BUG-232): billing_type=="none" → no active subscription → no renewal date to show.
    // Root cause: renews_label uses org_created_at unconditionally; has no billing_type param.
    // Pitfall: org_created_at may be present even when subscription is cancelled; must check
    //   billing_type BEFORE passing org_created_at to renews_label.
    GetField::Renews => if aq.account.as_ref().is_some_and( |a| a.billing_type == "none" )
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
    },
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
        match next_event_raw(
          seven_reset,
          ren_pair.map( |( s, _ )| s ),
          ren_pair.is_some_and( |( _, est )| est ),
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
#[ path = "render_tests.rs" ]
mod tests;
