# Invariant: No Built-In Print-Mode Session Timeout

### Scope

- **Purpose**: Guarantee that a session started without an expressed `--timeout`/`CLR_TIMEOUT` is never killed by a built-in clr deadline.
- **Responsibility**: State the expressed-only watchdog contract for `run_print_mode()` and `run_interactive()`, the zero constant that encodes it, and the test hook that keeps the default path testable.
- **In Scope**: `run_print_mode()` default timeout (`DEFAULT_PRINT_TIMEOUT_SECS = 0`), interactive-mode resolution (`run_interactive()` — both TTY branches unlimited in production), `_CLR_DEFAULT_TIMEOUT` test hook, explicit override behavior.
- **Out of Scope**: Timeout semantics for `isolated`/`refresh` (→ `005_isolated_subprocess_defaults.md`), exit code on timeout (→ `006_exit_codes.md`), parameter reference (→ `cli/param/036_timeout.md`).

### Invariant Statement

When no `--timeout` value is provided (neither via CLI nor `CLR_TIMEOUT` env var), no execution path may arm a watchdog: sessions run unlimited. A timeout kill may occur **only** when the caller expressed one.

| Execution path | Timeout when `cli.timeout` is `None` | Rationale |
|----------------|--------------------------------------|-----------|
| `run_print_mode()` | `DEFAULT_PRINT_TIMEOUT_SECS` (0 = unlimited) | Long agentic sessions routinely exceed any fixed deadline; clr already neutralizes claude's inner wind-down ceiling (`CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS=0`) precisely to let background work run, and a built-in outer kill contradicted that (TSK-503) |
| `run_interactive()` | `0` (unlimited — TTY and non-TTY branches alike) | Interactive sessions are user-attended; the TTY/non-TTY split (BUG-425) survives only so the `_CLR_DEFAULT_TIMEOUT` test hook can arm the non-TTY branch while a genuine TTY stays exempt |

**Constant:** `DEFAULT_PRINT_TIMEOUT_SECS: u32 = 0`

This constant must be defined adjacent to `run_print_mode()` in `src/cli/execution.rs` and referenced by name — not inlined as a literal — so the contract is visible in code review and grep output, and so the `default_print_timeout()` resolution chain (and with it the test hook) stays intact.

**Expressed values win unchanged:** When `cli.timeout` is `Some(n)`, both paths use `n` — including `Some(0)`, which expresses unlimited explicitly. The distinction still matters even though default and expressed zero resolve to the same watchdog behavior: only *expressed* timeouts feed the gate-wait budget (BUG-445 — see `../cli/param/033_max_sessions.md`).

### Enforcement Mechanism

In `run_print_mode()` (`src/cli/execution.rs`), the timeout is resolved as:

```rust
const DEFAULT_PRINT_TIMEOUT_SECS : u32 = 0;

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

The `_CLR_DEFAULT_TIMEOUT` internal env var exists solely for test injection — it lets integration tests arm a short default-path watchdog and verify the kill/retry/journal machinery that production no longer triggers by default. The underscore prefix signals internal/test-only use; it is not documented in user-facing param docs and must not appear in `clr --help`.

In `run_interactive()` (same file), non-TTY stdin adopts the same `default_print_timeout()` resolution (BUG-425) while a genuine TTY resolves `unwrap_or( 0 )` directly. In production both branches yield 0; the split is load-bearing only for the test hook and must not be collapsed.

### Violation Consequences

If `run_print_mode()` re-introduces a nonzero built-in default:
- Long agentic sessions are killed mid-work at an arbitrary deadline — the exact failure TSK-503 removed (the former 3600 s default killed sessions whose background work was still progressing)
- The default invocation contradicts clr's own neutralization of the inner ceiling (`CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS=0`)

If the resolution chain bypasses `default_print_timeout()` (literal `0` inlined at the call site):
- The `_CLR_DEFAULT_TIMEOUT` hook goes dead and every hook-based kill-path test (`ec_timeout_default_kills`, retry, journal EC-8, session verification) silently loses the very mechanism it exercises

**Accepted tradeoff (recorded, not a violation):** a wedged non-TTY session now hangs indefinitely unless the caller expresses a timeout. Supervisors must express their own deadline — `watchdog.sh` health probes already do (`--timeout $HEALTH_TIMEOUT` plus `CLR_REMAINING_TIMEOUT_SECS`).

### Features

| File | Relationship |
|------|--------------|
| [feature/001_runner_tool.md](../feature/001_runner_tool.md) | Defines the print-mode and interactive execution paths that this invariant governs |

**Not to be confused with:** this governs clr's own *outer* watchdog — when
armed (by an expressed `--timeout`/`CLR_TIMEOUT`, or the test hook), it kills
the entire `claude` subprocess unconditionally once the deadline elapses,
regardless of any background work in flight. It is independent of the *inner*
layer, claude's own `CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS` (which clr sets to
`0` — see [cli/003_env_param.md § Env Param 10](../cli/003_env_param.md)),
which governs how long claude's own print-mode wind-down waits for
backgrounded subagents/workflows before *its* internal sweep logic runs.
Disabling or raising one does not affect the other — an expressed outer
timeout can still kill a long-running background agent even though the inner
ceiling is neutralized. Since TSK-503 the two layers finally agree by
default: neither imposes a deadline unless the caller asks for one.

### Sources

| File | Relationship |
|------|--------------|
| `../../src/cli/execution.rs` | `run_print_mode()` and `run_interactive()` — timeout resolution via `unwrap_or()` |

### Tests

| File | Relationship |
|------|--------------|
| `../../tests/timeout_test.rs` | `ec_timeout_default_constant_value`, `ec_timeout_default_no_fire`, `ec_timeout_default_unlimited`, `ec_timeout_explicit_large_value`, `ec_timeout_unlimited_flag`, `ec_timeout_unlimited_env`, `ec_timeout_default_kills` (hook-armed kill path) |
| `../../tests/env_var_test.rs` | `ec_timeout_env_hour_value_accepted` |

### Provenance

| Source | Notes |
|--------|-------|
| TSK-227 | Introduced `DEFAULT_PRINT_TIMEOUT_SECS` (then 3600) and the print-mode watchdog default |
| BUG-305 | Bug that motivated TSK-227: `unwrap_or(0)` in `run_print_mode()` — no default print-mode timeout |
| BUG-425 | Non-TTY interactive adoption of the print default; the TTY/non-TTY split it introduced survives for the test hook |
| TSK-503 | Retired the built-in default (`3600` → `0`): watchdog is expressed-only; the constant, resolution chain, and test hook remain |
