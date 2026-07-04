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
| FT-05 | `org_uuid::1` on `.accounts` shows `Org ID:` per account from snapshot | AC-05 | Field Presence |
| FT-06 | `org_name::1` on `.accounts` shows `Org:` per account from snapshot | AC-06 | Field Presence |
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
| FT-05 | org_uuid::1 shows Org ID per account on .accounts | AC-05 | Field Presence |
| FT-06 | org_name::1 shows Org per account on .accounts | AC-06 | Field Presence |
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
- **Source fn:** `as20_lim_it_save_writes_roles_json` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [022_org_identity_snapshot.md AC-01](../../../docs/feature/022_org_identity_snapshot.md)

---

### FT-02: Endpoint 005 failure → no org identity written; `save` still exits 0

- **Given:** An account credential file with an invalid `accessToken` value (causes endpoint 005 to return HTTP 401); no pre-existing `{name}.json` in the credential store.
- **When:** `clp .account.save`
- **Then:** `{credential_store}/{name}.json` does NOT contain org identity fields after the command exits. Exit 0. No fatal error on stderr about roles; all other save operations complete normally.
- **Exit:** 0
- **Source fn:** `as19_save_best_effort_no_roles_json` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [022_org_identity_snapshot.md AC-02](../../../docs/feature/022_org_identity_snapshot.md)

---

### FT-03: Re-`save` overwrites org identity in `{name}.json` with fresh data

- **Given:** Account `alice@example.com` has an existing `alice@example.com.json` in the credential store (stale org content); the active `accessToken` is valid; endpoint 005 returns an updated response on the second call.
- **When:** `clp .account.save` (second invocation with the same name)
- **Then:** `{credential_store}/alice@example.com.json` is overwritten; the file contains org data from the second API response. Exit 0.
- **Exit:** 0
- **Live:** yes (lim_it — requires valid Anthropic credentials; verifies overwrite with real API response)
- **Source fn:** `as21_lim_it_resave_overwrites_roles_json` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [022_org_identity_snapshot.md AC-03](../../../docs/feature/022_org_identity_snapshot.md)

---

### FT-04: `delete` removes `{name}.json`; absent file causes no error

- **Given:** Account `alice@acme.com` whose credential store contains a pre-seeded `alice@acme.com.json` fixture. Account `bob@acme.com` whose credential store does NOT have `bob@acme.com.json`.
- **When:** `clp .account.delete name::alice@acme.com` and separately `clp .account.delete name::bob@acme.com`
- **Then:** After alice's delete: `{credential_store}/alice@acme.com.json` no longer exists; exit 0. After bob's delete: exits 0 with no error message about missing snapshot.
- **Exit:** 0
- **Source fn:** `ad15_delete_removes_roles_json` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [022_org_identity_snapshot.md AC-04](../../../docs/feature/022_org_identity_snapshot.md)

---

### FT-05: `org_uuid::1` shows `Org ID:` per account on `.accounts`

- **Given:** Two accounts `alice@acme.com` and `bob@acme.com`; `alice@acme.com.json` exists in the credential store with `"organization_uuid":"aaaaaaaa-1111-cccc-dddd-eeeeeeeeeeee"`; `bob@acme.com.json` does NOT exist.
- **When:** `clp .accounts org_uuid::1`
- **Then:** Alice's account block in stdout contains `Org ID:` followed by `aaaaaaaa-1111-cccc-dddd-eeeeeeeeeeee`. Bob's account block contains `Org ID: N/A`. Exit 0.
- **Exit:** 0
- **Source fn:** `acc42_org_uuid_shows_from_roles_json` (in `tests/cli/accounts_test.rs`)
- **Source:** [022_org_identity_snapshot.md AC-05](../../../docs/feature/022_org_identity_snapshot.md)

---

### FT-06: `org_name::1` shows `Org:` per account on `.accounts`

- **Given:** Two accounts `alice@acme.com` and `bob@acme.com`; `alice@acme.com.json` exists with `"organization_name":"Acme Corp"`; `bob@acme.com.json` does NOT exist.
- **When:** `clp .accounts org_name::1`
- **Then:** Alice's account block contains `Org:` followed by `Acme Corp`. Bob's account block contains `Org: N/A`. Exit 0.
- **Exit:** 0
- **Source fn:** `acc46_org_name_shows_from_roles_json` (in `tests/cli/accounts_test.rs`)
- **Source:** [022_org_identity_snapshot.md AC-06](../../../docs/feature/022_org_identity_snapshot.md)

---

### FT-07: `org_uuid::1` shows `Org ID:` on `.credentials.status` from active `{name}.json`

- **Given:** Active account `alice@acme.com`; `_active` marker points to alice; `{credential_store}/alice@acme.com.json` exists with `"organization_uuid":"aaaaaaaa-1111-cccc-dddd-eeeeeeeeeeee"`.
- **When:** `clp .credentials.status org_uuid::1`
- **Then:** Stdout contains `Org ID:` followed by `aaaaaaaa-1111-cccc-dddd-eeeeeeeeeeee`. Exit 0.
- **Exit:** 0
- **Source fn:** `cred31_org_uuid_shows_org_id_line` (in `tests/cli/credentials_test.rs`)
- **Source:** [022_org_identity_snapshot.md AC-07](../../../docs/feature/022_org_identity_snapshot.md)

---

### FT-08: `org_name::1` shows `Org:` on `.credentials.status` from active `{name}.json`

- **Given:** Active account `alice@acme.com`; `_active` marker points to alice; `{credential_store}/alice@acme.com.json` exists with `"organization_name":"Acme Corp"`.
- **When:** `clp .credentials.status org_name::1`
- **Then:** Stdout contains `Org:` followed by `Acme Corp`. Exit 0.
- **Exit:** 0
- **Source fn:** `cred38_org_name_shows_org_line` (in `tests/cli/credentials_test.rs`)
- **Source:** [022_org_identity_snapshot.md AC-08](../../../docs/feature/022_org_identity_snapshot.md)

---

### FT-09: `format::json` always includes all 5 org fields regardless of params

- **Given:** Active account `alice@acme.com` with `alice@acme.com.json` containing all 5 org fields (`organization_uuid`, `organization_name`, `organization_role`, `workspace_uuid`, `workspace_name`). `org_uuid::` and `org_name::` params are NOT passed.
- **When:** `clp .credentials.status format::json` and separately `clp .accounts format::json`
- **Then:** Both JSON outputs contain `organization_uuid`, `organization_name`, `organization_role`, `workspace_uuid`, and `workspace_name` keys regardless of display params. Exit 0 for both.
- **Exit:** 0
- **Source fn:** `cred45_ft09_format_json_includes_all_5_org_fields` (in `tests/cli/credentials_test.rs`)
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

- **Given:** Account `alice@personal.com` with `{credential_store}/alice@personal.com.json` containing `"workspace_uuid":null,"workspace_name":null` (personal account with no workspace membership); file also contains valid `organization_uuid` and `organization_name`.
- **When:** `clp .accounts org_uuid::1 org_name::1` and `clp .accounts format::json`
- **Then:** In text output, `Org ID:` and `Org:` lines appear with the organization values. In JSON output, `workspace_uuid` and `workspace_name` are present as `""` (null API values normalized to empty string in `Account` struct). Exit 0 for both.
- **Exit:** 0
- **Source fn:** `cred46_ft11_null_workspace_fields_render_as_empty_string` (in `tests/cli/credentials_test.rs`)
- **Source:** [022_org_identity_snapshot.md AC-11](../../../docs/feature/022_org_identity_snapshot.md)
