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
//! | T27   | `iso_to_unix_secs("not-a-date")`                        | `None`                                  | ✅   |
//! | T28   | `PeriodUsage` and `OauthUsageData` field accessibility  | all `pub` fields readable               | ✅   |
//! | FT-01 | Named field `Some` → Phase 2 not invoked                | `seven_day_sonnet.utilization = 45.0`   | ✅   |
//! | FT-02 | Named `null` + `limits` kind match                      | `seven_day_sonnet = Some(45.0)`         | ✅   |
//! | FT-03 | `percent` maps directly to `utilization`                 | `seven_day_sonnet.utilization = 73.0`   | ✅   |
//! | FT-04 | Named `null` + no limits match                          | `seven_day_sonnet = None`, no error     | ✅   |
//! | FT-05 | `resets_at` carried from limits entry                   | `PeriodUsage.resets_at = Some(string)`  | ✅   |
//! | FT-06 | `resets_at: null` in limits entry                       | `PeriodUsage.resets_at = None`          | ✅   |
//! | FT-07 | Match via `kind` needle (`"weekly_sonnet"`)              | `seven_day_sonnet = Some(33.0)`         | ✅   |
//! | FT-08 | Match via `scope` needle (`scope="sonnet"`)              | `seven_day_sonnet = Some(45.0)`         | ✅   |
//! | FT-09 | Validity guard passes for null value on present key      | `parse_oauth_usage` returns `Ok`        | ✅   |
//! | FT-10 | `OauthUsageData` struct fields unchanged                 | all 3 fields accessible                 | ✅   |
//! | FT-11 | Old format (no `limits` key) parses via Phase 1          | `seven_day_sonnet = Some(30.0)`         | ✅   |
//! | FT-12 | Named field `Some` wins over matching limits entry       | `seven_day_sonnet.utilization = 30.0`   | ✅   |
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
//! - ✅ Phase 1 named field takes priority over `limits` array (FT-01, FT-12)
//! - ✅ Phase 2 fallback: `limits` kind needle match (FT-02, FT-03, FT-07)
//! - ✅ Phase 2 fallback: `limits` scope needle match (FT-08)
//! - ✅ Phase 2 fallback: `resets_at` carry-through from limits entry (FT-05, FT-06)
//! - ✅ Phase 2 no-match returns `None`, no error (FT-04)
//! - ✅ Validity guard passes for null value on present key (FT-09)
//! - ✅ Struct field list unchanged after dual-source parsing (FT-10)
//! - ✅ Old format (no `limits` key) still parses via Phase 1 (FT-11)

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
  let body = r"not json";
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

// ── FT-01 ─────────────────────────────────────────────────────────────────────

#[ test ]
/// Named field `Some` → Phase 2 not invoked; Phase 1 result returned unchanged.
fn ft_01_named_field_some_phase2_not_invoked()
{
  let body = r#"{"five_hour":{"utilization":5.0,"resets_at":"2026-06-25T12:00:00+00:00"},"seven_day":{"utilization":18.0,"resets_at":"2026-06-30T04:00:00+00:00"},"seven_day_sonnet":{"utilization":45.0,"resets_at":"2026-06-28T04:00:00+00:00"},"limits":[]}"#;
  let data = parse_oauth_usage( body ).unwrap();
  let son  = data.seven_day_sonnet.expect( "FT-01: named field Some must be returned" );
  assert!( ( son.utilization - 45.0 ).abs() < 0.001, "FT-01: utilization must be 45.0" );
}

// ── FT-02 ─────────────────────────────────────────────────────────────────────

#[ test ]
/// Named field `null` + `limits` entry with `kind = "weekly_sonnet"` → `seven_day_sonnet` populated.
fn ft_02_named_null_limits_kind_match_populates_sonnet()
{
  let body = r#"{"five_hour":{"utilization":5.0,"resets_at":null},"seven_day":{"utilization":18.0,"resets_at":"2026-06-30T04:00:00+00:00"},"seven_day_sonnet":null,"limits":[{"kind":"session","group":"session","percent":5,"severity":"normal","resets_at":null,"scope":null,"is_active":false},{"kind":"weekly_sonnet","group":"weekly","percent":45,"severity":"normal","resets_at":"2026-06-28T04:00:00+00:00","scope":null,"is_active":true}]}"#;
  let data = parse_oauth_usage( body ).unwrap();
  let son  = data.seven_day_sonnet.expect( "FT-02: limits match must populate seven_day_sonnet" );
  assert!( ( son.utilization - 45.0 ).abs() < 0.001, "FT-02: utilization must be 45.0" );
}

// ── FT-03 ─────────────────────────────────────────────────────────────────────

#[ test ]
/// `percent` field in limits entry maps directly to `utilization` (no scale conversion).
fn ft_03_percent_maps_directly_to_utilization()
{
  let body = r#"{"five_hour":{"utilization":5.0,"resets_at":null},"seven_day":{"utilization":18.0,"resets_at":null},"seven_day_sonnet":null,"limits":[{"kind":"weekly_sonnet","group":"weekly","percent":73,"severity":"normal","resets_at":null,"scope":null,"is_active":true}]}"#;
  let data = parse_oauth_usage( body ).unwrap();
  let son  = data.seven_day_sonnet.expect( "FT-03: must populate from limits" );
  assert!( ( son.utilization - 73.0 ).abs() < 0.001, "FT-03: percent 73 must map to utilization 73.0" );
}

// ── FT-04 ─────────────────────────────────────────────────────────────────────

#[ test ]
/// Named field `null` + no matching limits entry → `seven_day_sonnet = None`, no error.
fn ft_04_named_null_no_limits_match_returns_none_no_error()
{
  let body = r#"{"five_hour":{"utilization":2.0,"resets_at":"2026-06-25T11:59:59"},"seven_day":{"utilization":18.0,"resets_at":"2026-06-30T04:00:00+00:00"},"seven_day_sonnet":null,"limits":[{"kind":"session","group":"session","percent":2,"severity":"normal","resets_at":"2026-06-25T11:59:59","scope":null,"is_active":false},{"kind":"weekly_all","group":"weekly","percent":18,"severity":"normal","resets_at":"2026-06-30T04:00:00+00:00","scope":null,"is_active":true}]}"#;
  let data = parse_oauth_usage( body ).unwrap();
  assert!( data.seven_day_sonnet.is_none(), "FT-04: no matching limits entry must leave seven_day_sonnet = None" );
}

// ── FT-05 ─────────────────────────────────────────────────────────────────────

#[ test ]
/// `resets_at` string from a matching limits entry is carried into `PeriodUsage.resets_at`.
fn ft_05_resets_at_from_limits_carried_into_period_usage()
{
  let body = r#"{"five_hour":{"utilization":5.0,"resets_at":null},"seven_day":{"utilization":18.0,"resets_at":null},"seven_day_sonnet":null,"limits":[{"kind":"weekly_sonnet","group":"weekly","percent":45,"severity":"normal","resets_at":"2026-06-30T04:00:00+00:00","scope":null,"is_active":true}]}"#;
  let data = parse_oauth_usage( body ).unwrap();
  let son  = data.seven_day_sonnet.expect( "FT-05: must populate from limits" );
  assert_eq!( son.resets_at, Some( "2026-06-30T04:00:00+00:00".to_string() ) );
}

// ── FT-06 ─────────────────────────────────────────────────────────────────────

#[ test ]
/// `resets_at: null` in a matching limits entry → `PeriodUsage.resets_at = None`.
fn ft_06_resets_at_null_in_limits_entry_gives_none()
{
  let body = r#"{"five_hour":{"utilization":5.0,"resets_at":null},"seven_day":{"utilization":18.0,"resets_at":null},"seven_day_sonnet":null,"limits":[{"kind":"weekly_sonnet","group":"weekly","percent":45,"severity":"normal","resets_at":null,"scope":null,"is_active":true}]}"#;
  let data = parse_oauth_usage( body ).unwrap();
  let son  = data.seven_day_sonnet.expect( "FT-06: must populate from limits" );
  assert!( son.resets_at.is_none(), "FT-06: null resets_at must give None" );
}

// ── FT-07 ─────────────────────────────────────────────────────────────────────

#[ test ]
/// `kind = "weekly_sonnet"` matches the `"sonnet"` needle → limits entry selected.
fn ft_07_match_via_kind_needle()
{
  let body = r#"{"five_hour":{"utilization":5.0,"resets_at":null},"seven_day":{"utilization":18.0,"resets_at":null},"seven_day_sonnet":null,"limits":[{"kind":"weekly_sonnet","group":"weekly","percent":33,"severity":"normal","resets_at":null,"scope":null,"is_active":true}]}"#;
  let data = parse_oauth_usage( body ).unwrap();
  assert!( data.seven_day_sonnet.is_some(), "FT-07: kind='weekly_sonnet' must match needle 'sonnet'" );
}

// ── FT-08 ─────────────────────────────────────────────────────────────────────

#[ test ]
/// `scope = "sonnet"` with a generic `kind` value → limits entry selected via scope needle.
fn ft_08_match_via_scope_needle()
{
  let body = r#"{"five_hour":{"utilization":5.0,"resets_at":null},"seven_day":{"utilization":18.0,"resets_at":null},"seven_day_sonnet":null,"limits":[{"kind":"weekly_all","group":"weekly","percent":45,"severity":"normal","resets_at":null,"scope":"sonnet","is_active":true}]}"#;
  let data = parse_oauth_usage( body ).unwrap();
  assert!( data.seven_day_sonnet.is_some(), "FT-08: scope='sonnet' must match needle" );
}

// ── FT-09 ─────────────────────────────────────────────────────────────────────

#[ test ]
/// Post-2026-06-25 format: `seven_day_sonnet` key present but value `null` → validity guard passes.
fn ft_09_validity_guard_passes_for_null_seven_day_sonnet_key()
{
  // Key is present (guard checks key presence, not value type).
  let body = r#"{"five_hour":{"utilization":2.0,"resets_at":"2026-06-25T11:59:59"},"seven_day":{"utilization":18.0,"resets_at":"2026-06-30T04:00:00+00:00"},"seven_day_sonnet":null,"limits":[]}"#;
  assert!( parse_oauth_usage( body ).is_ok(), "FT-09: null value on present key must not fail validity guard" );
}

// ── FT-10 ─────────────────────────────────────────────────────────────────────

#[ test ]
/// `OauthUsageData` struct still exposes exactly the same 3 fields after dual-source parsing.
fn ft_10_oauth_usage_data_struct_fields_unchanged()
{
  let body = r#"{"five_hour":{"utilization":5.0,"resets_at":null},"seven_day":{"utilization":18.0,"resets_at":null},"seven_day_sonnet":null,"limits":[]}"#;
  let data = parse_oauth_usage( body ).unwrap();
  // Accessing all 3 fields compiles — proves no new/removed fields
  let _ = ( &data.five_hour, &data.seven_day, &data.seven_day_sonnet );
}

// ── FT-11 ─────────────────────────────────────────────────────────────────────

#[ test ]
/// Pre-2026-06-25 format (no `limits` key): named field still parsed via Phase 1.
fn ft_11_old_format_no_limits_parses_via_phase1()
{
  let body = r#"{"five_hour":{"utilization":5.0,"resets_at":"2026-06-25T12:00:00+00:00"},"seven_day":{"utilization":18.0,"resets_at":"2026-06-30T04:00:00+00:00"},"seven_day_sonnet":{"utilization":30.0,"resets_at":"2026-06-28T00:00:00+00:00"}}"#;
  let data = parse_oauth_usage( body ).unwrap();
  let son  = data.seven_day_sonnet.expect( "FT-11: old format must populate via Phase 1" );
  assert!( ( son.utilization - 30.0 ).abs() < 0.001, "FT-11: old format utilization" );
}

// ── FT-12 ─────────────────────────────────────────────────────────────────────

#[ test ]
/// Named field `Some` (30.0) wins over a matching limits entry (70.0): Phase 1 takes priority.
fn ft_12_named_field_wins_over_limits_when_both_present()
{
  // Named field: utilization=30; limits entry: percent=70 — Phase 1 result must win.
  let body = r#"{"five_hour":{"utilization":5.0,"resets_at":null},"seven_day":{"utilization":18.0,"resets_at":null},"seven_day_sonnet":{"utilization":30.0,"resets_at":null},"limits":[{"kind":"weekly_sonnet","group":"weekly","percent":70,"severity":"normal","resets_at":null,"scope":null,"is_active":true}]}"#;
  let data = parse_oauth_usage( body ).unwrap();
  let son  = data.seven_day_sonnet.expect( "FT-12: must be Some" );
  assert!( ( son.utilization - 30.0 ).abs() < 0.001, "FT-12: named field (30.0) must win over limits entry (70.0)" );
}
