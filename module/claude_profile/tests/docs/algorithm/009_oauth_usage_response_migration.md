# Algorithm 009: OAuth Usage Response Dual-Source Parsing

AC test cases for `docs/algorithm/009_oauth_usage_response_migration.md`. Tests `parse_oauth_usage(body)` (entry point) and the private `scan_limits_for_kind(body, needles)` helper in `claude_quota/src/lib.rs`. Implementations reside in `claude_quota/tests/oauth_usage_test.rs` as `ft_01_*` through `ft_12_*`.

### AC Case Index

| AC | Short Name | Category | FT Mapping | Status |
|----|------------|----------|------------|--------|
| AC-1 | Phase 1 named field wins when `Some` — Phase 2 not invoked | Nominal | FT-01, FT-12 | ✅ |
| AC-2 | Phase 2 populates from `limits` array when named field is `null` | Nominal | FT-02, FT-07, FT-08 | ✅ |
| AC-3 | `percent` integer maps directly to `utilization` f64 (scale identity) | Nominal | FT-03 | ✅ |
| AC-4 | No matching `limits` entry returns `None` without error | Boundary | FT-04 | ✅ |
| AC-5 | `resets_at` string carried through from matched `limits` entry | Boundary | FT-05, FT-06 | ✅ |
| AC-6 | Body validity guard passes for new-format response (key present, value null) | Regression | FT-09 | ✅ |
| AC-7 | Pre-2026-06-25 format (no `limits` key) still parses via Phase 1 | Regression | FT-11 | ✅ |

---

### AC-1: Phase 1 named field wins when `Some`

- **Given:** Response body contains `"seven_day_sonnet": {"utilization": 45.0, "resets_at": "..."}` (named field is an object, not null); a `"limits":[...]` array also present with a matching entry at `"percent": 70`
- **When:** `parse_oauth_usage(body)` is called
- **Then:** Returns `OauthUsageData { seven_day_sonnet: Some(PeriodUsage { utilization: 45.0, ... }) }` — named-field value (Phase 1) wins; the `if seven_day_sonnet.is_none()` guard prevents Phase 2 from overriding a live `Some`
- **Note:** FT-01 tests named field alone; FT-12 tests both present simultaneously and confirms named field (30.0) beats the limits entry (70.0)

### AC-2: Phase 2 populates from `limits` array when named field is `null`

- **Given:** Response body contains `"seven_day_sonnet": null` and `"limits":[{"kind":"weekly_sonnet","percent":45,...}]`
- **When:** `parse_oauth_usage(body)` is called
- **Then:** Returns `seven_day_sonnet = Some(PeriodUsage { utilization: 45.0, ... })` — `scan_limits_for_kind(body, &["weekly_sonnet", "sonnet"])` matched the entry via the `"sonnet"` needle against `kind_val = "weekly_sonnet"`
- **Note:** Match succeeds via `kind` field (FT-07) or `scope` field (FT-08); both go through the same `kind_val.contains(n) || scope_val.contains(n)` check

### AC-3: `percent` integer maps directly to `utilization` f64

- **Given:** Response body contains `"seven_day_sonnet": null` and a matching `limits` entry with `"percent": 73` (integer)
- **When:** `parse_oauth_usage(body)` is called
- **Then:** `seven_day_sonnet.unwrap().utilization == 73.0_f64` — no scale conversion is applied; both `percent` (USED%) and `utilization` (USED%) share the 0–100 domain; the mapping is `utilization = percent as f64`

### AC-4: No matching `limits` entry returns `None` without error

- **Given:** Response body contains `"seven_day_sonnet": null` and `"limits":[{"kind":"session",...},{"kind":"weekly_all",...}]` — no entry whose `kind` or `scope` contains `"weekly_sonnet"` or `"sonnet"`
- **When:** `parse_oauth_usage(body)` is called
- **Then:** Returns `Ok(OauthUsageData { seven_day_sonnet: None })` — current API state (2026-06-25); `scan_limits_for_kind()` exhausts the array and returns `None`; the call succeeds without error

### AC-5: `resets_at` string carried through from matched `limits` entry

- **Given:** Response body contains `"seven_day_sonnet": null` and a matching `limits` entry with `"resets_at": "2026-06-30T04:00:00+00:00"` (FT-05) or `"resets_at": null` (FT-06)
- **When:** `parse_oauth_usage(body)` is called
- **Then:** For a non-null `resets_at`: `seven_day_sonnet.resets_at = Some("2026-06-30T04:00:00+00:00")`; for a null `resets_at`: `seven_day_sonnet.resets_at = None` — `parse_optional_string_in_block` handles both transparently

### AC-6: Body validity guard passes for new-format response

- **Given:** Response body contains all three named keys `"five_hour"`, `"seven_day"`, `"seven_day_sonnet"` with values; `"seven_day_sonnet"` value is `null`
- **When:** `parse_oauth_usage(body)` is called
- **Then:** Returns `Ok(...)` — the guard checks `body.contains("five_hour" or "seven_day" or "seven_day_sonnet")` using substring match on the body text; the key `"seven_day_sonnet"` is present in the body even when its value is `null`, so the guard passes without returning `Err`

### AC-7: Pre-2026-06-25 format without `limits` key parses via Phase 1

- **Given:** Response body in old format: `"seven_day_sonnet": {"utilization": 30.0, "resets_at": "2026-06-28T00:00:00+00:00"}` with no `"limits"` key present anywhere in the body
- **When:** `parse_oauth_usage(body)` is called
- **Then:** Returns `seven_day_sonnet = Some(PeriodUsage { utilization: 30.0, ... })` — Phase 1 populates from the named field; `seven_day_sonnet.is_none()` is `false`, so Phase 2 is skipped entirely; old API format continues to work
