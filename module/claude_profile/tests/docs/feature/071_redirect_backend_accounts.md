# FT — Feature 071: Redirect Backend Accounts

### Scope

- **Purpose**: Test cases for the `backend`/`base_url`/`redirect_model` account fields, the `.account.save backend::redirect` static-credential write path, `.account.use`'s `settings.json` `env.*` write/clear responsibility, the `static` token classification, and the Anthropic-only operation guards.
- **Source**: `docs/feature/071_redirect_backend_accounts.md`
- **Covers**: AC-01 through AC-17 (AC-08 superseded by AC-14 — see FT-08)

### Test Cases

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-01 | AC-01 | Full redirect save → `{name}.json` + `accessToken`-only credentials | `t01_save_redirect_full_succeeds` |
| FT-02 | AC-02 | Missing any required redirect param → exit 1 naming it, no files | `t02_save_redirect_missing_required_param_exits_1` |
| FT-03 | AC-03 | `base_url::` without `backend::redirect` → exit 1 | `t03_save_base_url_outside_redirect_exits_1` |
| FT-04 | AC-04 | No `backend::` at all → pre-071 OAuth-capture path unchanged | `t04_save_no_backend_unchanged_from_pre071` |
| FT-05 | AC-05 | Pre-071 account file (no `backend` key) reads as `anthropic` | `t05_accounts_and_credentials_status_no_backend_key_defaults_anthropic` |
| FT-06 | AC-06 | `.account.use` to redirect writes 3 `env.*` keys, preserves unrelated fields | `t06_use_redirect_account_writes_env_vars_and_skips_touch` |
| FT-07 | AC-07 | `.account.use` to anthropic clears exactly the 3 keys, prunes empty `env` | `t07_use_anthropic_after_redirect_clears_env_vars` |
| FT-08 | AC-08 | *Superseded by AC-14* — original target `.token.status` was removed | *(none — superseded, by design)* |
| FT-09 | AC-09 | `refresh_account_token()` no-op for redirect account | `ft26_071_refresh_redirect_account_returns_none_credentials_unchanged` |
| FT-10 | AC-10 | `apply_model_override()` no-op for active redirect account | `ft22_071_apply_model_override_redirect_backend_writes_nothing` |
| FT-11 | AC-11 | `.account.limits` rejects redirect account, no HTTP | `t10_limits_and_inspect_reject_redirect_account_exit_1` |
| FT-12 | AC-12 | `.account.inspect` rejects redirect account, no HTTP | `t10_limits_and_inspect_reject_redirect_account_exit_1` |
| FT-13 | AC-13 | `cols::+backend` column; JSON always includes `backend` | `t11_accounts_backend_column_text_and_json` |
| FT-14 | AC-14 | `.credentials.status` classifies active redirect account `Token: static` | `t12_credentials_status_active_redirect_account_classifies_static` |
| FT-15 | AC-15 | Re-save same name with different `backend::` rewrites from scratch | `t13_save_resave_different_backend_rewrites_from_scratch` |
| FT-16 | AC-16 | `.account.use` to redirect skips quota-fetch/touch unconditionally | `t06_use_redirect_account_writes_env_vars_and_skips_touch` (AC-16 assertions) |
| FT-17 | AC-17 | `.usage`/`.accounts` render redirect row as placeholder, no HTTP | `ft14_071_redirect_backend_produces_placeholder_no_http`, `ft14b_071_redirect_checked_before_not_owned_gate` |

### Notes

- ✅ Implemented — CLI-surface cases (FT-01–FT-07, FT-11–FT-16) live in `tests/cli/account_redirect_backend_test.rs`; FT-09 in `claude_profile_core/tests/account_refresh_test.rs`; FT-10 in `tests/usage/api_tests_a.rs`; FT-17 in `tests/usage/fetch_tests.rs`.
- Domain-level supplements in `claude_profile_core/tests/account_backend_test.rs` (`ft01`–`ft12_071`): `AccountBackend` parsing (`redirect` variant, absent/unrecognized/corrupt → `anthropic`), redirect save writes minimal credentials without touching the live `~/.claude/.credentials.json`, switch-path `env.*` write/clear/empty-`env`-prune/unrelated-subkey-preserve.
- All FT cases use a temporary isolated credential store and `$HOME`; no real user environment.
- Redirect account names are bare labels (e.g. `kimi`) — `validate_redirect_name()` drops the email-shape requirement; FT-01/FT-02 rely on this.
- FT-06/FT-16 share one source fn: the same switch asserts both the `env.*` writes (AC-06) and the unconditional quota-fetch/touch skip (AC-16).
- FT-17's second fn pins gate ordering: the redirect bypass fires before the non-owned gate in `fetch_quota_for_list()`.
- Feature 073's preset/Kimi-tier cases (`t14`–`t18` in the same CLI file) are indexed separately in `tests/docs/feature/073_kimi_provider_preset.md`.

---

### FT-01: Full redirect save succeeds

- **Given:** No pre-existing account `kimi`.
- **When:** `clp .account.save name::kimi backend::redirect base_url::https://api.moonshot.ai/anthropic api_key::sk-test redirect_model::kimi-k3`
- **Then:** `kimi.json` has `backend: "redirect"`, `base_url`, `redirect_model`; `kimi.credentials.json` contains only `accessToken` — no `refreshToken`/`expiresAt` keys.
- **Exit:** 0
- **Source fn:** `t01_save_redirect_full_succeeds`
- **Source:** [071_redirect_backend_accounts.md AC-01](../../../docs/feature/071_redirect_backend_accounts.md)

---

### FT-02: Missing required redirect param exits 1

- **Given:** No pre-existing account.
- **When:** The FT-01 command with any one of `base_url::`/`api_key::`/`redirect_model::` omitted.
- **Then:** Exits 1; stderr names the specific missing parameter(s); no files written.
- **Exit:** 1
- **Source fn:** `t02_save_redirect_missing_required_param_exits_1`
- **Source:** [071_redirect_backend_accounts.md AC-02](../../../docs/feature/071_redirect_backend_accounts.md)

---

### FT-03: `base_url::` outside redirect context exits 1

- **Given:** Any state.
- **When:** `clp .account.save name::alice@acme.com base_url::https://x` (no `backend::redirect`)
- **Then:** Exits 1 — `base_url::` is redirect-only.
- **Exit:** 1
- **Source fn:** `t03_save_base_url_outside_redirect_exits_1`
- **Source:** [071_redirect_backend_accounts.md AC-03](../../../docs/feature/071_redirect_backend_accounts.md)

---

### FT-04: No `backend::` preserves pre-071 behavior

- **Given:** A live `~/.claude/.credentials.json` fixture.
- **When:** `clp .account.save name::alice@acme.com` (no `backend::`)
- **Then:** Credentials copied as before; `alice@acme.com.json` carries `backend: "anthropic"`.
- **Exit:** 0
- **Source fn:** `t04_save_no_backend_unchanged_from_pre071`
- **Source:** [071_redirect_backend_accounts.md AC-04](../../../docs/feature/071_redirect_backend_accounts.md)

---

### FT-05: Absent `backend` key defaults to `anthropic`

- **Given:** A pre-071 account file with no `backend` key.
- **When:** `clp .accounts` / `clp .credentials.status`
- **Then:** Treated as `backend: anthropic` — no error, no misclassification.
- **Exit:** 0
- **Source fn:** `t05_accounts_and_credentials_status_no_backend_key_defaults_anthropic`
- **Source:** [071_redirect_backend_accounts.md AC-05](../../../docs/feature/071_redirect_backend_accounts.md)

---

### FT-06: Switch-to-redirect writes the 3 `env.*` keys

- **Given:** Redirect account `kimi` saved; `settings.json` with unrelated top-level fields.
- **When:** `clp .account.use name::kimi`
- **Then:** `env.ANTHROPIC_BASE_URL`/`env.ANTHROPIC_AUTH_TOKEN`/`env.ANTHROPIC_MODEL` match the stored values; unrelated `settings.json` fields untouched.
- **Exit:** 0
- **Source fn:** `t06_use_redirect_account_writes_env_vars_and_skips_touch`
- **Source:** [071_redirect_backend_accounts.md AC-06](../../../docs/feature/071_redirect_backend_accounts.md)

---

### FT-07: Switch-to-anthropic clears exactly the 3 keys

- **Given:** Active redirect account wrote the 3 `env.*` keys; an unrelated `env.*` sub-key also present.
- **When:** `clp .account.use name::alice@acme.com`
- **Then:** The 3 keys removed; unrelated `env.*` sub-key preserved; `env` itself removed if empty.
- **Exit:** 0
- **Source fn:** `t07_use_anthropic_after_redirect_clears_env_vars`
- **Source:** [071_redirect_backend_accounts.md AC-07](../../../docs/feature/071_redirect_backend_accounts.md)

---

### FT-08: *(superseded)*

> AC-08 originally targeted the now-removed `.token.status` command; `.credentials.status`'s `Token: static` classification (FT-14/AC-14) is the single surviving criterion. No test exists or is needed.

- **Source:** [071_redirect_backend_accounts.md AC-08](../../../docs/feature/071_redirect_backend_accounts.md)

---

### FT-09: `refresh_account_token()` is a no-op for redirect

- **Given:** A `backend: redirect` account.
- **When:** `refresh_account_token()` is invoked against it.
- **Then:** Returns `None`; no refresh subprocess; credential store file byte-for-byte unchanged.
- **Exit:** n/a (domain fn)
- **Source fn:** `ft26_071_refresh_redirect_account_returns_none_credentials_unchanged` (`claude_profile_core/tests/account_refresh_test.rs`)
- **Source:** [071_redirect_backend_accounts.md AC-09](../../../docs/feature/071_redirect_backend_accounts.md)

---

### FT-10: `apply_model_override()` is a no-op for redirect

- **Given:** Active account is `backend: redirect`.
- **When:** `apply_model_override()` runs.
- **Then:** Neither `model` nor `effortLevel` written — `settings.json` unchanged aside from the `env.*` keys FT-06 covers.
- **Exit:** n/a (domain fn)
- **Source fn:** `ft22_071_apply_model_override_redirect_backend_writes_nothing` (`tests/usage/api_tests_a.rs`)
- **Source:** [071_redirect_backend_accounts.md AC-10](../../../docs/feature/071_redirect_backend_accounts.md)

---

### FT-11: `.account.limits` rejects redirect accounts

- **Given:** Redirect account `kimi`.
- **When:** `clp .account.limits name::kimi`
- **Then:** Non-zero exit; stderr names the operation as Anthropic-only; no HTTP request.
- **Exit:** 1
- **Source fn:** `t10_limits_and_inspect_reject_redirect_account_exit_1`
- **Source:** [071_redirect_backend_accounts.md AC-11](../../../docs/feature/071_redirect_backend_accounts.md)

---

### FT-12: `.account.inspect` rejects redirect accounts

- **Given:** Redirect account `kimi`.
- **When:** `clp .account.inspect name::kimi`
- **Then:** Non-zero exit with the same Anthropic-only guard message; no HTTP request.
- **Exit:** 1
- **Source fn:** `t10_limits_and_inspect_reject_redirect_account_exit_1`
- **Source:** [071_redirect_backend_accounts.md AC-12](../../../docs/feature/071_redirect_backend_accounts.md)

---

### FT-13: `backend` column and JSON field

- **Given:** A mixed store (anthropic + redirect accounts).
- **When:** `clp .accounts cols::+backend`; `clp .accounts format::json`
- **Then:** Table shows `anthropic`/`redirect` per account; JSON always includes `backend` regardless of `cols::`.
- **Exit:** 0
- **Source fn:** `t11_accounts_backend_column_text_and_json`
- **Source:** [071_redirect_backend_accounts.md AC-13](../../../docs/feature/071_redirect_backend_accounts.md)

---

### FT-14: `Token: static` classification

- **Given:** Active account is `backend: redirect`.
- **When:** `clp .credentials.status name::kimi`
- **Then:** Reports `Token: static` (never `valid`/`expiring_soon`/`expired`); `refreshToken`/`expiresAt`-derived fields report absent/N/A without erroring.
- **Exit:** 0
- **Source fn:** `t12_credentials_status_active_redirect_account_classifies_static`
- **Source:** [071_redirect_backend_accounts.md AC-14](../../../docs/feature/071_redirect_backend_accounts.md)

---

### FT-15: Re-save with different backend rewrites from scratch

- **Given:** Account `kimi` saved as `backend: redirect`.
- **When:** `clp .account.save name::kimi backend::anthropic` (with live credentials fixture)
- **Then:** Both files rewritten per the anthropic path; no redirect fields survive.
- **Exit:** 0
- **Source fn:** `t13_save_resave_different_backend_rewrites_from_scratch`
- **Source:** [071_redirect_backend_accounts.md AC-15](../../../docs/feature/071_redirect_backend_accounts.md)

---

### FT-16: Redirect switch skips quota-fetch/touch unconditionally

- **Given:** Redirect account `kimi`; default `touch::1`.
- **When:** `clp .account.use name::kimi`
- **Then:** Exits 0 with zero HTTP calls; no credential-file read for quota fetch occurs before the skip — backend-driven, distinct from `touch::0`.
- **Exit:** 0
- **Source fn:** `t06_use_redirect_account_writes_env_vars_and_skips_touch` (AC-16 assertions)
- **Source:** [071_redirect_backend_accounts.md AC-16](../../../docs/feature/071_redirect_backend_accounts.md)

---

### FT-17: `.usage`/`.accounts` redirect row placeholder

- **Given:** Account list containing a redirect account among anthropic ones.
- **When:** `fetch_quota_for_list()` runs (via `.usage`/`.accounts refresh::1`).
- **Then:** The redirect row carries `result: Err("redirect backend — no Anthropic quota")` with no HTTP call, checked before the non-owned gate; anthropic rows unaffected.
- **Exit:** 0
- **Source fn:** `ft14_071_redirect_backend_produces_placeholder_no_http`, `ft14b_071_redirect_checked_before_not_owned_gate` (`tests/usage/fetch_tests.rs`)
- **Source:** [071_redirect_backend_accounts.md AC-17](../../../docs/feature/071_redirect_backend_accounts.md)
