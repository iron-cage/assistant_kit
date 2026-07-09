# Parameter 065: `offline::` — Edge Cases

**Behavioral Divergence Pair:** EC-01 ↔ EC-02 — `offline::1` returns the static embedded `STATIC_MODELS` catalog with no network call and no credentials required; `offline::0` (default) queries the live `GET /v1/models` endpoint using the current account's OAuth token — same command shape, observably different data source and network behavior.

### Test Case Index

| ID | Test | Scenario | Expected | Status |
|----|------|----------|----------|--------|
| EC-01 | `ec1_offline_1_uses_static_catalog_no_network` | `offline::1` | returns `STATIC_MODELS` constant; no network call; no OAuth token required; exit 0 | ✅ |
| EC-02 | `ec2_offline_0_default_queries_live_api` | `offline::0` (or omitted) with valid credentials | queries `GET /v1/models` using current account OAuth token; exit 0 | ✅ |
| EC-03 | `ec3_offline_omitted_defaults_to_0` | `.models` with no `offline::` | behaves identically to `offline::0` — live mode | ✅ |
| EC-04 | `ec4_offline_0_no_credentials_exits_1` | `offline::0` (default), no active account credentials | exit 1; stderr suggests `offline::1` | ✅ |
| EC-05 | `ec5_offline_true_false_aliases_accepted` | `offline::true` and `offline::false` | `true` behaves as `1`, `false` behaves as `0` | ✅ |
| EC-06 | `ec6_offline_invalid_value_exits_1` | `offline::maybe` (non-boolean) | exit 1 — invalid boolean value rejected | ✅ |
| EC-07 | `ec7_offline_1_combines_with_name_filter` | `offline::1 name::haiku` | static catalog filtered by `name::` substring match; exit 0 | ✅ |
| EC-08 | `ec8_offline_1_combines_with_format_json` | `offline::1 format::json` | static catalog rendered as valid JSON array; exit 0 | ✅ |
| EC-09 | `ec9_offline_1_omits_invite_only_models` | `offline::1` when live account would have `claude-fable-5` access | static catalog does not include invite-only models absent from the workspace-curated list — offline mode may lag behind live API | ✅ |

**Total:** 9 edge case tests

---

### EC-01: `offline::1` — static catalog, no network call

- **Given:** No active account credentials configured; no network access assumed.
- **When:** `clp .models offline::1`
- **Then:** Exits 0. stdout lists models from the `STATIC_MODELS` constant embedded in `claude_quota` (e.g. `claude-opus-4-8`, `claude-sonnet-5`, `claude-haiku-4-5-20251001`). No HTTP request is made. No OAuth token is required.
- **Exit:** 0
- **Source:** [param/065_offline.md](../../../../docs/cli/param/065_offline.md), [command/008_models.md](../../../../docs/cli/command/008_models.md)

---

### EC-02: `offline::0` — live mode queries the API

- **Given:** An active account with a valid OAuth token exists.
- **When:** `clp .models offline::0`
- **Then:** Exits 0. `GET /v1/models` is called using the current account's OAuth token; all pages collected (limit=1000 per page); results rendered.
- **Exit:** 0
- **Source:** [param/065_offline.md](../../../../docs/cli/param/065_offline.md), [command/008_models.md](../../../../docs/cli/command/008_models.md)

---

### EC-03: Omitted `offline::` defaults to `0` — live mode

- **Given:** An active account with a valid OAuth token exists.
- **When:** `clp .models` (no `offline::` provided)
- **Then:** Exits 0. Behavior is identical to `offline::0` — live API query, same as EC-02.
- **Exit:** 0
- **Source:** [param/065_offline.md](../../../../docs/cli/param/065_offline.md)

---

### EC-04: `offline::0` without valid credentials — exit 1

- **Given:** No active account credentials configured (or all expired/invalid).
- **When:** `clp .models offline::0`
- **Then:** Exits 1. stderr suggests using `offline::1` as a workaround.
- **Exit:** 1
- **Source:** [param/065_offline.md](../../../../docs/cli/param/065_offline.md), [command/008_models.md](../../../../docs/cli/command/008_models.md)

---

### EC-05: `true`/`false` boolean aliases accepted

- **Given:** No active account credentials configured (for the `true` case); valid credentials exist (for the `false` case).
- **When:**
  1. `clp .models offline::true`
  2. `clp .models offline::false` (with valid credentials)
- **Then:** 1. Behaves identically to `offline::1` — static catalog, no network call, exit 0.
  2. Behaves identically to `offline::0` — live API query, exit 0.
- **Exit:** 0
- **Source:** [param/065_offline.md](../../../../docs/cli/param/065_offline.md)

---

### EC-06: Invalid boolean value rejected

- **Given:** Any credential state.
- **When:** `clp .models offline::maybe`
- **Then:** Exits 1. stderr indicates `maybe` is not a valid boolean value for `offline::`.
- **Exit:** 1
- **Source:** [param/065_offline.md](../../../../docs/cli/param/065_offline.md)

---

### EC-07: `offline::1` combined with `name::` filter

- **Given:** No active account credentials configured.
- **When:** `clp .models offline::1 name::haiku`
- **Then:** Exits 0. stdout contains only the static catalog entry whose `id` substring-matches `haiku` (case-insensitive) — e.g. `claude-haiku-4-5-20251001`. No network call.
- **Exit:** 0
- **Source:** [param/065_offline.md](../../../../docs/cli/param/065_offline.md), [command/008_models.md](../../../../docs/cli/command/008_models.md)

---

### EC-08: `offline::1` combined with `format::json`

- **Given:** No active account credentials configured.
- **When:** `clp .models offline::1 format::json`
- **Then:** Exits 0. stdout is a valid JSON array of static catalog model objects (e.g. `[{"id":"claude-opus-4-8","display_name":"Claude Opus 4.8",...}, ...]`).
- **Exit:** 0
- **Source:** [param/065_offline.md](../../../../docs/cli/param/065_offline.md), [command/008_models.md](../../../../docs/cli/command/008_models.md)

---

### EC-09: `offline::1` may omit invite-only models present in live mode

- **Given:** A live account has access to an invite-only model (e.g. `claude-fable-5`) not present in the workspace-curated `STATIC_MODELS` catalog.
- **When:** `clp .models offline::1 name::fable`
- **Then:** Exits 0. stdout does not contain `claude-fable-5` — offline mode only shows the static workspace catalog derived from `contract/claude_code/docs/model/readme.md`, which may lag behind the live API for new or invite-only models.
- **Exit:** 0
- **Source:** [param/065_offline.md](../../../../docs/cli/param/065_offline.md)
