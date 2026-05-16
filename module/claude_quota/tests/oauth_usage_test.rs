//! Unit tests: OAuth usage endpoint response parsing and `iso_to_unix_secs`.
//!
//! All tests are offline — no network, no `ureq` in dev-dependencies.
//! `parse_oauth_usage` and `iso_to_unix_secs` are always available (no `enabled` feature).
//!
//! ## Test Matrix
//!
//! | ID  | Scenario                                              | Expected                             | Status |
//! |-----|-------------------------------------------------------|--------------------------------------|--------|
//! | T17 | Full JSON: all three periods Some, valid values        | `OauthUsageData` with all `Some`     | ✅     |
//! | T18 | `seven_day_sonnet: null`                              | `seven_day_sonnet: None`             | ✅     |
//! | T19 | `five_hour: null`                                     | `five_hour: None`                    | ✅     |
//! | T20 | All three periods `null`                              | All three `None`; `Ok(OauthUsageData)` | ✅   |
//! | T21 | Completely invalid JSON (`"not json"`)                | `Err(ResponseParse(...))`            | ✅     |
//! | T22 | Period object present but missing `utilization` key   | `Err(ResponseParse(...))`            | ✅     |
//! | T23 | `utilization` value not a number (`"abc"`)            | `Err(ResponseParse(...))`            | ✅     |
//! | T24 | `resets_at: null` (explicit null; utilization present) | `PeriodUsage { resets_at: None }`   | ✅     |
//! | T25 | Realistic body: `seven_day.utilization = 46.0`        | `seven_day.utilization == 46.0`      | ✅     |
//! | T26 | `iso_to_unix_secs("2026-05-15T12:20:00.499185+00:00")` | `Some(1778847600)`                  | ✅     |
//! | T27 | `iso_to_unix_secs("not-a-date")`                      | `None`                               | ✅     |
//! | T28 | `PeriodUsage` and `OauthUsageData` field accessibility | all `pub` fields readable            | ✅     |
//!
//! ## Corner Cases Covered
//!
//! - ✅ Happy path: all three periods present with valid values (T17)
//! - ✅ Each period null individually (T18, T19) or all null (T20)
//! - ✅ Invalid JSON body (T21)
//! - ✅ Missing required field in period object (T22)
//! - ✅ Non-numeric utilization value (T23)
//! - ✅ `resets_at: null` with utilization present (T24)
//! - ✅ Realistic API response body (T25)
//! - ✅ Known-date `iso_to_unix_secs` validation (T26)
//! - ✅ Invalid string returns `None` from `iso_to_unix_secs` (T27)
//! - ✅ Field accessibility on both public structs (T28)

use claude_quota::{ parse_oauth_usage, iso_to_unix_secs, OauthUsageData, PeriodUsage, QuotaError };

// ── T17 ───────────────────────────────────────────────────────────────────────

#[ test ]
fn t17_parse_oauth_usage_valid_all_some()
{
  let body = r#"{
    "five_hour":        { "utilization": 10.0, "resets_at": "2026-05-15T12:20:00+00:00" },
    "seven_day":        { "utilization": 46.0, "resets_at": "2026-05-20T04:00:00+00:00" },
    "seven_day_sonnet": { "utilization": 72.0, "resets_at": "2026-05-20T04:00:00+00:00" }
  }"#;
  let result = parse_oauth_usage( body ).expect( "T17: should parse valid JSON" );
  let five   = result.five_hour.as_ref().expect( "T17: five_hour should be Some" );
  let seven  = result.seven_day.as_ref().expect( "T17: seven_day should be Some" );
  let sonnet = result.seven_day_sonnet.as_ref().expect( "T17: seven_day_sonnet should be Some" );
  assert!( ( five.utilization   - 10.0 ).abs() < 0.001, "T17: five_hour.utilization" );
  assert!( ( seven.utilization  - 46.0 ).abs() < 0.001, "T17: seven_day.utilization" );
  assert!( ( sonnet.utilization - 72.0 ).abs() < 0.001, "T17: seven_day_sonnet.utilization" );
  assert_eq!( five.resets_at.as_deref(), Some( "2026-05-15T12:20:00+00:00" ), "T17: five_hour.resets_at" );
}

// ── T18 ───────────────────────────────────────────────────────────────────────

#[ test ]
fn t18_parse_oauth_usage_sonnet_null()
{
  let body = r#"{
    "five_hour":        { "utilization": 10.0, "resets_at": "2026-05-15T12:20:00+00:00" },
    "seven_day":        { "utilization": 46.0, "resets_at": "2026-05-20T04:00:00+00:00" },
    "seven_day_sonnet": null
  }"#;
  let result = parse_oauth_usage( body ).expect( "T18: should parse with null sonnet" );
  assert!( result.five_hour.is_some(),        "T18: five_hour should be Some" );
  assert!( result.seven_day.is_some(),        "T18: seven_day should be Some" );
  assert!( result.seven_day_sonnet.is_none(), "T18: seven_day_sonnet should be None" );
}

// ── T19 ───────────────────────────────────────────────────────────────────────

#[ test ]
fn t19_parse_oauth_usage_five_hour_null()
{
  let body = r#"{
    "five_hour":        null,
    "seven_day":        { "utilization": 46.0, "resets_at": "2026-05-20T04:00:00+00:00" },
    "seven_day_sonnet": { "utilization": 72.0, "resets_at": "2026-05-20T04:00:00+00:00" }
  }"#;
  let result = parse_oauth_usage( body ).expect( "T19: should parse with null five_hour" );
  assert!( result.five_hour.is_none(),        "T19: five_hour should be None" );
  assert!( result.seven_day.is_some(),        "T19: seven_day should be Some" );
  assert!( result.seven_day_sonnet.is_some(), "T19: seven_day_sonnet should be Some" );
}

// ── T20 ───────────────────────────────────────────────────────────────────────

#[ test ]
fn t20_parse_oauth_usage_all_null()
{
  let body = r#"{"five_hour":null,"seven_day":null,"seven_day_sonnet":null}"#;
  let result = parse_oauth_usage( body ).expect( "T20: should parse with all null" );
  assert!( result.five_hour.is_none(),        "T20: five_hour should be None" );
  assert!( result.seven_day.is_none(),        "T20: seven_day should be None" );
  assert!( result.seven_day_sonnet.is_none(), "T20: seven_day_sonnet should be None" );
}

// ── T21 ───────────────────────────────────────────────────────────────────────

#[ test ]
fn t21_parse_oauth_usage_invalid_json()
{
  let body = r#"not json"#;
  let err  = parse_oauth_usage( body )
    .expect_err( "T21: should fail on invalid JSON" );
  assert!(
    matches!( err, QuotaError::ResponseParse( _ ) ),
    "T21: expected ResponseParse, got {err:?}"
  );
}

// ── T22 ───────────────────────────────────────────────────────────────────────

#[ test ]
fn t22_parse_oauth_usage_period_missing_utilization()
{
  let body = r#"{"five_hour":{"resets_at":"2026-05-15T12:00:00+00:00"},"seven_day":null,"seven_day_sonnet":null}"#;
  let err  = parse_oauth_usage( body )
    .expect_err( "T22: should fail when utilization missing" );
  assert!(
    matches!( err, QuotaError::ResponseParse( _ ) ),
    "T22: expected ResponseParse, got {err:?}"
  );
}

// ── T23 ───────────────────────────────────────────────────────────────────────

#[ test ]
fn t23_parse_oauth_usage_utilization_not_a_number()
{
  let body = r#"{"five_hour":{"utilization":"abc","resets_at":"2026-05-15T12:00:00+00:00"},"seven_day":null,"seven_day_sonnet":null}"#;
  let err  = parse_oauth_usage( body )
    .expect_err( "T23: should fail when utilization is a string" );
  assert!(
    matches!( err, QuotaError::ResponseParse( _ ) ),
    "T23: expected ResponseParse, got {err:?}"
  );
}

// ── T24 ───────────────────────────────────────────────────────────────────────

#[ test ]
fn t24_parse_oauth_usage_resets_at_null()
{
  let body = r#"{"five_hour":{"utilization":5.0,"resets_at":null},"seven_day":null,"seven_day_sonnet":null}"#;
  let result = parse_oauth_usage( body ).expect( "T24: should parse resets_at null" );
  let five   = result.five_hour.as_ref().expect( "T24: five_hour should be Some" );
  assert!( ( five.utilization - 5.0 ).abs() < 0.001, "T24: utilization" );
  assert!( five.resets_at.is_none(), "T24: resets_at should be None" );
}

// ── T25 ───────────────────────────────────────────────────────────────────────

#[ test ]
fn t25_parse_oauth_usage_realistic_body()
{
  let body = concat!(
    r#"{"five_hour":{"utilization":0.0,"resets_at":"2026-05-15T12:20:00+00:00"},"#,
    r#""seven_day":{"utilization":46.0,"resets_at":"2026-05-20T04:00:00+00:00"},"#,
    r#""seven_day_sonnet":{"utilization":72.0,"resets_at":"2026-05-20T04:00:00+00:00"}}"#,
  );
  let result = parse_oauth_usage( body ).expect( "T25: should parse realistic body" );
  let seven  = result.seven_day.as_ref().expect( "T25: seven_day should be Some" );
  assert!( ( seven.utilization - 46.0 ).abs() < 0.001, "T25: seven_day.utilization == 46.0" );
}

// ── T26 ───────────────────────────────────────────────────────────────────────

#[ test ]
fn t26_iso_to_unix_secs_known_date()
{
  // 2026-05-15T12:20:00.499185+00:00
  // Expected: 1778847600
  let result = iso_to_unix_secs( "2026-05-15T12:20:00.499185+00:00" );
  assert_eq!(
    result,
    Some( 1_778_847_600 ),
    "T26: iso_to_unix_secs known date mismatch: got {result:?}"
  );
}

// ── T27 ───────────────────────────────────────────────────────────────────────

#[ test ]
fn t27_iso_to_unix_secs_invalid_string()
{
  assert_eq!( iso_to_unix_secs( "not-a-date" ), None, "T27: should return None for invalid string" );
  assert_eq!( iso_to_unix_secs( "" ), None, "T27: should return None for empty string" );
}

// ── T28 ───────────────────────────────────────────────────────────────────────

#[ test ]
fn t28_field_accessibility()
{
  // PeriodUsage: direct field construction and access
  let period = PeriodUsage
  {
    utilization : 33.5,
    resets_at   : Some( "2026-01-01T00:00:00+00:00".to_string() ),
  };
  assert!( ( period.utilization - 33.5 ).abs() < 0.001, "T28: PeriodUsage.utilization" );
  assert!( period.resets_at.is_some(),                   "T28: PeriodUsage.resets_at" );

  // OauthUsageData: direct construction
  let data = OauthUsageData
  {
    five_hour        : Some( period ),
    seven_day        : None,
    seven_day_sonnet : None,
  };
  assert!( data.five_hour.is_some(),        "T28: OauthUsageData.five_hour" );
  assert!( data.seven_day.is_none(),        "T28: OauthUsageData.seven_day" );
  assert!( data.seven_day_sonnet.is_none(), "T28: OauthUsageData.seven_day_sonnet" );
}
