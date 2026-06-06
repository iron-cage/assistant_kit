# Test: Feature 021 — Extended Snapshot Fields

Feature behavioral requirement test cases for `docs/feature/021_extended_snapshot_fields.md` (FR-21). Each FT case maps to one acceptance criterion. Parameter edge cases are in [cli/param/028_uuid.md](../cli/param/028_uuid.md) and [cli/param/029_capabilities.md](../cli/param/029_capabilities.md).

### AC Coverage Index

| FT | Criterion | AC | Category |
|----|-----------|----|---------|
| FT-01 | `uuid::1` on `.credentials.status` shows `ID:` from `taggedId` | AC-01 | Field Presence |
| FT-02 | `capabilities::1` on `.credentials.status` shows `Capabilities:` list | AC-02 | Field Presence |
| FT-03 | `uuid::1` on `.accounts` shows `ID:` per account from snapshot | AC-03 | Field Presence |
| FT-04 | `capabilities::1` on `.accounts` shows `Capabilities:` per account | AC-04 | Field Presence |
| FT-05 | Both params default to `0` — absent from default output | AC-05 | Default Behavior |
| FT-06 | `format::json` always includes `tagged_id` and `capabilities` keys | AC-06 | JSON Output |
| FT-07 | Absent fields in snapshot render `N/A` / `[]` without error | AC-07 | Missing Data |
| FT-08 | `parse_string_array_field` extracts string array values correctly | AC-08 | Parser |
| FT-09 | Empty `capabilities` array renders `N/A` in text, `[]` in JSON | AC-09 | Empty Array |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|----|---------|
| FT-01 | uuid::1 shows ID line on credentials.status | AC-01 | Field Presence |
| FT-02 | capabilities::1 shows Capabilities on credentials.status | AC-02 | Field Presence |
| FT-03 | uuid::1 shows ID per account on .accounts | AC-03 | Field Presence |
| FT-04 | capabilities::1 shows Capabilities per account on .accounts | AC-04 | Field Presence |
| FT-05 | No ID or Capabilities line in default output | AC-05 | Default Behavior |
| FT-06 | JSON always includes tagged_id and capabilities | AC-06 | JSON Output |
| FT-07 | Missing oauthAccount fields render N/A without error | AC-07 | Missing Data |
| FT-08 | parse_string_array_field extracts quoted strings from JSON array | AC-08 | Parser Unit Test |
| FT-09 | Empty capabilities array shows N/A in text and [] in JSON | AC-09 | Empty Array |

**Total:** 9 FT cases

---

### FT-01: `uuid::1` shows `ID:` line on `.credentials.status`

- **Given:** Active account set; the active account's `{name}.json` in the credential store contains `"oauthAccount":{"taggedId":"user_01AbCdEfGh","uuid":"xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"}`.
- **When:** `clp .credentials.status uuid::1`
- **Then:** Stdout contains a line matching `ID:` followed by `user_01AbCdEfGh`. Exit 0.
- **Exit:** 0
- **Source fn:** `cred16_uuid_opt_in_shows_id_line` (in `tests/cli/credentials_test.rs`)
- **Source:** [021_extended_snapshot_fields.md AC-01](../../../../docs/feature/021_extended_snapshot_fields.md)

---

### FT-02: `capabilities::1` shows `Capabilities:` line on `.credentials.status`

- **Given:** Active account set; the active account's `{name}.json` contains `"capabilities":["claude_max","chat"]`.
- **When:** `clp .credentials.status capabilities::1`
- **Then:** Stdout contains a line matching `Capabilities:` followed by `claude_max, chat` (comma-separated). Exit 0.
- **Exit:** 0
- **Source fn:** `cred23_capabilities_opt_in_shows_list` (in `tests/cli/credentials_test.rs`)
- **Source:** [021_extended_snapshot_fields.md AC-02](../../../../docs/feature/021_extended_snapshot_fields.md)

---

### FT-03: `uuid::1` shows `ID:` per account on `.accounts`

- **Given:** Two saved accounts `alice@a.com` and `bob@a.com`; each `{name}.json` contains `taggedId` values `"user_01Alice"` and `"user_01Bob"` respectively.
- **When:** `clp .accounts uuid::1`
- **Then:** Stdout contains an `ID: user_01Alice` line for alice's block and an `ID: user_01Bob` line for bob's block. Exit 0.
- **Exit:** 0
- **Source fn:** `acc35_uuid_shows_id_from_snapshot` (in `tests/cli/accounts_test.rs`)
- **Source:** [021_extended_snapshot_fields.md AC-03](../../../../docs/feature/021_extended_snapshot_fields.md)

---

### FT-04: `capabilities::1` shows `Capabilities:` per account on `.accounts`

- **Given:** Two saved accounts; each `{name}.json` contains `"capabilities":["claude_max"]`.
- **When:** `clp .accounts capabilities::1`
- **Then:** Stdout contains a `Capabilities: claude_max` line for each account block. Exit 0.
- **Exit:** 0
- **Source fn:** `acc38_capabilities_shows_list_from_snapshot` (in `tests/cli/accounts_test.rs`)
- **Source:** [021_extended_snapshot_fields.md AC-04](../../../../docs/feature/021_extended_snapshot_fields.md)

---

### FT-05: No `ID:` or `Capabilities:` line in default output

- **Given:** Active account set; `{name}.json` contains both `taggedId` and `capabilities` fields.
- **When:** `clp .credentials.status` (no `uuid::` or `capabilities::` params)
- **Then:** Stdout does NOT contain `ID:` or `Capabilities:` lines. All other standard credential fields appear. Exit 0.
- **Exit:** 0
- **Note:** AC-05 applies equally to `.accounts` — verified by absence of these lines in default `.accounts` output.
- **Source fn:** `cred19_uuid_absent_by_default` + `cred26_capabilities_absent_by_default` (in `tests/cli/credentials_test.rs`)
- **Source:** [021_extended_snapshot_fields.md AC-05](../../../../docs/feature/021_extended_snapshot_fields.md)

---

### FT-06: `format::json` always includes `tagged_id` and `capabilities` keys

- **Given:** Active account with `taggedId` and `capabilities` in `{name}.json`. `uuid::` and `capabilities::` params are NOT passed.
- **When:** `clp .credentials.status format::json`
- **Then:** JSON output contains a `tagged_id` key (string value, e.g. `"user_01AbCdEfGh"`) and a `capabilities` key (array value). Both keys present regardless of `uuid::` / `capabilities::` flags. Exit 0.
- **Exit:** 0
- **Note:** Same invariant applies to `clp .accounts format::json`.
- **Source fn:** `cred21_uuid_json_always_includes_tagged_id` + `cred28_capabilities_json_always_emits_key` (in `tests/cli/credentials_test.rs`)
- **Source:** [021_extended_snapshot_fields.md AC-06](../../../../docs/feature/021_extended_snapshot_fields.md)

---

### FT-07: Missing `oauthAccount` fields render `N/A` without error

- **Given:** Active account whose `{name}.json` does NOT contain `taggedId` or `capabilities` keys in `oauthAccount` (or has no `oauthAccount` object at all).
- **When:** `clp .credentials.status uuid::1 capabilities::1`
- **Then:** Stdout contains `ID: N/A` and `Capabilities: N/A`. Exit 0. No error message on stderr.
- **Exit:** 0
- **Source fn:** `cred22_uuid_missing_tagged_id_shows_na` + `cred30_capabilities_missing_field_shows_na` (in `tests/cli/credentials_test.rs`)
- **Source:** [021_extended_snapshot_fields.md AC-07](../../../../docs/feature/021_extended_snapshot_fields.md)

---

### FT-08: `parse_string_array_field` extracts quoted strings from JSON array

- **Given:** Unit test. Input JSON string `{"capabilities":["claude_max","chat"]}`.
- **When:** `parse_string_array_field(json, "capabilities")`
- **Then:** Returns `vec!["claude_max", "chat"]` (two elements, values match exactly). Also: missing key returns empty `Vec`; empty array `[]` returns empty `Vec`.
- **Exit:** n/a (unit test — function return assertion)
- **Source fn:** `ft08_parse_string_array_field_two_elements` + `ft08_parse_string_array_field_missing_key_returns_empty` + `ft08_parse_string_array_field_empty_array_returns_empty` (in `claude_profile_core/src/account.rs` `#[cfg(test)]` block)
- **Source:** [021_extended_snapshot_fields.md AC-08](../../../../docs/feature/021_extended_snapshot_fields.md)

---

### FT-09: Empty `capabilities` array renders `N/A` in text and `[]` in JSON

- **Given:** Active account whose `{name}.json` contains `"capabilities":[]` (present but empty array).
- **When:** `clp .credentials.status capabilities::1`
- **Then:** Stdout contains `Capabilities: N/A`. When `clp .credentials.status format::json` (no `capabilities::` param), the JSON output contains `"capabilities":[]`. Exit 0 for both.
- **Exit:** 0
- **Source fn:** `cred29_capabilities_empty_array_shows_na` (in `tests/cli/credentials_test.rs`)
- **Source:** [021_extended_snapshot_fields.md AC-09](../../../../docs/feature/021_extended_snapshot_fields.md)
