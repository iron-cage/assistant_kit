# Invariant: Print-Mode Timeout Default

### Scope

- **Purpose**: Enforce a 1-hour safety watchdog for print-mode sessions started without an explicit `--timeout` flag.
- **Responsibility**: State the mandatory default timeout for `run_print_mode()`, the exemption for interactive mode, and the constant that encodes the threshold.
- **In Scope**: `run_print_mode()` default timeout (`DEFAULT_PRINT_TIMEOUT_SECS = 3600`), interactive mode exclusion (`run_interactive()` stays at `0`), env-var fallback path, explicit override behavior.
- **Out of Scope**: Timeout semantics for `isolated`/`refresh` (→ `005_isolated_subprocess_defaults.md`), exit code on timeout (→ `006_exit_codes.md`), parameter reference (→ `cli/param/036_timeout.md`).

### Invariant Statement

When no `--timeout` value is provided (neither via CLI nor `CLR_TIMEOUT` env var), the print-mode execution path (`run_print_mode()`) must apply a 1-hour watchdog. Interactive mode (`run_interactive()`) must remain unbounded when no explicit timeout is given.

| Execution path | Timeout when `cli.timeout` is `None` | Rationale |
|----------------|--------------------------------------|-----------|
| `run_print_mode()` | `DEFAULT_PRINT_TIMEOUT_SECS` (3600 s = 1 hour) | Print-mode sessions should not run indefinitely by default; unattended stuck sessions consume resources and may indicate a stall |
| `run_interactive()` | `0` (unlimited) | Interactive REPL sessions are user-attended; an arbitrary timeout would interrupt long manual work sessions |

**Constant:** `DEFAULT_PRINT_TIMEOUT_SECS: u32 = 3600`

This constant must be defined adjacent to `run_print_mode()` in `src/cli/execution.rs` and referenced by name — not inlined as a literal — to make the threshold visible in code review and grep output.

**Explicit override still wins:** When `cli.timeout` is `Some(n)`, both paths use `n` unchanged. `Some(0)` means unlimited (user explicitly opted out of the watchdog). The 3600-second default applies **only** to the `None` case.

### Enforcement Mechanism

In `run_print_mode()` (`src/cli/execution.rs`), the timeout is resolved as:

```rust
const DEFAULT_PRINT_TIMEOUT_SECS : u32 = 3600;

fn default_print_timeout() -> u32
{
  std::env::var( "_CLR_DEFAULT_TIMEOUT" )
    .ok()
    .and_then( | s | s.parse().ok() )
    .unwrap_or( DEFAULT_PRINT_TIMEOUT_SECS )
}

// … inside run_print_mode():
let timeout_secs = cli.timeout.unwrap_or( default_print_timeout() );
```

The `_CLR_DEFAULT_TIMEOUT` internal env var exists solely for test injection — it allows integration tests to verify the default-path kill mechanism without waiting 3600 seconds. The underscore prefix signals internal/test-only use; it is not documented in user-facing param docs and must not appear in `clr --help`.

In `run_interactive()` (same file), the timeout must remain:

```rust
let timeout_secs = cli.timeout.unwrap_or( 0 );
```

The asymmetry is intentional and load-bearing. The enforcement mechanism is a named constant (not a magic number) so the threshold is greppable and auditable.

### Violation Consequences

If `run_print_mode()` uses `unwrap_or(0)` instead of `unwrap_or(DEFAULT_PRINT_TIMEOUT_SECS)`:
- Unattended print-mode sessions can run for hours without bound (BUG-305 symptom: 79h sessions observed)
- Resource exhaustion on shared machines goes undetected by `clr`
- Users relying on the 1-hour safety net are silently exposed

If `run_interactive()` adopts the 1-hour default:
- Active user sessions are killed mid-conversation with no warning
- The timeout error message (`"Error: timeout after 3600s"`) appears in interactive context where it cannot be scripted around

### Features

| File | Relationship |
|------|--------------|
| [feature/001_runner_tool.md](../feature/001_runner_tool.md) | Defines the print-mode and interactive execution paths that this invariant governs |

**Not to be confused with:** this is clr's own *outer* watchdog — it kills the
entire `claude` subprocess unconditionally once `DEFAULT_PRINT_TIMEOUT_SECS`
(or an explicit `--timeout`/`CLR_TIMEOUT`) elapses, regardless of any
background work in flight. It is independent of the *inner* layer, claude's
own `CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS` (which clr sets to `0` — see
[cli/003_env_param.md § Env Param 10](../cli/003_env_param.md)), which governs
how long claude's own print-mode wind-down waits for backgrounded
subagents/workflows before *its* internal sweep logic runs. Disabling or
raising one does not affect the other — a long-running background agent can
still be killed by this outer watchdog even when the inner ceiling has been
neutralized.

### Sources

| File | Relationship |
|------|--------------|
| `../../src/cli/execution.rs` | `run_print_mode()` and `run_interactive()` — timeout resolution via `unwrap_or()` |

### Tests

| File | Relationship |
|------|--------------|
| `../../tests/timeout_test.rs` | `ec_timeout_default_constant_value`, `ec_timeout_default_no_fire`, `ec_timeout_default_activates_watchdog`, `ec_timeout_explicit_above_default`, `ec_timeout_unlimited_flag`, `ec_timeout_unlimited_env` |
| `../../tests/env_var_test.rs` | `ec_timeout_env_matches_default` |

### Provenance

| Source | Notes |
|--------|-------|
| TSK-227 | Verified task introducing `DEFAULT_PRINT_TIMEOUT_SECS` and print-mode watchdog default |
| BUG-305 | Root bug: `unwrap_or(0)` in `run_print_mode()` — no default print-mode timeout |
