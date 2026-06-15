# CLI Type: ErrorKind

Classification of `claude` subprocess failure modes returned by `classify_error()`.
Callers use this to decide whether to retry, switch credentials, abort, or surface diagnostics.

- **Purpose:** Subprocess exit failure classification returned by `claude_runner_core`
- **Fundamental Type:** enumeration (6 variants)
- **Constants:** see below
- **Constraints:** returned only on non-zero exit; `None` on exit 0
- **Parsing:** not a CLI input type — produced by `ExecutionOutput::classify_error()` in `claude_runner_core`
- **Methods:** `classify_error() -> Option<ErrorKind>` on `ExecutionOutput`

### Variants

| Variant | Exit Trigger | Detection Method | Caller Response |
|---------|-------------|------------------|-----------------|
| `RateLimit` | exit 2 (no text match) | exit code fallback after pattern scan | retry with backoff |
| `QuotaExhausted` | exit 2 + text match | `"You've hit your limit"` in stdout/stderr | wait for period reset or switch account |
| `AuthError` | any nonzero + text match | `"Your organization does not have access to Claude"` in stdout/stderr | fix credentials |
| `ApiError` | any nonzero + text match | `"API Error: "` in stdout/stderr | surface to user; may be transient |
| `Signal` | exit code > 128 | exit code > 128 (POSIX 128+N convention) | report signal number; check for external killer |
| `Unknown` | other nonzero | no pattern match, exit ≤ 128, exit ≠ 2 | surface raw output to user |

### Classification Priority

`classify_error()` applies rules in this exact order — first match wins:

1. Scan stdout + stderr for `"You've hit your limit"` → `QuotaExhausted`
2. Scan stdout + stderr for `"Your organization does not have access to Claude"` → `AuthError`
3. Scan stdout + stderr for `"API Error: "` → `ApiError`
4. Exit code == 2 → `RateLimit`
5. Exit code > 128 → `Signal`
6. Default → `Unknown`

`QuotaExhausted` is checked before `AuthError` and before the exit-2 sentinel so period-exhaustion messages
(which also produce exit 2) are classified as `QuotaExhausted`, not `RateLimit`.
`AuthError` is checked before `ApiError` because a 401 response may contain both `"Your organization..."` and `"API Error: "`.

### CLR-Layer Ad-Hoc Error Codes

The following failure conditions are emitted by the CLR runner layer (`module/claude_runner/src/cli/`) before a subprocess runs or after subprocess death. They do not go through `classify_error()` and have no `ErrorKind` variant.

| Condition | Exit Code | stderr Label | Source |
|-----------|-----------|--------------|--------|
| Timeout — subprocess exceeded `--timeout` | 4 | `"Error: timeout after {N}s"` | `execution.rs poll_timeout()` |
| Expect mismatch — output did not match `--expect` | 3 | `"Error: [Validation] expected \"<pat>\", got \"<val>\" (exit 3)"` | `execution.rs apply_expect_validation()` |
| Binary not found — `claude` not in PATH | 1 | `"claude binary not found in PATH"` | `execution.rs spawn_error_msg()` |
| Spawn failed — OS error creating subprocess | 1 | `"Failed to execute Claude Code: {e}"` | `execution.rs spawn_error_msg()` |
| Gate timeout — waited too long for session slot | 1 | `"Error: session gate timed out"` | `gate.rs wait_for_session_slot()` |
| Output file write failed | 1 | `"Error: failed to write output file"` | `execution.rs write_output_file()` |

### Notes on Specific Patterns

- **E3 (Context Limit):** The API-overflow form begins `"API Error: 400 ..."` and classifies as `ApiError`. The interactive-only message `"Context limit reached"` never appears in print-mode stdout/stderr captured by `classify_error()`.
- **E4 (Request Timeout retry):** The retry-progress line uses `"API Error (Request timed out.)"` — note parenthesis, not colon-space — which does NOT match `"API Error: "`. The subprocess hangs after retries without exiting; when `--timeout` kills it, the result is `Signal`.

### Referenced Commands

| # | Command | Produced By |
|---|---------|-------------|
| 1 | [`run`](../command/01_run.md) | subprocess exit classification |
| 5 | [`ask`](../command/05_ask.md) | subprocess exit classification |

### Referenced Parameters

| # | Parameter | Interaction |
|---|-----------|-------------|
| 34 | [`--retry-on-transient`](../param/034_retry_on_transient.md) | triggers retry when `ErrorKind::RateLimit` (Transient class) |
| 35 | [`--transient-delay`](../param/035_transient_delay.md) | delay between Transient retries |
| 40 | [`--retry-on-account`](../param/040_retry_on_account.md) | triggers retry when `ErrorKind::QuotaExhausted` (Account class) |
| 42 | [`--retry-on-auth`](../param/042_retry_on_auth.md) | triggers retry when `ErrorKind::AuthError` (Auth class) |
| 44 | [`--retry-on-service`](../param/044_retry_on_service.md) | triggers retry when `ErrorKind::ApiError` (Service class) |
| 45 | [`--service-delay`](../param/045_service_delay.md) | delay between Service retries |
| 46 | [`--retry-on-process`](../param/046_retry_on_process.md) | triggers retry when `ErrorKind::Signal` (Process class) |
| 52 | [`--retry-on-unknown`](../param/052_retry_on_unknown.md) | triggers retry when `ErrorKind::Unknown` (Unknown class) |
| 54 | [`--retry-override`](../param/054_retry_override.md) | Tier 1: overrides retry count for all error classes |
| 56 | [`--retry-default`](../param/056_retry_default.md) | Tier 3: fallback retry count (default 2) |
| 57 | [`--retry-default-delay`](../param/057_retry_default_delay.md) | Tier 3: fallback delay (default 30s) |

### Cross-References

- [`invariant/006_exit_codes.md`](../../invariant/006_exit_codes.md) — complete exit code table and collision disambiguation
- [`docs/cli/type/14_error_class.md`](14_error_class.md) — caller-facing error class taxonomy
- `claude_runner_core/docs/failure_mode/` — silent failure modes related to classification
