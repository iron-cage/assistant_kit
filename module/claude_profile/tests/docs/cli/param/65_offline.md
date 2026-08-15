# Parameter 065: `offline::` — Edge Cases

> **Citation correction:** all 9 `ec0N_offline_*`-named functions cited below never existed in the test suite — grep across `src/`, `tests/`, and `claude_profile_core/tests/` finds each name only in this doc file itself. Unlike `58_assign.md` (whose citations described a since-*removed* parameter), `offline::` is a live, current parameter (Feature 068) — these were fabricated citations for still-relevant behavior from the outset, not stale references to removed functionality. Real (differently-scoped) coverage exists in `models_test.rs` (`it01`–`it10`/`ft01`–`ft10`, all via `offline::1`; the module's own doc comment states "All tests use `offline::1` to avoid network dependency in CI"): EC-01, EC-07, EC-08 have a genuine current equivalent there (see each EC's own "Source fn" note); EC-02/03/04 are coverage gaps by design (no live-mode test exists anywhere in the suite, consistent with that CI-avoidance policy); EC-09 has a partial equivalent (the general zero-match mechanism is tested; the specific invite-only-model framing is not). EC-05 and EC-06 additionally needed a **behavioral correction**, not just a citation fix: `offline::` is registered `Kind::Integer` (`src/registry.rs:209`), and its own design doc ([param/065_offline.md](../../../../docs/cli/param/065_offline.md)) states plainly that `false`/`true` are "rejected as a type mismatch before the command runs" — the original EC-05 claim that they're accepted as `1`/`0` aliases directly contradicts that authoritative doc; corrected below.

**Behavioral Divergence Pair:** EC-01 ↔ EC-02 — `offline::1` returns the static embedded `STATIC_MODELS` catalog with no network call and no credentials required; `offline::0` (default) queries the live `GET /v1/models` endpoint using the current account's OAuth token — same command shape, observably different data source and network behavior.

### Test Case Index

| ID | Test | Scenario | Expected | Status |
|----|------|----------|----------|--------|
| EC-01 | `ec1_offline_1_uses_static_catalog_no_network` | `offline::1` | returns `STATIC_MODELS` constant; no network call; no OAuth token required; exit 0 | ⚠️ re-cited |
| EC-02 | `ec2_offline_0_default_queries_live_api` | `offline::0` (or omitted) with valid credentials | queries `GET /v1/models` using current account OAuth token; exit 0 | ⚠️ gap |
| EC-03 | `ec3_offline_omitted_defaults_to_0` | `.models` with no `offline::` | behaves identically to `offline::0` — live mode | ⚠️ gap |
| EC-04 | `ec4_offline_0_no_credentials_exits_1` | `offline::0` (default), no active account credentials | exit 1; stderr suggests `offline::1` | ⚠️ gap |
| EC-05 | `ec5_offline_true_false_rejected_as_type_mismatch` | `offline::true` and `offline::false` | both exit 1 — `Kind::Integer` rejects non-integer literals; NOT accepted as `1`/`0` aliases | ⚠️ gap (corrected) |
| EC-06 | `ec6_offline_invalid_value_exits_1` | `offline::maybe` (non-integer) | exit 1 — invalid integer value rejected (not "boolean") | ⚠️ gap (corrected) |
| EC-07 | `ec7_offline_1_combines_with_name_filter` | `offline::1 name::opus` | static catalog filtered by `name::` substring match; exit 0 | ⚠️ re-cited |
| EC-08 | `ec8_offline_1_combines_with_format_json` | `offline::1 format::json` | static catalog rendered as valid JSON array; exit 0 | ⚠️ re-cited |
| EC-09 | `ec9_offline_1_omits_invite_only_models` | `offline::1` when live account would have `claude-fable-5` access | static catalog does not include invite-only models absent from the workspace-curated list — offline mode may lag behind live API | ⚠️ partial |

**Total:** 9 edge case tests

---

### EC-01: `offline::1` — static catalog, no network call

- **Given:** Real HOME credential state (test does not isolate or clear it) — irrelevant to the outcome, since the offline code path never reads credentials.
- **When:** `clp .models offline::1`
- **Then:** Exits 0. stdout contains `claude-opus-4-8` (verified by `it01`/`ft01`), `claude-sonnet-5` (`it02`/`ft02`), and `claude-haiku-4-5-20251001` (`it03`/`ft03`) — entries from the `STATIC_MODELS` constant embedded in `claude_quota`. "No HTTP request is made" and "no OAuth token is required" are not asserted by any test directly (no network mock/instrumentation exists in this suite) — they follow structurally from `models.rs`'s offline branch (`claude_quota::STATIC_MODELS.to_vec()`), which never calls `fetch_active_token()`.
- **Exit:** 0
- **Source fn:** *(fabricated — `ec1_offline_1_uses_static_catalog_no_network` never existed; current equivalent scenario is `it01_offline_contains_opus`/`ft01_offline_contains_opus` in `models_test.rs`, siblings `it02`/`it03`/`ft02`/`ft03` cover the other two static IDs — these check stdout content and exit 0 only; the "no network call" claim is a source-level inference, not something any test measures directly)*
- **Source:** [param/065_offline.md](../../../../docs/cli/param/065_offline.md), [command/008_models.md](../../../../docs/cli/command/008_models.md)

---

### EC-02: `offline::0` — live mode queries the API

- **Given:** An active account with a valid OAuth token exists.
- **When:** `clp .models offline::0`
- **Then:** Exits 0. `GET /v1/models` is called using the current account's OAuth token; all pages collected (limit=1000 per page); results rendered.
- **Exit:** 0
- **Source fn:** *(fabricated — `ec2_offline_0_default_queries_live_api` never existed; no test anywhere in the suite exercises live mode for `.models` — `models_test.rs`'s own module doc comment states "All tests use `offline::1` to avoid network dependency in CI" — coverage gap by design)*
- **Source:** [param/065_offline.md](../../../../docs/cli/param/065_offline.md), [command/008_models.md](../../../../docs/cli/command/008_models.md)

---

### EC-03: Omitted `offline::` defaults to `0` — live mode

- **Given:** An active account with a valid OAuth token exists.
- **When:** `clp .models` (no `offline::` provided)
- **Then:** Exits 0. Behavior is identical to `offline::0` — live API query, same as EC-02.
- **Exit:** 0
- **Source fn:** *(fabricated — `ec3_offline_omitted_defaults_to_0` never existed; no test in the suite invokes `.models` without an explicit `offline::` value — every `models_test.rs` case passes `offline::1` explicitly — coverage gap)*
- **Source:** [param/065_offline.md](../../../../docs/cli/param/065_offline.md)

---

### EC-04: `offline::0` without valid credentials — exit 1

- **Given:** No active account credentials configured (or all expired/invalid).
- **When:** `clp .models offline::0`
- **Then:** Exits 1. stderr suggests using `offline::1` as a workaround (per `fetch_active_token()`'s error message in `src/commands/models.rs`: "no active account — use offline::1 or set an active account with .account.use").
- **Exit:** 1
- **Source fn:** *(fabricated — `ec4_offline_0_no_credentials_exits_1` never existed; no test exercises this path. Note: unlike EC-02/03, this specific case does NOT require live network — `fetch_active_token()` fails locally (missing active-marker file) before any HTTP call — so it is feasible to test offline-of-network, just currently untested — coverage gap)*
- **Source:** [param/065_offline.md](../../../../docs/cli/param/065_offline.md), [command/008_models.md](../../../../docs/cli/command/008_models.md)

---

### EC-05: `true`/`false` rejected as a type mismatch — NOT accepted as boolean aliases

> **Behavioral correction:** the original claim here (`true`/`false` behave as aliases for `1`/`0`) is contradicted by the parameter's own design doc, not merely untested. `offline::` is registered `Kind::Integer` (`src/registry.rs:209`), and [param/065_offline.md](../../../../docs/cli/param/065_offline.md)'s own Constraints bullet states: "`false`/`true` are rejected as a type mismatch before the command runs, unlike the string-typed `lock::`/`reserve::`, which silently coerce non-`"1"` values to off." `src/output.rs`'s `parse_int_flag` doc comment corroborates the mechanism: the `"true"`/`"false"` alias arms are reachable only for `Kind::String`-typed params (e.g. `touch::`) — for a `Kind::Integer` param like `offline::`, the unilang framework layer calls `"true".parse::<i64>()` before the command routine ever runs, which fails and rejects with exit 1.

- **Given:** Any credential state — irrelevant, since the rejection happens at the framework parsing layer before `models_routine` runs.
- **When:**
  1. `clp .models offline::true`
  2. `clp .models offline::false`
- **Then:** Both exit 1. `offline::` accepts only integer literals `0`/`1`; `true`/`false` are rejected as a type mismatch at the framework layer, the same as any other non-integer string (see EC-06).
- **Exit:** 1
- **Source fn:** *(fabricated — `ec5_offline_true_false_aliases_accepted` never existed; no test in the suite exercises this rejection path — coverage gap; the original claim was also factually wrong, not just untested — see correction above)*
- **Source:** [param/065_offline.md](../../../../docs/cli/param/065_offline.md)

---

### EC-06: Invalid (non-integer) value rejected

> **Behavioral correction:** `offline::` is `Kind::Integer` (`src/registry.rs:209`), not `Kind::Boolean` — the rejection is an invalid *integer*, not an invalid *boolean*. `offline::maybe` fails the same framework-level `"maybe".parse::<i64>()` check as `offline::true`/`offline::false` (see EC-05), not a boolean-specific validator, and stderr wording (not confirmed by any test — see below) would be expected to reflect an integer-type mismatch, not "boolean."

- **Given:** Any credential state.
- **When:** `clp .models offline::maybe`
- **Then:** Exits 1 — `offline::` accepts only integer literals; `maybe` fails integer parsing at the framework layer before `models_routine` runs.
- **Exit:** 1
- **Source fn:** *(fabricated — `ec6_offline_invalid_value_exits_1` never existed; no test in the suite exercises this rejection path — coverage gap; the original "boolean value" framing was also imprecise — see correction above)*
- **Source:** [param/065_offline.md](../../../../docs/cli/param/065_offline.md)

---

### EC-07: `offline::1` combined with `name::` filter

- **Given:** Real HOME credential state (irrelevant — offline path never reads credentials).
- **When:** `clp .models offline::1 name::opus`
- **Then:** Exits 0. stdout contains only static catalog entries whose `id` substring-matches `opus` (case-insensitive) — `claude-sonnet-5` and `claude-haiku*` are confirmed absent; at least one `claude-opus*` entry present. No network call (same structural guarantee as EC-01). The doc's original example used `name::haiku`; the real, re-cited test uses `name::opus` — updated here to match. `it10`/`ft10` (`name::claude-opus`, substring match) cover the same combined-filter mechanism with a different substring.
- **Exit:** 0
- **Source fn:** *(fabricated — `ec7_offline_1_combines_with_name_filter` never existed; current equivalent scenario is `it07_name_filter_opus_only`/`ft07_name_filter_opus_only` in `models_test.rs` (sibling `it10`/`ft10` cover a substring variant) — same combined offline+name-filter behavior, different example value)*
- **Source:** [param/065_offline.md](../../../../docs/cli/param/065_offline.md), [command/008_models.md](../../../../docs/cli/command/008_models.md)

---

### EC-08: `offline::1` combined with `format::json`

- **Given:** Real HOME credential state (irrelevant — offline path never reads credentials).
- **When:** `clp .models offline::1 format::json`
- **Then:** Exits 0. stdout parses as valid JSON, is a non-empty array, and every element has a non-empty string `id` field (e.g. `[{"id":"claude-opus-4-8","display_name":"Claude Opus 4.8",...}, ...]` per `render_json()` in `src/commands/models.rs`).
- **Exit:** 0
- **Source fn:** `it06_offline_json_valid_array`/`ft06_offline_json_valid_array` (in `models_test.rs`) — direct match; previously cited as `ec8_offline_1_combines_with_format_json`, which never existed
- **Source:** [param/065_offline.md](../../../../docs/cli/param/065_offline.md), [command/008_models.md](../../../../docs/cli/command/008_models.md)

---

### EC-09: `offline::1` may omit invite-only models present in live mode

- **Given:** A live account has access to an invite-only model (e.g. `claude-fable-5`) not present in the workspace-curated `STATIC_MODELS` catalog. Independently confirmed: `STATIC_MODELS` (`claude_quota/src/lib.rs:932-978`) contains exactly 5 entries (`claude-opus-4-8`, `claude-sonnet-5`, `claude-haiku-4-5-20251001`, `claude-opus-4-5-20251101`, `claude-sonnet-4-5-20250929`) — no `claude-fable-5` entry, consistent with the design doc's own note that live mode is preferred for invite-only model checks.
- **When:** `clp .models offline::1 name::fable`
- **Then:** Exits 0. stdout does not contain `claude-fable-5` — offline mode only shows the static workspace catalog, which may lag behind the live API for new or invite-only models.
- **Exit:** 0
- **Source fn:** *(fabricated — `ec9_offline_1_omits_invite_only_models` never existed. Partial equivalent: the general mechanism this claim depends on — a `name::` filter matching zero catalog entries produces empty stdout, exit 0 — is verified by `it08_name_filter_no_match`/`ft08_name_filter_no_match` (`name::zz_no_match`) in `models_test.rs`. No test uses an "invite-only model" framing or asserts specifically on `fable`/`claude-fable-5` — that specific illustrative claim remains a coverage gap even though the underlying mechanism and the catalog-contents fact above are independently confirmed)*
- **Source:** [param/065_offline.md](../../../../docs/cli/param/065_offline.md)
