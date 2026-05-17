# Test: Fetch Behavior Group

Integration and edge case coverage for the Fetch Behavior parameter group (`refresh::`, `live::`, `interval::`, `jitter::`, `trace::`). See [parameter_groups.md](../../../../docs/cli/parameter_groups.md#group--3-fetch-behavior) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FB-1 | `refresh::1` retries once on 401 auth error | refresh retry |
| FB-2 | `refresh::1` retries once on 403 auth error | refresh retry |
| FB-3 | `refresh::1` does NOT retry on 429 rate-limit error | refresh guard |
| FB-4 | `refresh::0` (default) passes auth errors through as error rows | refresh default |
| FB-5 | `live::1 format::json` exits 1 before any fetch | live incompatibility |
| FB-6 | `interval::` and `jitter::` ignored (not validated) when `live::0` | live guard |
| FB-7 | `live::1 interval::29` exits 1 (interval too low) | interval validation |
| FB-8 | `live::1 jitter::` exceeding `interval::` exits 1 | jitter validation |
| FB-9 | `live::1` and `refresh::1` composable — refresh runs every cycle | composability |
| FB-10 | `trace::1` writes `[trace]` lines to stderr; stdout unchanged | trace output |
| FB-11 | `trace::0` (default) produces no stderr diagnostic output | trace default |
| FB-12 | `trace::1 refresh::1` shows per-account refresh path steps | trace + refresh |

### Test Coverage Summary

- refresh retry: 2 tests
- refresh guard: 1 test
- refresh default: 1 test
- live incompatibility: 1 test
- live guard: 1 test
- interval validation: 1 test
- jitter validation: 1 test
- composability: 1 test
- trace output: 2 tests
- trace + refresh: 1 test

**Total:** 12 tests

---

### FB-1: refresh retry on 401

- **Given:** One saved account whose `fetch_oauth_usage()` returns `Err` containing `"401"`.
- **When:** `clp .usage refresh::1`
- **Then:** Token refresh subprocess is launched for that account; quota fetch is retried once.

---

### FB-2: refresh retry on 403

- **Given:** One saved account whose `fetch_oauth_usage()` returns `Err` containing `"403"`.
- **When:** `clp .usage refresh::1`
- **Then:** Token refresh subprocess is launched for that account; quota fetch is retried once.

---

### FB-3: refresh does NOT retry on 429

- **Given:** One saved account whose `fetch_oauth_usage()` returns `Err` containing `"429"`.
- **When:** `clp .usage refresh::1`
- **Then:** No subprocess is launched. The 429 error appears directly as an error row in the table. No 30-second subprocess timeout is incurred.

---

### FB-4: refresh default passes errors through

- **Given:** One saved account returning an auth error.
- **When:** `clp .usage` (no `refresh::1`)
- **Then:** Error appears in the account's row with no retry. Exit 0.

---

### FB-5: live incompatible with format::json

- **Given:** Any credential state.
- **When:** `clp .usage live::1 format::json`
- **Then:** Exits 1 before any fetch with message `"live monitor mode is incompatible with format::json"`.

---

### FB-6: interval and jitter not validated when live::0

- **Given:** Any credential state.
- **When:** `clp .usage interval::5 jitter::999`
- **Then:** Exits 0. Neither `interval::` nor `jitter::` value is validated. Single fetch proceeds normally.

---

### FB-7: interval too low rejected in live mode

- **Given:** Any credential state.
- **When:** `clp .usage live::1 interval::29`
- **Then:** Exits 1 with error indicating `interval::` must be ≥ 30.

---

### FB-8: jitter exceeds interval rejected in live mode

- **Given:** Any credential state.
- **When:** `clp .usage live::1 interval::30 jitter::31`
- **Then:** Exits 1 with error indicating `jitter::` must not exceed `interval::`.

---

### FB-9: live and refresh composable

- **Given:** `live::1` loop runs multiple cycles; one account has an auth error on cycle 1 but succeeds after refresh.
- **When:** `clp .usage live::1 refresh::1 interval::30`
- **Then:** On cycle 1, auth error triggers refresh + retry for that account. On cycle 2, updated token is used. No conflict between the two parameters.

---

### FB-10: trace writes to stderr, not stdout

- **Given:** One saved account with valid quota data.
- **When:** `clp .usage trace::1`
- **Then:** stderr contains `[trace]` prefixed lines for credential read, API call, and result. stdout contains the normal quota table unchanged.

---

### FB-11: trace default produces no diagnostic output

- **Given:** One saved account with valid quota data.
- **When:** `clp .usage` (no `trace::1`)
- **Then:** stderr is empty. stdout contains the normal quota table.

---

### FB-12: trace shows refresh path steps

- **Given:** One saved account returning a 401 error.
- **When:** `clp .usage refresh::1 trace::1`
- **Then:** stderr contains `[trace]` lines showing: credential read, API call attempt, 401 result, subprocess launch, credential re-read, retry API call, retry result. All steps visible per account.
