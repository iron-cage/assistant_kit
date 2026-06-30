// Items are pub for test_bridge re-export; lints suppressed — internal API.
#![ allow( clippy::missing_inline_in_public_items, clippy::must_use_candidate ) ]
//! Text-table, plain, and field-extract renderers for quota results.
//!
//! `render_text` produces the human-readable `data_fmt` table; `render_plain`
//! produces a no-emoji version of the text table; `extract_get_field` extracts one
//! column value as a bare string. JSON, TSV, and sessions renderers live in sibling
//! modules and are re-exported here for callers. All consumed by `api.rs::usage_routine`.

use data_fmt::{ RowBuilder, TableFormatter, TableConfig, Format };
use crate::output::format_duration_secs;
use super::types::{ AccountQuota, SortStrategy, PreferStrategy, ColsVisibility, GetField };
use super::format::{
  recommended_model,
  compute_expires_cell, sub_label, shorten_error,
  quota_text_cells, status_emoji, renews_label, next_event_label, next_event_raw, renewal_secs,
};
use super::sort::{ sort_indices, find_next_for_strategy, strategy_metric };
use super::render_sessions::append_sessions_table;
pub use super::render_json::render_json;
pub use super::render_tsv::render_tsv;

// ── Text renderer ─────────────────────────────────────────────────────────────

/// Render quota results as a plain-text table using `data_fmt`.
///
/// Empty store renders `(no accounts configured)` without a table.
/// Column visibility is controlled by `cols` (structural `flag` and `account`
/// columns are always shown). Footer: single-strategy recommendation line when
/// ≥2 accounts have valid quota — shows the winner for the active `sort::`.
/// Footer is omitted when < 2 accounts have valid quota data.
#[ allow( clippy::too_many_lines, clippy::too_many_arguments ) ]
pub fn render_text(
  accounts       : &[ AccountQuota ],
  sort           : SortStrategy,
  desc           : Option< bool >,
  prefer         : PreferStrategy,
  cols           : &ColsVisibility,
  session_model  : Option< &str >,
  session_effort : Option< &str >,
  store_path     : Option< &std::path::Path >,
  who            : Option< bool >,
  gate_ownership : bool,
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
  let table = Format::format( &TableFormatter::with_config( TableConfig::default().with_auto_wrap( false ) ), &view ).unwrap_or_default();
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

  // Fix(BUG-320): gate_ownership was hardcoded false — footer could recommend a non-owned
  //   account while auto-switch (gate_ownership=true) would skip it. Root cause: render_text
  //   had no gate_ownership param; false was hardcoded at the call site.
  // Pitfall: api.rs must pass params.rotate && !params.force; live.rs and mod.rs pass false.
  let Some( idx ) = find_next_for_strategy( accounts, sort, prefer, now_secs, gate_ownership ) else
  {
    return append_sessions_table( body, store_path, who );
  };

  let rec    = &accounts[ idx ];
  let metric = strategy_metric( rec, sort, prefer, now_secs );
  let rec_model  = recommended_model( rec );
  // Fix(H3/TSK-335): rec_display is model-derived, not carry-forward from current account's session_effort.
  // Root cause: matching session_effort passed effort from the CURRENT account into the NEXT account display,
  //   yielding the wrong effort level when accounts differ in model assignment.
  // Pitfall: session_effort still governs the Current line's model_effort display — do not conflate the two.
  let rec_effort  = if rec_model == "opus" { "max" } else { "high" };
  let rec_display = rec_model.to_string() + "/" + rec_effort;

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
  gate_ownership : bool,
) -> String
{
  let raw = render_text( accounts, sort, desc, prefer, cols, session_model, session_effort, store_path, who, gate_ownership );
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
