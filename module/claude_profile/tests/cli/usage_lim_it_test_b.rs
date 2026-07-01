//! Integration tests: IT-205–IT-247 `.usage lim_it` — Part B.
//!
//! Continuation of `usage_lim_it_test.rs`.

use crate::cli_runner::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_account, write_account_with_token, write_account_profile_json,
  live_active_token, require_live_api,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── it217: lim_it format::tsv with status text labels (028 FT-13) ─────────────

/// it217 `lim_it` (028 FT-13): `format::tsv` produces tab-separated output;
/// the status column contains text labels (`ok`, `warn`, `err`) not emoji.
///
/// Spec: [`tests/docs/feature/028_usage_row_filtering.md` FT-13]
#[ test ]
fn it217_lim_it_ft028_13_format_tsv_status_text()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it217: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out = run_cs_with_env( &[ ".usage", "format::tsv" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // TSV header row uses tabs.
  let has_tab = text.contains( '\t' );
  assert!( has_tab, "format::tsv output must contain tab characters, got:\n{text}" );
  // Status column uses text label, not emoji.
  assert!(
    !text.contains( "🟢" ) && !text.contains( "🟡" ) && !text.contains( "🔴" ),
    "format::tsv status column must use text labels (ok/warn/err), not emoji, got:\n{text}",
  );
  assert!(
    text.contains( "ok" ) || text.contains( "warn" ) || text.contains( "err" ),
    "format::tsv status column must contain a text label, got:\n{text}",
  );
}

// ── it218: lim_it no_color::1 produces emoji-free output (028 FT-14) ─────────

/// it218 `lim_it` (028 FT-14): `no_color::1` with a valid account produces
/// emoji-free output; status column shows plain text labels.
///
/// Spec: [`tests/docs/feature/028_usage_row_filtering.md` FT-14]
#[ test ]
fn it218_lim_it_ft028_14_no_color_emoji_free()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it218: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out = run_cs_with_env( &[ ".usage", "no_color::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "🟢" ) && !text.contains( "🟡" ) && !text.contains( "→" ),
    "no_color::1 must produce emoji-free output for valid account, got:\n{text}",
  );
}

// ── it219: lim_it filters compose with sort/count/cols (028 FT-16) ───────────

/// it219 `lim_it` (028 FT-16): `sort::name only_valid::1 count::2 cols::+sub`
/// composes all filter/sort/col params correctly. At most 2 non-🔴 rows, sorted
/// alphabetically, Sub column present.
///
/// Spec: [`tests/docs/feature/028_usage_row_filtering.md` FT-16]
#[ test ]
fn it219_lim_it_ft028_16_filters_compose()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it219: no live token — skipping" );
    return;
  };
  require_live_api( "it219" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true  );
  write_account_with_token( dir.path(), "acct-b@test.com", &token, false );

  let out = run_cs_with_env(
    &[ ".usage", "sort::name", "only_valid::1", "count::2", "cols::+sub" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Sub column must be present in header.
  assert!(
    text.contains( "Sub" ),
    "cols::+sub must add Sub column header, got:\n{text}",
  );
  // At most 2 data rows.
  let data_rows = text.lines()
    .filter( | l | l.contains( "@test.com" ) )
    .count();
  assert!(
    data_rows <= 2,
    "count::2 must limit result to at most 2 rows, got {data_rows} rows:\n{text}",
  );
}

// ── it222: lim_it IT-72 format::json new renewal fields ──────────────────────

/// it222 `lim_it` (IT-72): `format::json` output contains the new renewal and
/// next-event fields; the legacy `next_renewal_est` key must be absent.
///
/// Required fields: `renewal_secs`, `renewal_is_estimate`, `next_event_type`,
/// `next_event_secs`. Legacy `next_renewal_est` must not appear.
///
/// Spec: [`tests/docs/cli/command/009_usage.md` IT-72]
///       [`docs/feature/009_token_usage.md` AC-29]
#[ test ]
fn it222_lim_it_it72_json_new_renewal_fields()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it222: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "renewal_secs" ),
    "format::json must contain 'renewal_secs' field (IT-72), got:\n{text}",
  );
  assert!(
    text.contains( "renewal_is_estimate" ),
    "format::json must contain 'renewal_is_estimate' field (IT-72), got:\n{text}",
  );
  assert!(
    text.contains( "next_event_type" ),
    "format::json must contain 'next_event_type' field (IT-72), got:\n{text}",
  );
  assert!(
    text.contains( "next_event_secs" ),
    "format::json must contain 'next_event_secs' field (IT-72), got:\n{text}",
  );
  assert!(
    !text.contains( "next_renewal_est" ),
    "format::json must NOT contain legacy 'next_renewal_est' field (IT-72), got:\n{text}",
  );
}

// ── it223–it224: lim_it abs::1 / abs::true show token counts (046 EC-4/EC-6) ─

/// it223 `lim_it` (046 EC-4): `abs::1` shows absolute token counts instead of
/// percentages. Quota columns must not contain `%` suffix.
///
/// Spec: [`tests/docs/cli/param/046_abs.md` EC-4]
#[ test ]
fn it223_lim_it_abs_1_shows_token_counts()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it223: no live token — skipping" );
    return;
  };
  require_live_api( "it223" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out_pct = run_cs_with_env( &[ ".usage", "abs::0" ], &[ ( "HOME", home ) ] );
  let out_abs = run_cs_with_env( &[ ".usage", "abs::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_abs, 0 );

  let text_pct = stdout( &out_pct );
  let text_abs = stdout( &out_abs );

  // Default (abs::0) shows % values; abs::1 must not.
  assert!(
    text_pct.contains( '%' ),
    "abs::0 (default) must show percentage values, got:\n{text_pct}",
  );
  assert!(
    !text_abs.contains( '%' ) || text_abs.lines().filter( | l | l.contains( '%' ) ).all( | l | l.contains( "Reset" ) ),
    "abs::1 quota columns must show absolute counts without % suffix, got:\n{text_abs}",
  );
}

/// it224 `lim_it` (046 EC-6): `abs::true` produces the same output as `abs::1`.
///
/// Spec: [`tests/docs/cli/param/046_abs.md` EC-6]
#[ test ]
fn it224_lim_it_abs_true_shows_token_counts()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it224: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out_1    = run_cs_with_env( &[ ".usage", "abs::1"    ], &[ ( "HOME", home ) ] );
  let out_true = run_cs_with_env( &[ ".usage", "abs::true" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_true, 0 );
  // abs::true and abs::1 must produce identical output.
  assert_eq!(
    stdout( &out_1 ), stdout( &out_true ),
    "abs::true must produce the same output as abs::1 (046 EC-6)",
  );
}

// ── it225: → Next cell shows event label + duration (live) ───────────────────

/// it225 — The `→ Next` column cells contain a recognized strategic event-label-and-duration string.
///
/// Given a live account with valid quota data, the `→ Next` column must show the soonest
/// upcoming strategic event as `<label> in <duration>` — not an empty cell or bare header.
///
/// After TSK-228, only `+7d` (7-day reset) and `$ren` (billing renewal) are candidates.
/// Token expiry (`!tok`) and 5h session reset (`+5h`) are no longer included.
///
/// Spec: [`tests/docs/cli/command/009_usage.md` IT-71]
#[ test ]
fn it225_lim_it_it71_next_event_cell_shows_label_and_duration()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it225: no live token — skipping" );
    return;
  };
  require_live_api( "it225" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  // Column header must be present.
  assert!(
    text.contains( "\u{2192} Next" ),
    "→ Next column header must appear in default output (IT-71), got:\n{text}",
  );
  // At least one strategic event-label pattern must appear in the output.
  // Valid labels after TSK-228: +7d, $ren — now formatted as "in <dur> +7d" / "in <dur> $ren".
  // !tok and +5h are not candidates (token expiry / 5h reset excluded from → Next).
  let has_event_label =
    text.contains( " +7d" )
    || text.contains( " $ren" );
  assert!(
    has_event_label,
    "→ Next cell must contain 'in ... +7d' or 'in ... $ren' for live account (IT-71), got:\n{text}",
  );
}

// ── it226–it227: only_next:: live tests (040 EC-3/6) ─────────────────────────

/// it226 `lim_it` (040 EC-3): `only_next::1 sort::renews` shows → row from renews strategy.
///
/// With two live accounts sharing the same token, `only_next::1 sort::renews`
/// must show exactly one row — the renews-strategy winner — which has the `→` marker.
///
/// Spec: [`tests/docs/cli/param/040_only_next.md` EC-3]
#[ test ]
fn it226_lim_it_only_next_1_renews_shows_winner()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it226: no live token — skipping" );
    return;
  };
  require_live_api( "it226" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true  );
  write_account_with_token( dir.path(), "acct-b@test.com", &token, false );

  let out = run_cs_with_env(
    &[ ".usage", "only_next::1", "sort::renews" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let data_rows = text.lines()
    .filter( | l | l.contains( "@test.com" ) )
    .count();
  assert_eq!(
    data_rows, 1,
    "only_next::1 sort::renews must show exactly 1 row (040 EC-3), got:\n{text}",
  );
  let arrow_rows = text.lines()
    .filter( | l | l.contains( "\u{2192}" ) && l.contains( "@test.com" ) )
    .count();
  assert_eq!(
    arrow_rows, 1,
    "only_next::1 sort::renews must show the → account row (040 EC-3), got:\n{text}",
  );
}

/// it227 `lim_it` (040 EC-6): `only_next::true` accepted as alias for 1.
///
/// With two live accounts, `only_next::true` must behave like `only_next::1` —
/// exactly one row shown, the → account.
///
/// Spec: [`tests/docs/cli/param/040_only_next.md` EC-6]
#[ test ]
fn it227_lim_it_only_next_true_shows_arrow_row()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it227: no live token — skipping" );
    return;
  };
  require_live_api( "it227" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true  );
  write_account_with_token( dir.path(), "acct-b@test.com", &token, false );

  let out = run_cs_with_env(
    &[ ".usage", "only_next::true" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let data_rows = text.lines()
    .filter( | l | l.contains( "@test.com" ) )
    .count();
  assert_eq!(
    data_rows, 1,
    "only_next::true must show exactly 1 row (040 EC-6), got:\n{text}",
  );
  let arrow_rows = text.lines()
    .filter( | l | l.contains( "\u{2192}" ) && l.contains( "@test.com" ) )
    .count();
  assert_eq!(
    arrow_rows, 1,
    "only_next::true must show the → account row (040 EC-6), got:\n{text}",
  );
}

// ── it228–it230: only_valid/exclude_exhausted live tests (043/044 EC-1/3) ─────

/// it228 `lim_it` (043 EC-1): `only_valid::1` shows 🟢 account; hides 🔴 error.
///
/// With one live account (🟢) and one error account (🔴), `only_valid::1`
/// must show only the live account and hide the error account.
///
/// Spec: [`tests/docs/cli/param/043_only_valid.md` EC-1]
#[ test ]
fn it228_lim_it_only_valid_1_shows_green_hides_red()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it228: no live token — skipping" );
    return;
  };
  require_live_api( "it228" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "live-acct@test.com",  &token, true  );
  write_account( dir.path(), "error-acct@test.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "only_valid::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "live-acct@test.com" ),
    "only_valid::1 must show 🟢 live account (043 EC-1), got:\n{text}",
  );
  assert!(
    !text.contains( "error-acct@test.com" ),
    "only_valid::1 must hide 🔴 error account (043 EC-1), got:\n{text}",
  );
}

/// it229 `lim_it` (044 EC-1): `exclude_exhausted::1` shows 🟢; hides 🔴 error.
///
/// With one live account (🟢) and one error account (🔴), `exclude_exhausted::1`
/// must show only the live account and hide the error account.
///
/// Note: the 🟡 (quota-exhausted, valid token) divergence from `only_valid::1`
/// requires a real exhausted account state unavailable with shared tokens.
///
/// Spec: [`tests/docs/cli/param/044_exclude_exhausted.md` EC-1]
#[ test ]
fn it229_lim_it_exclude_exhausted_1_shows_green()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it229: no live token — skipping" );
    return;
  };
  require_live_api( "it229" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "live-acct@test.com",  &token, true  );
  write_account( dir.path(), "error-acct@test.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "exclude_exhausted::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "live-acct@test.com" ),
    "exclude_exhausted::1 must show 🟢 live account (044 EC-1), got:\n{text}",
  );
  assert!(
    !text.contains( "error-acct@test.com" ),
    "exclude_exhausted::1 must hide 🔴 error account (044 EC-1), got:\n{text}",
  );
}

/// it230 `lim_it` (044 EC-3): `exclude_exhausted::1` is at least as strict as `only_valid::1`.
///
/// Both filters applied to the same accounts: `exclude_exhausted::1` must show
/// no more rows than `only_valid::1`. The 🟡-divergence (kept by `only_valid::1`,
/// filtered by `exclude_exhausted::1`) requires an exhausted account state that
/// cannot be manufactured with shared live tokens.
///
/// Spec: [`tests/docs/cli/param/044_exclude_exhausted.md` EC-3]
#[ test ]
fn it230_lim_it_exclude_exhausted_stricter_than_only_valid()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it230: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "live-acct@test.com",  &token, true  );
  write_account( dir.path(), "error-acct@test.com", "max", "default", FAR_FUTURE_MS, false );

  let out_valid = run_cs_with_env( &[ ".usage", "only_valid::1" ],        &[ ( "HOME", home ) ] );
  let out_excl  = run_cs_with_env( &[ ".usage", "exclude_exhausted::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_valid, 0 );
  assert_exit( &out_excl,  0 );

  let rows_valid = stdout( &out_valid ).lines().filter( | l | l.contains( "@test.com" ) ).count();
  let rows_excl  = stdout( &out_excl  ).lines().filter( | l | l.contains( "@test.com" ) ).count();

  assert!(
    rows_excl <= rows_valid,
    "exclude_exhausted::1 must show ≤ rows than only_valid::1 (044 EC-3): valid={rows_valid} excl={rows_excl}",
  );
  assert!(
    !stdout( &out_excl ).contains( "error-acct@test.com" ),
    "exclude_exhausted::1 must hide 🔴 error account (044 EC-3)",
  );
}

// ── it231–it234: get:: live/offline tests (045 EC-1/3/5/7) ───────────────────

/// it231 `lim_it` (045 EC-1): `get::7d_left` extracts bare percentage string.
///
/// With a live account, `get::7d_left` must output exactly one percentage string
/// (e.g., `65%`) on stdout — no column headers, no footer.
///
/// Spec: [`tests/docs/cli/param/045_get.md` EC-1]
#[ test ]
fn it231_lim_it_get_7d_left_extracts_bare_pct()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it231: no live token — skipping" );
    return;
  };
  require_live_api( "it231" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out = run_cs_with_env(
    &[ ".usage", "sort::name", "get::7d_left" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text  = stdout( &out );
  let trimmed = text.trim();

  assert!(
    trimmed.ends_with( '%' ),
    "get::7d_left must output a percentage string e.g. '65%' (045 EC-1), got:\n{trimmed}",
  );
  assert!(
    !trimmed.contains( "7d Left" ),
    "get::7d_left must not contain column headers (045 EC-1), got:\n{trimmed}",
  );
  assert!(
    !trimmed.contains( "Valid:" ),
    "get::7d_left must not contain footer (045 EC-1), got:\n{trimmed}",
  );
}

/// it232 `lim_it` (045 EC-3): `get::status` extracts bare 🟢 emoji for live account.
///
/// With a live (🟢) account, `get::status` must output `🟢` as the sole content.
///
/// Spec: [`tests/docs/cli/param/045_get.md` EC-3]
#[ test ]
fn it232_lim_it_get_status_extracts_green_emoji()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it232: no live token — skipping" );
    return;
  };
  require_live_api( "it232" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out = run_cs_with_env( &[ ".usage", "get::status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text    = stdout( &out );
  let trimmed = text.trim();

  assert_eq!(
    trimmed, "\u{1f7e2}",
    "get::status on live (🟢) account must output exactly '🟢' (045 EC-3), got:\n{trimmed}",
  );
}

/// it233 (045 EC-5): `get::bogus` exits 1; stderr names all valid field IDs.
///
/// After TSK-225, `host`, `role`, `next_event_type`, `next_event_secs` were
/// added as valid `get::` field IDs. The error message must list all of them.
///
/// Spec: [`tests/docs/cli/param/045_get.md` EC-5]
#[ test ]
fn it233_get_bogus_exits_1_names_valid_fields()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "get::bogus" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "next_event_type" ),
    "get::bogus stderr must list 'next_event_type' (045 EC-5), got:\n{err}",
  );
  assert!(
    err.contains( "next_event_secs" ),
    "get::bogus stderr must list 'next_event_secs' (045 EC-5), got:\n{err}",
  );
  assert!(
    err.contains( "7d_left" ),
    "get::bogus stderr must list '7d_left' (045 EC-5), got:\n{err}",
  );
  assert!(
    err.contains( "account" ),
    "get::bogus stderr must list 'account' (045 EC-5), got:\n{err}",
  );
}

/// it234 `lim_it` (045 EC-7): `get::next_event_type` outputs strategic label; `get::next_event_secs` outputs integer.
///
/// With a live account with an upcoming quota event, `get::next_event_type` must
/// output a recognized strategic event-label string (`+7d` or `$ren`); `get::next_event_secs`
/// must output a bare non-negative integer.
///
/// After TSK-228, only `+7d` and `$ren` are candidates. `!tok` and `+5h` are excluded.
///
/// Spec: [`tests/docs/cli/param/045_get.md` EC-7]
#[ test ]
fn it234_lim_it_get_next_event_type_and_secs()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it234: no live token — skipping" );
    return;
  };
  require_live_api( "it234" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out_type = run_cs_with_env(
    &[ ".usage", "get::next_event_type" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_type, 0 );
  let type_text = stdout( &out_type );
  let type_str  = type_text.trim();
  // After TSK-228: only +7d and $ren are strategic next-event candidates.
  let valid_labels = [ "+7d", "$ren" ];
  assert!(
    valid_labels.contains( &type_str ),
    "get::next_event_type must output '+7d' or '$ren' (045 EC-7 after TSK-228), got:\n{type_str}",
  );

  let out_secs = run_cs_with_env(
    &[ ".usage", "get::next_event_secs" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_secs, 0 );
  let secs_text = stdout( &out_secs );
  let secs_str  = secs_text.trim();
  assert!(
    secs_str.parse::<u64>().is_ok(),
    "get::next_event_secs must output a bare integer (045 EC-7), got:\n{secs_str}",
  );
}

// ── it235–it236: no_color:: live tests (047 EC-3/5) ──────────────────────────

/// it235 `lim_it` (047 EC-3): `no_color::0` (default) includes 🟢 emoji.
///
/// With a live (🟢) account, `no_color::0` does not suppress status emoji.
/// Stdout must contain `🟢`.
///
/// Spec: [`tests/docs/cli/param/047_no_color.md` EC-3]
#[ test ]
fn it235_lim_it_no_color_0_output_includes_emoji()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it235: no live token — skipping" );
    return;
  };
  require_live_api( "it235" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out  = run_cs_with_env( &[ ".usage", "no_color::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "\u{1f7e2}" ),
    "no_color::0 must include 🟢 status emoji for live account (047 EC-3), got:\n{text}",
  );
}

/// it236 `lim_it` (047 EC-5): `no_color::1` replaces `✓` with `*` in flag column.
///
/// With a live active account, `no_color::1` must replace the unicode `✓` check mark
/// with ASCII `*` in the flag column, and must not contain the unicode character.
///
/// Spec: [`tests/docs/cli/param/047_no_color.md` EC-5]
#[ test ]
fn it236_lim_it_no_color_1_check_mark_replaced_by_star()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it236: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true  );
  write_account_with_token( dir.path(), "acct-b@test.com", &token, false );

  let out  = run_cs_with_env( &[ ".usage", "no_color::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    !text.contains( '\u{2713}' ),
    "no_color::1 must replace unicode '✓' with '*' (047 EC-5), got:\n{text}",
  );
  assert!(
    text.contains( '*' ),
    "no_color::1 must contain '*' (replaced from '✓') (047 EC-5), got:\n{text}",
  );
}

// ── it237: clear:: live test (051 EC-4) ──────────────────────────────────────

/// it237 `lim_it` (051 EC-4): after `clear::1`, `_renewal_at` is absent from `.json`.
///
/// With a live account that has an injected `_renewal_at` override, `clear::1`
/// must remove it. After clearing, the `.json` must not contain `_renewal_at`.
///
/// Spec: [`tests/docs/cli/param/051_clear.md` EC-4]
#[ test ]
fn it237_lim_it_clear_usage_shows_tilde_estimate()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it237: no live token — skipping" );
    return;
  };
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  // Inject a far-future _renewal_at override.
  std::fs::write(
    store.join( "acct-a@test.com.json" ),
    r#"{"_renewal_at":"2030-01-01T00:00:00Z"}"#,
  ).unwrap();

  // Clear the renewal override.
  let clear_out = run_cs_with_env(
    &[ ".account.renewal", "name::acct-a@test.com", "clear::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &clear_out, 0 );

  // After clear, _renewal_at must be absent from the file.
  let content = std::fs::read_to_string( store.join( "acct-a@test.com.json" ) ).unwrap();
  assert!(
    !content.contains( "_renewal_at" ),
    "clear::1 must remove _renewal_at from .json (051 EC-4), got: {content}",
  );

  // .usage must still succeed after clear.
  let usage_out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &usage_out, 0 );
}

// ── it238–it239: display control param group (005 CC-3/4) ────────────────────

/// it238 `lim_it` (005 CC-3): `get::` bypasses `cols::` column visibility.
///
/// `cols::-7d_left` hides the `7d_left` column from table output, but
/// `get::7d_left` must still extract the underlying data value unchanged —
/// `get::` reads from the data model, not the rendered column set.
///
/// Spec: [`tests/docs/cli/param_group/005_display_control.md` CC-3]
#[ test ]
fn it238_lim_it_get_bypasses_cols_restriction()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it238: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out_hidden = run_cs_with_env(
    &[ ".usage", "cols::-7d_left", "get::7d_left" ],
    &[ ( "HOME", home ) ],
  );
  let out_normal = run_cs_with_env(
    &[ ".usage", "get::7d_left" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_hidden, 0 );
  assert_exit( &out_normal, 0 );

  assert_eq!(
    stdout( &out_hidden ).trim(),
    stdout( &out_normal ).trim(),
    "get::7d_left with cols::-7d_left must produce same output as without cols:: (005 CC-3)",
  );
}

/// it239 (005 CC-4): `cols::+sub` and `no_color::1` apply simultaneously and independently.
///
/// `cols::+sub` adds the Sub column; `no_color::1` strips emoji. Both must be
/// independently active: Sub column header present in output, no emoji in output.
///
/// Spec: [`tests/docs/cli/param_group/005_display_control.md` CC-4]
#[ test ]
fn it239_cols_sub_and_no_color_independent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env(
    &[ ".usage", "cols::+sub", "no_color::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  // cols::+sub applies — Sub column header present.
  assert!(
    text.contains( "Sub" ),
    "cols::+sub must add 'Sub' column header (005 CC-4), got:\n{text}",
  );
  // no_color::1 applies — no emoji in output.
  assert!(
    !text.contains( "\u{1f534}" ),
    "no_color::1 must remove 🔴 emoji (005 CC-4), got:\n{text}",
  );
  assert!(
    !text.contains( "\u{1f7e2}" ),
    "no_color::1 must not contain 🟢 (005 CC-4), got:\n{text}",
  );
}

// ── it240: account targeting param group (006 CC-4) ──────────────────────────

/// it240 `lim_it` (006 CC-4): `cols::+host,+role` shows both columns from profile.json.
///
/// When an account has a `profile.json` with `host` and `role`, `.usage` with
/// `cols::+host,+role` must show both the Host and Role columns populated with
/// the stored values, regardless of token validity.
///
/// Spec: [`tests/docs/cli/param_group/006_account_targeting.md` CC-4]
#[ test ]
fn it240_lim_it_cols_host_role_shows_profile_data()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it240: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );
  write_account_profile_json( dir.path(), "acct-a@test.com", Some( "mybox" ), Some( "work" ) );

  let out  = run_cs_with_env(
    &[ ".usage", "cols::+host,+role" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "Host" ),
    "cols::+host,+role must add 'Host' column header (006 CC-4), got:\n{text}",
  );
  assert!(
    text.contains( "Role" ),
    "cols::+host,+role must add 'Role' column header (006 CC-4), got:\n{text}",
  );
  assert!(
    text.contains( "mybox" ),
    "cols::+host must show 'mybox' host value from profile.json (006 CC-4), got:\n{text}",
  );
  assert!(
    text.contains( "work" ),
    "cols::+role must show 'work' role value from profile.json (006 CC-4), got:\n{text}",
  );
}

