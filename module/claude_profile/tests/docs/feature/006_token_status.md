# Test: Feature 006 — Token Status

### Scope

- **Purpose**: Test cases for token expiry classification (Valid/ExpiringSoon/Expired) and threshold behavior.
- **Source**: `docs/feature/006_token_status.md`
- **Covers**: AC-01 through AC-04

Feature behavioral requirement test cases for `docs/feature/006_token_status.md` (FR-11). Each FT case maps to one or more acceptance criteria.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | Valid and expired token classification at default threshold | AC-01 |
| FT-02 | Token within threshold classified as `ExpiringSoon` | AC-02 |
| FT-03 | `threshold::1800` changes classification boundary | AC-03 |
| FT-04 | `format::json` returns structured status with `expires_in_secs` | AC-04 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | Far-future token → Valid; past token → Expired | AC-01 | Classification |
| FT-02 | Token within 3600s threshold → ExpiringSoon | AC-02 | Classification |
| FT-03 | Custom `threshold::1800` changes boundary to 30 min | AC-03 | Custom Threshold |
| FT-04 | `format::json` returns `{"status":…,"expires_in_secs":N}` | AC-04 | JSON Format |

**Total:** 4 FT cases

---

### FT-01: Far-future token → Valid; past token → Expired

- **Given:** A credentials file where `expiresAt` is more than 3600 seconds in the future (Valid case) or in the past (Expired case).
- **When:** `token::status(credential_store, paths)` is called.
- **Then (Valid):** Returns `TokenStatus::Valid`.
- **Then (Expired):** Returns `TokenStatus::Expired`.
- **Exit:** Ok
- **Source fn:** `status_returns_valid_when_far_future`, `status_returns_expired_when_expires_at_in_past`
- **Source:** [006_token_status.md AC-01](../../../docs/feature/006_token_status.md)

---

### FT-02: Token within 3600s threshold → ExpiringSoon

- **Given:** A credentials file where `expiresAt` is in the future but within 3600 seconds of now.
- **When:** `token::status(credential_store, paths)` is called with default threshold.
- **Then:** Returns `TokenStatus::ExpiringSoon`.
- **Exit:** Ok
- **Source fn:** `status_returns_expiring_soon_within_default_threshold`
- **Source:** [006_token_status.md AC-02](../../../docs/feature/006_token_status.md)

---

### FT-03: Custom `threshold::1800` changes boundary to 30 min

- **Given:** A credentials file where `expiresAt` is 2000 seconds in the future (within 1800s but outside default 3600s).
- **When:** `token::status_with_threshold(credential_store, paths, 1800)` is called.
- **Then:** Returns `TokenStatus::ExpiringSoon` (within custom threshold). With default threshold (3600s) it would have returned `TokenStatus::ExpiringSoon` too, but at 4000s it would return `Valid` with custom threshold.
- **Exit:** Ok
- **Source fn:** `status_with_custom_threshold_classifies_correctly`, `status_with_threshold_zero_classifies_non_expired_as_expiring_soon`
- **Source:** [006_token_status.md AC-03](../../../docs/feature/006_token_status.md)

---

### FT-04: `format::json` returns `{"status":…,"expires_in_secs":N}`

- **Given:** A valid credentials file with a future `expiresAt`.
- **When:** `clp .token.status format::json`
- **Then:** Output is valid JSON containing `"status"` and `"expires_in_secs"` keys. `"status"` is one of `"valid"`, `"expiring_soon"`, `"expired"`. `"expires_in_secs"` is a non-negative integer.
- **Exit:** 0
- **Source fn:** `ts06_token_valid_json`, `ts07_token_expired_json`, `ts14_token_expiring_soon_json`
- **Source:** [006_token_status.md AC-04](../../../docs/feature/006_token_status.md)
