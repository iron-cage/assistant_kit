# Invariant: Exit Code Contract

`clr` exits with one of six well-defined exit code classes. The mapping from condition to exit code is a contract — callers may rely on it for scripting and automation.

- **ID:** 006
- **Name:** Exit Code Contract
- **Category:** Observable Behavior
- **Enforced By:** `module/claude_runner/src/cli/` (CLR layer), `module/claude_runner_core/src/types.rs` (`ErrorKind` classification)

---

### Scope

- **Purpose**: Define the six exit code classes that `clr` produces and guarantee their stability as a caller contract.
- **Responsibility**: State the exit code mapping, per-class rules, exit-2 disambiguation requirement, and enforcement sources.
- **In Scope**: All exit codes produced by `clr run`, `clr ask`, `clr isolated`, `clr refresh`; the `clr kill` and `clr ps` exceptions noted in Notes.
- **Out of Scope**: `claude` binary exit codes not remapped by `clr` (relayed as-is → `ErrorKind::Unknown`); process signal semantics beyond the 128+N convention.

### Exit Code Table

| Exit Code | Class | Condition | Source |
|-----------|-------|-----------|--------|
| 0 | Success | Subprocess exited 0; expect validation passed (if set) | subprocess |
| 1 | Runner Error | Binary not found, spawn OS error, session gate timed out, output file write failed | `execution.rs`, `gate.rs` |
| 2 | Transient / Account | Subprocess rate-limited (exit 2, no text); or quota exhausted (exit 2 + text) | subprocess |
| 3 | Validation | `--expect` pattern not matched within `--retry-on-validation` count | `execution.rs apply_expect_validation()` |
| 4 | Timeout | CLR timeout watchdog killed subprocess after `--timeout` seconds | `execution.rs poll_timeout()` |
| 128+N | Process/Signal | Subprocess killed by signal N (POSIX 128+signal convention) | `claude_runner_core/src/exit_code.rs signal_exit_code()` |

**All other nonzero exit codes** are relayed unchanged from the subprocess and map to `ErrorKind::Unknown`.

---

### Invariant Statement

`clr` MUST exit with one of the six well-defined exit codes from the table above. The mapping from condition to exit code is a caller contract — callers may unconditionally rely on it for scripting and automation. Each exit code class is exclusive: any given execution produces exactly one outcome class.

**Rule 1 — Exit 0 means success:** `clr` exits 0 only when the subprocess exits 0 AND all post-processing steps (expect validation, output file write) succeed.

**Rule 2 — Exit 1 means runner error:** Exit 1 is reserved for CLR-layer failures that occur before or after subprocess execution (binary not found, spawn error, gate timeout, output file write error). Subprocess output is not consulted.

**Rule 3 — Exit 2 is subprocess-only:** Exit 2 is reserved for subprocess rate-limiting and quota exhaustion. Scripts disambiguate via output text: presence of `"You've hit your limit"` identifies quota exhaustion (Account class); absence of that text identifies a transient rate limit (Transient class).

**Rule 4 — Exit 3 means expect mismatch:** Exit 3 is exclusively the CLR-layer expect validation failure. It is never produced by the subprocess.

**Rule 5 — Exit 4 means timeout:** Exit 4 is exclusively the CLR timeout watchdog. Stderr always contains `"Error: timeout after {N}s"`. It is never produced by the subprocess. Increase `--timeout` or use `--timeout 0` (unlimited) to prevent.

**Rule 6 — Exit > 128 means signal:** Follows POSIX convention. `signal_exit_code(n)` computes `128 + n`. The `claude` subprocess may exit this way when killed externally or when `--timeout` SIGTERM is followed by a non-zero subprocess status. The actual signal number is `exit_code - 128`.

---

### Exit-2 Disambiguation

Exit 2 has two distinct causes distinguishable by output text:

| Cause | Text Contains (stdout/stderr) |
|-------|-------------------------------|
| `RateLimit` (subprocess) | no quota message |
| `QuotaExhausted` (subprocess) | `"You've hit your limit"` |

Callers check for `"You've hit your limit"` in stdout/stderr to distinguish quota exhaustion from transient rate-limiting. Timeout no longer uses exit 2 — it uses exit 4 (see Rule 5).

---

### Enforcement Mechanism

Four source components collaborate to enforce this contract:

1. **`cli/execution.rs`** — `poll_timeout()` issues exit 4; `write_output_file()` issues exit 1 on write error; `apply_expect_validation()` issues exit 3 on expect mismatch; `spawn_error_msg()` issues exit 1 on spawn failure.
2. **`cli/gate.rs`** — Session gate timeout issues exit 1 via the gate handler.
3. **`claude_runner_core/src/types.rs`** — `ErrorKind` enum and `classify_error()` map subprocess output to error classes (`RateLimit` → exit 2, `QuotaExhausted` → exit 2); unknown exit codes relay unchanged.
4. **`claude_runner_core/src/exit_code.rs`** — `signal_exit_code(n)` computes `128 + n` for signal-killed subprocesses.

Any new subprocess-executing command added to `clr` MUST route its error outcomes through the same `ErrorKind` classification and exit code mapping.

---

### Violation Consequences

If the exit code contract is violated (a wrong code is produced for a given condition):

- **Caller scripts misroute silently**: Scripts keying on exit 2 for rate-limiting, exit 3 for expect failure, or exit 4 for timeout route to the wrong handling branch with no error signal.
- **Internal retry logic breaks**: `clr` classifies retry eligibility by exit code and error class. A wrong exit code causes the wrong retry budget to be consumed or no retry to fire.
- **CI/CD pipelines produce incorrect status**: Pipelines using `clr` exit codes as status signals execute the wrong handler.
- **The `--expect` feedback loop breaks**: Exit 3 from `apply_expect_validation()` drives `--retry-on-validation` retry cycling; if exit 3 is remapped, validation retries stop.

---

### Notes

- The `clr kill` command is exempt from this table — it reports via exit 0 (success) or exit 1 (not a Claude session / missing PID).
- `clr ps` and `clr --help` / subcommand `--help` always exit 0 on success, 1 on bad arguments.
- Interactive mode (`clr` without `--print`, started as a REPL) relays the subprocess exit code unchanged, but the timeout watchdog and gate remain active.

---

### Sources

| File | Role |
|------|------|
| `module/claude_runner/src/cli/execution.rs` | `poll_timeout()` (exit 4), `write_output_file()` (exit 1), `apply_expect_validation()` (exit 3), `spawn_error_msg()` (exit 1) |
| `module/claude_runner/src/cli/gate.rs` | gate timeout (exit 1) |
| `module/claude_runner_core/src/types.rs` | `ErrorKind` enum and `classify_error()` |
| `module/claude_runner_core/src/exit_code.rs` | `signal_exit_code()` |

---

### Types

| File | Relationship |
|------|--------------|
| [../cli/type/13_error_kind.md](../cli/type/13_error_kind.md) | `ErrorKind` enum — subprocess classification |
| [../cli/type/14_error_class.md](../cli/type/14_error_class.md) | Caller-facing error class taxonomy |

### Parameters

| File | Relationship |
|------|--------------|
| [../cli/param/020_timeout.md](../cli/param/020_timeout.md) | Subprocess timeout configuration (kill subcommand) |
| [../cli/param/034_retry_on_transient.md](../cli/param/034_retry_on_transient.md) | Automatic retry on `RateLimit` (Transient class) |
| [../cli/param/036_timeout.md](../cli/param/036_timeout.md) | Run/ask timeout configuration (CLR watchdog) |
| [../cli/param/044_retry_on_service.md](../cli/param/044_retry_on_service.md) | Automatic retry on `ApiError` (Service class) |
| [../cli/param/052_retry_on_unknown.md](../cli/param/052_retry_on_unknown.md) | Automatic retry on `Unknown` (Unknown class) |

### Features

| File | Relationship |
|------|--------------|
| [../feature/003_retry_hierarchy.md](../feature/003_retry_hierarchy.md) | 3-tier retry system — exit codes drive error class classification and retry eligibility |
| `claude_runner_core/docs/failure_mode/004_exit_1_ambiguity.md` | Exit-1 disambiguation detail |
