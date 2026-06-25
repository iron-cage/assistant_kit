# Feature: Dual-Source OAuth Quota Parsing

### Scope

- **Purpose**: Maintain correct per-model quota data in `OauthUsageData` across Anthropic API format changes by parsing both the legacy named-field format (`seven_day_sonnet`, etc.) and the new `limits` array format.
- **Responsibility**: When `parse_oauth_usage()` receives a response where the `seven_day_sonnet` named field is `null`, it scans the `limits` array for a matching per-model entry and populates the field from that source. Consumers receive a populated `seven_day_sonnet` (and eventually `seven_day_opus`) regardless of which API format is active.
- **In Scope**: `parse_oauth_usage()` in `claude_quota/src/lib.rs` — Phase 1 (named-field) + Phase 2 (limits-array fallback); `scan_limits_for_kind()` helper; `percent` → `utilization` mapping; backward compatibility with pre-2026-06-25 responses; forward compatibility when Anthropic re-enables per-model `limits` entries. `OauthUsageData` struct and all downstream consumer signatures unchanged.
- **Out of Scope**: Changes to `OauthUsageData`, `PeriodUsage`, `apply_model_override()`, `resolve_model()`, or `recommended_model()` — those functions auto-recover once `seven_day_sonnet` is populated. Parsing of non-quota fields (`spend`, `extra_usage`, codename fields) — those are not consumed by `clp`. API authentication or HTTP transport.

### Design

`parse_oauth_usage()` is extended with a two-phase strategy. Phase 1 is the existing named-field scan (unchanged). Phase 2 is a new additive fallback: when Phase 1 returns `None` for a per-model field, `scan_limits_for_kind()` walks the `limits` array looking for an entry whose `kind` or `scope` value matches the target model.

**Phase 1 (named-field, backward compat):** Unchanged. `parse_period(body, "seven_day_sonnet")` returns `None` when the field is `null` — no error, no change to the guard condition. The existing body guard (`"five_hour"` / `"seven_day"` / `"seven_day_sonnet"` keys must be present) continues to pass since the new API response still includes these keys.

**Phase 2 (limits-array, forward compat):** Runs only when Phase 1 returned `None`. Scans `"limits":[...]` array; extracts each object block via `extract_object_block()`; checks `"kind"` and `"scope"` string fields against model needles (e.g. `["weekly_sonnet", "sonnet"]` for Sonnet); on match, reads `"percent"` as `utilization` (both are consumed percentage 0–100) and `"resets_at"` as the reset timestamp.

**Recovery path:** When Anthropic re-enables per-model `limits` entries (e.g. `{"kind": "weekly_sonnet", "percent": 45, "resets_at": "..."}` or `{"kind": "weekly_all", "scope": "sonnet", "percent": 45, ...}`), `scan_limits_for_kind()` populates `seven_day_sonnet` automatically. All three blind-spot algorithms (`apply_model_override`, `resolve_model`, `recommended_model`) receive the populated value and resume correct behavior with zero code changes.

**`scan_limits_for_kind()` matching strategy:** Checks both `"kind"` and `"scope"` fields against each needle. Uses substring matching to be resilient to future name changes (e.g. `"kind": "weekly_sonnet"` matches needle `"sonnet"`). Named field takes priority — Phase 2 never overrides a `Some` from Phase 1.

See [algorithm/009_oauth_usage_response_migration.md](../algorithm/009_oauth_usage_response_migration.md) for the full API change history, `limits` field semantics, and algorithm pseudocode.

### Acceptance Criteria

- **AC-01**: When `parse_period(body, "seven_day_sonnet")` returns `Some(...)` (named field is a non-null object), the result is used as-is and Phase 2 does not run. Backward compatibility with pre-2026-06-25 responses is preserved.
- **AC-02**: When `parse_period(body, "seven_day_sonnet")` returns `None` (named field is `null` or absent), `scan_limits_for_kind()` is invoked with needles `["weekly_sonnet", "sonnet"]`. If a matching `limits` entry is found, `seven_day_sonnet` is populated from that entry's `percent` (as `utilization`) and `resets_at`.
- **AC-03**: `percent` in the `limits` entry maps directly to `utilization`: `utilization = percent as f64`. No scale conversion. Both fields represent the consumed percentage (0–100).
- **AC-04**: When the `limits` array contains no entry matching the Sonnet needles (current state as of 2026-06-25), `seven_day_sonnet` remains `None`. No error is raised. `OauthUsageData` is returned successfully with `seven_day_sonnet: None`.
- **AC-05**: When Phase 2 populates `seven_day_sonnet`, all downstream algorithms (`apply_model_override()`, `resolve_model()`, `recommended_model()`) receive the populated `Some(PeriodUsage)` and produce correct behavior without code changes.
- **AC-06**: `OauthUsageData`, `PeriodUsage`, and all downstream function signatures are unchanged. This feature is purely additive to `parse_oauth_usage()`.
- **AC-07**: `resets_at` from the `limits` entry is preserved in `PeriodUsage.resets_at` when present; `None` when the entry's `resets_at` field is `null` or absent. Same semantics as the named-field path.
- **AC-08**: The `"seven_day_sonnet"` named-field guard remains in the body-validity guard: the guard requires at least one of `"five_hour"`, `"seven_day"`, `"seven_day_sonnet"` to be present as a key (not necessarily non-null). The new API response always includes all three keys, so the guard continues to pass.

### Bugs

_(none yet)_

### Dependencies

| File | Relationship |
|------|-------------|
| `claude_quota` | `parse_oauth_usage()`, `scan_limits_for_kind()` (new helper), `extract_object_block()`, `parse_f64_in_block()`, `parse_optional_string_in_block()`, `OauthUsageData`, `PeriodUsage` |

### Features

| File | Relationship |
|------|-------------|
| [009_token_usage.md](009_token_usage.md) | `7d(Son)` column sourced from `OauthUsageData.seven_day_sonnet`; affected by API change |
| [026_subprocess_model_effort.md](026_subprocess_model_effort.md) | `resolve_model(Auto)` uses `seven_day_sonnet` for touch model selection |
| [062_unified_session_config.md](062_unified_session_config.md) | `recommended_model()` uses `seven_day_sonnet` for footer Opus/Sonnet recommendation |

### Algorithm Docs

| File | Relationship |
|------|-------------|
| [algorithm/009_oauth_usage_response_migration.md](../algorithm/009_oauth_usage_response_migration.md) | Full API change history, `limits` field semantics, dual-source pseudocode, blind-spot table |

### Sources

| File | Relationship |
|------|-------------|
| `claude_quota/src/lib.rs` | `parse_oauth_usage()` — Phase 1 + Phase 2 implementation |

### Tests

| File | Relationship |
|------|-------------|
| `claude_quota/tests/oauth_usage_test.rs` | Unit tests for `parse_oauth_usage()` covering both format paths |
