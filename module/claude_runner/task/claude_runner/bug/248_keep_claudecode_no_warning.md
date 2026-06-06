# BUG-248 — No Warning When `--keep-claudecode` Disables CLAUDECODE Protection

## Execution State

- **State:** Verified
- **Fixed:** 2026-06-07

## Symptom

When `CLAUDECODE=1` is set in the environment and the user passes `--keep-claudecode`, `clr` silently allows the child `claude` to inherit `CLAUDECODE=1`. No warning is emitted. The protection disable is invisible.

```bash
export CLAUDECODE=1
clr --keep-claudecode --dry-run "test" 2>&1
# Expected (after fix):
#   Warning: --keep-claudecode is set and CLAUDECODE is present in environment; child claude will run in nested-agent mode
#   env ... (dry-run output)
# Actual (bug):
#   claude ... (dry-run output, no warning)
```

## Impact

- **Who**: Operators or scripts that pass `--keep-claudecode` without realizing `CLAUDECODE` is actually set in the invoking environment (e.g., running `clr` from within a Claude Code session).
- **Conditions**: `CLAUDECODE` is present in the parent environment AND `--keep-claudecode` is explicitly passed.
- **Severity**: Minor — the protection disable is intentional (user asked for it). The gap is that no feedback confirms the consequence.
- **Silent**: Yes — the child process silently operates in nested-agent mode; no log, no warning, no exit-code change.

## How Discovered

Code analysis of `run_built_command()` in `module/claude_runner/src/cli/mod.rs` and `build_claude_command()` in `module/claude_runner/src/cli/builder.rs`. When `cli.keep_claudecode == true`, `builder.with_unset_claudecode(false)` is called silently. Neither `build_claude_command()` nor `run_built_command()` checks whether `CLAUDECODE` is actually set in the environment before allowing the inheritance.

## MRE

```bash
# Step 1: confirm CLAUDECODE is set
export CLAUDECODE=1
echo "CLAUDECODE=${CLAUDECODE}"

# Step 2: run clr with --keep-claudecode; check for warning
clr --keep-claudecode --dry-run "test" 2>stderr_actual.txt
echo "exit: $?"
echo "--- stderr ---"
cat stderr_actual.txt
echo "--- expected ---"
echo "Warning: --keep-claudecode is set and CLAUDECODE is present in environment; child claude will run in nested-agent mode"
```

Validation: `grep -q "Warning.*keep-claudecode.*CLAUDECODE" stderr_actual.txt` must succeed. Currently it fails — no warning is emitted.

## Root Cause

### Root Cause

`run_built_command()` (`module/claude_runner/src/cli/mod.rs`) dispatches to print or interactive mode after computing verbosity and optionally emitting a trace preview. There is no check between verbosity computation and dispatch that inspects:
1. Whether `cli.keep_claudecode` is true
2. Whether `CLAUDECODE` is currently set in the environment

Without this check, the protection disable from `--keep-claudecode` is unconditionally silent. The `--keep-claudecode` flag was designed for rare, intentional use — but its consequence (nested-agent mode for the child) is non-obvious and warrants a warning.

### Why Not Caught

`--keep-claudecode` was implemented (BUG-246 context) to suppress the `env -u CLAUDECODE` prefix in dry-run output when the user explicitly wants to keep `CLAUDECODE` in the child environment. The warning requirement was not identified during that implementation. No test exercises `--keep-claudecode` with `CLAUDECODE` set in the environment — that combination was not in the test matrix.

### Fix Location

`module/claude_runner/src/cli/mod.rs` — `run_built_command()` — after computing `verbosity`, add:

```rust
// Fix(BUG-248): warn when --keep-claudecode disables CLAUDECODE protection
//   and CLAUDECODE is actually set in the parent environment.
// Root cause: no warning was implemented when protection is disabled.
// Pitfall: gate on shows_warnings() (level >= 2) so --verbosity 0/1 users
//   who want silence still get silence; the warning is informational not fatal.
if cli.keep_claudecode
  && verbosity.shows_warnings()
  && std::env::var( "CLAUDECODE" ).is_ok()
{
  eprintln!(
    "Warning: --keep-claudecode is set and CLAUDECODE is present in environment; \
     child claude will run in nested-agent mode"
  );
}
```

### Prevention

Any flag that disables a safety feature should have a corresponding warning when the safety feature would have been active. Test matrix for protection-disabling flags must include a row with the protected condition present — not only the flag in isolation.

### Generalized Version

**Pattern: Protection-disable flags emit no feedback when the protection would have been active.**

A flag (`--keep-claudecode`) disables a default protection (`unset_claudecode: true`). When the protection would have been active (CLAUDECODE is in env), the user receives no indication that their flag has had its intended effect AND that the consequence (nested-agent mode) is now in play. The canonical fix: emit a warning at the warning verbosity threshold when a protection-disable flag is detected AND the condition it would have protected against is confirmed present.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|------------|-------|---------|----------|
| H1 | `run_built_command()` has no warning check for keep_claudecode + CLAUDECODE | ✅ Root Cause | No conditional block checks `cli.keep_claudecode && env::var("CLAUDECODE").is_ok()` | `src/cli/mod.rs:224-246` |
| H2 | Warning is emitted somewhere else (e.g., builder or apply_env_vars) | DISPROVED | `build_claude_command()` line 126-129 and `apply_env_vars()` neither check CLAUDECODE env var | `src/cli/builder.rs:126-129`, `src/cli/parse.rs:351-352` |

## Evidence Table

| Location | What It Shows |
|----------|---------------|
| `module/claude_runner/src/cli/mod.rs:224-246` | `run_built_command()` — no warning check for keep_claudecode + CLAUDECODE present |
| `module/claude_runner/src/cli/builder.rs:126-129` | `if cli.keep_claudecode { builder.with_unset_claudecode(false) }` — silently disables protection |
| `module/claude_runner/src/cli/parse.rs:351-352` | `apply_env_vars()` — applies `CLR_KEEP_CLAUDECODE` to `parsed.keep_claudecode`; no CLAUDECODE check |
| `claude_runner_core/docs/failure_mode/003_claudecode_env_leak.md` | Documents the consequence: CLAUDECODE=1 in child silently changes behavior |

## History

| Date | Event | Note |
|------|-------|------|
| 2026-06-07 | filed | Source: code analysis of `run_built_command()` — no warning for protection-disable + active condition |
| 2026-06-07 | fixed | `src/cli/mod.rs:514` — `Fix(BUG-248)`: warn when `--keep-claudecode` set and `CLAUDECODE` present |
| 2026-06-07 | verified | `bug_reproducers_248_test.rs` passes; automated tests confirm warning emitted |
