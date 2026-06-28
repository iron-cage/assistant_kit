# Test: Feature 066 — Dual-Source OAuth Quota Parsing

Feature behavioral requirement test cases for `docs/feature/066_dual_source_quota_parsing.md`. Each FT case maps to one acceptance criterion. All tests live in `claude_quota/tests/oauth_usage_test.rs`.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | Named field `Some` → Phase 2 not invoked; result unchanged | AC-01 |
| FT-02 | Named field `null` + `limits` has matching sonnet entry → `seven_day_sonnet` populated | AC-02 |
| FT-03 | `limits` entry `percent` maps directly to `utilization` (no scale change) | AC-03 |
| FT-04 | Named field `null` + no matching entry in `limits` → `seven_day_sonnet = None`, no error | AC-04 |
| FT-05 | `limits` entry `resets_at` carried into `PeriodUsage.resets_at` | AC-07 |
| FT-06 | `limits` entry `resets_at = null` → `PeriodUsage.resets_at = None` | AC-07 |
| FT-07 | `limits` entry matched by `kind` needle (e.g. `"weekly_sonnet"`) | AC-02 |
| FT-08 | `limits` entry matched by `scope` needle (e.g. `scope: "sonnet"`) | AC-02 |
| FT-09 | Body validity guard passes when `seven_day_sonnet` key present but `null` | AC-08 |
| FT-10 | `OauthUsageData` struct fields unchanged — no new fields, no removed fields | AC-06 |
| FT-11 | Pre-2026-06-25 response (no `limits` array) still parses correctly via Phase 1 | AC-01 |
| FT-12 | Response with both named field `Some` AND matching `limits` entry → named field wins | AC-01 |

### Test Function Naming

All tests follow the convention `ft_NNN_<description>` in `claude_quota/tests/oauth_usage_test.rs`. MRE (minimal reproducer) tests for future bugs follow `mre_bugNNN_<description>`.

### FT Case Descriptions

**FT-01** — Named field populated, Phase 2 skipped
Input: old-format body with `"seven_day_sonnet": {"utilization": 45.0, "resets_at": "2026-06-30T..."}`.
Expected: `OauthUsageData.seven_day_sonnet = Some(PeriodUsage { utilization: 45.0, resets_at: Some("...") })`. No `limits` scan occurs.

**FT-02** — Named field null, `limits` match found
Input: new-format body with `"seven_day_sonnet": null` and `limits` containing `{"kind": "weekly_sonnet", "percent": 45, "resets_at": "2026-06-30T..."}`.
Expected: `seven_day_sonnet = Some(PeriodUsage { utilization: 45.0, resets_at: Some("...") })`.

**FT-03** — `percent` → `utilization` direct mapping
Input: `limits` entry with `"percent": 73`.
Expected: `PeriodUsage.utilization = 73.0_f64`. No multiplication or division.

**FT-04** — Named field null, no `limits` match
Input: body with `"seven_day_sonnet": null` and `limits` containing only `session` and `weekly_all` entries.
Expected: `seven_day_sonnet = None`. `Ok(OauthUsageData {...})` returned — no error.

**FT-05** — `resets_at` from `limits` entry preserved
Input: `limits` match with `"resets_at": "2026-06-30T04:00:00+00:00"`.
Expected: `PeriodUsage.resets_at = Some("2026-06-30T04:00:00+00:00".to_string())`.

**FT-06** — `resets_at` null in `limits` entry
Input: `limits` match with `"resets_at": null`.
Expected: `PeriodUsage.resets_at = None`.

**FT-07** — Match via `kind` needle
Input: entry with `"kind": "weekly_sonnet"`.
Expected: matched and populated.

**FT-08** — Match via `scope` needle
Input: entry with `"kind": "weekly_all", "scope": "sonnet"`.
Expected: matched and populated.

**FT-09** — Validity guard passes for new format
Input: post-2026-06-25 body containing `"seven_day_sonnet": null` (key present, value null).
Expected: `parse_oauth_usage()` does not return `Err(ResponseParse("five_hour/seven_day/seven_day_sonnet"))`. Guard passes because `"seven_day_sonnet"` substring is present.

**FT-10** — Struct unchanged
Structural: `OauthUsageData` has exactly 3 fields: `five_hour`, `seven_day`, `seven_day_sonnet`. No new fields added by this feature.

**FT-11** — Old format still parses
Input: pre-2026-06-25 body with no `limits` key and `"seven_day_sonnet": {"utilization": 30.0, ...}`.
Expected: Phase 1 populates all fields; `Ok(OauthUsageData {...})`.

**FT-12** — Named field wins over `limits` when both present
Input: body with `"seven_day_sonnet": {"utilization": 30.0, ...}` AND `limits` entry with `percent: 70`.
Expected: `seven_day_sonnet.utilization = 30.0` (Phase 1 result, not Phase 2).

### Cross-References

| File | Relationship |
|------|-------------|
| [../../../docs/feature/066_dual_source_quota_parsing.md](../../../docs/feature/066_dual_source_quota_parsing.md) | Feature spec — ACs being tested |
| [../../../docs/algorithm/009_oauth_usage_response_migration.md](../../../docs/algorithm/009_oauth_usage_response_migration.md) | Algorithm pseudocode and `limits` field semantics |
