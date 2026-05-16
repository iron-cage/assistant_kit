# Feature: Live Quota Monitor Mode

### Scope

- **Purpose**: Allow `.usage` to run as a continuous ambient monitor — repeatedly fetching quota for all accounts, clearing the screen, and re-rendering the table in-place — so the user sees a live, auto-refreshing dashboard without re-invoking the command.
- **Responsibility**: Documents the `live::`, `interval::`, and `jitter::` parameters, the staggered inter-account fetch algorithm (thunder-herd mitigation), the countdown footer format, the Ctrl-C exit path via `AtomicBool` signal handler, and the incompatibility guard with `format::json`.
- **In Scope**: `live::` mode gate; `interval::` (seconds between full refresh cycles); `jitter::` (maximum random variation on the outer cycle delay); staggered per-account delays (200–1500 ms pseudo-random); ANSI screen clear (`\x1B[2J\x1B[H`); countdown footer with UTC timestamp; `live::1 format::json` rejection; `interval < 30` rejection; `jitter > interval` rejection.
- **Out of Scope**: `refresh::` parameter (→ `017_token_refresh.md`); `run_isolated()` internals (→ `claude_runner_core/docs/feature/004_run_isolated.md`); table rendering internals (→ `009_token_usage.md`).

### Design

The `live::` parameter takes `0` (default, off) or `1` (on). When `0`, `.usage` behaves identically to the baseline — one fetch, one render, exit. When `live::1`, the command enters a continuous loop: fetch all accounts, render the table, display a countdown footer, wait, then repeat.

**Parameter summary:**

| Parameter | Default | Constraints | Purpose |
|-----------|---------|-------------|---------|
| `live::` | `0` | `0` or `1` | Enable continuous refresh mode |
| `interval::` | `30` | ≥ 30 (seconds) | Seconds between full refresh cycles |
| `jitter::` | `0` | 0 ≤ jitter ≤ interval | Max random addition to outer delay |

**Validation (before first fetch):**
1. `live::1` with `format::json` → exit 1 with `"live monitor mode is incompatible with format::json"`.
2. `interval::` value < 30 → exit 1 with `"interval must be >= 30"`.
3. `jitter::` value > `interval::` value → exit 1 with `"jitter must not exceed interval"`.

These checks run once at startup before any network call is made.

**Monitor loop algorithm:**

```
validate params (exit 1 on error)
signal_handler: AtomicBool QUIT = false
  on SIGINT: set QUIT = true

loop:
    print \x1B[2J\x1B[H   // clear screen, cursor to top-left
    for each account (sorted):
        delay = pseudo_random(200..=1500) milliseconds
        sleep(delay)
        fetch and collect result
    render table (same format as baseline .usage)
    compute next_at = now + interval + pseudo_random(0..=jitter)
    render countdown footer (see below)
    countdown from interval+jitter seconds:
        each second: rewrite footer line with updated countdown
        if QUIT: break outer loop
    if QUIT: break
```

**Staggered inter-account fetch (thunder-herd mitigation):**

Each account's HTTP fetch is preceded by a random sleep of 200–1500 ms. This prevents the scenario where many concurrent users all running `.usage live::1` with the same `interval::` send simultaneous bursts to Anthropic's API.

Pseudo-random source: `std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().subsec_nanos() % range` — sufficient quality for jitter/stagger; no `rand` crate dependency required.

**ANSI screen clear:**

`\x1B[2J\x1B[H` is written to stdout at the start of each cycle. This clears the entire terminal screen and moves the cursor to the top-left (row 1, col 1), causing the new table to render over the previous one with no scroll.

**Countdown footer format:**

A fixed-format footer is written below the table on each cycle. The footer line is rewritten in-place each second using carriage return (`\r`) to overwrite the previous content:

```
  Next update in M:SS (at HH:MM:SS UTC)  [Ctrl-C to exit]
```

Where:
- `M:SS` — remaining minutes and seconds until next fetch cycle begins (e.g., `0:29`, `1:02`)
- `HH:MM:SS UTC` — wall-clock time at which the next fetch will start (24-hour UTC)
- The line is right-padded to a fixed width (80 chars minimum) to erase any leftover characters from a previous longer line

**Signal handling (no `ctrlc` crate):**

A `static AtomicBool QUIT` is registered as a SIGINT handler via `unsafe { libc::signal(SIGINT, handler as libc::sighandler_t) }`. `libc` is a transitive dependency — no new crate is added. The countdown loop polls `QUIT.load(Ordering::Relaxed)` once per second; when set, the loop exits cleanly, the cursor is restored, and the process exits 0.

**Feature gate:**

The live monitor loop is compiled only under `#[cfg(feature = "enabled")]`. When `enabled` is absent, `live::1` is accepted as a parameter but no loop is entered — the command runs once (baseline behaviour). This matches the `fetch_oauth_usage` feature gate and prevents offline builds from registering a signal handler or depending on `libc` for process signals.

**Output format:**

In live mode, `format::json` is rejected before the first fetch (see Validation). `format::text` is the only valid format. The rendered table is identical to the baseline `.usage` text output; only the surrounding loop and footer are new.

**No behavioral change at default:** `live::0` introduces no overhead, no signal handler, and no screen clear. Existing tests are unaffected.

### Acceptance Criteria

- **AC-24**: `live::0` (default) produces exactly one fetch cycle and exits; behavior is unchanged from the baseline `.usage`.
- **AC-25**: `live::1 format::json` exits 1 before any fetch with `"live monitor mode is incompatible with format::json"`.
- **AC-26**: `live::1 interval::29` exits 1 before any fetch with `"interval must be >= 30"`.
- **AC-27**: `live::1 interval::60 jitter::70` exits 1 before any fetch with `"jitter must not exceed interval"`.
- **AC-28**: `live::1` (default `interval::30`) renders the table, displays a countdown footer (`Next update in M:SS (at HH:MM:SS UTC)  [Ctrl-C to exit]`), waits `interval` seconds (plus up to `jitter` seconds), then repeats the cycle.
- **AC-29**: Each per-account fetch in live mode is preceded by a random sleep of 200–1500 ms to stagger API calls (thunder-herd mitigation).
- **AC-30**: Ctrl-C (SIGINT) in live mode exits 0 cleanly; the partial countdown is not treated as an error.
- **AC-31**: `interval::` and `jitter::` have no effect when `live::0`; their values are not validated unless `live::1` is present.
- **AC-32**: `live::`, `interval::`, and `jitter::` appear in `.usage --help` output with their default values (`0`, `30`, `0`).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/usage.rs` | `live::` loop; stagger delay; countdown footer; SIGINT handler; param validation |
| source | `src/lib.rs` | `live::`, `interval::`, `jitter::` parameter registration via `register_commands()` |
| dep | `libc` | `signal()` for SIGINT handler (transitive dep, no new crate) |
| test | `tests/cli/usage_test.rs` | it21 (live loop), it22–it24 (guards), it25–it29 (boundary/edge), it30 (SIGINT clean-exit) |
| task | `task/claude_profile/138_usage_live_monitor.md` | Implementation task for this feature |
| doc | [009_token_usage.md](009_token_usage.md) | Baseline `.usage` algorithm and table format that this extends |
| doc | [017_token_refresh.md](017_token_refresh.md) | `refresh::` parameter — composable with live mode |
| doc | [cli/commands.md](../cli/commands.md#command--9-usage) | `.usage` CLI command specification |
| doc | [cli/params.md](../cli/params.md#parameter--20-live) | `live::`, `interval::`, `jitter::` parameter specifications |
