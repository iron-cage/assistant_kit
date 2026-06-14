# Invariant: Exit Code Contract

`clr` exits with one of five well-defined exit code classes. The mapping from condition to exit code is a contract — callers may rely on it for scripting and automation.

- **ID:** 006
- **Name:** Exit Code Contract
- **Category:** Observable Behavior
- **Enforced By:** `module/claude_runner/src/cli/` (CLR layer), `module/claude_runner_core/src/types.rs` (`ErrorKind` classification)

---

### Exit Code Table

| Exit Code | Class | Condition | Source |
|-----------|-------|-----------|--------|
| 0 | Success | Subprocess exited 0; expect validation passed (if set) | subprocess |
| 1 | Runner Error | Binary not found, spawn OS error, session gate timed out, output file write failed | `execution.rs`, `gate.rs` |
| 2 | Transient / Account / Timeout | Subprocess rate-limited (exit 2, no text); or quota exhausted (exit 2 + text); or CLR timeout watchdog killed subprocess | subprocess, `execution.rs poll_timeout()` |
| 3 | Validation | `--expect` pattern not matched within `--expect-retries` count | `execution.rs apply_expect_validation()` |
| 128+N | Process/Signal | Subprocess killed by signal N (POSIX 128+signal convention) | `claude_runner_core/src/exit_code.rs signal_exit_code()` |

**All other nonzero exit codes** are relayed unchanged from the subprocess and map to `ErrorKind::Unknown`.

---

### Invariant Rules

**Rule 1 — Exit 0 means success:** `clr` exits 0 only when the subprocess exits 0 AND all post-processing steps (expect validation, output file write) succeed.

**Rule 2 — Exit 1 means runner error:** Exit 1 is reserved for CLR-layer failures that occur before or after subprocess execution (binary not found, spawn error, gate timeout, output file write error). Subprocess output is not consulted.

**Rule 3 — Exit 2 is dual-use:** Exit 2 is used for both transient subprocess rate-limiting (no distinguishing stderr) and for the CLR timeout watchdog. Scripts must disambiguate via stderr content: presence of `"Error: timeout after "` identifies a timeout; absence of that prefix with no quota-exhaustion text identifies a rate limit.

**Rule 4 — Exit 3 means expect mismatch:** Exit 3 is exclusively the CLR-layer expect validation failure. It is never produced by the subprocess.

**Rule 5 — Exit > 128 means signal:** Follows POSIX convention. `signal_exit_code(n)` computes `128 + n`. The `claude` subprocess may exit this way when killed externally or when `--timeout` SIGTERM is followed by a non-zero subprocess status. The actual signal number is `exit_code - 128`.

---

### Exit-2 Disambiguation

Exit 2 has three distinct causes distinguishable only at runtime:

| Cause | stderr Contains | Text Contains (stdout/stderr) |
|-------|----------------|-------------------------------|
| `Timeout` (CLR watchdog) | `"Error: timeout after {N}s"` | — |
| `RateLimit` (subprocess) | no timeout prefix | no quota message |
| `QuotaExhausted` (subprocess) | no timeout prefix | `"You've hit your limit"` |

Callers that need to distinguish these should check stderr for `"Error: timeout after "` first, then check for `"You've hit your limit"`.

---

### Notes

- The `clr kill` command is exempt from this table — it reports via exit 0 (success) or exit 1 (not a Claude session / missing PID).
- `clr ps` and `clr --help` / subcommand `--help` always exit 0 on success, 1 on bad arguments.
- Interactive mode (`clr` without `--print`, started as a REPL) relays the subprocess exit code unchanged, but the timeout watchdog and gate remain active.

---

### Sources

| File | Role |
|------|------|
| `module/claude_runner/src/cli/execution.rs` | `poll_timeout()` (exit 2), `write_output_file()` (exit 1), `apply_expect_validation()` (exit 3), `spawn_error_msg()` (exit 1) |
| `module/claude_runner/src/cli/gate.rs` | gate timeout (exit 1) |
| `module/claude_runner_core/src/types.rs` | `ErrorKind` enum and `classify_error()` |
| `module/claude_runner_core/src/exit_code.rs` | `signal_exit_code()` |

---

### Cross-References

- [`docs/cli/type/13_error_kind.md`](../cli/type/13_error_kind.md) — `ErrorKind` enum (subprocess classification)
- [`docs/cli/type/14_error_class.md`](../cli/type/14_error_class.md) — caller-facing error class taxonomy
- `claude_runner_core/docs/failure_mode/004_exit_1_ambiguity.md` — exit-1 disambiguation detail
- [`param/020_timeout.md`](../cli/param/020_timeout.md) — subprocess timeout configuration (kill subcommand)
- [`param/036_timeout.md`](../cli/param/036_timeout.md) — run/ask timeout configuration (CLR watchdog)
- [`param/034_retry_on_rate_limit.md`](../cli/param/034_retry_on_rate_limit.md) — automatic retry on `RateLimit`
