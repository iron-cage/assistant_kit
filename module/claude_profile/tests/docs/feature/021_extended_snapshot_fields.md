# Test: Feature 021 — Extended Snapshot Fields

### Scope

- **Purpose**: Test cases for extended snapshot fields capture.
- **Source**: `docs/feature/021_extended_snapshot_fields.md`
- **Covers**: AC-01 through AC-09

Feature behavioral requirement test cases for `docs/feature/021_extended_snapshot_fields.md` (FR-21). Each FT case maps to one acceptance criterion. Parameter edge cases are in [cli/param/028_uuid.md](../cli/param/28_uuid.md) and [cli/param/029_capabilities.md](../cli/param/29_capabilities.md).

### AC Coverage Index

| FT | Criterion | AC | Category |
|----|-----------|----|---------|
| FT-01 | `uuid::1` on `.credentials.status` shows `ID:` from `taggedId` | AC-01 | Field Presence |
| FT-02 | `capabilities::1` on `.credentials.status` shows `Capabilities:` list | AC-02 | Field Presence |
| FT-03 | `cols::+uuid` on `.accounts` shows `ID:` per account from snapshot | AC-03 | Field Presence |
| FT-04 | `cols::+capabilities` on `.accounts` shows `Capabilities:` per account | AC-04 | Field Presence |
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
| FT-03 | cols::+uuid shows ID per account on .accounts | AC-03 | Field Presence |
| FT-04 | cols::+capabilities shows Capabilities per account on .accounts | AC-04 | Field Presence |
| FT-05 | No ID or Capabilities line in default output | AC-05 | Default Behavior |
| FT-06 | JSON always includes tagged_id and capabilities | AC-06 | JSON Output |
| FT-07 | Missing oauthAccount fields render N/A without error | AC-07 | Missing Data |
| FT-08 | parse_string_array_field extracts quoted strings from JSON array | AC-08 | Parser Unit Test |
| FT-09 | Empty capabilities array shows N/A in text and [] in JSON | AC-09 | Empty Array |

**Total:** 9 FT cases

---

### FT-01: `uuid::1` shows `ID:` line on `.credentials.status`

- **Given:** `~/.claude.json` (Claude Code's own live account file, not a per-account credential-store snapshot) contains `"oauthAccount":{"taggedId":"user_abc123","uuid":"some-uuid","capabilities":["claude_code"]}`.
- **When:** `clp .credentials.status uuid::1`
- **Then:** Stdout contains a line matching `ID:` followed by `user_abc123`. Exit 0.
- **Exit:** 0
- **Note:** Corrected — the previous Given described the source as "the active account's `{name}.json` in the credential store," which is the mechanism for `.accounts` (FT-03/FT-04), not `.credentials.status`. `.credentials.status` reads `taggedId`/`uuid`/`capabilities` from `~/.claude.json` directly (via `write_claude_json_extended` in the cited test), independent of any saved/named account.
- **Source fn:** `cred16_uuid_opt_in_shows_id_line` (in `tests/cli/credentials_test.rs`)
- **Source:** [021_extended_snapshot_fields.md AC-01](../../../docs/feature/021_extended_snapshot_fields.md)

---

### FT-02: `capabilities::1` shows `Capabilities:` line on `.credentials.status`

- **Given:** `~/.claude.json` (Claude Code's own live account file, not a per-account credential-store snapshot) contains `"capabilities":["claude_code","pro"]`.
- **When:** `clp .credentials.status capabilities::1`
- **Then:** Stdout contains a line matching `Capabilities:` followed by `claude_code, pro` (comma-separated). Exit 0.
- **Exit:** 0
- **Note:** Corrected — the previous Given implied the data source was a per-account credential-store `{name}.json`; `.credentials.status` reads `capabilities` from `~/.claude.json` directly, independent of any saved/named account. Values also corrected to match the cited test's actual fixture (`claude_code`, `pro`), not `claude_max`/`chat`.
- **Source fn:** `cred23_capabilities_opt_in_shows_list` (in `tests/cli/credentials_test.rs`)
- **Source:** [021_extended_snapshot_fields.md AC-02](../../../docs/feature/021_extended_snapshot_fields.md)

---

### FT-03: `cols::+uuid` shows `ID:` per account on `.accounts`

- **Given:** Saved account `alice@acme.com`; its `{name}.json` (`~/.persistent/claude/credential/alice@acme.com.json`) contains `"oauthAccount":{"taggedId":"user_abc123",...}`.
- **When:** `clp .accounts cols::+uuid`
- **Then:** Stdout contains an `ID: user_abc123` line for alice's block. Exit 0.
- **Exit:** 0
- **Note:** Corrected — `uuid::1` as a standalone boolean param on `.accounts` was removed (Feature 037); passing it now errors with `"parameter 'uuid' removed — use 'cols::+uuid' instead"` (confirmed via `src/commands/accounts.rs:315-343`'s `REMOVED_TOGGLES` rejection list). The real mechanism is the `cols::` column-visibility modifier. The cited test also exercises only one account, not two.
- **Source fn:** `acc35_uuid_shows_id_from_snapshot` (in `accounts_list_test_b.rs`)
- **Source:** [021_extended_snapshot_fields.md AC-03](../../../docs/feature/021_extended_snapshot_fields.md)

---

### FT-04: `cols::+capabilities` shows `Capabilities:` per account on `.accounts`

- **Given:** Saved account `alice@acme.com`; its `{name}.json` contains `"oauthAccount":{"capabilities":["claude_max","chat"]}`.
- **When:** `clp .accounts cols::+capabilities`
- **Then:** Stdout contains a `Capabilities: claude_max, chat` line for alice's block. Exit 0.
- **Exit:** 0
- **Note:** Corrected — `capabilities::1` as a standalone boolean param on `.accounts` was removed (Feature 037); passing it now errors with `"parameter 'capabilities' removed — use 'cols::+capabilities' instead"` (confirmed via `src/commands/accounts.rs:315-343`). The real mechanism is the `cols::` column-visibility modifier. The cited test also exercises only one account, not two.
- **Source fn:** `acc38_capabilities_shows_list_from_snapshot` (in `accounts_list_test_b.rs`)
- **Source:** [021_extended_snapshot_fields.md AC-04](../../../docs/feature/021_extended_snapshot_fields.md)

---

### FT-05: No `ID:` or `Capabilities:` line in default output

- **Given:** Active account set; `{name}.json` contains both `taggedId` and `capabilities` fields.
- **When:** `clp .credentials.status` (no `uuid::` or `capabilities::` params)
- **Then:** Stdout does NOT contain `ID:` or `Capabilities:` lines. All other standard credential fields appear. Exit 0.
- **Exit:** 0
- **Note:** AC-05 applies equally to `.accounts` — verified by absence of these lines in default `.accounts` output.
- **Source fn:** `cred19_uuid_absent_by_default` (in `credentials_test.rs`) + `cred26_capabilities_absent_by_default` (in `credentials_test_b.rs`) — corrected; the first was previously misattributed to `credentials_test_b.rs`.
- **Source:** [021_extended_snapshot_fields.md AC-05](../../../docs/feature/021_extended_snapshot_fields.md)

---

### FT-06: `format::json` always includes `tagged_id` and `capabilities` keys

- **Given:** Active account with `taggedId` and `capabilities` in `{name}.json`. `uuid::` and `capabilities::` params are explicitly set to `0` (disabled) — proving the JSON keys survive even the opt-out flag, a stronger demonstration than merely omitting the params.
- **When:** `clp .credentials.status format::json uuid::0` (tagged_id case) / `clp .credentials.status format::json capabilities::0` (capabilities case) — two separate invocations, one per cited test.
- **Then:** JSON output contains a `tagged_id` key (string value, e.g. `"user_abc123"`) and, separately, a `capabilities` key (array value, e.g. containing `"claude_code"`). Both keys present regardless of the `uuid::0` / `capabilities::0` flags. Exit 0.
- **Exit:** 0
- **Note:** Same invariant applies to `clp .accounts format::json`. Corrected — the previous Given/When claimed the two opt-in params were simply omitted and combined into one `format::json` call; the cited tests instead each explicitly pass their own param as `::0` (disabled) in two separate invocations.
- **Source fn:** `cred21_uuid_json_always_includes_tagged_id` (in `credentials_test.rs`) + `cred28_capabilities_json_always_emits_key` (in `credentials_test_b.rs`) — corrected; the first was previously misattributed to `credentials_test_b.rs`.
- **Source:** [021_extended_snapshot_fields.md AC-06](../../../docs/feature/021_extended_snapshot_fields.md)

---

### FT-07: Missing `oauthAccount` fields render `N/A` without error

- **Given:** Active account whose `{name}.json` does NOT contain `taggedId` or `capabilities` keys in `oauthAccount` (or has no `oauthAccount` object at all). Both cited tests use `write_claude_json(...)`, which writes only `{"oauthAccount":{"emailAddress":"..."}}`.
- **When:** `clp .credentials.status uuid::1` (tagged_id case) / `clp .credentials.status capabilities::1` (capabilities case) — two separate invocations, one per cited test; neither test passes both params together.
- **Then:** For the `uuid::1` invocation, stdout contains `ID:` and `N/A`. For the `capabilities::1` invocation, stdout contains `Capabilities:` and `N/A`. Exit 0 for both. No error message on stderr.
- **Exit:** 0
- **Note:** Corrected — the previous When/Then combined both params into a single `uuid::1 capabilities::1` call; the cited tests each pass only their own param.
- **Source fn:** `cred22_uuid_missing_tagged_id_shows_na` (in `credentials_test.rs`) + `cred30_capabilities_missing_field_shows_na` (in `credentials_test_b.rs`) — corrected; the first was previously misattributed to `credentials_test_b.rs`.
- **Source:** [021_extended_snapshot_fields.md AC-07](../../../docs/feature/021_extended_snapshot_fields.md)

---

### FT-08: `parse_string_array_field` extracts quoted strings from JSON array

- **Given:** Unit test. Input JSON string `{"capabilities":["claude_max","chat"]}`.
- **When:** `parse_string_array_field(json, "capabilities")`
- **Then:** Returns `vec!["claude_max", "chat"]` (two elements, values match exactly). Also: missing key returns empty `Vec`; empty array `[]` returns empty `Vec`.
- **Exit:** n/a (unit test — function return assertion)
- **Source fn:** `ft08_parse_string_array_field_two_elements` + `ft08_parse_string_array_field_missing_key_returns_empty` + `ft08_parse_string_array_field_empty_array_returns_empty` (in `claude_profile_core/src/account.rs` `#[cfg(test)]` block)
- **Source:** [021_extended_snapshot_fields.md AC-08](../../../docs/feature/021_extended_snapshot_fields.md)

---

### FT-09: Empty `capabilities` array renders `N/A` in text and `[]` in JSON

- **Given:** `~/.claude.json` contains `"capabilities":[]` (present but empty array).
- **When:** `clp .credentials.status capabilities::1`
- **Then:** Stdout contains `Capabilities: N/A`. Exit 0.
- **Exit:** 0
- **Note:** Corrected — the previous Given described the source as a per-account `{name}.json`; `.credentials.status` reads `capabilities` from `~/.claude.json` directly. The `format::json` claim ("JSON output contains `"capabilities":[]`") is not exercised by the cited test at all (it only checks the text format via `capabilities::1`), but is confirmed true by direct source inspection: `caps_to_json()` (`src/commands/cmd_context.rs:78-85`) returns the literal string `"[]"` when its input slice is empty, and this is the function used to render the `capabilities` JSON key.
- **Source fn:** `cred29_capabilities_empty_array_shows_na` (in `credentials_test_b.rs`)
- **Source:** [021_extended_snapshot_fields.md AC-09](../../../docs/feature/021_extended_snapshot_fields.md)
