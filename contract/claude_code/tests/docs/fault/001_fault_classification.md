# FT — Fault Classification

### Scope

- **Purpose**: Test cases for the `classify_error()` priority algorithm and dual-channel detection signals documented in `docs/fault/readme.md`.
- **Source**: `docs/fault/readme.md` — Error Classification Priority section, Categories A and B.
- **Covers**: FT-01 through FT-07

### Test Case Index

| FT | Category | Scenario |
|----|----------|----------|
| FT-01 | Priority: QuotaExhausted (1st) | `"You've hit your limit"` + reset timestamp in stderr + exit 1 → `QuotaExhausted` |
| FT-02 | Priority: AuthError (2nd) | `"Your organization does not have access"` in stdout + exit 1 → `AuthError` |
| FT-03 | Dual-channel: stderr scan | `"Your organization does not have access"` in **stderr** (not stdout) + exit 1 → `AuthError` |
| FT-04 | Priority: ApiError | `"API Error: 529"` in stderr + exit 1 → `ApiError` (below AuthError/Quota, above Signal) |
| FT-05 | Priority: Signal | exit code 143 (128+15), no text → `Signal` |
| FT-06 | Priority: RateLimit exit-2 | exit code 2, empty stdout + empty stderr → `RateLimit` (F1 detection) |
| FT-07 | Priority: AuthError beats exit-1 ambiguity | `"Your organization does not have access"` in stdout beats bare exit 1 (F4 anti-pattern guard) |

### Notes

- FT-01 through FT-07 scenarios are covered by T-numbered tests in `claude_runner_core/tests/classify_error_test.rs` (T03, T04, T12, T05/T11, T07, T01, T04/T08 respectively). The FT-numbering here is a contract-level grouping; `tests/docs/error/001_classify_error.md` in the runner-core crate uses an independent T-numbering scheme.
- FT-02 and FT-03 together prove dual-channel scanning: FT-02 uses the stdout path (empty stderr), FT-03 uses the stderr path (empty stdout).
- FT-05 and FT-06 verify the lower-priority fallback paths (signal and exit-2) that activate when no text pattern matches.
- FT-07 covers the F4 (Exit 1 Ambiguity) anti-pattern: branching on exit code alone is wrong — text patterns must be checked first.

---

### FT-01: QuotaExhausted pattern with reset timestamp → highest priority classification

- **Given:** `classify_error()` called with stdout=`""`, stderr=`"You've hit your limit · Resets in 3h22m"`, exit_code=`1`
- **When:** classification runs; no AuthError pattern present
- **Then:** returns `ErrorKind::QuotaExhausted`; period-boundary quota exhaustion is priority 1 — highest of all pattern matches

---

### FT-02: AuthError pattern → priority-2 classification

- **Given:** `classify_error()` called with stdout=`"Your organization does not have access to Claude"`, stderr=`""`, exit_code=`1`
- **When:** classification runs against the priority order from `docs/fault/readme.md`
- **Then:** returns `ErrorKind::AuthError`; AuthError is priority 2 in the pattern scan (QuotaExhausted is 1st, AuthError is 2nd, ApiError is 3rd)
- **Note:** Tests that `classify_error()` respects the documented priority order — QuotaExhausted before AuthError before ApiError

---

### FT-03: AuthError in stderr (dual-channel scan) → detected correctly

- **Given:** `classify_error()` called with stdout=`""`, stderr=`"Your organization does not have access to Claude"`, exit_code=`1`
- **When:** classification scans both channels (F2 guard: `claude` may write diagnostics to stderr OR stdout)
- **Then:** returns `ErrorKind::AuthError`; scanning stderr alone is sufficient — mirrors FT-02 via the stderr channel
- **Note:** Dual-channel complement of FT-02 — FT-02 proves the stdout path, FT-03 proves the stderr path; together they confirm both channels are scanned

---

### FT-04: ApiError pattern → classified below auth and quota

- **Given:** `classify_error()` called with stdout=`""`, stderr=`"API Error: 529 overloaded_error"`, exit_code=`1`
- **When:** classification runs; no AuthError or QuotaExhausted pattern present
- **Then:** returns `ErrorKind::ApiError`; priority 3 — below QuotaExhausted/AuthError, above exit-code fallbacks (RateLimit/Signal/Unknown)

---

### FT-05: Signal exit code → classified above Unknown (priority 5)

- **Given:** `classify_error()` called with stdout=`""`, stderr=`""`, exit_code=`143`
- **When:** classification runs; no text patterns match; exit_code > 128
- **Then:** returns `ErrorKind::Signal`; priority 5 — exit_code > 128 with no pattern match; above Unknown (priority 6)
- **Note:** 143 = 128 + 15 (SIGTERM); exit_code > 128 (strict) with no matching text pattern → Signal; exit_code == 128 gives Unknown (boundary is strict); exit_code == 2 gives RateLimit (priority 4, mutually exclusive with > 128)

---

### FT-06: Exit-2 with empty output → RateLimit (F1 silent failure detection)

- **Given:** `classify_error()` called with stdout=`""`, stderr=`""`, exit_code=`2`
- **When:** classification runs; no text patterns match; exit_code is exactly 2 (not > 128)
- **Then:** returns `ErrorKind::RateLimit`; priority 4 (exit-code fallback) — F1 silent failure mode; exit 2 with no diagnostic text is the canonical rate-limit signal from the claude binary
- **Note:** F1 (Rate-Limit Exit 2) silent failure mode — the only signal is the exit code; pattern scan found nothing, signal range not triggered, exit_code == 2 fires

---

### FT-07: AuthError text beats bare exit-1 (F4 anti-pattern guard)

- **Given:** `classify_error()` called with stdout=`"Your organization does not have access to Claude"`, stderr=`""`, exit_code=`1`
- **When:** classification runs; exit code 1 alone would be ambiguous (F4: overloaded across 4 distinct failures)
- **Then:** returns `ErrorKind::AuthError`; text-pattern scan fires before exit-code fallback
- **Note:** F4 anti-pattern guard — branching on exit_code==1 alone would produce `Unknown`; the priority order prevents this
