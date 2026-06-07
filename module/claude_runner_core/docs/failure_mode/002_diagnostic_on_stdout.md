# Failure Mode: Diagnostic Text Written to Stdout

### Scope

- **Purpose**: Document that `claude` writes error diagnostic text to **stdout** instead of stderr, causing stderr-only readers to miss failure context.
- **Responsibility**: List known stdout-routed patterns, explain dual-channel scan requirement, and document priority ordering.
- **In Scope**: Rate-limit text, auth-error text, API-error text on stdout; dual-channel scan in `classify_error()`.
- **Out of Scope**: Exit-code-only detection (→ `failure_mode/001_rate_limit_exit_2.md`, `failure_mode/004_exit_1_ambiguity.md`).

### Behavior

When `claude` fails with certain conditions, the diagnostic explanation appears on **stdout**, not stderr. A caller that reads only `stderr` — or pipes `2>&1` — will see an empty or misleading error stream and get no actionable information.

Known stdout-routed patterns:

| Pattern | `ErrorKind` | Confirmed by |
|---------|-------------|--------------|
| `"You've hit your limit"` | `QuotaExhausted` | T03 — also appears in stderr |
| `"Your organization does not have access to Claude"` | `AuthError` | T04 (stdout), T12 (stderr) |
| `"API Error: "` | `ApiError` | T05 (stderr), T11 (stderr); auth-before-api priority tested |

All three patterns can appear on **either** channel depending on the error path inside claude. The canonical observation is that T04 fires `AuthError` from stdout alone with an empty stderr — confirming stdout-routed diagnostic.

### Detection Rule

`classify_error()` scans **both** `stdout` and `stderr` for all patterns before falling back to exit-code-based rules:

```
priority order:
  1. QuotaExhausted — "You've hit your limit"
  2. AuthError      — "Your organization does not have access to Claude"
  3. ApiError       — "API Error: "
  4. RateLimit      — exit_code == 2 (no text)
  5. Signal         — exit_code > 128
  6. Unknown        — any other non-zero exit
```

QuotaExhausted precedes AuthError (distinct patterns; both scanned before exit-code rules).
AuthError precedes ApiError to handle 401 responses that include both patterns.

### Anti-Pattern

```rust
// Wrong: stderr-only
if output.stderr.contains("You've hit your limit") { ... }

// Correct: scan both channels
let combined = format!("{}{}", output.stdout, output.stderr);
if combined.contains("You've hit your limit") { ... }
// Or: use output.classify_error() which handles this automatically.
```

### clr Response

`clr` handles detection correctly: `classify_error()` scans both `output.stdout` and `output.stderr` before applying exit-code rules. The classification and labeled diagnostic reach the caller.

**Fixed (BUG-247 ✅, task 016):** `run_print_mode()` now forwards stdout content to stderr when `exit_code != 0`, so diagnostic text on stdout (e.g., `"API Error: 529 overloaded"`) is not silently discarded. Fix: `eprint!("{}", output.stdout)` added on the failure path in `module/claude_runner/src/cli/mod.rs` before `std::process::exit(output.exit_code)`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [failure_mode/001_rate_limit_exit_2.md](001_rate_limit_exit_2.md) | Rate-limit with zero output (exit 2) |
| doc | [failure_mode/004_exit_1_ambiguity.md](004_exit_1_ambiguity.md) | Exit code 1 overloaded across all these error kinds |
| source | `../../src/types.rs` | `classify_error()` — dual-channel scan and priority ordering |
| source | `../../../claude_runner/src/cli/mod.rs` | `run_print_mode()` — stdout forward gap (BUG-247) |
| test | `../../tests/classify_error_test.rs` | T03 (QuotaExhausted in stderr), T04 (AuthError in stdout), T05 (ApiError in stderr), T12 (AuthError in stderr), priority test |
| bug | BUG-247 | `run_print_mode()` discards stdout content when exit_code != 0 |

### Sources

| File | Notes |
|------|-------|
| `tests/classify_error_test.rs` | T04 is the definitive evidence: auth pattern in stdout only, empty stderr, yields `AuthError` |
| BUG-037 | Root cause: single-channel scan missed stdout-routed diagnostics |
| BUG-247 | Gap: stdout content swallowed in `run_print_mode()` on non-zero exit |
