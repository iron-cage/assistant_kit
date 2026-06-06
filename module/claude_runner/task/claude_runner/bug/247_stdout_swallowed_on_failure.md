# BUG-247 — `run_print_mode()` Discards Stdout Content When exit_code != 0

## Execution State

- **State:** Verified
- **Fixed:** 2026-06-07

## Symptom

When `claude` exits with a non-zero code AND writes diagnostic text to stdout (e.g., `"API Error: 529 overloaded"`), `clr` discards that stdout content entirely. The caller receives only the classified error label on stderr but loses the actual diagnostic message.

```bash
# Mock claude that exits 1 with diagnostic on stdout
PATH=/tmp/mre247:$PATH clr --no-persist "test" 2>&1
# Expected (after fix):
#   API Error: 529 overloaded       ← forwarded from stdout to stderr
#   Error: API error (exit 1)       ← classify_error() label
# Actual (bug):
#   Error: API error (exit 1)       ← only the label; stdout diagnostic swallowed
```

## Impact

- **Who**: Any operator using `clr` in scripted or automated contexts where the failure diagnostic matters for debugging or retry logic.
- **Conditions**: `claude` exits non-zero AND has diagnostic text on stdout (confirmed possible by `failure_mode/002_diagnostic_on_stdout.md`).
- **Severity**: Medium — partial degradation. Error is detected and labeled correctly; only the raw diagnostic payload from stdout is lost.
- **Silent**: Yes — no indication that content was discarded. Caller has no way to know stdout had additional context.

## How Discovered

Code analysis of `run_print_mode()` in `module/claude_runner/src/cli/mod.rs`. The stdout forward (`print!("{out}")`) at line 311 only executes when `exit_code == 0`. The failure path exits at line 308 (`std::process::exit(output.exit_code)`) without forwarding `output.stdout`. There is no branch for non-zero exit with stdout content.

## MRE

```bash
mkdir -p /tmp/mre247
cat > /tmp/mre247/claude << 'EOF'
#!/bin/bash
echo "API Error: 529 overloaded"
exit 1
EOF
chmod +x /tmp/mre247/claude

# Run clr — the diagnostic on stdout should appear on stderr after fix
PATH=/tmp/mre247:$PATH clr --no-persist "test" 2>stderr_actual.txt
echo "exit: $?"
echo "--- stderr ---"
cat stderr_actual.txt
echo "--- expected ---"
echo "API Error: 529 overloaded"
echo "Error: API error (exit 1)"
```

Validation: `grep -q "API Error: 529 overloaded" stderr_actual.txt` must succeed. Currently it fails — stdout diagnostic is absent.

## Root Cause

### Root Cause

`run_print_mode()` (`module/claude_runner/src/cli/mod.rs`) contains two forwarding paths:

1. `if !output.stderr.is_empty() { eprint!("{}", output.stderr); }` — always executes
2. `print!("{out}")` — only executes after the `if output.exit_code != 0 { std::process::exit(...); }` guard

There is no path that forwards `output.stdout` when `exit_code != 0`. The diagnostic text that `claude` wrote to stdout is stored in `output.stdout` but never printed.

### Why Not Caught

The `classify_error()` tests (`tests/classify_error_test.rs`) test pattern detection on `ExecutionOutput` directly — they do not exercise `run_print_mode()`'s forwarding behavior. No integration test covers the "non-zero exit with non-empty stdout" path in `run_print_mode()`. The gap was invisible because classification (detecting the error kind) and display (forwarding the raw content) are separate concerns — and only the classification side was tested.

### Fix Location

`module/claude_runner/src/cli/mod.rs` — `run_print_mode()` — after the stderr forward, before the `classify_error()` block, add:

```rust
// Fix(BUG-247): forward stdout to stderr when exit_code != 0 so diagnostic
//   text written by claude to stdout reaches the caller.
// Root cause: the success-path stdout forward (`print!("{out}")`) never executes
//   on failure; stdout content is silently discarded after the non-zero exit branch.
// Pitfall: mirrors the unconditional stderr forward above — no verbosity gate.
//   Verbosity controls runner diagnostics; raw subprocess output is always forwarded.
if output.exit_code != 0 && !output.stdout.is_empty()
{
  eprint!( "{}", output.stdout );
}
```

### Prevention

Any function that captures subprocess output must have integration tests verifying that BOTH stdout and stderr are forwarded on failure. The test matrix must include a "non-zero exit with stdout content" row, not only "non-zero exit with empty stdout."

### Generalized Version

**Pattern: Separate capture paths for success and failure leave one channel unforwarded on failure.**

When a function captures subprocess output and prints it only on the success path, non-zero exit silently discards one or both channels. The canonical form: stderr-only forward on failure is correct for runner diagnostics; stdout-only forward on success is correct for clean output delivery — but the combination discards stdout content when the subprocess fails with diagnostic text on stdout.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|------------|-------|---------|----------|
| H1 | `run_print_mode()` has no stdout-forward path for non-zero exit | ✅ Root Cause | Line 311 `print!` only reachable when exit_code == 0; line 308 exits before it | `src/cli/mod.rs:306-312` |
| H2 | classify_error() suppresses stdout forward | DISPROVED | classify_error() only matches patterns; does not affect output forwarding | `src/cli/mod.rs:285-299` |

## Evidence Table

| Location | What It Shows |
|----------|---------------|
| `module/claude_runner/src/cli/mod.rs:277` | stderr forward: `if !output.stderr.is_empty() { eprint!(...) }` — always runs |
| `module/claude_runner/src/cli/mod.rs:306-308` | `if output.exit_code != 0 { std::process::exit(output.exit_code); }` — exits before stdout |
| `module/claude_runner/src/cli/mod.rs:311` | `print!("{out}")` — only reachable when exit_code == 0 |
| `claude_runner_core/docs/failure_mode/002_diagnostic_on_stdout.md` | Confirms claude writes diagnostic text to stdout on certain failure paths |

## History

| Date | Event | Note |
|------|-------|------|
| 2026-06-07 | filed | Source: code analysis of `run_print_mode()` — stdout forward missing on failure path |
| 2026-06-07 | fixed | `src/cli/mod.rs:435` — `Fix(BUG-247)`: eprint stdout to stderr before exit on non-zero exit code |
| 2026-06-07 | verified | `bug_reproducers_247_test.rs` passes; automated tests confirm stdout forwarded on failure |
