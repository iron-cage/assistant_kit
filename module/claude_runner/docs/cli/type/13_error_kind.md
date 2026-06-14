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
| Timeout — subprocess exceeded `--timeout` | 2 ⚠️ | `"Error: timeout after {N}s"` | `execution.rs poll_timeout()` |
| Expect mismatch — output did not match `--expect` | 3 | `"Error: output did not match --expect"` | `execution.rs apply_expect_validation()` |
| Binary not found — `claude` not in PATH | 1 | `"claude binary not found in PATH"` | `execution.rs spawn_error_msg()` |
| Spawn failed — OS error creating subprocess | 1 | `"Failed to execute Claude Code: {e}"` | `execution.rs spawn_error_msg()` |
| Gate timeout — waited too long for session slot | 1 | `"Error: session gate timed out"` | `gate.rs wait_for_session_slot()` |
| Output file write failed | 1 | `"Error: failed to write output file"` | `execution.rs write_output_file()` |

⚠️ **Exit-2 collision:** `Timeout` (CLR-layer) and `RateLimit` (subprocess) both produce exit 2.
Distinguish them by stderr: a timeout always prints `"Error: timeout after {N}s"` on stderr;
a rate-limit exit has no such stderr prefix. See [`invariant/006_exit_codes.md`](../../invariant/006_exit_codes.md).

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
| 34 | [`--retry-on-rate-limit`](../param/034_retry_on_rate_limit.md) | triggers retry when `ErrorKind::RateLimit` |
| 35 | [`--retry-delay`](../param/035_retry_delay.md) | delay between `RateLimit` retries |

### Cross-References

- [`invariant/006_exit_codes.md`](../../invariant/006_exit_codes.md) — complete exit code table and collision disambiguation
- [`docs/cli/type/14_error_class.md`](14_error_class.md) — caller-facing error class taxonomy
- `claude_runner_core/docs/failure_mode/` — silent failure modes related to classification
