// Items are pub for test_bridge re-export; lints suppressed — internal API.
#![ allow( clippy::missing_inline_in_public_items, clippy::must_use_candidate, clippy::missing_panics_doc ) ]
//! TSV renderer for quota results.

use super::types::{ AccountQuota, SortStrategy, PreferStrategy, ColsVisibility };
use super::format::{
  expires_cell_for, sub_cell_for, renews_cell_for, shorten_error, with_lock_marker,
  quota_cells_for, PctStyle, status_emoji, next_event_label, renewal_secs,
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
  if cols.tags         { headers.push( "tags".to_string() ); }
  let mut out = headers.join( "\t" );
  out.push( '\n' );

  if accounts.is_empty() { return out; }

  let sorted_indices = sort_indices( accounts, sort, desc, prefer, now_secs );
  for idx in sorted_indices
  {
    let aq         = &accounts[ idx ];
    // BUG-344 (reverted): a fetch-result conjunct was added here, then reverted — see render.rs's
    //   flag_cell comment for the full explanation (docs/feature/009_token_usage.md AC-02/AC-11/
    //   Algorithm step 5a specify ✓ gated solely on is_current).
    let flag_cell  = if aq.is_current { "\u{2713}" } else if aq.is_active { "*" } else if aq.is_occupied_elsewhere { "@" } else { "" };
    let status_str = match status_emoji( aq )
    {
      "🟢" => "ok",
      "🟡" => "warn",
      // Feature 071: redirect-backend row — no Anthropic quota semantics; matches
      // `.credentials.status`'s `Token: static` vocabulary, distinct from a real "err".
      "⚪" => "static",
      _    => "err",
    };
    // Fix(BUG-345): compute_expires_cell alone cannot show cache-fallback staleness.
    // Root cause: aq.cached (fetch provenance) was never combined with expires_at_ms here.
    // Pitfall: use the aq-aware wrapper (cache `~`-prefix + redirect `static`), not
    //   compute_expires_cell directly, wherever an aq is in scope.
    let expires_str = expires_cell_for( aq, now_secs );
    // Fix(BUG-540): sub/renews come from their aq-aware helpers — the known-absence
    //   predicates (BUG-232 billing "none", BUG-538 redirect backend) live once in
    //   format.rs; this file previously carried the second of three renews copies.
    // Pitfall: the TSV reason cell deliberately keeps the FULL redirect descriptor
    //   (unlike the text table's compact `(redirect)`) — only sub/renews cells
    //   delegate to the helpers here.
    let sub_str     = sub_cell_for( aq );
    let renews_str  = renews_cell_for( aq, now_secs );

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
    // Feature 070 lock visibility: 🔒 suffix keeps name-prefix matching intact for
    // TSV consumers (same suffix convention as the fallback-reason cell above).
    row.push( with_lock_marker( aq, name_cell ) );

    match &aq.result
    {
      Ok( data ) =>
      {
        // Fix(BUG-553 S1/S3): this arm rebuilt every quota cell from `quota_text_cells` plus a
        //   local `pct_bare`, and so referenced `aq.cached` nowhere — a cache-fallback row read
        //   as live on the one surface with no `cached` column to disclose it otherwise (unlike
        //   JSON, which emits `cached`/`cache_age_secs` outright). BUG-551's touch projection
        //   was absent here for the same reason. The metadata cells below (`expires_str`,
        //   `sub_str`, `renews_str`) already routed through their shared aq-aware helpers — the
        //   quota block was the outlier within its own function.
        // Root cause: `quota_text_cells` takes only `data`, so no account-dependent rule could
        //   live inside it; every surface had to re-apply them by hand, and this one never did.
        // Pitfall: take the cells from `quota_cells_for` — `PctStyle::Bare` already covers the
        //   only legitimate TSV difference (no emoji), so nothing here needs rebuilding.
        let cells = quota_cells_for( aq, data, now_secs, PctStyle::Bare );

        let ( ren_secs, ren_est ) = renewal_secs(
          aq.renewal_at.as_deref(),
          aq.org_created_at.as_deref(),
          now_secs,
        ).unzip();
        let next_str = next_event_label(
          data.seven_day.as_ref().and_then( |p| p.resets_at.as_deref() )
            .and_then( claude_quota::iso_to_unix_secs )
            .map( |t| t.saturating_sub( now_secs ) ),
          ren_secs,
          ren_est.unwrap_or( false ),
        );
        if cols.h5_left      { row.push( cells[ 0 ].clone() ); }
        if cols.h5_reset     { row.push( cells[ 1 ].clone() ); }
        if cols.d7_left      { row.push( cells[ 2 ].clone() ); }
        if cols.d7_son       { row.push( cells[ 3 ].clone() ); }
        if cols.d7_reset     { row.push( cells[ 4 ].clone() ); }
        if cols.d7_son_reset { row.push( cells[ 5 ].clone() ); }
        if cols.expires      { row.push( expires_str ); }
        if cols.sub          { row.push( sub_str ); }
        if cols.renews       { row.push( renews_str ); }
        if cols.host         { row.push( aq.host.clone() ); }
        if cols.role         { row.push( aq.role.clone() ); }
        if cols.owner        { row.push( aq.owner.clone() ); }
        if cols.next         { row.push( next_str ); }
        if cols.tags         { row.push( aq.tags.join( ", " ) ); }
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
        if cols.tags    { row.push( aq.tags.join( ", " ) ); }
      }
    }

    out.push_str( &row.join( "\t" ) );
    out.push( '\n' );
  }

  out
}
