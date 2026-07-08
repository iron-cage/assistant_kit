// Integration tests for the usage module — render_text, render_json, sort/prefer strategy parsing.
// Accesses pub(crate) items through claude_profile::usage::test_bridge (testing feature).

use tempfile::TempDir;
use claude_profile::usage::test_bridge::{
  render_text, render_json, apply_refresh,
  FAR_FUTURE_MS, mk_aq_ok, mk_aq_err, mk_aq_sort, mk_aq_sort_weekly,
  mk_named_aq, mk_named_aq_err,
};
use claude_profile::usage::test_bridge::types::{
  AccountQuota, SortStrategy, PreferStrategy, ColsVisibility,
  SubprocessModel, SubprocessEffort,
};

// ── status_emoji via render_text / render_json ────────────────────────────────

/// SE-1 — Err result → 🔴.
#[ test ]
fn test_status_emoji_red()
{
  let aq = mk_aq_err();
  let output = render_text(
    &[ aq ], SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false,
  );
  assert!( output.contains( "🔴" ), "Err account must show 🔴. Got:\n{output}" );
}

/// SE-2 — Ok, `5h_left` = 90% (util=10.0) → 🟢.
#[ test ]
fn test_status_emoji_green()
{
  let aq = mk_aq_ok( 10.0 );
  let output = render_text(
    &[ aq ], SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false,
  );
  assert!( output.contains( "🟢" ), "90% left must show 🟢. Got:\n{output}" );
}

/// SE-3 — Ok, `5h_left` = 3% (util=97.0) → 🟡.
#[ test ]
fn test_status_emoji_yellow()
{
  let aq = mk_aq_ok( 97.0 );
  let output = render_text(
    &[ aq ], SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false,
  );
  assert!( output.contains( "🟡" ), "3% left must show 🟡. Got:\n{output}" );
}

/// SE-4 — Boundary: 15% exactly (util=85.0) → 🟡 (inclusive at 15% for 5h).
/// SE-4b — Boundary: 15.1% (util=84.9) → 🟢.
#[ test ]
fn test_status_emoji_boundary()
{
  let aq_15pct   = mk_aq_ok( 85.0 );
  let aq_15_1pct = mk_aq_ok( 84.9 );
  let out_15   = render_text(
    &[ aq_15pct ], SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false,
  );
  let out_15_1 = render_text(
    &[ aq_15_1pct ], SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false,
  );
  assert!( out_15.contains( "🟡" ),   "exactly 15% left must show 🟡. Got:\n{out_15}" );
  assert!( out_15_1.contains( "🟢" ), "15.1% left must show 🟢. Got:\n{out_15_1}" );
}

/// SE-5 — Synthetic current-session row (`is_current=true`) shows correct emoji.
#[ test ]
fn test_status_emoji_on_synthetic_row()
{
  let mut aq = mk_aq_ok( 20.0 );
  aq.is_current = true;
  aq.name = "(current session)".to_string();
  let output = render_text(
    &[ aq ], SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false,
  );
  assert!( output.contains( "🟢" ), "80% left synthetic row must show 🟢. Got:\n{output}" );
}

/// SE-6 — JSON output must NOT contain emoji (AC-20 no JSON equivalent).
#[ test ]
fn test_status_emoji_absent_in_json()
{
  let aq = mk_aq_ok( 50.0 );
  let json = render_json( &[ aq ] );
  assert!(
    !json.contains( "🔴" ) && !json.contains( "🟡" ) && !json.contains( "🟢" ),
    "JSON must not contain status emoji. Got:\n{json}",
  );
}

// ── render_text ───────────────────────────────────────────────────────────────

/// C19 — Empty accounts → "(no accounts configured)".
#[ test ]
fn test_render_text_empty()
{
  let result = render_text(
    &[], SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false,
  );
  assert!( result.contains( "no accounts configured" ), "empty must say no accounts, got: {result}" );
}

// ── render_json ───────────────────────────────────────────────────────────────

/// C20 — Empty accounts → "[]".
#[ test ]
fn test_render_json_empty()
{
  let result = render_json( &[] );
  assert_eq!( result.trim(), "[]" );
}

/// C21 — Err account → JSON contains "error" field.
#[ test ]
fn test_render_json_error_account()
{
  let accounts = vec![
    AccountQuota
    {
      name : "fail@test.com".to_string(), is_current : false, is_active : false, is_occupied_elsewhere : false,
      expires_at_ms : 0, result : Err( "auth failed".to_string() ), account : None,
      host : String::new(), role : String::new(), renewal_at : None,
      cached : false, cache_age_secs : None, is_owned : true, owner : String::new(),
    },
  ];
  let result = render_json( &accounts );
  assert!( result.contains( "\"error\":" ), "Err account must have error field, got: {result}" );
  assert!( result.contains( "auth failed" ), "error message must be preserved, got: {result}" );
}

/// C22 — Account name with quotes is JSON-escaped.
#[ test ]
fn test_render_json_escapes_quotes_in_name()
{
  let accounts = vec![
    AccountQuota
    {
      name : "test\"@evil.com".to_string(), is_current : false, is_active : false, is_occupied_elsewhere : false,
      expires_at_ms : 0, result : Err( "fail".to_string() ), account : None,
      host : String::new(), role : String::new(), renewal_at : None,
      cached : false, cache_age_secs : None, is_owned : true, owner : String::new(),
    },
  ];
  let result = render_json( &accounts );
  assert!(
    result.contains( r#"test\"@evil.com"# ),
    "quotes in name must be escaped, got: {result}",
  );
}

/// FT-08 — Mixed Ok and Err accounts both appear in `format::json` output.
#[ test ]
fn test_render_json_ft8_mixed_ok_and_err_both_present()
{
  let store = TempDir::new().unwrap();
  let quota = claude_quota::OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : None,
  };
  let mut accounts = vec![
    AccountQuota
    {
      name          : "ok@example.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : FAR_FUTURE_MS,
      result        : Ok( quota ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at    : None,
      cached        : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    },
    AccountQuota
    {
      name          : "err@example.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : 0,
      result        : Err( "HTTP transport error: HTTP 401".to_string() ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at    : None,
      cached        : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    },
  ];

  apply_refresh( &mut accounts, store.path(), None, false, SubprocessModel::Auto, SubprocessEffort::Auto, false );

  let json = render_json( &accounts );

  assert!( json.contains( "ok@example.com" ),  "Ok account must appear in JSON; got: {json}" );
  assert!( json.contains( "err@example.com" ), "Err account must appear in JSON; got: {json}" );
  assert!( json.contains( "\"error\":" ),               "Err account must have error field; got: {json}" );
  assert!( json.contains( "\"session_5h_left_pct\":" ), "Ok account must have quota fields; got: {json}" );

  let trimmed = json.trim();
  assert!( trimmed.starts_with( '[' ), "JSON must start with '['; got: {json}" );
  assert!( trimmed.ends_with(   ']' ), "JSON must end with ']'; got: {json}" );
}

// ── SortStrategy / PreferStrategy enum parsing ────────────────────────────────

/// AC-09 — `SortStrategy::parse` rejects unknown values with descriptive error.
#[ test ]
fn test_sort_strategy_parse_invalid_rejected()
{
  let err = SortStrategy::parse( "bogus" ).unwrap_err();
  assert!( err.contains( "bogus" ),  "error must name the bad value; got: {err}" );
  assert!( err.contains( "name" ),   "error must name valid values; got: {err}" );
  assert!( err.contains( "renew" ),  "error must name valid values; got: {err}" );
  assert!( err.contains( "renews" ), "error must name valid values; got: {err}" );
}

/// AC-10 — `PreferStrategy::parse` rejects unknown values with descriptive error.
#[ test ]
fn test_prefer_strategy_parse_invalid_rejected()
{
  let err = PreferStrategy::parse( "bogus" ).unwrap_err();
  assert!( err.contains( "bogus" ),  "error must name the bad value; got: {err}" );
  assert!( err.contains( "any" ),    "error must name valid values; got: {err}" );
  assert!( err.contains( "opus" ),   "error must name valid values; got: {err}" );
  assert!( err.contains( "sonnet" ), "error must name valid values; got: {err}" );
}

// ── sort display order via render_text ────────────────────────────────────────

/// AC-13 — `render_json` output is NOT sorted by `sort::` strategy.
#[ test ]
fn test_json_unaffected_by_sort()
{
  let accounts = vec![
    mk_aq_sort( "zzz@test.com", 30.0, FAR_FUTURE_MS ),
    mk_aq_sort( "aaa@test.com", 80.0, FAR_FUTURE_MS ),
  ];
  let json = render_json( &accounts );
  let zzz_pos = json.find( "zzz@test.com" ).unwrap_or( 0 );
  let aaa_pos = json.find( "aaa@test.com" ).unwrap_or( usize::MAX );
  assert!(
    zzz_pos < aaa_pos,
    "render_json must preserve input order; zzz first in input must appear first in JSON",
  );
}

/// AC-11 — `sort::`-driven single-strategy footer recommends the first eligible account.
#[ test ]
fn test_sort_recommendation_unaffected_by_sort_strategy()
{
  let a = mk_aq_sort( "a@x.com", 20.0, FAR_FUTURE_MS );
  let b = mk_aq_sort( "b@x.com", 75.0, FAR_FUTURE_MS );
  let mut cur = mk_aq_sort( "cur@x.com", 10.0, FAR_FUTURE_MS );
  cur.is_current = true;
  let accounts = vec![ a, b, cur ];

  let output = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false,
  );

  assert!( output.contains( "a@x.com" ), "output must contain a@x.com; got:\n{output}" );
  assert!(
    output.contains( "Next (name) ·" ),
    "footer must show 'Next (name) ·'; got:\n{output}",
  );
  assert!(
    output.contains( "Next (name) · a@x.com" ),
    "footer must recommend a@x.com (first alphabetically under sort::name); got:\n{output}",
  );
}

// ── Three-tier grouping ───────────────────────────────────────────────────────

/// TT-T07/T08 — three-tier grouping: 🟢 → 🟡 → 🔴 overrides sort order.
#[ test ]
fn test_three_tier_grouping_green_before_yellow_before_red()
{
  let a = mk_named_aq(     "a@x.com", 97.0, 0.0  );
  let b = mk_named_aq(     "b@x.com", 10.0, 10.0 );
  let c = mk_named_aq_err( "c@x.com"             );
  let accounts = vec![ a, b, c ];
  let output = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false,
  );
  let pos_a = output.find( "a@x.com" ).expect( "a@x.com must appear in output" );
  let pos_b = output.find( "b@x.com" ).expect( "b@x.com must appear in output" );
  let pos_c = output.find( "c@x.com" ).expect( "c@x.com must appear in output" );
  assert!( pos_b < pos_a, "🟢(b) must appear before 🟡(a). Got:\n{output}" );
  assert!( pos_a < pos_c, "🟡(a) must appear before 🔴(c). Got:\n{output}" );
}

/// FT-16 of feature/009 — within 🟡 tier, session-exhausted appears before weekly-exhausted.
#[ test ]
fn test_ft16_009_yellow_tier_session_before_weekly()
{
  let a = mk_named_aq( "a@x.com", 10.0, 98.0 );
  let b = mk_named_aq( "b@x.com", 99.0, 30.0 );
  let c = mk_named_aq( "c@x.com", 97.0, 50.0 );
  let d = mk_named_aq( "d@x.com", 10.0, 10.0 );
  let accounts = vec![ a, b, c, d ];

  let output = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false,
  );

  let pos_d = output.find( "d@x.com" ).expect( "d@x.com must appear" );
  let pos_b = output.find( "b@x.com" ).expect( "b@x.com must appear" );
  let pos_c = output.find( "c@x.com" ).expect( "c@x.com must appear" );
  let pos_a = output.find( "a@x.com" ).expect( "a@x.com must appear" );

  assert!( pos_d < pos_b, "🟢(d) must appear before session-yellow(b) (FT-16/009 AC-26);\n{output}" );
  assert!( pos_b < pos_a, "session-exhausted(b) must appear before weekly-exhausted(a) (FT-16/009 AC-26);\n{output}" );
  assert!( pos_c < pos_a, "session-exhausted(c) must appear before weekly-exhausted(a) (FT-16/009 AC-26);\n{output}" );
  assert!( pos_b < pos_c, "within session-yellow sub-group, alpha order must be preserved (FT-16/009 AC-26);\n{output}" );
}

/// FT-15 of feature/020 — `desc::1` reverses within each 🟡 sub-group but does NOT swap sub-group order.
#[ test ]
fn test_ft15_020_yellow_sub_grouping_not_reversed_by_desc()
{
  let a = mk_named_aq( "a@x.com", 99.0, 30.0 );
  let b = mk_named_aq( "b@x.com", 97.0, 50.0 );
  let c = mk_named_aq( "c@x.com", 10.0, 10.0 );
  let z = mk_named_aq( "z@x.com", 10.0, 98.0 );
  let accounts = vec![ a, b, c, z ];

  let output = render_text(
    &accounts, SortStrategy::Name, Some( true ), PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false,
  );

  let pos_c = output.find( "c@x.com" ).expect( "c@x.com must appear" );
  let pos_b = output.find( "b@x.com" ).expect( "b@x.com must appear" );
  let pos_a = output.find( "a@x.com" ).expect( "a@x.com must appear" );
  let pos_z = output.find( "z@x.com" ).expect( "z@x.com must appear" );

  assert!( pos_b < pos_z, "session-exhausted(b) must appear before weekly-exhausted(z) even with desc::1 (FT-15/020 AC-14);\n{output}" );
  assert!( pos_a < pos_z, "session-exhausted(a) must appear before weekly-exhausted(z) even with desc::1 (FT-15/020 AC-14);\n{output}" );
  assert!( pos_c < pos_b, "🟢(c) must appear before session-yellow(b) (FT-15/020 AC-14);\n{output}" );
  assert!( pos_b < pos_a, "within session-yellow, desc::1 puts b before a (FT-15/020 AC-14);\n{output}" );
}

// ── Footer: no eligible candidate ─────────────────────────────────────────────

/// FT-08 of feature/020 — footer omits the `→ Next` recommendation line when no eligible
/// candidate exists (all accounts are `is_current`).
#[ test ]
fn test_ft08_020_footer_omits_recommendation_when_no_eligible_candidate()
{
  let mut a = mk_aq_sort( "a@test.com", 30.0, FAR_FUTURE_MS );
  let mut b = mk_aq_sort( "b@test.com", 60.0, FAR_FUTURE_MS );
  a.is_current = true;
  b.is_current = true;
  let accounts = vec![ a, b ];

  let output = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false,
  );

  assert!(
    !output.contains( "→ Next (" ),
    "footer must omit recommendation line when no eligible candidate (FT-08/020), got:\n{output}",
  );
}

// ── Footer: model label ───────────────────────────────────────────────────────

/// FT-28 of feature/009 — footer shows session model (sonnet/opus) for the
/// recommended account, based on `seven_day_sonnet` utilization threshold.
#[ test ]
fn test_ft28_009_footer_model_label()
{
  // Scenario 1: recommended account has seven_day_sonnet.utilization = 50.0
  //             → sonnet_left = 50.0 ≥ 15.0 → session model = sonnet
  let mut current_1 = mk_aq_sort_weekly( "a@x.com", 10.0, 10.0, 50.0 );
  current_1.is_current = true;
  let sonnet_ok = mk_aq_sort_weekly( "b@x.com", 10.0, 10.0, 50.0 );
  let output = render_text(
    &[ current_1, sonnet_ok ], SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false,
  );
  assert!(
    output.contains( "· sonnet" ),
    "FT-28 scenario 1: footer line 2 must show '· sonnet' when sonnet_left=50% ≥ 10%; got:\n{output}",
  );

  // Scenario 2: recommended account has seven_day_sonnet.utilization = 91.0
  //             → sonnet_left = 9.0 < 10.0 → session model = opus (override fires)
  let mut current_2 = mk_aq_sort_weekly( "a@x.com", 10.0, 10.0, 50.0 );
  current_2.is_current = true;
  let mut opus_override = mk_aq_sort_weekly( "c@x.com", 10.0, 10.0, 50.0 );
  if let Ok( ref mut data ) = opus_override.result
  {
    if let Some( ref mut son ) = data.seven_day_sonnet
    {
      son.utilization = 91.0;
    }
  }
  let output = render_text(
    &[ current_2, opus_override ], SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false,
  );
  assert!(
    output.contains( "· opus" ),
    "FT-28 scenario 2: footer line 2 must show '· opus' when sonnet_left=9% < 10%; got:\n{output}",
  );
}

// ── BUG-334 reproducer ────────────────────────────────────────────────────

/// BUG-334 — `ColsVisibility::default_set()` must hide the dead `7d(Son)` column by default.
///
/// # Root Cause
/// `seven_day_sonnet` has been universally `None` for every account since Anthropic's
/// 2026-06-25 API restructuring (`docs/algorithm/009_oauth_usage_response_migration.md`),
/// yet `default_set()` still set `d7_son: true`, so the column was shown by default and
/// always rendered blank.
///
/// # Why Not Caught
/// No test asserted `default_set().d7_son`'s value directly; the column's blank rendering
/// was indistinguishable from a legitimately-absent-but-shown column in visual inspection.
///
/// # Fix Applied
/// Flipped `default_set()`'s `d7_son` field from `true` to `false`, matching the already-
/// hidden-by-default siblings `sub` and `7d Son Reset`. Re-enabled via `cols::+7d_son`.
///
/// # Prevention
/// A column's default visibility must be re-audited whenever its underlying data source
/// is deprecated — an always-true default silently ages into an always-blank column.
///
/// # Pitfall
/// Do not re-enable `d7_son` by default even if convenient — only flip it back if Anthropic
/// restores the underlying API field (out of scope until that happens).
///
/// Spec: [`docs/feature/009_token_usage.md` AC-22]
#[ doc = "bug_reproducer(BUG-334)" ]
#[ test ]
fn mre_bug334_d7_son_hidden_by_default()
{
  assert!(
    !ColsVisibility::default_set().d7_son,
    "BUG-334: d7_son must be hidden by default — seven_day_sonnet has been universally \
     None since the 2026-06-25 API restructuring; showing it by default always renders blank",
  );
}

/// `cols::+7d_son` override re-enables the `7d(Son)` column — zero prior coverage of this
/// specific override existed before BUG-334 (its sibling `7d_son_reset` override is covered
/// elsewhere; this is a distinct field with its own `apply_modifier` match arm).
#[ test ]
fn cols_plus_7d_son_override_shows_column()
{
  let cols = ColsVisibility::parse( "+7d_son" ).expect( "cols::+7d_son must parse" );
  assert!(
    cols.d7_son,
    "cols::+7d_son must re-enable the 7d(Son) column even though it is hidden by default",
  );
}
