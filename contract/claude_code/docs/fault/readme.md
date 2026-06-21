# Fault Collection

### Scope

- **Purpose**: Consolidated index of all known fault conditions of the `claude` binary, combining terminal errors, silent failure modes, and behavioral quirks into one reference table.
- **Responsibility**: Single lookup point for every documented way `claude` can fail or misbehave. Each row links to the detailed doc instance.
- **In Scope**: Error messages (HTTP 4xx/5xx), silent failures (wrong channel, empty output, env leak), exit code semantics, detection signals, recovery summaries.
- **Out of Scope**: Normal behaviors (-> `../behavior/readme.md`); internal crate error types (-> `module/*/src/`); Anthropic API pricing or subscription terms.

---

### Fault Table

#### Category A — Terminal Errors (explicit message, non-zero exit)

| ID | Fault | HTTP | Exit | Trigger | Severity | Detection Signal | Recovery | Detail |
|----|-------|------|------|---------|----------|------------------|----------|--------|
| E1 | Rate Limit Reached | 429 | 1 or 2 | Per-minute request volume exceeded | Transient | `"You've hit your limit"` in stdout/stderr; or exit 2 with empty output | Wait 30-60s; reduce concurrency | [error/001](../../../../docs/error/001_rate_limit_reached.md) |
| E2 | Authentication Failed | 401 | 1 | Expired OAuth token; wrong API key; org disabled | Fatal | `"authentication_error"` or `"Your organization does not have access"` in stdout/stderr | Re-authenticate (`claude login`); unset `ANTHROPIC_API_KEY` | [error/002](../../../../docs/error/002_authentication_failed.md) |
| E3 | Context Limit Reached | 400 | varies | Conversation exceeds model context window (200k tokens) | Blocking | `"Context limit reached"` in UI; or `"input length and max_tokens exceed context limit"` in API error | `/compact` or `/clear`; reduce `--max-tokens`; break into subtasks | [error/003](../../../../docs/error/003_context_limit_reached.md) |
| E4 | Request Timed Out | — | varies | Server-side generation timeout | Transient | `"Request timed out"` with retry counter (attempt N/10) | Auto-retries 10x with exponential backoff; after 10 failures, Ctrl-C and restart | [error/004](../../../../docs/error/004_request_timed_out.md) |
| E5 | API Overloaded | 529 | 1 | Anthropic API at capacity | Transient | `"API Error: 529"` with `"overloaded_error"` in body | Wait 30-120s; check status.anthropic.com; no auto-retry | [error/005](../../../../docs/error/005_api_overloaded.md) |
| E6 | Quota Exhausted | — | 1 | 5h session or 7d rolling budget fully consumed | Period | `"You've hit your limit"` + reset timestamp | Switch account (`clp .account.use`); wait for reset | [error/006](../../../../docs/error/006_quota_exhausted.md) |

#### Category B — Silent Failure Modes (no obvious signal)

| ID | Fault | Exit | Observable | Danger | Detection Rule | clr Mitigation | Detail |
|----|-------|------|------------|--------|----------------|----------------|--------|
| F1 | Rate-Limit Exit 2 | 2 | Empty stdout + empty stderr | Caller gets no explanation; generic error handler misclassifies | `exit_code == 2` regardless of output | `classify_error()` returns `RateLimit` | [failure_mode/001](../../../../module/claude_runner_core/docs/failure_mode/001_rate_limit_exit_2.md) |
| F2 | Diagnostic on Stdout | 1 | Error text on stdout instead of stderr | stderr-only readers miss all context | Scan both `stdout` and `stderr` for patterns | `classify_error()` scans both channels; `run_print_mode()` forwards stdout to stderr on non-zero exit (BUG-247 ✅) | [failure_mode/002](../../../../module/claude_runner_core/docs/failure_mode/002_diagnostic_on_stdout.md) |
| F3 | CLAUDECODE Env Leak | 0 | No signal — child claude silently changes behavior | Non-deterministic behavior across invocation contexts | Check if `CLAUDECODE` env var is inherited | `ClaudeCommand` defaults `unset_claudecode: true`; warning emitted when `--keep-claudecode` disables protection (BUG-248 ✅) | [failure_mode/003](../../../../module/claude_runner_core/docs/failure_mode/003_claudecode_env_leak.md) |
| F4 | Exit 1 Ambiguity | 1 | Exit 1 means rate-limit OR auth OR API OR unknown | Generic `exit != 0` handler conflates 4 distinct failures | Pattern-priority scan of stdout+stderr before exit code | `classify_error()` with ordered pattern matching | [failure_mode/004](../../../../module/claude_runner_core/docs/failure_mode/004_exit_1_ambiguity.md) |

#### Category C — Behavioral Quirks (not errors, but surprising)

| ID | Behavior | Surprise Factor | Impact | Detail |
|----|----------|-----------------|--------|--------|
| Q1 | Zero-byte `.jsonl` placeholders | Created on startup; remain after crash | Pollute session listing; look like corrupted files | [behavior/B8](../behavior/008_b8_zero_byte_placeholder.md) |
| Q2 | No cross-session links | `--continue` resumes last session; no metadata chain between sessions | Cannot trace conversation lineage across sessions | [behavior/B18](../behavior/018_b18_no_cross_session_links.md) |
| Q3 | `parentUuid` compaction exceptions | < 0.2% of entries have orphaned `parentUuid` after context compaction | Thread-walking code must handle broken chains | [behavior/B17](../behavior/017_b17_parentuuid_self_contained.md) |
| Q4 | Tool definitions in system prompt despite `--tools ""` | ~12k tokens of tool definitions remain even when all tools disabled | Token budget waste; unconfirmed | [behavior/B16h](../behavior/016h_b16h_tools_system_prompt.md) |
| Q5 | Autocompact thrash | Large tool output immediately refills context after compaction | Session becomes unusable; `/clear` required | [error/003](../../../../docs/error/003_context_limit_reached.md) |

---

### Error Classification Priority

When `claude` exits non-zero, detection MUST follow this priority order (higher wins):

```
1. QuotaExhausted  — "You've hit your limit"                             (stdout or stderr)
2. AuthError       — "Your organization does not have access to Claude"  (stdout or stderr)
3. ApiError        — "API Error: "                                       (stdout or stderr)
4. RateLimit       — exit_code == 2, no pattern matched                 (exit code)
5. Signal          — exit_code > 128, no pattern matched                (exit code)
6. Unknown         — any other non-zero exit                            (exit code)
```

**Anti-pattern**: Branching on exit code alone — exit 1 is overloaded across 4 distinct failure modes (see F4).

#### E3 and E4 in print mode

- **E3 (Context Limit) — API overflow form**: When the API rejects the request with HTTP 400, the output contains `"API Error: 400 ..."` and `classify_error()` returns `ApiError` via the existing `"API Error: "` pattern. The in-session UI message (`"Context limit reached · /compact or /clear to continue"`) is interactive-mode-only and never appears in stdout/stderr captured by `clr --print`.

- **E4 (Request Timed Out) — hang scenario**: Claude Code retries internally (up to 10 times) and then becomes unresponsive without exiting. `classify_error()` is never called because the subprocess never exits cleanly. The retry progress message (`"API Error (Request timed out.)"`) does NOT match the `"API Error: "` pattern (parenthesis, not colon-space). If `--timeout` terminates the hanging subprocess, it receives SIGTERM → `Signal`.

---

### Resolved Bugs in Fault Handling

| Bug | Fault | Fix |
|-----|-------|-----|
| BUG-247 ✅ | F2 (Diagnostic on Stdout) | Fixed in task 016: `run_print_mode()` now forwards stdout to stderr when `exit_code != 0` |
| BUG-248 ✅ | F3 (CLAUDECODE Env Leak) | Fixed in task 017: warning emitted when `--keep-claudecode` disables protection with `CLAUDECODE` in env |

---

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| collection | [`../../../../docs/error/readme.md`](../../../../docs/error/readme.md) | Error collection — 6 terminal error instances |
| collection | [`../../../../module/claude_runner_core/docs/failure_mode/readme.md`](../../../../module/claude_runner_core/docs/failure_mode/readme.md) | Failure mode collection — 4 silent failure instances |
| collection | [`../behavior/readme.md`](../behavior/readme.md) | Behavior collection — 27 observed behaviors (B1-B26 + B16h) |
| source | `../../../../module/claude_runner_core/src/types.rs` | `ErrorKind` enum, `classify_error()` implementation |
