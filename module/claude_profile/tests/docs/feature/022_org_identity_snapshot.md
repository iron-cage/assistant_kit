# Test: Feature 022 — Org Identity Snapshot

### Scope

- **Purpose**: Test cases for org identity snapshot capture.
- **Source**: `docs/feature/022_org_identity_snapshot.md`
- **Covers**: AC-01 through AC-11

Feature behavioral requirement test cases for `docs/feature/022_org_identity_snapshot.md` (FR-22). Each FT case maps to one acceptance criterion. Parameter edge cases are in [cli/param/030_org_uuid.md](../cli/param/30_org_uuid.md) and [cli/param/031_org_name.md](../cli/param/31_org_name.md).

### AC Coverage Index

| FT | Criterion | AC | Category |
|----|-----------|----|---------|
| FT-01 | `save` writes org identity to `{name}.json` when endpoint 005 succeeds | AC-01 | Lifecycle |
| FT-02 | Endpoint 005 failure → org identity absent in `{name}.json`; `save` still exits 0 | AC-02 | Best-Effort |
| FT-03 | Re-`save` overwrites org identity in `{name}.json` with fresh data | AC-03 | Idempotency |
| FT-04 | `delete` removes `{name}.json`; absent snapshot causes no error | AC-04 | Lifecycle |
| FT-05 | `cols::+org_uuid` on `.accounts` shows `Org ID:` per account from snapshot | AC-05 | Field Presence |
| FT-06 | `cols::+org_name` on `.accounts` shows `Org:` per account from snapshot | AC-06 | Field Presence |
| FT-07 | `org_uuid::1` on `.credentials.status` shows `Org ID:` from active `{name}.json` | AC-07 | Field Presence |
| FT-08 | `org_name::1` on `.credentials.status` shows `Org:` from active `{name}.json` | AC-08 | Field Presence |
| FT-09 | `format::json` always includes all 5 org fields regardless of params | AC-09 | JSON Output |
| FT-10 | `--no-default-features` compile passes without `claude_quota` dep | AC-10 | Feature Gate |
| FT-11 | Null workspace fields in `{name}.json` → empty string in `Account`, `N/A` in text | AC-11 | Personal Account |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|----|---------|
| FT-01 | save writes org identity to {name}.json when endpoint 005 returns valid response | AC-01 | Lifecycle |
| FT-02 | endpoint 005 failure → no org identity written; save still exits 0 | AC-02 | Best-Effort |
| FT-03 | re-save overwrites org identity in {name}.json with fresh data | AC-03 | Idempotency |
| FT-04 | delete removes {name}.json; absent file causes no error | AC-04 | Lifecycle |
| FT-05 | cols::+org_uuid shows Org ID per account on .accounts | AC-05 | Field Presence |
| FT-06 | cols::+org_name shows Org per account on .accounts | AC-06 | Field Presence |
| FT-07 | org_uuid::1 shows Org ID on .credentials.status from active {name}.json | AC-07 | Field Presence |
| FT-08 | org_name::1 shows Org on .credentials.status from active {name}.json | AC-08 | Field Presence |
| FT-09 | format::json includes organization_uuid, organization_name, organization_role, workspace_uuid, workspace_name | AC-09 | JSON Output |
| FT-10 | cargo check --no-default-features exits 0 | AC-10 | Feature Gate |
| FT-11 | null workspace fields in {name}.json render as empty string and N/A | AC-11 | Personal Account |

**Total:** 11 FT cases

---

### FT-01: `save` writes org identity to `{name}.json` when endpoint 005 returns valid response

- **Given:** Active account; `~/.claude/.credentials.json` contains a valid `accessToken`; endpoint 005 (`GET /api/oauth/claude_cli/roles`) returns a valid JSON response with `organization_uuid`, `organization_name`, and `organization_role`.
- **When:** `clp .account.save`
- **Then:** `{credential_store}/{name}.json` exists after the command exits; the file parses as valid JSON containing `organization_uuid` and `organization_name` fields. Exit 0.
- **Exit:** 0
- **Live:** yes (lim_it — requires valid Anthropic credentials with roles scope)
- **Source fn:** `as20_lim_it_save_writes_roles_json` (in `account_relogin_test.rs`)
- **Source:** [022_org_identity_snapshot.md AC-01](../../../docs/feature/022_org_identity_snapshot.md)

---

### FT-02: Endpoint 005 failure → no org identity written; `save` still exits 0

- **Given:** `~/.claude/.credentials.json` has no `accessToken` field at all (not merely an invalid value) — the cited test's own comment states this means `fetch_claude_cli_roles` is never called; no pre-existing `{name}.json` in the credential store.
- **When:** `clp .account.save name::user@example.com`
- **Then:** `{credential_store}/user@example.com.json` (if it exists at all) does NOT contain `organization_uuid`. Exit 0. No fatal error on stderr about roles; all other save operations complete normally.
- **Exit:** 0
- **Note:** Corrected — the doc previously described this as "an invalid `accessToken` value (causes endpoint 005 to return HTTP 401)"; per direct source inspection (`claude_profile_core/src/account/store.rs`; account.rs:412-429 pre-split), the org-identity capture is gated by `if let Some(token) = parse_string_field(&creds_text, "accessToken")` — with no `accessToken` field present at all, this guard short-circuits and `fetch_claude_cli_roles` is never invoked, so no HTTP call (and therefore no 401) occurs. An actual invalid-token-causing-401 scenario would still hit the same best-effort `if let Ok(roles) = ...` skip (network errors are silently ignored too), but the cited test does not exercise that specific path.
- **Source fn:** `as19_save_best_effort_no_roles_json` (in `account_relogin_test.rs`)
- **Source:** [022_org_identity_snapshot.md AC-02](../../../docs/feature/022_org_identity_snapshot.md)

---

### FT-03: Re-`save` overwrites org identity in `{name}.json` with fresh data

- **Given:** Account `alice@example.com` has an existing `alice@example.com.json` in the credential store (stale org content); the active `accessToken` is valid; endpoint 005 returns an updated response on the second call.
- **When:** `clp .account.save` (second invocation with the same name)
- **Then:** `{credential_store}/alice@example.com.json` is overwritten; the file contains org data from the second API response. Exit 0.
- **Exit:** 0
- **Live:** yes (lim_it — requires valid Anthropic credentials; verifies overwrite with real API response)
- **Source fn:** `as21_lim_it_resave_overwrites_roles_json` (in `account_relogin_test.rs`)
- **Source:** [022_org_identity_snapshot.md AC-03](../../../docs/feature/022_org_identity_snapshot.md)

---

### FT-04: `delete` removes `{name}.json`; absent file causes no error

- **Given:** Account `old@archive.com` (inactive) whose credential store contains a pre-seeded `old@archive.com.json` roles snapshot; a second account `work@acme.com` is active.
- **When:** `clp .account.delete name::old@archive.com`
- **Then:** `{credential_store}/old@archive.com.credentials.json` and `{credential_store}/old@archive.com.json` both no longer exist; exit 0.
- **Exit:** 0
- **Note:** Corrected — the cited test exercises only the "roles.json present" deletion case (illustrative names corrected from `alice@acme.com` to the test's actual `old@archive.com`). The doc's second claim (deleting an account whose `{name}.json` does NOT exist causes no error) is not exercised by any cited test, but is confirmed true by direct source inspection: `claude_profile_core/src/account/store.rs` (account.rs:967 pre-split) removes `{name}.json` via `let _ = std::fs::remove_file(...)`, explicitly discarding the file-not-found error.
- **Source fn:** `ad15_delete_removes_roles_json` (in `account_relogin_test.rs`)
- **Source:** [022_org_identity_snapshot.md AC-04](../../../docs/feature/022_org_identity_snapshot.md)

---

### FT-05: `cols::+org_uuid` shows `Org ID:` per account on `.accounts`

- **Given:** Two separate single-account tests, both for `alice@acme.com`: (1) `alice@acme.com.json` exists in the credential store with `"organization_uuid":"org-xyz-789"`; (2) no `alice@acme.com.json` at all.
- **When:** `clp .accounts cols::+org_uuid`
- **Then:** In scenario (1), stdout contains `Org ID:` followed by `org-xyz-789`. In scenario (2), stdout contains `Org ID:` and `N/A`. Exit 0 for both.
- **Exit:** 0
- **Note:** Corrected — `org_uuid::1` as a standalone boolean param on `.accounts` was removed (Feature 037); passing it now errors with `"parameter 'org_uuid' removed — use 'cols::+org_uuid' instead"` (confirmed via `src/commands/accounts.rs:315-343`'s `REMOVED_TOGGLES` rejection list). Also corrected — the two cited tests are each single-account (`alice@acme.com`, present vs. absent roles.json), not a two-account `alice`-vs-`bob` comparison in one run; the example UUID value was also corrected to match the fixture (`org-xyz-789`, via `write_account_roles_json`).
- **Source fn:** `acc42_org_uuid_shows_from_roles_json` (roles.json present) + `acc44_org_uuid_missing_roles_json_na` (roles.json absent) (in `accounts_list_test_b.rs`)
- **Source:** [022_org_identity_snapshot.md AC-05](../../../docs/feature/022_org_identity_snapshot.md)

---

### FT-06: `cols::+org_name` shows `Org:` per account on `.accounts`

- **Given:** Two separate single-account tests, both for `alice@acme.com`: (1) `alice@acme.com.json` exists with `"organization_name":"Acme Corp"`; (2) no `alice@acme.com.json` at all.
- **When:** `clp .accounts cols::+org_name`
- **Then:** In scenario (1), stdout contains `Org:` followed by `Acme Corp`. In scenario (2), stdout contains `Org:` and `N/A`. Exit 0 for both.
- **Exit:** 0
- **Note:** Corrected — `org_name::1` as a standalone boolean param on `.accounts` was removed (Feature 037); passing it now errors with `"parameter 'org_name' removed — use 'cols::+org_name' instead"` (confirmed via `src/commands/accounts.rs:315-343`). Also corrected — the two cited tests are each single-account (`alice@acme.com`, present vs. absent roles.json), not a two-account `alice`-vs-`bob` comparison in one run.
- **Source fn:** `acc46_org_name_shows_from_roles_json` (roles.json present) + `acc48_org_name_missing_roles_json_na` (roles.json absent) (in `accounts_list_test_b.rs`)
- **Source:** [022_org_identity_snapshot.md AC-06](../../../docs/feature/022_org_identity_snapshot.md)

---

### FT-07: `org_uuid::1` shows `Org ID:` on `.credentials.status` from active `{name}.json`

- **Given:** Active account `alice@acme.com`; `_active` marker points to alice; `{credential_store}/alice@acme.com.json` exists with `"organization_uuid":"aaaaaaaa-1111-cccc-dddd-eeeeeeeeeeee"`.
- **When:** `clp .credentials.status org_uuid::1`
- **Then:** Stdout contains `Org ID:` followed by `aaaaaaaa-1111-cccc-dddd-eeeeeeeeeeee`. Exit 0.
- **Exit:** 0
- **Source fn:** `cred31_org_uuid_shows_org_id_line` (in `credentials_test_b.rs`)
- **Source:** [022_org_identity_snapshot.md AC-07](../../../docs/feature/022_org_identity_snapshot.md)

---

### FT-08: `org_name::1` shows `Org:` on `.credentials.status` from active `{name}.json`

- **Given:** Active account `alice@acme.com`; `_active` marker points to alice; `{credential_store}/alice@acme.com.json` exists with `"organization_name":"Acme Corp"`.
- **When:** `clp .credentials.status org_name::1`
- **Then:** Stdout contains `Org:` followed by `Acme Corp`. Exit 0.
- **Exit:** 0
- **Source fn:** `cred38_org_name_shows_org_line` (in `credentials_test_b.rs`)
- **Source:** [022_org_identity_snapshot.md AC-08](../../../docs/feature/022_org_identity_snapshot.md)

---

### FT-09: `format::json` always includes all 5 org fields regardless of params

- **Given:** Active account `user@example.com` with `user@example.com.json` containing all 5 org fields (`organization_uuid`, `organization_name`, `organization_role`, `workspace_uuid`, `workspace_name`). `org_uuid::` and `org_name::` params are NOT passed.
- **When:** `clp .credentials.status format::json` and separately `clp .accounts format::json`
- **Then:** The `.credentials.status format::json` output contains all 5 keys (`organization_uuid`, `organization_name`, `organization_role`, `workspace_uuid`, `workspace_name`), fully asserted by the cited test. The `.accounts format::json` output likewise contains all 5 keys per the unconditional format string in `accounts_render.rs:299-320`, though the cited `.accounts`-side test only explicitly asserts the `organization_uuid` key. Exit 0 for both.
- **Exit:** 0
- **Note:** Corrected — the previous single citation covered only the `.credentials.status` half; the `.accounts` half is a separate test (`acc45_json_includes_org_uuid`) that was not cited at all, and that test itself only asserts 1 of the 5 claimed keys. The other 4 keys' presence in `.accounts format::json` is confirmed structurally (source always emits all 5), not by test assertion.
- **Source fn:** `cred45_ft09_format_json_includes_all_5_org_fields` (`.credentials.status`, all 5 keys asserted) + `acc45_json_includes_org_uuid` (`.accounts`, `organization_uuid` only) (in `credentials_test_b.rs` / `accounts_list_test_b.rs`)
- **Source:** [022_org_identity_snapshot.md AC-09](../../../docs/feature/022_org_identity_snapshot.md)

---

### FT-10: `--no-default-features` compile passes without `claude_quota` dep

- **Given:** `claude_profile_core` crate built without the `enabled` feature (`default-features = false`); `claude_quota` is feature-gated behind `dep:claude_quota`.
- **When:** `cargo check -p claude_profile_core --no-default-features`
- **Then:** Compilation exits 0 with no errors. `fetch_claude_cli_roles()` transport is excluded from the build.
- **Exit:** 0 (cargo exit code)
- **Source fn:** n/a (compile gate — verified by `cargo check`)
- **Source:** [022_org_identity_snapshot.md AC-10](../../../docs/feature/022_org_identity_snapshot.md)

---

### FT-11: Null workspace fields in `{name}.json` render as empty string and `N/A`

- **Given:** Account `user@example.com` with `{credential_store}/user@example.com.json` containing `"workspace_uuid":null,"workspace_name":null` (`write_account_roles_json` always writes these two fields as `null` — personal account / no workspace membership); file also contains valid `organization_uuid` and `organization_name`.
- **When:** `clp .credentials.status format::json`
- **Then:** In JSON output, `workspace_uuid` and `workspace_name` are present as `""` (null API values normalized to empty string), i.e. `"workspace_uuid":""` and `"workspace_name":""`. Exit 0.
- **Exit:** 0
- **Note:** Corrected — the previous When/Then described `.accounts org_uuid::1 org_name::1` (text output) and `.accounts format::json`; the cited test actually exercises only `.credentials.status format::json` — a different command entirely, and JSON-only (no text-output/`Org ID:`/`Org:` assertion at all). Separately, `org_uuid::1`/`org_name::1` as standalone boolean params on `.accounts` were removed (Feature 037; see FT-05/FT-06's corrected `cols::+org_uuid`/`cols::+org_name` mechanism) — the doc's original `.accounts`-side "When" would have errored had it been run as written. The null-to-empty-string normalization is confirmed structurally true for `.accounts` as well (same `parse_string_field()` mechanism — `claude_profile_core/src/account/json_field.rs` (account.rs:1888-1897 pre-split) returns `None` for a bare JSON `null`, and `.unwrap_or_default()` in `list()` (store.rs) turns that into `""`), but no `.accounts`-side test exercises this specific null-workspace scenario.
- **Source fn:** `cred46_ft11_null_workspace_fields_render_as_empty_string` (in `credentials_test_b.rs`)
- **Source:** [022_org_identity_snapshot.md AC-11](../../../docs/feature/022_org_identity_snapshot.md)
