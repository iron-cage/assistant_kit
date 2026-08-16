# Format 002: json

FM test cases for `docs/cli/format/002_json.md`. Verifies the `format::json` output
contract: single-line JSON, object vs array shape by command type, field-presence param
ignorance, incompatibility with `live::1`, and error row representation.

**Source:** [docs/cli/format/002_json.md](../../../../docs/cli/format/002_json.md)

### FM Case Index

| ID | Short Name | Category | Status |
|----|------------|----------|--------|
| FM-1 | Multi-record commands output JSON array; single-record commands output JSON object | Structure | ✅ |
| FM-2 | `format::json` ignores field-presence params — all fields always appear | Field Presence | ✅ |
| FM-3 | `format::json` combined with `live::1` exits 1 (incompatible combination) | Incompatibility | ✅ |
| FM-4 | Error accounts in `.usage` JSON appear as `{"account":"...","error":"..."}` | Error Representation | ✅ |

**Behavioral Divergence Pair:** FM-2 (`format::json` with `cols::-sub,-tier,-active` — field suppression params silently ignored, all fields always serialized) ↔ FM-3 (`format::json` with `live::1` — incompatible combination exits 1 before any fetch; some params are blocked outright while field-presence params are silently overridden)

---

### FM-1: Multi-record commands → JSON array; single-record commands → JSON object

- **Given:** `.accounts format::json` (multi-record) in one test, and `.credentials.status format::json` (single-record) in another test
- **When:** Each command's output is captured
- **Then:** `.accounts` output parses as a JSON array (`.as_array()` succeeds); `.credentials.status` output parses as a JSON object (`.as_object()` succeeds) — the record multiplicity determines the top-level JSON shape
- **Source fn:** `account_nc2_json_output_schema_valid` (array half) and `credentials_nc2_json_output_schema_valid` (object half) (both cli/command_noun_test.rs)
- **Source:** [docs/cli/format/002_json.md §Structure](../../../../docs/cli/format/002_json.md)

---

### FM-2: Field-presence params are ignored in JSON mode — all fields always included

- **Given:** `.accounts cols::-sub,-tier,-active format::json` (field-presence suppression params present; `.accounts` no longer accepts the legacy `sub::`/`tier::` boolean toggles directly — Feature 037 replaced them with `cols::`)
- **When:** The command runs
- **Then:** JSON output still contains `subscription_type`, `rate_limit_tier`, and `is_active` fields — `format::json` overrides field-presence toggles; all fields serialize unconditionally
- **Source fn:** `acc10_json_ignores_field_presence` (cli/accounts_list_test.rs)
- **Source:** [docs/cli/format/002_json.md §Notes](../../../../docs/cli/format/002_json.md)

---

### FM-3: `format::json` combined with `live::1` exits 1 before any fetch

- **Given:** `.usage format::json live::1`
- **When:** The command runs
- **Then:** Exits with code 1 before performing any API fetch — `format::json` and `live::1` are mutually incompatible; an error message is emitted
- **Source fn:** `it024_live_incompatible_with_json` (cli/usage_live_test.rs)
- **Source:** [docs/cli/format/002_json.md §Notes](../../../../docs/cli/format/002_json.md)

---

### FM-4: Error accounts in `.usage` JSON appear as `{"account":"...","error":"..."}` objects

- **Given:** `.usage format::json` where one account has a fetch error (e.g., network failure)
- **When:** The command runs
- **Then:** The error account appears in the JSON array as `{"account": "alice@example.com", "error": "..."}` — no full quota fields; error is inline alongside successful account objects
- **Source fn:** `it027_json_error_field_on_failed_account` (cli/usage_live_test.rs)
- **Source:** [docs/cli/format/002_json.md §Structure §Notes](../../../../docs/cli/format/002_json.md)
