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
| FT-03 | Custom threshold changes classification boundary (incl. zero-threshold edge case) | AC-03 |
| FT-04 | `format::json` returns structured status with `expires_in_secs` | AC-04 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | Far-future token → Valid; past token → Expired | AC-01 | Classification |
| FT-02 | Token within 3600s threshold → ExpiringSoon | AC-02 | Classification |
| FT-03 | Custom threshold (7200s) widens window; zero threshold narrows to exact boundary | AC-03 | Custom Threshold |
| FT-04 | `format::json` returns `"token"`/`"expires_in_secs"` fields | AC-04 | JSON Format |

**Total:** 4 FT cases

---

### FT-01: Far-future token → Valid; past token → Expired

- **Given:** A credentials file where `expiresAt` is more than 3600 seconds in the future (Valid case) or in the past (Expired case).
- **When:** `token::status()` is called.
- **Then (Valid):** Returns `TokenStatus::Valid`.
- **Then (Expired):** Returns `TokenStatus::Expired`.
- **Exit:** Ok
- **Source fn:** `status_returns_valid_when_far_future`, `status_returns_expired_when_expires_at_in_past`
- **Source:** [006_token_status.md AC-01](../../../docs/feature/006_token_status.md)

---

### FT-02: Token within 3600s threshold → ExpiringSoon

- **Given:** A credentials file where `expiresAt` is in the future but within 3600 seconds of now.
- **When:** `token::status()` is called with default threshold.
- **Then:** Returns `TokenStatus::ExpiringSoon`.
- **Exit:** Ok
- **Source fn:** `status_returns_expiring_soon_within_default_threshold`
- **Source:** [006_token_status.md AC-02](../../../docs/feature/006_token_status.md)

---

### FT-03: Custom threshold (7200s) widens window; zero threshold narrows to exact boundary

- **Given (a):** A credentials file where `expiresAt` is 3600 seconds (1 hour) in the future.
- **When (a):** `token::status_with_threshold(7200)` is called (a custom 2-hour threshold, larger than the 1-hour default).
- **Then (a):** Returns `TokenStatus::ExpiringSoon` — the 1-hour remaining time falls within the custom 7200s threshold.
- **Given (b):** A credentials file where `expiresAt` is far in the future (`u64::MAX` ms).
- **When (b):** `token::status_with_threshold(0)` is called (a zero-second threshold).
- **Then (b):** Returns `TokenStatus::Valid` — with `warning_secs == 0`, `ExpiringSoon` only fires at the exact zero-remaining boundary; any positive remaining time is `Valid`.
- **Exit:** Ok
- **Source fn:** `status_with_custom_threshold_classifies_correctly`, `status_with_threshold_zero_classifies_non_expired_as_expiring_soon`
- **Source:** [006_token_status.md AC-03](../../../docs/feature/006_token_status.md)

---

### FT-04: `format::json` returns `"token"`/`"expires_in_secs"` fields

- **Given:** A valid credentials file with a future `expiresAt`.
- **When:** `clp .credentials.status format::json`
- **Then:** Output is valid JSON containing `"token"` and `"expires_in_secs"` keys (among the command's full 16-field object). `"token"` is one of `"valid"`, `"expiring in Xm"`, `"expired"`, `"unknown"`. `"expires_in_secs"` is a non-negative integer.
- **Exit:** 0
- **Source fn:** `ts06_credentials_valid_json`, `ts07_credentials_expired_json`, `ts14_credentials_expiring_soon_json` (in `tests/cli/token_paths_test.rs`) — renamed from `cred_status_json_token_*`
- **Source:** [006_token_status.md AC-04](../../../docs/feature/006_token_status.md)
