# Test Spec: classify_error

### Scope

- **Purpose**: Verify `ExecutionOutput::classify_error()` returns the correct `ErrorKind` for each subprocess failure mode.
- **Source**: `docs/error/001_rate_limit_reached.md` (transient HTTP 429), `docs/error/006_quota_exhausted.md` (period quota depletion).
- **Implementation**: `claude_runner_core/src/types.rs` — `ErrorKind` enum, `ERROR_PATTERNS`, `classify_error()`.
- **Test File**: `tests/classify_error_test.rs`

### Test Cases

#### FT-01: Exit code 2, empty output → RateLimit

- **Given**: `ExecutionOutput { exit_code: 2, stdout: "", stderr: "" }`
- **When**: `classify_error()` is called
- **Then**: Returns `Some(ErrorKind::RateLimit)` — exit code 2 is the canonical rate-limit sentinel

#### FT-02: Exit code 0 → None (success)

- **Given**: `ExecutionOutput { exit_code: 0, stdout: "", stderr: "" }`
- **When**: `classify_error()` is called
- **Then**: Returns `None` — success short-circuits before any pattern scanning

#### FT-03: "You've hit your limit" in stderr → QuotaExhausted

- **Given**: `ExecutionOutput { exit_code: 1, stdout: "", stderr: "You've hit your limit" }`
- **When**: `classify_error()` is called
- **Then**: Returns `Some(ErrorKind::QuotaExhausted)` — period quota exhaustion, not transient rate limit

#### FT-04: Auth pattern in stdout → AuthError

- **Given**: `ExecutionOutput { exit_code: 1, stdout: "Your organization does not have access to Claude", stderr: "" }`
- **When**: `classify_error()` is called
- **Then**: Returns `Some(ErrorKind::AuthError)` — stdout scan path verified

#### FT-05: API Error in stderr → ApiError

- **Given**: `ExecutionOutput { exit_code: 1, stdout: "", stderr: "API Error: 529 overloaded" }`
- **When**: `classify_error()` is called
- **Then**: Returns `Some(ErrorKind::ApiError)`

#### FT-06: Exit code 130 (SIGINT) → Signal

- **Given**: `ExecutionOutput { exit_code: 130, stdout: "", stderr: "" }`
- **When**: `classify_error()` is called
- **Then**: Returns `Some(ErrorKind::Signal)` — 128 + 2 = SIGINT

#### FT-07: Exit code 143 (SIGTERM) → Signal

- **Given**: `ExecutionOutput { exit_code: 143, stdout: "", stderr: "" }`
- **When**: `classify_error()` is called
- **Then**: Returns `Some(ErrorKind::Signal)` — 128 + 15 = SIGTERM

#### FT-08: Exit code 1, empty output → Unknown

- **Given**: `ExecutionOutput { exit_code: 1, stdout: "", stderr: "" }`
- **When**: `classify_error()` is called
- **Then**: Returns `Some(ErrorKind::Unknown)` — no pattern match, no signal code

#### FT-09: API Error not misclassified as Unknown

- **Given**: `ExecutionOutput { exit_code: 1, stdout: "", stderr: "API Error: 500 internal server error" }`
- **When**: `classify_error()` is called
- **Then**: Returns `Some(ErrorKind::ApiError)` — pattern match takes priority over exit-code fallback

#### FT-10: Auth pattern in stderr (not stdout)

- **Given**: `ExecutionOutput { exit_code: 1, stdout: "", stderr: "Your organization does not have access to Claude" }`
- **When**: `classify_error()` is called
- **Then**: Returns `Some(ErrorKind::AuthError)` — stderr scan path verified

#### FT-11: Auth pattern takes priority over ApiError

- **Given**: `ExecutionOutput { exit_code: 1, stdout: "", stderr: "Your organization does not have access to Claude\nAPI Error: 401 unauthorized" }`
- **When**: `classify_error()` is called
- **Then**: Returns `Some(ErrorKind::AuthError)` — auth pattern is scanned before API pattern; priority order matters for 401 responses

#### FT-12: "You've hit your limit" in stdout → QuotaExhausted

- **Given**: `ExecutionOutput { exit_code: 1, stdout: "You've hit your limit", stderr: "" }`
- **When**: `classify_error()` is called
- **Then**: Returns `Some(ErrorKind::QuotaExhausted)` — stdout scan path for quota exhaustion

#### FT-13: Exit code 0 with quota pattern → None

- **Given**: `ExecutionOutput { exit_code: 0, stdout: "You've hit your limit", stderr: "" }`
- **When**: `classify_error()` is called
- **Then**: Returns `None` — success short-circuit: exit code 0 overrides any pattern content

#### FT-14: Exit code 2 + "You've hit your limit" → QuotaExhausted (pattern wins over exit code)

- **Given**: `ExecutionOutput { exit_code: 2, stdout: "", stderr: "You've hit your limit" }`
- **When**: `classify_error()` is called
- **Then**: Returns `Some(ErrorKind::QuotaExhausted)` — pattern match fires before exit code 2 fallback; quota pattern always wins regardless of exit code

#### FT-15: All 6 variants round-trip via Debug + Clone + PartialEq + Eq

- **Given**: All `ErrorKind` variants: `RateLimit`, `QuotaExhausted`, `ApiError`, `AuthError`, `Signal`, `Unknown`
- **When**: Each is cloned and compared
- **Then**: `v == v.clone()` for all variants; `Debug` output contains variant name

#### FT-16: Exit code 128 → Unknown (boundary: strict `> 128`, not `>=`)

- **Given**: `ExecutionOutput { exit_code: 128, stdout: "", stderr: "" }`
- **When**: `classify_error()` is called
- **Then**: Returns `Some(ErrorKind::Unknown)` — 128 does NOT satisfy `> 128`; the signal range begins at 129

#### FT-17: Exit code 129 (SIGHUP) → Signal (first code that satisfies `> 128`)

- **Given**: `ExecutionOutput { exit_code: 129, stdout: "", stderr: "" }`
- **When**: `classify_error()` is called
- **Then**: Returns `Some(ErrorKind::Signal)` — 128+1 satisfies `> 128`; pairs with FT-16 to pin both sides of the boundary

#### FT-18: Uppercase quota pattern → Unknown (pattern match is case-sensitive)

- **Given**: `ExecutionOutput { exit_code: 1, stdout: "", stderr: "YOU'VE HIT YOUR LIMIT" }`
- **When**: `classify_error()` is called
- **Then**: Returns `Some(ErrorKind::Unknown)` — `str::contains` is case-sensitive; uppercase variant does not match `"You've hit your limit"`

#### FT-19: `authentication_error` 401 → AuthError (not ApiError)

- **Given**: `ExecutionOutput { exit_code: 1, stdout: "", stderr: "Failed to authenticate. API Error: 401 {\"type\":\"authentication_error\",\"message\":\"Invalid authentication credentials\"}" }`
- **When**: `classify_error()` is called
- **Then**: Returns `Some(ErrorKind::AuthError)` — the `"authentication_error"` pattern fires before the `"API Error: "` catch-all; without this fix the same string would be misclassified as `ApiError`

### Non-Coverage Notes

The following scenarios are intentionally not tested because they are not observable through `classify_error()` in print mode:

**E3 (Context Limit) — in-session UI form**: `"Context limit reached · /compact or /clear to continue"` is displayed by the interactive TUI and never written to stdout/stderr in `--print` mode. The API-level overflow form (`"API Error: 400 ..."`) is already covered by FT-05/FT-09/FT-11 via the `"API Error: "` pattern → `ApiError`.

**E4 (Request Timed Out) — hang scenario**: After 10 retries the subprocess becomes unresponsive without exiting. `classify_error()` is never called because no exit code is produced. The retry message `"API Error (Request timed out.)"` uses parenthesis, not `"API Error: "` (colon-space), so it would not match the `ApiError` pattern if the subprocess did exit — but since it hangs, this is moot. When `--timeout` kills the subprocess it gets SIGTERM → already covered by FT-06/FT-07 (`Signal`).
